use crate::conversion;
use crate::core::{mouse, window};
use crate::core::{Color, Size};
use crate::graphics::Viewport;
use crate::program::{self, Program};
use std::fmt::{Debug, Formatter};

use winit::event::{Touch, WindowEvent};
use winit::window::Window;

/// The state of a multi-windowed [`Program`].
pub struct State<P: Program>
where
    P::Theme: program::DefaultStyle,
{
    title: String,
    scale_factor: f64,
    viewport: Viewport,
    viewport_version: u64,
    cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    modifiers: winit::keyboard::ModifiersState,
    theme: P::Theme,
    appearance: program::Appearance,
}

impl<P: Program> Debug for State<P>
where
    P::Theme: program::DefaultStyle,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("multi_window::State")
            .field("title", &self.title)
            .field("scale_factor", &self.scale_factor)
            .field("viewport", &self.viewport)
            .field("viewport_version", &self.viewport_version)
            .field("cursor_position", &self.cursor_position)
            .field("appearance", &self.appearance)
            .finish()
    }
}

impl<P: Program> State<P>
where
    P::Theme: program::DefaultStyle,
{
    /// Creates a new [`State`] for the provided [`Program`]'s `window`.
    pub fn new(
        application: &P,
        window_id: window::Id,
        window: &Window,
    ) -> Self {
        let title = application.title(window_id);
        let scale_factor = application.scale_factor(window_id);
        let theme = application.theme(window_id);
        let appearance = application.style(&theme);

        let viewport = {
            let physical_size = window.inner_size();

            Viewport::with_physical_size(
                Size::new(physical_size.width, physical_size.height),
                window.scale_factor() * scale_factor,
            )
        };

        Self {
            title,
            scale_factor,
            viewport,
            viewport_version: 0,
            cursor_position: None,
            modifiers: winit::keyboard::ModifiersState::default(),
            theme,
            appearance,
        }
    }

    /// Returns the current [`Viewport`] of the [`State`].
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns the version of the [`Viewport`] of the [`State`].
    ///
    /// The version is incremented every time the [`Viewport`] changes.
    pub fn viewport_version(&self) -> u64 {
        self.viewport_version
    }

    /// Returns the physical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn physical_size(&self) -> Size<u32> {
        self.viewport.physical_size()
    }

    /// Returns the logical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn logical_size(&self) -> Size<f32> {
        self.viewport.logical_size()
    }

    /// Returns the current scale factor of the [`Viewport`] of the [`State`].
    pub fn scale_factor(&self) -> f64 {
        self.viewport.scale_factor()
    }

    /// Returns the current cursor position of the [`State`].
    pub fn cursor(&self) -> mouse::Cursor {
        self.cursor_position
            .map(|cursor_position| {
                conversion::cursor_position(
                    cursor_position,
                    self.viewport.scale_factor(),
                )
            })
            .map(mouse::Cursor::Available)
            .unwrap_or(mouse::Cursor::Unavailable)
    }

    /// Returns the current keyboard modifiers of the [`State`].
    pub fn modifiers(&self) -> winit::keyboard::ModifiersState {
        self.modifiers
    }

    /// Returns the current theme of the [`State`].
    pub fn theme(&self) -> &P::Theme {
        &self.theme
    }

    /// Returns the current background [`Color`] of the [`State`].
    pub fn background_color(&self) -> Color {
        self.appearance.background_color
    }

    /// Returns the current text [`Color`] of the [`State`].
    pub fn text_color(&self) -> Color {
        self.appearance.text_color
    }

    /// Processes the provided window event and updates the [`State`] accordingly.
    pub fn update(
        &mut self,
        window: &Window,
        event: &WindowEvent,
        _debug: &mut crate::runtime::Debug,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                let size = Size::new(new_size.width, new_size.height);

                self.viewport = Viewport::with_physical_size(
                    size,
                    window.scale_factor() * self.scale_factor,
                );

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: new_scale_factor,
                ..
            } => {
                let size = self.viewport.physical_size();

                self.viewport = Viewport::with_physical_size(
                    size,
                    new_scale_factor * self.scale_factor,
                );

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            WindowEvent::CursorMoved { position, .. }
            | WindowEvent::Touch(Touch {
                location: position, ..
            }) => {
                self.cursor_position = Some(*position);
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_position = None;
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }
            #[cfg(feature = "debug")]
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key:
                            winit::keyboard::Key::Named(
                                winit::keyboard::NamedKey::F12,
                            ),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                _debug.toggle();
                window.request_redraw();
            }
            _ => {}
        }
    }

    /// Synchronizes the [`State`] with its [`Program`] and its respective
    /// window.
    ///
    /// Normally, a [`Program`] should be synchronized with its [`State`]
    /// and window after calling [`State::update`].
    pub fn synchronize(
        &mut self,
        application: &P,
        window_id: window::Id,
        window: &Window,
    ) {
        // Update window title
        let new_title = application.title(window_id);

        if self.title != new_title {
            window.set_title(&new_title);
            self.title = new_title;
        }

        // Update scale factor and size
        let new_scale_factor = application.scale_factor(window_id);
        let new_size = window.inner_size();
        let current_size = self.viewport.physical_size();

        if self.scale_factor != new_scale_factor
            || (current_size.width, current_size.height)
                != (new_size.width, new_size.height)
        {
            self.viewport = Viewport::with_physical_size(
                Size::new(new_size.width, new_size.height),
                window.scale_factor() * new_scale_factor,
            );
            self.viewport_version = self.viewport_version.wrapping_add(1);

            self.scale_factor = new_scale_factor;
        }

        // Update theme and appearance
        self.theme = application.theme(window_id);
        self.appearance = application.style(&self.theme);
    }
}
