//! Convert [`tao`] types into [`iced_runtime`] types, and viceversa.
//!
//! [`tao`]: https://github.com/tauri-apps/tao
//! [`iced_runtime`]: https://github.com/iced-rs/iced/tree/0.13/runtime
use tao::dpi::LogicalUnit;
use tao::dpi::PixelUnit;
use tao::window::WindowSizeConstraints;

use crate::core::input_method;
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::touch;
use crate::core::window;
use crate::core::{Event, Point, Size};

/// Converts some [`window::Settings`] into some `WindowAttributes` from `tao`.
pub fn window_attributes(
    settings: window::Settings,
    title: &str,
    primary_monitor: Option<tao::monitor::MonitorHandle>,
    _id: Option<String>,
) -> tao::window::WindowAttributes {
    let mut attributes = tao::window::WindowAttributes::default();

    attributes.title = title.into();
    attributes.inner_size =
        Some(tao::dpi::Size::Logical(tao::dpi::LogicalSize {
            width: settings.size.width.into(),
            height: settings.size.height.into(),
        }));
    attributes.maximized = settings.maximized;
    attributes.fullscreen = settings
        .fullscreen
        .then_some(tao::window::Fullscreen::Borderless(None));
    attributes.resizable = settings.resizable;
    attributes.maximizable = settings.resizable;
    attributes.minimizable = settings.resizable;
    attributes.decorations = settings.decorations;
    attributes.transparent = settings.transparent;
    attributes.window_icon = settings.icon.and_then(icon);
    attributes.visible = settings.visible;

    if let Some(position) =
        position(primary_monitor.as_ref(), settings.size, settings.position)
    {
        attributes.position = Some(position);
    }

    attributes.inner_size_constraints = WindowSizeConstraints {
        min_width: settings.min_size.and_then(|min_size| {
            Some(PixelUnit::Logical(LogicalUnit(min_size.width.into())))
        }),
        min_height: settings.min_size.and_then(|min_size| {
            Some(PixelUnit::Logical(LogicalUnit(min_size.height.into())))
        }),
        max_width: settings.max_size.and_then(|max_size| {
            Some(PixelUnit::Logical(LogicalUnit(max_size.width.into())))
        }),
        max_height: settings.max_size.and_then(|max_size| {
            Some(PixelUnit::Logical(LogicalUnit(max_size.height.into())))
        }),
    };

    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        use ::tao::platform::wayland::WindowAttributesExtWayland;

        if let Some(id) = _id {
            attributes = attributes.with_name(id.clone(), id);
        }
    }

    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowAttributesExtWindows;

        attributes = attributes
            .with_drag_and_drop(settings.platform_specific.drag_and_drop);

        attributes = attributes
            .with_skip_taskbar(settings.platform_specific.skip_taskbar);

        attributes = attributes.with_undecorated_shadow(
            settings.platform_specific.undecorated_shadow,
        );
    }

    #[cfg(target_os = "linux")]
    {
        #[cfg(feature = "x11")]
        {
            use tao::platform::x11::WindowAttributesExtX11;

            attributes = attributes
                .with_override_redirect(
                    settings.platform_specific.override_redirect,
                )
                .with_name(
                    &settings.platform_specific.application_id,
                    &settings.platform_specific.application_id,
                );
        }
        #[cfg(feature = "wayland")]
        {
            use tao::platform::wayland::WindowAttributesExtWayland;

            attributes = attributes.with_name(
                &settings.platform_specific.application_id,
                &settings.platform_specific.application_id,
            );
        }
    }

    attributes
}

/// Converts a tao window event into an iced event.
pub fn window_event(
    event: tao::event::WindowEvent<'static>,
    scale_factor: f64,
    modifiers: tao::keyboard::ModifiersState,
) -> Option<Event> {
    use tao::event::WindowEvent;

    match event {
        WindowEvent::Resized(new_size) => {
            let logical_size = new_size.to_logical(scale_factor);

            Some(Event::Window(window::Event::Resized(Size {
                width: logical_size.width,
                height: logical_size.height,
            })))
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
            let button = mouse_button(button);

            Some(Event::Mouse(match state {
                tao::event::ElementState::Pressed => {
                    mouse::Event::ButtonPressed(button)
                }
                tao::event::ElementState::Released => {
                    mouse::Event::ButtonReleased(button)
                }
                _ => todo!(),
            }))
        }
        WindowEvent::MouseWheel { delta, .. } => match delta {
            tao::event::MouseScrollDelta::LineDelta(delta_x, delta_y) => {
                Some(Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines {
                        x: delta_x,
                        y: delta_y,
                    },
                }))
            }
            tao::event::MouseScrollDelta::PixelDelta(position) => {
                Some(Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels {
                        x: position.x as f32,
                        y: position.y as f32,
                    },
                }))
            }
            _ => todo!(),
        },
        // Ignore keyboard presses/releases during window focus/unfocus
        WindowEvent::KeyboardInput { is_synthetic, .. } if is_synthetic => None,
        WindowEvent::KeyboardInput { event, .. } => Some(Event::Keyboard({
            let key = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    event.key_without_modifiers()
                }

                #[cfg(target_arch = "wasm32")]
                {
                    // TODO: Fix inconsistent API on Wasm
                    event.logical_key.clone()
                }
            };

            let text = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use crate::core::SmolStr;
                    event.text_with_all_modifiers().map(SmolStr::new)
                }

                #[cfg(target_arch = "wasm32")]
                {
                    // TODO: Fix inconsistent API on Wasm
                    event.text
                }
            }
            .filter(|text| !text.as_str().chars().any(is_private_use));

            let tao::event::KeyEvent {
                state,
                location,
                logical_key,
                physical_key,
                ..
            } = event;

            let key = self::key(key);
            let modified_key = self::key(logical_key);
            let modifiers = self::modifiers(modifiers);

            let location = match location {
                tao::keyboard::KeyLocation::Standard => {
                    keyboard::Location::Standard
                }
                tao::keyboard::KeyLocation::Left => keyboard::Location::Left,
                tao::keyboard::KeyLocation::Right => keyboard::Location::Right,
                tao::keyboard::KeyLocation::Numpad => {
                    keyboard::Location::Numpad
                }
                _ => todo!(),
            };

            match state {
                tao::event::ElementState::Pressed => {
                    keyboard::Event::KeyPressed {
                        key,
                        modified_key,
                        modifiers,
                        location,
                        text,
                    }
                }
                tao::event::ElementState::Released => {
                    keyboard::Event::KeyReleased {
                        key,
                        modified_key,
                        modifiers,
                        location,
                    }
                }
                _ => todo!(),
            }
        })),
        WindowEvent::ModifiersChanged(new_modifiers) => Some(Event::Keyboard(
            keyboard::Event::ModifiersChanged(self::modifiers(new_modifiers)),
        )),
        WindowEvent::Focused(focused) => Some(Event::Window(if focused {
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
            Some(Event::Touch(touch_event(touch, scale_factor)))
        }
        WindowEvent::Moved(position) => {
            let tao::dpi::LogicalPosition { x, y } =
                position.to_logical(scale_factor);

            Some(Event::Window(window::Event::Moved(Point::new(x, y))))
        }
        _ => None,
    }
}

