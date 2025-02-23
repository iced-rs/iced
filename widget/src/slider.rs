//! Sliders let users set a value by moving an indicator.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::slider;
//!
//! struct State {
//!    value: f32,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     ValueChanged(f32),
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     slider(0.0..=100.0, state.value, Message::ValueChanged).into()
//! }
//!
//! fn update(state: &mut State, message: Message) {
//!     match message {
//!         Message::ValueChanged(value) => {
//!             state.value = value;
//!         }
//!     }
//! }
//! ```
use crate::core::border::{self, Border};
use crate::core::keyboard;
use crate::core::keyboard::key::{self, Key};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Background, Clipboard, Color, Element, Event, Layout, Length, Pixels,
    Point, Rectangle, Shell, Size, Theme, Widget,
};

use std::ops::RangeInclusive;

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// A [`Slider`] will try to fill the horizontal space of its container.
///
/// The [`Slider`] range of numeric values is generic and its step size defaults
/// to 1 unit.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::slider;
///
/// struct State {
///    value: f32,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     ValueChanged(f32),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     slider(0.0..=100.0, state.value, Message::ValueChanged).into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::ValueChanged(value) => {
///             state.value = value;
///         }
///     }
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Slider<'a, T, Message, Theme = crate::Theme>
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
    width: Length,
    height: f32,
    class: Theme::Class<'a>,
    status: Option<Status>,
}

impl<'a, T, Message, Theme> Slider<'a, T, Message, Theme>
where
    T: Copy + From<u8> + PartialOrd,
    Message: Clone,
    Theme: Catalog,
{
    /// The default height of a [`Slider`].
    pub const DEFAULT_HEIGHT: f32 = 16.0;

    /// Creates a new [`Slider`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Slider`]
    ///   * a function that will be called when the [`Slider`] is dragged.
    ///     It receives the new value of the [`Slider`] and must produce a
    ///     `Message`.
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

        Slider {
            value,
            default: None,
            range,
            step: T::from(1),
            shift_step: None,
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            height: Self::DEFAULT_HEIGHT,
            class: Theme::default(),
            status: None,
        }
    }

    /// Sets the optional default value for the [`Slider`].
    ///
    /// If set, the [`Slider`] will reset to this value when ctrl-clicked or command-clicked.
    pub fn default(mut self, default: impl Into<T>) -> Self {
        self.default = Some(default.into());
        self
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
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Slider`].
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into().0;
        self
    }

    /// Sets the step size of the [`Slider`].
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the optional "shift" step for the [`Slider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    /// Sets the style of the [`Slider`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Slider`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Slider<'_, T, Message, Theme>
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
            width: self.width,
            height: Length::Shrink,
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

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let mut update = || {
            let current_value = self.value;

            let locate = |cursor_position: Point| -> Option<T> {
                let bounds = layout.bounds();

                let new_value = if cursor_position.x <= bounds.x {
                    Some(*self.range.start())
                } else if cursor_position.x >= bounds.x + bounds.width {
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

                    let percent = f64::from(cursor_position.x - bounds.x)
                        / f64::from(bounds.width);

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

            match &event {
                Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(cursor_position) =
                        cursor.position_over(layout.bounds())
                    {
                        if state.keyboard_modifiers.command() {
                            let _ = self.default.map(change);
                            state.is_dragging = false;
                        } else {
                            let _ = locate(cursor_position).map(change);
                            state.is_dragging = true;
                        }

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    if state.is_dragging {
                        if let Some(on_release) = self.on_release.clone() {
                            shell.publish(on_release);
                        }
                        state.is_dragging = false;

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    if state.is_dragging {
                        let _ = cursor.position().and_then(locate).map(change);

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta })
                    if state.keyboard_modifiers.control() =>
                {
                    if cursor.is_over(layout.bounds()) {
                        let delta = match delta {
                            mouse::ScrollDelta::Lines { x: _, y } => y,
                            mouse::ScrollDelta::Pixels { x: _, y } => y,
                        };

                        if *delta < 0.0 {
                            let _ = decrement(current_value).map(change);
                        } else {
                            let _ = increment(current_value).map(change);
                        }

                        shell.capture_event();
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key, ..
                }) => {
                    if cursor.is_over(layout.bounds()) {
                        match key {
                            Key::Named(key::Named::ArrowUp) => {
                                let _ = increment(current_value).map(change);
                            }
                            Key::Named(key::Named::ArrowDown) => {
                                let _ = decrement(current_value).map(change);
                            }
                            _ => (),
                        }

                        shell.capture_event();
                    }
                }
                Event::Keyboard(keyboard::Event::ModifiersChanged(
                    modifiers,
                )) => {
                    state.keyboard_modifiers = *modifiers;
                }
                _ => {}
            }
        };

        update();

        let current_status = if state.is_dragging {
            Status::Dragged
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status) {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let style =
            theme.style(&self.class, self.status.unwrap_or(Status::Active));

        let (handle_width, handle_height, handle_border_radius) =
            match style.handle.shape {
                HandleShape::Circle { radius } => {
                    (radius * 2.0, radius * 2.0, radius.into())
                }
                HandleShape::Rectangle {
                    width,
                    border_radius,
                } => (f32::from(width), bounds.height, border_radius),
            };

        let value = self.value.into() as f32;
        let (range_start, range_end) = {
            let (start, end) = self.range.clone().into_inner();

            (start.into() as f32, end.into() as f32)
        };

        let offset = if range_start >= range_end {
            0.0
        } else {
            (bounds.width - handle_width) * (value - range_start)
                / (range_end - range_start)
        };

        let rail_y = bounds.y + bounds.height / 2.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y - style.rail.width / 2.0,
                    width: offset + handle_width / 2.0,
                    height: style.rail.width,
                },
                border: style.rail.border,
                ..renderer::Quad::default()
            },
            style.rail.backgrounds.0,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + offset + handle_width / 2.0,
                    y: rail_y - style.rail.width / 2.0,
                    width: bounds.width - offset - handle_width / 2.0,
                    height: style.rail.width,
                },
                border: style.rail.border,
                ..renderer::Quad::default()
            },
            style.rail.backgrounds.1,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + offset,
                    y: rail_y - handle_height / 2.0,
                    width: handle_width,
                    height: handle_height,
                },
                border: Border {
                    radius: handle_border_radius,
                    width: style.handle.border_width,
                    color: style.handle.border_color,
                },
                ..renderer::Quad::default()
            },
            style.handle.background,
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

