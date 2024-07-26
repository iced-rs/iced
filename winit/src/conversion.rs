//! Convert [`winit`] types into [`iced_runtime`] types, and viceversa.
//!
//! [`winit`]: https://github.com/rust-windowing/winit
//! [`iced_runtime`]: https://github.com/iced-rs/iced/tree/0.12/runtime
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::touch;
use crate::core::window;
use crate::core::{Event, Point, Size};
use winit::keyboard::{KeyCode, NativeKeyCode, PhysicalKey};

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

            attributes = attributes.with_name(
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
        WindowEvent::KeyboardInput { event, .. } => Some(Event::Keyboard({
            let logical_key = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
                    event.key_without_modifiers()
                }

                #[cfg(target_arch = "wasm32")]
                {
                    // TODO: Fix inconsistent API on Wasm
                    event.logical_key
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
                physical_key,
                ..
            } = event;
            let key = key(logical_key);
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
                        physical_key,
                        modifiers,
                        location,
                        text,
                    }
                }
                winit::event::ElementState::Released => {
                    keyboard::Event::KeyReleased {
                        key,
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
        Interaction::NotAllowed => winit::window::CursorIcon::NotAllowed,
        Interaction::ZoomIn => winit::window::CursorIcon::ZoomIn,
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

/// Converts a `VirtualKeyCode` from [`winit`] to an [`iced`] key code.
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

/// Converts physical key code from winit to an iced key code.
pub fn physical_key(key: PhysicalKey) -> keyboard::PhysicalKey {
    match key {
        PhysicalKey::Code(KeyCode::Backquote) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Backquote)
        }
        PhysicalKey::Code(KeyCode::Backslash) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Backslash)
        }
        PhysicalKey::Code(KeyCode::BracketLeft) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BracketLeft)
        }
        PhysicalKey::Code(KeyCode::BracketRight) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BracketRight)
        }
        PhysicalKey::Code(KeyCode::Comma) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Comma)
        }
        PhysicalKey::Code(KeyCode::Digit0) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit0)
        }
        PhysicalKey::Code(KeyCode::Digit1) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit1)
        }
        PhysicalKey::Code(KeyCode::Digit2) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit2)
        }
        PhysicalKey::Code(KeyCode::Digit3) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit3)
        }
        PhysicalKey::Code(KeyCode::Digit4) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit4)
        }
        PhysicalKey::Code(KeyCode::Digit5) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit5)
        }
        PhysicalKey::Code(KeyCode::Digit6) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit6)
        }
        PhysicalKey::Code(KeyCode::Digit7) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit7)
        }
        PhysicalKey::Code(KeyCode::Digit8) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit8)
        }
        PhysicalKey::Code(KeyCode::Digit9) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Digit9)
        }
        PhysicalKey::Code(KeyCode::Equal) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Equal)
        }
        PhysicalKey::Code(KeyCode::IntlBackslash) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::IntlBackslash)
        }
        PhysicalKey::Code(KeyCode::IntlRo) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::IntlRo)
        }
        PhysicalKey::Code(KeyCode::IntlYen) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::IntlYen)
        }
        PhysicalKey::Code(KeyCode::KeyA) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyA)
        }
        PhysicalKey::Code(KeyCode::KeyB) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyB)
        }
        PhysicalKey::Code(KeyCode::KeyC) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyC)
        }
        PhysicalKey::Code(KeyCode::KeyD) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyD)
        }
        PhysicalKey::Code(KeyCode::KeyE) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyE)
        }
        PhysicalKey::Code(KeyCode::KeyF) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyF)
        }
        PhysicalKey::Code(KeyCode::KeyG) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyG)
        }
        PhysicalKey::Code(KeyCode::KeyH) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyH)
        }
        PhysicalKey::Code(KeyCode::KeyI) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyI)
        }
        PhysicalKey::Code(KeyCode::KeyJ) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyJ)
        }
        PhysicalKey::Code(KeyCode::KeyK) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyK)
        }
        PhysicalKey::Code(KeyCode::KeyL) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyL)
        }
        PhysicalKey::Code(KeyCode::KeyM) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyM)
        }
        PhysicalKey::Code(KeyCode::KeyN) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyN)
        }
        PhysicalKey::Code(KeyCode::KeyO) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyO)
        }
        PhysicalKey::Code(KeyCode::KeyP) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyP)
        }
        PhysicalKey::Code(KeyCode::KeyQ) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyQ)
        }
        PhysicalKey::Code(KeyCode::KeyR) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyR)
        }
        PhysicalKey::Code(KeyCode::KeyS) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyS)
        }
        PhysicalKey::Code(KeyCode::KeyT) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyT)
        }
        PhysicalKey::Code(KeyCode::KeyU) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyU)
        }
        PhysicalKey::Code(KeyCode::KeyV) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyV)
        }
        PhysicalKey::Code(KeyCode::KeyW) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyW)
        }
        PhysicalKey::Code(KeyCode::KeyX) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyX)
        }
        PhysicalKey::Code(KeyCode::KeyY) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyY)
        }
        PhysicalKey::Code(KeyCode::KeyZ) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KeyZ)
        }
        PhysicalKey::Code(KeyCode::Minus) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Minus)
        }
        PhysicalKey::Code(KeyCode::Period) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Period)
        }
        PhysicalKey::Code(KeyCode::Quote) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Quote)
        }
        PhysicalKey::Code(KeyCode::Semicolon) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Semicolon)
        }
        PhysicalKey::Code(KeyCode::Slash) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Slash)
        }
        PhysicalKey::Code(KeyCode::AltLeft) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::AltLeft)
        }
        PhysicalKey::Code(KeyCode::AltRight) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::AltRight)
        }
        PhysicalKey::Code(KeyCode::Backspace) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Backspace)
        }
        PhysicalKey::Code(KeyCode::CapsLock) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::CapsLock)
        }
        PhysicalKey::Code(KeyCode::ContextMenu) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ContextMenu)
        }
        PhysicalKey::Code(KeyCode::ControlLeft) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ControlLeft)
        }
        PhysicalKey::Code(KeyCode::ControlRight) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ControlRight)
        }
        PhysicalKey::Code(KeyCode::Enter) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Enter)
        }
        PhysicalKey::Code(KeyCode::SuperLeft) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::SuperLeft)
        }
        PhysicalKey::Code(KeyCode::SuperRight) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::SuperRight)
        }
        PhysicalKey::Code(KeyCode::ShiftLeft) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ShiftLeft)
        }
        PhysicalKey::Code(KeyCode::ShiftRight) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ShiftRight)
        }
        PhysicalKey::Code(KeyCode::Space) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Space)
        }
        PhysicalKey::Code(KeyCode::Tab) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Tab)
        }
        PhysicalKey::Code(KeyCode::Convert) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Convert)
        }
        PhysicalKey::Code(KeyCode::KanaMode) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::KanaMode)
        }
        PhysicalKey::Code(KeyCode::Lang1) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Lang1)
        }
        PhysicalKey::Code(KeyCode::Lang2) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Lang2)
        }
        PhysicalKey::Code(KeyCode::Lang3) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Lang3)
        }
        PhysicalKey::Code(KeyCode::Lang4) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Lang4)
        }
        PhysicalKey::Code(KeyCode::Lang5) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Lang5)
        }
        PhysicalKey::Code(KeyCode::NonConvert) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NonConvert)
        }
        PhysicalKey::Code(KeyCode::Delete) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Delete)
        }
        PhysicalKey::Code(KeyCode::End) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::End)
        }
        PhysicalKey::Code(KeyCode::Help) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Help)
        }
        PhysicalKey::Code(KeyCode::Home) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Home)
        }
        PhysicalKey::Code(KeyCode::Insert) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Insert)
        }
        PhysicalKey::Code(KeyCode::PageDown) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::PageDown)
        }
        PhysicalKey::Code(KeyCode::PageUp) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::PageUp)
        }
        PhysicalKey::Code(KeyCode::ArrowDown) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ArrowDown)
        }
        PhysicalKey::Code(KeyCode::ArrowLeft) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ArrowLeft)
        }
        PhysicalKey::Code(KeyCode::ArrowRight) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ArrowRight)
        }
        PhysicalKey::Code(KeyCode::ArrowUp) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ArrowUp)
        }
        PhysicalKey::Code(KeyCode::NumLock) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumLock)
        }
        PhysicalKey::Code(KeyCode::Numpad0) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad0)
        }
        PhysicalKey::Code(KeyCode::Numpad1) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad1)
        }
        PhysicalKey::Code(KeyCode::Numpad2) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad2)
        }
        PhysicalKey::Code(KeyCode::Numpad3) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad3)
        }
        PhysicalKey::Code(KeyCode::Numpad4) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad4)
        }
        PhysicalKey::Code(KeyCode::Numpad5) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad5)
        }
        PhysicalKey::Code(KeyCode::Numpad6) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad6)
        }
        PhysicalKey::Code(KeyCode::Numpad7) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad7)
        }
        PhysicalKey::Code(KeyCode::Numpad8) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad8)
        }
        PhysicalKey::Code(KeyCode::Numpad9) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Numpad9)
        }
        PhysicalKey::Code(KeyCode::NumpadAdd) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadAdd)
        }
        PhysicalKey::Code(KeyCode::NumpadBackspace) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadBackspace)
        }
        PhysicalKey::Code(KeyCode::NumpadClear) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadClear)
        }
        PhysicalKey::Code(KeyCode::NumpadClearEntry) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadClearEntry)
        }
        PhysicalKey::Code(KeyCode::NumpadComma) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadComma)
        }
        PhysicalKey::Code(KeyCode::NumpadDecimal) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadDecimal)
        }
        PhysicalKey::Code(KeyCode::NumpadDivide) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadDivide)
        }
        PhysicalKey::Code(KeyCode::NumpadEnter) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadEnter)
        }
        PhysicalKey::Code(KeyCode::NumpadEqual) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadEqual)
        }
        PhysicalKey::Code(KeyCode::NumpadHash) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadHash)
        }
        PhysicalKey::Code(KeyCode::NumpadMemoryAdd) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadMemoryAdd)
        }
        PhysicalKey::Code(KeyCode::NumpadMemoryClear) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadMemoryClear)
        }
        PhysicalKey::Code(KeyCode::NumpadMemoryRecall) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadMemoryRecall)
        }
        PhysicalKey::Code(KeyCode::NumpadMemoryStore) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadMemoryStore)
        }
        PhysicalKey::Code(KeyCode::NumpadMemorySubtract) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadMemorySubtract)
        }
        PhysicalKey::Code(KeyCode::NumpadMultiply) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadMultiply)
        }
        PhysicalKey::Code(KeyCode::NumpadParenLeft) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadParenLeft)
        }
        PhysicalKey::Code(KeyCode::NumpadParenRight) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadParenRight)
        }
        PhysicalKey::Code(KeyCode::NumpadStar) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadStar)
        }
        PhysicalKey::Code(KeyCode::NumpadSubtract) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::NumpadSubtract)
        }
        PhysicalKey::Code(KeyCode::Escape) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Escape)
        }
        PhysicalKey::Code(KeyCode::Fn) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Fn)
        }
        PhysicalKey::Code(KeyCode::FnLock) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::FnLock)
        }
        PhysicalKey::Code(KeyCode::PrintScreen) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::PrintScreen)
        }
        PhysicalKey::Code(KeyCode::ScrollLock) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::ScrollLock)
        }
        PhysicalKey::Code(KeyCode::Pause) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Pause)
        }
        PhysicalKey::Code(KeyCode::BrowserBack) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BrowserBack)
        }
        PhysicalKey::Code(KeyCode::BrowserFavorites) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BrowserFavorites)
        }
        PhysicalKey::Code(KeyCode::BrowserForward) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BrowserForward)
        }
        PhysicalKey::Code(KeyCode::BrowserHome) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BrowserHome)
        }
        PhysicalKey::Code(KeyCode::BrowserRefresh) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BrowserRefresh)
        }
        PhysicalKey::Code(KeyCode::BrowserSearch) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BrowserSearch)
        }
        PhysicalKey::Code(KeyCode::BrowserStop) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::BrowserStop)
        }
        PhysicalKey::Code(KeyCode::Eject) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Eject)
        }
        PhysicalKey::Code(KeyCode::LaunchApp1) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::LaunchApp1)
        }
        PhysicalKey::Code(KeyCode::LaunchApp2) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::LaunchApp2)
        }
        PhysicalKey::Code(KeyCode::LaunchMail) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::LaunchMail)
        }
        PhysicalKey::Code(KeyCode::MediaPlayPause) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::MediaPlayPause)
        }
        PhysicalKey::Code(KeyCode::MediaSelect) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::MediaSelect)
        }
        PhysicalKey::Code(KeyCode::MediaStop) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::MediaStop)
        }
        PhysicalKey::Code(KeyCode::MediaTrackNext) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::MediaTrackNext)
        }
        PhysicalKey::Code(KeyCode::MediaTrackPrevious) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::MediaTrackPrevious)
        }
        PhysicalKey::Code(KeyCode::Power) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Power)
        }
        PhysicalKey::Code(KeyCode::Sleep) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Sleep)
        }
        PhysicalKey::Code(KeyCode::AudioVolumeDown) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::AudioVolumeDown)
        }
        PhysicalKey::Code(KeyCode::AudioVolumeMute) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::AudioVolumeMute)
        }
        PhysicalKey::Code(KeyCode::AudioVolumeUp) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::AudioVolumeUp)
        }
        PhysicalKey::Code(KeyCode::WakeUp) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::WakeUp)
        }
        PhysicalKey::Code(KeyCode::Meta) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Meta)
        }
        PhysicalKey::Code(KeyCode::Hyper) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Hyper)
        }
        PhysicalKey::Code(KeyCode::Turbo) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Turbo)
        }
        PhysicalKey::Code(KeyCode::Abort) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Abort)
        }
        PhysicalKey::Code(KeyCode::Resume) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Resume)
        }
        PhysicalKey::Code(KeyCode::Suspend) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Suspend)
        }
        PhysicalKey::Code(KeyCode::Again) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Again)
        }
        PhysicalKey::Code(KeyCode::Copy) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Copy)
        }
        PhysicalKey::Code(KeyCode::Cut) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Cut)
        }
        PhysicalKey::Code(KeyCode::Find) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Find)
        }
        PhysicalKey::Code(KeyCode::Open) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Open)
        }
        PhysicalKey::Code(KeyCode::Paste) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Paste)
        }
        PhysicalKey::Code(KeyCode::Props) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Props)
        }
        PhysicalKey::Code(KeyCode::Select) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Select)
        }
        PhysicalKey::Code(KeyCode::Undo) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Undo)
        }
        PhysicalKey::Code(KeyCode::Hiragana) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Hiragana)
        }
        PhysicalKey::Code(KeyCode::Katakana) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::Katakana)
        }
        PhysicalKey::Code(KeyCode::F1) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F1)
        }
        PhysicalKey::Code(KeyCode::F2) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F2)
        }
        PhysicalKey::Code(KeyCode::F3) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F3)
        }
        PhysicalKey::Code(KeyCode::F4) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F4)
        }
        PhysicalKey::Code(KeyCode::F5) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F5)
        }
        PhysicalKey::Code(KeyCode::F6) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F6)
        }
        PhysicalKey::Code(KeyCode::F7) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F7)
        }
        PhysicalKey::Code(KeyCode::F8) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F8)
        }
        PhysicalKey::Code(KeyCode::F9) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F9)
        }
        PhysicalKey::Code(KeyCode::F10) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F10)
        }
        PhysicalKey::Code(KeyCode::F11) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F11)
        }
        PhysicalKey::Code(KeyCode::F12) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F12)
        }
        PhysicalKey::Code(KeyCode::F13) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F13)
        }
        PhysicalKey::Code(KeyCode::F14) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F14)
        }
        PhysicalKey::Code(KeyCode::F15) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F15)
        }
        PhysicalKey::Code(KeyCode::F16) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F16)
        }
        PhysicalKey::Code(KeyCode::F17) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F17)
        }
        PhysicalKey::Code(KeyCode::F18) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F18)
        }
        PhysicalKey::Code(KeyCode::F19) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F19)
        }
        PhysicalKey::Code(KeyCode::F20) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F20)
        }
        PhysicalKey::Code(KeyCode::F21) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F21)
        }
        PhysicalKey::Code(KeyCode::F22) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F22)
        }
        PhysicalKey::Code(KeyCode::F23) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F23)
        }
        PhysicalKey::Code(KeyCode::F24) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F24)
        }
        PhysicalKey::Code(KeyCode::F25) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F25)
        }
        PhysicalKey::Code(KeyCode::F26) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F26)
        }
        PhysicalKey::Code(KeyCode::F27) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F27)
        }
        PhysicalKey::Code(KeyCode::F28) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F28)
        }
        PhysicalKey::Code(KeyCode::F29) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F29)
        }
        PhysicalKey::Code(KeyCode::F30) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F30)
        }
        PhysicalKey::Code(KeyCode::F31) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F31)
        }
        PhysicalKey::Code(KeyCode::F32) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F32)
        }
        PhysicalKey::Code(KeyCode::F33) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F33)
        }
        PhysicalKey::Code(KeyCode::F34) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F34)
        }
        PhysicalKey::Code(KeyCode::F35) => {
            keyboard::PhysicalKey::Code(keyboard::KeyCode::F35)
        }
        PhysicalKey::Code(_) => unreachable!(), // needed because of #[non_exhaustive] in winit
        PhysicalKey::Unidentified(native_key_code) => {
            let native_key_code = match native_key_code {
                NativeKeyCode::Unidentified => {
                    keyboard::NativeKeyCode::Unidentified
                }
                NativeKeyCode::Android(scancode) => {
                    keyboard::NativeKeyCode::Android(scancode)
                }
                NativeKeyCode::MacOS(scancode) => {
                    keyboard::NativeKeyCode::MacOS(scancode)
                }
                NativeKeyCode::Windows(scancode) => {
                    keyboard::NativeKeyCode::Windows(scancode)
                }
                NativeKeyCode::Xkb(keycode) => {
                    keyboard::NativeKeyCode::Xkb(keycode)
                }
            };

            keyboard::PhysicalKey::Unidentified(native_key_code)
        }
    }
}

/// Converts some [`UserAttention`] into it's `winit` counterpart.
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

/// Converts some [`window::Icon`] into it's `winit` counterpart.
///
/// Returns `None` if there is an error during the conversion.
pub fn icon(icon: window::Icon) -> Option<winit::window::Icon> {
    let (pixels, size) = icon.into_raw();

    winit::window::Icon::from_rgba(pixels, size.width, size.height).ok()
}

// See: https://en.wikipedia.org/wiki/Private_Use_Areas
fn is_private_use(c: char) -> bool {
    ('\u{E000}'..='\u{F8FF}').contains(&c)
}
