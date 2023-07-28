//! Display an interactive selector of a single value from a range of values.
//!
//! A [`VerticalSlider`] has some local [`State`].
use std::ops::RangeInclusive;

pub use crate::style::slider::{Appearance, Handle, HandleShape, StyleSheet};

use crate::core;
use crate::core::event::{self, Event};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Clipboard, Color, Element, Length, Pixels, Point, Rectangle, Shell, Size,
    Widget,
};

/// An vertical bar and a handle that selects a single value from a range of
/// values.
///
/// A [`VerticalSlider`] will try to fill the vertical space of its container.
///
/// The [`VerticalSlider`] range of numeric values is generic and its step size defaults
/// to 1 unit.
///
/// # Example
/// ```no_run
/// # type VerticalSlider<'a, T, Message> =
/// #     iced_widget::VerticalSlider<'a, T, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
/// #
/// #[derive(Clone)]
/// pub enum Message {
///     SliderChanged(f32),
/// }
///
/// let value = 50.0;
///
/// VerticalSlider::new(0.0..=100.0, value, Message::SliderChanged);
/// ```
#[allow(missing_debug_implementations)]
pub struct VerticalSlider<'a, T, Message, Renderer = crate::Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: StyleSheet,
{
    range: RangeInclusive<T>,
    step: T,
    value: T,
    on_change: Box<dyn Fn(T) -> Message + 'a>,
    on_release: Option<Message>,
    width: f32,
    height: Length,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, T, Message, Renderer> VerticalSlider<'a, T, Message, Renderer>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Renderer: core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// The default width of a [`VerticalSlider`].
    pub const DEFAULT_WIDTH: f32 = 22.0;

    /// Creates a new [`VerticalSlider`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`VerticalSlider`]
    ///   * a function that will be called when the [`VerticalSlider`] is dragged.
    ///   It receives the new value of the [`VerticalSlider`] and must produce a
    ///   `Message`.
    pub fn new<F>(range: RangeInclusive<T>, value: T, on_change: F) -> Self
    where
        F: 'a + Fn(T) -> Message,
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

        VerticalSlider {
            value,
            range,
            step: T::from(1),
            on_change: Box::new(on_change),
            on_release: None,
            width: Self::DEFAULT_WIDTH,
            height: Length::Fill,
            style: Default::default(),
        }
    }

    /// Sets the release message of the [`VerticalSlider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default on_change message could create too many events.
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`VerticalSlider`].
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into().0;
        self
    }

    /// Sets the height of the [`VerticalSlider`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the style of the [`VerticalSlider`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the step size of the [`VerticalSlider`].
    pub fn step(mut self, step: T) -> Self {
        self.step = step;
        self
    }
}

impl<'a, T, Message, Renderer> Widget<Message, Renderer>
    for VerticalSlider<'a, T, Message, Renderer>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
    Renderer: core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn width(&self) -> Length {
        Length::Shrink
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
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        update(
            event,
            layout,
            cursor,
            shell,
            tree.state.downcast_mut::<State>(),
            &mut self.value,
            &self.range,
            self.step,
            self.on_change.as_ref(),
            &self.on_release,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            layout,
            cursor,
            tree.state.downcast_ref::<State>(),
            self.value,
            &self.range,
            theme,
            &self.style,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor, tree.state.downcast_ref::<State>())
    }
}

impl<'a, T, Message, Renderer> From<VerticalSlider<'a, T, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    T: 'a + Copy + Into<f64> + num_traits::FromPrimitive,
    Message: 'a + Clone,
    Renderer: 'a + core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        slider: VerticalSlider<'a, T, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(slider)
    }
}

