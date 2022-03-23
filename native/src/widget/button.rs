//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::touch;
use crate::{
    Background, Clipboard, Color, Element, Layout, Length, Padding, Point,
    Rectangle, Shell, Vector, Widget,
};

pub use iced_style::button::{Style, StyleSheet};

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_native::widget::{button, Text};
/// #
/// # type Button<'a, Message> =
/// #     iced_native::widget::Button<'a, Message, iced_native::renderer::Null>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// let mut state = button::State::new();
/// let button = Button::new(&mut state, Text::new("Press me!"))
///     .on_press(Message::ButtonPressed);
/// ```
///
/// If a [`Button::on_press`] handler is not set, the resulting [`Button`] will
/// be disabled:
///
/// ```
/// # use iced_native::widget::{button, Text};
/// #
/// # type Button<'a, Message> =
/// #     iced_native::widget::Button<'a, Message, iced_native::renderer::Null>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn disabled_button(state: &mut button::State) -> Button<'_, Message> {
///     Button::new(state, Text::new("I'm disabled!"))
/// }
///
/// fn enabled_button(state: &mut button::State) -> Button<'_, Message> {
///     disabled_button(state).on_press(Message::ButtonPressed)
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Button<'a, Message, Renderer> {
    state: &'a mut State,
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: crate::Renderer,
{
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    pub fn new<E>(state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Button {
            state,
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5),
            style_sheet: Default::default(),
        }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    /// If on_press isn't set, button will be disabled.
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// Sets the style of the [`Button`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}

/// Processes the given [`Event`] and updates the [`State`] of a [`Button`]
/// accordingly.
pub fn update<'a, Message: Clone>(
    event: Event,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    on_press: &Option<Message>,
    state: impl FnOnce() -> &'a mut State,
) -> event::Status {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            if on_press.is_some() {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    let state = state();

                    state.is_pressed = true;

                    return event::Status::Captured;
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) => {
            if let Some(on_press) = on_press.clone() {
                let state = state();

                if state.is_pressed {
                    state.is_pressed = false;

                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        shell.publish(on_press);
                    }

                    return event::Status::Captured;
                }
            }
        }
        Event::Touch(touch::Event::FingerLost { .. }) => {
            let state = state();

            state.is_pressed = false;
        }
        _ => {}
    }

    event::Status::Ignored
}

/// Draws a [`Button`].
pub fn draw<'a, Renderer: crate::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    cursor_position: Point,
    is_enabled: bool,
    style_sheet: &dyn StyleSheet,
    state: impl FnOnce() -> &'a State,
) -> Style {
    let is_mouse_over = bounds.contains(cursor_position);

    let styling = if !is_enabled {
        style_sheet.disabled()
    } else if is_mouse_over {
        let state = state();

        if state.is_pressed {
            style_sheet.pressed()
        } else {
            style_sheet.hovered()
        }
    } else {
        style_sheet.active()
    };

    if styling.background.is_some() || styling.border_width > 0.0 {
        if styling.shadow_offset != Vector::default() {
            // TODO: Implement proper shadow support
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + styling.shadow_offset.x,
                        y: bounds.y + styling.shadow_offset.y,
                        ..bounds
                    },
                    border_radius: styling.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                Background::Color([0.0, 0.0, 0.0, 0.5].into()),
            );
        }

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: styling.border_radius,
                border_width: styling.border_width,
                border_color: styling.border_color,
            },
            styling
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }

    styling
}

/// Computes the layout of a [`Button`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    padding: Padding,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.width(width).height(height).pad(padding);

    let mut content = layout_content(renderer, &limits);
    content.move_to(Point::new(padding.left.into(), padding.top.into()));

    let size = limits.resolve(content.size()).pad(padding);

    layout::Node::with_children(size, vec![content])
}

/// Returns the [`mouse::Interaction`] of a [`Button`].
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor_position: Point,
    is_enabled: bool,
) -> mouse::Interaction {
    let is_mouse_over = layout.bounds().contains(cursor_position);

    if is_mouse_over && is_enabled {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: crate::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            |renderer, limits| self.content.layout(renderer, limits),
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.on_event(
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        update(
            event,
            layout,
            cursor_position,
            shell,
            &self.on_press,
            || &mut self.state,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor_position, self.on_press.is_some())
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
        let content_layout = layout.children().next().unwrap();

        let styling = draw(
            renderer,
            bounds,
            cursor_position,
            self.on_press.is_some(),
            self.style_sheet.as_ref(),
            || &self.state,
        );

        self.content.draw(
            renderer,
            &renderer::Style {
                text_color: styling.text_color,
            },
            content_layout,
            cursor_position,
            &bounds,
        );
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content
            .overlay(layout.children().next().unwrap(), renderer)
    }
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::Renderer,
{
    fn from(
        button: Button<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(button)
    }
}
