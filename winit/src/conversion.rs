//! Convert [`winit`] types into [`iced_native`] types, and viceversa.
//!
//! [`winit`]: https://github.com/rust-windowing/winit
//! [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
use crate::keyboard;
use crate::menu::{self, Menu};
use crate::mouse;
use crate::touch;
use crate::window;
use crate::{Event, Mode, Point, Position};

/// Converts a winit window event into an iced event.
pub fn window_event(
    event: &winit::event::WindowEvent<'_>,
    scale_factor: f64,
    modifiers: winit::event::ModifiersState,
) -> Option<Event> {
    use winit::event::WindowEvent;

    match event {
        WindowEvent::Resized(new_size) => {
            let logical_size = new_size.to_logical(scale_factor);

            Some(Event::Window(window::Event::Resized {
                width: logical_size.width,
                height: logical_size.height,
            }))
        }
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
            let logical_size = new_inner_size.to_logical(scale_factor);

            Some(Event::Window(window::Event::Resized {
                width: logical_size.width,
                height: logical_size.height,
            }))
        }
        WindowEvent::CloseRequested => {
            Some(Event::Window(window::Event::CloseRequested))
        }
        WindowEvent::CursorMoved { position, .. } => {
            let position = position.to_logical::<f64>(scale_factor);

            Some(Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(position.x as f32, position.y as f32),
            }))
        }
        WindowEvent::CursorEntered { .. } => {
            Some(Event::Mouse(mouse::Event::CursorEntered))
        }
        WindowEvent::CursorLeft { .. } => {
            Some(Event::Mouse(mouse::Event::CursorLeft))
        }
        WindowEvent::MouseInput { button, state, .. } => {
            let button = mouse_button(*button);

            Some(Event::Mouse(match state {
                winit::event::ElementState::Pressed => {
                    mouse::Event::ButtonPressed(button)
                }
                winit::event::ElementState::Released => {
                    mouse::Event::ButtonReleased(button)
                }
            }))
        }
        WindowEvent::MouseWheel { delta, .. } => match delta {
            winit::event::MouseScrollDelta::LineDelta(delta_x, delta_y) => {
                Some(Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines {
                        x: *delta_x,
                        y: *delta_y,
                    },
                }))
            }
            winit::event::MouseScrollDelta::PixelDelta(position) => {
                Some(Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels {
                        x: position.x as f32,
                        y: position.y as f32,
                    },
                }))
            }
        },
        WindowEvent::ReceivedCharacter(c) if !is_private_use_character(*c) => {
            Some(Event::Keyboard(keyboard::Event::CharacterReceived(*c)))
        }
        WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(virtual_keycode),
                    state,
                    ..
                },
            ..
        } => Some(Event::Keyboard({
            let key_code = key_code(*virtual_keycode);
            let modifiers = self::modifiers(modifiers);

            match state {
                winit::event::ElementState::Pressed => {
                    keyboard::Event::KeyPressed {
                        key_code,
                        modifiers,
                    }
                }
                winit::event::ElementState::Released => {
                    keyboard::Event::KeyReleased {
                        key_code,
                        modifiers,
                    }
                }
            }
        })),
        WindowEvent::ModifiersChanged(new_modifiers) => Some(Event::Keyboard(
            keyboard::Event::ModifiersChanged(self::modifiers(*new_modifiers)),
        )),
        WindowEvent::Focused(focused) => Some(Event::Window(if *focused {
            window::Event::Focused
        } else {
            window::Event::Unfocused
        })),
        WindowEvent::HoveredFile(path) => {
            Some(Event::Window(window::Event::FileHovered(path.clone())))
        }
        WindowEvent::DroppedFile(path) => {
            Some(Event::Window(window::Event::FileDropped(path.clone())))
        }
        WindowEvent::HoveredFileCancelled => {
            Some(Event::Window(window::Event::FilesHoveredLeft))
        }
        WindowEvent::Touch(touch) => {
            Some(Event::Touch(touch_event(*touch, scale_factor)))
        }
        WindowEvent::Moved(position) => {
            let winit::dpi::LogicalPosition { x, y } =
                position.to_logical(scale_factor);

            Some(Event::Window(window::Event::Moved { x, y }))
        }
        _ => None,
    }
}