/// Converts a [`window::Position`] to a [`tao`] logical position for a given monitor.
///
/// [`tao`]: https://github.com/rust-windowing/tao
pub fn position(
    monitor: Option<&tao::monitor::MonitorHandle>,
    size: Size,
    position: window::Position,
) -> Option<tao::dpi::Position> {
    match position {
        window::Position::Default => None,
        window::Position::Specific(position) => {
            Some(tao::dpi::Position::Logical(tao::dpi::LogicalPosition {
                x: f64::from(position.x),
                y: f64::from(position.y),
            }))
        }
        window::Position::SpecificWith(to_position) => {
            if let Some(monitor) = monitor {
                let start = monitor.position();

                let resolution: tao::dpi::LogicalSize<f32> =
                    monitor.size().to_logical(monitor.scale_factor());

                let position = to_position(
                    size,
                    Size::new(resolution.width, resolution.height),
                );

                let centered: tao::dpi::PhysicalPosition<i32> =
                    tao::dpi::LogicalPosition {
                        x: position.x,
                        y: position.y,
                    }
                    .to_physical(monitor.scale_factor());

                Some(tao::dpi::Position::Physical(tao::dpi::PhysicalPosition {
                    x: start.x + centered.x,
                    y: start.y + centered.y,
                }))
            } else {
                None
            }
        }
        window::Position::Centered => {
            if let Some(monitor) = monitor {
                let start = monitor.position();

                let resolution: tao::dpi::LogicalSize<f64> =
                    monitor.size().to_logical(monitor.scale_factor());

                let centered: tao::dpi::PhysicalPosition<i32> =
                    tao::dpi::LogicalPosition {
                        x: (resolution.width - f64::from(size.width)) / 2.0,
                        y: (resolution.height - f64::from(size.height)) / 2.0,
                    }
                    .to_physical(monitor.scale_factor());

                Some(tao::dpi::Position::Physical(tao::dpi::PhysicalPosition {
                    x: start.x + centered.x,
                    y: start.y + centered.y,
                }))
            } else {
                None
            }
        }
    }
}

/// Converts a [`window::Mode`] to a [`tao`] fullscreen mode.
///
/// [`tao`]: https://github.com/rust-windowing/tao
pub fn fullscreen(
    monitor: Option<tao::monitor::MonitorHandle>,
    mode: window::Mode,
) -> Option<tao::window::Fullscreen> {
    match mode {
        window::Mode::Windowed | window::Mode::Hidden => None,
        window::Mode::Fullscreen => {
            Some(tao::window::Fullscreen::Borderless(monitor))
        }
    }
}

/// Converts a [`window::Mode`] to a visibility flag.
pub fn visible(mode: window::Mode) -> bool {
    match mode {
        window::Mode::Windowed | window::Mode::Fullscreen => true,
        window::Mode::Hidden => false,
    }
}

/// Converts a [`tao`] fullscreen mode to a [`window::Mode`].
///
/// [`tao`]: https://github.com/rust-windowing/tao
pub fn mode(mode: Option<tao::window::Fullscreen>) -> window::Mode {
    match mode {
        None => window::Mode::Windowed,
        Some(_) => window::Mode::Fullscreen,
    }
}

/// Converts a [`mouse::Interaction`] to a [`tao`] cursor icon.
///
/// [`tao`]: https://github.com/rust-windowing/tao
pub fn mouse_interaction(
    interaction: mouse::Interaction,
) -> tao::window::CursorIcon {
    use mouse::Interaction;

    match interaction {
        Interaction::None | Interaction::Idle => {
            tao::window::CursorIcon::Default
        }
        Interaction::Pointer => tao::window::CursorIcon::Default,
        Interaction::Working => tao::window::CursorIcon::Progress,
        Interaction::Grab => tao::window::CursorIcon::Grab,
        Interaction::Grabbing => tao::window::CursorIcon::Grabbing,
        Interaction::Crosshair => tao::window::CursorIcon::Crosshair,
        Interaction::Text => tao::window::CursorIcon::Text,
        Interaction::ResizingHorizontally => tao::window::CursorIcon::EwResize,
        Interaction::ResizingVertically => tao::window::CursorIcon::NsResize,
        Interaction::ResizingDiagonallyUp => {
            tao::window::CursorIcon::NeswResize
        }
        Interaction::ResizingDiagonallyDown => {
            tao::window::CursorIcon::NwseResize
        }
        Interaction::NotAllowed => tao::window::CursorIcon::NotAllowed,
        Interaction::ZoomIn => tao::window::CursorIcon::ZoomIn,
        Interaction::ZoomOut => tao::window::CursorIcon::ZoomOut,
        Interaction::Cell => tao::window::CursorIcon::Cell,
        Interaction::Move => tao::window::CursorIcon::Move,
        Interaction::Copy => tao::window::CursorIcon::Copy,
        Interaction::Help => tao::window::CursorIcon::Help,
    }
}

/// Converts a `MouseButton` from [`tao`] to an [`iced`] mouse button.
///
/// [`tao`]: https://github.com/rust-windowing/tao
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn mouse_button(mouse_button: tao::event::MouseButton) -> mouse::Button {
    match mouse_button {
        tao::event::MouseButton::Left => mouse::Button::Left,
        tao::event::MouseButton::Right => mouse::Button::Right,
        tao::event::MouseButton::Middle => mouse::Button::Middle,
        tao::event::MouseButton::Other(other) => mouse::Button::Other(other),
        _ => todo!(),
    }
}

/// Converts some `ModifiersState` from [`tao`] to an [`iced`] modifiers
/// state.
///
/// [`tao`]: https://github.com/rust-windowing/tao
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn modifiers(
    modifiers: tao::keyboard::ModifiersState,
) -> keyboard::Modifiers {
    let mut result = keyboard::Modifiers::empty();

    result.set(keyboard::Modifiers::SHIFT, modifiers.shift_key());
    result.set(keyboard::Modifiers::CTRL, modifiers.control_key());
    result.set(keyboard::Modifiers::ALT, modifiers.alt_key());
    result.set(keyboard::Modifiers::LOGO, modifiers.super_key());

    result
}

/// Converts a physical cursor position to a logical `Point`.
pub fn cursor_position(
    position: tao::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
) -> Point {
    let logical_position = position.to_logical(scale_factor);

    Point::new(logical_position.x, logical_position.y)
}

/// Converts a `Touch` from [`tao`] to an [`iced`] touch event.
///
/// [`tao`]: https://github.com/rust-windowing/tao
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn touch_event(
    touch: tao::event::Touch,
    scale_factor: f64,
) -> touch::Event {
    let id = touch::Finger(touch.id);
    let position = {
        let location = touch.location.to_logical::<f64>(scale_factor);

        Point::new(location.x as f32, location.y as f32)
    };

    match touch.phase {
        tao::event::TouchPhase::Started => {
            touch::Event::FingerPressed { id, position }
        }
        tao::event::TouchPhase::Moved => {
            touch::Event::FingerMoved { id, position }
        }
        tao::event::TouchPhase::Ended => {
            touch::Event::FingerLifted { id, position }
        }
        tao::event::TouchPhase::Cancelled => {
            touch::Event::FingerLost { id, position }
        }
        _ => todo!(),
    }
}

