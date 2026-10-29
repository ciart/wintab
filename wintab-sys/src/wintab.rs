//! Rust declarations of Wacom's Wintab 1.4 SDK headers (`WINTAB.H`, `PKTDEF.H`), Win32 only.
#![cfg(target_os = "windows")]
#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::upper_case_acronyms
)]

mod pktdef;

#[doc(hidden)]
pub use pktdef::__assert_order;

use std::ffi::c_void;

use windows::Win32::Foundation::{FreeLibrary, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use windows::core::{BOOL, PCSTR, PSTR, PWSTR, w};

pub type UINT = u32;
pub type DWORD = u32;
pub type WORD = u16;
pub type BYTE = u8;
pub type LONG = i32;
pub type WCHAR = u16;
pub type LPVOID = *mut c_void;
pub type LPINT = *mut i32;
pub type LPBYTE = *mut u8;

pub const WT_DEFBASE: UINT = 0x7FF0;
pub const WT_MAXOFFSET: UINT = 0xF;

pub const fn _WT_PACKET(b: UINT) -> UINT {
    b
}
pub const fn _WT_CTXOPEN(b: UINT) -> UINT {
    b + 1
}
pub const fn _WT_CTXCLOSE(b: UINT) -> UINT {
    b + 2
}
pub const fn _WT_CTXUPDATE(b: UINT) -> UINT {
    b + 3
}
pub const fn _WT_CTXOVERLAP(b: UINT) -> UINT {
    b + 4
}
pub const fn _WT_PROXIMITY(b: UINT) -> UINT {
    b + 5
}
pub const fn _WT_INFOCHANGE(b: UINT) -> UINT {
    b + 6
}
pub const fn _WT_CSRCHANGE(b: UINT) -> UINT {
    b + 7
}
pub const fn _WT_PACKETEXT(b: UINT) -> UINT {
    b + 8
}
pub const fn _WT_MAX(b: UINT) -> UINT {
    b + WT_MAXOFFSET
}

pub const WT_PACKET: UINT = _WT_PACKET(WT_DEFBASE);
pub const WT_CTXOPEN: UINT = _WT_CTXOPEN(WT_DEFBASE);
pub const WT_CTXCLOSE: UINT = _WT_CTXCLOSE(WT_DEFBASE);
pub const WT_CTXUPDATE: UINT = _WT_CTXUPDATE(WT_DEFBASE);
pub const WT_CTXOVERLAP: UINT = _WT_CTXOVERLAP(WT_DEFBASE);
pub const WT_PROXIMITY: UINT = _WT_PROXIMITY(WT_DEFBASE);
pub const WT_INFOCHANGE: UINT = _WT_INFOCHANGE(WT_DEFBASE);
pub const WT_CSRCHANGE: UINT = _WT_CSRCHANGE(WT_DEFBASE);
pub const WT_PACKETEXT: UINT = _WT_PACKETEXT(WT_DEFBASE);
pub const WT_MAX: UINT = _WT_MAX(WT_DEFBASE);

macro_rules! declare_handle {
    ($($name:ident),*) => {$(
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(pub *mut c_void);

        impl Default for $name {
            fn default() -> Self {
                Self(std::ptr::null_mut())
            }
        }
    )*};
}

declare_handle!(HMGR, HCTX, HWTHOOK);

pub type WTPKT = DWORD;

pub const PK_CONTEXT: WTPKT = 0x0001;
pub const PK_STATUS: WTPKT = 0x0002;
pub const PK_TIME: WTPKT = 0x0004;
pub const PK_CHANGED: WTPKT = 0x0008;
pub const PK_SERIAL_NUMBER: WTPKT = 0x0010;
pub const PK_CURSOR: WTPKT = 0x0020;
pub const PK_BUTTONS: WTPKT = 0x0040;
pub const PK_X: WTPKT = 0x0080;
pub const PK_Y: WTPKT = 0x0100;
pub const PK_Z: WTPKT = 0x0200;
pub const PK_NORMAL_PRESSURE: WTPKT = 0x0400;
pub const PK_TANGENT_PRESSURE: WTPKT = 0x0800;
pub const PK_ORIENTATION: WTPKT = 0x1000;
pub const PK_ROTATION: WTPKT = 0x2000;

pub type FIX32 = DWORD;

pub const fn INT(x: FIX32) -> WORD {
    (x >> 16) as WORD
}

pub const fn FRAC(x: FIX32) -> WORD {
    x as WORD
}

pub fn CASTFIX32(x: f64) -> FIX32 {
    (x * 65536.0) as i64 as FIX32
}

pub const fn ROUND(x: FIX32) -> DWORD {
    INT(x) as DWORD + (FRAC(x) > 0x8000) as DWORD
}

pub const fn FIX_MUL(a: FIX32, b: FIX32) -> FIX32 {
    (((FRAC(a) as DWORD).wrapping_mul(FRAC(b) as DWORD)) >> 16)
        .wrapping_add((INT(a) as DWORD).wrapping_mul(FRAC(b) as DWORD))
        .wrapping_add((INT(b) as DWORD).wrapping_mul(FRAC(a) as DWORD))
        .wrapping_add((INT(a) as DWORD).wrapping_mul(INT(b) as DWORD) << 16)
}

pub const fn FIX_DIV(a: FIX32, b: FIX32) -> FIX32 {
    let mut temp = (a / b) << 16;
    let mut rem = a % b;
    let mut btemp = b;
    if INT(btemp) < 256 {
        rem <<= 8;
    } else {
        btemp >>= 8;
    }
    temp = temp.wrapping_add((rem / btemp) << 8);
    rem %= btemp;
    rem <<= 8;
    temp.wrapping_add(rem / btemp)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AXIS {
    pub axMin: LONG,
    pub axMax: LONG,
    pub axUnits: UINT,
    pub axResolution: FIX32,
}
pub type PAXIS = *mut AXIS;
pub type NPAXIS = *mut AXIS;
pub type LPAXIS = *mut AXIS;

pub const TU_NONE: UINT = 0;
pub const TU_INCHES: UINT = 1;
pub const TU_CENTIMETERS: UINT = 2;
pub const TU_CIRCLE: UINT = 3;

pub const SBN_NONE: BYTE = 0x00;
pub const SBN_LCLICK: BYTE = 0x01;
pub const SBN_LDBLCLICK: BYTE = 0x02;
pub const SBN_LDRAG: BYTE = 0x03;
pub const SBN_RCLICK: BYTE = 0x04;
pub const SBN_RDBLCLICK: BYTE = 0x05;
pub const SBN_RDRAG: BYTE = 0x06;
pub const SBN_MCLICK: BYTE = 0x07;
pub const SBN_MDBLCLICK: BYTE = 0x08;
pub const SBN_MDRAG: BYTE = 0x09;
pub const SBN_PTCLICK: BYTE = 0x10;
pub const SBN_PTDBLCLICK: BYTE = 0x20;
pub const SBN_PTDRAG: BYTE = 0x30;
pub const SBN_PNCLICK: BYTE = 0x40;
pub const SBN_PNDBLCLICK: BYTE = 0x50;
pub const SBN_PNDRAG: BYTE = 0x60;
pub const SBN_P1CLICK: BYTE = 0x70;
pub const SBN_P1DBLCLICK: BYTE = 0x80;
pub const SBN_P1DRAG: BYTE = 0x90;
pub const SBN_P2CLICK: BYTE = 0xA0;
pub const SBN_P2DBLCLICK: BYTE = 0xB0;
pub const SBN_P2DRAG: BYTE = 0xC0;
pub const SBN_P3CLICK: BYTE = 0xD0;
pub const SBN_P3DBLCLICK: BYTE = 0xE0;
pub const SBN_P3DRAG: BYTE = 0xF0;

pub const HWC_INTEGRATED: UINT = 0x0001;
pub const HWC_TOUCH: UINT = 0x0002;
pub const HWC_HARDPROX: UINT = 0x0004;
pub const HWC_PHYSID_CURSORS: UINT = 0x0008;

pub const CRC_MULTIMODE: UINT = 0x0001;
pub const CRC_AGGREGATE: UINT = 0x0002;
pub const CRC_INVERT: UINT = 0x0004;

pub const WTI_INTERFACE: UINT = 1;
pub const IFC_WINTABID: UINT = 1;
pub const IFC_SPECVERSION: UINT = 2;
pub const IFC_IMPLVERSION: UINT = 3;
pub const IFC_NDEVICES: UINT = 4;
pub const IFC_NCURSORS: UINT = 5;
pub const IFC_NCONTEXTS: UINT = 6;
pub const IFC_CTXOPTIONS: UINT = 7;
pub const IFC_CTXSAVESIZE: UINT = 8;
pub const IFC_NEXTENSIONS: UINT = 9;
pub const IFC_NMANAGERS: UINT = 10;
pub const IFC_MAX: UINT = 10;

pub const WTI_STATUS: UINT = 2;
pub const STA_CONTEXTS: UINT = 1;
pub const STA_SYSCTXS: UINT = 2;
pub const STA_PKTRATE: UINT = 3;
pub const STA_PKTDATA: UINT = 4;
pub const STA_MANAGERS: UINT = 5;
pub const STA_SYSTEM: UINT = 6;
pub const STA_BUTTONUSE: UINT = 7;
pub const STA_SYSBTNUSE: UINT = 8;
pub const STA_MAX: UINT = 8;

pub const WTI_DEFCONTEXT: UINT = 3;
pub const WTI_DEFSYSCTX: UINT = 4;
pub const WTI_DDCTXS: UINT = 400;
pub const WTI_DSCTXS: UINT = 500;
pub const CTX_NAME: UINT = 1;
pub const CTX_OPTIONS: UINT = 2;
pub const CTX_STATUS: UINT = 3;
pub const CTX_LOCKS: UINT = 4;
pub const CTX_MSGBASE: UINT = 5;
pub const CTX_DEVICE: UINT = 6;
pub const CTX_PKTRATE: UINT = 7;
pub const CTX_PKTDATA: UINT = 8;
pub const CTX_PKTMODE: UINT = 9;
pub const CTX_MOVEMASK: UINT = 10;
pub const CTX_BTNDNMASK: UINT = 11;
pub const CTX_BTNUPMASK: UINT = 12;
pub const CTX_INORGX: UINT = 13;
pub const CTX_INORGY: UINT = 14;
pub const CTX_INORGZ: UINT = 15;
pub const CTX_INEXTX: UINT = 16;
pub const CTX_INEXTY: UINT = 17;
pub const CTX_INEXTZ: UINT = 18;
pub const CTX_OUTORGX: UINT = 19;
pub const CTX_OUTORGY: UINT = 20;
pub const CTX_OUTORGZ: UINT = 21;
pub const CTX_OUTEXTX: UINT = 22;
pub const CTX_OUTEXTY: UINT = 23;
pub const CTX_OUTEXTZ: UINT = 24;
pub const CTX_SENSX: UINT = 25;
pub const CTX_SENSY: UINT = 26;
pub const CTX_SENSZ: UINT = 27;
pub const CTX_SYSMODE: UINT = 28;
pub const CTX_SYSORGX: UINT = 29;
pub const CTX_SYSORGY: UINT = 30;
pub const CTX_SYSEXTX: UINT = 31;
pub const CTX_SYSEXTY: UINT = 32;
pub const CTX_SYSSENSX: UINT = 33;
pub const CTX_SYSSENSY: UINT = 34;
pub const CTX_MAX: UINT = 34;

pub const WTI_DEVICES: UINT = 100;
pub const DVC_NAME: UINT = 1;
pub const DVC_HARDWARE: UINT = 2;
pub const DVC_NCSRTYPES: UINT = 3;
pub const DVC_FIRSTCSR: UINT = 4;
pub const DVC_PKTRATE: UINT = 5;
pub const DVC_PKTDATA: UINT = 6;
pub const DVC_PKTMODE: UINT = 7;
pub const DVC_CSRDATA: UINT = 8;
pub const DVC_XMARGIN: UINT = 9;
pub const DVC_YMARGIN: UINT = 10;
pub const DVC_ZMARGIN: UINT = 11;
pub const DVC_X: UINT = 12;
pub const DVC_Y: UINT = 13;
pub const DVC_Z: UINT = 14;
pub const DVC_NPRESSURE: UINT = 15;
pub const DVC_TPRESSURE: UINT = 16;
pub const DVC_ORIENTATION: UINT = 17;
pub const DVC_ROTATION: UINT = 18;
pub const DVC_PNPID: UINT = 19;
pub const DVC_MAX: UINT = 19;

pub const WTI_CURSORS: UINT = 200;
pub const CSR_NAME: UINT = 1;
pub const CSR_ACTIVE: UINT = 2;
pub const CSR_PKTDATA: UINT = 3;
pub const CSR_BUTTONS: UINT = 4;
pub const CSR_BUTTONBITS: UINT = 5;
pub const CSR_BTNNAMES: UINT = 6;
pub const CSR_BUTTONMAP: UINT = 7;
pub const CSR_SYSBTNMAP: UINT = 8;
pub const CSR_NPBUTTON: UINT = 9;
pub const CSR_NPBTNMARKS: UINT = 10;
pub const CSR_NPRESPONSE: UINT = 11;
pub const CSR_TPBUTTON: UINT = 12;
pub const CSR_TPBTNMARKS: UINT = 13;
pub const CSR_TPRESPONSE: UINT = 14;
pub const CSR_PHYSID: UINT = 15;
pub const CSR_MODE: UINT = 16;
pub const CSR_MINPKTDATA: UINT = 17;
pub const CSR_MINBUTTONS: UINT = 18;
pub const CSR_CAPABILITIES: UINT = 19;
pub const CSR_TYPE: UINT = 20;
pub const CSR_MAX: UINT = 20;

pub const WTI_EXTENSIONS: UINT = 300;
pub const EXT_NAME: UINT = 1;
pub const EXT_TAG: UINT = 2;
pub const EXT_MASK: UINT = 3;
pub const EXT_SIZE: UINT = 4;
pub const EXT_AXES: UINT = 5;
pub const EXT_DEFAULT: UINT = 6;
pub const EXT_DEFCONTEXT: UINT = 7;
pub const EXT_DEFSYSCTX: UINT = 8;
pub const EXT_CURSORS: UINT = 9;
pub const EXT_DEVICES: UINT = 110;
pub const EXT_MAX: UINT = 210;

pub const LCNAMELEN: usize = 40;
pub const LC_NAMELEN: usize = 40;

macro_rules! logcontext {
    ($name:ident, $char:ty) => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name {
            pub lcName: [$char; LCNAMELEN],
            pub lcOptions: UINT,
            pub lcStatus: UINT,
            pub lcLocks: UINT,
            pub lcMsgBase: UINT,
            pub lcDevice: UINT,
            pub lcPktRate: UINT,
            pub lcPktData: WTPKT,
            pub lcPktMode: WTPKT,
            pub lcMoveMask: WTPKT,
            pub lcBtnDnMask: DWORD,
            pub lcBtnUpMask: DWORD,
            pub lcInOrgX: LONG,
            pub lcInOrgY: LONG,
            pub lcInOrgZ: LONG,
            pub lcInExtX: LONG,
            pub lcInExtY: LONG,
            pub lcInExtZ: LONG,
            pub lcOutOrgX: LONG,
            pub lcOutOrgY: LONG,
            pub lcOutOrgZ: LONG,
            pub lcOutExtX: LONG,
            pub lcOutExtY: LONG,
            pub lcOutExtZ: LONG,
            pub lcSensX: FIX32,
            pub lcSensY: FIX32,
            pub lcSensZ: FIX32,
            pub lcSysMode: BOOL,
            pub lcSysOrgX: i32,
            pub lcSysOrgY: i32,
            pub lcSysExtX: i32,
            pub lcSysExtY: i32,
            pub lcSysSensX: FIX32,
            pub lcSysSensY: FIX32,
        }

        impl Default for $name {
            fn default() -> Self {
                // SAFETY: every field is an integer or an array of integers.
                unsafe { std::mem::zeroed() }
            }
        }
    };
}

logcontext!(LOGCONTEXTA, i8);
logcontext!(LOGCONTEXTW, WCHAR);
pub type PLOGCONTEXTA = *mut LOGCONTEXTA;
pub type NPLOGCONTEXTA = *mut LOGCONTEXTA;
pub type LPLOGCONTEXTA = *mut LOGCONTEXTA;
pub type PLOGCONTEXTW = *mut LOGCONTEXTW;
pub type NPLOGCONTEXTW = *mut LOGCONTEXTW;
pub type LPLOGCONTEXTW = *mut LOGCONTEXTW;

pub const CXO_SYSTEM: UINT = 0x0001;
pub const CXO_PEN: UINT = 0x0002;
pub const CXO_MESSAGES: UINT = 0x0004;
pub const CXO_MARGIN: UINT = 0x8000;
pub const CXO_MGNINSIDE: UINT = 0x4000;
pub const CXO_CSRMESSAGES: UINT = 0x0008;

pub const CXS_DISABLED: UINT = 0x0001;
pub const CXS_OBSCURED: UINT = 0x0002;
pub const CXS_ONTOP: UINT = 0x0004;

pub const CXL_INSIZE: UINT = 0x0001;
pub const CXL_INASPECT: UINT = 0x0002;
pub const CXL_SENSITIVITY: UINT = 0x0004;
pub const CXL_MARGIN: UINT = 0x0008;
pub const CXL_SYSOUT: UINT = 0x0010;

pub const TPS_PROXIMITY: UINT = 0x0001;
pub const TPS_QUEUE_ERR: UINT = 0x0002;
pub const TPS_MARGIN: UINT = 0x0004;
pub const TPS_GRAB: UINT = 0x0008;
pub const TPS_INVERT: UINT = 0x0010;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ORIENTATION {
    pub orAzimuth: i32,
    pub orAltitude: i32,
    pub orTwist: i32,
}
pub type PORIENTATION = *mut ORIENTATION;
pub type NPORIENTATION = *mut ORIENTATION;
pub type LPORIENTATION = *mut ORIENTATION;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ROTATION {
    pub roPitch: i32,
    pub roRoll: i32,
    pub roYaw: i32,
}
pub type PROTATION = *mut ROTATION;
pub type NPROTATION = *mut ROTATION;
pub type LPROTATION = *mut ROTATION;

pub const TBN_NONE: UINT = 0;
pub const TBN_UP: UINT = 1;
pub const TBN_DOWN: UINT = 2;

pub const WTDC_NONE: UINT = 0;
pub const WTDC_CANCEL: UINT = 1;
pub const WTDC_OK: UINT = 2;
pub const WTDC_RESTART: UINT = 3;

pub const WTH_PLAYBACK: i32 = 1;
pub const WTH_RECORD: i32 = 2;

pub const WTHC_GETLPLPFN: i32 = -3;
pub const WTHC_LPLPFNNEXT: i32 = -2;
pub const WTHC_LPFNNEXT: i32 = -1;
pub const WTHC_ACTION: i32 = 0;
pub const WTHC_GETNEXT: i32 = 1;
pub const WTHC_SKIP: i32 = 2;

pub const WTP_LPDEFAULT: LPVOID = -1isize as LPVOID;
pub const WTP_DWDEFAULT: DWORD = -1i32 as DWORD;

pub const PKEXT_ABSOLUTE: UINT = 1;
pub const PKEXT_RELATIVE: UINT = 2;

pub const WTX_OBT: UINT = 0;
pub const WTX_FKEYS: UINT = 1;
pub const WTX_TILT: UINT = 2;
pub const WTX_CSRMASK: UINT = 3;
pub const WTX_XBTNMASK: UINT = 4;
pub const WTX_EXPKEYS: UINT = 5;
pub const WTX_TOUCHSTRIP: UINT = 6;
pub const WTX_TOUCHRING: UINT = 7;
pub const WTX_EXPKEYS2: UINT = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XBTNMASK {
    pub xBtnDnMask: [BYTE; 32],
    pub xBtnUpMask: [BYTE; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TILT {
    pub tiltX: i32,
    pub tiltY: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EXTENSIONBASE {
    pub nContext: HCTX,
    pub nStatus: UINT,
    pub nTime: DWORD,
    pub nSerialNumber: UINT,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EXPKEYSDATA {
    pub nTablet: BYTE,
    pub nControl: BYTE,
    pub nLocation: BYTE,
    pub nReserved: BYTE,
    pub nState: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SLIDERDATA {
    pub nTablet: BYTE,
    pub nControl: BYTE,
    pub nMode: BYTE,
    pub nReserved: BYTE,
    pub nPosition: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EXTPROPERTY {
    pub version: BYTE,
    pub tabletIndex: BYTE,
    pub controlIndex: BYTE,
    pub functionIndex: BYTE,
    pub propertyID: WORD,
    pub reserved: WORD,
    pub dataSize: DWORD,
    pub data: [BYTE; 1],
}

pub const TABLET_PROPERTY_CONTROLCOUNT: WORD = 0;
pub const TABLET_PROPERTY_FUNCCOUNT: WORD = 1;
pub const TABLET_PROPERTY_AVAILABLE: WORD = 2;
pub const TABLET_PROPERTY_MIN: WORD = 3;
pub const TABLET_PROPERTY_MAX: WORD = 4;
pub const TABLET_PROPERTY_OVERRIDE: WORD = 5;
pub const TABLET_PROPERTY_OVERRIDE_NAME: WORD = 6;
pub const TABLET_PROPERTY_LOCATION: WORD = 11;

pub const TABLET_LOC_LEFT: u32 = 0;
pub const TABLET_LOC_RIGHT: u32 = 1;
pub const TABLET_LOC_TOP: u32 = 2;
pub const TABLET_LOC_BOTTOM: u32 = 3;
pub const TABLET_LOC_TRANSDUCER: u32 = 4;

pub type WTENUMPROC = Option<unsafe extern "system" fn(HCTX, LPARAM) -> BOOL>;
pub type WTCONFIGPROC = Option<unsafe extern "system" fn(HCTX, HWND) -> BOOL>;
pub type WTHOOKPROC = Option<unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT>;
pub type LPWTHOOKPROC = *mut WTHOOKPROC;

macro_rules! functions {
    ($($name:ident = $ordinal:ident($ord:literal): fn($($arg:ty),*) -> $ret:ty;)*) => {
        $(pub const $ordinal: u16 = $ord;)*

        /// Entry points of a loaded `wintab32.dll`. A function the driver does not export is `None`.
        /// The pointers are valid only while this value is alive.
        pub struct Wintab32 {
            module: HMODULE,
            $(pub $name: Option<unsafe extern "system" fn($($arg),*) -> $ret>,)*
        }

        impl Wintab32 {
            /// Loads `wintab32.dll` from the system directory.
            ///
            /// # Safety
            /// Runs the driver's `DllMain`.
            pub unsafe fn load() -> windows::core::Result<Self> {
                let module = unsafe {
                    LoadLibraryExW(w!("wintab32.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32)?
                };
                Ok(Self {
                    module,
                    $($name: unsafe {
                        GetProcAddress(module, PCSTR(concat!(stringify!($name), "\0").as_ptr()))
                            .map(|f| std::mem::transmute::<
                                unsafe extern "system" fn() -> isize,
                                unsafe extern "system" fn($($arg),*) -> $ret,
                            >(f))
                    },)*
                })
            }
        }
    };
}

functions! {
    WTInfoA = ORD_WTInfoA(20): fn(UINT, UINT, LPVOID) -> UINT;
    WTInfoW = ORD_WTInfoW(1020): fn(UINT, UINT, LPVOID) -> UINT;
    WTOpenA = ORD_WTOpenA(21): fn(HWND, LPLOGCONTEXTA, BOOL) -> HCTX;
    WTOpenW = ORD_WTOpenW(1021): fn(HWND, LPLOGCONTEXTW, BOOL) -> HCTX;
    WTClose = ORD_WTClose(22): fn(HCTX) -> BOOL;
    WTPacketsGet = ORD_WTPacketsGet(23): fn(HCTX, i32, LPVOID) -> i32;
    WTPacket = ORD_WTPacket(24): fn(HCTX, UINT, LPVOID) -> BOOL;
    WacomCleanup = ORD_WacomCleanup(607): fn() -> BOOL;
    WTEnable = ORD_WTEnable(40): fn(HCTX, BOOL) -> BOOL;
    WTOverlap = ORD_WTOverlap(41): fn(HCTX, BOOL) -> BOOL;
    WTConfig = ORD_WTConfig(60): fn(HCTX, HWND) -> BOOL;
    WTGetA = ORD_WTGetA(61): fn(HCTX, LPLOGCONTEXTA) -> BOOL;
    WTGetW = ORD_WTGetW(1061): fn(HCTX, LPLOGCONTEXTW) -> BOOL;
    WTSetA = ORD_WTSetA(62): fn(HCTX, LPLOGCONTEXTA) -> BOOL;
    WTSetW = ORD_WTSetW(1062): fn(HCTX, LPLOGCONTEXTW) -> BOOL;
    WTExtGet = ORD_WTExtGet(63): fn(HCTX, UINT, LPVOID) -> BOOL;
    WTExtSet = ORD_WTExtSet(64): fn(HCTX, UINT, LPVOID) -> BOOL;
    WTSave = ORD_WTSave(65): fn(HCTX, LPVOID) -> BOOL;
    WTRestore = ORD_WTRestore(66): fn(HWND, LPVOID, BOOL) -> HCTX;
    WTPacketsPeek = ORD_WTPacketsPeek(80): fn(HCTX, i32, LPVOID) -> i32;
    WTDataGet = ORD_WTDataGet(81): fn(HCTX, UINT, UINT, i32, LPVOID, LPINT) -> i32;
    WTDataPeek = ORD_WTDataPeek(82): fn(HCTX, UINT, UINT, i32, LPVOID, LPINT) -> i32;
    WTQueueSizeGet = ORD_WTQueueSizeGet(84): fn(HCTX) -> i32;
    WTQueueSizeSet = ORD_WTQueueSizeSet(85): fn(HCTX, i32) -> BOOL;
    WTMgrOpen = ORD_WTMgrOpen(100): fn(HWND, UINT) -> HMGR;
    WTMgrClose = ORD_WTMgrClose(101): fn(HMGR) -> BOOL;
    WTMgrContextEnum = ORD_WTMgrContextEnum(120): fn(HMGR, WTENUMPROC, LPARAM) -> BOOL;
    WTMgrContextOwner = ORD_WTMgrContextOwner(121): fn(HMGR, HCTX) -> HWND;
    WTMgrDefContext = ORD_WTMgrDefContext(122): fn(HMGR, BOOL) -> HCTX;
    WTMgrDefContextEx = ORD_WTMgrDefContextEx(206): fn(HMGR, UINT, BOOL) -> HCTX;
    WTMgrDeviceConfig = ORD_WTMgrDeviceConfig(140): fn(HMGR, UINT, HWND) -> UINT;
    WTMgrExt = ORD_WTMgrExt(180): fn(HMGR, UINT, LPVOID) -> BOOL;
    WTMgrCsrEnable = ORD_WTMgrCsrEnable(181): fn(HMGR, UINT, BOOL) -> BOOL;
    WTMgrCsrButtonMap = ORD_WTMgrCsrButtonMap(182): fn(HMGR, UINT, LPBYTE, LPBYTE) -> BOOL;
    WTMgrCsrPressureBtnMarks = ORD_WTMgrCsrPressureBtnMarks(183): fn(HMGR, UINT, DWORD, DWORD) -> BOOL;
    WTMgrCsrPressureResponse = ORD_WTMgrCsrPressureResponse(184): fn(HMGR, UINT, *mut UINT, *mut UINT) -> BOOL;
    WTMgrCsrExt = ORD_WTMgrCsrExt(185): fn(HMGR, UINT, UINT, LPVOID) -> BOOL;
    WTQueuePacketsEx = ORD_WTQueuePacketsEx(200): fn(HCTX, *mut UINT, *mut UINT) -> BOOL;
    WTMgrConfigReplaceExA = ORD_WTMgrConfigReplaceExA(202): fn(HMGR, BOOL, PSTR, PSTR) -> BOOL;
    WTMgrConfigReplaceExW = ORD_WTMgrConfigReplaceExW(1202): fn(HMGR, BOOL, PWSTR, PSTR) -> BOOL;
    WTMgrPacketHookExA = ORD_WTMgrPacketHookExA(203): fn(HMGR, i32, PSTR, PSTR) -> HWTHOOK;
    WTMgrPacketHookExW = ORD_WTMgrPacketHookExW(1203): fn(HMGR, i32, PWSTR, PSTR) -> HWTHOOK;
    WTMgrPacketUnhook = ORD_WTMgrPacketUnhook(204): fn(HWTHOOK) -> BOOL;
    WTMgrPacketHookNext = ORD_WTMgrPacketHookNext(205): fn(HWTHOOK, i32, WPARAM, LPARAM) -> LRESULT;
    WTMgrCsrPressureBtnMarksEx = ORD_WTMgrCsrPressureBtnMarksEx(201): fn(HMGR, UINT, *mut UINT, *mut UINT) -> BOOL;
}

impl Drop for Wintab32 {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.module).ok();
        }
    }
}
