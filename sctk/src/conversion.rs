use iced_native::{
    keyboard::{self, KeyCode},
    mouse::{self, ScrollDelta},
};
use sctk::{
    reexports::client::protocol::wl_pointer::AxisSource,
    seat::{
        keyboard::Modifiers,
        pointer::{AxisScroll, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT},
    },
};
/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
#[error("the futures executor could not be created")]
pub struct KeyCodeError(u32);

pub fn pointer_button_to_native(button: u32) -> Option<mouse::Button> {
    if button == BTN_LEFT {
        Some(mouse::Button::Left)
    } else if button == BTN_RIGHT {
        Some(mouse::Button::Right)
    } else if button == BTN_MIDDLE {
        Some(mouse::Button::Right)
    } else {
        button.try_into().ok().map(|b| mouse::Button::Other(b))
    }
}

pub fn pointer_axis_to_native(
    source: Option<AxisSource>,
    horizontal: AxisScroll,
    vertical: AxisScroll,
) -> Option<ScrollDelta> {
    source.map(|source| match source {
        AxisSource::Wheel | AxisSource::WheelTilt => ScrollDelta::Lines {
            x: horizontal.discrete as f32,
            y: vertical.discrete as f32,
        },
        AxisSource::Finger | AxisSource::Continuous | _ => {
            ScrollDelta::Pixels {
                x: horizontal.absolute as f32,
                y: vertical.absolute as f32,
            }
        }
    })
}

pub fn modifiers_to_native(mods: Modifiers) -> keyboard::Modifiers {
    let mut native_mods = keyboard::Modifiers::empty();
    if mods.alt {
        native_mods = native_mods.union(keyboard::Modifiers::ALT);
    }
    if mods.ctrl {
        native_mods = native_mods.union(keyboard::Modifiers::CTRL);
    }
    if mods.logo {
        native_mods = native_mods.union(keyboard::Modifiers::LOGO);
    }
    if mods.shift {
        native_mods = native_mods.union(keyboard::Modifiers::SHIFT);
    }
    // TODO Ashley: missing modifiers as platform specific additions?
    // if mods.caps_lock {
    // native_mods = native_mods.union(keyboard::Modifier);
    // }
    // if mods.num_lock {
    //     native_mods = native_mods.union(keyboard::Modifiers::);
    // }
    native_mods
}