/// Converts a [`Position`] to a [`winit`] logical position for a given monitor.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn position(
    monitor: Option<&winit::monitor::MonitorHandle>,
    (width, height): (u32, u32),
    position: Position,
) -> Option<winit::dpi::Position> {
    match position {
        Position::Default => None,
        Position::Specific(x, y) => {
            Some(winit::dpi::Position::Logical(winit::dpi::LogicalPosition {
                x: f64::from(x),
                y: f64::from(y),
            }))
        }
        Position::Centered => {
            if let Some(monitor) = monitor {
                let start = monitor.position();

                let resolution: winit::dpi::LogicalSize<f64> =
                    monitor.size().to_logical(monitor.scale_factor());

                let centered: winit::dpi::PhysicalPosition<i32> =
                    winit::dpi::LogicalPosition {
                        x: (resolution.width - f64::from(width)) / 2.0,
                        y: (resolution.height - f64::from(height)) / 2.0,
                    }
                    .to_physical(monitor.scale_factor());

                Some(winit::dpi::Position::Physical(
                    winit::dpi::PhysicalPosition {
                        x: start.x + centered.x,
                        y: start.y + centered.y,
                    },
                ))
            } else {
                None
            }
        }
    }
}

/// Converts a [`Mode`] to a [`winit`] fullscreen mode.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn fullscreen(
    monitor: Option<winit::monitor::MonitorHandle>,
    mode: Mode,
) -> Option<winit::window::Fullscreen> {
    match mode {
        Mode::Windowed | Mode::Hidden => None,
        Mode::Fullscreen => {
            Some(winit::window::Fullscreen::Borderless(monitor))
        }
    }
}

/// Converts a [`Mode`] to a visibility flag.
pub fn visible(mode: Mode) -> bool {
    match mode {
        Mode::Windowed | Mode::Fullscreen => true,
        Mode::Hidden => false,
    }
}

/// Converts a `Hotkey` from [`iced_native`] to a [`winit`] Hotkey.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
fn hotkey(hotkey: keyboard::Hotkey) -> winit::window::Hotkey {
    use winit::event::ModifiersState;

    let mut modifiers = ModifiersState::empty();
    modifiers.set(ModifiersState::CTRL, hotkey.modifiers.control());
    modifiers.set(ModifiersState::SHIFT, hotkey.modifiers.shift());
    modifiers.set(ModifiersState::ALT, hotkey.modifiers.alt());
    modifiers.set(ModifiersState::LOGO, hotkey.modifiers.logo());

    winit::window::Hotkey::new(modifiers, to_virtual_keycode(hotkey.key))
}

/// Converts a `Menu` from [`iced_native`] to a [`winit`] menu.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn menu<Message>(menu: &Menu<Message>) -> winit::window::Menu {
    fn menu_i<Message>(
        converted: &mut winit::window::Menu,
        starting_id: usize,
        menu: &Menu<Message>,
    ) -> usize {
        let mut id = starting_id;

        for item in menu.iter() {
            match item {
                menu::Entry::Item { title, hotkey, .. } => {
                    converted.add_item(id, title, hotkey.map(self::hotkey));

                    id += 1;
                }
                menu::Entry::Dropdown { title, submenu } => {
                    let mut converted_submenu = winit::window::Menu::new();
                    let n_children =
                        menu_i(&mut converted_submenu, id, submenu);

                    converted.add_dropdown(title, converted_submenu);

                    id += n_children;
                }
                menu::Entry::Separator => {
                    converted.add_separator();
                }
            }
        }

        id - starting_id
    }

    let mut converted = winit::window::Menu::default();
    let _ = menu_i(&mut converted, 0, menu);

    converted
}

/// Given a [`Menu`] and an identifier of a [`menu::Entry`], it returns the
/// `Message` that should be produced when that entry is activated.
pub fn menu_message<Message>(menu: &Menu<Message>, id: u32) -> Option<Message>
where
    Message: Clone,
{
    fn find_message<Message>(
        target: u32,
        starting_id: u32,
        menu: &Menu<Message>,
    ) -> Result<Message, u32>
    where
        Message: Clone,
    {
        let mut id = starting_id;

        for entry in menu.iter() {
            match entry {
                menu::Entry::Item { on_activation, .. } => {
                    if id == target {
                        return Ok(on_activation.clone());
                    }

                    id += 1;
                }
                menu::Entry::Dropdown { submenu, .. } => {
                    match find_message(target, id, submenu) {
                        Ok(message) => {
                            return Ok(message);
                        }
                        Err(n_children) => {
                            id += n_children;
                        }
                    }
                }
                menu::Entry::Separator => {}
            }
        }

        Err(id - starting_id)
    }

    find_message(id, 0, menu).ok()
}

