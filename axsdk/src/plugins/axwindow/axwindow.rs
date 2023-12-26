use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::LibraryLoader::*,
        UI::WindowsAndMessaging::*,
        UI::HiDpi::*,
        UI::Input::*,
        UI::Input::KeyboardAndMouse::*,
        Devices::HumanInterfaceDevice::*,
        UI::TextServices::*,
        Graphics::Gdi::*,
        UI::Controls::*,
        UI::Controls::Dialogs::*
    }
};

// TODO(mdeforge): wndproc needs to somehow know the window title before it gets the AxWindow...

/// Turns a Rust string slice into a null-terminated utf-16 vector.
pub fn wide_null(text: String) -> Vec<u16> {
    let mut new_text: Vec<_> = text.encode_utf16().collect();
    new_text.push(0);

    new_text
}

#[derive(Default)]
pub struct AxWindow {
    title: String,
    handle: HWND,
    instance: HMODULE,
    pub has_requested_close: bool,
    pub has_requested_quit: bool
}

impl AxWindow {
    pub fn new<S: AsRef<str>>(title: S) -> Box<AxWindow> {
        let mut window = Box::new(AxWindow::default());

        unsafe {
            window.instance = GetModuleHandleA(None).unwrap();
            debug_assert!(window.instance.0 != 0);
            
            window.title = title.as_ref().to_string();

            let class_name = w!("window");
            let window_class = WNDCLASSW {
                //cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                hInstance: window.instance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                //hbrBackground: HBRUSH(0),
                lpszClassName: class_name,
                ..Default::default()
            };

            let atom = RegisterClassW(&window_class);
            debug_assert!(atom != 0);

            window.handle = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                PCWSTR(wide_null(window.title.clone()).as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                window.instance,
                None,
            );

            let address = Box::as_mut(&mut window) as *mut _ as isize;
            let name: PCWSTR = PCWSTR(wide_null(window.title.clone()).as_ptr());
            SetPropW(window.handle, name, HANDLE(address)).unwrap();
        }

        window
    }

    pub fn destroy(&mut self) {
        unsafe {
            if self.handle.0 != 0 {
                println!("Destroying window");
                let title = String::from("AxonEngine");
                let wide_title = PCWSTR(wide_null(title.clone()).as_ptr());

                RemovePropW(self.handle, wide_title).unwrap();
                DestroyWindow(self.handle).unwrap();
                //PostQuitMessage(0);
                self.handle = HWND(0);
            }
        }
    }

    pub fn poll_events(&mut self) {
        unsafe {
            let mut msg = MSG::default();

            loop {
                let mut got_message: BOOL = FALSE;
                let skip_messages: [u32; 2] = [
                    0x738, // TODO(mdeforge): What message is this again?
                    0xFFFFFFFF // TODO(mdeforge): What message is this again?
                ];

                let mut last_message: u32 = 0;
                for &skip in skip_messages.iter() {
                    got_message = PeekMessageW(&mut msg, None, last_message, skip - 1, PM_REMOVE);

                    if got_message == TRUE {
                        break;
                    }

                    last_message = skip.wrapping_add(1);
                }

                if got_message == FALSE {
                    break;
                }

                match msg.message {
                    WM_QUIT => {
                        println!("Got WM_QUIT message, setting has_requested_close = true");
                        self.has_requested_close = true;
                    },
                    WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                        // TODO(mdeforge): Consider input
                    },
                    _ => {
                        TranslateMessage(&msg);
                        DispatchMessageA(&msg);
                    }
                }
            }
        }
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let title = String::from("AxonEngine");
        let wide_title = PCWSTR(wide_null(title.clone()).as_ptr());

        // Get AxWindow
        let handle = GetPropW(hwnd, wide_title);
        let restored_pointer: *mut AxWindow = handle.0 as *mut AxWindow; 
        if restored_pointer.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        
        let window = &mut *restored_pointer;

        // Match message and return
        match msg {
            WM_CLOSE => {
                println!("WM_CLOSE");
                window.has_requested_close = true;
                //window.destroy();
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                LRESULT(0)
            }
            // WM_PAINT => {
            //     println!("WM_PAINT");
            //     ValidateRect(hwnd, None);
            //     LRESULT(0)
            // }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}