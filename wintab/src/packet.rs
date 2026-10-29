use crate::sys::WTPKT;

/// A packet layout that a [`Context`](crate::Context) can report.
///
/// # Safety
/// The type must have exactly the layout the driver writes for `DATA` and `MODE`.
/// Implement it with [`packet!`](crate::packet) rather than by hand.
pub unsafe trait Packet: Copy + Default + 'static {
    /// The `lcPktData` mask.
    const DATA: WTPKT;
    /// The `lcPktMode` mask.
    const MODE: WTPKT;
}

/// A packet's `status` field.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Status(pub u32);

impl Status {
    /// The cursor is outside the context.
    pub fn out_of_context(self) -> bool {
        self.0 & crate::sys::TPS_PROXIMITY != 0
    }

    /// The packet queue overflowed and packets were lost.
    pub fn queue_overflowed(self) -> bool {
        self.0 & crate::sys::TPS_QUEUE_ERR != 0
    }

    /// The cursor is in the context's margin.
    pub fn in_margin(self) -> bool {
        self.0 & crate::sys::TPS_MARGIN != 0
    }

    /// The cursor is outside the context, which keeps reporting until the buttons are
    /// released.
    pub fn grabbed(self) -> bool {
        self.0 & crate::sys::TPS_GRAB != 0
    }

    /// The cursor is turned over, such as a pen's eraser end.
    pub fn inverted(self) -> bool {
        self.0 & crate::sys::TPS_INVERT != 0
    }
}

/// A packet's `buttons` field in absolute mode: the logical buttons held.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Buttons(pub u32);

impl Buttons {
    /// Whether the logical button is held.
    pub fn is_pressed(self, button: u8) -> bool {
        button < 32 && self.0 & (1 << button) != 0
    }
}

/// A packet's `buttons` field in relative mode: the button that changed, if any.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ButtonChange(pub u32);

/// How a button changed in a [`ButtonChange`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    /// The button was pressed.
    Pressed,
    /// The button was released.
    Released,
}

impl ButtonChange {
    /// The logical button that changed.
    pub fn button(self) -> u16 {
        (self.0 >> 16) as u16
    }

    /// How it changed, or `None` when no button changed.
    pub fn action(self) -> Option<ButtonAction> {
        match self.0 & 0xFFFF {
            crate::sys::TBN_DOWN => Some(ButtonAction::Pressed),
            crate::sys::TBN_UP => Some(ButtonAction::Released),
            _ => None,
        }
    }
}

/// Cursor orientation in a packet's `orientation` field. Angles are in the units of the
/// device's orientation [`Axis`](crate::Axis), usually tenths of a degree.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Orientation {
    /// Clockwise rotation of the cursor about the tablet's z axis.
    pub azimuth: i32,
    /// Angle between the cursor and the tablet plane; negative when the cursor points away.
    pub altitude: i32,
    /// Clockwise rotation of the cursor about its own axis.
    pub twist: i32,
}

/// Cursor rotation in a packet's `rotation` field.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rotation {
    /// Pitch of the cursor.
    pub pitch: i32,
    /// Roll of the cursor.
    pub roll: i32,
    /// Yaw of the cursor.
    pub yaw: i32,
}

const _: () = assert!(size_of::<Orientation>() == size_of::<crate::sys::ORIENTATION>());
const _: () = assert!(size_of::<Rotation>() == size_of::<crate::sys::ROTATION>());

/// Declares a packet type for [`Context`](crate::Context).
///
/// List the fields to report, in this order: `context`, `status`, `time`, `changed`,
/// `serial_number`, `cursor`, `buttons`, `x`, `y`, `z`, `normal_pressure`,
/// `tangent_pressure`, `orientation`, `rotation`. Append `(relative)` to report a field in
/// relative mode; relative pressures are `i32` instead of `u32`, and relative `buttons` are a
/// [`ButtonChange`] instead of [`Buttons`].
///
/// ```ignore
/// wintab::packet! {
///     pub struct Sample { status, buttons, x, y, normal_pressure, orientation }
/// }
/// ```
#[macro_export]
macro_rules! packet {
    ($(#[$attr:meta])* $vis:vis struct $name:ident { $($field:ident $(($mode:ident))?),* $(,)? }) => {
        $(#[$attr])*
        // Drivers pack packets without the tail padding MSVC adds after a 64-bit `context`;
        // `WTPacketsGet` arrays misalign if this changes.
        #[repr(C, packed(4))]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        $vis struct $name {
            $(pub $field: $crate::packet!(@type $field $($mode)?),)*
        }

        // SAFETY: the fields follow `PKTDEF.H` order (checked below) with its types, and the
        // masks name exactly those fields.
        unsafe impl $crate::Packet for $name {
            const DATA: $crate::sys::WTPKT = 0 $(| $crate::packet!(@bit $field))*;
            const MODE: $crate::sys::WTPKT = 0 $(| $crate::packet!(@mode $field $($mode)?))*;
        }

        const _: () = $crate::sys::__assert_order(&[$($crate::packet!(@order $field)),*]);
    };

    (@mode $field:ident relative) => { $crate::packet!(@bit $field) };
    (@mode $field:ident) => { 0 };

    (@type normal_pressure relative) => { i32 };
    (@type tangent_pressure relative) => { i32 };
    (@type buttons relative) => { $crate::ButtonChange };
    (@type context) => { $crate::sys::HCTX };
    (@type status) => { $crate::Status };
    (@type time) => { u32 };
    (@type changed) => { $crate::sys::WTPKT };
    (@type serial_number) => { u32 };
    (@type cursor) => { u32 };
    (@type buttons) => { $crate::Buttons };
    (@type x) => { i32 };
    (@type y) => { i32 };
    (@type z) => { i32 };
    (@type normal_pressure) => { u32 };
    (@type tangent_pressure) => { u32 };
    (@type orientation) => { $crate::Orientation };
    (@type rotation) => { $crate::Rotation };
    (@type $field:ident relative) => { $crate::packet!(@type $field) };

    (@bit context) => { $crate::sys::PK_CONTEXT };
    (@bit status) => { $crate::sys::PK_STATUS };
    (@bit time) => { $crate::sys::PK_TIME };
    (@bit changed) => { $crate::sys::PK_CHANGED };
    (@bit serial_number) => { $crate::sys::PK_SERIAL_NUMBER };
    (@bit cursor) => { $crate::sys::PK_CURSOR };
    (@bit buttons) => { $crate::sys::PK_BUTTONS };
    (@bit x) => { $crate::sys::PK_X };
    (@bit y) => { $crate::sys::PK_Y };
    (@bit z) => { $crate::sys::PK_Z };
    (@bit normal_pressure) => { $crate::sys::PK_NORMAL_PRESSURE };
    (@bit tangent_pressure) => { $crate::sys::PK_TANGENT_PRESSURE };
    (@bit orientation) => { $crate::sys::PK_ORIENTATION };
    (@bit rotation) => { $crate::sys::PK_ROTATION };

    (@order context) => { 0 };
    (@order status) => { 1 };
    (@order time) => { 2 };
    (@order changed) => { 3 };
    (@order serial_number) => { 4 };
    (@order cursor) => { 5 };
    (@order buttons) => { 6 };
    (@order x) => { 7 };
    (@order y) => { 8 };
    (@order z) => { 9 };
    (@order normal_pressure) => { 10 };
    (@order tangent_pressure) => { 11 };
    (@order orientation) => { 12 };
    (@order rotation) => { 13 };
}
