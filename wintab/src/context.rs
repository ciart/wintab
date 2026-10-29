use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::RangeInclusive;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::core::BOOL;

use crate::extension::{Control, ControlOverride, Extension, Location, PacketExt};
use crate::sys::{self, HCTX, LOGCONTEXTW};
use crate::{Error, Message, Packet, Result, Wintab};

/// Where a context's margin lies, inside which the cursor keeps reporting at the edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Margin {
    /// No margin.
    #[default]
    None,
    /// Around the outside of the input area.
    Outside,
    /// Along the inside of the input area.
    Inside,
}

/// How [`Wintab::open`] sets up a context.
#[derive(Clone, Debug)]
pub struct ContextOptions {
    system: bool,
    enabled: bool,
    queue_size: Option<u32>,
    name: Option<String>,
    margin: Margin,
    cursor_messages: bool,
    button_masks: Option<(u32, u32)>,
    extensions: Vec<Extension>,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            system: true,
            enabled: true,
            queue_size: Some(128),
            name: None,
            margin: Margin::None,
            cursor_messages: false,
            button_masks: None,
            extensions: Vec::new(),
        }
    }
}

impl ContextOptions {
    /// Starts from the defaults: a system context, enabled, with room for 128 packets.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens a system context, which also moves the system cursor, instead of a digitizing
    /// context, which only reports packets and keeps the cursor from following the pen while
    /// it is on top. On by default.
    pub fn system(mut self, system: bool) -> Self {
        self.system = system;
        self
    }

    /// Whether the context starts enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The packet queue length to ask for. The driver drops packets silently once the queue
    /// is full; when it refuses the size, smaller sizes are tried. `None` keeps the driver's
    /// default.
    pub fn queue_size(mut self, packets: Option<u32>) -> Self {
        self.queue_size = packets;
        self
    }

    /// The name the driver shows for the context, at most 39 characters.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The context's margin.
    pub fn margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    /// Sends [`Message::CursorChange`] when a different cursor type comes into use.
    pub fn cursor_messages(mut self, enabled: bool) -> Self {
        self.cursor_messages = enabled;
        self
    }

    /// The logical buttons whose presses and releases generate packets. All by default.
    pub fn button_masks(mut self, down: u32, up: u32) -> Self {
        self.button_masks = Some((down, up));
        self
    }

    /// Reports these extensions in extension packets. See [`Context::packet_ext`].
    pub fn extensions(mut self, extensions: &[Extension]) -> Self {
        self.extensions = extensions.to_vec();
        self.extensions.sort();
        self.extensions.dedup();
        self
    }
}

/// An open Wintab context reporting packets of type `P`. Closed on drop.
///
/// A context belongs to the window it was opened for and to that window's thread.
pub struct Context<P: Packet> {
    wintab: Wintab,
    handle: HCTX,
    logical: Cell<LOGCONTEXTW>,
    extensions: Vec<Extension>,
    _packet: PhantomData<P>,
}

impl<P: Packet> Context<P> {
    pub(crate) fn open(wintab: Wintab, hwnd: HWND, options: &ContextOptions) -> Result<Self> {
        let category = if options.system {
            sys::WTI_DEFSYSCTX
        } else {
            sys::WTI_DEFCONTEXT
        };
        let mut logical: LOGCONTEXTW = wintab.info(category, 0).ok_or(Error::Unavailable)?;
        logical.lcOptions |= sys::CXO_MESSAGES;
        logical.lcOptions &= !(sys::CXO_SYSTEM | sys::CXO_MARGIN | sys::CXO_MGNINSIDE);
        if options.system {
            logical.lcOptions |= sys::CXO_SYSTEM;
        }
        logical.lcOptions |= match options.margin {
            Margin::None => 0,
            Margin::Outside => sys::CXO_MARGIN,
            Margin::Inside => sys::CXO_MARGIN | sys::CXO_MGNINSIDE,
        };
        if options.cursor_messages {
            logical.lcOptions |= sys::CXO_CSRMESSAGES;
        }
        if let Some(name) = &options.name {
            logical.lcName = [0; sys::LCNAMELEN];
            for (slot, unit) in logical.lcName[..sys::LCNAMELEN - 1]
                .iter_mut()
                .zip(name.encode_utf16())
            {
                *slot = unit;
            }
        }
        if let Some((down, up)) = options.button_masks {
            logical.lcBtnDnMask = down;
            logical.lcBtnUpMask = up;
        }
        logical.lcPktData = P::DATA;
        logical.lcPktMode = P::MODE;
        logical.lcMoveMask = P::DATA;
        for &extension in &options.extensions {
            logical.lcPktData |= wintab
                .extension_mask(extension.tag())
                .ok_or(Error::UnsupportedExtension(extension))?;
        }
        logical.lcOutOrgX = logical.lcInOrgX;
        logical.lcOutOrgY = logical.lcInOrgY;
        logical.lcOutExtX = logical.lcInExtX;
        logical.lcOutExtY = logical.lcInExtY;

        let lib = wintab.lib();
        let handle =
            unsafe { (lib.WTOpenW.unwrap())(hwnd, &mut logical, BOOL::from(options.enabled)) };
        if handle.0.is_null() {
            return Err(Error::OpenFailed);
        }
        let context = Self {
            wintab,
            handle,
            logical: Cell::new(logical),
            extensions: options.extensions.clone(),
            _packet: PhantomData,
        };
        if let Some(mut packets) = options.queue_size {
            while packets > 0 && !context.set_queue_size(packets) {
                packets /= 2;
            }
        }
        context.refresh();
        Ok(context)
    }

