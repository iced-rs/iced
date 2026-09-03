use std::sync::Arc;

use winit::dpi::LogicalPosition;
use winit::dpi::LogicalSize;

use crate::Proxy;
use crate::conversion;
use crate::core;
use crate::core::InputMethod;
use crate::core::Point;
use crate::core::Rectangle;
use crate::core::input_method;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::shell;
use crate::core::time::Instant;
use crate::core::window::Id;
use crate::core::window::RedrawRequest;
use crate::graphics::Compositor;
use crate::program::Program;
use crate::window::Preedit;
use crate::window::state::State;

pub struct Interface<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
{
    pub waker: shell::Waker,
    pub mouse_interaction: mouse::Interaction,
    pub surface: C::Surface,
    pub surface_version: u64,
    pub renderer: P::Renderer,
    pub redraw_at: Option<Instant>,
    preedit: Option<Preedit<P::Renderer>>,
    ime_state: Option<(Rectangle, input_method::Purpose)>,
}

impl<P, C> Interface<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
{
    pub fn new(
        id: Id,
        window: Arc<winit::window::Window>,
        compositor: &mut C,
        proxy: Proxy<P::Message>,
        renderer_settings: renderer::Settings,
        state: &State<P>,
    ) -> Self {
        let surface_size = state.physical_size();
        let surface_version = state.surface_version();
        let surface =
            compositor.create_surface(window.clone(), surface_size.width, surface_size.height);
        let renderer = compositor.create_renderer(renderer_settings);

        let waker = shell::Waker::new(move || {
            proxy.send_action(iced_runtime::Action::Event {
                window: id,
                event: core::Event::Waken,
            });
        });

        Interface {
            waker,
            mouse_interaction: mouse::Interaction::None,
            surface,
            surface_version,
            renderer,
            redraw_at: None,
            preedit: None,
            ime_state: None,
        }
    }
}

impl<P, C> Interface<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
{
    pub fn request_redraw(&mut self, redraw_request: RedrawRequest, raw: &winit::window::Window) {
        match redraw_request {
            RedrawRequest::NextFrame => {
                raw.request_redraw();
                self.redraw_at = None;
            }
            RedrawRequest::At(at) => {
                self.redraw_at = Some(at);
            }
            RedrawRequest::Wait => {}
        }
    }

    pub fn request_input_method(
        &mut self,
        input_method: InputMethod,
        raw: &winit::window::Window,
        state: &State<P>,
    ) {
        match input_method {
            InputMethod::Disabled => {
                self.disable_ime(raw);
            }
            InputMethod::Enabled {
                cursor,
                purpose,
                preedit,
            } => {
                self.enable_ime(cursor, purpose, raw);

                if let Some(preedit) = preedit {
                    if preedit.content.is_empty() {
                        self.preedit = None;
                    } else {
                        let mut overlay = self.preedit.take().unwrap_or_else(Preedit::new);

                        overlay.update(cursor, &preedit, state.background_color(), &self.renderer);

                        self.preedit = Some(overlay);
                    }
                } else {
                    self.preedit = None;
                }
            }
        }
    }

    pub fn update_mouse(&mut self, interaction: mouse::Interaction, raw: &winit::window::Window) {
        if interaction != self.mouse_interaction {
            if let Some(icon) = conversion::mouse_interaction(interaction) {
                raw.set_cursor(icon);

                if self.mouse_interaction == mouse::Interaction::Hidden {
                    raw.set_cursor_visible(true);
                }
            } else {
                raw.set_cursor_visible(false);
            }

            self.mouse_interaction = interaction;
        }
    }

    pub fn draw_preedit(&mut self, state: &State<P>) {
        if let Some(preedit) = &self.preedit {
            preedit.draw(
                &mut self.renderer,
                state.text_color(),
                state.background_color(),
                &Rectangle::new(Point::ORIGIN, state.viewport().logical_size()),
            );
        }
    }

    fn enable_ime(
        &mut self,
        cursor: Rectangle,
        purpose: input_method::Purpose,
        raw: &winit::window::Window,
    ) {
        if self.ime_state.is_none() {
            raw.set_ime_allowed(true);
        }

        if self.ime_state != Some((cursor, purpose)) {
            raw.set_ime_cursor_area(
                LogicalPosition::new(cursor.x, cursor.y),
                LogicalSize::new(cursor.width, cursor.height),
            );
            raw.set_ime_purpose(conversion::ime_purpose(purpose));

            self.ime_state = Some((cursor, purpose));
        }
    }

    fn disable_ime(&mut self, raw: &winit::window::Window) {
        if self.ime_state.is_some() {
            raw.set_ime_allowed(false);
            self.ime_state = None;
        }

        self.preedit = None;
    }
}
