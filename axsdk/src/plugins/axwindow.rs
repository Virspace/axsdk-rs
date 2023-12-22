#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::foundation::axtypes::*;
use core::slice;
use std::{mem::size_of, ops::BitOrAssign};
use bitflags::{bitflags, Flags};
use widestring::*;
use std::ffi::CStr;

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

fn loword(l: usize) -> u16 {
    ((l as usize) & 0xffff) as u16
}

fn hiword(l: usize) -> u16 {
    (((l as usize) >> 16) & 0xffff) as u16
}

struct PWSTRWrapper(Vec<u16>);

impl PWSTRWrapper {
    /// Get a raw PCWSTR to the string
    /// While this is possible to do using e.g. PCWSTR::from_raw, these are not marked as
    /// unsafe, and it is easy to accidentally violate the invariants (even if it seems obvious at first).
    /// For that reason, the below function aims to explicitly bring the safety invariants to notice
    /// so we don't accidentally make that mistake. It also lets you convert a string to utf-16,
    /// so it's also convenient in that sense.
    ///
    /// SAFETY:
    /// - You must bind PCWSTRWrapper to a variable, or it'll create a temporary and drop it.
    ///   E.g. `let foo = "bar".to_pcwstr().as_pcwstr()` drops after statement, and
    ///        the raw pointer is dangling
    ///   However, `foo("bar".to_pcwstr().as_pcwstr())` is fine since it'll drop after
    ///   the fn call
    /// - Since this hands out a raw pointer, it can easily escape the lifetime of PCWSTRWrapper.
    ///   Ensure you or the function you called does not use the PCWSTR after PCWSTRWrapper gets dropped
    unsafe fn as_pwstr(&self) -> PWSTR {
        PWSTR::from_raw(self.0.as_ptr())
    }

    fn new<T: AsRef<str>>(text: T) -> Self {
        let text = text.as_ref();
        // do not drop when scope ends, by moving it into struct
        let mut text = text.encode_utf16().collect::<Vec<_>>();
        text.push(0);

        Self(text)
    }
}

trait ToPWSTRWrapper {
    fn to_pwstr(&self) -> PWSTRWrapper;
}

impl ToPWSTRWrapper for &str {
    fn to_pwstr(&self) -> PWSTRWrapper {
        PWSTRWrapper::new(self)
    }
}

impl ToPWSTRWrapper for String {
    fn to_pwstr(&self) -> PWSTRWrapper {
        PWSTRWrapper::new(self)
    }
}


// Structs ////////////////////////////////////////////////////

struct AxWindowContext {
    Major: i32,
    Minor: i32
}

struct AxWindowHints {
    Context: AxWindowContext
    //struct AxWindowConfig Config;
}

// TODO(mdeforge): Is this even used??
#[cfg(windows)]
struct AxMonitorPlatformData {
  //HMONITOR Handle;      // A handle to the physical display
  adapter_name: String,
  display_name: String,
  HasModesPruned: bool
}

//#[cfg(unix)]
// TODO(mdeforge): Need linux version
// struct AxMonitorPlatformData {
//   HMONITOR Handle;      // A handle to the physical display
//   AdapterName [32];
//   CHAR DisplayName[32];
//   bool HasModesPruned;
// }

// An opaque object that represents a display.
//typedef struct AxDisplay AxDisplay;

#[cfg(windows)]
struct AxWindowData {
    /// A handle to a window associated with a particualr module instance.
    handle: HWND,
    /// The module instance the window belongs to (a particular EXE or DLL).
    instance: HINSTANCE,
    ///
    cursor_in_window: bool,
    ///
    last_cursor_pos: AxVec2
}

#[cfg(unix)]
struct AxWindowData {
    // TODO(mdeforge)
}

impl AxWindowData {
    pub fn new() -> Self {
        AxWindow::default()
    }

    pub fn default() -> Self {
        AxWindowData { handle: 0, instance: 0, cursor_in_window: false, last_cursor_pos: AxVec2::default() }
    }
}

// Enums //////////////////////////////////////////////////////

// Window style flags
bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct AxWindowStyle: u32 {
        const AX_WINDOW_STYLE_VISIBLE              = 1 << 0;
        const AX_WINDOW_STYLE_CENTERED             = 1 << 1;
        const AX_WINDOW_STYLE_DECORATED            = 1 << 2;
        const AX_WINDOW_STYLE_RESIZABLE            = 1 << 3;
        const AX_WINDOW_STYLE_LOCKASPECT           = 1 << 4;
        const AX_WINDOW_STYLE_MAXIMIZED            = 1 << 5;
        const AX_WINDOW_STYLE_FULLSCREEN           = 1 << 6; // Fullscreen Borderless
    }
}

// MessageBox button flags
enum AxMessageBoxResponse {
    AX_MESSAGEBOX_RESPONSE_OK            = 1,
    AX_MESSAGEBOX_RESPONSE_CANCEL        = 2,
    AX_MESSAGEBOX_RESPONSE_ABORT         = 3,
    AX_MESSAGEBOX_RESPONSE_RETRY         = 4,
    AX_MESSAGEBOX_RESPONSE_IGNORE        = 5,
    AX_MESSAGEBOX_RESPONSE_YES           = 6,
    AX_MESSAGEBOX_RESPONSE_NO            = 7,
    AX_MESSAGEBOX_RESPONSE_TRYAGAIN      = 10,
    AX_MESSAGEBOX_RESPONSE_CONTINUE      = 11
}

