use crate::Wintab;
use crate::sys::{self, AXIS};

/// A `major.minor` version as Wintab reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// The major version.
    pub major: u8,
    /// The minor version.
    pub minor: u8,
}

impl Version {
    fn from_word(word: u16) -> Self {
        Self {
            major: (word >> 8) as u8,
            minor: word as u8,
        }
    }
}

/// Identification of the Wintab implementation (`WTI_INTERFACE`).
#[derive(Clone, Debug)]
pub struct Interface {
    /// The driver's description of itself.
    pub id: String,
    /// The Wintab specification version the driver implements.
    pub spec_version: Version,
    /// The driver's own version.
    pub impl_version: Version,
    /// The number of tablets the driver supports.
    pub devices: u32,
    /// The number of cursor types the driver supports.
    pub cursors: u32,
}

pub(crate) fn interface(wintab: &Wintab) -> Interface {
    let word =
        |index| Version::from_word(wintab.info::<u16>(sys::WTI_INTERFACE, index).unwrap_or(0));
    Interface {
        id: wintab
            .info_string(sys::WTI_INTERFACE, sys::IFC_WINTABID)
            .unwrap_or_default(),
        spec_version: word(sys::IFC_SPECVERSION),
        impl_version: word(sys::IFC_IMPLVERSION),
        devices: wintab
            .info(sys::WTI_INTERFACE, sys::IFC_NDEVICES)
            .unwrap_or(0),
        cursors: wintab
            .info(sys::WTI_INTERFACE, sys::IFC_NCURSORS)
            .unwrap_or(0),
    }
}

/// The unit of an [`Axis`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Units {
    /// No physical unit.
    None,
    /// Inches.
    Inches,
    /// Centimeters.
    Centimeters,
    /// Angles; [`Axis::resolution`] is the number of units in a full circle.
    Circle,
    /// A unit this crate does not know.
    Other(u32),
}

/// The range and resolution of one tablet axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis {
    /// The smallest value the axis reports.
    pub min: i32,
    /// The largest value the axis reports.
    pub max: i32,
    /// The physical unit of [`Self::resolution`].
    pub units: Units,
    /// Values per unit.
    pub resolution: f64,
}

impl Axis {
    fn new(axis: AXIS) -> Option<Self> {
        (axis.axMax > axis.axMin || axis.axResolution != 0).then(|| Self {
            min: axis.axMin,
            max: axis.axMax,
            units: match axis.axUnits {
                sys::TU_NONE => Units::None,
                sys::TU_INCHES => Units::Inches,
                sys::TU_CENTIMETERS => Units::Centimeters,
                sys::TU_CIRCLE => Units::Circle,
                other => Units::Other(other),
            },
            resolution: axis.axResolution as f64 / 65536.,
        })
    }

    /// Maps `value` from `min..=max` to `0.0..=1.0`.
    pub fn normalize(&self, value: i32) -> f32 {
        if self.max <= self.min {
            return 0.;
        }
        ((value - self.min) as f32 / (self.max - self.min) as f32).clamp(0., 1.)
    }

    /// Converts an angle `value` to degrees. `None` unless the axis measures angles.
    pub fn degrees(&self, value: i32) -> Option<f64> {
        (self.units == Units::Circle && self.resolution > 0.)
            .then(|| value as f64 * 360. / self.resolution)
    }
}

/// One tablet (`WTI_DEVICES`).
#[derive(Clone, Debug)]
pub struct Device {
    /// The tablet's name.
    pub name: String,
    /// The x axis.
    pub x: Option<Axis>,
    /// The y axis.
    pub y: Option<Axis>,
    /// Tip pressure.
    pub pressure: Option<Axis>,
    /// Barrel pressure, such as an airbrush wheel.
    pub tangent_pressure: Option<Axis>,
    /// Azimuth, altitude and twist.
    pub orientation: Option<[Axis; 3]>,
}

pub(crate) fn device(wintab: &Wintab, index: u32) -> Option<Device> {
    let category = sys::WTI_DEVICES + index;
    let name = wintab.info_string(category, sys::DVC_NAME)?;
    let axis = |item| wintab.info::<AXIS>(category, item).and_then(Axis::new);
    let orientation = wintab
        .info::<[AXIS; 3]>(category, sys::DVC_ORIENTATION)
        .and_then(|[azimuth, altitude, twist]| {
            Some([Axis::new(azimuth)?, Axis::new(altitude)?, Axis::new(twist)?])
        });
    Some(Device {
        name,
        x: axis(sys::DVC_X),
        y: axis(sys::DVC_Y),
        pressure: axis(sys::DVC_NPRESSURE),
        tangent_pressure: axis(sys::DVC_TPRESSURE),
        orientation,
    })
}

/// What a [`Cursor`] can do (`CSR_CAPABILITIES`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CursorCapabilities {
    /// The cursor shares its buttons and axes with other cursor types of one device.
    pub multimode: bool,
    /// The cursor reports as one cursor type with the others of its device.
    pub aggregate: bool,
    /// The cursor can be turned over, like a pen's eraser end.
    pub invert: bool,
}

/// One cursor type (`WTI_CURSORS`), such as a pen tip or eraser.
#[derive(Clone, Debug)]
pub struct Cursor {
    /// The cursor's name.
    pub name: String,
    /// Whether the cursor is currently connected.
    pub active: bool,
    /// Identifies the physical cursor within its [`Self::kind`]; `0` when unknown.
    pub physical_id: u32,
    /// The driver's cursor type code (`CSR_TYPE`).
    pub kind: u32,
    /// What the cursor can do.
    pub capabilities: CursorCapabilities,
    /// The packet data the cursor can report.
    pub packet_data: sys::WTPKT,
    /// The names of the cursor's physical buttons, in button order.
    pub button_names: Vec<String>,
    /// For each physical button, the logical button it reports as in `buttons` packets.
    pub button_map: Vec<u8>,
    /// For each logical button, the `SBN_*` action the driver performs in system contexts.
    pub system_button_map: Vec<u8>,
}

pub(crate) fn cursor(wintab: &Wintab, index: u32) -> Option<Cursor> {
    let category = sys::WTI_CURSORS + index;
    let buttons = wintab.info::<u8>(category, sys::CSR_BUTTONS).unwrap_or(0) as usize;
    let map = |item| {
        let mut map = wintab
            .info::<[u8; 32]>(category, item)
            .unwrap_or([0; 32])
            .to_vec();
        map.truncate(buttons);
        map
    };
    let capabilities = wintab
        .info::<u32>(category, sys::CSR_CAPABILITIES)
        .unwrap_or(0);
    Some(Cursor {
        name: wintab.info_string(category, sys::CSR_NAME)?,
        active: wintab
            .info::<i32>(category, sys::CSR_ACTIVE)
            .is_some_and(|active| active != 0),
        physical_id: wintab.info(category, sys::CSR_PHYSID).unwrap_or(0),
        kind: wintab.info(category, sys::CSR_TYPE).unwrap_or(0),
        capabilities: CursorCapabilities {
            multimode: capabilities & sys::CRC_MULTIMODE != 0,
            aggregate: capabilities & sys::CRC_AGGREGATE != 0,
            invert: capabilities & sys::CRC_INVERT != 0,
        },
        packet_data: wintab.info(category, sys::CSR_PKTDATA).unwrap_or(0),
        button_names: wintab
            .info_strings(category, sys::CSR_BTNNAMES)
            .into_iter()
            .take(buttons)
            .collect(),
        button_map: map(sys::CSR_BUTTONMAP),
        system_button_map: map(sys::CSR_SYSBTNMAP),
    })
}
