//! Convert [`winit`] types into [`iced_runtime`] types, and viceversa.
//!
//! [`winit`]: https://github.com/rust-windowing/winit
//! [`iced_runtime`]: https://github.com/iced-rs/iced/tree/0.13/runtime
use crate::core::input_method;
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::touch;
use crate::core::window;
use crate::core::{Event, Point, Size};

/// Converts some [`window::Settings`] into some `WindowAttributes` from `winit`.
pub fn window_attributes(
    settings: window::Settings,
    title: &str,
    primary_monitor: Option<winit::monitor::MonitorHandle>,
    _id: Option<String>,
) -> winit::window::WindowAttributes {
    let mut attributes = winit::window::WindowAttributes::default();

    attributes = attributes
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize {
            width: settings.size.width,
            height: settings.size.height,
        })
        .with_maximized(settings.maximized)
        .with_fullscreen(
            settings
                .fullscreen
                .then_some(winit::window::Fullscreen::Borderless(None)),
        )
        .with_resizable(settings.resizable)
        .with_enabled_buttons(if settings.resizable {
            winit::window::WindowButtons::all()
        } else {
            winit::window::WindowButtons::CLOSE
                | winit::window::WindowButtons::MINIMIZE
        })
        .with_decorations(settings.decorations)
        .with_transparent(settings.transparent)
        .with_window_icon(settings.icon.and_then(icon))
        .with_window_level(window_level(settings.level))
        .with_visible(settings.visible);

    if let Some(position) =
        position(primary_monitor.as_ref(), settings.size, settings.position)
    {
        attributes = attributes.with_position(position);
    }

    if let Some(min_size) = settings.min_size {
        attributes = attributes.with_min_inner_size(winit::dpi::LogicalSize {
            width: min_size.width,
            height: min_size.height,
        });
    }

    if let Some(max_size) = settings.max_size {
        attributes = attributes.with_max_inner_size(winit::dpi::LogicalSize {
            width: max_size.width,
            height: max_size.height,
        });
    }

    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        use ::winit::platform::wayland::WindowAttributesExtWayland;

        if let Some(id) = _id {
            attributes = attributes.with_name(id.clone(), id);
        }
    }

    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowAttributesExtWindows;

        attributes = attributes
            .with_drag_and_drop(settings.platform_specific.drag_and_drop);

        attributes = attributes
            .with_skip_taskbar(settings.platform_specific.skip_taskbar);

        attributes = attributes.with_undecorated_shadow(
            settings.platform_specific.undecorated_shadow,
        );
    }

    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::WindowAttributesExtMacOS;

        attributes = attributes
            .with_title_hidden(settings.platform_specific.title_hidden)
            .with_titlebar_transparent(
                settings.platform_specific.titlebar_transparent,
            )
            .with_fullsize_content_view(
                settings.platform_specific.fullsize_content_view,
            );
    }

    #[cfg(target_os = "linux")]
    {
        #[cfg(feature = "x11")]
        {
            use winit::platform::x11::WindowAttributesExtX11;

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
            use winit::platform::wayland::WindowAttributesExtWayland;

            attributes = attributes.with_name(
                &settings.platform_specific.application_id,
                &settings.platform_specific.application_id,
            );
        }
    }

    attributes
}