impl AxMessageBoxResponse {
    fn from_i32(value: i32) -> AxMessageBoxResponse {
        match value {
            1 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_OK,
            2 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_CANCEL,
            3 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_ABORT,
            4 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_RETRY,
            5 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_IGNORE,
            6 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_YES,
            7 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_NO,
            10 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_TRYAGAIN,
            11 => AxMessageBoxResponse::AX_MESSAGEBOX_RESPONSE_CONTINUE,
            _ => panic!("Unknown AxMessageBoxReponse value: {}", value)
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct AxMessageBoxFlags : u32 {
        const AX_MESSAGEBOX_TYPE_ABORTRETRYIGNORE  = 1 << 0;
        const AX_MESSAGEBOX_TYPE_CANCELTRYCONTINUE = 1 << 1;
        const AX_MESSAGEBOX_TYPE_HELP              = 1 << 2;
        const AX_MESSAGEBOX_TYPE_OK                = 1 << 3;
        const AX_MESSAGEBOX_TYPE_OKCANCEL          = 1 << 4;
        const AX_MESSAGEBOX_TYPE_RETRYCANCEL       = 1 << 5;
        const AX_MESSAGEBOX_TYPE_YESNO             = 1 << 6;
        const AX_MESSAGEBOX_TYPE_YESNOCANCEL       = 1 << 7;
        const AX_MESSAGEBOX_ICON_EXCLAMATION       = 1 << 8;
        const AX_MESSAGEBOX_ICON_WARNING           = 1 << 9;
        const AX_MESSAGEBOX_ICON_INFORMATION       = 1 << 10;
        const AX_MESSAGEBOX_ICON_QUESTION          = 1 << 11;
        const AX_MESSAGEBOX_ICON_STOP              = 1 << 12;
        const AX_MESSAGEBOX_ICON_ERROR             = 1 << 13;
        const AX_MESSAGEBOX_DEFBUTTON1             = 1 << 14;
        const AX_MESSAGEBOX_DEFBUTTON2             = 1 << 15;
        const AX_MESSAGEBOX_DEFBUTTON3             = 1 << 16;
        const AX_MESSAGEBOX_DEFBUTTON4             = 1 << 17;
        const AX_MESSAGEBOX_APPLMODAL              = 1 << 18;
        const AX_MESSAGEBOX_SYSTEMMODAL            = 1 << 19;
        const AX_MESSAGEBOX_TASKMODAL              = 1 << 20;    }
}

// Key state flags
enum AxKeyState {
    AX_KEY_PRESSED,
    AX_KEY_RELEASED
}

// Cursor modes
#[derive(PartialEq)]
enum AxCursorMode {
    /// Applies pointer acceleration (ballistics) to a visible active cursor and limits movement to screen resolution.
    AX_CURSOR_NORMAL,
    /// Applies pointer acceleration (ballistics) to a hidden active cursor and limits movement to screen resolution.
    AX_CURSOR_HIDDEN,
    /// Hides the active cursor and locks it to the window, a virtual cursor position is provided.
    AX_CURSOR_DISABLED
}

// Keyboard modes
enum AxKeyboardMode {
    AX_KEYBOARD_ENABLED,
    AX_KEYBOARD_DISABLED
}

fn Win32RegisterWindowClass() -> bool {
    let window_class = s!("AXONENGINE");
    
    // TODO(mdeforge): Load user provided icon if available
    unsafe {
        let instance = GetModuleHandleA(None).unwrap();
        let wnd_class = WNDCLASSEXA {
            cbSize: size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(Win32MainWindowCallback),
            hInstance: HINSTANCE::from(instance),
            hCursor: LoadCursorA(HINSTANCE(0), windows::core::PCSTR(32512u16 as _)),
            hbrBackground: HBRUSH(0), // Setting this to ANYTHING causes black window while resizin,
            //hbrBackground: (HBRUSH)(COLOR_WINDOW + 1); // Using this doesn't work with DPI change,
            lpszClassName: window_class,
            lpszMenuName: PCSTR(std::ptr::null()),
            ..Default::default()
        };

        let atom = RegisterClassExA(&wnd_class);
        match atom {
            0 => {
                MessageBoxA(None, s!("Window Registration Failed!"), s!("Abandon Ship!"), MB_ICONEXCLAMATION | MB_OK);
                return false;
            },
            _ => { return true }
        }
    }
}

fn Win32MainWindowCallback(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT
    {
        let window: AxWindow = unsafe { GetPropA(hwnd, s!("AxonEngine")) };
        if !window {
            return unsafe { DefWindowProcA(hwnd, message, wparam, lparam) };
        }

        let mut result: LRESULT = 0;

        match message
        {
            WM_CLOSE => window.is_requesting_close = true,

            // case WM_SIZE:
            // {
            //     // if (Window)
            //     // {
            //     //     RECT ClipRect;
            //     //     GetWindowRect((HWND)Window->Platform.Win32.Handle, &ClipRect);
            //     //     ClientToScreen((HWND)Window->Platform.Win32.Handle, (POINT *) &ClipRect.left);
            //     //     ClientToScreen((HWND)Window->Platform.Win32.Handle, (POINT *) &ClipRect.right);
            //     //     ClipCursor(&ClipRect);
            //     // }

            //     return (0);
            // }
            // NOTE(mdeforge): This takes the place of WM_MOVE and WM_SIZE
            // https://devblogs.microsoft.com/oldnewthing/20080115-00/?p=23813
            // https://devblogs.microsoft.com/oldnewthing/20080116-00/?p=23803
            WM_WINDOWPOSCHANGING => {
                let mut rect: RECT;
                unsafe { GetWindowRect(hwnd, &rect) };

                // TODO(mdeforge): If resizing is now allowed, enforce here

                window.rect = rect;
                result = unsafe { DefWindowProcA(hwnd, message, wparam, lparam) };
            },
            WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP =>
            {
                let mut scan_code: i32 = 0;

                let mut state: AxKeyState = 0;
                if hiword(lparam) & KF_UP {
                    state = AxKeyState::AxKeys::AX_KEY_RELEASED;
                } else {
                    state = AxKeyState::AxKeys::AX_KEY_PRESSED
                }

                let mods: AxKeyModifier = GetKeyModifiers();

                scan_code = hiword(lparam) & (KF_EXTENDED | 0xff);
                if !scan_code {
                    scan_code = unsafe { MapVirtualKeyA(wparam, MAPVK_VK_TO_VSC) };
                }
            },
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN |
            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => {
                let mut button = 0;
                let mut action = 0;

                // XBUTTON1 and XBUTTON2 are often located on the sides of the mouse, near the base.
                if message == WM_LBUTTONDOWN || message == WM_LBUTTONUP {
                    button = AxMouse::AX_MOUSE_BUTTON_LEFT;
                } else if message == WM_RBUTTONDOWN || message == WM_RBUTTONUP {
                    button = AxMouse::AX_MOUSE_BUTTON_RIGHT;
                } else if message == WM_MBUTTONDOWN || message == WM_MBUTTONUP {
                    button = AxMouse::AX_MOUSE_BUTTON_MIDDLE;
                } else if hiword(wparam) == XBUTTON1 {
                    button = AxMouse::AX_MOUSE_BUTTON_4;
                } else {
                    button = AxMouse::AX_MOUSE_BUTTON_5;
                }

                if message == WM_LBUTTONDOWN || message == WM_RBUTTONDOWN ||
                message == WM_MBUTTONDOWN || message == WM_XBUTTONDOWN {
                    action = AX_PRESS;
                } else {
                    action = AX_RELEASE;
                }

                // TODO(mdeforge): More detection needed here, sticky keys? caps lock?
                // TODO(mdeforge): Click callback?
                window.mouse_buttons[button] = action;

                if message == WM_XBUTTONDOWN || message == WM_XBUTTONUP {
                    return true;
                }

                return false;
            },

            // NOTE(mdeforge): WM_MOUSEMOVE is only received when the mouse moves INSIDE the window OR while "captured"
            // https://docs.microsoft.com/en-us/windows/win32/learnwin32/mouse-movement
            // NOTE(mdeforge): WM_MOUSEMOVE should be used for GUI's, while WM_INPUT should be used otherwise
            // https://docs.microsoft.com/en-us/windows/win32/dxtecharts/taking-advantage-of-high-dpi-mouse-movement
            WM_MOUSEMOVE => self
                .mouse_move_handler(wparam, lparam)
                .expect("WM_MOUSEMOVE")
            ,
            WM_INPUT => {
                // If the cursor mode is normal or hidden, and the keyboard is disabled, break. This section is for raw input.
                if !window.cursor_mode == AxCursorMode::AX_CURSOR_DISABLED &&
                    window.keyboard_mode == AxKeyboardMode::AxKeys::AX_KEYBOARD_DISABLED {
                }

                let raw_input_size: u32 = 0;
                let mut raw_input_data: [BYTE; size_of::<RAWINPUT>()];

                let get_raw_input_data_result = unsafe {
                    GetRawInputData(lparam, RID_INPUT, raw_input_data, &raw_input_size, size_of::<RAWINPUTHEADER>())
                };

                if !get_raw_input_data_result {
                    return;
                }

                let MouseDelta: AxVec2 = AxVec2::default();
                let raw_input: *mut RAWINPUT = raw_input_data;

                if raw_input.header.dwType == RIM_TYPEMOUSE {
                    if (raw_input.data.mouse.usFlags & MOUSE_MOVE_ABSOLUTE) {
                        MouseDelta = AxVec2::new(
                            raw_input.data.mouse.lLastX - window.platform.last_cursor_pos.x,
                            raw_input.data.mouse.lLastY - window.platform.last_cursor_pos.y
                        );
                    } else {
                        MouseDelta = AxVec2::new(
                            raw_input.data.mouse.lLastX,
                            raw_input.data.mouse.lLastY
                        )
                    }
                } else if (raw_input.header.dwType == RIM_TYPEKEYBOARD) {
                    // NOTE(mdeforge): https://blog.molecular-matters.com/2011/09/05/properly-handling-keyboard-input/
                    let raw_keyboard: RAWKEYBOARD = raw_input.data.keyboard;

                    let mut virtual_key: u32 = raw_keyboard.VKey;
                    let mut scan_code: u32 = raw_keyboard.MakeCode;
                    let FLAGS: u32 = raw_keyboard.Flags;

                    if virtual_key == 255 {
                        // Discard fake keys that are a part of an escape sequence
                    } else if virtual_key == VK_SHIFT {
                        // Correct left-hand/right-hand SHIFT
                        virtual_key = unsafe { MapVirtualKeyA(scan_code, MAPVK_VSC_TO_VK_EX) };
                    } else if virtual_key == VK_NUMLOCK {
                        // Correct pause/break and num lock issues, and set the extra extended bit
                        scan_code = unsafe { MapVirtualKeyA(virtual_key, MAPVK_VK_TO_VSC) | 0x100 };
                    }

                    let IS_ESC_SEQ_0: bool = (FLAGS.as_dynamic_ & RI_KEY_E0) != 0;
                    let IS_ESC_SEQ_1: bool = (FLAGS & RI_KEY_E1) != 0;
                    let WAS_UP: bool = (FLAGS & RI_KEY_BREAK) != 0;

                    if IS_ESC_SEQ_0 {
                        // For escaped sequences, turn the virtual key into the correct scan code using MapVirtualKey.
                        // However, MapVirtualKey is unable to map VK_PAUSE (known bug), so we map it by hand.
                        if virtual_key == VK_PAUSE {
                            scan_code = 0x45;
                        } else {
                            scan_code = unsafe { MapVirtualKeyA(virtual_key, MAPVK_VK_TO_VSC) };
                        }
                    }

                    match virtual_key {
                        // Right-hand CONTROL and ALT have their E0 bit set
                        VK_CONTROL => {
                            if IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_RIGHT_CTRL;
                            } else {
                                virtual_key = AxKeys::AX_KEY_LEFT_CTRL;
                            }
                        },
                        VK_MENU => {
                            if IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_RIGHT_ALT;
                            } else {
                                virtual_key = AxKeys::AX_KEY_LEFT_ALT;
                            }
                        },
                        // NUMPAD ENTER has its E0 bit set
                        VK_RETURN => {
                            if IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_ENTER;
                            }
                        },
                        // The standard INSERT, DELETE, HOME, END, PRIOR, and NEXT keys will always have the E0 bit set
                        // but the same keys on the numpad will not.
                        VK_INSERT => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_0;
                            }
                        },
                        VK_DELETE => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_DECIMAL;
                            }
                        },
                        VK_HOME => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_7;
                            }
                        },
                        VK_END => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_1;
                            }
                        },
                        VK_PRIOR => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_9;
                            }
                        },
                        VK_NEXT => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_3;
                            }
                        }
                        // The standard arrow keys will always have the E0 bit set, but the same keys on the numpad will not.
                        VK_UP => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_8;
                            }
                        },
                        VK_DOWN => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_2;
                            }
                        }
                        VK_LEFT => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_4;
                            }
                        },
                        VK_RIGHT => {
                            if !IS_ESC_SEQ_0 {
                                virtual_key = AxKeys::AX_KEY_NUMPAD_6;
                            }
                        }
                    }
                }

                window.virtual_cursor_pos = Vec2Add(window.virtual_cursor_pos, MouseDelta);
                window.platform.last_cursor_pos = Vec2Add(window.platform.last_cursor_pos, MouseDelta);
            },
            WM_MOUSELEAVE => {
                window.platform.cursor_in_window = false;
            },
            // Only useful if we need to scale the window non-linearly
            // case WM_GETDPISCALEDSIZE:
            // {
            //     RECT Source = {0}, Target = {0};
            //     SIZE *Size = (SIZE *)LParam;

            //     u32 DPI = GetNearestMonitorDPI(Window);
            //     DWORD Style = GetWindowStyle(Window);
            //     AdjustWindowRectExForDpi(&Source, Style, FALSE, 0, DPI);

            //     AdjustWindowRectExForDpi(&Target, Style, FALSE, 0, loword(WParam));

            //     Size->cx += (Target.right - Target.left) -
            //                 (Source.right - Source.left);
            //     Size->cy += (Target.bottom - Target.top) -
            //                 (Source.bottom - Source.top);

            //     return (TRUE);
            // };

            // NOTE(mdeforge): When this gets triggered, it's because the DPI
            // has changed at runtime either because the user has moved to a new
            // monitor with a different DPI or the DPI of the monitor has changed.
            // The LParam contains a pointer to a RECT that provides a new suggested
            // size and position of the window for the new DPI which is why we don't
            // need to call AdjustWindowRectExForDpi ourselves.
            // https://docs.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged
            WM_DPICHANGED => self
                .dpi_changed_handler(wparam, lparam)
                .expect("WM_DPICHANGED")
            ,
            // NOTE(mdeforge): Not sure we're going to be polling displays anymore
            // The display resolution has changed
            // case WM_DISPLAYCHANGE:
            // {
            //     PollDisplays();
            //     break;
            // }

            // NOTE(mdeforge): Warning, using this prevents the resize cursor
            WM_SETCURSOR => {
                if loword(lparam) == HTCLIENT {
                    window.UpdateCursorImage();
                }
            },
            _ => {
                result = unsafe { DefWindowProcA(hwnd, message, wparam, lparam) };
            }
        }

        return result;
    }


