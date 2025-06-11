mod state;

use state::State;

pub use crate::core::window::{Event, Id, RedrawRequest, Settings};

use crate::conversion;
use crate::core::alignment;
use crate::core::input_method;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::theme;
use crate::core::time::Instant;
use crate::core::{
    Color, InputMethod, Padding, Point, Rectangle, Size, Text, Vector,
};
use crate::graphics::Compositor;
use crate::program::{self, Program};

use winit::dpi::{LogicalPosition, LogicalSize};
use winit::monitor::MonitorHandle;

use std::collections::BTreeMap;
use std::sync::Arc;

#[allow(missing_debug_implementations)]
pub struct WindowManager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    aliases: BTreeMap<winit::window::WindowId, Id>,
    entries: BTreeMap<Id, Window<P, C>>,
}

impl<P, C> WindowManager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    pub fn new() -> Self {
        Self {
            aliases: BTreeMap::new(),
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        id: Id,
        window: Arc<winit::window::Window>,
        program: &program::Instance<P>,
        compositor: &mut C,
        exit_on_close_request: bool,
    ) -> &mut Window<P, C> {
        let state = State::new(program, id, &window);
        let viewport_version = state.viewport_version();
        let physical_size = state.physical_size();
        let surface = compositor.create_surface(
            window.clone(),
            physical_size.width,
            physical_size.height,
        );
        let renderer = compositor.create_renderer();

        let _ = self.aliases.insert(window.id(), id);

        let _ = self.entries.insert(
            id,
            Window {
                raw: window,
                state,
                viewport_version,
                exit_on_close_request,
                surface,
                renderer,
                mouse_interaction: mouse::Interaction::None,
                redraw_at: None,
                preedit: None,
                ime_state: None,
            },
        );

        self.entries
            .get_mut(&id)
            .expect("Get window that was just inserted")
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn is_idle(&self) -> bool {
        self.entries
            .values()
            .all(|window| window.redraw_at.is_none())
    }

    pub fn redraw_at(&self) -> Option<Instant> {
        self.entries
            .values()
            .filter_map(|window| window.redraw_at)
            .min()
    }

    pub fn first(&self) -> Option<&Window<P, C>> {
        self.entries.first_key_value().map(|(_id, window)| window)
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (Id, &mut Window<P, C>)> {
        self.entries.iter_mut().map(|(k, v)| (*k, v))
    }

    pub fn get(&self, id: Id) -> Option<&Window<P, C>> {
        self.entries.get(&id)
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Window<P, C>> {
        self.entries.get_mut(&id)
    }

    pub fn get_mut_alias(
        &mut self,
        id: winit::window::WindowId,
    ) -> Option<(Id, &mut Window<P, C>)> {
        let id = self.aliases.get(&id).copied()?;

        Some((id, self.get_mut(id)?))
    }

    pub fn last_monitor(&self) -> Option<MonitorHandle> {
        self.entries.values().last()?.raw.current_monitor()
    }

    pub fn remove(&mut self, id: Id) -> Option<Window<P, C>> {
        let window = self.entries.remove(&id)?;
        let _ = self.aliases.remove(&window.raw.id());

        Some(window)
    }
}

impl<P, C> Default for WindowManager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    fn default() -> Self {
        Self::new()
    }
}

#[allow(missing_debug_implementations)]
pub struct Window<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    pub raw: Arc<winit::window::Window>,
    pub state: State<P>,
    pub viewport_version: u64,
    pub exit_on_close_request: bool,
    pub mouse_interaction: mouse::Interaction,
    pub surface: C::Surface,
    pub renderer: P::Renderer,
    pub redraw_at: Option<Instant>,
    preedit: Option<Preedit<P::Renderer>>,
    ime_state: Option<(Point, input_method::Purpose)>,
}

impl<P, C> Window<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    pub fn position(&self) -> Option<Point> {
        self.raw
            .outer_position()
            .ok()
            .map(|position| position.to_logical(self.raw.scale_factor()))
            .map(|position| Point {
                x: position.x,
                y: position.y,
            })
    }

    pub fn size(&self) -> Size {
        let size = self.raw.inner_size().to_logical(self.raw.scale_factor());

        Size::new(size.width, size.height)
    }

    pub fn request_redraw(&mut self, redraw_request: RedrawRequest) {
        match redraw_request {
            RedrawRequest::NextFrame => {
                self.raw.request_redraw();
                self.redraw_at = None;
            }
            RedrawRequest::At(at) => {
                self.redraw_at = Some(at);
            }
            RedrawRequest::Wait => {}
        }
    }

