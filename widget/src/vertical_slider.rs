//! Display an interactive selector of a single value from a range of values.
use std::ops::RangeInclusive;

pub use crate::slider::{
    default, Catalog, Handle, HandleShape, Status, Style, StyleFn,
};

use crate::core::border::{self, Border};
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::keyboard::key::{self, Key};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Clipboard, Element, Length, Pixels, Point, Rectangle, Shell, Size,
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
/// # type VerticalSlider<'a, T, Message> = iced_widget::VerticalSlider<'a, T, Message>;
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
pub struct VerticalSlider<'a, T, Message, Theme = crate::Theme>
where
    Theme: Catalog,
{
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    value: T,
    default: Option<T>,
    on_change: Box<dyn Fn(T) -> Message + 'a>,
    on_release: Option<Message>,
    width: f32,
    height: Length,
    class: Theme::Class<'a>,
}

impl<'a, T, Message, Theme> VerticalSlider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Theme: Catalog,
{
    /// The default width of a [`VerticalSlider`].
    pub const DEFAULT_WIDTH: f32 = 16.0;

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
            default: None,
            range,
            step: T::from(1),
            shift_step: None,
            on_change: Box::new(on_change),
            on_release: None,
            width: Self::DEFAULT_WIDTH,
            height: Length::Fill,
            class: Theme::default(),
        }
    }

    /// Sets the optional default value for the [`VerticalSlider`].
    ///
    /// If set, the [`VerticalSlider`] will reset to this value when ctrl-clicked or command-clicked.
    pub fn default(mut self, default: impl Into<T>) -> Self {
        self.default = Some(default.into());
        self
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

    /// Sets the step size of the [`VerticalSlider`].
    pub fn step(mut self, step: T) -> Self {
        self.step = step;
        self
    }

    /// Sets the optional "shift" step for the [`VerticalSlider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    /// Sets the style of the [`VerticalSlider`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`VerticalSlider`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VerticalSlider<'a, T, Message, Theme>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
    Theme: Catalog,
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
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
        let state = tree.state.downcast_mut::<State>();
        let is_dragging = state.is_dragging;
        let current_value = self.value;

        let locate = |cursor_position: Point| -> Option<T> {
            let bounds = layout.bounds();

            let new_value = if cursor_position.y >= bounds.y + bounds.height {
                Some(*self.range.start())
            } else if cursor_position.y <= bounds.y {
                Some(*self.range.end())
            } else {
                let step = if state.keyboard_modifiers.shift() {
                    self.shift_step.unwrap_or(self.step)
                } else {
                    self.step
                }
                .into();

                let start = (*self.range.start()).into();
                let end = (*self.range.end()).into();

                let percent = 1.0
                    - f64::from(cursor_position.y - bounds.y)
                        / f64::from(bounds.height);

                let steps = (percent * (end - start) / step).round();
                let value = steps * step + start;

                T::from_f64(value.min(end))
            };

            new_value
        };

        let increment = |value: T| -> Option<T> {
            let step = if state.keyboard_modifiers.shift() {
                self.shift_step.unwrap_or(self.step)
            } else {
                self.step
            }
            .into();

            let steps = (value.into() / step).round();
            let new_value = step * (steps + 1.0);

            if new_value > (*self.range.end()).into() {
                return Some(*self.range.end());
            }

            T::from_f64(new_value)
        };

        let decrement = |value: T| -> Option<T> {
            let step = if state.keyboard_modifiers.shift() {
                self.shift_step.unwrap_or(self.step)
            } else {
                self.step
            }
            .into();

            let steps = (value.into() / step).round();
            let new_value = step * (steps - 1.0);

            if new_value < (*self.range.start()).into() {
                return Some(*self.range.start());
            }

            T::from_f64(new_value)
        };

        let change = |new_value: T| {
            if (self.value.into() - new_value.into()).abs() > f64::EPSILON {
                shell.publish((self.on_change)(new_value));

                self.value = new_value;
            }
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(cursor_position) =
                    cursor.position_over(layout.bounds())
                {
                    if state.keyboard_modifiers.control()
                        || state.keyboard_modifiers.command()
                    {
                        let _ = self.default.map(change);
                        state.is_dragging = false;
                    } else {
                        let _ = locate(cursor_position).map(change);
                        state.is_dragging = true;
                    }

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
                    state.is_dragging = false;

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if is_dragging {
                    let _ = cursor.position().and_then(locate).map(change);

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if cursor.position_over(layout.bounds()).is_some() {
                    match key {
                        Key::Named(key::Named::ArrowUp) => {
                            let _ = increment(current_value).map(change);
                        }
                        Key::Named(key::Named::ArrowDown) => {
                            let _ = decrement(current_value).map(change);
                        }
                        _ => (),
                    }

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.keyboard_modifiers = modifiers;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        let style = theme.style(
            &self.class,
            if state.is_dragging {
                Status::Dragged
            } else if is_mouse_over {
                Status::Hovered
            } else {
                Status::Active
            },
        );

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

        let value = self.value.into() as f32;
        let (range_start, range_end) = {
            let (start, end) = self.range.clone().into_inner();

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
                border: border::rounded(style.rail.border_radius),
                ..renderer::Quad::default()
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
                border: border::rounded(style.rail.border_radius),
                ..renderer::Quad::default()
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
                border: Border {
                    radius: handle_border_radius,
                    width: style.handle.border_width,
                    color: style.handle.border_color,
                },
                ..renderer::Quad::default()
            },
            style.handle.color,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
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
}

impl<'a, T, Message, Theme, Renderer>
    From<VerticalSlider<'a, T, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive + 'a,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(
        slider: VerticalSlider<'a, T, Message, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(slider)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_dragging: bool,
    keyboard_modifiers: keyboard::Modifiers,
}