fn GetKeyModifiers() -> AxKeyModifier {
    let mut mods: AxKeyModifier = 0;

    if unsafe { GetKeyState(VK_CONTROL) & 0x8000 } {
      mods |= AxKeyModifier::AxKeyCtrl;
    }

    if unsafe { GetKeyState(VK_MENU) & 0x8000 } {
      mods |= AxKeyModifier::AxKeyAlt;
    }

    if unsafe { GetKeyState(VK_SHIFT) & 0x8000 } {
      mods |= AxKeyModifier::AxKeyShift;
    }

    if unsafe { GetKeyState(VK_LWIN) } | unsafe { GetKeyState(VK_RWIN) & 0x8000 } {
      mods |= AxKeyModifier::AxKeyWin;
    }

    if unsafe { GetKeyState(VK_CAPITAL) & 0x8000 } {
      mods |= AxKeyModifier::AxKeyCaps;
    }

    return mods;
}

// Public API /////////////////////////////////////////////////

pub struct AxWindow
{
    /// Window title
    title: String,
    /// User has requested close
    is_requesting_close: bool,
    /// Size and Position
    rect: RECT,
    /// Platform specific data
    platform: AxWindowData,
    /// Window style flags
    style: AxWindowStyle,
    /// Cursor modes
    cursor_mode: AxCursorMode,
    /// Keyboard modes
    keyboard_mode: AxKeyboardMode,
    /// Virtual cursor position
    virtual_cursor_pos: AxVec2,
    /// Mouse button state
    mouse_buttons: [i32; AxMouse::AX_MOUSE_BUTTON_LAST as usize]
}

