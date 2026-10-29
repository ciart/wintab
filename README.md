# wintab

Rust bindings for the Wintab 1.4 tablet API, in two crates:

- `wintab`: a safe wrapper. `Wintab::load()` loads the driver, `Wintab::open` opens a
  `Context<P>` whose packet type `P` (declared with `wintab::packet!`) fixes the packet mask,
  and `Context::message` decodes the context's window messages inside the application's own
  message loop. Device, cursor and axis queries return plain Rust types.
- `wintab-sys`: raw declarations translated one to one from the `WINTAB.H` and `PKTDEF.H`
  headers in Wacom's
  [`wacom-device-kit-windows`](https://github.com/Wacom-Developer/wacom-device-kit-windows/tree/master/WintabSDK).
  Names, fields and constants keep their C spelling (`WTInfoW`, `LOGCONTEXTW::lcPktData`,
  `PK_NORMAL_PRESSURE`).

Windows only; both crates are empty on other targets.

## wintab

```rust
wintab::packet! {
    struct Sample { status, buttons, x, y, normal_pressure }
}

let wintab = unsafe { wintab::Wintab::load()? };
let pressure = wintab.device(0).and_then(|device| device.pressure);
let context = wintab.open::<Sample>(hwnd, &wintab::ContextOptions::new())?;

// In the window procedure of `hwnd`:
match context.message(msg, wparam, lparam) {
    Some(wintab::Message::Packet { .. }) => {
        let mut packets = Vec::new();
        context.drain(&mut packets);
        for &Sample { x, y, normal_pressure, .. } in &packets {
            let (screen_x, screen_y) = context.to_screen(x, y);
            let pressure = pressure.map_or(0., |axis| axis.normalize(normal_pressure as i32));
        }
    }
    Some(wintab::Message::Update { .. } | wintab::Message::InfoChange { .. }) => context.refresh(),
    _ => {}
}
```

Call `context.overlap(true)` on `WM_ACTIVATE` so the active window gets packets before other
applications. The context is closed when dropped.

[`wintab/examples/draw.rs`](wintab/examples/draw.rs) is a small canvas: the pen draws with
pressure-dependent width, the eraser end erases and Esc clears.

```sh
cargo run --example draw
```

## wintab-sys

### Loading

`wintab32.dll` is installed by tablet drivers and is missing on many machines, so it is not
linked. `Wintab32::load()` loads it from the system directory and resolves every function the
header declares. A function the driver does not export is `None`.

```rust
let wt = unsafe { wintab_sys::Wintab32::load()? };
let mut lc = wintab_sys::LOGCONTEXTW::default();
unsafe { wt.WTInfoW.unwrap()(wintab_sys::WTI_DEFCONTEXT, 0, &mut lc as *mut _ as _) };
```

### Packets

List the fields and the macro derives the masks:

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

## License

MIT. The Wintab headers are copyright Wacom Company, Ltd., with portions copyright 1991-1998
by LCS/Telegraphics.
