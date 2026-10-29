/// `PKTDEF.H` for one packet format. Fields must follow the header's order; append
/// `(relative)` to a field to put its bit in `PACKETMODE`. `pkFKeys` and `pkTilt` have no
/// fixed bit, so OR their `EXT_MASK` into `lcPktData` (and `lcPktMode`) yourself.
///
/// ```ignore
/// wintab_sys::packet! {
///     pub struct PACKET { pkStatus, pkButtons(relative), pkX, pkY, pkNormalPressure }
/// }
/// lc.lcPktData = PACKET::PACKETDATA;
/// lc.lcPktMode = PACKET::PACKETMODE;
/// ```
#[macro_export]
macro_rules! packet {
    ($(#[$attr:meta])* $vis:vis struct $name:ident { $($field:ident $(($mode:ident))?),* $(,)? }) => {
        $(#[$attr])*
        // Drivers pack packets without the tail padding MSVC adds after a 64-bit `pkContext`;
        // `WTPacketsGet` arrays misalign if this changes.
        #[repr(C, packed(4))]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        #[allow(non_snake_case)]
        $vis struct $name {
            $(pub $field: $crate::packet!(@type $field $($mode)?),)*
        }

        impl $name {
            pub const PACKETDATA: $crate::WTPKT = 0 $(| $crate::packet!(@bit $field))*;
            pub const PACKETMODE: $crate::WTPKT = 0 $(| $crate::packet!(@mode $field $($mode)?))*;
        }

        const _: () = $crate::__assert_order(&[$($crate::packet!(@order $field)),*]);
    };

    (@mode $field:ident relative) => { $crate::packet!(@bit $field) };
    (@mode $field:ident) => { 0 };

    (@type pkNormalPressure relative) => { i32 };
    (@type pkTangentPressure relative) => { i32 };

    (@type pkContext) => { $crate::HCTX };
    (@type pkStatus) => { $crate::UINT };
    (@type pkTime) => { $crate::DWORD };
    (@type pkChanged) => { $crate::WTPKT };
    (@type pkSerialNumber) => { $crate::UINT };
    (@type pkCursor) => { $crate::UINT };
    (@type pkButtons) => { $crate::DWORD };
    (@type pkX) => { $crate::LONG };
    (@type pkY) => { $crate::LONG };
    (@type pkZ) => { $crate::LONG };
    (@type pkNormalPressure) => { $crate::UINT };
    (@type pkTangentPressure) => { $crate::UINT };
    (@type pkOrientation) => { $crate::ORIENTATION };
    (@type pkRotation) => { $crate::ROTATION };
    (@type pkFKeys) => { $crate::UINT };
    (@type pkTilt) => { $crate::TILT };
    (@type $field:ident relative) => { $crate::packet!(@type $field) };

    (@bit pkContext) => { $crate::PK_CONTEXT };
    (@bit pkStatus) => { $crate::PK_STATUS };
    (@bit pkTime) => { $crate::PK_TIME };
    (@bit pkChanged) => { $crate::PK_CHANGED };
    (@bit pkSerialNumber) => { $crate::PK_SERIAL_NUMBER };
    (@bit pkCursor) => { $crate::PK_CURSOR };
    (@bit pkButtons) => { $crate::PK_BUTTONS };
    (@bit pkX) => { $crate::PK_X };
    (@bit pkY) => { $crate::PK_Y };
    (@bit pkZ) => { $crate::PK_Z };
    (@bit pkNormalPressure) => { $crate::PK_NORMAL_PRESSURE };
    (@bit pkTangentPressure) => { $crate::PK_TANGENT_PRESSURE };
    (@bit pkOrientation) => { $crate::PK_ORIENTATION };
    (@bit pkRotation) => { $crate::PK_ROTATION };
    (@bit pkFKeys) => { 0 };
    (@bit pkTilt) => { 0 };

    (@order pkContext) => { 0 };
    (@order pkStatus) => { 1 };
    (@order pkTime) => { 2 };
    (@order pkChanged) => { 3 };
    (@order pkSerialNumber) => { 4 };
    (@order pkCursor) => { 5 };
    (@order pkButtons) => { 6 };
    (@order pkX) => { 7 };
    (@order pkY) => { 8 };
    (@order pkZ) => { 9 };
    (@order pkNormalPressure) => { 10 };
    (@order pkTangentPressure) => { 11 };
    (@order pkOrientation) => { 12 };
    (@order pkRotation) => { 13 };
    (@order pkFKeys) => { 14 };
    (@order pkTilt) => { 15 };
}

/// `PKTDEF.H`'s `PACKETEXT` for one packet format. `pkBase` is always first; list the
/// extensions to include, in the header's order.
#[macro_export]
macro_rules! packetext {
    ($(#[$attr:meta])* $vis:vis struct $name:ident { $($field:ident),* $(,)? }) => {
        $(#[$attr])*
        // Unlike `PACKET`, drivers keep MSVC's padding after `pkBase`; packing this misreads every field.
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        #[allow(non_snake_case)]
        $vis struct $name {
            pub pkBase: $crate::EXTENSIONBASE,
            $(pub $field: $crate::packetext!(@type $field),)*
        }

        const _: () = $crate::__assert_order(&[$($crate::packetext!(@order $field)),*]);
    };

    (@type pkExpKeys) => { $crate::EXPKEYSDATA };
    (@type pkTouchStrip) => { $crate::SLIDERDATA };
    (@type pkTouchRing) => { $crate::SLIDERDATA };

    (@order pkExpKeys) => { 0 };
    (@order pkTouchStrip) => { 1 };
    (@order pkTouchRing) => { 2 };
}

#[doc(hidden)]
pub const fn __assert_order(order: &[u32]) {
    let mut i = 1;
    while i < order.len() {
        assert!(
            order[i - 1] < order[i],
            "fields must follow PKTDEF.H order without repeats"
        );
        i += 1;
    }
}