impl AxWindow
{
    fn init() {
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        Win32RegisterWindowClass();
    }

    /// Returns a window with the given properties.
    ///
    /// # Arguments
    /// * `title` - The window title that shows in the title bar.
    /// * `x` - The horizontal position of the window.
    /// * `y` - The vertical position of the window.
    /// * `width` - The width of the window.
    /// * `height` - The height of the window.
    /// * `style_flags` - A bitmask of AxWindowStyle.
    pub fn create(&self, title: String, x: i32, y: i32, width: i32, height: i32, style: AxWindowStyle ) -> Result<AxWindow> {
        if title.is_empty() {
            panic!("Tried to create a window with an empty title!");
        }

        if width <= 0 || height <= 0 {
            panic!("Tried to create a window with a width or height less than or equal to zero!");
        }

        // TODO(mdeforge): Technically, this only needs to be called once regardless of number of windows
        self.init();

        let rect = RECT { left: x, top: y, right: x + width, bottom: y + height };

        let window = AxWindow::default();
        window.platform.instance = unsafe { GetModuleHandleA(None)? };
        window.is_requesting_close = false;
        window.title = title;
        window.rect = rect;
        window.style = style;

        if !self.create_native_window() {
            unsafe {
                DestroyWindow(window);
            }

            panic!("Failed to create native window!");
        }

        if style.contains(AxWindowStyle::AX_WINDOW_STYLE_FULLSCREEN) || style.contains(AxWindowStyle::AX_WINDOW_STYLE_VISIBLE) {
            unsafe {
                ShowWindow(window.platform.handle, SW_SHOW);
                SetFocus(self.platform.handle);
                UpdateWindow(window.platform.handle);
            }
        }

        return window;
    }

