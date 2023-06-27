//! Show a linear progress indicator.
use iced::advanced::layout;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::event;
use iced::mouse;
use iced::time::Instant;
use iced::window::{self, RedrawRequest};
use iced::{Background, Color, Element, Event, Length, Rectangle, Size};

use super::easing::{self, Easing};

use std::time::Duration;

#[allow(missing_debug_implementations)]
pub struct Linear<'a, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    width: Length,
    height: Length,
    style: <Renderer::Theme as StyleSheet>::Style,
    easing: &'a Easing,
    cycle_duration: Duration,
}

impl<'a, Renderer> Linear<'a, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Linear`] with the given content.
    pub fn new() -> Self {
        Linear {
            width: Length::Fixed(100.0),
            height: Length::Fixed(4.0),
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            easing: &easing::STANDARD,
            cycle_duration: Duration::from_millis(600),
        }
    }

    /// Sets the width of the [`Linear`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Linear`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the style variant of this [`Linear`].
    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }

    /// Sets the motion easing of this [`Linear`].
    pub fn easing(mut self, easing: &'a Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the cycle duration of this [`Linear`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration / 2;
        self
    }
}

impl<'a, Renderer> Default for Linear<'a, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
enum State {
    Expanding { start: Instant, progress: f32 },
    Contracting { start: Instant, progress: f32 },
}

impl Default for State {
    fn default() -> Self {
        Self::Expanding {
            start: Instant::now(),
            progress: 0.0,
        }
    }
}

impl State {
    fn next(&self, now: Instant) -> Self {
        match self {
            Self::Expanding { .. } => Self::Contracting {
                start: now,
                progress: 0.0,
            },
            Self::Contracting { .. } => Self::Expanding {
                start: now,
                progress: 0.0,
            },
        }
    }

    fn start(&self) -> Instant {
        match self {
            Self::Expanding { start, .. } | Self::Contracting { start, .. } => {
                *start
            }
        }
    }

    fn timed_transition(&self, cycle_duration: Duration, now: Instant) -> Self {
        let elapsed = now.duration_since(self.start());

        match elapsed {
            elapsed if elapsed > cycle_duration => self.next(now),
            _ => self.with_elapsed(cycle_duration, elapsed),
        }
    }

    fn with_elapsed(
        &self,
        cycle_duration: Duration,
        elapsed: Duration,
    ) -> Self {
        let progress = elapsed.as_secs_f32() / cycle_duration.as_secs_f32();
        match self {
            Self::Expanding { start, .. } => Self::Expanding {
                start: *start,
                progress,
            },
            Self::Contracting { start, .. } => Self::Contracting {
                start: *start,
                progress,
            },
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Linear<'a, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        const FRAME_RATE: u64 = 60;

        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            *state = state.timed_transition(self.cycle_duration, now);

            shell.request_redraw(RedrawRequest::At(
                now + Duration::from_millis(1000 / FRAME_RATE),
            ));
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let custom_style = theme.appearance(&self.style);
        let state = tree.state.downcast_ref::<State>();

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            Background::Color(custom_style.track_color),
        );

        match state {
            State::Expanding { progress, .. } => renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bounds.y,
                        width: self.easing.y_at_x(*progress) * bounds.width,
                        height: bounds.height,
                    },
                    border_radius: 0.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color(custom_style.bar_color),
            ),

            State::Contracting { progress, .. } => renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: bounds.x
                            + self.easing.y_at_x(*progress) * bounds.width,
                        y: bounds.y,
                        width: (1.0 - self.easing.y_at_x(*progress))
                            * bounds.width,
                        height: bounds.height,
                    },
                    border_radius: 0.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color(custom_style.bar_color),
            ),
        }
    }
}

impl<'a, Message, Renderer> From<Linear<'a, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn from(linear: Linear<'a, Renderer>) -> Self {
        Self::new(linear)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The track [`Color`] of the progress indicator.
    pub track_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            track_color: Color::TRANSPARENT,
            bar_color: Color::BLACK,
        }
    }
}

/// A set of rules that dictate the style of an indicator.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a indicator.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}

impl StyleSheet for iced::Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        let palette = self.extended_palette();

        Appearance {
            track_color: palette.background.weak.color,
            bar_color: palette.primary.base.color,
        }
    }
}
