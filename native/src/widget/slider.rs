//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::touch;
use crate::{
    Background, Clipboard, Color, Element, Hasher, Layout, Length, Point,
    Rectangle, Shell, Size, Widget,
};

use std::hash::Hash;
use std::ops::RangeInclusive;

pub use iced_style::slider::{Handle, HandleShape, Style, StyleSheet};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
///
/// The [`Slider`] range of numeric values is generic and its step size defaults
/// to 1 unit.
///
/// # Example
/// ```
/// # use iced_native::widget::slider::{self, Slider};
/// #
/// #[derive(Clone)]
/// pub enum Message {
///     SliderChanged(f32),
/// }
///
/// let state = &mut slider::State::new();
/// let value = 50.0;
///
/// Slider::new(state, 0.0..=100.0, value, Message::SliderChanged);
/// ```
///
/// ![Slider drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/slider.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Slider<'a, T, Message> {
    state: &'a mut State,
    range: RangeInclusive<T>,
    step: T,
    value: T,
    on_change: Box<dyn Fn(T) -> Message>,
    on_release: Option<Message>,
    width: Length,
    height: u16,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
{
    /// The default height of a [`Slider`].
    pub const DEFAULT_HEIGHT: u16 = 22;

    /// Creates a new [`Slider`].
    ///
    /// It expects:
    ///   * the local [`State`] of the [`Slider`]
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Slider`]
    ///   * a function that will be called when the [`Slider`] is dragged.
    ///   It receives the new value of the [`Slider`] and must produce a
    ///   `Message`.
    pub fn new<F>(
        state: &'a mut State,
        range: RangeInclusive<T>,
        value: T,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(T) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        Slider {
            state,
            value,
            range,
            step: T::from(1),
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            height: Self::DEFAULT_HEIGHT,
            style_sheet: Default::default(),
        }
    }

    /// Sets the release message of the [`Slider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default on_change message could create too many events.
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`Slider`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Slider`].
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Sets the style of the [`Slider`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }

    /// Sets the step size of the [`Slider`].
    pub fn step(mut self, step: T) -> Self {
        self.step = step;
        self
    }
}

/// The local state of a [`Slider`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_dragging: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for Slider<'a, T, Message>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
    Renderer: crate::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits =
            limits.width(self.width).height(Length::Units(self.height));

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let is_dragging = self.state.is_dragging;

        let mut change = || {
            let bounds = layout.bounds();
            let new_value = if cursor_position.x <= bounds.x {
                *self.range.start()
            } else if cursor_position.x >= bounds.x + bounds.width {
                *self.range.end()
            } else {
                let step = self.step.into();
                let start = (*self.range.start()).into();
                let end = (*self.range.end()).into();

                let percent = f64::from(cursor_position.x - bounds.x)
                    / f64::from(bounds.width);

                let steps = (percent * (end - start) / step).round();
                let value = steps * step + start;

                if let Some(value) = T::from_f64(value) {
                    value
                } else {
                    return;
                }
            };

            if (self.value.into() - new_value.into()).abs() > f64::EPSILON {
                shell.publish((self.on_change)(new_value));

                self.value = new_value;
            }
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    change();
                    self.state.is_dragging = true;

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if is_dragging {
                    if let Some(on_release) = self.on_release.clone() {
                        shell.publish(on_release);
                    }
                    self.state.is_dragging = false;

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if is_dragging {
                    change();

                    return event::Status::Captured;
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let style = if self.state.is_dragging {
            self.style_sheet.dragging()
        } else if is_mouse_over {
            self.style_sheet.hovered()
        } else {
            self.style_sheet.active()
        };

        let rail_y = bounds.y + (bounds.height / 2.0).round();

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y,
                    width: bounds.width,
                    height: 2.0,
                },
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            style.rail_colors.0,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y + 2.0,
                    width: bounds.width,
                    height: 2.0,
                },
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
            Background::Color(style.rail_colors.1),
        );

        let (handle_width, handle_height, handle_border_radius) = match style
            .handle
            .shape
        {
            HandleShape::Circle { radius } => {
                (radius * 2.0, radius * 2.0, radius)
            }
            HandleShape::Rectangle {
                width,
                border_radius,
            } => (f32::from(width), f32::from(bounds.height), border_radius),
        };

        let value = self.value.into() as f32;
        let (range_start, range_end) = {
            let (start, end) = self.range.clone().into_inner();

            (start.into() as f32, end.into() as f32)
        };

        let handle_offset = if range_start >= range_end {
            0.0
        } else {
            (bounds.width - handle_width) * (value - range_start)
                / (range_end - range_start)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + handle_offset.round(),
                    y: rail_y - handle_height / 2.0,
                    width: handle_width,
                    height: handle_height,
                },
                border_radius: handle_border_radius,
                border_width: style.handle.border_width,
                border_color: style.handle.border_color,
            },
            style.handle.color,
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        if self.state.is_dragging {
            mouse::Interaction::Grabbing
        } else if is_mouse_over {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
    }
}

impl<'a, T, Message, Renderer> From<Slider<'a, T, Message>>
    for Element<'a, Message, Renderer>
where
    T: 'a + Copy + Into<f64> + num_traits::FromPrimitive,
    Message: 'a + Clone,
    Renderer: 'a + crate::Renderer,
{
    fn from(slider: Slider<'a, T, Message>) -> Element<'a, Message, Renderer> {
        Element::new(slider)
    }
}