/// Converts a `MouseCursor` from [`iced_native`] to a [`winit`] cursor icon.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn mouse_interaction(
    interaction: mouse::Interaction,
) -> winit::window::CursorIcon {
    use mouse::Interaction;

    match interaction {
        Interaction::Idle => winit::window::CursorIcon::Default,
        Interaction::Pointer => winit::window::CursorIcon::Hand,
        Interaction::Working => winit::window::CursorIcon::Progress,
        Interaction::Grab => winit::window::CursorIcon::Grab,
        Interaction::Grabbing => winit::window::CursorIcon::Grabbing,
        Interaction::Crosshair => winit::window::CursorIcon::Crosshair,
        Interaction::Text => winit::window::CursorIcon::Text,
        Interaction::ResizingHorizontally => {
            winit::window::CursorIcon::EwResize
        }
        Interaction::ResizingVertically => winit::window::CursorIcon::NsResize,
    }
}

/// Converts a `MouseButton` from [`winit`] to an [`iced_native`] mouse button.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn mouse_button(mouse_button: winit::event::MouseButton) -> mouse::Button {
    match mouse_button {
        winit::event::MouseButton::Left => mouse::Button::Left,
        winit::event::MouseButton::Right => mouse::Button::Right,
        winit::event::MouseButton::Middle => mouse::Button::Middle,
        winit::event::MouseButton::Other(other) => {
            mouse::Button::Other(other as u8)
        }
    }
}

/// Converts some `ModifiersState` from [`winit`] to an [`iced_native`]
/// modifiers state.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn modifiers(
    modifiers: winit::event::ModifiersState,
) -> keyboard::Modifiers {
    let mut result = keyboard::Modifiers::empty();

    result.set(keyboard::Modifiers::SHIFT, modifiers.shift());
    result.set(keyboard::Modifiers::CTRL, modifiers.ctrl());
    result.set(keyboard::Modifiers::ALT, modifiers.alt());
    result.set(keyboard::Modifiers::LOGO, modifiers.logo());

    result
}

/// Converts a physical cursor position to a logical `Point`.
pub fn cursor_position(
    position: winit::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
) -> Point {
    let logical_position = position.to_logical(scale_factor);

    Point::new(logical_position.x, logical_position.y)
}

/// Converts a `Touch` from [`winit`] to an [`iced_native`] touch event.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn touch_event(
    touch: winit::event::Touch,
    scale_factor: f64,
) -> touch::Event {
    let id = touch::Finger(touch.id);
    let position = {
        let location = touch.location.to_logical::<f64>(scale_factor);

        Point::new(location.x as f32, location.y as f32)
    };

    match touch.phase {
        winit::event::TouchPhase::Started => {
            touch::Event::FingerPressed { id, position }
        }
        winit::event::TouchPhase::Moved => {
            touch::Event::FingerMoved { id, position }
        }
        winit::event::TouchPhase::Ended => {
            touch::Event::FingerLifted { id, position }
        }
        winit::event::TouchPhase::Cancelled => {
            touch::Event::FingerLost { id, position }
        }
    }
}

