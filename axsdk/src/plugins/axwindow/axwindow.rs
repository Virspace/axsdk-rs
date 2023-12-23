use std::mem;
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

/// Turns a Rust string slice into a null-terminated utf-16 vector.
pub fn wide_null(s: String) -> Vec<u16> {
    s.encode_utf16().collect()
  }

#[derive(Default)]
pub struct AxWindow {
    title: String,
    handle: HWND,
    instance: HMODULE,
    pub has_requested_close: bool
}

impl AxWindow {
    pub fn new<S: AsRef<str>>(title: S) -> Self {
        let mut window = AxWindow::default();
        unsafe {
            window.instance = GetModuleHandleA(None).unwrap();
            window.title = title.as_ref().to_string();
            
            debug_assert!(window.instance.0 != 0);

            let window_class = w!("window");

            let _wc = WNDCLASSW {
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hInstance: window.instance.into(),
                lpszClassName: window_class,
                
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                ..Default::default()
            };

            let atom = RegisterClassW(&_wc);
            debug_assert!(atom != 0);

            window.handle = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
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

            //let address: isize = &window as *const _ as isize;
            let address = &mut window as *mut _ as isize;
            let s = PCWSTR(wide_null(window.title.clone()).as_ptr());
            SetPropW(window.handle, s, HANDLE(address)).unwrap();
            //SetPropW(window.handle, s, HANDLE(123)).unwrap();
            println!("Done creating!");
        }

        window
    }

    // pub fn register(&self) {
    //     unsafe {
    //         SetPropW(self.handle, PCWSTR(wide_null(self.title.clone()).as_ptr()), );
    //     }
    // }

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
        let name = String::from("AxonEngine");
        let title = PCWSTR(wide_null(name.clone()).as_ptr());
        //let window: *mut AxWindow = mem::transmute(GetPropW(hwnd, w!("AxonEngine")));
        let handle = GetPropW(hwnd, title);
        //let address = &mut window as *mut _ as isize;
        let restored_pointer: *mut AxWindow = handle.0 as *mut AxWindow; 
        let window = &mut *restored_pointer;
        //let window: *mut AxWindow = &handle.0 as *const _ as *mut AxWindow;
        if restored_pointer.is_null() {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        match msg {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(hwnd, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CLOSE => {
                println!("WM_CLOSE");
                window.has_requested_close = true;
                LRESULT(0)
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}