/// Processes an [`Event`] and updates the [`State`] of a [`VerticalSlider`]
/// accordingly.
pub fn update<Message, T>(
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    state: &mut State,
    value: &mut T,
    range: &RangeInclusive<T>,
    step: T,
    on_change: &dyn Fn(T) -> Message,
    on_release: &Option<Message>,
) -> event::Status
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
{
    let is_dragging = state.is_dragging;

    let mut change = |cursor_position: Point| {
        let bounds = layout.bounds();

        let new_value = if cursor_position.y >= bounds.y + bounds.height {
            *range.start()
        } else if cursor_position.y <= bounds.y {
            *range.end()
        } else {
            let step = step.into();
            let start = (*range.start()).into();
            let end = (*range.end()).into();

            let percent = 1.0
                - f64::from(cursor_position.y - bounds.y)
                    / f64::from(bounds.height);

            let steps = (percent * (end - start) / step).round();
            let value = steps * step + start;

            if let Some(value) = T::from_f64(value) {
                value
            } else {
                return;
            }
        };

        if ((*value).into() - new_value.into()).abs() > f64::EPSILON {
            shell.publish((on_change)(new_value));

            *value = new_value;
        }
    };

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            if let Some(cursor_position) = cursor.position_over(layout.bounds())
            {
                change(cursor_position);
                state.is_dragging = true;

                return event::Status::Captured;
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. })
        | Event::Touch(touch::Event::FingerLost { .. }) => {
            if is_dragging {
                if let Some(on_release) = on_release.clone() {
                    shell.publish(on_release);
                }
                state.is_dragging = false;

                return event::Status::Captured;
            }
        }
        Event::Mouse(mouse::Event::CursorMoved { .. })
        | Event::Touch(touch::Event::FingerMoved { .. }) => {
            if is_dragging {
                let _ = cursor.position().map(change);

                return event::Status::Captured;
            }
        }
        _ => {}
    }

    event::Status::Ignored
}

/// Draws a [`VerticalSlider`].
pub fn draw<T, R>(
    renderer: &mut R,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    state: &State,
    value: T,
    range: &RangeInclusive<T>,
    style_sheet: &dyn StyleSheet<Style = <R::Theme as StyleSheet>::Style>,
    style: &<R::Theme as StyleSheet>::Style,
) where
    T: Into<f64> + Copy,
    R: core::Renderer,
    R::Theme: StyleSheet,
{
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    let style = if state.is_dragging {
        style_sheet.dragging(style)
    } else if is_mouse_over {
        style_sheet.hovered(style)
    } else {
        style_sheet.active(style)
    };

    let (handle_width, handle_height, handle_border_radius) =
        match style.handle.shape {
            HandleShape::Circle { radius } => {
                (radius * 2.0, radius * 2.0, radius.into())
            }
            HandleShape::Rectangle {
                width,
                border_radius,
            } => (f32::from(width), bounds.width, border_radius),
        };

    let value = value.into() as f32;
    let (range_start, range_end) = {
        let (start, end) = range.clone().into_inner();

        (start.into() as f32, end.into() as f32)
    };

    let offset = if range_start >= range_end {
        0.0
    } else {
        (bounds.height - handle_width) * (value - range_end)
            / (range_start - range_end)
    };

    let rail_x = bounds.x + bounds.width / 2.0;

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: rail_x - style.rail.width / 2.0,
                y: bounds.y,
                width: style.rail.width,
                height: offset + handle_width / 2.0,
            },
            border_radius: style.rail.border_radius,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
        style.rail.colors.1,
    );

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: rail_x - style.rail.width / 2.0,
                y: bounds.y + offset + handle_width / 2.0,
                width: style.rail.width,
                height: bounds.height - offset - handle_width / 2.0,
            },
            border_radius: style.rail.border_radius,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
        style.rail.colors.0,
    );

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: rail_x - handle_height / 2.0,
                y: bounds.y + offset,
                width: handle_height,
                height: handle_width,
            },
            border_radius: handle_border_radius,
            border_width: style.handle.border_width,
            border_color: style.handle.border_color,
        },
        style.handle.color,
    );
}

/// Computes the current [`mouse::Interaction`] of a [`VerticalSlider`].
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    state: &State,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    if state.is_dragging {
        mouse::Interaction::Grabbing
    } else if is_mouse_over {
        mouse::Interaction::Grab
    } else {
        mouse::Interaction::default()
    }
}

/// The local state of a [`VerticalSlider`].
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
