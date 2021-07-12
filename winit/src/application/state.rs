use crate::conversion;
use crate::{Application, Color, Debug, Menu, Mode, Point, Size, Viewport};

use std::marker::PhantomData;
use winit::event::{Touch, WindowEvent};
use winit::window::Window;

/// The state of a windowed [`Application`].
#[derive(Debug, Clone)]
pub struct State<A: Application> {
    title: String,
    menu: Menu<A::Message>,
    mode: Mode,
    background_color: Color,
    scale_factor: f64,
    viewport: Viewport,
    viewport_version: usize,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    modifiers: winit::event::ModifiersState,
    application: PhantomData<A>,
}

impl<A: Application> State<A> {
    /// Creates a new [`State`] for the provided [`Application`] and window.
    pub fn new(application: &A, window: &Window) -> Self {
        let title = application.title();
        let menu = application.menu();
        let mode = application.mode();
        let background_color = application.background_color();
        let scale_factor = application.scale_factor();

        let viewport = {
            let physical_size = window.inner_size();

            Viewport::with_physical_size(
                Size::new(physical_size.width, physical_size.height),
                window.scale_factor() * scale_factor,
            )
        };

        Self {
            title,
            menu,
            mode,
            background_color,
            scale_factor,
            viewport,
            viewport_version: 0,
            // TODO: Encode cursor availability in the type-system
            cursor_position: winit::dpi::PhysicalPosition::new(-1.0, -1.0),
            modifiers: winit::event::ModifiersState::default(),
            application: PhantomData,
        }
    }

    /// Returns the current [`Menu`] of the [`State`].
    pub fn menu(&self) -> &Menu<A::Message> {
        &self.menu
    }

    /// Returns the current background [`Color`] of the [`State`].
    pub fn background_color(&self) -> Color {
        self.background_color
    }

    /// Returns the current [`Viewport`] of the [`State`].
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns the version of the [`Viewport`] of the [`State`].
    ///
    /// The version is incremented every time the [`Viewport`] changes.
    pub fn viewport_version(&self) -> usize {
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
    pub fn cursor_position(&self) -> Point {
        conversion::cursor_position(
            self.cursor_position,
            self.viewport.scale_factor(),
        )
    }

    /// Returns the current keyboard modifiers of the [`State`].
    pub fn modifiers(&self) -> winit::event::ModifiersState {
        self.modifiers
    }

    /// Processes the provided window event and updates the [`State`]
    /// accordingly.
    pub fn update(
        &mut self,
        window: &Window,
        event: &WindowEvent<'_>,
        _debug: &mut Debug,
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
                new_inner_size,
            } => {
                let size =
                    Size::new(new_inner_size.width, new_inner_size.height);

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
                self.cursor_position = *position;
            }
            WindowEvent::CursorLeft { .. } => {
                // TODO: Encode cursor availability in the type-system
                self.cursor_position =
                    winit::dpi::PhysicalPosition::new(-1.0, -1.0);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
            }
            #[cfg(feature = "debug")]
            WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        virtual_keycode: Some(winit::event::VirtualKeyCode::F12),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => _debug.toggle(),
            _ => {}
        }
    }

    /// Synchronizes the [`State`] with its [`Application`] and its respective
    /// window.
    ///
    /// Normally an [`Application`] should be synchronized with its [`State`]
    /// and window after calling [`Application::update`].
    ///
    /// [`Application::update`]: crate::Program::update
    pub fn synchronize(&mut self, application: &A, window: &Window) {
        // Update window title
        let new_title = application.title();

        if self.title != new_title {
            window.set_title(&new_title);

            self.title = new_title;
        }

        // Update window mode
        let new_mode = application.mode();

        if self.mode != new_mode {
            window.set_fullscreen(conversion::fullscreen(
                window.current_monitor(),
                new_mode,
            ));

            window.set_visible(conversion::visible(new_mode));

            self.mode = new_mode;
        }

        // Update background color
        self.background_color = application.background_color();

        // Update scale factor
        let new_scale_factor = application.scale_factor();

        if self.scale_factor != new_scale_factor {
            let size = window.inner_size();

            self.viewport = Viewport::with_physical_size(
                Size::new(size.width, size.height),
                window.scale_factor() * new_scale_factor,
            );

            self.scale_factor = new_scale_factor;
        }

        // Update menu
        let new_menu = application.menu();

        if self.menu != new_menu {
            window.set_menu(Some(conversion::menu(&new_menu)));

            self.menu = new_menu;
        }
    }
}