impl<'a, T, Message, Theme, Renderer> From<Slider<'a, T, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive + 'a,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(
        slider: Slider<'a, T, Message, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(slider)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_dragging: bool,
    keyboard_modifiers: keyboard::Modifiers,
}

/// The possible status of a [`Slider`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Slider`] can be interacted with.
    Active,
    /// The [`Slider`] is being hovered.
    Hovered,
    /// The [`Slider`] is being dragged.
    Dragged,
}

/// The appearance of a slider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The colors of the rail of the slider.
    pub rail: Rail,
    /// The appearance of the [`Handle`] of the slider.
    pub handle: Handle,
}

impl Style {
    /// Changes the [`HandleShape`] of the [`Style`] to a circle
    /// with the given radius.
    pub fn with_circular_handle(mut self, radius: impl Into<Pixels>) -> Self {
        self.handle.shape = HandleShape::Circle {
            radius: radius.into().0,
        };
        self
    }
}

/// The appearance of a slider rail
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rail {
    /// The backgrounds of the rail of the slider.
    pub backgrounds: (Background, Background),
    /// The width of the stroke of a slider rail.
    pub width: f32,
    /// The border of the rail.
    pub border: Border,
}

/// The appearance of the handle of a slider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Handle {
    /// The shape of the handle.
    pub shape: HandleShape,
    /// The [`Background`] of the handle.
    pub background: Background,
    /// The border width of the handle.
    pub border_width: f32,
    /// The border [`Color`] of the handle.
    pub border_color: Color,
}

/// The shape of the handle of a slider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandleShape {
    /// A circular handle.
    Circle {
        /// The radius of the circle.
        radius: f32,
    },
    /// A rectangular shape.
    Rectangle {
        /// The width of the rectangle.
        width: u16,
        /// The border radius of the corners of the rectangle.
        border_radius: border::Radius,
    },
}

/// The theme catalog of a [`Slider`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Slider`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Slider`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let color = match status {
        Status::Active => palette.primary.base.color,
        Status::Hovered => palette.primary.strong.color,
        Status::Dragged => palette.primary.weak.color,
    };

    Style {
        rail: Rail {
            backgrounds: (color.into(), palette.background.strong.color.into()),
            width: 4.0,
            border: Border {
                radius: 2.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
        },
        handle: Handle {
            shape: HandleShape::Circle { radius: 7.0 },
            background: color.into(),
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
        },
    }
}
