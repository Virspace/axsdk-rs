// Key codes
pub mod AxKeys {
    pub const AX_KEY_1: i8                 = 0x002;
    pub const AX_KEY_2: i8                 = 0x003;
    pub const AX_KEY_3: i8                 = 0x004;
    pub const AX_KEY_4: i8                 = 0x005;
    pub const AX_KEY_5: i8                 = 0x006;
    pub const AX_KEY_6: i8                 = 0x007;
    pub const AX_KEY_7: i8                 = 0x008;
    pub const AX_KEY_8: i8                 = 0x009;
    pub const AX_KEY_9: i8                 = 0x00A;
    pub const AX_KEY_0: i8                 = 0x00B;
    pub const AX_KEY_A: i8                 = 0x01E;
    pub const AX_KEY_B: i8                 = 0x030;
    pub const AX_KEY_C: i8                 = 0x02E;
    pub const AX_KEY_D: i8                 = 0x020;
    pub const AX_KEY_E: i8                 = 0x012;
    pub const AX_KEY_F: i8                 = 0x021;
    pub const AX_KEY_G: i8                 = 0x022;
    pub const AX_KEY_H: i8                 = 0x023;
    pub const AX_KEY_I: i8                 = 0x017;
    pub const AX_KEY_J: i8                 = 0x024;
    pub const AX_KEY_K: i8                 = 0x025;
    pub const AX_KEY_L: i8                 = 0x026;
    pub const AX_KEY_M: i8                 = 0x032;
    pub const AX_KEY_N: i8                 = 0x031;
    pub const AX_KEY_O: i8                 = 0x018;
    pub const AX_KEY_P: i8                 = 0x019;
    pub const AX_KEY_Q: i8                 = 0x010;
    pub const AX_KEY_R: i8                 = 0x013;
    pub const AX_KEY_S: i8                 = 0x01F;
    pub const AX_KEY_T: i8                 = 0x014;
    pub const AX_KEY_U: i8                 = 0x016;
    pub const AX_KEY_V: i8                 = 0x02F;
    pub const AX_KEY_W: i8                 = 0x011;
    pub const AX_KEY_X: i8                 = 0x02D;
    pub const AX_KEY_Y: i8                 = 0x015;
    pub const AX_KEY_Z: i8                 = 0x02C;
    pub const AX_KEY_BACKSPACE: i8         = 0x00E;
    pub const AX_KEY_DELETE: i8            = 0x153;
    pub const AX_KEY_END: i8               = 0x14F;
    pub const AX_KEY_ENTER: i8             = 0x01C;
    pub const AX_KEY_ESCAPE: i8            = 0x001;
    pub const AX_KEY_HOME: i8              = 0x147;
    pub const AX_KEY_INSERT: i8            = 0x152;
    pub const AX_KEY_PAGE_DOWN: i8         = 0x151;
    pub const AX_KEY_PAGE_UP: i8           = 0x149;
    pub const AX_KEY_PAUSE: i8             = 0x045;
    pub const AX_KEY_UP: i8                = 0x148;
    pub const AX_KEY_DOWN: i8              = 0x150;
    pub const AX_KEY_LEFT: i8              = 0x14B;
    pub const AX_KEY_RIGHT: i8             = 0x14D;
    pub const AX_KEY_LEFT_ALT: i8          = 0x038;
    pub const AX_KEY_RIGHT_ALT: i8         = 0x138;
    pub const AX_KEY_LEFT_SHIFT: i8        = 0x02A;
    pub const AX_KEY_RIGHT_SHIFT: i8       = 0x036;
    pub const AX_KEY_SPACE: i8             = 0x039;
    pub const AX_KEY_TAB: i8               = 0x00F;
    pub const AX_KEY_LEFT_CTRL: i8         = 0x01D;
    pub const AX_KEY_RIGHT_CTRL: i8        = 0x11D;
    pub const AX_KEY_NUMPAD_0: i8          = 0x052;
    pub const AX_KEY_NUMPAD_1: i8          = 0x04F;
    pub const AX_KEY_NUMPAD_2: i8          = 0x050;
    pub const AX_KEY_NUMPAD_3: i8          = 0x051;
    pub const AX_KEY_NUMPAD_4: i8          = 0x04B;
    pub const AX_KEY_NUMPAD_5: i8          = 0x04C;
    pub const AX_KEY_NUMPAD_6: i8          = 0x04D;
    pub const AX_KEY_NUMPAD_7: i8          = 0x047;
    pub const AX_KEY_NUMPAD_8: i8          = 0x048;
    pub const AX_KEY_NUMPAD_9: i8          = 0x049;
    pub const AX_KEY_NUMPAD_ENTER: i8      = 0x11C;
    pub const AX_KEY_NUMPAD_DECIMAL: i8    = 0x053;
}

// Key modifer flags
pub enum AxKeyModifier {
    AxKeyShift               = 1 << 0,
    AxKeyCtrl                = 1 << 1,
    AxKeyAlt                 = 1 << 2,
    AxKeyWin                 = 1 << 3,
    AxKeyCaps                = 1 << 4,
    AxKeyNumlock             = 1 << 5
}

// Mouse buttons
pub mod AxMouse {
    pub const AX_MOUSE_BUTTON_1: i32             = 0;
    pub const AX_MOUSE_BUTTON_2: i32             = 1;
    pub const AX_MOUSE_BUTTON_3: i32             = 2;
    pub const AX_MOUSE_BUTTON_4: i32             = 3;
    pub const AX_MOUSE_BUTTON_5: i32             = 4;
    pub const AX_MOUSE_BUTTON_6: i32             = 5;
    pub const AX_MOUSE_BUTTON_7: i32             = 6;
    pub const AX_MOUSE_BUTTON_8: i32             = 7;
    pub const AX_MOUSE_BUTTON_LAST: i32          = AX_MOUSE_BUTTON_8;
    pub const AX_MOUSE_BUTTON_LEFT: i32          = AX_MOUSE_BUTTON_1; // Alias
    pub const AX_MOUSE_BUTTON_RIGHT: i32         = AX_MOUSE_BUTTON_2; // Alias
    pub const AX_MOUSE_BUTTON_MIDDLE: i32        = AX_MOUSE_BUTTON_3; // Alias
}

// Mouse button states
pub const AX_RELEASE: i32                        = 0;
pub const AX_PRESS: i32                          = 1;

#[no_mangle]
pub fn kb(value: i64) -> i64 {
    value * 1024
}

#[no_mangle]
pub fn mb(value: i64) -> i64 {
    kb(value) * 1024
}

#[no_mangle]
pub fn gb(value: i64) -> i64 {
    mb(value) * 1024
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct AxRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32
}

impl AxRect {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        AxRect { left, top, right, bottom }
    }

    pub fn default() -> Self {
        AxRect { left: 0.0, top: 0.0, right: 0.0, bottom: 0.0 }
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub fn position(&self) -> AxVec2 {
        AxVec2::new(self.left, self.top)
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct AxVec2 {
    pub x: f32,
    pub y: f32
}

impl AxVec2 {
    pub fn new(x: f32, y: f32) -> Self {
        AxVec2 { x, y }
    }

    pub fn default() -> Self {
        AxVec2 { x: 0.0, y: 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kb_test() {
        let result = kb(4);
        assert_eq!(result, 4096);
    }

    #[test]
    fn mb_test() {
        let result = mb(2);
        assert_eq!(result, 2097152);
    }

    #[test]
    fn gb_test() {
        let result = gb(2);
        assert_eq!(result, 2147483648)
    }
}