pub fn keysym_to_vkey(keysym: u32) -> Option<KeyCode> {
    use sctk::seat::keyboard::keysyms;
    match keysym {
        // Numbers.
        keysyms::XKB_KEY_1 => Some(KeyCode::Key1),
        keysyms::XKB_KEY_2 => Some(KeyCode::Key2),
        keysyms::XKB_KEY_3 => Some(KeyCode::Key3),
        keysyms::XKB_KEY_4 => Some(KeyCode::Key4),
        keysyms::XKB_KEY_5 => Some(KeyCode::Key5),
        keysyms::XKB_KEY_6 => Some(KeyCode::Key6),
        keysyms::XKB_KEY_7 => Some(KeyCode::Key7),
        keysyms::XKB_KEY_8 => Some(KeyCode::Key8),
        keysyms::XKB_KEY_9 => Some(KeyCode::Key9),
        keysyms::XKB_KEY_0 => Some(KeyCode::Key0),
        // Letters.
        keysyms::XKB_KEY_A | keysyms::XKB_KEY_a => Some(KeyCode::A),
        keysyms::XKB_KEY_B | keysyms::XKB_KEY_b => Some(KeyCode::B),
        keysyms::XKB_KEY_C | keysyms::XKB_KEY_c => Some(KeyCode::C),
        keysyms::XKB_KEY_D | keysyms::XKB_KEY_d => Some(KeyCode::D),
        keysyms::XKB_KEY_E | keysyms::XKB_KEY_e => Some(KeyCode::E),
        keysyms::XKB_KEY_F | keysyms::XKB_KEY_f => Some(KeyCode::F),
        keysyms::XKB_KEY_G | keysyms::XKB_KEY_g => Some(KeyCode::G),
        keysyms::XKB_KEY_H | keysyms::XKB_KEY_h => Some(KeyCode::H),
        keysyms::XKB_KEY_I | keysyms::XKB_KEY_i => Some(KeyCode::I),
        keysyms::XKB_KEY_J | keysyms::XKB_KEY_j => Some(KeyCode::J),
        keysyms::XKB_KEY_K | keysyms::XKB_KEY_k => Some(KeyCode::K),
        keysyms::XKB_KEY_L | keysyms::XKB_KEY_l => Some(KeyCode::L),
        keysyms::XKB_KEY_M | keysyms::XKB_KEY_m => Some(KeyCode::M),
        keysyms::XKB_KEY_N | keysyms::XKB_KEY_n => Some(KeyCode::N),
        keysyms::XKB_KEY_O | keysyms::XKB_KEY_o => Some(KeyCode::O),
        keysyms::XKB_KEY_P | keysyms::XKB_KEY_p => Some(KeyCode::P),
        keysyms::XKB_KEY_Q | keysyms::XKB_KEY_q => Some(KeyCode::Q),
        keysyms::XKB_KEY_R | keysyms::XKB_KEY_r => Some(KeyCode::R),
        keysyms::XKB_KEY_S | keysyms::XKB_KEY_s => Some(KeyCode::S),
        keysyms::XKB_KEY_T | keysyms::XKB_KEY_t => Some(KeyCode::T),
        keysyms::XKB_KEY_U | keysyms::XKB_KEY_u => Some(KeyCode::U),
        keysyms::XKB_KEY_V | keysyms::XKB_KEY_v => Some(KeyCode::V),
        keysyms::XKB_KEY_W | keysyms::XKB_KEY_w => Some(KeyCode::W),
        keysyms::XKB_KEY_X | keysyms::XKB_KEY_x => Some(KeyCode::X),
        keysyms::XKB_KEY_Y | keysyms::XKB_KEY_y => Some(KeyCode::Y),
        keysyms::XKB_KEY_Z | keysyms::XKB_KEY_z => Some(KeyCode::Z),
        // Escape.
        keysyms::XKB_KEY_Escape => Some(KeyCode::Escape),
        // Function keys.
        keysyms::XKB_KEY_F1 => Some(KeyCode::F1),
        keysyms::XKB_KEY_F2 => Some(KeyCode::F2),
        keysyms::XKB_KEY_F3 => Some(KeyCode::F3),
        keysyms::XKB_KEY_F4 => Some(KeyCode::F4),
        keysyms::XKB_KEY_F5 => Some(KeyCode::F5),
        keysyms::XKB_KEY_F6 => Some(KeyCode::F6),
        keysyms::XKB_KEY_F7 => Some(KeyCode::F7),
        keysyms::XKB_KEY_F8 => Some(KeyCode::F8),
        keysyms::XKB_KEY_F9 => Some(KeyCode::F9),
        keysyms::XKB_KEY_F10 => Some(KeyCode::F10),
        keysyms::XKB_KEY_F11 => Some(KeyCode::F11),
        keysyms::XKB_KEY_F12 => Some(KeyCode::F12),
        keysyms::XKB_KEY_F13 => Some(KeyCode::F13),
        keysyms::XKB_KEY_F14 => Some(KeyCode::F14),
        keysyms::XKB_KEY_F15 => Some(KeyCode::F15),
        keysyms::XKB_KEY_F16 => Some(KeyCode::F16),
        keysyms::XKB_KEY_F17 => Some(KeyCode::F17),
        keysyms::XKB_KEY_F18 => Some(KeyCode::F18),
        keysyms::XKB_KEY_F19 => Some(KeyCode::F19),
        keysyms::XKB_KEY_F20 => Some(KeyCode::F20),
        keysyms::XKB_KEY_F21 => Some(KeyCode::F21),
        keysyms::XKB_KEY_F22 => Some(KeyCode::F22),
        keysyms::XKB_KEY_F23 => Some(KeyCode::F23),
        keysyms::XKB_KEY_F24 => Some(KeyCode::F24),
        // Flow control.
        keysyms::XKB_KEY_Print => Some(KeyCode::Snapshot),
        keysyms::XKB_KEY_Scroll_Lock => Some(KeyCode::Scroll),
        keysyms::XKB_KEY_Pause => Some(KeyCode::Pause),
        keysyms::XKB_KEY_Insert => Some(KeyCode::Insert),
        keysyms::XKB_KEY_Home => Some(KeyCode::Home),
        keysyms::XKB_KEY_Delete => Some(KeyCode::Delete),
        keysyms::XKB_KEY_End => Some(KeyCode::End),
        keysyms::XKB_KEY_Page_Down => Some(KeyCode::PageDown),
        keysyms::XKB_KEY_Page_Up => Some(KeyCode::PageUp),
        // Arrows.
        keysyms::XKB_KEY_Left => Some(KeyCode::Left),
        keysyms::XKB_KEY_Up => Some(KeyCode::Up),
        keysyms::XKB_KEY_Right => Some(KeyCode::Right),
        keysyms::XKB_KEY_Down => Some(KeyCode::Down),

        keysyms::XKB_KEY_BackSpace => Some(KeyCode::Backspace),
        keysyms::XKB_KEY_Return => Some(KeyCode::Enter),
        keysyms::XKB_KEY_space => Some(KeyCode::Space),

        keysyms::XKB_KEY_Multi_key => Some(KeyCode::Compose),
        keysyms::XKB_KEY_caret => Some(KeyCode::Caret),

        // Keypad.
        keysyms::XKB_KEY_Num_Lock => Some(KeyCode::Numlock),
        keysyms::XKB_KEY_KP_0 => Some(KeyCode::Numpad0),
        keysyms::XKB_KEY_KP_1 => Some(KeyCode::Numpad1),
        keysyms::XKB_KEY_KP_2 => Some(KeyCode::Numpad2),
        keysyms::XKB_KEY_KP_3 => Some(KeyCode::Numpad3),
        keysyms::XKB_KEY_KP_4 => Some(KeyCode::Numpad4),
        keysyms::XKB_KEY_KP_5 => Some(KeyCode::Numpad5),
        keysyms::XKB_KEY_KP_6 => Some(KeyCode::Numpad6),
        keysyms::XKB_KEY_KP_7 => Some(KeyCode::Numpad7),
        keysyms::XKB_KEY_KP_8 => Some(KeyCode::Numpad8),
        keysyms::XKB_KEY_KP_9 => Some(KeyCode::Numpad9),
        // Misc.
        // => Some(KeyCode::AbntC1),
        // => Some(KeyCode::AbntC2),
        keysyms::XKB_KEY_plus => Some(KeyCode::Plus),
        keysyms::XKB_KEY_apostrophe => Some(KeyCode::Apostrophe),
        // => Some(KeyCode::Apps),
        keysyms::XKB_KEY_at => Some(KeyCode::At),
        // => Some(KeyCode::Ax),
        keysyms::XKB_KEY_backslash => Some(KeyCode::Backslash),
        keysyms::XKB_KEY_XF86Calculator => Some(KeyCode::Calculator),
        // => Some(KeyCode::Capital),
        keysyms::XKB_KEY_colon => Some(KeyCode::Colon),
        keysyms::XKB_KEY_comma => Some(KeyCode::Comma),
        // => Some(KeyCode::Convert),
        keysyms::XKB_KEY_equal => Some(KeyCode::Equals),
        keysyms::XKB_KEY_grave => Some(KeyCode::Grave),
        // => Some(KeyCode::Kana),
        keysyms::XKB_KEY_Kanji => Some(KeyCode::Kanji),
        keysyms::XKB_KEY_Alt_L => Some(KeyCode::LAlt),
        keysyms::XKB_KEY_bracketleft => Some(KeyCode::LBracket),
        keysyms::XKB_KEY_Control_L => Some(KeyCode::LControl),
        keysyms::XKB_KEY_Shift_L => Some(KeyCode::LShift),
        keysyms::XKB_KEY_Super_L => Some(KeyCode::LWin),
        keysyms::XKB_KEY_XF86Mail => Some(KeyCode::Mail),
        // => Some(KeyCode::MediaSelect),
        // => Some(KeyCode::MediaStop),
        keysyms::XKB_KEY_minus => Some(KeyCode::Minus),
        keysyms::XKB_KEY_asterisk => Some(KeyCode::Asterisk),
        keysyms::XKB_KEY_XF86AudioMute => Some(KeyCode::Mute),
        // => Some(KeyCode::MyComputer),
        keysyms::XKB_KEY_XF86AudioNext => Some(KeyCode::NextTrack),
        // => Some(KeyCode::NoConvert),
        keysyms::XKB_KEY_KP_Separator => Some(KeyCode::NumpadComma),
        keysyms::XKB_KEY_KP_Enter => Some(KeyCode::NumpadEnter),
        keysyms::XKB_KEY_KP_Equal => Some(KeyCode::NumpadEquals),
        keysyms::XKB_KEY_KP_Add => Some(KeyCode::NumpadAdd),
        keysyms::XKB_KEY_KP_Subtract => Some(KeyCode::NumpadSubtract),
        keysyms::XKB_KEY_KP_Multiply => Some(KeyCode::NumpadMultiply),
        keysyms::XKB_KEY_KP_Divide => Some(KeyCode::NumpadDivide),
        keysyms::XKB_KEY_KP_Decimal => Some(KeyCode::NumpadDecimal),
        keysyms::XKB_KEY_KP_Page_Up => Some(KeyCode::PageUp),
        keysyms::XKB_KEY_KP_Page_Down => Some(KeyCode::PageDown),
        keysyms::XKB_KEY_KP_Home => Some(KeyCode::Home),
        keysyms::XKB_KEY_KP_End => Some(KeyCode::End),
        keysyms::XKB_KEY_KP_Left => Some(KeyCode::Left),
        keysyms::XKB_KEY_KP_Up => Some(KeyCode::Up),
        keysyms::XKB_KEY_KP_Right => Some(KeyCode::Right),
        keysyms::XKB_KEY_KP_Down => Some(KeyCode::Down),
        // => Some(KeyCode::OEM102),
        keysyms::XKB_KEY_period => Some(KeyCode::Period),
        // => Some(KeyCode::Playpause),
        keysyms::XKB_KEY_XF86PowerOff => Some(KeyCode::Power),
        keysyms::XKB_KEY_XF86AudioPrev => Some(KeyCode::PrevTrack),
        keysyms::XKB_KEY_Alt_R => Some(KeyCode::RAlt),
        keysyms::XKB_KEY_bracketright => Some(KeyCode::RBracket),
        keysyms::XKB_KEY_Control_R => Some(KeyCode::RControl),
        keysyms::XKB_KEY_Shift_R => Some(KeyCode::RShift),
        keysyms::XKB_KEY_Super_R => Some(KeyCode::RWin),
        keysyms::XKB_KEY_semicolon => Some(KeyCode::Semicolon),
        keysyms::XKB_KEY_slash => Some(KeyCode::Slash),
        keysyms::XKB_KEY_XF86Sleep => Some(KeyCode::Sleep),
        // => Some(KeyCode::Stop),
        // => Some(KeyCode::Sysrq),
        keysyms::XKB_KEY_Tab => Some(KeyCode::Tab),
        keysyms::XKB_KEY_ISO_Left_Tab => Some(KeyCode::Tab),
        keysyms::XKB_KEY_underscore => Some(KeyCode::Underline),
        // => Some(KeyCode::Unlabeled),
        keysyms::XKB_KEY_XF86AudioLowerVolume => Some(KeyCode::VolumeDown),
        keysyms::XKB_KEY_XF86AudioRaiseVolume => Some(KeyCode::VolumeUp),
        // => Some(KeyCode::Wake),
        // => Some(KeyCode::Webback),
        // => Some(KeyCode::WebFavorites),
        // => Some(KeyCode::WebForward),
        // => Some(KeyCode::WebHome),
        // => Some(KeyCode::WebRefresh),
        // => Some(KeyCode::WebSearch),
        // => Some(KeyCode::WebStop),
        keysyms::XKB_KEY_yen => Some(KeyCode::Yen),
        keysyms::XKB_KEY_XF86Copy => Some(KeyCode::Copy),
        keysyms::XKB_KEY_XF86Paste => Some(KeyCode::Paste),
        keysyms::XKB_KEY_XF86Cut => Some(KeyCode::Cut),
        // Fallback.
        _ => None,
    }
}
