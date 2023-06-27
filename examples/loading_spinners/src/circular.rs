//! Show a circular progress indicator.
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Renderer, Shell, Widget};
use iced::event;
use iced::mouse;
use iced::time::Instant;
use iced::widget::canvas;
use iced::window::{self, RedrawRequest};
use iced::{
    Background, Color, Element, Event, Length, Rectangle, Size, Vector,
};

use super::easing::{self, Easing};

use std::f32::consts::PI;
use std::time::Duration;

const MIN_RADIANS: f32 = PI / 8.0;
const WRAP_RADIANS: f32 = 2.0 * PI - PI / 4.0;
const BASE_ROTATION_SPEED: u32 = u32::MAX / 80;

#[allow(missing_debug_implementations)]
pub struct Circular<'a, Theme>
where
    Theme: StyleSheet,
{
    size: f32,
    bar_height: f32,
    style: <Theme as StyleSheet>::Style,
    easing: &'a Easing,
    cycle_duration: Duration,
    rotation_duration: Duration,
}

impl<'a, Theme> Circular<'a, Theme>
where
    Theme: StyleSheet,
{
    /// Creates a new [`Circular`] with the given content.
    pub fn new() -> Self {
        Circular {
            size: 40.0,
            bar_height: 4.0,
            style: <Theme as StyleSheet>::Style::default(),
            easing: &easing::STANDARD,
            cycle_duration: Duration::from_millis(600),
            rotation_duration: Duration::from_secs(2),
        }
    }

    /// Sets the size of the [`Circular`].
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the bar height of the [`Circular`].
    pub fn bar_height(mut self, bar_height: f32) -> Self {
        self.bar_height = bar_height;
        self
    }

    /// Sets the style variant of this [`Circular`].
    pub fn style(mut self, style: <Theme as StyleSheet>::Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the easing of this [`Circular`].
    pub fn easing(mut self, easing: &'a Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the cycle duration of this [`Circular`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration / 2;
        self
    }

    /// Sets the base rotation duration of this [`Circular`]. This is the duration that a full
    /// rotation would take if the cycle rotation were set to 0.0 (no expanding or contracting)
    pub fn rotation_duration(mut self, duration: Duration) -> Self {
        self.rotation_duration = duration;
        self
    }
}

impl<'a, Theme> Default for Circular<'a, Theme>
where
    Theme: StyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
enum Animation {
    Expanding {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
    Contracting {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
}

impl Default for Animation {
    fn default() -> Self {
        Self::Expanding {
            start: Instant::now(),
            progress: 0.0,
            rotation: 0,
            last: Instant::now(),
        }
    }
}

impl Animation {
    fn next(&self, additional_rotation: u32, now: Instant) -> Self {
        match self {
            Self::Expanding { rotation, .. } => Self::Contracting {
                start: now,
                progress: 0.0,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting { rotation, .. } => Self::Expanding {
                start: now,
                progress: 0.0,
                rotation: rotation.wrapping_add(
                    BASE_ROTATION_SPEED.wrapping_add(
                        ((WRAP_RADIANS / (2.0 * PI)) * u32::MAX as f32) as u32,
                    ),
                ),
                last: now,
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

    fn last(&self) -> Instant {
        match self {
            Self::Expanding { last, .. } | Self::Contracting { last, .. } => {
                *last
            }
        }
    }

    fn timed_transition(
        &self,
        cycle_duration: Duration,
        rotation_duration: Duration,
        now: Instant,
    ) -> Self {
        let elapsed = now.duration_since(self.start());
        let additional_rotation = ((now - self.last()).as_secs_f32()
            / rotation_duration.as_secs_f32()
            * (u32::MAX) as f32) as u32;

        match elapsed {
            elapsed if elapsed > cycle_duration => {
                self.next(additional_rotation, now)
            }
            _ => self.with_elapsed(
                cycle_duration,
                additional_rotation,
                elapsed,
                now,
            ),
        }
    }

    fn with_elapsed(
        &self,
        cycle_duration: Duration,
        additional_rotation: u32,
        elapsed: Duration,
        now: Instant,
    ) -> Self {
        let progress = elapsed.as_secs_f32() / cycle_duration.as_secs_f32();
        match self {
            Self::Expanding {
                start, rotation, ..
            } => Self::Expanding {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting {
                start, rotation, ..
            } => Self::Contracting {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
        }
    }

    fn rotation(&self) -> f32 {
        match self {
            Self::Expanding { rotation, .. }
            | Self::Contracting { rotation, .. } => {
                *rotation as f32 / u32::MAX as f32
            }
        }
    }
}

#[derive(Default)]
struct State {
    animation: Animation,
    cache: canvas::Cache,
}

impl<'a, Message, Theme> Widget<Message, iced::Renderer<Theme>>
    for Circular<'a, Theme>
where
    Message: 'a + Clone,
    Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn width(&self) -> Length {
        Length::Fixed(self.size)
    }

    fn height(&self) -> Length {
        Length::Fixed(self.size)
    }

    fn layout(
        &self,
        _renderer: &iced::Renderer<Theme>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.size).height(self.size);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer<Theme>,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        const FRAME_RATE: u64 = 60;

        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.animation = state.animation.timed_transition(
                self.cycle_duration,
                self.rotation_duration,
                now,
            );

            state.cache.clear();
            shell.request_redraw(RedrawRequest::At(
                now + Duration::from_millis(1000 / FRAME_RATE),
            ));
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer<Theme>,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let custom_style =
            <Theme as StyleSheet>::appearance(theme, &self.style);

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - self.bar_height;
            let track_path = canvas::Path::circle(frame.center(), track_radius);

            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(custom_style.track_color)
                    .with_width(self.bar_height),
            );

            let mut builder = canvas::path::Builder::new();

            let start = state.animation.rotation() * 2.0 * PI;

            match state.animation {
                Animation::Expanding { progress, .. } => {
                    builder.arc(canvas::path::Arc {
                        center: frame.center(),
                        radius: track_radius,
                        start_angle: start,
                        end_angle: start
                            + MIN_RADIANS
                            + WRAP_RADIANS * (self.easing.y_at_x(progress)),
                    });
                }
                Animation::Contracting { progress, .. } => {
                    builder.arc(canvas::path::Arc {
                        center: frame.center(),
                        radius: track_radius,
                        start_angle: start
                            + WRAP_RADIANS * (self.easing.y_at_x(progress)),
                        end_angle: start + MIN_RADIANS + WRAP_RADIANS,
                    });
                }
            }

            let bar_path = builder.build();

            frame.stroke(
                &bar_path,
                canvas::Stroke::default()
                    .with_color(custom_style.bar_color)
                    .with_width(self.bar_height),
            );
        });

        renderer.with_translation(
            Vector::new(bounds.x, bounds.y),
            |renderer| {
                renderer.draw_primitive(geometry.0);
            },
        );
    }
}

impl<'a, Message, Theme> From<Circular<'a, Theme>>
    for Element<'a, Message, iced::Renderer<Theme>>
where
    Message: Clone + 'a,
    Theme: StyleSheet + 'a,
{
    fn from(circular: Circular<'a, Theme>) -> Self {
        Self::new(circular)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the progress indicator.
    pub background: Option<Background>,
    /// The track [`Color`] of the progress indicator.
    pub track_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            background: None,
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
            background: None,
            track_color: palette.background.weak.color,
            bar_color: palette.primary.base.color,
        }
    }
}