/// Converts a `Key` from [`tao`] to an [`iced`] key.
///
/// [`tao`]: https://github.com/rust-windowing/tao
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn key(key: tao::keyboard::Key<'static>) -> keyboard::Key {
    use keyboard::key::Named;

    match key {
        tao::keyboard::Key::Character(c) => keyboard::Key::Character(c.into()),
        tao::keyboard::Key::Alt => keyboard::Key::Named(Named::Alt),
        tao::keyboard::Key::AltGraph => keyboard::Key::Named(Named::AltGraph),
        tao::keyboard::Key::CapsLock => keyboard::Key::Named(Named::CapsLock),
        tao::keyboard::Key::Control => keyboard::Key::Named(Named::Control),
        tao::keyboard::Key::Fn => keyboard::Key::Named(Named::Fn),
        tao::keyboard::Key::FnLock => keyboard::Key::Named(Named::FnLock),
        tao::keyboard::Key::NumLock => keyboard::Key::Named(Named::NumLock),
        tao::keyboard::Key::ScrollLock => {
            keyboard::Key::Named(Named::ScrollLock)
        }
        tao::keyboard::Key::Shift => keyboard::Key::Named(Named::Shift),
        tao::keyboard::Key::Symbol => keyboard::Key::Named(Named::Symbol),
        tao::keyboard::Key::SymbolLock => {
            keyboard::Key::Named(Named::SymbolLock)
        }
        tao::keyboard::Key::Super => keyboard::Key::Named(Named::Meta),
        tao::keyboard::Key::Hyper => keyboard::Key::Named(Named::Hyper),
        tao::keyboard::Key::Enter => keyboard::Key::Named(Named::Enter),
        tao::keyboard::Key::Tab => keyboard::Key::Named(Named::Tab),
        tao::keyboard::Key::Space => keyboard::Key::Named(Named::Space),
        tao::keyboard::Key::ArrowDown => keyboard::Key::Named(Named::ArrowDown),
        tao::keyboard::Key::ArrowLeft => keyboard::Key::Named(Named::ArrowLeft),
        tao::keyboard::Key::ArrowRight => {
            keyboard::Key::Named(Named::ArrowRight)
        }
        tao::keyboard::Key::ArrowUp => keyboard::Key::Named(Named::ArrowUp),
        tao::keyboard::Key::End => keyboard::Key::Named(Named::End),
        tao::keyboard::Key::Home => keyboard::Key::Named(Named::Home),
        tao::keyboard::Key::PageDown => keyboard::Key::Named(Named::PageDown),
        tao::keyboard::Key::PageUp => keyboard::Key::Named(Named::PageUp),
        tao::keyboard::Key::Backspace => keyboard::Key::Named(Named::Backspace),
        tao::keyboard::Key::Clear => keyboard::Key::Named(Named::Clear),
        tao::keyboard::Key::Copy => keyboard::Key::Named(Named::Copy),
        tao::keyboard::Key::CrSel => keyboard::Key::Named(Named::CrSel),
        tao::keyboard::Key::Cut => keyboard::Key::Named(Named::Cut),
        tao::keyboard::Key::Delete => keyboard::Key::Named(Named::Delete),
        tao::keyboard::Key::EraseEof => keyboard::Key::Named(Named::EraseEof),
        tao::keyboard::Key::ExSel => keyboard::Key::Named(Named::ExSel),
        tao::keyboard::Key::Insert => keyboard::Key::Named(Named::Insert),
        tao::keyboard::Key::Paste => keyboard::Key::Named(Named::Paste),
        tao::keyboard::Key::Redo => keyboard::Key::Named(Named::Redo),
        tao::keyboard::Key::Undo => keyboard::Key::Named(Named::Undo),
        tao::keyboard::Key::Accept => keyboard::Key::Named(Named::Accept),
        tao::keyboard::Key::Again => keyboard::Key::Named(Named::Again),
        tao::keyboard::Key::Attn => keyboard::Key::Named(Named::Attn),
        tao::keyboard::Key::Cancel => keyboard::Key::Named(Named::Cancel),
        tao::keyboard::Key::ContextMenu => {
            keyboard::Key::Named(Named::ContextMenu)
        }
        tao::keyboard::Key::Escape => keyboard::Key::Named(Named::Escape),
        tao::keyboard::Key::Execute => keyboard::Key::Named(Named::Execute),
        tao::keyboard::Key::Find => keyboard::Key::Named(Named::Find),
        tao::keyboard::Key::Help => keyboard::Key::Named(Named::Help),
        tao::keyboard::Key::Pause => keyboard::Key::Named(Named::Pause),
        tao::keyboard::Key::Play => keyboard::Key::Named(Named::Play),
        tao::keyboard::Key::Props => keyboard::Key::Named(Named::Props),
        tao::keyboard::Key::Select => keyboard::Key::Named(Named::Select),
        tao::keyboard::Key::ZoomIn => keyboard::Key::Named(Named::ZoomIn),
        tao::keyboard::Key::ZoomOut => keyboard::Key::Named(Named::ZoomOut),
        tao::keyboard::Key::BrightnessDown => {
            keyboard::Key::Named(Named::BrightnessDown)
        }
        tao::keyboard::Key::BrightnessUp => {
            keyboard::Key::Named(Named::BrightnessUp)
        }
        tao::keyboard::Key::Eject => keyboard::Key::Named(Named::Eject),
        tao::keyboard::Key::LogOff => keyboard::Key::Named(Named::LogOff),
        tao::keyboard::Key::Power => keyboard::Key::Named(Named::Power),
        tao::keyboard::Key::PowerOff => keyboard::Key::Named(Named::PowerOff),
        tao::keyboard::Key::PrintScreen => {
            keyboard::Key::Named(Named::PrintScreen)
        }
        tao::keyboard::Key::Hibernate => keyboard::Key::Named(Named::Hibernate),
        tao::keyboard::Key::Standby => keyboard::Key::Named(Named::Standby),
        tao::keyboard::Key::WakeUp => keyboard::Key::Named(Named::WakeUp),
        tao::keyboard::Key::AllCandidates => {
            keyboard::Key::Named(Named::AllCandidates)
        }
        tao::keyboard::Key::Alphanumeric => {
            keyboard::Key::Named(Named::Alphanumeric)
        }
        tao::keyboard::Key::CodeInput => keyboard::Key::Named(Named::CodeInput),
        tao::keyboard::Key::Compose => keyboard::Key::Named(Named::Compose),
        tao::keyboard::Key::Convert => keyboard::Key::Named(Named::Convert),
        tao::keyboard::Key::FinalMode => keyboard::Key::Named(Named::FinalMode),
        tao::keyboard::Key::GroupFirst => {
            keyboard::Key::Named(Named::GroupFirst)
        }
        tao::keyboard::Key::GroupLast => keyboard::Key::Named(Named::GroupLast),
        tao::keyboard::Key::GroupNext => keyboard::Key::Named(Named::GroupNext),
        tao::keyboard::Key::GroupPrevious => {
            keyboard::Key::Named(Named::GroupPrevious)
        }
        tao::keyboard::Key::ModeChange => {
            keyboard::Key::Named(Named::ModeChange)
        }
        tao::keyboard::Key::NextCandidate => {
            keyboard::Key::Named(Named::NextCandidate)
        }
        tao::keyboard::Key::NonConvert => {
            keyboard::Key::Named(Named::NonConvert)
        }
        tao::keyboard::Key::PreviousCandidate => {
            keyboard::Key::Named(Named::PreviousCandidate)
        }
        tao::keyboard::Key::Process => keyboard::Key::Named(Named::Process),
        tao::keyboard::Key::SingleCandidate => {
            keyboard::Key::Named(Named::SingleCandidate)
        }
        tao::keyboard::Key::HangulMode => {
            keyboard::Key::Named(Named::HangulMode)
        }
        tao::keyboard::Key::HanjaMode => keyboard::Key::Named(Named::HanjaMode),
        tao::keyboard::Key::JunjaMode => keyboard::Key::Named(Named::JunjaMode),
        tao::keyboard::Key::Eisu => keyboard::Key::Named(Named::Eisu),
        tao::keyboard::Key::Hankaku => keyboard::Key::Named(Named::Hankaku),
        tao::keyboard::Key::Hiragana => keyboard::Key::Named(Named::Hiragana),
        tao::keyboard::Key::HiraganaKatakana => {
            keyboard::Key::Named(Named::HiraganaKatakana)
        }
        tao::keyboard::Key::KanaMode => keyboard::Key::Named(Named::KanaMode),
        tao::keyboard::Key::KanjiMode => keyboard::Key::Named(Named::KanjiMode),
        tao::keyboard::Key::Katakana => keyboard::Key::Named(Named::Katakana),
        tao::keyboard::Key::Romaji => keyboard::Key::Named(Named::Romaji),
        tao::keyboard::Key::Zenkaku => keyboard::Key::Named(Named::Zenkaku),
        tao::keyboard::Key::ZenkakuHankaku => {
            keyboard::Key::Named(Named::ZenkakuHankaku)
        }
        tao::keyboard::Key::Soft1 => keyboard::Key::Named(Named::Soft1),
        tao::keyboard::Key::Soft2 => keyboard::Key::Named(Named::Soft2),
        tao::keyboard::Key::Soft3 => keyboard::Key::Named(Named::Soft3),
        tao::keyboard::Key::Soft4 => keyboard::Key::Named(Named::Soft4),
        tao::keyboard::Key::ChannelDown => {
            keyboard::Key::Named(Named::ChannelDown)
        }
        tao::keyboard::Key::ChannelUp => keyboard::Key::Named(Named::ChannelUp),
        tao::keyboard::Key::Close => keyboard::Key::Named(Named::Close),
        tao::keyboard::Key::MailForward => {
            keyboard::Key::Named(Named::MailForward)
        }
        tao::keyboard::Key::MailReply => keyboard::Key::Named(Named::MailReply),
        tao::keyboard::Key::MailSend => keyboard::Key::Named(Named::MailSend),
        tao::keyboard::Key::MediaClose => {
            keyboard::Key::Named(Named::MediaClose)
        }
        tao::keyboard::Key::MediaFastForward => {
            keyboard::Key::Named(Named::MediaFastForward)
        }
        tao::keyboard::Key::MediaPause => {
            keyboard::Key::Named(Named::MediaPause)
        }
        tao::keyboard::Key::MediaPlay => keyboard::Key::Named(Named::MediaPlay),
        tao::keyboard::Key::MediaPlayPause => {
            keyboard::Key::Named(Named::MediaPlayPause)
        }
        tao::keyboard::Key::MediaRecord => {
            keyboard::Key::Named(Named::MediaRecord)
        }
        tao::keyboard::Key::MediaRewind => {
            keyboard::Key::Named(Named::MediaRewind)
        }
        tao::keyboard::Key::MediaStop => keyboard::Key::Named(Named::MediaStop),
        tao::keyboard::Key::MediaTrackNext => {
            keyboard::Key::Named(Named::MediaTrackNext)
        }
        tao::keyboard::Key::MediaTrackPrevious => {
            keyboard::Key::Named(Named::MediaTrackPrevious)
        }
        tao::keyboard::Key::New => keyboard::Key::Named(Named::New),
        tao::keyboard::Key::Open => keyboard::Key::Named(Named::Open),
        tao::keyboard::Key::Print => keyboard::Key::Named(Named::Print),
        tao::keyboard::Key::Save => keyboard::Key::Named(Named::Save),
        tao::keyboard::Key::SpellCheck => {
            keyboard::Key::Named(Named::SpellCheck)
        }
        tao::keyboard::Key::Key11 => keyboard::Key::Named(Named::Key11),
        tao::keyboard::Key::Key12 => keyboard::Key::Named(Named::Key12),
        tao::keyboard::Key::AudioBalanceLeft => {
            keyboard::Key::Named(Named::AudioBalanceLeft)
        }
        tao::keyboard::Key::AudioBalanceRight => {
            keyboard::Key::Named(Named::AudioBalanceRight)
        }
        tao::keyboard::Key::AudioBassBoostDown => {
            keyboard::Key::Named(Named::AudioBassBoostDown)
        }
        tao::keyboard::Key::AudioBassBoostToggle => {
            keyboard::Key::Named(Named::AudioBassBoostToggle)
        }
        tao::keyboard::Key::AudioBassBoostUp => {
            keyboard::Key::Named(Named::AudioBassBoostUp)
        }
        tao::keyboard::Key::AudioFaderFront => {
            keyboard::Key::Named(Named::AudioFaderFront)
        }
        tao::keyboard::Key::AudioFaderRear => {
            keyboard::Key::Named(Named::AudioFaderRear)
        }
        tao::keyboard::Key::AudioSurroundModeNext => {
            keyboard::Key::Named(Named::AudioSurroundModeNext)
        }
        tao::keyboard::Key::AudioTrebleDown => {
            keyboard::Key::Named(Named::AudioTrebleDown)
        }
        tao::keyboard::Key::AudioTrebleUp => {
            keyboard::Key::Named(Named::AudioTrebleUp)
        }
        tao::keyboard::Key::AudioVolumeDown => {
            keyboard::Key::Named(Named::AudioVolumeDown)
        }
        tao::keyboard::Key::AudioVolumeUp => {
            keyboard::Key::Named(Named::AudioVolumeUp)
        }
        tao::keyboard::Key::AudioVolumeMute => {
            keyboard::Key::Named(Named::AudioVolumeMute)
        }
        tao::keyboard::Key::MicrophoneToggle => {
            keyboard::Key::Named(Named::MicrophoneToggle)
        }
        tao::keyboard::Key::MicrophoneVolumeDown => {
            keyboard::Key::Named(Named::MicrophoneVolumeDown)
        }
        tao::keyboard::Key::MicrophoneVolumeUp => {
            keyboard::Key::Named(Named::MicrophoneVolumeUp)
        }
        tao::keyboard::Key::MicrophoneVolumeMute => {
            keyboard::Key::Named(Named::MicrophoneVolumeMute)
        }
        tao::keyboard::Key::SpeechCorrectionList => {
            keyboard::Key::Named(Named::SpeechCorrectionList)
        }
        tao::keyboard::Key::SpeechInputToggle => {
            keyboard::Key::Named(Named::SpeechInputToggle)
        }
        tao::keyboard::Key::LaunchApplication1 => {
            keyboard::Key::Named(Named::LaunchApplication1)
        }
        tao::keyboard::Key::LaunchApplication2 => {
            keyboard::Key::Named(Named::LaunchApplication2)
        }
        tao::keyboard::Key::LaunchCalendar => {
            keyboard::Key::Named(Named::LaunchCalendar)
        }
        tao::keyboard::Key::LaunchContacts => {
            keyboard::Key::Named(Named::LaunchContacts)
        }
        tao::keyboard::Key::LaunchMail => {
            keyboard::Key::Named(Named::LaunchMail)
        }
        tao::keyboard::Key::LaunchMediaPlayer => {
            keyboard::Key::Named(Named::LaunchMediaPlayer)
        }
        tao::keyboard::Key::LaunchMusicPlayer => {
            keyboard::Key::Named(Named::LaunchMusicPlayer)
        }
        tao::keyboard::Key::LaunchPhone => {
            keyboard::Key::Named(Named::LaunchPhone)
        }
        tao::keyboard::Key::LaunchScreenSaver => {
            keyboard::Key::Named(Named::LaunchScreenSaver)
        }
        tao::keyboard::Key::LaunchSpreadsheet => {
            keyboard::Key::Named(Named::LaunchSpreadsheet)
        }
        tao::keyboard::Key::LaunchWebBrowser => {
            keyboard::Key::Named(Named::LaunchWebBrowser)
        }
        tao::keyboard::Key::LaunchWebCam => {
            keyboard::Key::Named(Named::LaunchWebCam)
        }
        tao::keyboard::Key::LaunchWordProcessor => {
            keyboard::Key::Named(Named::LaunchWordProcessor)
        }
        tao::keyboard::Key::BrowserBack => {
            keyboard::Key::Named(Named::BrowserBack)
        }
        tao::keyboard::Key::BrowserFavorites => {
            keyboard::Key::Named(Named::BrowserFavorites)
        }
        tao::keyboard::Key::BrowserForward => {
            keyboard::Key::Named(Named::BrowserForward)
        }
        tao::keyboard::Key::BrowserHome => {
            keyboard::Key::Named(Named::BrowserHome)
        }
        tao::keyboard::Key::BrowserRefresh => {
            keyboard::Key::Named(Named::BrowserRefresh)
        }
        tao::keyboard::Key::BrowserSearch => {
            keyboard::Key::Named(Named::BrowserSearch)
        }
        tao::keyboard::Key::BrowserStop => {
            keyboard::Key::Named(Named::BrowserStop)
        }
        tao::keyboard::Key::AppSwitch => keyboard::Key::Named(Named::AppSwitch),
        tao::keyboard::Key::Call => keyboard::Key::Named(Named::Call),
        tao::keyboard::Key::Camera => keyboard::Key::Named(Named::Camera),
        tao::keyboard::Key::CameraFocus => {
            keyboard::Key::Named(Named::CameraFocus)
        }
        tao::keyboard::Key::EndCall => keyboard::Key::Named(Named::EndCall),
        tao::keyboard::Key::GoBack => keyboard::Key::Named(Named::GoBack),
        tao::keyboard::Key::GoHome => keyboard::Key::Named(Named::GoHome),
        tao::keyboard::Key::HeadsetHook => {
            keyboard::Key::Named(Named::HeadsetHook)
        }
        tao::keyboard::Key::LastNumberRedial => {
            keyboard::Key::Named(Named::LastNumberRedial)
        }
        tao::keyboard::Key::Notification => {
            keyboard::Key::Named(Named::Notification)
        }
        tao::keyboard::Key::MannerMode => {
            keyboard::Key::Named(Named::MannerMode)
        }
        tao::keyboard::Key::VoiceDial => keyboard::Key::Named(Named::VoiceDial),
        tao::keyboard::Key::TV => keyboard::Key::Named(Named::TV),
        tao::keyboard::Key::TV3DMode => keyboard::Key::Named(Named::TV3DMode),
        tao::keyboard::Key::TVAntennaCable => {
            keyboard::Key::Named(Named::TVAntennaCable)
        }
        tao::keyboard::Key::TVAudioDescription => {
            keyboard::Key::Named(Named::TVAudioDescription)
        }
        tao::keyboard::Key::TVAudioDescriptionMixDown => {
            keyboard::Key::Named(Named::TVAudioDescriptionMixDown)
        }
        tao::keyboard::Key::TVAudioDescriptionMixUp => {
            keyboard::Key::Named(Named::TVAudioDescriptionMixUp)
        }
        tao::keyboard::Key::TVContentsMenu => {
            keyboard::Key::Named(Named::TVContentsMenu)
        }
        tao::keyboard::Key::TVDataService => {
            keyboard::Key::Named(Named::TVDataService)
        }
        tao::keyboard::Key::TVInput => keyboard::Key::Named(Named::TVInput),
        tao::keyboard::Key::TVInputComponent1 => {
            keyboard::Key::Named(Named::TVInputComponent1)
        }
        tao::keyboard::Key::TVInputComponent2 => {
            keyboard::Key::Named(Named::TVInputComponent2)
        }
        tao::keyboard::Key::TVInputComposite1 => {
            keyboard::Key::Named(Named::TVInputComposite1)
        }
        tao::keyboard::Key::TVInputComposite2 => {
            keyboard::Key::Named(Named::TVInputComposite2)
        }
        tao::keyboard::Key::TVInputHDMI1 => {
            keyboard::Key::Named(Named::TVInputHDMI1)
        }
        tao::keyboard::Key::TVInputHDMI2 => {
            keyboard::Key::Named(Named::TVInputHDMI2)
        }
        tao::keyboard::Key::TVInputHDMI3 => {
            keyboard::Key::Named(Named::TVInputHDMI3)
        }
        tao::keyboard::Key::TVInputHDMI4 => {
            keyboard::Key::Named(Named::TVInputHDMI4)
        }
        tao::keyboard::Key::TVInputVGA1 => {
            keyboard::Key::Named(Named::TVInputVGA1)
        }
        tao::keyboard::Key::TVMediaContext => {
            keyboard::Key::Named(Named::TVMediaContext)
        }
        tao::keyboard::Key::TVNetwork => keyboard::Key::Named(Named::TVNetwork),
        tao::keyboard::Key::TVNumberEntry => {
            keyboard::Key::Named(Named::TVNumberEntry)
        }
        tao::keyboard::Key::TVPower => keyboard::Key::Named(Named::TVPower),
        tao::keyboard::Key::TVRadioService => {
            keyboard::Key::Named(Named::TVRadioService)
        }
        tao::keyboard::Key::TVSatellite => {
            keyboard::Key::Named(Named::TVSatellite)
        }
        tao::keyboard::Key::TVSatelliteBS => {
            keyboard::Key::Named(Named::TVSatelliteBS)
        }
        tao::keyboard::Key::TVSatelliteCS => {
            keyboard::Key::Named(Named::TVSatelliteCS)
        }
        tao::keyboard::Key::TVSatelliteToggle => {
            keyboard::Key::Named(Named::TVSatelliteToggle)
        }
        tao::keyboard::Key::TVTerrestrialAnalog => {
            keyboard::Key::Named(Named::TVTerrestrialAnalog)
        }
        tao::keyboard::Key::TVTerrestrialDigital => {
            keyboard::Key::Named(Named::TVTerrestrialDigital)
        }
        tao::keyboard::Key::TVTimer => keyboard::Key::Named(Named::TVTimer),
        tao::keyboard::Key::AVRInput => keyboard::Key::Named(Named::AVRInput),
        tao::keyboard::Key::AVRPower => keyboard::Key::Named(Named::AVRPower),
        tao::keyboard::Key::ColorF0Red => {
            keyboard::Key::Named(Named::ColorF0Red)
        }
        tao::keyboard::Key::ColorF1Green => {
            keyboard::Key::Named(Named::ColorF1Green)
        }
        tao::keyboard::Key::ColorF2Yellow => {
            keyboard::Key::Named(Named::ColorF2Yellow)
        }
        tao::keyboard::Key::ColorF3Blue => {
            keyboard::Key::Named(Named::ColorF3Blue)
        }
        tao::keyboard::Key::ColorF4Grey => {
            keyboard::Key::Named(Named::ColorF4Grey)
        }
        tao::keyboard::Key::ColorF5Brown => {
            keyboard::Key::Named(Named::ColorF5Brown)
        }
        tao::keyboard::Key::ClosedCaptionToggle => {
            keyboard::Key::Named(Named::ClosedCaptionToggle)
        }
        tao::keyboard::Key::Dimmer => keyboard::Key::Named(Named::Dimmer),
        tao::keyboard::Key::DisplaySwap => {
            keyboard::Key::Named(Named::DisplaySwap)
        }
        tao::keyboard::Key::DVR => keyboard::Key::Named(Named::DVR),
        tao::keyboard::Key::Exit => keyboard::Key::Named(Named::Exit),
        tao::keyboard::Key::FavoriteClear0 => {
            keyboard::Key::Named(Named::FavoriteClear0)
        }
        tao::keyboard::Key::FavoriteClear1 => {
            keyboard::Key::Named(Named::FavoriteClear1)
        }
        tao::keyboard::Key::FavoriteClear2 => {
            keyboard::Key::Named(Named::FavoriteClear2)
        }
        tao::keyboard::Key::FavoriteClear3 => {
            keyboard::Key::Named(Named::FavoriteClear3)
        }
        tao::keyboard::Key::FavoriteRecall0 => {
            keyboard::Key::Named(Named::FavoriteRecall0)
        }
        tao::keyboard::Key::FavoriteRecall1 => {
            keyboard::Key::Named(Named::FavoriteRecall1)
        }
        tao::keyboard::Key::FavoriteRecall2 => {
            keyboard::Key::Named(Named::FavoriteRecall2)
        }
        tao::keyboard::Key::FavoriteRecall3 => {
            keyboard::Key::Named(Named::FavoriteRecall3)
        }
        tao::keyboard::Key::FavoriteStore0 => {
            keyboard::Key::Named(Named::FavoriteStore0)
        }
        tao::keyboard::Key::FavoriteStore1 => {
            keyboard::Key::Named(Named::FavoriteStore1)
        }
        tao::keyboard::Key::FavoriteStore2 => {
            keyboard::Key::Named(Named::FavoriteStore2)
        }
        tao::keyboard::Key::FavoriteStore3 => {
            keyboard::Key::Named(Named::FavoriteStore3)
        }
        tao::keyboard::Key::Guide => keyboard::Key::Named(Named::Guide),
        tao::keyboard::Key::GuideNextDay => {
            keyboard::Key::Named(Named::GuideNextDay)
        }
        tao::keyboard::Key::GuidePreviousDay => {
            keyboard::Key::Named(Named::GuidePreviousDay)
        }
        tao::keyboard::Key::Info => keyboard::Key::Named(Named::Info),
        tao::keyboard::Key::InstantReplay => {
            keyboard::Key::Named(Named::InstantReplay)
        }
        tao::keyboard::Key::Link => keyboard::Key::Named(Named::Link),
        tao::keyboard::Key::ListProgram => {
            keyboard::Key::Named(Named::ListProgram)
        }
        tao::keyboard::Key::LiveContent => {
            keyboard::Key::Named(Named::LiveContent)
        }
        tao::keyboard::Key::Lock => keyboard::Key::Named(Named::Lock),
        tao::keyboard::Key::MediaApps => keyboard::Key::Named(Named::MediaApps),
        tao::keyboard::Key::MediaAudioTrack => {
            keyboard::Key::Named(Named::MediaAudioTrack)
        }
        tao::keyboard::Key::MediaLast => keyboard::Key::Named(Named::MediaLast),
        tao::keyboard::Key::MediaSkipBackward => {
            keyboard::Key::Named(Named::MediaSkipBackward)
        }
        tao::keyboard::Key::MediaSkipForward => {
            keyboard::Key::Named(Named::MediaSkipForward)
        }
        tao::keyboard::Key::MediaStepBackward => {
            keyboard::Key::Named(Named::MediaStepBackward)
        }
        tao::keyboard::Key::MediaStepForward => {
            keyboard::Key::Named(Named::MediaStepForward)
        }
        tao::keyboard::Key::MediaTopMenu => {
            keyboard::Key::Named(Named::MediaTopMenu)
        }
        tao::keyboard::Key::NavigateIn => {
            keyboard::Key::Named(Named::NavigateIn)
        }
        tao::keyboard::Key::NavigateNext => {
            keyboard::Key::Named(Named::NavigateNext)
        }
        tao::keyboard::Key::NavigateOut => {
            keyboard::Key::Named(Named::NavigateOut)
        }
        tao::keyboard::Key::NavigatePrevious => {
            keyboard::Key::Named(Named::NavigatePrevious)
        }
        tao::keyboard::Key::NextFavoriteChannel => {
            keyboard::Key::Named(Named::NextFavoriteChannel)
        }
        tao::keyboard::Key::NextUserProfile => {
            keyboard::Key::Named(Named::NextUserProfile)
        }
        tao::keyboard::Key::OnDemand => keyboard::Key::Named(Named::OnDemand),
        tao::keyboard::Key::Pairing => keyboard::Key::Named(Named::Pairing),
        tao::keyboard::Key::PinPDown => keyboard::Key::Named(Named::PinPDown),
        tao::keyboard::Key::PinPMove => keyboard::Key::Named(Named::PinPMove),
        tao::keyboard::Key::PinPToggle => {
            keyboard::Key::Named(Named::PinPToggle)
        }
        tao::keyboard::Key::PinPUp => keyboard::Key::Named(Named::PinPUp),
        tao::keyboard::Key::PlaySpeedDown => {
            keyboard::Key::Named(Named::PlaySpeedDown)
        }
        tao::keyboard::Key::PlaySpeedReset => {
            keyboard::Key::Named(Named::PlaySpeedReset)
        }
        tao::keyboard::Key::PlaySpeedUp => {
            keyboard::Key::Named(Named::PlaySpeedUp)
        }
        tao::keyboard::Key::RandomToggle => {
            keyboard::Key::Named(Named::RandomToggle)
        }
        tao::keyboard::Key::RcLowBattery => {
            keyboard::Key::Named(Named::RcLowBattery)
        }
        tao::keyboard::Key::RecordSpeedNext => {
            keyboard::Key::Named(Named::RecordSpeedNext)
        }
        tao::keyboard::Key::RfBypass => keyboard::Key::Named(Named::RfBypass),
        tao::keyboard::Key::ScanChannelsToggle => {
            keyboard::Key::Named(Named::ScanChannelsToggle)
        }
        tao::keyboard::Key::ScreenModeNext => {
            keyboard::Key::Named(Named::ScreenModeNext)
        }
        tao::keyboard::Key::Settings => keyboard::Key::Named(Named::Settings),
        tao::keyboard::Key::SplitScreenToggle => {
            keyboard::Key::Named(Named::SplitScreenToggle)
        }
        tao::keyboard::Key::STBInput => keyboard::Key::Named(Named::STBInput),
        tao::keyboard::Key::STBPower => keyboard::Key::Named(Named::STBPower),
        tao::keyboard::Key::Subtitle => keyboard::Key::Named(Named::Subtitle),
        tao::keyboard::Key::Teletext => keyboard::Key::Named(Named::Teletext),
        tao::keyboard::Key::VideoModeNext => {
            keyboard::Key::Named(Named::VideoModeNext)
        }
        tao::keyboard::Key::Wink => keyboard::Key::Named(Named::Wink),
        tao::keyboard::Key::ZoomToggle => {
            keyboard::Key::Named(Named::ZoomToggle)
        }
        tao::keyboard::Key::F1 => keyboard::Key::Named(Named::F1),
        tao::keyboard::Key::F2 => keyboard::Key::Named(Named::F2),
        tao::keyboard::Key::F3 => keyboard::Key::Named(Named::F3),
        tao::keyboard::Key::F4 => keyboard::Key::Named(Named::F4),
        tao::keyboard::Key::F5 => keyboard::Key::Named(Named::F5),
        tao::keyboard::Key::F6 => keyboard::Key::Named(Named::F6),
        tao::keyboard::Key::F7 => keyboard::Key::Named(Named::F7),
        tao::keyboard::Key::F8 => keyboard::Key::Named(Named::F8),
        tao::keyboard::Key::F9 => keyboard::Key::Named(Named::F9),
        tao::keyboard::Key::F10 => keyboard::Key::Named(Named::F10),
        tao::keyboard::Key::F11 => keyboard::Key::Named(Named::F11),
        tao::keyboard::Key::F12 => keyboard::Key::Named(Named::F12),
        tao::keyboard::Key::F13 => keyboard::Key::Named(Named::F13),
        tao::keyboard::Key::F14 => keyboard::Key::Named(Named::F14),
        tao::keyboard::Key::F15 => keyboard::Key::Named(Named::F15),
        tao::keyboard::Key::F16 => keyboard::Key::Named(Named::F16),
        tao::keyboard::Key::F17 => keyboard::Key::Named(Named::F17),
        tao::keyboard::Key::F18 => keyboard::Key::Named(Named::F18),
        tao::keyboard::Key::F19 => keyboard::Key::Named(Named::F19),
        tao::keyboard::Key::F20 => keyboard::Key::Named(Named::F20),
        tao::keyboard::Key::F21 => keyboard::Key::Named(Named::F21),
        tao::keyboard::Key::F22 => keyboard::Key::Named(Named::F22),
        tao::keyboard::Key::F23 => keyboard::Key::Named(Named::F23),
        tao::keyboard::Key::F24 => keyboard::Key::Named(Named::F24),
        tao::keyboard::Key::F25 => keyboard::Key::Named(Named::F25),
        tao::keyboard::Key::F26 => keyboard::Key::Named(Named::F26),
        tao::keyboard::Key::F27 => keyboard::Key::Named(Named::F27),
        tao::keyboard::Key::F28 => keyboard::Key::Named(Named::F28),
        tao::keyboard::Key::F29 => keyboard::Key::Named(Named::F29),
        tao::keyboard::Key::F30 => keyboard::Key::Named(Named::F30),
        tao::keyboard::Key::F31 => keyboard::Key::Named(Named::F31),
        tao::keyboard::Key::F32 => keyboard::Key::Named(Named::F32),
        tao::keyboard::Key::F33 => keyboard::Key::Named(Named::F33),
        tao::keyboard::Key::F34 => keyboard::Key::Named(Named::F34),
        tao::keyboard::Key::F35 => keyboard::Key::Named(Named::F35),
        _ => return keyboard::Key::Unidentified,
    }
}

