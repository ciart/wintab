# wintab-sys

Raw Rust bindings for the Wintab 1.4 tablet API, translated one to one from the
`WINTAB.H` and `PKTDEF.H` headers in Wacom's
[`wacom-device-kit-windows`](https://github.com/Wacom-Developer/wacom-device-kit-windows/tree/master/WintabSDK).
Names, fields and constants keep their C spelling (`WTInfoW`, `LOGCONTEXTW::lcPktData`,
`PK_NORMAL_PRESSURE`). Windows only; the crate is empty on other targets.

## Loading

`wintab32.dll` is installed by tablet drivers and is missing on many machines, so it is not
linked. `Wintab32::load()` loads it from the system directory and resolves every function the
header declares. A function the driver does not export is `None`.

```rust
let wt = unsafe { wintab_sys::Wintab32::load()? };
let mut lc = wintab_sys::LOGCONTEXTW::default();
unsafe { wt.WTInfoW.unwrap()(wintab_sys::WTI_DEFCONTEXT, 0, &mut lc as *mut _ as _) };
```

## Packets

`PKTDEF.H` builds the `PACKET` struct from the `PACKETDATA` bitmask with the preprocessor.
Here you list the fields instead, and the macro derives the masks:

```rust
wintab_sys::packet! {
    pub struct PACKET { pkStatus, pkButtons(relative), pkX, pkY, pkNormalPressure }
}
lc.lcPktData = PACKET::PACKETDATA;
lc.lcPktMode = PACKET::PACKETMODE;
```

Fields must follow the header's order, which is checked at compile time. `(relative)` adds the
field to `PACKETMODE`; relative pressure fields become `i32` as in the header. Extension fields
(`pkFKeys`, `pkTilt`) have no fixed bit: OR their `EXT_MASK` into `lcPktData` yourself.
`packetext!` builds `PACKETEXT` the same way, with `pkBase` always first.

### Layout differs from the header in one place

Measured with a Wacom driver (Wintab implementation 1.39) on x64:

- `PACKET`: the driver concatenates fields without padding. With `PK_CONTEXT` (8 bytes on
  64-bit) and an odd number of 4-byte fields, MSVC pads the header's struct by 4 bytes and
  arrays from `WTPacketsGet` misalign from the second packet on. `packet!` structs are
  therefore `repr(C, packed(4))`.
- `PACKETEXT`: the driver keeps MSVC's padding after `pkBase`, so `packetext!` structs are
  plain `repr(C)`.

Other drivers and 32-bit builds have not been measured.

`WT_PACKETEXT` carries a context handle in `lParam` that differs from the one `WTOpenW`
returned; pass the `lParam` handle to `WTPacket`.

## Not included

- The 16-bit branches of the headers and the functions marked obsolete in Win32.
- Aliases that depend on the `UNICODE` macro (`WTInfo`, `LOGCONTEXT`): pick `A` or `W`.
- `MSGPACK.H`, which holds `windowsx.h` message helpers rather than Wintab API.
- The OLED icon properties (`TABLET_PROPERTY_ICON_*`) of older per-sample headers, which the
  current SDK header no longer has.

## License

MIT. The Wintab headers are copyright Wacom Co., Ltd. and LCS/Telegraphics and state that
they "may be freely used, copied, or distributed without compensation or licensing
restrictions".