/// Converts a `KeyCode` from [`iced_native`] to an [`winit`] key code.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
fn to_virtual_keycode(
    keycode: keyboard::KeyCode,
) -> winit::event::VirtualKeyCode {
    use keyboard::KeyCode;
    use winit::event::VirtualKeyCode;

    match keycode {
        KeyCode::Key1 => VirtualKeyCode::Key1,
        KeyCode::Key2 => VirtualKeyCode::Key2,
        KeyCode::Key3 => VirtualKeyCode::Key3,
        KeyCode::Key4 => VirtualKeyCode::Key4,
        KeyCode::Key5 => VirtualKeyCode::Key5,
        KeyCode::Key6 => VirtualKeyCode::Key6,
        KeyCode::Key7 => VirtualKeyCode::Key7,
        KeyCode::Key8 => VirtualKeyCode::Key8,
        KeyCode::Key9 => VirtualKeyCode::Key9,
        KeyCode::Key0 => VirtualKeyCode::Key0,
        KeyCode::A => VirtualKeyCode::A,
        KeyCode::B => VirtualKeyCode::B,
        KeyCode::C => VirtualKeyCode::C,
        KeyCode::D => VirtualKeyCode::D,
        KeyCode::E => VirtualKeyCode::E,
        KeyCode::F => VirtualKeyCode::F,
        KeyCode::G => VirtualKeyCode::G,
        KeyCode::H => VirtualKeyCode::H,
        KeyCode::I => VirtualKeyCode::I,
        KeyCode::J => VirtualKeyCode::J,
        KeyCode::K => VirtualKeyCode::K,
        KeyCode::L => VirtualKeyCode::L,
        KeyCode::M => VirtualKeyCode::M,
        KeyCode::N => VirtualKeyCode::N,
        KeyCode::O => VirtualKeyCode::O,
        KeyCode::P => VirtualKeyCode::P,
        KeyCode::Q => VirtualKeyCode::Q,
        KeyCode::R => VirtualKeyCode::R,
        KeyCode::S => VirtualKeyCode::S,
        KeyCode::T => VirtualKeyCode::T,
        KeyCode::U => VirtualKeyCode::U,
        KeyCode::V => VirtualKeyCode::V,
        KeyCode::W => VirtualKeyCode::W,
        KeyCode::X => VirtualKeyCode::X,
        KeyCode::Y => VirtualKeyCode::Y,
        KeyCode::Z => VirtualKeyCode::Z,
        KeyCode::Escape => VirtualKeyCode::Escape,
        KeyCode::F1 => VirtualKeyCode::F1,
        KeyCode::F2 => VirtualKeyCode::F2,
        KeyCode::F3 => VirtualKeyCode::F3,
        KeyCode::F4 => VirtualKeyCode::F4,
        KeyCode::F5 => VirtualKeyCode::F5,
        KeyCode::F6 => VirtualKeyCode::F6,
        KeyCode::F7 => VirtualKeyCode::F7,
        KeyCode::F8 => VirtualKeyCode::F8,
        KeyCode::F9 => VirtualKeyCode::F9,
        KeyCode::F10 => VirtualKeyCode::F10,
        KeyCode::F11 => VirtualKeyCode::F11,
        KeyCode::F12 => VirtualKeyCode::F12,
        KeyCode::F13 => VirtualKeyCode::F13,
        KeyCode::F14 => VirtualKeyCode::F14,
        KeyCode::F15 => VirtualKeyCode::F15,
        KeyCode::F16 => VirtualKeyCode::F16,
        KeyCode::F17 => VirtualKeyCode::F17,
        KeyCode::F18 => VirtualKeyCode::F18,
        KeyCode::F19 => VirtualKeyCode::F19,
        KeyCode::F20 => VirtualKeyCode::F20,
        KeyCode::F21 => VirtualKeyCode::F21,
        KeyCode::F22 => VirtualKeyCode::F22,
        KeyCode::F23 => VirtualKeyCode::F23,
        KeyCode::F24 => VirtualKeyCode::F24,
        KeyCode::Snapshot => VirtualKeyCode::Snapshot,
        KeyCode::Scroll => VirtualKeyCode::Scroll,
        KeyCode::Pause => VirtualKeyCode::Pause,
        KeyCode::Insert => VirtualKeyCode::Insert,
        KeyCode::Home => VirtualKeyCode::Home,
        KeyCode::Delete => VirtualKeyCode::Delete,
        KeyCode::End => VirtualKeyCode::End,
        KeyCode::PageDown => VirtualKeyCode::PageDown,
        KeyCode::PageUp => VirtualKeyCode::PageUp,
        KeyCode::Left => VirtualKeyCode::Left,
        KeyCode::Up => VirtualKeyCode::Up,
        KeyCode::Right => VirtualKeyCode::Right,
        KeyCode::Down => VirtualKeyCode::Down,
        KeyCode::Backspace => VirtualKeyCode::Back,
        KeyCode::Enter => VirtualKeyCode::Return,
        KeyCode::Space => VirtualKeyCode::Space,
        KeyCode::Compose => VirtualKeyCode::Compose,
        KeyCode::Caret => VirtualKeyCode::Caret,
        KeyCode::Numlock => VirtualKeyCode::Numlock,
        KeyCode::Numpad0 => VirtualKeyCode::Numpad0,
        KeyCode::Numpad1 => VirtualKeyCode::Numpad1,
        KeyCode::Numpad2 => VirtualKeyCode::Numpad2,
        KeyCode::Numpad3 => VirtualKeyCode::Numpad3,
        KeyCode::Numpad4 => VirtualKeyCode::Numpad4,
        KeyCode::Numpad5 => VirtualKeyCode::Numpad5,
        KeyCode::Numpad6 => VirtualKeyCode::Numpad6,
        KeyCode::Numpad7 => VirtualKeyCode::Numpad7,
        KeyCode::Numpad8 => VirtualKeyCode::Numpad8,
        KeyCode::Numpad9 => VirtualKeyCode::Numpad9,
        KeyCode::AbntC1 => VirtualKeyCode::AbntC1,
        KeyCode::AbntC2 => VirtualKeyCode::AbntC2,
        KeyCode::NumpadAdd => VirtualKeyCode::NumpadAdd,
        KeyCode::Plus => VirtualKeyCode::Plus,
        KeyCode::Apostrophe => VirtualKeyCode::Apostrophe,
        KeyCode::Apps => VirtualKeyCode::Apps,
        KeyCode::At => VirtualKeyCode::At,
        KeyCode::Ax => VirtualKeyCode::Ax,
        KeyCode::Backslash => VirtualKeyCode::Backslash,
        KeyCode::Calculator => VirtualKeyCode::Calculator,
        KeyCode::Capital => VirtualKeyCode::Capital,
        KeyCode::Colon => VirtualKeyCode::Colon,
        KeyCode::Comma => VirtualKeyCode::Comma,
        KeyCode::Convert => VirtualKeyCode::Convert,
        KeyCode::NumpadDecimal => VirtualKeyCode::NumpadDecimal,
        KeyCode::NumpadDivide => VirtualKeyCode::NumpadDivide,
        KeyCode::Equals => VirtualKeyCode::Equals,
        KeyCode::Grave => VirtualKeyCode::Grave,
        KeyCode::Kana => VirtualKeyCode::Kana,
        KeyCode::Kanji => VirtualKeyCode::Kanji,
        KeyCode::LAlt => VirtualKeyCode::LAlt,
        KeyCode::LBracket => VirtualKeyCode::LBracket,
        KeyCode::LControl => VirtualKeyCode::LControl,
        KeyCode::LShift => VirtualKeyCode::LShift,
        KeyCode::LWin => VirtualKeyCode::LWin,
        KeyCode::Mail => VirtualKeyCode::Mail,
        KeyCode::MediaSelect => VirtualKeyCode::MediaSelect,
        KeyCode::MediaStop => VirtualKeyCode::MediaStop,
        KeyCode::Minus => VirtualKeyCode::Minus,
        KeyCode::NumpadMultiply => VirtualKeyCode::NumpadMultiply,
        KeyCode::Mute => VirtualKeyCode::Mute,
        KeyCode::MyComputer => VirtualKeyCode::MyComputer,
        KeyCode::NavigateForward => VirtualKeyCode::NavigateForward,
        KeyCode::NavigateBackward => VirtualKeyCode::NavigateBackward,
        KeyCode::NextTrack => VirtualKeyCode::NextTrack,
        KeyCode::NoConvert => VirtualKeyCode::NoConvert,
        KeyCode::NumpadComma => VirtualKeyCode::NumpadComma,
        KeyCode::NumpadEnter => VirtualKeyCode::NumpadEnter,
        KeyCode::NumpadEquals => VirtualKeyCode::NumpadEquals,
        KeyCode::OEM102 => VirtualKeyCode::OEM102,
        KeyCode::Period => VirtualKeyCode::Period,
        KeyCode::PlayPause => VirtualKeyCode::PlayPause,
        KeyCode::Power => VirtualKeyCode::Power,
        KeyCode::PrevTrack => VirtualKeyCode::PrevTrack,
        KeyCode::RAlt => VirtualKeyCode::RAlt,
        KeyCode::RBracket => VirtualKeyCode::RBracket,
        KeyCode::RControl => VirtualKeyCode::RControl,
        KeyCode::RShift => VirtualKeyCode::RShift,
        KeyCode::RWin => VirtualKeyCode::RWin,
        KeyCode::Semicolon => VirtualKeyCode::Semicolon,
        KeyCode::Slash => VirtualKeyCode::Slash,
        KeyCode::Sleep => VirtualKeyCode::Sleep,
        KeyCode::Stop => VirtualKeyCode::Stop,
        KeyCode::NumpadSubtract => VirtualKeyCode::NumpadSubtract,
        KeyCode::Sysrq => VirtualKeyCode::Sysrq,
        KeyCode::Tab => VirtualKeyCode::Tab,
        KeyCode::Underline => VirtualKeyCode::Underline,
        KeyCode::Unlabeled => VirtualKeyCode::Unlabeled,
        KeyCode::VolumeDown => VirtualKeyCode::VolumeDown,
        KeyCode::VolumeUp => VirtualKeyCode::VolumeUp,
        KeyCode::Wake => VirtualKeyCode::Wake,
        KeyCode::WebBack => VirtualKeyCode::WebBack,
        KeyCode::WebFavorites => VirtualKeyCode::WebFavorites,
        KeyCode::WebForward => VirtualKeyCode::WebForward,
        KeyCode::WebHome => VirtualKeyCode::WebHome,
        KeyCode::WebRefresh => VirtualKeyCode::WebRefresh,
        KeyCode::WebSearch => VirtualKeyCode::WebSearch,
        KeyCode::WebStop => VirtualKeyCode::WebStop,
        KeyCode::Yen => VirtualKeyCode::Yen,
        KeyCode::Copy => VirtualKeyCode::Copy,
        KeyCode::Paste => VirtualKeyCode::Paste,
        KeyCode::Cut => VirtualKeyCode::Cut,
        KeyCode::Asterisk => VirtualKeyCode::Asterisk,
    }
}