    pub fn request_input_method(&mut self, input_method: InputMethod) {
        match input_method {
            InputMethod::Disabled => {
                self.disable_ime();
            }
            InputMethod::Enabled {
                position,
                purpose,
                preedit,
            } => {
                self.enable_ime(position, purpose);

                if let Some(preedit) = preedit {
                    if preedit.content.is_empty() {
                        self.preedit = None;
                    } else {
                        let mut overlay =
                            self.preedit.take().unwrap_or_else(Preedit::new);

                        overlay.update(
                            position,
                            &preedit,
                            self.state.background_color(),
                            &self.renderer,
                        );

                        self.preedit = Some(overlay);
                    }
                } else {
                    self.preedit = None;
                }
            }
        }
    }

    pub fn update_mouse(&mut self, interaction: mouse::Interaction) {
        if interaction != self.mouse_interaction {
            self.raw
                .set_cursor(conversion::mouse_interaction(interaction));

            self.mouse_interaction = interaction;
        }
    }

    pub fn draw_preedit(&mut self) {
        if let Some(preedit) = &self.preedit {
            preedit.draw(
                &mut self.renderer,
                self.state.text_color(),
                self.state.background_color(),
                &Rectangle::new(
                    Point::ORIGIN,
                    self.state.viewport().logical_size(),
                ),
            );
        }
    }

    fn enable_ime(&mut self, position: Point, purpose: input_method::Purpose) {
        if self.ime_state.is_none() {
            self.raw.set_ime_allowed(true);
        }

        if self.ime_state != Some((position, purpose)) {
            self.raw.set_ime_cursor_area(
                LogicalPosition::new(position.x, position.y),
                LogicalSize::new(10, 10), // TODO?
            );
            self.raw.set_ime_purpose(conversion::ime_purpose(purpose));

            self.ime_state = Some((position, purpose));
        }
    }

    fn disable_ime(&mut self) {
        if self.ime_state.is_some() {
            self.raw.set_ime_allowed(false);
            self.ime_state = None;
        }

        self.preedit = None;
    }
}

struct Preedit<Renderer>
where
    Renderer: text::Renderer,
{
    position: Point,
    content: Renderer::Paragraph,
    spans: Vec<text::Span<'static, (), Renderer::Font>>,
}

impl<Renderer> Preedit<Renderer>
where
    Renderer: text::Renderer,
{
    fn new() -> Self {
        Self {
            position: Point::ORIGIN,
            spans: Vec::new(),
            content: Renderer::Paragraph::default(),
        }
    }

    fn update(
        &mut self,
        position: Point,
        preedit: &input_method::Preedit,
        background: Color,
        renderer: &Renderer,
    ) {
        self.position = position;

        let spans = match &preedit.selection {
            Some(selection) => {
                vec![
                    text::Span::new(&preedit.content[..selection.start]),
                    text::Span::new(if selection.start == selection.end {
                        "\u{200A}"
                    } else {
                        &preedit.content[selection.start..selection.end]
                    })
                    .color(background),
                    text::Span::new(&preedit.content[selection.end..]),
                ]
            }
            _ => vec![text::Span::new(&preedit.content)],
        };

        if spans != self.spans.as_slice() {
            use text::Paragraph as _;

            self.content = Renderer::Paragraph::with_spans(Text {
                content: &spans,
                bounds: Size::INFINITY,
                size: preedit
                    .text_size
                    .unwrap_or_else(|| renderer.default_size()),
                line_height: text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            });

            self.spans.clear();
            self.spans
                .extend(spans.into_iter().map(text::Span::to_static));
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        color: Color,
        background: Color,
        viewport: &Rectangle,
    ) {
        use text::Paragraph as _;

        if self.content.min_width() < 1.0 {
            return;
        }

        let mut bounds = Rectangle::new(
            self.position - Vector::new(0.0, self.content.min_height()),
            self.content.min_bounds(),
        );

        bounds.x = bounds
            .x
            .max(viewport.x)
            .min(viewport.x + viewport.width - bounds.width);

        bounds.y = bounds
            .y
            .max(viewport.y)
            .min(viewport.y + viewport.height - bounds.height);

        renderer.with_layer(bounds, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..Default::default()
                },
                background,
            );

            renderer.fill_paragraph(
                &self.content,
                bounds.position(),
                color,
                bounds,
            );

            const UNDERLINE: f32 = 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds.shrink(Padding {
                        top: bounds.height - UNDERLINE,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                color,
            );

            for span_bounds in self.content.span_bounds(1) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: span_bounds
                            + (bounds.position() - Point::ORIGIN),
                        ..Default::default()
                    },
                    color,
                );
            }
        });
    }
}
