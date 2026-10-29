//! Safe wrapper for the Wintab 1.4 tablet API.
//!
//! [`Wintab::load`] loads the driver's `wintab32.dll`. [`Wintab::open`] opens a [`Context`]
//! whose packet layout is a type declared with [`packet!`], so the packet mask the driver
//! fills and the struct the application reads cannot disagree. Contexts deliver window
//! messages that [`Context::message`] decodes; the application keeps its own message loop.
//!
//! The safe API trusts the installed tablet driver, the only source of `wintab32.dll`, to
//! behave as the Wintab specification describes.
//!
//! ```ignore
//! wintab::packet! {
//!     pub struct Sample { status, buttons, x, y, normal_pressure, orientation }
//! }
//!
//! let wintab = wintab::Wintab::load()?;
//! let context = wintab.open::<Sample>(hwnd, &wintab::ContextOptions::default())?;
//! // In the window procedure:
//! if let Some(wintab::Message::Packet { serial }) = context.message(msg, wparam, lparam) {
//!     let mut packets = Vec::new();
//!     context.drain(&mut packets);
//! }
//! ```
#![cfg(target_os = "windows")]

mod context;
mod error;
mod extension;
mod info;
mod message;
mod packet;

pub use context::{Context, ContextOptions, Margin};
pub use error::{Error, Result};
pub use extension::{
    Control, ControlOverride, ExpressKey, Extension, ExtensionBase, Location, PacketExt, Slider,
};
pub use info::{Axis, Cursor, CursorCapabilities, Device, Interface, Units, Version};
pub use message::Message;
pub use packet::{ButtonAction, ButtonChange, Buttons, Orientation, Packet, Rotation, Status};
pub use wintab_sys as sys;

use std::ffi::c_void;
use std::rc::Rc;

use wintab_sys::Wintab32;

/// A loaded `wintab32.dll`. Cheap to clone; the library stays loaded while any clone or
/// [`Context`] is alive.
#[derive(Clone)]
pub struct Wintab {
    lib: Rc<Wintab32>,
}

impl Wintab {
    /// Loads `wintab32.dll` from the system directory and checks that the functions this
    /// crate calls are exported.
    pub fn load() -> Result<Self> {
        // SAFETY: only the system directory is searched, and the installed tablet driver is
        // trusted like any other system library.
        let lib = unsafe { Wintab32::load() }.map_err(Error::NotInstalled)?;
        for (name, present) in [
            ("WTInfoW", lib.WTInfoW.is_some()),
            ("WTOpenW", lib.WTOpenW.is_some()),
            ("WTGetW", lib.WTGetW.is_some()),
            ("WTClose", lib.WTClose.is_some()),
            ("WTPacket", lib.WTPacket.is_some()),
            ("WTPacketsGet", lib.WTPacketsGet.is_some()),
            ("WTEnable", lib.WTEnable.is_some()),
            ("WTOverlap", lib.WTOverlap.is_some()),
            ("WTQueueSizeSet", lib.WTQueueSizeSet.is_some()),
        ] {
            if !present {
                return Err(Error::MissingFunction(name));
            }
        }
        let wintab = Self { lib: Rc::new(lib) };
        if unsafe { wintab.info_raw(0, 0, std::ptr::null_mut()) } == 0 {
            return Err(Error::Unavailable);
        }
        Ok(wintab)
    }

    pub(crate) fn lib(&self) -> &Wintab32 {
        &self.lib
    }

    /// # Safety
    /// `out` must be null or point to enough writable memory for the item.
    pub(crate) unsafe fn info_raw(&self, category: u32, index: u32, out: *mut c_void) -> u32 {
        unsafe { (self.lib.WTInfoW.unwrap())(category, index, out) }
    }

    pub(crate) fn info<T: Copy + Default>(&self, category: u32, index: u32) -> Option<T> {
        let needed = unsafe { self.info_raw(category, index, std::ptr::null_mut()) } as usize;
        if needed == 0 || needed > size_of::<T>() {
            return None;
        }
        let mut value = T::default();
        unsafe { self.info_raw(category, index, &mut value as *mut T as _) };
        Some(value)
    }

    pub(crate) fn info_strings(&self, category: u32, index: u32) -> Vec<String> {
        let bytes = unsafe { self.info_raw(category, index, std::ptr::null_mut()) } as usize;
        let mut buffer = vec![0u16; bytes.div_ceil(2)];
        if bytes > 0 {
            unsafe { self.info_raw(category, index, buffer.as_mut_ptr() as _) };
        }
        buffer
            .split(|&c| c == 0)
            .take_while(|name| !name.is_empty())
            .map(String::from_utf16_lossy)
            .collect()
    }

    pub(crate) fn info_string(&self, category: u32, index: u32) -> Option<String> {
        let bytes = unsafe { self.info_raw(category, index, std::ptr::null_mut()) } as usize;
        if bytes == 0 {
            return None;
        }
        let mut buffer = vec![0u16; bytes.div_ceil(2)];
        unsafe { self.info_raw(category, index, buffer.as_mut_ptr() as _) };
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Some(String::from_utf16_lossy(&buffer[..len]))
    }

    /// Identification of the driver and the specification it implements.
    pub fn interface(&self) -> Interface {
        info::interface(self)
    }

    /// The tablet with the given index, below [`Interface::devices`].
    pub fn device(&self, index: u32) -> Option<Device> {
        info::device(self, index)
    }

    /// The cursor type with the given index, as reported in a packet's `cursor` field.
    pub fn cursor(&self, index: u32) -> Option<Cursor> {
        info::cursor(self, index)
    }

    /// The `lcPktData` bit of the extension with the given `WTX_*` tag, when the driver
    /// supports it.
    pub fn extension_mask(&self, tag: u32) -> Option<sys::WTPKT> {
        (0..)
            .map_while(|index| {
                self.info::<u32>(sys::WTI_EXTENSIONS + index, sys::EXT_TAG)
                    .map(|found| (index, found))
            })
            .find(|&(_, found)| found == tag)
            .and_then(|(index, _)| self.info(sys::WTI_EXTENSIONS + index, sys::EXT_MASK))
    }

    /// Opens a context that reports packets of type `P` to `hwnd`.
    pub fn open<P: Packet>(
        &self,
        hwnd: windows::Win32::Foundation::HWND,
        options: &ContextOptions,
    ) -> Result<Context<P>> {
        Context::open(self.clone(), hwnd, options)
    }
}
