//! Create choices using radio buttons.
use crate::{alignment, keyboard};
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text;
use crate::touch;
use crate::widget::tree::{self, Tree};
use crate::widget::{self, Row, Text};
use crate::widget::operation::{self, Operation};
use crate::{
    Alignment, Clipboard, Color, Element, Layout, Length, Point, Rectangle,
    Shell, Widget,
};

pub use iced_style::radio::{Appearance, StyleSheet};


/// The identifier of a [`Checkbox`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

/// The local state of a [`Checkbox`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_focused: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }

    /// Creates a new [`State`], representing a focused [`Button`].
    pub fn focused() -> Self {
        Self {
            is_focused: true,
        }
    }

    /// Returns whether the [`Button`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Focuses the [`Button`].
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocuses the [`Button`].
    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        State::is_focused(self)
    }

    fn focus(&mut self) {
        State::focus(self)
    }

    fn unfocus(&mut self) {
        State::unfocus(self)
    }
}


/// A circular button representing a choice.
///
/// # Example
/// ```
/// # type Radio<Message> =
/// #     iced_native::widget::Radio<Message, iced_native::renderer::Null>;
/// #
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum Choice {
///     A,
///     B,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// pub enum Message {
///     RadioSelected(Choice),
/// }
///
/// let selected_choice = Some(Choice::A);
///
/// Radio::new(Choice::A, "This is A", selected_choice, Message::RadioSelected);
///
/// Radio::new(Choice::B, "This is B", selected_choice, Message::RadioSelected);
/// ```
///
/// ![Radio buttons drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/radio.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Radio<Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    is_selected: bool,
    on_click: Message,
    label: String,
    width: Length,
    size: u16,
    spacing: u16,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<Message, Renderer> Radio<Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// The default size of a [`Radio`] button.
    pub const DEFAULT_SIZE: u16 = 28;

    /// The default spacing of a [`Radio`] button.
    pub const DEFAULT_SPACING: u16 = 15;

    /// Creates a new [`Radio`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Radio`] button
    ///   * the label of the [`Radio`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Radio`] is selected. It
    ///   receives the value of the radio and must produce a `Message`.
    pub fn new<F, V>(
        value: V,
        label: impl Into<String>,
        selected: Option<V>,
        f: F,
    ) -> Self
    where
        V: Eq + Copy,
        F: FnOnce(V) -> Message,
    {
        Radio {
            id: None,
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: Self::DEFAULT_SPACING, //15
            text_size: None,
            font: Default::default(),
            style: Default::default(),
        }
    }


    /// Sets the [`Id`] of the [`Checkbox`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the size of the [`Radio`] button.
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the [`Radio`] button.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the spacing between the [`Radio`] button and the text.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the text size of the [`Radio`] button.
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the text font of the [`Radio`] button.
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Radio`] button.
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Radio<Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + widget::text::StyleSheet,
{
    
    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Row::<(), Renderer>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Alignment::Center)
            .push(
                Row::new()
                    .width(Length::Units(self.size))
                    .height(Length::Units(self.size)),
            )
            .push(Text::new(&self.label).width(self.width).size(
                self.text_size.unwrap_or_else(|| renderer.default_size()),
            ))
            .layout(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<State>();
        operation.focusable(state, self.id.as_ref().map(|id| &id.0));
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    shell.publish(self.on_click.clone());

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {    
                if state.is_focused  {
                    match key_code {
                        keyboard::KeyCode::Enter
                        | keyboard::KeyCode::NumpadEnter 
                        | keyboard::KeyCode::Space => {
                            shell.publish(self.on_click.clone());
                            return event::Status::Captured;
                        }
                        _ => {
                            return event::Status::Ignored;
                        }
                    }    
                }
                return event::Status::Ignored;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if layout.bounds().contains(cursor_position) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);
        let is_focused = is_mouse_over | state.is_focused();

        let mut children = layout.children();

        let styling = if is_focused {
            theme.hovered(&self.style, self.is_selected)
        } else {
            theme.active(&self.style, self.is_selected)
        };

        {
            let layout = children.next().unwrap();
            let bounds = layout.bounds();

            let size = bounds.width;
            let dot_size = size / 2.0;

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: size / 2.0,
                    border_width: styling.border_width,
                    border_color: styling.border_color,
                },
                styling.background,
            );

            if self.is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + dot_size / 2.0,
                            y: bounds.y + dot_size / 2.0,
                            width: bounds.width - dot_size,
                            height: bounds.height - dot_size,
                        },
                        border_radius: dot_size / 2.0,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    styling.dot_color,
                );
            }
        }

        {
            let label_layout = children.next().unwrap();

            widget::text::draw(
                renderer,
                style,
                label_layout,
                &self.label,
                self.text_size,
                self.font.clone(),
                widget::text::Appearance {
                    color: styling.text_color,
                },
                alignment::Horizontal::Left,
                alignment::Vertical::Center,
            );
        }
    }
}

impl<'a, Message, Renderer> From<Radio<Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet + widget::text::StyleSheet,
{
    fn from(radio: Radio<Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(radio)
    }
}