/// Converts a winit window event into an iced event.
pub fn window_event(
    event: winit::event::WindowEvent,
    scale_factor: f64,
    modifiers: winit::keyboard::ModifiersState,
) -> Option<Event> {
    use winit::event::Ime;
    use winit::event::WindowEvent;

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
                        x: delta_x,
                        y: delta_y,
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
        // Ignore keyboard presses/releases during window focus/unfocus
        WindowEvent::KeyboardInput { is_synthetic, .. } if is_synthetic => None,
        WindowEvent::KeyboardInput { event, .. } => Some(Event::Keyboard({
            let key = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
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
                    use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;

                    event.text_with_all_modifiers().map(SmolStr::new)
                }

                #[cfg(target_arch = "wasm32")]
                {
                    // TODO: Fix inconsistent API on Wasm
                    event.text
                }
            }.filter(|text| !text.as_str().chars().any(is_private_use));

            let winit::event::KeyEvent {
                state,
                location,
                logical_key,
                physical_key,
                ..
            } = event;

            let key = self::key(key);
            let modified_key = self::key(logical_key);
            let physical_key = self::physical_key(physical_key);
            let modifiers = self::modifiers(modifiers);

            let location = match location {
                winit::keyboard::KeyLocation::Standard => {
                    keyboard::Location::Standard
                }
                winit::keyboard::KeyLocation::Left => keyboard::Location::Left,
                winit::keyboard::KeyLocation::Right => {
                    keyboard::Location::Right
                }
                winit::keyboard::KeyLocation::Numpad => {
                    keyboard::Location::Numpad
                }
            };

            match state {
                winit::event::ElementState::Pressed => {
                    keyboard::Event::KeyPressed {
                        key,
                        modified_key,
                        physical_key,
                        modifiers,
                        location,
                        text,
                    }
                }
                winit::event::ElementState::Released => {
                    keyboard::Event::KeyReleased {
                        key,
                        modified_key,
                        physical_key,
                        modifiers,
                        location,
                    }
                }
            }
        })),
        WindowEvent::ModifiersChanged(new_modifiers) => {
            Some(Event::Keyboard(keyboard::Event::ModifiersChanged(
                self::modifiers(new_modifiers.state()),
            )))
        }
        WindowEvent::Ime(event) => Some(Event::InputMethod(match event {
            Ime::Enabled => input_method::Event::Opened,
            Ime::Preedit(content, size) => input_method::Event::Preedit(
                content,
                size.map(|(start, end)| (start..end)),
            ),
            Ime::Commit(content) => input_method::Event::Commit(content),
            Ime::Disabled => input_method::Event::Closed,
        })),
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
            let winit::dpi::LogicalPosition { x, y } =
                position.to_logical(scale_factor);

            Some(Event::Window(window::Event::Moved(Point::new(x, y))))
        }
        _ => None,
    }
}

/// Converts a [`window::Level`] to a [`winit`] window level.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn window_level(level: window::Level) -> winit::window::WindowLevel {
    match level {
        window::Level::Normal => winit::window::WindowLevel::Normal,
        window::Level::AlwaysOnBottom => {
            winit::window::WindowLevel::AlwaysOnBottom
        }
        window::Level::AlwaysOnTop => winit::window::WindowLevel::AlwaysOnTop,
    }
}