    fn set_queue_size(&self, packets: u32) -> bool {
        let lib = self.wintab.lib();
        unsafe { (lib.WTQueueSizeSet.unwrap())(self.handle, packets as i32) }.as_bool()
    }

    /// The raw handle, for calls this crate does not wrap.
    pub fn handle(&self) -> HCTX {
        self.handle
    }

    /// The context's current settings, as the driver reports them.
    pub fn logical(&self) -> LOGCONTEXTW {
        self.logical.get()
    }

    /// Reads the settings again. Call after [`Message::Update`] or [`Message::InfoChange`],
    /// since the driver may change the context's mapping.
    pub fn refresh(&self) {
        let mut logical = self.logical.get();
        let lib = self.wintab.lib();
        if unsafe { (lib.WTGetW.unwrap())(self.handle, &mut logical) }.as_bool() {
            self.logical.set(logical);
        }
    }

    /// Changes the context's settings, such as its input area or screen mapping. The packet
    /// masks always stay those of `P`.
    pub fn update(&self, change: impl FnOnce(&mut LOGCONTEXTW)) -> Result<()> {
        let lib = self.wintab.lib();
        let set = lib.WTSetW.ok_or(Error::MissingFunction("WTSetW"))?;
        let mut logical = self.logical.get();
        let (data, mode, moves) = (logical.lcPktData, logical.lcPktMode, logical.lcMoveMask);
        change(&mut logical);
        logical.lcPktData = data;
        logical.lcPktMode = mode;
        logical.lcMoveMask = moves;
        if !unsafe { set(self.handle, &mut logical) }.as_bool() {
            return Err(Error::CallFailed("WTSetW"));
        }
        self.refresh();
        Ok(())
    }