    /**
     * @brief Destroys the target window.
     * @param Window The window to be destroyed.
     */
    pub fn destroy(&self) {
        if self.platform.handle {
            unsafe {
                RemovePropA(self.platform.handle, "AxonEngine");
                DestroyWindow(self.platform.handle);
            }

            self.platform.handle = 0;
        }
    }

    /**
     * @brief Polls the events of the target window.
     * @param Window The target window.
     */
    pub fn PollEvents(window: &AxWindow) {
        let mut msg: MSG = MSG::default();

        loop
        {
            let mut got_message = false;
            let skip_messages: [u32; 2] = [
                0x738, // TODO(mdeforge): What message is this again?
                0xFFFFFFFF // TODO(mdeforge): What message is this again?
            ];

            let last_message: u32 = 0;
            for &skip in skip_messages.iter() {
                got_message = unsafe {
                    PeekMessage(&mut msg, 0, last_message, skip - 1, PM_REMOVE)
                };

                if !got_message {
                    break;
                }

                last_message = skip + 1;
            }

            if !got_message {
                break;
            }

            match msg.message {
                WM_QUIT => {
                    window.is_requesting_close = true;
                },
                WM_SYSKEYDOWN | WM_SYSKEYUP | WM_KEYDOWN | WM_KEYUP => {
                    // TODO(mdeforge): Consider input
                },
                _ => {
                    unsafe {
                        TranslateMessage(&msg);
                        DispatchMessageA(&msg);
                    }
                }
            }
        }
    }

    /**
     * @brief Checks if the target window has requested to close.
     * @param Window The target window.
     * @return True if the target window has requested a close, otherwise false.
     */
    pub fn has_requested_close(&self) -> bool {
        self.is_requesting_close
    }

    /**
     * @brief Gets the position of the target window.
     * @param Window The target window.
     * @param X The X position of the target window.
     * @param Y The Y position of the target window.
     */
    pub fn get_position(&self) -> AxVec2 {
        let mut rect = AxRect::default();
        unsafe { GetClientRect(self.platform.handle, &rect) };

        rect.position()
    }

    /**
     * @brief Sets the position of the target window.
     * @param Window The target window.
     * @param X The desired X position of the target window.
     * @param Y The desired Y position of the target window.
     */
    pub fn set_position(&self, pos: AxVec2) {
        if self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_FULLSCREEN) {
            return;
        }

        let rect = AxRect::new(pos.x, pos.y, 0.0, 0.0);
        let DPI: u32 = self.get_nearest_monitor_dpi();

        // Adjust size to account for non-client area
        let window_style: u32 = self.get_window_style();
        unsafe {
            AdjustWindowRectExForDpi(&self.rect, WINDOW_STYLE(window_style), false, WINDOW_EX_STYLE(0), DPI);
            SetWindowPos(self.platform.handle, None,
                rect.left, rect.top, 0, 0,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE);
        }