/// Converts a [`window::Position`] to a [`winit`] logical position for a given monitor.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn position(
    monitor: Option<&winit::monitor::MonitorHandle>,
    size: Size,
    position: window::Position,
) -> Option<winit::dpi::Position> {
    match position {
        window::Position::Default => None,
        window::Position::Specific(position) => {
            Some(winit::dpi::Position::Logical(winit::dpi::LogicalPosition {
                x: f64::from(position.x),
                y: f64::from(position.y),
            }))
        }
        window::Position::SpecificWith(to_position) => {
            if let Some(monitor) = monitor {
                let start = monitor.position();

                let resolution: winit::dpi::LogicalSize<f32> =
                    monitor.size().to_logical(monitor.scale_factor());

                let position = to_position(
                    size,
                    Size::new(resolution.width, resolution.height),
                );

                let centered: winit::dpi::PhysicalPosition<i32> =
                    winit::dpi::LogicalPosition {
                        x: position.x,
                        y: position.y,
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
        window::Position::Centered => {
            if let Some(monitor) = monitor {
                let start = monitor.position();

                let resolution: winit::dpi::LogicalSize<f64> =
                    monitor.size().to_logical(monitor.scale_factor());

                let centered: winit::dpi::PhysicalPosition<i32> =
                    winit::dpi::LogicalPosition {
                        x: (resolution.width - f64::from(size.width)) / 2.0,
                        y: (resolution.height - f64::from(size.height)) / 2.0,
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

/// Converts a [`window::Mode`] to a [`winit`] fullscreen mode.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn fullscreen(
    monitor: Option<winit::monitor::MonitorHandle>,
    mode: window::Mode,
) -> Option<winit::window::Fullscreen> {
    match mode {
        window::Mode::Windowed | window::Mode::Hidden => None,
        window::Mode::Fullscreen => {
            Some(winit::window::Fullscreen::Borderless(monitor))
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

/// Converts a [`winit`] fullscreen mode to a [`window::Mode`].
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn mode(mode: Option<winit::window::Fullscreen>) -> window::Mode {
    match mode {
        None => window::Mode::Windowed,
        Some(_) => window::Mode::Fullscreen,
    }
}

/// Converts a [`mouse::Interaction`] to a [`winit`] cursor icon.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn mouse_interaction(
    interaction: mouse::Interaction,
) -> winit::window::CursorIcon {
    use mouse::Interaction;

    match interaction {
        Interaction::None | Interaction::Idle => {
            winit::window::CursorIcon::Default
        }
        Interaction::Pointer => winit::window::CursorIcon::Pointer,
        Interaction::Working => winit::window::CursorIcon::Progress,
        Interaction::Grab => winit::window::CursorIcon::Grab,
        Interaction::Grabbing => winit::window::CursorIcon::Grabbing,
        Interaction::Crosshair => winit::window::CursorIcon::Crosshair,
        Interaction::Text => winit::window::CursorIcon::Text,
        Interaction::ResizingHorizontally => {
            winit::window::CursorIcon::EwResize
        }
        Interaction::ResizingVertically => winit::window::CursorIcon::NsResize,
        Interaction::ResizingDiagonallyUp => {
            winit::window::CursorIcon::NeswResize
        }
        Interaction::ResizingDiagonallyDown => {
            winit::window::CursorIcon::NwseResize
        }
        Interaction::NotAllowed => winit::window::CursorIcon::NotAllowed,
        Interaction::ZoomIn => winit::window::CursorIcon::ZoomIn,
        Interaction::ZoomOut => winit::window::CursorIcon::ZoomOut,
        Interaction::Cell => winit::window::CursorIcon::Cell,
        Interaction::Move => winit::window::CursorIcon::Move,
        Interaction::Copy => winit::window::CursorIcon::Copy,
        Interaction::Help => winit::window::CursorIcon::Help,
    }
}

/// Converts a `MouseButton` from [`winit`] to an [`iced`] mouse button.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn mouse_button(mouse_button: winit::event::MouseButton) -> mouse::Button {
    match mouse_button {
        winit::event::MouseButton::Left => mouse::Button::Left,
        winit::event::MouseButton::Right => mouse::Button::Right,
        winit::event::MouseButton::Middle => mouse::Button::Middle,
        winit::event::MouseButton::Back => mouse::Button::Back,
        winit::event::MouseButton::Forward => mouse::Button::Forward,
        winit::event::MouseButton::Other(other) => mouse::Button::Other(other),
    }
}

/// Converts some `ModifiersState` from [`winit`] to an [`iced`] modifiers
/// state.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn modifiers(
    modifiers: winit::keyboard::ModifiersState,
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
    position: winit::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
) -> Point {
    let logical_position = position.to_logical(scale_factor);

    Point::new(logical_position.x, logical_position.y)
}

/// Converts a `Touch` from [`winit`] to an [`iced`] touch event.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
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

/// Converts a `Key` from [`winit`] to an [`iced`] key.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn key(key: winit::keyboard::Key) -> keyboard::Key {
    use keyboard::key::Named;
    use winit::keyboard::NamedKey;

    match key {
        winit::keyboard::Key::Character(c) => keyboard::Key::Character(c),
        winit::keyboard::Key::Named(named_key) => {
            keyboard::Key::Named(match named_key {
                NamedKey::Alt => Named::Alt,
                NamedKey::AltGraph => Named::AltGraph,
                NamedKey::CapsLock => Named::CapsLock,
                NamedKey::Control => Named::Control,
                NamedKey::Fn => Named::Fn,
                NamedKey::FnLock => Named::FnLock,
                NamedKey::NumLock => Named::NumLock,
                NamedKey::ScrollLock => Named::ScrollLock,
                NamedKey::Shift => Named::Shift,
                NamedKey::Symbol => Named::Symbol,
                NamedKey::SymbolLock => Named::SymbolLock,
                NamedKey::Meta => Named::Meta,
                NamedKey::Hyper => Named::Hyper,
                NamedKey::Super => Named::Super,
                NamedKey::Enter => Named::Enter,
                NamedKey::Tab => Named::Tab,
                NamedKey::Space => Named::Space,
                NamedKey::ArrowDown => Named::ArrowDown,
                NamedKey::ArrowLeft => Named::ArrowLeft,
                NamedKey::ArrowRight => Named::ArrowRight,
                NamedKey::ArrowUp => Named::ArrowUp,
                NamedKey::End => Named::End,
                NamedKey::Home => Named::Home,
                NamedKey::PageDown => Named::PageDown,
                NamedKey::PageUp => Named::PageUp,
                NamedKey::Backspace => Named::Backspace,
                NamedKey::Clear => Named::Clear,
                NamedKey::Copy => Named::Copy,
                NamedKey::CrSel => Named::CrSel,
                NamedKey::Cut => Named::Cut,
                NamedKey::Delete => Named::Delete,
                NamedKey::EraseEof => Named::EraseEof,
                NamedKey::ExSel => Named::ExSel,
                NamedKey::Insert => Named::Insert,
                NamedKey::Paste => Named::Paste,
                NamedKey::Redo => Named::Redo,
                NamedKey::Undo => Named::Undo,
                NamedKey::Accept => Named::Accept,
                NamedKey::Again => Named::Again,
                NamedKey::Attn => Named::Attn,
                NamedKey::Cancel => Named::Cancel,
                NamedKey::ContextMenu => Named::ContextMenu,
                NamedKey::Escape => Named::Escape,
                NamedKey::Execute => Named::Execute,
                NamedKey::Find => Named::Find,
                NamedKey::Help => Named::Help,
                NamedKey::Pause => Named::Pause,
                NamedKey::Play => Named::Play,
                NamedKey::Props => Named::Props,
                NamedKey::Select => Named::Select,
                NamedKey::ZoomIn => Named::ZoomIn,
                NamedKey::ZoomOut => Named::ZoomOut,
                NamedKey::BrightnessDown => Named::BrightnessDown,
                NamedKey::BrightnessUp => Named::BrightnessUp,
                NamedKey::Eject => Named::Eject,
                NamedKey::LogOff => Named::LogOff,
                NamedKey::Power => Named::Power,
                NamedKey::PowerOff => Named::PowerOff,
                NamedKey::PrintScreen => Named::PrintScreen,
                NamedKey::Hibernate => Named::Hibernate,
                NamedKey::Standby => Named::Standby,
                NamedKey::WakeUp => Named::WakeUp,
                NamedKey::AllCandidates => Named::AllCandidates,
                NamedKey::Alphanumeric => Named::Alphanumeric,
                NamedKey::CodeInput => Named::CodeInput,
                NamedKey::Compose => Named::Compose,
                NamedKey::Convert => Named::Convert,
                NamedKey::FinalMode => Named::FinalMode,
                NamedKey::GroupFirst => Named::GroupFirst,
                NamedKey::GroupLast => Named::GroupLast,
                NamedKey::GroupNext => Named::GroupNext,
                NamedKey::GroupPrevious => Named::GroupPrevious,
                NamedKey::ModeChange => Named::ModeChange,
                NamedKey::NextCandidate => Named::NextCandidate,
                NamedKey::NonConvert => Named::NonConvert,
                NamedKey::PreviousCandidate => Named::PreviousCandidate,
                NamedKey::Process => Named::Process,
                NamedKey::SingleCandidate => Named::SingleCandidate,
                NamedKey::HangulMode => Named::HangulMode,
                NamedKey::HanjaMode => Named::HanjaMode,
                NamedKey::JunjaMode => Named::JunjaMode,
                NamedKey::Eisu => Named::Eisu,
                NamedKey::Hankaku => Named::Hankaku,
                NamedKey::Hiragana => Named::Hiragana,
                NamedKey::HiraganaKatakana => Named::HiraganaKatakana,
                NamedKey::KanaMode => Named::KanaMode,
                NamedKey::KanjiMode => Named::KanjiMode,
                NamedKey::Katakana => Named::Katakana,
                NamedKey::Romaji => Named::Romaji,
                NamedKey::Zenkaku => Named::Zenkaku,
                NamedKey::ZenkakuHankaku => Named::ZenkakuHankaku,
                NamedKey::Soft1 => Named::Soft1,
                NamedKey::Soft2 => Named::Soft2,
                NamedKey::Soft3 => Named::Soft3,
                NamedKey::Soft4 => Named::Soft4,
                NamedKey::ChannelDown => Named::ChannelDown,
                NamedKey::ChannelUp => Named::ChannelUp,
                NamedKey::Close => Named::Close,
                NamedKey::MailForward => Named::MailForward,
                NamedKey::MailReply => Named::MailReply,
                NamedKey::MailSend => Named::MailSend,
                NamedKey::MediaClose => Named::MediaClose,
                NamedKey::MediaFastForward => Named::MediaFastForward,
                NamedKey::MediaPause => Named::MediaPause,
                NamedKey::MediaPlay => Named::MediaPlay,
                NamedKey::MediaPlayPause => Named::MediaPlayPause,
                NamedKey::MediaRecord => Named::MediaRecord,
                NamedKey::MediaRewind => Named::MediaRewind,
                NamedKey::MediaStop => Named::MediaStop,
                NamedKey::MediaTrackNext => Named::MediaTrackNext,
                NamedKey::MediaTrackPrevious => Named::MediaTrackPrevious,
                NamedKey::New => Named::New,
                NamedKey::Open => Named::Open,
                NamedKey::Print => Named::Print,
                NamedKey::Save => Named::Save,
                NamedKey::SpellCheck => Named::SpellCheck,
                NamedKey::Key11 => Named::Key11,
                NamedKey::Key12 => Named::Key12,
                NamedKey::AudioBalanceLeft => Named::AudioBalanceLeft,
                NamedKey::AudioBalanceRight => Named::AudioBalanceRight,
                NamedKey::AudioBassBoostDown => Named::AudioBassBoostDown,
                NamedKey::AudioBassBoostToggle => Named::AudioBassBoostToggle,
                NamedKey::AudioBassBoostUp => Named::AudioBassBoostUp,
                NamedKey::AudioFaderFront => Named::AudioFaderFront,
                NamedKey::AudioFaderRear => Named::AudioFaderRear,
                NamedKey::AudioSurroundModeNext => Named::AudioSurroundModeNext,
                NamedKey::AudioTrebleDown => Named::AudioTrebleDown,
                NamedKey::AudioTrebleUp => Named::AudioTrebleUp,
                NamedKey::AudioVolumeDown => Named::AudioVolumeDown,
                NamedKey::AudioVolumeUp => Named::AudioVolumeUp,
                NamedKey::AudioVolumeMute => Named::AudioVolumeMute,
                NamedKey::MicrophoneToggle => Named::MicrophoneToggle,
                NamedKey::MicrophoneVolumeDown => Named::MicrophoneVolumeDown,
                NamedKey::MicrophoneVolumeUp => Named::MicrophoneVolumeUp,
                NamedKey::MicrophoneVolumeMute => Named::MicrophoneVolumeMute,
                NamedKey::SpeechCorrectionList => Named::SpeechCorrectionList,
                NamedKey::SpeechInputToggle => Named::SpeechInputToggle,
                NamedKey::LaunchApplication1 => Named::LaunchApplication1,
                NamedKey::LaunchApplication2 => Named::LaunchApplication2,
                NamedKey::LaunchCalendar => Named::LaunchCalendar,
                NamedKey::LaunchContacts => Named::LaunchContacts,
                NamedKey::LaunchMail => Named::LaunchMail,
                NamedKey::LaunchMediaPlayer => Named::LaunchMediaPlayer,
                NamedKey::LaunchMusicPlayer => Named::LaunchMusicPlayer,
                NamedKey::LaunchPhone => Named::LaunchPhone,
                NamedKey::LaunchScreenSaver => Named::LaunchScreenSaver,
                NamedKey::LaunchSpreadsheet => Named::LaunchSpreadsheet,
                NamedKey::LaunchWebBrowser => Named::LaunchWebBrowser,
                NamedKey::LaunchWebCam => Named::LaunchWebCam,
                NamedKey::LaunchWordProcessor => Named::LaunchWordProcessor,
                NamedKey::BrowserBack => Named::BrowserBack,
                NamedKey::BrowserFavorites => Named::BrowserFavorites,
                NamedKey::BrowserForward => Named::BrowserForward,
                NamedKey::BrowserHome => Named::BrowserHome,
                NamedKey::BrowserRefresh => Named::BrowserRefresh,
                NamedKey::BrowserSearch => Named::BrowserSearch,
                NamedKey::BrowserStop => Named::BrowserStop,
                NamedKey::AppSwitch => Named::AppSwitch,
                NamedKey::Call => Named::Call,
                NamedKey::Camera => Named::Camera,
                NamedKey::CameraFocus => Named::CameraFocus,
                NamedKey::EndCall => Named::EndCall,
                NamedKey::GoBack => Named::GoBack,
                NamedKey::GoHome => Named::GoHome,
                NamedKey::HeadsetHook => Named::HeadsetHook,
                NamedKey::LastNumberRedial => Named::LastNumberRedial,
                NamedKey::Notification => Named::Notification,
                NamedKey::MannerMode => Named::MannerMode,
                NamedKey::VoiceDial => Named::VoiceDial,
                NamedKey::TV => Named::TV,
                NamedKey::TV3DMode => Named::TV3DMode,
                NamedKey::TVAntennaCable => Named::TVAntennaCable,
                NamedKey::TVAudioDescription => Named::TVAudioDescription,
                NamedKey::TVAudioDescriptionMixDown => {
                    Named::TVAudioDescriptionMixDown
                }
                NamedKey::TVAudioDescriptionMixUp => {
                    Named::TVAudioDescriptionMixUp
                }
                NamedKey::TVContentsMenu => Named::TVContentsMenu,
                NamedKey::TVDataService => Named::TVDataService,
                NamedKey::TVInput => Named::TVInput,
                NamedKey::TVInputComponent1 => Named::TVInputComponent1,
                NamedKey::TVInputComponent2 => Named::TVInputComponent2,
                NamedKey::TVInputComposite1 => Named::TVInputComposite1,
                NamedKey::TVInputComposite2 => Named::TVInputComposite2,
                NamedKey::TVInputHDMI1 => Named::TVInputHDMI1,
                NamedKey::TVInputHDMI2 => Named::TVInputHDMI2,
                NamedKey::TVInputHDMI3 => Named::TVInputHDMI3,
                NamedKey::TVInputHDMI4 => Named::TVInputHDMI4,
                NamedKey::TVInputVGA1 => Named::TVInputVGA1,
                NamedKey::TVMediaContext => Named::TVMediaContext,
                NamedKey::TVNetwork => Named::TVNetwork,
                NamedKey::TVNumberEntry => Named::TVNumberEntry,
                NamedKey::TVPower => Named::TVPower,
                NamedKey::TVRadioService => Named::TVRadioService,
                NamedKey::TVSatellite => Named::TVSatellite,
                NamedKey::TVSatelliteBS => Named::TVSatelliteBS,
                NamedKey::TVSatelliteCS => Named::TVSatelliteCS,
                NamedKey::TVSatelliteToggle => Named::TVSatelliteToggle,
                NamedKey::TVTerrestrialAnalog => Named::TVTerrestrialAnalog,
                NamedKey::TVTerrestrialDigital => Named::TVTerrestrialDigital,
                NamedKey::TVTimer => Named::TVTimer,
                NamedKey::AVRInput => Named::AVRInput,
                NamedKey::AVRPower => Named::AVRPower,
                NamedKey::ColorF0Red => Named::ColorF0Red,
                NamedKey::ColorF1Green => Named::ColorF1Green,
                NamedKey::ColorF2Yellow => Named::ColorF2Yellow,
                NamedKey::ColorF3Blue => Named::ColorF3Blue,
                NamedKey::ColorF4Grey => Named::ColorF4Grey,
                NamedKey::ColorF5Brown => Named::ColorF5Brown,
                NamedKey::ClosedCaptionToggle => Named::ClosedCaptionToggle,
                NamedKey::Dimmer => Named::Dimmer,
                NamedKey::DisplaySwap => Named::DisplaySwap,
                NamedKey::DVR => Named::DVR,
                NamedKey::Exit => Named::Exit,
                NamedKey::FavoriteClear0 => Named::FavoriteClear0,
                NamedKey::FavoriteClear1 => Named::FavoriteClear1,
                NamedKey::FavoriteClear2 => Named::FavoriteClear2,
                NamedKey::FavoriteClear3 => Named::FavoriteClear3,
                NamedKey::FavoriteRecall0 => Named::FavoriteRecall0,
                NamedKey::FavoriteRecall1 => Named::FavoriteRecall1,
                NamedKey::FavoriteRecall2 => Named::FavoriteRecall2,
                NamedKey::FavoriteRecall3 => Named::FavoriteRecall3,
                NamedKey::FavoriteStore0 => Named::FavoriteStore0,
                NamedKey::FavoriteStore1 => Named::FavoriteStore1,
                NamedKey::FavoriteStore2 => Named::FavoriteStore2,
                NamedKey::FavoriteStore3 => Named::FavoriteStore3,
                NamedKey::Guide => Named::Guide,
                NamedKey::GuideNextDay => Named::GuideNextDay,
                NamedKey::GuidePreviousDay => Named::GuidePreviousDay,
                NamedKey::Info => Named::Info,
                NamedKey::InstantReplay => Named::InstantReplay,
                NamedKey::Link => Named::Link,
                NamedKey::ListProgram => Named::ListProgram,
                NamedKey::LiveContent => Named::LiveContent,
                NamedKey::Lock => Named::Lock,
                NamedKey::MediaApps => Named::MediaApps,
                NamedKey::MediaAudioTrack => Named::MediaAudioTrack,
                NamedKey::MediaLast => Named::MediaLast,
                NamedKey::MediaSkipBackward => Named::MediaSkipBackward,
                NamedKey::MediaSkipForward => Named::MediaSkipForward,
                NamedKey::MediaStepBackward => Named::MediaStepBackward,
                NamedKey::MediaStepForward => Named::MediaStepForward,
                NamedKey::MediaTopMenu => Named::MediaTopMenu,
                NamedKey::NavigateIn => Named::NavigateIn,
                NamedKey::NavigateNext => Named::NavigateNext,
                NamedKey::NavigateOut => Named::NavigateOut,
                NamedKey::NavigatePrevious => Named::NavigatePrevious,
                NamedKey::NextFavoriteChannel => Named::NextFavoriteChannel,
                NamedKey::NextUserProfile => Named::NextUserProfile,
                NamedKey::OnDemand => Named::OnDemand,
                NamedKey::Pairing => Named::Pairing,
                NamedKey::PinPDown => Named::PinPDown,
                NamedKey::PinPMove => Named::PinPMove,
                NamedKey::PinPToggle => Named::PinPToggle,
                NamedKey::PinPUp => Named::PinPUp,
                NamedKey::PlaySpeedDown => Named::PlaySpeedDown,
                NamedKey::PlaySpeedReset => Named::PlaySpeedReset,
                NamedKey::PlaySpeedUp => Named::PlaySpeedUp,
                NamedKey::RandomToggle => Named::RandomToggle,
                NamedKey::RcLowBattery => Named::RcLowBattery,
                NamedKey::RecordSpeedNext => Named::RecordSpeedNext,
                NamedKey::RfBypass => Named::RfBypass,
                NamedKey::ScanChannelsToggle => Named::ScanChannelsToggle,
                NamedKey::ScreenModeNext => Named::ScreenModeNext,
                NamedKey::Settings => Named::Settings,
                NamedKey::SplitScreenToggle => Named::SplitScreenToggle,
                NamedKey::STBInput => Named::STBInput,
                NamedKey::STBPower => Named::STBPower,
                NamedKey::Subtitle => Named::Subtitle,
                NamedKey::Teletext => Named::Teletext,
                NamedKey::VideoModeNext => Named::VideoModeNext,
                NamedKey::Wink => Named::Wink,
                NamedKey::ZoomToggle => Named::ZoomToggle,
                NamedKey::F1 => Named::F1,
                NamedKey::F2 => Named::F2,
                NamedKey::F3 => Named::F3,
                NamedKey::F4 => Named::F4,
                NamedKey::F5 => Named::F5,
                NamedKey::F6 => Named::F6,
                NamedKey::F7 => Named::F7,
                NamedKey::F8 => Named::F8,
                NamedKey::F9 => Named::F9,
                NamedKey::F10 => Named::F10,
                NamedKey::F11 => Named::F11,
                NamedKey::F12 => Named::F12,
                NamedKey::F13 => Named::F13,
                NamedKey::F14 => Named::F14,
                NamedKey::F15 => Named::F15,
                NamedKey::F16 => Named::F16,
                NamedKey::F17 => Named::F17,
                NamedKey::F18 => Named::F18,
                NamedKey::F19 => Named::F19,
                NamedKey::F20 => Named::F20,
                NamedKey::F21 => Named::F21,
                NamedKey::F22 => Named::F22,
                NamedKey::F23 => Named::F23,
                NamedKey::F24 => Named::F24,
                NamedKey::F25 => Named::F25,
                NamedKey::F26 => Named::F26,
                NamedKey::F27 => Named::F27,
                NamedKey::F28 => Named::F28,
                NamedKey::F29 => Named::F29,
                NamedKey::F30 => Named::F30,
                NamedKey::F31 => Named::F31,
                NamedKey::F32 => Named::F32,
                NamedKey::F33 => Named::F33,
                NamedKey::F34 => Named::F34,
                NamedKey::F35 => Named::F35,
                _ => return keyboard::Key::Unidentified,
            })
        }
        _ => keyboard::Key::Unidentified,
    }
}

/// Converts a `PhysicalKey` from [`winit`] to an [`iced`] physical key.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn physical_key(
    physical_key: winit::keyboard::PhysicalKey,
) -> keyboard::key::Physical {
    match physical_key {
        winit::keyboard::PhysicalKey::Code(code) => key_code(code)
            .map(keyboard::key::Physical::Code)
            .unwrap_or(keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            )),
        winit::keyboard::PhysicalKey::Unidentified(code) => {
            keyboard::key::Physical::Unidentified(native_key_code(code))
        }
    }
}

/// Converts a `KeyCode` from [`winit`] to an [`iced`] key code.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn key_code(
    key_code: winit::keyboard::KeyCode,
) -> Option<keyboard::key::Code> {
    use winit::keyboard::KeyCode;

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
        KeyCode::Meta => keyboard::key::Code::Meta,
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

/// Converts a `NativeKeyCode` from [`winit`] to an [`iced`] native key code.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced`]: https://github.com/iced-rs/iced/tree/0.12
pub fn native_key_code(
    native_key_code: winit::keyboard::NativeKeyCode,
) -> keyboard::key::NativeCode {
    use winit::keyboard::NativeKeyCode;

    match native_key_code {
        NativeKeyCode::Unidentified => keyboard::key::NativeCode::Unidentified,
        NativeKeyCode::Android(code) => {
            keyboard::key::NativeCode::Android(code)
        }
        NativeKeyCode::MacOS(code) => keyboard::key::NativeCode::MacOS(code),
        NativeKeyCode::Windows(code) => {
            keyboard::key::NativeCode::Windows(code)
        }
        NativeKeyCode::Xkb(code) => keyboard::key::NativeCode::Xkb(code),
    }
}

/// Converts some [`UserAttention`] into its `winit` counterpart.
///
/// [`UserAttention`]: window::UserAttention
pub fn user_attention(
    user_attention: window::UserAttention,
) -> winit::window::UserAttentionType {
    match user_attention {
        window::UserAttention::Critical => {
            winit::window::UserAttentionType::Critical
        }
        window::UserAttention::Informational => {
            winit::window::UserAttentionType::Informational
        }
    }
}

/// Converts some [`window::Direction`] into a [`winit::window::ResizeDirection`].
pub fn resize_direction(
    resize_direction: window::Direction,
) -> winit::window::ResizeDirection {
    match resize_direction {
        window::Direction::North => winit::window::ResizeDirection::North,
        window::Direction::South => winit::window::ResizeDirection::South,
        window::Direction::East => winit::window::ResizeDirection::East,
        window::Direction::West => winit::window::ResizeDirection::West,
        window::Direction::NorthEast => {
            winit::window::ResizeDirection::NorthEast
        }
        window::Direction::NorthWest => {
            winit::window::ResizeDirection::NorthWest
        }
        window::Direction::SouthEast => {
            winit::window::ResizeDirection::SouthEast
        }
        window::Direction::SouthWest => {
            winit::window::ResizeDirection::SouthWest
        }
    }
}

/// Converts some [`window::Icon`] into its `winit` counterpart.
///
/// Returns `None` if there is an error during the conversion.
pub fn icon(icon: window::Icon) -> Option<winit::window::Icon> {
    let (pixels, size) = icon.into_raw();

    winit::window::Icon::from_rgba(pixels, size.width, size.height).ok()
}

/// Convertions some [`input_method::Purpose`] to its `winit` counterpart.
pub fn ime_purpose(
    purpose: input_method::Purpose,
) -> winit::window::ImePurpose {
    match purpose {
        input_method::Purpose::Normal => winit::window::ImePurpose::Normal,
        input_method::Purpose::Secure => winit::window::ImePurpose::Password,
        input_method::Purpose::Terminal => winit::window::ImePurpose::Terminal,
    }
}

// See: https://en.wikipedia.org/wiki/Private_Use_Areas
fn is_private_use(c: char) -> bool {
    ('\u{E000}'..='\u{F8FF}').contains(&c)
}