/// Converts a `VirtualKeyCode` from [`winit`] to an [`iced_native`] key code.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn key_code(
    virtual_keycode: winit::event::VirtualKeyCode,
) -> keyboard::KeyCode {
    use keyboard::KeyCode;

    match virtual_keycode {
        winit::event::VirtualKeyCode::Key1 => KeyCode::Key1,
        winit::event::VirtualKeyCode::Key2 => KeyCode::Key2,
        winit::event::VirtualKeyCode::Key3 => KeyCode::Key3,
        winit::event::VirtualKeyCode::Key4 => KeyCode::Key4,
        winit::event::VirtualKeyCode::Key5 => KeyCode::Key5,
        winit::event::VirtualKeyCode::Key6 => KeyCode::Key6,
        winit::event::VirtualKeyCode::Key7 => KeyCode::Key7,
        winit::event::VirtualKeyCode::Key8 => KeyCode::Key8,
        winit::event::VirtualKeyCode::Key9 => KeyCode::Key9,
        winit::event::VirtualKeyCode::Key0 => KeyCode::Key0,
        winit::event::VirtualKeyCode::A => KeyCode::A,
        winit::event::VirtualKeyCode::B => KeyCode::B,
        winit::event::VirtualKeyCode::C => KeyCode::C,
        winit::event::VirtualKeyCode::D => KeyCode::D,
        winit::event::VirtualKeyCode::E => KeyCode::E,
        winit::event::VirtualKeyCode::F => KeyCode::F,
        winit::event::VirtualKeyCode::G => KeyCode::G,
        winit::event::VirtualKeyCode::H => KeyCode::H,
        winit::event::VirtualKeyCode::I => KeyCode::I,
        winit::event::VirtualKeyCode::J => KeyCode::J,
        winit::event::VirtualKeyCode::K => KeyCode::K,
        winit::event::VirtualKeyCode::L => KeyCode::L,
        winit::event::VirtualKeyCode::M => KeyCode::M,
        winit::event::VirtualKeyCode::N => KeyCode::N,
        winit::event::VirtualKeyCode::O => KeyCode::O,
        winit::event::VirtualKeyCode::P => KeyCode::P,
        winit::event::VirtualKeyCode::Q => KeyCode::Q,
        winit::event::VirtualKeyCode::R => KeyCode::R,
        winit::event::VirtualKeyCode::S => KeyCode::S,
        winit::event::VirtualKeyCode::T => KeyCode::T,
        winit::event::VirtualKeyCode::U => KeyCode::U,
        winit::event::VirtualKeyCode::V => KeyCode::V,
        winit::event::VirtualKeyCode::W => KeyCode::W,
        winit::event::VirtualKeyCode::X => KeyCode::X,
        winit::event::VirtualKeyCode::Y => KeyCode::Y,
        winit::event::VirtualKeyCode::Z => KeyCode::Z,
        winit::event::VirtualKeyCode::Escape => KeyCode::Escape,
        winit::event::VirtualKeyCode::F1 => KeyCode::F1,
        winit::event::VirtualKeyCode::F2 => KeyCode::F2,
        winit::event::VirtualKeyCode::F3 => KeyCode::F3,
        winit::event::VirtualKeyCode::F4 => KeyCode::F4,
        winit::event::VirtualKeyCode::F5 => KeyCode::F5,
        winit::event::VirtualKeyCode::F6 => KeyCode::F6,
        winit::event::VirtualKeyCode::F7 => KeyCode::F7,
        winit::event::VirtualKeyCode::F8 => KeyCode::F8,
        winit::event::VirtualKeyCode::F9 => KeyCode::F9,
        winit::event::VirtualKeyCode::F10 => KeyCode::F10,
        winit::event::VirtualKeyCode::F11 => KeyCode::F11,
        winit::event::VirtualKeyCode::F12 => KeyCode::F12,
        winit::event::VirtualKeyCode::F13 => KeyCode::F13,
        winit::event::VirtualKeyCode::F14 => KeyCode::F14,
        winit::event::VirtualKeyCode::F15 => KeyCode::F15,
        winit::event::VirtualKeyCode::F16 => KeyCode::F16,
        winit::event::VirtualKeyCode::F17 => KeyCode::F17,
        winit::event::VirtualKeyCode::F18 => KeyCode::F18,
        winit::event::VirtualKeyCode::F19 => KeyCode::F19,
        winit::event::VirtualKeyCode::F20 => KeyCode::F20,
        winit::event::VirtualKeyCode::F21 => KeyCode::F21,
        winit::event::VirtualKeyCode::F22 => KeyCode::F22,
        winit::event::VirtualKeyCode::F23 => KeyCode::F23,
        winit::event::VirtualKeyCode::F24 => KeyCode::F24,
        winit::event::VirtualKeyCode::Snapshot => KeyCode::Snapshot,
        winit::event::VirtualKeyCode::Scroll => KeyCode::Scroll,
        winit::event::VirtualKeyCode::Pause => KeyCode::Pause,
        winit::event::VirtualKeyCode::Insert => KeyCode::Insert,
        winit::event::VirtualKeyCode::Home => KeyCode::Home,
        winit::event::VirtualKeyCode::Delete => KeyCode::Delete,
        winit::event::VirtualKeyCode::End => KeyCode::End,
        winit::event::VirtualKeyCode::PageDown => KeyCode::PageDown,
        winit::event::VirtualKeyCode::PageUp => KeyCode::PageUp,
        winit::event::VirtualKeyCode::Left => KeyCode::Left,
        winit::event::VirtualKeyCode::Up => KeyCode::Up,
        winit::event::VirtualKeyCode::Right => KeyCode::Right,
        winit::event::VirtualKeyCode::Down => KeyCode::Down,
        winit::event::VirtualKeyCode::Back => KeyCode::Backspace,
        winit::event::VirtualKeyCode::Return => KeyCode::Enter,
        winit::event::VirtualKeyCode::Space => KeyCode::Space,
        winit::event::VirtualKeyCode::Compose => KeyCode::Compose,
        winit::event::VirtualKeyCode::Caret => KeyCode::Caret,
        winit::event::VirtualKeyCode::Numlock => KeyCode::Numlock,
        winit::event::VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
        winit::event::VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
        winit::event::VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
        winit::event::VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
        winit::event::VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
        winit::event::VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
        winit::event::VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
        winit::event::VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
        winit::event::VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
        winit::event::VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
        winit::event::VirtualKeyCode::AbntC1 => KeyCode::AbntC1,
        winit::event::VirtualKeyCode::AbntC2 => KeyCode::AbntC2,
        winit::event::VirtualKeyCode::NumpadAdd => KeyCode::NumpadAdd,
        winit::event::VirtualKeyCode::Plus => KeyCode::Plus,
        winit::event::VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
        winit::event::VirtualKeyCode::Apps => KeyCode::Apps,
        winit::event::VirtualKeyCode::At => KeyCode::At,
        winit::event::VirtualKeyCode::Ax => KeyCode::Ax,
        winit::event::VirtualKeyCode::Backslash => KeyCode::Backslash,
        winit::event::VirtualKeyCode::Calculator => KeyCode::Calculator,
        winit::event::VirtualKeyCode::Capital => KeyCode::Capital,
        winit::event::VirtualKeyCode::Colon => KeyCode::Colon,
        winit::event::VirtualKeyCode::Comma => KeyCode::Comma,
        winit::event::VirtualKeyCode::Convert => KeyCode::Convert,
        winit::event::VirtualKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
        winit::event::VirtualKeyCode::NumpadDivide => KeyCode::NumpadDivide,
        winit::event::VirtualKeyCode::Equals => KeyCode::Equals,
        winit::event::VirtualKeyCode::Grave => KeyCode::Grave,
        winit::event::VirtualKeyCode::Kana => KeyCode::Kana,
        winit::event::VirtualKeyCode::Kanji => KeyCode::Kanji,
        winit::event::VirtualKeyCode::LAlt => KeyCode::LAlt,
        winit::event::VirtualKeyCode::LBracket => KeyCode::LBracket,
        winit::event::VirtualKeyCode::LControl => KeyCode::LControl,
        winit::event::VirtualKeyCode::LShift => KeyCode::LShift,
        winit::event::VirtualKeyCode::LWin => KeyCode::LWin,
        winit::event::VirtualKeyCode::Mail => KeyCode::Mail,
        winit::event::VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
        winit::event::VirtualKeyCode::MediaStop => KeyCode::MediaStop,
        winit::event::VirtualKeyCode::Minus => KeyCode::Minus,
        winit::event::VirtualKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
        winit::event::VirtualKeyCode::Mute => KeyCode::Mute,
        winit::event::VirtualKeyCode::MyComputer => KeyCode::MyComputer,
        winit::event::VirtualKeyCode::NavigateForward => {
            KeyCode::NavigateForward
        }
        winit::event::VirtualKeyCode::NavigateBackward => {
            KeyCode::NavigateBackward
        }
        winit::event::VirtualKeyCode::NextTrack => KeyCode::NextTrack,
        winit::event::VirtualKeyCode::NoConvert => KeyCode::NoConvert,
        winit::event::VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
        winit::event::VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
        winit::event::VirtualKeyCode::NumpadEquals => KeyCode::NumpadEquals,
        winit::event::VirtualKeyCode::OEM102 => KeyCode::OEM102,
        winit::event::VirtualKeyCode::Period => KeyCode::Period,
        winit::event::VirtualKeyCode::PlayPause => KeyCode::PlayPause,
        winit::event::VirtualKeyCode::Power => KeyCode::Power,
        winit::event::VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
        winit::event::VirtualKeyCode::RAlt => KeyCode::RAlt,
        winit::event::VirtualKeyCode::RBracket => KeyCode::RBracket,
        winit::event::VirtualKeyCode::RControl => KeyCode::RControl,
        winit::event::VirtualKeyCode::RShift => KeyCode::RShift,
        winit::event::VirtualKeyCode::RWin => KeyCode::RWin,
        winit::event::VirtualKeyCode::Semicolon => KeyCode::Semicolon,
        winit::event::VirtualKeyCode::Slash => KeyCode::Slash,
        winit::event::VirtualKeyCode::Sleep => KeyCode::Sleep,
        winit::event::VirtualKeyCode::Stop => KeyCode::Stop,
        winit::event::VirtualKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
        winit::event::VirtualKeyCode::Sysrq => KeyCode::Sysrq,
        winit::event::VirtualKeyCode::Tab => KeyCode::Tab,
        winit::event::VirtualKeyCode::Underline => KeyCode::Underline,
        winit::event::VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
        winit::event::VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
        winit::event::VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
        winit::event::VirtualKeyCode::Wake => KeyCode::Wake,
        winit::event::VirtualKeyCode::WebBack => KeyCode::WebBack,
        winit::event::VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
        winit::event::VirtualKeyCode::WebForward => KeyCode::WebForward,
        winit::event::VirtualKeyCode::WebHome => KeyCode::WebHome,
        winit::event::VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
        winit::event::VirtualKeyCode::WebSearch => KeyCode::WebSearch,
        winit::event::VirtualKeyCode::WebStop => KeyCode::WebStop,
        winit::event::VirtualKeyCode::Yen => KeyCode::Yen,
        winit::event::VirtualKeyCode::Copy => KeyCode::Copy,
        winit::event::VirtualKeyCode::Paste => KeyCode::Paste,
        winit::event::VirtualKeyCode::Cut => KeyCode::Cut,
        winit::event::VirtualKeyCode::Asterisk => KeyCode::Asterisk,
    }
}

// As defined in: http://www.unicode.org/faq/private_use.html
pub(crate) fn is_private_use_character(c: char) -> bool {
    match c {
        '\u{E000}'..='\u{F8FF}'
        | '\u{F0000}'..='\u{FFFFD}'
        | '\u{100000}'..='\u{10FFFD}' => true,
        _ => false,
    }
}
