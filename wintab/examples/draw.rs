#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(not(target_os = "windows"))]
fn main() {}

#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    draw::run()
}

#[cfg(target_os = "windows")]
mod draw {
    use std::cell::RefCell;

    use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::HiDpi::{
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
    };
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::w;
    use wintab::{Axis, Context, ContextOptions, Message, Wintab};

    wintab::packet! {
        struct Sample { status, buttons, x, y, normal_pressure }
    }

    const INK: COLORREF = COLORREF(0x000000);
    const PAPER: COLORREF = COLORREF(0xFFFFFF);

    struct Segment {
        from: POINT,
        to: POINT,
        width: i32,
        color: COLORREF,
    }

    struct Canvas {
        hwnd: HWND,
        context: Context<Sample>,
        pressure: Option<Axis>,
        packets: Vec<Sample>,
        last: Option<POINT>,
        segments: Vec<Segment>,
    }

    thread_local! {
        static CANVAS: RefCell<Option<Canvas>> = const { RefCell::new(None) };
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)? };
        let wintab = Wintab::load()?;
        let pressure = wintab.device(0).and_then(|device| device.pressure);

        let instance = unsafe { GetModuleHandleW(None)? };
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            hCursor: unsafe { LoadCursorW(None, IDC_CROSS)? },
            hbrBackground: HBRUSH(unsafe { GetStockObject(WHITE_BRUSH) }.0),
            lpszClassName: w!("wintab-draw"),
            ..Default::default()
        };
        unsafe { RegisterClassW(&class) };
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class.lpszClassName,
                w!("wintab draw"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                960,
                720,
                None,
                None,
                Some(instance.into()),
                None,
            )?
        };

        let context = wintab.open::<Sample>(hwnd, &ContextOptions::new().name("wintab draw"))?;
        CANVAS.set(Some(Canvas {
            hwnd,
            context,
            pressure,
            packets: Vec::new(),
            last: None,
            segments: Vec::new(),
        }));

        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        CANVAS.take();
        Ok(())
    }

    extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let handled = CANVAS.with_borrow_mut(|canvas| {
            let Some(canvas) = canvas else {
                return false;
            };
            match msg {
                WM_PAINT => canvas.paint(),
                WM_ACTIVATE => {
                    let _ = canvas.context.overlap(wparam.0 & 0xFFFF != WA_INACTIVE as usize);
                    return false;
                }
                WM_KEYDOWN if wparam.0 == 0x1B => {
                    canvas.segments.clear();
                    unsafe {
                        let _ = InvalidateRect(Some(hwnd), None, true);
                    }
                }
                _ => return canvas.handle(msg, wparam, lparam),
            }
            true
        });
        if handled {
            return LRESULT(0);
        }
        if msg == WM_DESTROY {
            unsafe { PostQuitMessage(0) };
            return LRESULT(0);
        }
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    impl Canvas {
        fn handle(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
            let Some(message) = self.context.message(msg, wparam, lparam) else {
                return false;
            };
            match message {
                Message::Packet { .. } => {
                    self.packets.clear();
                    self.context.drain(&mut self.packets);
                    let hdc = unsafe { GetDC(Some(self.hwnd)) };
                    for index in 0..self.packets.len() {
                        if let Some(segment) = self.stroke(self.packets[index]) {
                            draw_segment(hdc, &segment);
                            self.segments.push(segment);
                        }
                    }
                    unsafe { ReleaseDC(Some(self.hwnd), hdc) };
                }
                Message::Proximity { entering: false, .. } => self.last = None,
                Message::Update { .. } | Message::InfoChange { .. } => self.context.refresh(),
                _ => {}
            }
            true
        }

        fn stroke(&mut self, packet: Sample) -> Option<Segment> {
            let Sample { status, buttons, x, y, normal_pressure } = packet;
            let pressure = self
                .pressure
                .map_or(1., |axis| axis.normalize(normal_pressure as i32));
            if !buttons.is_pressed(0) || pressure == 0. {
                self.last = None;
                return None;
            }
            let (screen_x, screen_y) = self.context.to_screen(x, y);
            let mut point = POINT {
                x: screen_x.round() as i32,
                y: screen_y.round() as i32,
            };
            unsafe {
                let _ = ScreenToClient(self.hwnd, &mut point);
            }
            let from = self.last.replace(point).unwrap_or(point);
            let (width, color) = if status.inverted() {
                (8 + (pressure * 40.) as i32, PAPER)
            } else {
                (1 + (pressure * 12.) as i32, INK)
            };
            Some(Segment {
                from,
                to: point,
                width,
                color,
            })
        }

        fn paint(&self) {
            let mut paint = PAINTSTRUCT::default();
            let hdc = unsafe { BeginPaint(self.hwnd, &mut paint) };
            for segment in &self.segments {
                draw_segment(hdc, segment);
            }
            unsafe {
                let _ = EndPaint(self.hwnd, &paint);
            }
        }
    }

    fn draw_segment(hdc: HDC, segment: &Segment) {
        unsafe {
            let pen = CreatePen(PS_SOLID, segment.width, segment.color);
            let previous = SelectObject(hdc, pen.into());
            let _ = MoveToEx(hdc, segment.from.x, segment.from.y, None);
            let _ = LineTo(hdc, segment.to.x, segment.to.y);
            SelectObject(hdc, previous);
            let _ = DeleteObject(pen.into());
        }
    }
}