/// Converts a `KeyCode` from [`tao`] to an [`iced`] key code.
///
/// [`tao`]: https://github.com/rust-windowing/tao
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn key_code(
    key_code: tao::keyboard::KeyCode,
) -> Option<keyboard::key::Code> {
    use tao::keyboard::KeyCode;

    Some(match key_code {
        KeyCode::Backquote => keyboard::key::Code::Backquote,
        KeyCode::Backslash => keyboard::key::Code::Backslash,
        KeyCode::BracketLeft => keyboard::key::Code::BracketLeft,
        KeyCode::BracketRight => keyboard::key::Code::BracketRight,
        KeyCode::Comma => keyboard::key::Code::Comma,
        KeyCode::Digit0 => keyboard::key::Code::Digit0,
        KeyCode::Digit1 => keyboard::key::Code::Digit1,
        KeyCode::Digit2 => keyboard::key::Code::Digit2,
        KeyCode::Digit3 => keyboard::key::Code::Digit3,
        KeyCode::Digit4 => keyboard::key::Code::Digit4,
        KeyCode::Digit5 => keyboard::key::Code::Digit5,
        KeyCode::Digit6 => keyboard::key::Code::Digit6,
        KeyCode::Digit7 => keyboard::key::Code::Digit7,
        KeyCode::Digit8 => keyboard::key::Code::Digit8,
        KeyCode::Digit9 => keyboard::key::Code::Digit9,
        KeyCode::Equal => keyboard::key::Code::Equal,
        KeyCode::IntlBackslash => keyboard::key::Code::IntlBackslash,
        KeyCode::IntlRo => keyboard::key::Code::IntlRo,
        KeyCode::IntlYen => keyboard::key::Code::IntlYen,
        KeyCode::KeyA => keyboard::key::Code::KeyA,
        KeyCode::KeyB => keyboard::key::Code::KeyB,
        KeyCode::KeyC => keyboard::key::Code::KeyC,
        KeyCode::KeyD => keyboard::key::Code::KeyD,
        KeyCode::KeyE => keyboard::key::Code::KeyE,
        KeyCode::KeyF => keyboard::key::Code::KeyF,
        KeyCode::KeyG => keyboard::key::Code::KeyG,
        KeyCode::KeyH => keyboard::key::Code::KeyH,
        KeyCode::KeyI => keyboard::key::Code::KeyI,
        KeyCode::KeyJ => keyboard::key::Code::KeyJ,
        KeyCode::KeyK => keyboard::key::Code::KeyK,
        KeyCode::KeyL => keyboard::key::Code::KeyL,
        KeyCode::KeyM => keyboard::key::Code::KeyM,
        KeyCode::KeyN => keyboard::key::Code::KeyN,
        KeyCode::KeyO => keyboard::key::Code::KeyO,
        KeyCode::KeyP => keyboard::key::Code::KeyP,
        KeyCode::KeyQ => keyboard::key::Code::KeyQ,
        KeyCode::KeyR => keyboard::key::Code::KeyR,
        KeyCode::KeyS => keyboard::key::Code::KeyS,
        KeyCode::KeyT => keyboard::key::Code::KeyT,
        KeyCode::KeyU => keyboard::key::Code::KeyU,
        KeyCode::KeyV => keyboard::key::Code::KeyV,
        KeyCode::KeyW => keyboard::key::Code::KeyW,
        KeyCode::KeyX => keyboard::key::Code::KeyX,
        KeyCode::KeyY => keyboard::key::Code::KeyY,
        KeyCode::KeyZ => keyboard::key::Code::KeyZ,
        KeyCode::Minus => keyboard::key::Code::Minus,
        KeyCode::Period => keyboard::key::Code::Period,
        KeyCode::Quote => keyboard::key::Code::Quote,
        KeyCode::Semicolon => keyboard::key::Code::Semicolon,
        KeyCode::Slash => keyboard::key::Code::Slash,
        KeyCode::AltLeft => keyboard::key::Code::AltLeft,
        KeyCode::AltRight => keyboard::key::Code::AltRight,
        KeyCode::Backspace => keyboard::key::Code::Backspace,
        KeyCode::CapsLock => keyboard::key::Code::CapsLock,
        KeyCode::ContextMenu => keyboard::key::Code::ContextMenu,
        KeyCode::ControlLeft => keyboard::key::Code::ControlLeft,
        KeyCode::ControlRight => keyboard::key::Code::ControlRight,
        KeyCode::Enter => keyboard::key::Code::Enter,
        KeyCode::SuperLeft => keyboard::key::Code::SuperLeft,
        KeyCode::SuperRight => keyboard::key::Code::SuperRight,
        KeyCode::ShiftLeft => keyboard::key::Code::ShiftLeft,
        KeyCode::ShiftRight => keyboard::key::Code::ShiftRight,
        KeyCode::Space => keyboard::key::Code::Space,
        KeyCode::Tab => keyboard::key::Code::Tab,
        KeyCode::Convert => keyboard::key::Code::Convert,
        KeyCode::KanaMode => keyboard::key::Code::KanaMode,
        KeyCode::Lang1 => keyboard::key::Code::Lang1,
        KeyCode::Lang2 => keyboard::key::Code::Lang2,
        KeyCode::Lang3 => keyboard::key::Code::Lang3,
        KeyCode::Lang4 => keyboard::key::Code::Lang4,
        KeyCode::Lang5 => keyboard::key::Code::Lang5,
        KeyCode::NonConvert => keyboard::key::Code::NonConvert,
        KeyCode::Delete => keyboard::key::Code::Delete,
        KeyCode::End => keyboard::key::Code::End,
        KeyCode::Help => keyboard::key::Code::Help,
        KeyCode::Home => keyboard::key::Code::Home,
        KeyCode::Insert => keyboard::key::Code::Insert,
        KeyCode::PageDown => keyboard::key::Code::PageDown,
        KeyCode::PageUp => keyboard::key::Code::PageUp,
        KeyCode::ArrowDown => keyboard::key::Code::ArrowDown,
        KeyCode::ArrowLeft => keyboard::key::Code::ArrowLeft,
        KeyCode::ArrowRight => keyboard::key::Code::ArrowRight,
        KeyCode::ArrowUp => keyboard::key::Code::ArrowUp,
        KeyCode::NumLock => keyboard::key::Code::NumLock,
        KeyCode::Numpad0 => keyboard::key::Code::Numpad0,
        KeyCode::Numpad1 => keyboard::key::Code::Numpad1,
        KeyCode::Numpad2 => keyboard::key::Code::Numpad2,
        KeyCode::Numpad3 => keyboard::key::Code::Numpad3,
        KeyCode::Numpad4 => keyboard::key::Code::Numpad4,
        KeyCode::Numpad5 => keyboard::key::Code::Numpad5,
        KeyCode::Numpad6 => keyboard::key::Code::Numpad6,
        KeyCode::Numpad7 => keyboard::key::Code::Numpad7,
        KeyCode::Numpad8 => keyboard::key::Code::Numpad8,
        KeyCode::Numpad9 => keyboard::key::Code::Numpad9,
        KeyCode::NumpadAdd => keyboard::key::Code::NumpadAdd,
        KeyCode::NumpadBackspace => keyboard::key::Code::NumpadBackspace,
        KeyCode::NumpadClear => keyboard::key::Code::NumpadClear,
        KeyCode::NumpadClearEntry => keyboard::key::Code::NumpadClearEntry,
        KeyCode::NumpadComma => keyboard::key::Code::NumpadComma,
        KeyCode::NumpadDecimal => keyboard::key::Code::NumpadDecimal,
        KeyCode::NumpadDivide => keyboard::key::Code::NumpadDivide,
        KeyCode::NumpadEnter => keyboard::key::Code::NumpadEnter,
        KeyCode::NumpadEqual => keyboard::key::Code::NumpadEqual,
        KeyCode::NumpadHash => keyboard::key::Code::NumpadHash,
        KeyCode::NumpadMemoryAdd => keyboard::key::Code::NumpadMemoryAdd,
        KeyCode::NumpadMemoryClear => keyboard::key::Code::NumpadMemoryClear,
        KeyCode::NumpadMemoryRecall => keyboard::key::Code::NumpadMemoryRecall,
        KeyCode::NumpadMemoryStore => keyboard::key::Code::NumpadMemoryStore,
        KeyCode::NumpadMemorySubtract => {
            keyboard::key::Code::NumpadMemorySubtract
        }
        KeyCode::NumpadMultiply => keyboard::key::Code::NumpadMultiply,
        KeyCode::NumpadParenLeft => keyboard::key::Code::NumpadParenLeft,
        KeyCode::NumpadParenRight => keyboard::key::Code::NumpadParenRight,
        KeyCode::NumpadStar => keyboard::key::Code::NumpadStar,
        KeyCode::NumpadSubtract => keyboard::key::Code::NumpadSubtract,
        KeyCode::Escape => keyboard::key::Code::Escape,
        KeyCode::Fn => keyboard::key::Code::Fn,
        KeyCode::FnLock => keyboard::key::Code::FnLock,
        KeyCode::PrintScreen => keyboard::key::Code::PrintScreen,
        KeyCode::ScrollLock => keyboard::key::Code::ScrollLock,
        KeyCode::Pause => keyboard::key::Code::Pause,
        KeyCode::BrowserBack => keyboard::key::Code::BrowserBack,
        KeyCode::BrowserFavorites => keyboard::key::Code::BrowserFavorites,
        KeyCode::BrowserForward => keyboard::key::Code::BrowserForward,
        KeyCode::BrowserHome => keyboard::key::Code::BrowserHome,
        KeyCode::BrowserRefresh => keyboard::key::Code::BrowserRefresh,
        KeyCode::BrowserSearch => keyboard::key::Code::BrowserSearch,
        KeyCode::BrowserStop => keyboard::key::Code::BrowserStop,
        KeyCode::Eject => keyboard::key::Code::Eject,
        KeyCode::LaunchApp1 => keyboard::key::Code::LaunchApp1,
        KeyCode::LaunchApp2 => keyboard::key::Code::LaunchApp2,
        KeyCode::LaunchMail => keyboard::key::Code::LaunchMail,
        KeyCode::MediaPlayPause => keyboard::key::Code::MediaPlayPause,
        KeyCode::MediaSelect => keyboard::key::Code::MediaSelect,
        KeyCode::MediaStop => keyboard::key::Code::MediaStop,
        KeyCode::MediaTrackNext => keyboard::key::Code::MediaTrackNext,
        KeyCode::MediaTrackPrevious => keyboard::key::Code::MediaTrackPrevious,
        KeyCode::Power => keyboard::key::Code::Power,
        KeyCode::Sleep => keyboard::key::Code::Sleep,
        KeyCode::AudioVolumeDown => keyboard::key::Code::AudioVolumeDown,
        KeyCode::AudioVolumeMute => keyboard::key::Code::AudioVolumeMute,
        KeyCode::AudioVolumeUp => keyboard::key::Code::AudioVolumeUp,
        KeyCode::WakeUp => keyboard::key::Code::WakeUp,
        KeyCode::Hyper => keyboard::key::Code::Hyper,
        KeyCode::Turbo => keyboard::key::Code::Turbo,
        KeyCode::Abort => keyboard::key::Code::Abort,
        KeyCode::Resume => keyboard::key::Code::Resume,
        KeyCode::Suspend => keyboard::key::Code::Suspend,
        KeyCode::Again => keyboard::key::Code::Again,
        KeyCode::Copy => keyboard::key::Code::Copy,
        KeyCode::Cut => keyboard::key::Code::Cut,
        KeyCode::Find => keyboard::key::Code::Find,
        KeyCode::Open => keyboard::key::Code::Open,
        KeyCode::Paste => keyboard::key::Code::Paste,
        KeyCode::Props => keyboard::key::Code::Props,
        KeyCode::Select => keyboard::key::Code::Select,
        KeyCode::Undo => keyboard::key::Code::Undo,
        KeyCode::Hiragana => keyboard::key::Code::Hiragana,
        KeyCode::Katakana => keyboard::key::Code::Katakana,
        KeyCode::F1 => keyboard::key::Code::F1,
        KeyCode::F2 => keyboard::key::Code::F2,
        KeyCode::F3 => keyboard::key::Code::F3,
        KeyCode::F4 => keyboard::key::Code::F4,
        KeyCode::F5 => keyboard::key::Code::F5,
        KeyCode::F6 => keyboard::key::Code::F6,
        KeyCode::F7 => keyboard::key::Code::F7,
        KeyCode::F8 => keyboard::key::Code::F8,
        KeyCode::F9 => keyboard::key::Code::F9,
        KeyCode::F10 => keyboard::key::Code::F10,
        KeyCode::F11 => keyboard::key::Code::F11,
        KeyCode::F12 => keyboard::key::Code::F12,
        KeyCode::F13 => keyboard::key::Code::F13,
        KeyCode::F14 => keyboard::key::Code::F14,
        KeyCode::F15 => keyboard::key::Code::F15,
        KeyCode::F16 => keyboard::key::Code::F16,
        KeyCode::F17 => keyboard::key::Code::F17,
        KeyCode::F18 => keyboard::key::Code::F18,
        KeyCode::F19 => keyboard::key::Code::F19,
        KeyCode::F20 => keyboard::key::Code::F20,
        KeyCode::F21 => keyboard::key::Code::F21,
        KeyCode::F22 => keyboard::key::Code::F22,
        KeyCode::F23 => keyboard::key::Code::F23,
        KeyCode::F24 => keyboard::key::Code::F24,
        KeyCode::F25 => keyboard::key::Code::F25,
        KeyCode::F26 => keyboard::key::Code::F26,
        KeyCode::F27 => keyboard::key::Code::F27,
        KeyCode::F28 => keyboard::key::Code::F28,
        KeyCode::F29 => keyboard::key::Code::F29,
        KeyCode::F30 => keyboard::key::Code::F30,
        KeyCode::F31 => keyboard::key::Code::F31,
        KeyCode::F32 => keyboard::key::Code::F32,
        KeyCode::F33 => keyboard::key::Code::F33,
        KeyCode::F34 => keyboard::key::Code::F34,
        KeyCode::F35 => keyboard::key::Code::F35,
        _ => None?,
    })
}