    /// Decodes a window message addressed to this context. `None` for any other message.
    pub fn message(&self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<Message> {
        let offset = msg.checked_sub(self.logical.get().lcMsgBase)?;
        if offset > sys::WT_MAXOFFSET {
            return None;
        }
        let from_lparam = lparam.0 as *mut _ == self.handle.0;
        let from_wparam = wparam.0 as *mut _ == self.handle.0;
        let low = lparam.0 as u32 as u16;
        let high = (lparam.0 as u32 >> 16) as u16;
        let status = lparam.0 as u32;
        match offset + sys::WT_DEFBASE {
            sys::WT_PACKET if from_lparam => Some(Message::Packet {
                serial: wparam.0 as u32,
            }),
            sys::WT_CSRCHANGE if from_lparam => Some(Message::CursorChange {
                serial: wparam.0 as u32,
            }),
            // The driver's `lParam` here is not the handle `WTOpenW` returned, so it can't be matched.
            sys::WT_PACKETEXT if !self.extensions.is_empty() => Some(Message::PacketExt {
                serial: wparam.0 as u32,
                source: HCTX(lparam.0 as _),
            }),
            sys::WT_PROXIMITY if from_wparam => Some(Message::Proximity {
                entering: low != 0,
                hardware: high != 0,
            }),
            sys::WT_CTXOPEN if from_wparam => Some(Message::Opened { status }),
            sys::WT_CTXCLOSE if from_wparam => Some(Message::Closed { status }),
            sys::WT_CTXOVERLAP if from_wparam => Some(Message::Overlap { status }),
            sys::WT_CTXUPDATE if from_wparam => Some(Message::Update { status }),
            sys::WT_INFOCHANGE => Some(Message::InfoChange {
                category: low,
                index: high,
            }),
            _ => None,
        }
    }

    /// Moves every queued packet, oldest first, to the end of `packets`. Returns how many.
    pub fn drain(&self, packets: &mut Vec<P>) -> usize {
        const CHUNK: usize = 64;
        let lib = self.wintab.lib();
        let start = packets.len();
        loop {
            packets.reserve(CHUNK);
            let spare = packets.spare_capacity_mut();
            // SAFETY: `P` has the layout the driver writes, and `spare` has room for `CHUNK`.
            let read = unsafe {
                (lib.WTPacketsGet.unwrap())(self.handle, CHUNK as i32, spare.as_mut_ptr() as _)
            }
            .clamp(0, CHUNK as i32) as usize;
            unsafe { packets.set_len(packets.len() + read) };
            if read < CHUNK {
                return packets.len() - start;
            }
        }
    }

    /// Returns the packet with the given serial number and discards every older one.
    pub fn packet(&self, serial: u32) -> Option<P> {
        let mut packet = P::default();
        let lib = self.wintab.lib();
        unsafe { (lib.WTPacket.unwrap())(self.handle, serial, &mut packet as *mut P as _) }
            .as_bool()
            .then_some(packet)
    }

    /// Reads the extension packet of a [`Message::PacketExt`]. `None` when `E` does not
    /// hold exactly the extensions the context was opened with, or when the packet belongs
    /// to another context.
    pub fn packet_ext<E: PacketExt>(&self, source: HCTX, serial: u32) -> Option<E> {
        #[repr(C, align(8))]
        struct Buffer([u8; 256]);
        const { assert!(size_of::<E>() <= size_of::<Buffer>() && align_of::<E>() <= 8) };

        if E::EXTENSIONS != self.extensions.as_slice() {
            return None;
        }
        let mut buffer = Buffer([0; 256]);
        let lib = self.wintab.lib();
        if !unsafe { (lib.WTPacket.unwrap())(source, serial, buffer.0.as_mut_ptr() as _) }
            .as_bool()
        {
            return None;
        }
        let base = unsafe { std::ptr::read(buffer.0.as_ptr() as *const sys::EXTENSIONBASE) };
        (base.nContext.0 == self.handle.0)
            .then(|| unsafe { std::ptr::read(buffer.0.as_ptr() as *const E) })
    }

    /// The length of the packet queue.
    pub fn queue_size(&self) -> Result<u32> {
        let get = self
            .wintab
            .lib()
            .WTQueueSizeGet
            .ok_or(Error::MissingFunction("WTQueueSizeGet"))?;
        Ok(unsafe { get(self.handle) }.max(0) as u32)
    }

    /// The serial numbers of the oldest and newest queued packets. `None` when the queue is
    /// empty.
    pub fn queued(&self) -> Result<Option<RangeInclusive<u32>>> {
        let get = self
            .wintab
            .lib()
            .WTQueuePacketsEx
            .ok_or(Error::MissingFunction("WTQueuePacketsEx"))?;
        let (mut oldest, mut newest) = (0, 0);
        Ok(unsafe { get(self.handle, &mut oldest, &mut newest) }
            .as_bool()
            .then_some(oldest..=newest))
    }

    /// Enables or disables packet reporting.
    pub fn enable(&self, enabled: bool) -> Result<()> {
        let lib = self.wintab.lib();
        unsafe { (lib.WTEnable.unwrap())(self.handle, BOOL::from(enabled)) }
            .as_bool()
            .then_some(())
            .ok_or(Error::CallFailed("WTEnable"))
    }

    /// Moves the context to the top or bottom of the overlap order. Call with `true` when
    /// the window is activated, so that it receives packets before other applications.
    pub fn overlap(&self, top: bool) -> Result<()> {
        let lib = self.wintab.lib();
        unsafe { (lib.WTOverlap.unwrap())(self.handle, BOOL::from(top)) }
            .as_bool()
            .then_some(())
            .ok_or(Error::CallFailed("WTOverlap"))
    }

    /// Maps packet coordinates to virtual-screen pixels through the screen area the driver
    /// assigned to the context. Packet y grows upward, screen y downward.
    pub fn to_screen(&self, x: i32, y: i32) -> (f64, f64) {
        let logical = self.logical.get();
        let ratio = |value: i32, origin: i32, extent: i32| {
            if extent == 0 {
                0.
            } else {
                (value - origin) as f64 / extent as f64
            }
        };
        let u = ratio(x, logical.lcOutOrgX, logical.lcOutExtX);
        let v = ratio(y, logical.lcOutOrgY, logical.lcOutExtY);
        (
            logical.lcSysOrgX as f64 + u * logical.lcSysExtX as f64,
            logical.lcSysOrgY as f64 + (1. - v) * logical.lcSysExtY as f64,
        )
    }

    /// Takes over every available function of `extension` on every tablet, so that its
    /// controls report extension packets instead of running the driver's actions.
    pub fn override_controls(&self, extension: Extension) -> Result<ControlOverride<'_, P>> {
        let mut controls = Vec::new();
        for tablet in 0..16u8 {
            let mut probe = LOGCONTEXTW::default();
            let category = sys::WTI_DDCTXS + tablet as u32;
            if unsafe { self.wintab.info_raw(category, 0, &mut probe as *mut _ as _) } == 0 {
                break;
            }
            let at = |control, function| Control {
                tablet,
                control,
                function,
                location: None,
                min: None,
                max: None,
            };
            let count = self
                .property(extension, &at(0, 0), sys::TABLET_PROPERTY_CONTROLCOUNT)?
                .unwrap_or(0);
            for control in 0..count.min(u8::MAX as u32) as u8 {
                let functions = self
                    .property(extension, &at(control, 0), sys::TABLET_PROPERTY_FUNCCOUNT)?
                    .unwrap_or(0);
                for function in 0..functions.min(u8::MAX as u32) as u8 {
                    let mut entry = at(control, function);
                    if self.property(extension, &entry, sys::TABLET_PROPERTY_AVAILABLE)? != Some(1)
                    {
                        continue;
                    }
                    if self
                        .set_property(
                            extension,
                            &entry,
                            sys::TABLET_PROPERTY_OVERRIDE,
                            &1u32.to_ne_bytes(),
                        )
                        .is_err()
                    {
                        continue;
                    }
                    entry.location = self
                        .property(extension, &entry, sys::TABLET_PROPERTY_LOCATION)?
                        .map(Location::new);
                    entry.min = self.property(extension, &entry, sys::TABLET_PROPERTY_MIN)?;
                    entry.max = self.property(extension, &entry, sys::TABLET_PROPERTY_MAX)?;
                    controls.push(entry);
                }
            }
        }
        Ok(ControlOverride {
            context: self,
            extension,
            controls,
        })
    }