        self.rect = rect;
    }

    /**
     * @brief Gets the size of the target window.
     * @param Window The target window.
     * @param Width The Width of the target window.
     * @param Height The Height of the target window.
     */
    pub fn get_window_size(window: &AxWindow) -> AxVec2 {
        let mut rect = AxRect::default();
        unsafe { GetClientRect(window.platform.handle, &rect) };

        return AxVec2::new(rect.width(), rect.height());
    }

    /**
     * @brief Sets the size of the target window.
     * @param Window The target window.
     * @param Width The desired width of the target window.
     * @param Height The desired height of the target window.
     */
    pub fn SetWindowSize(window: &AxWindow, width: i32, height: i32) {

    }

    /**
     * @brief Sets the visibility of the target window.
     * @param Window The target window.
     * @param IsVisible The desired visibility of the target window.
     */
    pub fn set_window_visibility(window: &mut AxWindow, visibility: bool) {
        if visibility {
            unsafe { ShowWindow(window.platform.handle, SW_SHOW); }
        } else {
            unsafe { ShowWindow(window.platform.handle, SW_HIDE); }
        }
    }

    /**
     * @brief Returns platform window data
     * @param Window The target window.
     * @return The platform data for the target window if target is valid, otherwise a zero-initialized AxWindowData struct.
     */
    pub fn GetPlatformData(&self) -> AxWindowData {
        self.platform
    }

    /**
     * @brief Gets the mouse coordinates of the target window.
     * @param Window The target window.
     * @param Position To be filled out by GetMouseCoords.
     */
    pub fn get_mouse_coords(&self) -> AxVec2 {
        // TODO(mdeforge): Does cursor mode have an effect on this?
        // if (Window->CursorMode == AX_CURSOR_DISABLED)
        // {
        //     if (Position) {
                self.virtual_cursor_pos
        //     }
        // }
    }

    /**
     * @brief Gets the state of the target mouse button on the target window.
     * @param Window The target window.
     * @param Button The target button.
     * @return An integer representing the AxKeyState of the button.
     */
    pub fn get_mouse_button(&self, button: i32) -> i32 {
        if button < AxMouse::AX_MOUSE_BUTTON_1 || button > AxMouse::AX_MOUSE_BUTTON_LAST {
            return AX_RELEASE;
        }

        self.mouse_buttons[button as usize]
    }

    /**
     * @brief Sets the cursor mode of the target window.
     * @param Window The target window.
     * @param CursorMode The desired AxCursorMode.
     */
    // TODO(mdeforge): Update cursor image using enable/disable cursor functions
    pub fn set_cursor_mode(&self, cursor_mode: AxCursorMode) {
        self.cursor_mode = cursor_mode;
        self.UpdateCursorImage();
    }

    /**
     * @brief Gets the cursor mode of the target window.
     * @param Window The target window.
     */
    pub fn get_cursor_mode(window: AxWindow) -> AxCursorMode {
        window.cursor_mode
    }

    /**
     * @brief Sets the keyboard mode of the target window.
     * @param Window The target window.
     * @param KeyboardMode The desired KeyboardMode of the target window.
     */
    pub fn set_keyboard_mode(&self, keyboard_mode: AxKeyboardMode) {
        self.keyboard_mode = keyboard_mode;
    }

    // /**
    //  * @brief Enables the cursor on the target window.
    //  * @param The target window.
    //  */
    // void (*EnableCursor)(const AxWindow *Window);

    // /**
    //  * @brief Disables the cursor on the target window.
    //  * @param The target window.
    //  */
    // void (*DisableCursor)(const AxWindow *Window);

    /**
     * @brief Opens a File Open dialog.
     * @param Window A handle to the owner window of the dialog to be created. If this parameter is NULL, the dialog will no owner window.
     * @param Title The title of the dialog box, NULL sets default.
     * @param Filter The file types to filter on, e.g. "Supported Files(*.ms, *.txt, *.cpp, *.h)\0*.ms;*.txt;*.cpp;*.h\0";
     * @param IntialDirectory Sets the initial directory to open the file open dialog in, can be NULL.
     * @param FileName Pointer to character array that returns filename. Setting it sets default filename, otherwise should be zero-initialized.
     * @param FileNameSize Size of zero-initialized character array for filename.
     */
    pub fn open_file_dialog(&self, mut title: String, mut filter: String, mut initial_directory: String) -> Option<String> {
        let mut filename: [u8; 256] = [0; 256]; // TODO(mdeforge): MAX_PATH?

        let mut open_file_name = OPENFILENAMEA::default();
        open_file_name.lStructSize = size_of::<OPENFILENAMEA>() as u32;
        open_file_name.hwndOwner = self.platform.handle;
        open_file_name.lpstrFile = PSTR::from_raw(filename.as_mut_ptr());
        open_file_name.nMaxFile = filename.len() as u32;

        if !filter.is_empty() {
            open_file_name.lpstrFilter = PCSTR::from_raw(filter.as_mut_ptr());
        } else {
            open_file_name.lpstrFilter = s!("All files\0*.*\0\0");
        }

        open_file_name.nFilterIndex = 1;
        open_file_name.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;
        open_file_name.lpstrInitialDir = PCSTR::from_raw(initial_directory.as_mut_ptr());

        if !title.is_empty() {
            open_file_name.lpstrTitle = PCSTR::from_raw(title.as_mut_ptr());
        } else {
            open_file_name.lpstrTitle = s!("Open File");
        }

        if unsafe { GetOpenFileNameA(&mut open_file_name).into() } {
            // Get PSTR
            let pstr = open_file_name.lpstrFile.as_ptr() as *const i8;

            // Convert PSTR to a Rust String
            let c_str = unsafe { CStr::from_ptr(pstr) };
            let bytes = c_str.to_bytes();
            let rust_string = String::from_utf8_lossy(bytes).into_owned();

            Some(rust_string)
        } else {
            None
        }
    }

    /**
     * @brief Opens a File Save dialog.
     * @param Window A handle to the owner window of the dialog to be created. If this parameter is NULL, the dialog will no owner window.
     * @param Title The title of the dialog box, NULL sets default.
     * @param Filter The file types to filter on, e.g. "Supported Files(*.ms, *.txt, *.cpp, *.h)\0*.ms;*.txt;*.cpp;*.h\0";
     * @param IntialDirectory Sets the initial directory to open the file save dialog in, can be NULL.
     * @param FileName Pointer to character array that returns filename. Setting it sets default filename, otherwise should be zero-initialized.
     * @param FileNameSize Size of zero-initialized character array for filename.
     */
    pub fn SaveFileDialog(&self, mut title: String, mut filter: String, mut initial_directory: String, filename: String) -> Option<String> {
        let mut filename: [u8; 256] = [0; 256]; // TODO(mdeforge): MAX_PATH?

        let mut open_file_name = OPENFILENAMEA::default();
        open_file_name.lStructSize = size_of::<OPENFILENAMEA>() as u32;
        open_file_name.hwndOwner = self.platform.handle;
        open_file_name.lpstrFile = PSTR::from_raw(filename.as_mut_ptr());
        open_file_name.nMaxFile = filename.len() as u32;

        if !filter.is_empty() {
            open_file_name.lpstrFilter = PCSTR::from_raw(filter.as_mut_ptr());
        } else {
            open_file_name.lpstrFilter = s!("All files\0*.*\0\0");
        }

        open_file_name.nFilterIndex = 1;
        open_file_name.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;
        open_file_name.lpstrInitialDir = PCSTR::from_raw(initial_directory.as_mut_ptr());

        if !title.is_empty() {
            open_file_name.lpstrTitle = PCSTR::from_raw(title.as_mut_ptr());
        } else {
            open_file_name.lpstrFilter = s!("Save File");
        }

        if unsafe { GetSaveFileNameA(&mut open_file_name) == true } {
            // Get PSTR
            let pstr = open_file_name.lpstrFile.as_ptr() as *const i8;

            // Convert PSTR to a Rust String
            let c_str = unsafe { CStr::from_ptr(pstr) };
            let bytes = c_str.to_bytes();
            let rust_string = String::from_utf8_lossy(bytes).into_owned();

            Some(rust_string)
        } else {
            None
        }

    }

    /**
     * @brief Opens a basic File Open dialog.
     * @param Window A handle to the owner window of the dialog to be created. If this parameter is NULL, the dialog will no owner window.
     * @param Message The message above the file tree, can be NULL.
     * @param InitialDirectory Sets the initial directory for the open folder dialog, can be NULL.
     * @param FolderName Pointer to character array that returns folder name.
     * @param FolderNameSize Size of zero-initialized character array for folder name.
     */
    // pub fn OpenFolderDialog(&self, message: String, initial_directory: String, folder_name: String) -> bool {
    //     let mut browse_info: BROWSEINFOW;

    //     browse_info.hwndOwner = self.platform.Handle;
    //     browse_info.pszDisplayName = folder_name.to_string();
    //     browse_info.pidlRoot = 0;

    //     if message.is_empty() {
    //         browse_info.lpszTitle = "Open Folder";
    //     } else {
    //        browse_info.lpszTitle = message;
    //     }

    //     browse_info.ulFlags = BIF_NEWDIALOGSTYLE;
    //     browse_info.lParam = initial_directory;
    //     browse_info.lpfn = BrowseCallbackProc;

    //     let IDL: LPITEMIDLIST = unsafe { SHBrowseForFolderW(&browse_info) };
    //     if IDL != 0 {
    //         unsafe { SHGetPathFromIDListEx(IDL, folder_name.as_str(), folder_name.len(), 0) };
    //         return true;
    //     }

    //     return false;
    // }

    /**
     * @brief Opens a Message Box.
     * @param Window A handle to the owner window of the message box to be created. If this parameter is NULL, the message box has no owner window.
     * @param Message The message above the file tree, can be NULL.
     * @param Title The title of the dialog box, NULL sets default.
     * @param Type The contents and behavior of the dialog box set by a combination of AxMessageBoxFlags flags.
     */
    pub fn CreateMessageBox(&self, message: String, title: String, box_type: AxMessageBoxFlags) -> AxMessageBoxResponse {
        let mut flags = MESSAGEBOX_STYLE::default();
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_ABORTRETRYIGNORE) { flags |= MB_ABORTRETRYIGNORE; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_CANCELTRYCONTINUE) { flags |= MB_CANCELTRYCONTINUE; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_HELP) { flags |= MB_HELP; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_OK) { flags |= MB_OK; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_OKCANCEL) { flags |= MB_OKCANCEL; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_RETRYCANCEL) { flags |= MB_RETRYCANCEL; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_YESNO) { flags |= MB_YESNO; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TYPE_YESNOCANCEL) { flags |= MB_YESNOCANCEL; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_ICON_EXCLAMATION) { flags |= MB_ICONEXCLAMATION; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_ICON_WARNING) { flags |= MB_ICONWARNING; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_ICON_INFORMATION) { flags |= MB_ICONINFORMATION; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_ICON_QUESTION) { flags |= MB_ICONQUESTION; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_ICON_STOP) { flags |= MB_ICONSTOP; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_ICON_ERROR) { flags |= MB_ICONERROR; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_DEFBUTTON1) { flags |= MB_DEFBUTTON1; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_DEFBUTTON2) { flags |= MB_DEFBUTTON2; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_DEFBUTTON3) { flags |= MB_DEFBUTTON3; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_DEFBUTTON4) { flags |= MB_DEFBUTTON4; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_APPLMODAL) { flags |= MB_APPLMODAL; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_SYSTEMMODAL) { flags |= MB_SYSTEMMODAL; }
        if box_type.contains(AxMessageBoxFlags::AX_MESSAGEBOX_TASKMODAL) { flags |= MB_TASKMODAL; }

        let rv: MESSAGEBOX_RESULT = unsafe {
            MessageBoxA(
                self.platform.handle,
                PCSTR::from_raw(message.as_mut_ptr()),
                PCSTR::from_raw(title.as_mut_ptr()),
                flags
            )
        };

        AxMessageBoxResponse::from_i32(rv.0)
    }

    // Gets the DPI of the monitor the window is currently mostly on
    // NOTE(mdeforge): Basically does what GetDpiForWindow() does, but is compatible back to Windows 8.1
    fn get_nearest_monitor_dpi(&self) -> u32 {
        unsafe {
            let monitor = MonitorFromWindow(self.platform.handle, MONITOR_DEFAULTTONEAREST);
            let mut dpi = (0, 0);
            let result = GetDpiForMonitor(monitor, MONITOR_DPI_TYPE(0), &mut dpi.0, &mut dpi.1);
            match result {
                Ok(()) => dpi.0,
                Err(err) => USER_DEFAULT_SCREEN_DPI
            }
        }
    }

    fn create_native_window(&self) -> bool {
        let mut style: u32 = self.get_window_style();

        if self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_FULLSCREEN) || self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_MAXIMIZED) {
            style |= WS_MAXIMIZE;
        }

        let DPI: u32 = self.get_nearest_monitor_dpi();
        unsafe { AdjustWindowRectExForDpi(&self.rect, style, false, 0, DPI) };

        // TODO(mdeforge): If centered flag, calculate display center
        let instance = unsafe { GetModuleHandleA(None)? };
        let handle: HWND = unsafe {
            CreateWindowExA(
    None, //WS_EX_TOPMOST | WS_EX_LAYERED,
    PCSTR::from_raw(AXON_WNDCLASSNAME.as_mut_ptr()),
    PCSTR::from_raw(self.title.as_mut_ptr()),
    style,
    self.rect.left,
    self.rect.top,
    self.rect.right - self.rect.left,
    self.rect.top - self.rect.bottom,
    None,
    None,
    instance,
    None
            )
        };

        if !handle {
            let error: u32 = unsafe { GetLastError() };
            unsafe { MessageBoxA(None, "Window Creation Failed!", "Abandon Ship!", MB_ICONEXCLAMATION | MB_OK) };

            return false;
        }

        // Add the AxWindow pointer to the window property list
        unsafe { SetPropA(handle, "AxonEngine", self) };
        self.platform.handle = handle;

        // If fullscreen, disable legacy window messages such as WM_KEYDOWN, WM_CHAR, WM_MOUSEMOVE, etc.
        let flags: RAWINPUTDEVICE_FLAGS = RAWINPUTDEVICE_FLAGS::default();
        if self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_FULLSCREEN) {
            flags = RIDEV_NOLEGACY;
        }

        // Register raw input devices for this window
        let mut RawInputDevice: [RAWINPUTDEVICE; 2];
        RawInputDevice[0] = RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: flags,
            hwndTarget: handle
        };

        RawInputDevice[1] = RAWINPUTDEVICE{
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            dwFlags: flags,
            hwndTarget: handle
        };

        if unsafe { !RegisterRawInputDevices(&RawInputDevice, 2).is_ok() } {
            return false;
        }

        return true;
    }

    fn get_window_style(&self) -> u32 {
        // Clips all other overlapping sibling and child windows out of the draw region.
        // If these are not specified and they overlap, it is possible, when drawing within the client area
        // of a sibling or child window, to draw within the client area of a neighboring sibling or child window.
        let mut window_style = WS_CLIPSIBLINGS | WS_CLIPCHILDREN; // Default
        if self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_FULLSCREEN) {
            window_style |= WS_POPUP;
        } else {
            window_style |= WS_SYSMENU | WS_MINIMIZEBOX;
            if self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_DECORATED) {
                window_style |= WS_CAPTION; // Title bar
                if self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_RESIZABLE) {
                    window_style |= WS_MAXIMIZEBOX | WS_THICKFRAME;
                }
            } else {
                window_style |= WS_POPUP;
            }
        }

        // Get at the wrapped u32 value
        let result = window_style.0;

        return result;
    }

    fn UpdateCursorImage(&self) {
        if self.cursor_mode == AxCursorMode::AX_CURSOR_NORMAL {
            let cursor: HCURSOR = unsafe { LoadCursorA(None, windows::core::PCSTR(32512u16 as _)).unwrap() };
            unsafe { SetCursor(cursor); }
        } else {
            unsafe { SetCursor(None); }
        }
    }

    fn dpi_changed_handler(&mut self, wparam: WPARAM, lparam: LPARAM) -> Result<()> {
        unsafe {
            let dpi = (wparam.0 as u16 as f32, (wparam.0 >> 16) as f32);
            let rect = &*(lparam.0 as *const RECT);

            // DPI changes don't affect fullscreen windows
            if !self.style.contains(AxWindowStyle::AX_WINDOW_STYLE_FULLSCREEN) {
                    SetWindowPos(
                        self.platform.handle,
                        None,
                        rect.left,
                        rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                SWP_NOZORDER | SWP_NOACTIVATE);
            }

            Ok(())
        }
    }

    fn mouse_move_handler(&mut self, wparam: WPARAM, lparam: usize) {
        let mouse_pos: AxVec2 = AxVec2::new(loword(lparam) as f32, hiword(lparam) as f32);

        if !self.platform.cursor_in_window {
            // This sets things up to post a WM_LEAVE message when the mouse leaves the window
            let mut TME: TRACKMOUSEEVENT = TRACKMOUSEEVENT::default();
            TME.cbSize = size_of::<TRACKMOUSEEVENT>() as u32;
            TME.dwFlags = TME_LEAVE;
            TME.hwndTrack = self.platform.handle;
            unsafe { TrackMouseEvent(&mut TME) };

            self.platform.cursor_in_window = true;
        }

        if self.cursor_mode != AxCursorMode::AX_CURSOR_DISABLED {
            // Use the X and Y directly from WM_MOUSEMOVE (cursor bounded to the resolution)
            self.virtual_cursor_pos = mouse_pos;
        }

        self.platform.last_cursor_pos = mouse_pos;
    }
}