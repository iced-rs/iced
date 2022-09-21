use crate::application::{self, StyleSheet as _};
use crate::conversion;
use crate::multi_window::{Application, Event};
use crate::window;
use crate::{Color, Debug, Point, Size, Viewport};

use glutin::event::{Touch, WindowEvent};
use glutin::event_loop::EventLoopProxy;
use glutin::window::Window;
use std::collections::HashMap;
use std::marker::PhantomData;

/// The state of a windowed [`Application`].
#[allow(missing_debug_implementations)]
pub struct State<A: Application>
where
    <A::Renderer as crate::Renderer>::Theme: application::StyleSheet,
{
    title: String,
    scale_factor: f64,
    viewport: Viewport,
    viewport_changed: bool,
    cursor_position: glutin::dpi::PhysicalPosition<f64>,
    modifiers: glutin::event::ModifiersState,
    theme: <A::Renderer as crate::Renderer>::Theme,
    appearance: iced_winit::application::Appearance,
    application: PhantomData<A>,
}

impl<A: Application> State<A>
where
    <A::Renderer as crate::Renderer>::Theme: application::StyleSheet,
{
    /// Creates a new [`State`] for the provided [`Application`] and window.
    pub fn new(application: &A, window: &Window) -> Self {
        let title = application.title();
        let scale_factor = application.scale_factor();
        let theme = application.theme();
        let appearance = theme.appearance(application.style());

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
            viewport_changed: false,
            // TODO: Encode cursor availability in the type-system
            cursor_position: glutin::dpi::PhysicalPosition::new(-1.0, -1.0),
            modifiers: glutin::event::ModifiersState::default(),
            theme,
            appearance,
            application: PhantomData,
        }
    }

    /// Returns the current [`Viewport`] of the [`State`].
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// TODO(derezzedex)
    pub fn viewport_changed(&self) -> bool {
        self.viewport_changed
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
    pub fn modifiers(&self) -> glutin::event::ModifiersState {
        self.modifiers
    }

    /// Returns the current theme of the [`State`].
    pub fn theme(&self) -> &<A::Renderer as crate::Renderer>::Theme {
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

                self.viewport_changed = true;
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

                self.viewport_changed = true;
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
                    glutin::dpi::PhysicalPosition::new(-1.0, -1.0);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
            }
            #[cfg(feature = "debug")]
            WindowEvent::KeyboardInput {
                input:
                    glutin::event::KeyboardInput {
                        virtual_keycode:
                            Some(glutin::event::VirtualKeyCode::F12),
                        state: glutin::event::ElementState::Pressed,
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
    pub fn synchronize(
        &mut self,
        application: &A,
        windows: &HashMap<window::Id, Window>,
        proxy: &EventLoopProxy<Event<A::Message>>,
    ) {
        let new_windows = application.windows();

        // Check for windows to close
        for window_id in windows.keys() {
            if !new_windows.iter().any(|(id, _)| id == window_id) {
                proxy
                    .send_event(Event::CloseWindow(*window_id))
                    .expect("Failed to send message");
            }
        }

        // Check for windows to spawn
        for (id, settings) in new_windows {
            if !windows.contains_key(&id) {
                proxy
                    .send_event(Event::NewWindow(id, settings))
                    .expect("Failed to send message");
            }
        }

        let window = windows.values().next().expect("No window found");

        // Update window title
        let new_title = application.title();

        if self.title != new_title {
            window.set_title(&new_title);

            self.title = new_title;
        }

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

        // Update theme and appearance
        self.theme = application.theme();
        self.appearance = self.theme.appearance(application.style());
    }
}