    fn property(&self, extension: Extension, control: &Control, id: u16) -> Result<Option<u32>> {
        let get = self
            .wintab
            .lib()
            .WTExtGet
            .ok_or(Error::MissingFunction("WTExtGet"))?;
        let mut buffer = PropertyBuffer::new(control, id, &0u32.to_ne_bytes());
        Ok(
            unsafe { get(self.handle, extension.tag(), buffer.as_mut_ptr()) }
                .as_bool()
                .then(|| buffer.u32()),
        )
    }

    pub(crate) fn set_property(
        &self,
        extension: Extension,
        control: &Control,
        id: u16,
        data: &[u8],
    ) -> Result<()> {
        let set = self
            .wintab
            .lib()
            .WTExtSet
            .ok_or(Error::MissingFunction("WTExtSet"))?;
        let mut buffer = PropertyBuffer::new(control, id, data);
        unsafe { set(self.handle, extension.tag(), buffer.as_mut_ptr()) }
            .as_bool()
            .then_some(())
            .ok_or(Error::CallFailed("WTExtSet"))
    }
}

impl<P: Packet> Drop for Context<P> {
    fn drop(&mut self) {
        let lib = self.wintab.lib();
        let _ = unsafe { (lib.WTClose.unwrap())(self.handle) };
    }
}

#[repr(C, align(4))]
struct PropertyBuffer([u8; 12 + 64]);

impl PropertyBuffer {
    const DATA: usize = 12;

    fn new(control: &Control, id: u16, data: &[u8]) -> Self {
        let mut buffer = Self([0; 12 + 64]);
        let len = data.len().min(buffer.0.len() - Self::DATA);
        buffer.0[1] = control.tablet;
        buffer.0[2] = control.control;
        buffer.0[3] = control.function;
        buffer.0[4..6].copy_from_slice(&id.to_ne_bytes());
        buffer.0[8..12].copy_from_slice(&(len as u32).to_ne_bytes());
        buffer.0[Self::DATA..Self::DATA + len].copy_from_slice(&data[..len]);
        buffer
    }

    fn as_mut_ptr(&mut self) -> *mut std::ffi::c_void {
        self.0.as_mut_ptr() as _
    }

    fn u32(&self) -> u32 {
        u32::from_ne_bytes(self.0[Self::DATA..Self::DATA + 4].try_into().unwrap())
    }
}

const _: () = assert!(std::mem::offset_of!(sys::EXTPROPERTY, data) == PropertyBuffer::DATA);
