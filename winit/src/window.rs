mod state;

use state::State;

pub use crate::core::window::{Event, Id, RedrawRequest, Settings};

use crate::Proxy;
use crate::conversion;
use crate::core;
use crate::core::alignment;
use crate::core::input_method;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::shell;
use crate::core::text;
use crate::core::theme;
use crate::core::time::{Duration, Instant};
use crate::core::{Color, InputMethod, Padding, Point, Rectangle, Renderer, Size, Text, Vector};
use crate::graphics::Compositor;
use crate::program::{self, Program};

use winit::dpi::{LogicalPosition, LogicalSize};
use winit::monitor::MonitorHandle;

use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Manager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    aliases: BTreeMap<winit::window::WindowId, Id>,
    entries: BTreeMap<Id, Window<P, C>>,
}

impl<P, C> Manager<P, C>
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
        proxy: Proxy<P::Message>,
        renderer_settings: renderer::Settings,
        exit_on_close_request: bool,
        system_theme: theme::Mode,
    ) -> &mut Window<P, C> {
        let state = State::new(program, id, &window, system_theme);
        let mut surface = compositor.create_surface(window.clone());
        let surface_size = state.physical_size();
        let surface_version = state.surface_version();

        compositor.configure_surface(&mut surface, surface_size.width, surface_size.height);

        let renderer = compositor.create_renderer(renderer_settings);

        let waker = shell::Waker::new(move || {
            proxy.send_action(iced_runtime::Action::Event {
                window: id,
                event: core::Event::Waken,
            });
        });

        let _ = self.aliases.insert(window.id(), id);

        let _ = self.entries.insert(
            id,
            Window {
                raw: window,
                waker,
                state,
                exit_on_close_request,
                surface,
                surface_version,
                surface_error_at: None,
                renderer,
                mouse_interaction: mouse::Interaction::None,
                redraw_at: None,
                frame_clock: FrameClock {
                    anchor: Instant::now(),
                    period: None,
                },
                preedit: None,
                ime_state: None,
                last_frame: None,
                frame_burst: 0,
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

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Id, &mut Window<P, C>)> {
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

    pub fn replace_with(&mut self, mut f: impl FnMut(Window<P, C>) -> Window<P, C>) {
        self.entries = std::mem::take(&mut self.entries)
            .into_iter()
            .map(|(id, window)| (id, f(window)))
            .collect();
    }
}

impl<P, C> Default for Manager<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Window<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    pub raw: Arc<winit::window::Window>,
    pub waker: shell::Waker,
    pub state: State<P>,
    pub exit_on_close_request: bool,
    pub mouse_interaction: mouse::Interaction,
    pub surface: C::Surface,
    pub surface_version: u64,
    pub surface_error_at: Option<Instant>,
    pub renderer: P::Renderer,
    pub redraw_at: Option<Instant>,
    frame_clock: FrameClock,
    preedit: Option<Preedit<P::Renderer>>,
    ime_state: Option<(Rectangle, input_method::Purpose)>,
    last_frame: Option<Instant>,
    frame_burst: usize,
}

struct FrameClock {
    anchor: Instant,
    period: Option<Duration>,
}

impl FrameClock {
    fn reanchor(&mut self, now: Instant, period: Duration) {
        if self.period != Some(period) {
            log::info!("Frame clock period: {:?} -> {period:?}", self.period);
        }

        self.period = Some(period);
        self.anchor = now - period / 2;
    }

    /// Snaps the given time to the frame grid, rounding up to the next
    /// point of the grid.
    ///
    /// The event loop wakes up slightly after a `WaitUntil` deadline, so
    /// rounding down could stamp the frame before the redraw was requested,
    /// scheduling the next redraw (e.g. a cursor blink) in the past and
    /// making the loop draw redundant frames.
    fn quantize(&self, now: Instant) -> Instant {
        let Some(period) = self.period else {
            return now;
        };

        let elapsed = now.saturating_duration_since(self.anchor);
        let steps = elapsed.as_nanos().div_ceil(period.as_nanos());
        let snapped = Duration::from_nanos_u128(steps * period.as_nanos());

        self.anchor + snapped
    }
}

impl<P, C> Window<P, C>
where
    P: Program,
    C: Compositor<Renderer = P::Renderer>,
    P::Theme: theme::Base,
{
    pub fn tick(&mut self, now: Instant) -> Instant {
        let mut next = self.frame_clock.quantize(now);

        if let Some(period) = self.frame_clock.period
            && let Some(last) = self.last_frame
        {
            let frame_time = next.saturating_duration_since(last);

            if frame_time > period && self.frame_burst > 1 {
                log::debug!("Frame dropped! (target={period:?}, took={frame_time:?})");

                self.frame_clock.reanchor(now, period);
                next = self.frame_clock.quantize(now);
            }
        }

        self.last_frame = Some(next);
        next
    }

    pub fn reanchor(&mut self, now: Instant) {
        let period = self
            .raw
            .current_monitor()
            .and_then(|monitor| monitor.refresh_rate_millihertz())
            .map(|millihertz| {
                Duration::from_nanos(1_000_000_000_000 / u64::from(millihertz.max(1)))
            });

        if let Some(period) = period {
            self.frame_clock.reanchor(now, period);
        }
    }

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

    pub fn configure_surface(&mut self, compositor: &mut C) {
        let physical_size = self.state.physical_size();

        if physical_size.width == 0 || physical_size.height == 0 {
            return;
        }

        compositor.configure_surface(&mut self.surface, physical_size.width, physical_size.height);

        self.renderer.hint(self.state.scale());
        self.surface_version = self.state.surface_version();
    }

    pub fn request_redraw(&mut self, redraw_request: RedrawRequest) {
        match redraw_request {
            RedrawRequest::NextFrame => {
                self.raw.request_redraw();
                self.redraw_at = None;
                self.frame_burst += 1;
            }
            RedrawRequest::At(at) => {
                self.redraw_at = Some(at);
                self.frame_burst = 0;
            }
            RedrawRequest::Wait => {
                self.frame_burst = 0;
            }
        }
    }

    pub fn request_input_method(&mut self, input_method: InputMethod) {
        match input_method {
            InputMethod::Disabled => {
                self.disable_ime();
            }
            InputMethod::Enabled {
                cursor,
                purpose,
                preedit,
            } => {
                self.enable_ime(cursor, purpose);

                if let Some(preedit) = preedit {
                    if preedit.content.is_empty() {
                        self.preedit = None;
                    } else {
                        let mut overlay = self.preedit.take().unwrap_or_else(Preedit::new);

                        overlay.update(
                            cursor,
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
            if let Some(icon) = conversion::mouse_interaction(interaction) {
                self.raw.set_cursor(icon);

                if self.mouse_interaction == mouse::Interaction::Hidden {
                    self.raw.set_cursor_visible(true);
                }
            } else {
                self.raw.set_cursor_visible(false);
            }

            self.mouse_interaction = interaction;
        }
    }

    pub fn draw_preedit(&mut self) {
        if let Some(preedit) = &self.preedit {
            preedit.draw(
                &mut self.renderer,
                self.state.text_color(),
                self.state.background_color(),
                &Rectangle::new(Point::ORIGIN, self.state.viewport().logical_size()),
            );
        }
    }

    fn enable_ime(&mut self, cursor: Rectangle, purpose: input_method::Purpose) {
        if self.ime_state.is_none() {
            self.raw.set_ime_allowed(true);
        }

        if self.ime_state != Some((cursor, purpose)) {
            self.raw.set_ime_cursor_area(
                LogicalPosition::new(cursor.x, cursor.y),
                LogicalSize::new(cursor.width, cursor.height),
            );
            self.raw.set_ime_purpose(conversion::ime_purpose(purpose));

            self.ime_state = Some((cursor, purpose));
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
    spans: Vec<text::Span<'static, ()>>,
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
        cursor: Rectangle,
        preedit: &input_method::Preedit,
        background: Color,
        renderer: &Renderer,
    ) {
        self.position = cursor.position() + Vector::new(0.0, cursor.height);

        let background = Color {
            a: 1.0,
            ..background
        };

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
                bounds: Size::INFINITE,
                size: preedit.text_size.unwrap_or_else(|| renderer.text_size()),
                line_height: renderer.line_height(),
                font: renderer.font(),
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: renderer.hint_factor(),
            });

            self.spans.clear();
            self.spans
                .extend(spans.into_iter().map(text::Span::to_static));
        }
    }

    fn draw(&self, renderer: &mut Renderer, color: Color, background: Color, viewport: &Rectangle) {
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
            let background = Color {
                a: 1.0,
                ..background
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    ..Default::default()
                },
                background,
            );

            renderer.fill_paragraph(&self.content, bounds.position(), color, bounds);

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
                        bounds: span_bounds + (bounds.position() - Point::ORIGIN),
                        ..Default::default()
                    },
                    color,
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::FrameClock;
    use crate::core::time::{Duration, Instant};

    /// The period of a 60 Hz frame clock, as derived by `Window::reanchor`
    /// from a monitor with a 60 000 mHz refresh rate.
    const PERIOD: Duration = Duration::from_nanos(1_000_000_000_000 / 60_000);

    /// The interval of the text editor's blinking cursor
    /// (`core::text::editor::Focus::CURSOR_BLINK_INTERVAL_MILLIS`).
    const BLINK: u128 = 500;

    fn clock(anchor: Instant) -> FrameClock {
        FrameClock {
            anchor,
            period: Some(PERIOD),
        }
    }

    /// The next blink boundary after the given stamped time, as computed by
    /// `core::text::editor::Focus` when handling a `RedrawRequested` event.
    fn next_blink(updated_at: Instant, stamped_now: Instant) -> Instant {
        let elapsed = stamped_now.saturating_duration_since(updated_at);
        let millis_until = BLINK - (elapsed.as_millis() % BLINK);

        stamped_now + Duration::from_millis(millis_until as u64)
    }

    /// Simulates the event loop settling around a scheduled redraw (a cursor
    /// blink): the loop waits until the deadline, wakes up `wakeup` late,
    /// draws a frame (costing `frame_cost`), stamps it with the frame clock,
    /// and schedules the next redraw from the stamp. A deadline in the past
    /// is processed immediately, as the real event loop does: a
    /// `ControlFlow::WaitUntil` of a past deadline wakes up without waiting,
    /// and the loop draws again until the next deadline is in the future.
    ///
    /// Returns the stamps of the frames drawn while settling.
    fn settle(
        clock: &FrameClock,
        updated_at: Instant,
        deadline: Instant,
        wakeup: Duration,
        frame_cost: Duration,
    ) -> Vec<Instant> {
        let mut stamps = Vec::new();
        let mut now = deadline + wakeup;

        loop {
            let stamp = clock.quantize(now);
            stamps.push(stamp);

            let next = next_blink(updated_at, stamp);
            now += frame_cost;

            if next > now {
                // The event loop can wait until the next deadline
                break;
            }

            // Past deadline: the event loop wakes up immediately
            now += wakeup;
        }

        stamps
    }

    #[test]
    fn quantize_is_identity_without_a_period() {
        let now = Instant::now();
        let clock = FrameClock {
            anchor: now,
            period: None,
        };
        let later = now + Duration::from_millis(10);

        assert_eq!(clock.quantize(later), later);
    }

    #[test]
    fn quantize_keeps_a_point_on_the_grid() {
        let anchor = Instant::now();
        let clock = clock(anchor);

        for step in [0, 1, 42] {
            let now = anchor + PERIOD * step;

            assert_eq!(clock.quantize(now), now);
        }
    }

    #[test]
    fn quantize_rounds_up_to_the_next_point_of_the_grid() {
        let anchor = Instant::now();
        let clock = clock(anchor);

        // A moment after a grid point snaps to the next point
        let just_after = anchor + PERIOD + Duration::from_nanos(1);
        assert_eq!(clock.quantize(just_after), anchor + PERIOD * 2);

        // A moment before a grid point snaps to that point
        let just_before = anchor + PERIOD * 4 - Duration::from_nanos(1);
        assert_eq!(clock.quantize(just_before), anchor + PERIOD * 4);
    }

    /// A text input's cursor blink schedules a redraw exactly on a blink
    /// boundary, and the event loop wakes up a little after that deadline.
    ///
    /// The frame drawn there must be stamped with a time not before the
    /// boundary, so that:
    ///
    /// - the cursor is drawn with the new phase, and
    /// - the next blink, scheduled from the stamp, is in the future, so the
    ///   event loop can keep waiting instead of drawing redundant frames
    ///   until the wall clock passes the boundary (the window "flashing"
    ///   once per blink).
    #[test]
    fn blink_redraw_is_not_stamped_or_scheduled_in_the_past() {
        let updated_at = Instant::now() + Duration::from_secs(10);
        let boundary = updated_at + Duration::from_millis(BLINK as u64 * 2);

        // Place a grid point 10 ms before the boundary, so the frame drawn
        // at the boundary (plus wakeup latency) falls between grid points
        let anchor = boundary - Duration::from_millis(10) - PERIOD;
        let clock = clock(anchor);

        let wake = boundary + Duration::from_millis(1);
        let stamp = clock.quantize(wake);

        assert!(
            stamp >= boundary,
            "the frame is stamped before the blink boundary"
        );
        assert_eq!(stamp, boundary - Duration::from_millis(10) + PERIOD);

        let next = next_blink(updated_at, stamp);
        assert!(
            next > wake,
            "the next redraw ({next:?}) is scheduled in the past (wake: {wake:?})"
        );

        // With the previous round-down, the frame was stamped 10 ms before
        // the boundary and the next blink was scheduled right at it, i.e.
        // before the event loop even woke up
        let elapsed = (wake - anchor).as_nanos();
        let steps = elapsed / PERIOD.as_nanos();
        let floor_stamp = anchor + Duration::from_nanos_u128(steps * PERIOD.as_nanos());

        assert!(floor_stamp < boundary);
        assert!(next_blink(updated_at, floor_stamp) <= wake);
    }

    /// Settle the event loop around a cursor blink: with a round-up stamp
    /// the loop draws a single frame and can wait again; a round-down stamp
    /// schedules the next redraw in the past, so the loop settles only after
    /// several redundant frames.
    #[test]
    fn blink_settles_in_a_single_frame() {
        let updated_at = Instant::now() + Duration::from_secs(10);
        let boundary = updated_at + Duration::from_millis(BLINK as u64 * 2);

        let anchor = boundary - Duration::from_millis(10) - PERIOD;
        let clock = clock(anchor);

        let wakeup = Duration::from_millis(1);
        let frame_cost = Duration::from_millis(4);

        let stamps = settle(&clock, updated_at, boundary, wakeup, frame_cost);

        assert_eq!(stamps.len(), 1, "a blink should settle in a single frame");
        assert!(
            stamps[0] >= boundary,
            "the frame is stamped before the blink boundary"
        );
    }
}
