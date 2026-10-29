use crate::sys::{self, HCTX};
use crate::{Packet, Status};

/// A tablet control extension that reports through extension packets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Extension {
    /// ExpressKeys (`WTX_EXPKEYS2`).
    ExpressKeys,
    /// Touch strips (`WTX_TOUCHSTRIP`).
    TouchStrip,
    /// Touch rings (`WTX_TOUCHRING`).
    TouchRing,
}

impl Extension {
    /// The `WTX_*` tag.
    pub fn tag(self) -> u32 {
        match self {
            Self::ExpressKeys => sys::WTX_EXPKEYS2,
            Self::TouchStrip => sys::WTX_TOUCHSTRIP,
            Self::TouchRing => sys::WTX_TOUCHRING,
        }
    }
}

/// The fields every extension packet starts with.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExtensionBase {
    /// The context that reported the packet.
    pub context: HCTX,
    /// Status bits.
    pub status: Status,
    /// When the packet was generated, in milliseconds.
    pub time: u32,
    /// The packet's serial number.
    pub serial_number: u32,
}

/// The state of one ExpressKey.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExpressKey {
    /// The tablet index.
    pub tablet: u8,
    /// The key index.
    pub control: u8,
    /// Where the key is on the tablet (`TABLET_LOC_*`).
    pub location: u8,
    reserved: u8,
    /// `1` while pressed, `0` once released.
    pub state: u32,
}

impl ExpressKey {
    /// Whether the key is pressed.
    pub fn is_pressed(&self) -> bool {
        self.state != 0
    }
}

/// The state of one touch ring or touch strip.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Slider {
    /// The tablet index.
    pub tablet: u8,
    /// The ring or strip index.
    pub control: u8,
    /// The active mode of the control.
    pub mode: u8,
    reserved: u8,
    /// The finger position, within the range the control reports.
    pub position: u32,
}

const _: () = assert!(size_of::<ExtensionBase>() == size_of::<sys::EXTENSIONBASE>());
const _: () = assert!(size_of::<ExpressKey>() == size_of::<sys::EXPKEYSDATA>());
const _: () = assert!(size_of::<Slider>() == size_of::<sys::SLIDERDATA>());

/// An extension packet layout, read with [`Context::packet_ext`](crate::Context::packet_ext).
///
/// # Safety
/// The type must have the layout drivers write for exactly the extensions in
/// `EXTENSIONS`. Implement it with [`packet_ext!`](crate::packet_ext).
pub unsafe trait PacketExt: Copy + Default + 'static {
    /// The extensions the packet holds, in `PKTDEF.H` order.
    const EXTENSIONS: &'static [Extension];
}

/// Declares an extension packet type. List the extensions it holds, in this order:
/// `express_keys`, `touch_strip`, `touch_ring`. Open the context with the same extensions
/// through [`ContextOptions::extensions`](crate::ContextOptions::extensions).
///
/// ```ignore
/// wintab::packet_ext! {
///     pub struct Controls { express_keys, touch_ring }
/// }
/// ```
#[macro_export]
macro_rules! packet_ext {
    ($(#[$attr:meta])* $vis:vis struct $name:ident { $($field:ident),* $(,)? }) => {
        $(#[$attr])*
        // Unlike `packet!` types, drivers keep MSVC's padding after `base`.
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        $vis struct $name {
            pub base: $crate::ExtensionBase,
            $(pub $field: $crate::packet_ext!(@type $field),)*
        }

        // SAFETY: the fields follow `PKTDEF.H` order (checked below) with its types.
        unsafe impl $crate::PacketExt for $name {
            const EXTENSIONS: &'static [$crate::Extension] =
                &[$($crate::packet_ext!(@extension $field)),*];
        }

        const _: () = $crate::sys::__assert_order(&[$($crate::packet_ext!(@order $field)),*]);
    };

    (@type express_keys) => { $crate::ExpressKey };
    (@type touch_strip) => { $crate::Slider };
    (@type touch_ring) => { $crate::Slider };

    (@extension express_keys) => { $crate::Extension::ExpressKeys };
    (@extension touch_strip) => { $crate::Extension::TouchStrip };
    (@extension touch_ring) => { $crate::Extension::TouchRing };

    (@order express_keys) => { 0 };
    (@order touch_strip) => { 1 };
    (@order touch_ring) => { 2 };
}

/// Where a tablet control is (`TABLET_LOC_*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Location {
    /// The left edge.
    Left,
    /// The right edge.
    Right,
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// On the pen or puck.
    Transducer,
    /// A location this crate does not know.
    Other(u32),
}

impl Location {
    pub(crate) fn new(value: u32) -> Self {
        match value {
            sys::TABLET_LOC_LEFT => Self::Left,
            sys::TABLET_LOC_RIGHT => Self::Right,
            sys::TABLET_LOC_TOP => Self::Top,
            sys::TABLET_LOC_BOTTOM => Self::Bottom,
            sys::TABLET_LOC_TRANSDUCER => Self::Transducer,
            other => Self::Other(other),
        }
    }
}

/// One function of a tablet control that a [`ControlOverride`] took over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Control {
    /// The tablet index.
    pub tablet: u8,
    /// The control index, as in [`ExpressKey::control`] and [`Slider::control`].
    pub control: u8,
    /// The function index within the control, such as a touch ring mode.
    pub function: u8,
    /// Where the control is.
    pub location: Option<Location>,
    /// The smallest value the control reports.
    pub min: Option<u32>,
    /// The largest value the control reports.
    pub max: Option<u32>,
}

/// Tablet controls taken over from the driver, so that their presses arrive as extension
/// packets instead of running the user's driver settings. Released on drop.
pub struct ControlOverride<'a, P: Packet> {
    pub(crate) context: &'a crate::Context<P>,
    pub(crate) extension: Extension,
    pub(crate) controls: Vec<Control>,
}

impl<P: Packet> ControlOverride<'_, P> {
    /// The functions taken over.
    pub fn controls(&self) -> &[Control] {
        &self.controls
    }

    /// Sets the name the tablet shows for an overridden control, on tablets with displays.
    pub fn set_name(&self, control: &Control, name: &str) -> crate::Result<()> {
        let mut data = name.as_bytes().to_vec();
        data.push(0);
        self.context.set_property(
            self.extension,
            control,
            sys::TABLET_PROPERTY_OVERRIDE_NAME,
            &data,
        )
    }
}

impl<P: Packet> Drop for ControlOverride<'_, P> {
    fn drop(&mut self) {
        for control in &self.controls {
            self.context
                .set_property(
                    self.extension,
                    control,
                    sys::TABLET_PROPERTY_OVERRIDE,
                    &0u32.to_ne_bytes(),
                )
                .ok();
        }
    }
}
