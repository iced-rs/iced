//! Convert [`smithay-client-toolkit`] types into [`iced_native`] types, and viceversa.
//!
//! [`smithay-client-toolkit`]: https://github.com/smithay/client-toolkit
//! [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
use crate::{
    input::{
        keyboard::{KeyCode, ModifiersState},
        mouse,
    },
    MouseCursor,
};

/// Converts a `MouseCursor` from `iced_native` to a `cursor-spec` cursor icon.
pub fn mouse_cursor(mouse_cursor: MouseCursor) -> &'static str {
    match mouse_cursor {
        MouseCursor::OutOfBounds | MouseCursor::Idle => "left_ptr",
        MouseCursor::Pointer => "hand",
        MouseCursor::Working => "progress",
        MouseCursor::Grab => "grab",
        MouseCursor::Grabbing => "grabbing",
        MouseCursor::Text => "text",
        MouseCursor::ResizingHorizontally => "h_double_arrow",
        MouseCursor::ResizingVertically => "v_double_arrow",
    }
}

/// Converts from `input-event-codes` to an `iced_native` mouse button.
pub fn button(button: u32) -> mouse::Button {
    match button {
        0x110 => mouse::Button::Left,
        0x111 => mouse::Button::Right,
        0x112 => mouse::Button::Middle,
        other => mouse::Button::Other(other as u8),
    }
}

/// Converts a `smithay_client_toolkit::seat::keyboard::ModifiersState` to an `iced_native`
/// modifiers state.
pub fn modifiers(
    modifiers: smithay_client_toolkit::seat::keyboard::ModifiersState,
) -> ModifiersState {
    let smithay_client_toolkit::seat::keyboard::ModifiersState {
        shift,
        ctrl,
        alt,
        logo,
        caps_lock: _,
        num_lock: _,
    } = modifiers;
    ModifiersState {
        shift,
        control: ctrl,
        alt,
        logo,
    }
}