/// Converts a `NativeKeyCode` from [`tao`] to an [`iced`] native key code.
///
/// [`tao`]: https://github.com/rust-windowing/tao
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn native_key_code(
    native_key_code: tao::keyboard::NativeKeyCode,
) -> keyboard::key::NativeCode {
    use tao::keyboard::NativeKeyCode;

    match native_key_code {
        NativeKeyCode::Unidentified => keyboard::key::NativeCode::Unidentified,
        NativeKeyCode::Android(code) => keyboard::key::NativeCode::Android(
            u32::try_from(code).expect("keycode conversion"),
        ),
        NativeKeyCode::MacOS(code) => keyboard::key::NativeCode::MacOS(code),
        NativeKeyCode::Windows(code) => {
            keyboard::key::NativeCode::Windows(code)
        }
        _ => todo!(),
    }
}

/// Converts some [`UserAttention`] into its `tao` counterpart.
///
/// [`UserAttention`]: window::UserAttention
pub fn user_attention(
    user_attention: window::UserAttention,
) -> tao::window::UserAttentionType {
    match user_attention {
        window::UserAttention::Critical => {
            tao::window::UserAttentionType::Critical
        }
        window::UserAttention::Informational => {
            tao::window::UserAttentionType::Informational
        }
    }
}

/// Converts some [`window::Direction`] into a [`tao::window::ResizeDirection`].
pub fn resize_direction(
    resize_direction: window::Direction,
) -> tao::window::ResizeDirection {
    match resize_direction {
        window::Direction::North => tao::window::ResizeDirection::North,
        window::Direction::South => tao::window::ResizeDirection::South,
        window::Direction::East => tao::window::ResizeDirection::East,
        window::Direction::West => tao::window::ResizeDirection::West,
        window::Direction::NorthEast => tao::window::ResizeDirection::NorthEast,
        window::Direction::NorthWest => tao::window::ResizeDirection::NorthWest,
        window::Direction::SouthEast => tao::window::ResizeDirection::SouthEast,
        window::Direction::SouthWest => tao::window::ResizeDirection::SouthWest,
    }
}

/// Converts some [`window::Icon`] into its `tao` counterpart.
///
/// Returns `None` if there is an error during the conversion.
pub fn icon(icon: window::Icon) -> Option<tao::window::Icon> {
    let (pixels, size) = icon.into_raw();

    tao::window::Icon::from_rgba(pixels, size.width, size.height).ok()
}

// See: https://en.wikipedia.org/wiki/Private_Use_Areas
fn is_private_use(c: char) -> bool {
    ('\u{E000}'..='\u{F8FF}').contains(&c)
}
