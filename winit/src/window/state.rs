use crate::conversion;
use crate::core::{Color, Size};
use crate::core::{mouse, theme, window};
use crate::graphics::Viewport;
use crate::program::{self, Program};

use winit::event::{Touch, WindowEvent};
use winit::window::Window;

use std::fmt::{Debug, Formatter};

/// The state of the window of a [`Program`].
pub struct State<P: Program>
where
    P::Theme: theme::Base,
{
    title: String,
    scale_factor: f32,
    viewport: Viewport,
    viewport_version: u64,
    cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    modifiers: winit::keyboard::ModifiersState,
    theme: Option<P::Theme>,
    theme_mode: theme::Mode,
    default_theme: P::Theme,
    style: theme::Style,
}

impl<P: Program> Debug for State<P>
where
    P::Theme: theme::Base,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("window::State")
            .field("title", &self.title)
            .field("scale_factor", &self.scale_factor)
            .field("viewport", &self.viewport)
            .field("viewport_version", &self.viewport_version)
            .field("cursor_position", &self.cursor_position)
            .field("style", &self.style)
            .finish()
    }
}

impl<P: Program> State<P>
where
    P::Theme: theme::Base,
{
    /// Creates a new [`State`] for the provided [`Program`]'s `window`.
    pub fn new(
        program: &program::Instance<P>,
        window_id: window::Id,
        window: &Window,
        system_theme: theme::Mode,
    ) -> Self {
        let title = program.title(window_id);
        let scale_factor = program.scale_factor(window_id);
        let theme = program.theme(window_id);
        let theme_mode =
            theme.as_ref().map(theme::Base::mode).unwrap_or_default();
        let default_theme = <P::Theme as theme::Base>::default(system_theme);
        let style = program.style(theme.as_ref().unwrap_or(&default_theme));

        let viewport = {
            let physical_size = window.inner_size();

            Viewport::with_physical_size(
                Size::new(physical_size.width, physical_size.height),
                window.scale_factor() as f32 * scale_factor,
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
            theme_mode,
            default_theme,
            style,
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
    pub fn scale_factor(&self) -> f32 {
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
        self.theme.as_ref().unwrap_or(&self.default_theme)
    }

    /// Returns the current [`theme::Mode`] of the [`State`].
    pub fn theme_mode(&self) -> theme::Mode {
        self.theme_mode
    }

    /// Returns the current background [`Color`] of the [`State`].
    pub fn background_color(&self) -> Color {
        self.style.background_color
    }

    /// Returns the current text [`Color`] of the [`State`].
    pub fn text_color(&self) -> Color {
        self.style.text_color
    }

    /// Processes the provided window event and updates the [`State`] accordingly.
    pub fn update(
        &mut self,
        program: &program::Instance<P>,
        window: &Window,
        event: &WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                let size = Size::new(new_size.width, new_size.height);

                self.viewport = Viewport::with_physical_size(
                    size,
                    window.scale_factor() as f32 * self.scale_factor,
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
                    *new_scale_factor as f32 * self.scale_factor,
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
            WindowEvent::ThemeChanged(theme) => {
                self.default_theme = <P::Theme as theme::Base>::default(
                    conversion::theme_mode(*theme),
                );

                if self.theme.is_none() {
                    self.style = program.style(&self.default_theme);
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    /// Synchronizes the [`State`] with its [`Program`] and its respective
    /// window.
    ///
    /// Normally, a [`Program`] should be synchronized with its [`State`]
    /// and window after calling [`Program::update`].
    pub fn synchronize(
        &mut self,
        program: &program::Instance<P>,
        window_id: window::Id,
        window: &Window,
    ) {
        // Update window title
        let new_title = program.title(window_id);

        if self.title != new_title {
            window.set_title(&new_title);
            self.title = new_title;
        }

        // Update scale factor and size
        let new_scale_factor = program.scale_factor(window_id);
        let new_size = window.inner_size();
        let current_size = self.viewport.physical_size();

        if self.scale_factor != new_scale_factor
            || (current_size.width, current_size.height)
                != (new_size.width, new_size.height)
        {
            self.viewport = Viewport::with_physical_size(
                Size::new(new_size.width, new_size.height),
                window.scale_factor() as f32 * new_scale_factor,
            );
            self.viewport_version = self.viewport_version.wrapping_add(1);

            self.scale_factor = new_scale_factor;
        }

        // Update theme and appearance
        self.theme = program.theme(window_id);
        self.style = program.style(self.theme());

        let new_mode = self
            .theme
            .as_ref()
            .map(theme::Base::mode)
            .unwrap_or_default();

        if self.theme_mode != new_mode {
            #[cfg(not(target_os = "linux"))]
            {
                window.set_theme(conversion::window_theme(new_mode));

                // Assume the old mode matches the system one
                // We will be notified otherwise
                if new_mode == theme::Mode::None {
                    self.default_theme =
                        <P::Theme as theme::Base>::default(self.theme_mode);

                    if self.theme.is_none() {
                        self.style = program.style(&self.default_theme);
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                // mundy always notifies system theme changes, so we
                // just restore the default theme mode.
                let new_mode = if new_mode == theme::Mode::None {
                    theme::Base::mode(&self.default_theme)
                } else {
                    new_mode
                };

                window.set_theme(conversion::window_theme(new_mode));
            }

            self.theme_mode = new_mode;
        }
    }
}
