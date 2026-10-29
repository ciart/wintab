/// A Wintab window message, decoded by [`Context::message`](crate::Context::message).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    /// Packets are queued; `serial` is the newest one.
    Packet {
        /// The serial number of the newest queued packet.
        serial: u32,
    },
    /// A cursor entered or left the context.
    Proximity {
        /// Whether the cursor entered rather than left.
        entering: bool,
        /// Whether the cursor entered or left the tablet's sensing range, rather than only
        /// the context.
        hardware: bool,
    },
    /// A new cursor type came into use; `serial` is the packet that reports it.
    CursorChange {
        /// The serial number of the packet that reports the change.
        serial: u32,
    },
    /// The context was opened.
    Opened {
        /// `CXS_*` status bits.
        status: u32,
    },
    /// The context was closed, possibly by the driver.
    Closed {
        /// `CXS_*` status bits.
        status: u32,
    },
    /// An extension packet is available. Read it with
    /// [`Context::packet_ext`](crate::Context::packet_ext).
    PacketExt {
        /// The serial number of the extension packet.
        serial: u32,
        /// The handle to read it from. Drivers report a handle other than the context's own.
        source: crate::sys::HCTX,
    },
    /// The context moved in the overlap order or changed its enabled state.
    Overlap {
        /// `CXS_*` status bits.
        status: u32,
    },
    /// The context was updated by the driver or another application.
    Update {
        /// `CXS_*` status bits.
        status: u32,
    },
    /// Driver information changed, for example a tablet was plugged in.
    InfoChange {
        /// The `WTI_*` category that changed.
        category: u16,
        /// The index within the category.
        index: u16,
    },
}