/// Converts an `xkb` keysym to an `iced_native` key code.
pub fn key(rawkey: u32, keysym: u32) -> KeyCode {
    use {smithay_client_toolkit::seat::keyboard::keysyms::*, KeyCode::*};
    #[allow(non_upper_case_globals)]
    match rawkey {
        1 => Escape,
        2 => Key1,
        3 => Key2,
        4 => Key3,
        5 => Key4,
        6 => Key5,
        7 => Key6,
        8 => Key7,
        9 => Key8,
        10 => Key9,
        11 => Key0,
        _ => match keysym {
            // letters
            XKB_KEY_A | XKB_KEY_a => A,
            XKB_KEY_B | XKB_KEY_b => B,
            XKB_KEY_C | XKB_KEY_c => C,
            XKB_KEY_D | XKB_KEY_d => (D),
            XKB_KEY_E | XKB_KEY_e => E,
            XKB_KEY_F | XKB_KEY_f => F,
            XKB_KEY_G | XKB_KEY_g => G,
            XKB_KEY_H | XKB_KEY_h => H,
            XKB_KEY_I | XKB_KEY_i => I,
            XKB_KEY_J | XKB_KEY_j => J,
            XKB_KEY_K | XKB_KEY_k => K,
            XKB_KEY_L | XKB_KEY_l => L,
            XKB_KEY_M | XKB_KEY_m => M,
            XKB_KEY_N | XKB_KEY_n => N,
            XKB_KEY_O | XKB_KEY_o => O,
            XKB_KEY_P | XKB_KEY_p => P,
            XKB_KEY_Q | XKB_KEY_q => Q,
            XKB_KEY_R | XKB_KEY_r => R,
            XKB_KEY_S | XKB_KEY_s => S,
            XKB_KEY_T | XKB_KEY_t => T,
            XKB_KEY_U | XKB_KEY_u => U,
            XKB_KEY_V | XKB_KEY_v => V,
            XKB_KEY_W | XKB_KEY_w => W,
            XKB_KEY_X | XKB_KEY_x => X,
            XKB_KEY_Y | XKB_KEY_y => Y,
            XKB_KEY_Z | XKB_KEY_z => Z,
            // F--
            XKB_KEY_F1 => F1,
            XKB_KEY_F2 => F2,
            XKB_KEY_F3 => F3,
            XKB_KEY_F4 => F4,
            XKB_KEY_F5 => F5,
            XKB_KEY_F6 => F6,
            XKB_KEY_F7 => F7,
            XKB_KEY_F8 => F8,
            XKB_KEY_F9 => F9,
            XKB_KEY_F10 => F10,
            XKB_KEY_F11 => F11,
            XKB_KEY_F12 => F12,
            XKB_KEY_F13 => F13,
            XKB_KEY_F14 => F14,
            XKB_KEY_F15 => F15,
            XKB_KEY_F16 => F16,
            XKB_KEY_F17 => F17,
            XKB_KEY_F18 => F18,
            XKB_KEY_F19 => F19,
            XKB_KEY_F20 => F20,
            XKB_KEY_F21 => F21,
            XKB_KEY_F22 => F22,
            XKB_KEY_F23 => F23,
            XKB_KEY_F24 => F24,
            // flow control
            XKB_KEY_Print => Snapshot,
            XKB_KEY_Scroll_Lock => Scroll,
            XKB_KEY_Pause => Pause,
            XKB_KEY_Insert => Insert,
            XKB_KEY_Home => Home,
            XKB_KEY_Delete => Delete,
            XKB_KEY_End => End,
            XKB_KEY_Page_Down => PageDown,
            XKB_KEY_Page_Up => PageUp,
            // arrows
            XKB_KEY_Left => Left,
            XKB_KEY_Up => Up,
            XKB_KEY_Right => Right,
            XKB_KEY_Down => Down,
            //
            XKB_KEY_BackSpace => Backspace,
            XKB_KEY_Return => Enter,
            XKB_KEY_space => Space,
            // keypad
            XKB_KEY_Num_Lock => Numlock,
            XKB_KEY_KP_0 => Numpad0,
            XKB_KEY_KP_1 => Numpad1,
            XKB_KEY_KP_2 => Numpad2,
            XKB_KEY_KP_3 => Numpad3,
            XKB_KEY_KP_4 => Numpad4,
            XKB_KEY_KP_5 => Numpad5,
            XKB_KEY_KP_6 => Numpad6,
            XKB_KEY_KP_7 => Numpad7,
            XKB_KEY_KP_8 => Numpad8,
            XKB_KEY_KP_9 => Numpad9,
            // misc
            XKB_KEY_plus => Add,
            XKB_KEY_apostrophe => Apostrophe,
            XKB_KEY_backslash => Backslash,
            XKB_KEY_colon => Colon,
            XKB_KEY_comma => Comma,
            XKB_KEY_equal => Equals,
            XKB_KEY_Alt_L => LAlt,
            XKB_KEY_Control_L => LControl,
            XKB_KEY_Shift_L => LShift,
            XKB_KEY_minus => Minus,
            XKB_KEY_asterisk => Multiply,
            XKB_KEY_KP_Separator => NumpadComma,
            XKB_KEY_KP_Enter => NumpadEnter,
            XKB_KEY_KP_Equal => NumpadEquals,
            XKB_KEY_KP_Add => Add,
            XKB_KEY_KP_Subtract => Subtract,
            XKB_KEY_KP_Divide => Divide,
            XKB_KEY_KP_Page_Up => PageUp,
            XKB_KEY_KP_Page_Down => PageDown,
            XKB_KEY_KP_Home => Home,
            XKB_KEY_KP_End => End,
            XKB_KEY_Alt_R => RAlt,
            XKB_KEY_Control_R => RControl,
            XKB_KEY_Shift_R => RShift,
            XKB_KEY_semicolon => Semicolon,
            XKB_KEY_slash => Slash,
            XKB_KEY_Tab => Tab,
            XKB_KEY_ISO_Left_Tab => Tab,
            XKB_KEY_XF86AudioLowerVolume => VolumeDown,
            XKB_KEY_XF86AudioRaiseVolume => VolumeUp,
            XKB_KEY_XF86Copy => Copy,
            XKB_KEY_XF86Paste => Paste,
            XKB_KEY_XF86Cut => Cut,
            _ => panic!("Unknown keysym"),
        },
    }
}
