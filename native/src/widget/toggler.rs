//! Show toggle controls using togglers.
use crate::alignment;
use crate::event;
use crate::keyboard;
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text;
use crate::widget::tree::{self, Tree};
use crate::widget::{self, Row, Text};
use crate::widget::operation::{self, Operation};
use crate::{
    Alignment, Clipboard, Element, Event, Layout, Length, Point, Rectangle,
    Shell, Widget,
};

pub use iced_style::toggler::{Appearance, StyleSheet};



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


/// A toggler widget.
///
/// # Example
///
/// ```
/// # type Toggler<'a, Message> = iced_native::widget::Toggler<'a, Message, iced_native::renderer::Null>;
/// #
/// pub enum Message {
///     TogglerToggled(bool),
/// }
///
/// let is_active = true;
///
/// Toggler::new(is_active, String::from("Toggle me!"), |b| Message::TogglerToggled(b));
/// ```
#[allow(missing_debug_implementations)]
pub struct Toggler<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    is_active: bool,
    on_toggle: Box<dyn Fn(bool) -> Message + 'a>,
    label: Option<String>,
    width: Length,
    size: u16,
    text_size: Option<u16>,
    text_alignment: alignment::Horizontal,
    spacing: u16,
    font: Renderer::Font,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Toggler<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// The default size of a [`Toggler`].
    pub const DEFAULT_SIZE: u16 = 20;

    /// Creates a new [`Toggler`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Toggler`] is checked or not
    ///   * An optional label for the [`Toggler`]
    ///   * a function that will be called when the [`Toggler`] is toggled. It
    ///     will receive the new state of the [`Toggler`] and must produce a
    ///     `Message`.
    pub fn new<F>(
        is_active: bool,
        label: impl Into<Option<String>>,
        f: F,
    ) -> Self
    where
        F: 'a + Fn(bool) -> Message,
    {
        Toggler {
            id: None,
            is_active,
            on_toggle: Box::new(f),
            label: label.into(),
            width: Length::Fill,
            size: Self::DEFAULT_SIZE,
            text_size: None,
            text_alignment: alignment::Horizontal::Left,
            spacing: 0,
            font: Renderer::Font::default(),
            style: Default::default(),
        }
    }


    /// Sets the [`Id`] of the [`Checkbox`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the size of the [`Toggler`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Sets the width of the [`Toggler`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the text size o the [`Toggler`].
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = Some(text_size);
        self
    }

    /// Sets the horizontal alignment of the text of the [`Toggler`]
    pub fn text_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.text_alignment = alignment;
        self
    }

    /// Sets the spacing between the [`Toggler`] and the text.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the [`Font`] of the text of the [`Toggler`]
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Toggler`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Toggler<'a, Message, Renderer>
where
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
        let mut row = Row::<(), Renderer>::new()
            .width(self.width)
            .spacing(self.spacing)
            .align_items(Alignment::Center);

        if let Some(label) = &self.label {
            row = row.push(
                Text::new(label)
                    .horizontal_alignment(self.text_alignment)
                    .font(self.font.clone())
                    .width(self.width)
                    .size(
                        self.text_size
                            .unwrap_or_else(|| renderer.default_size()),
                    ),
            );
        }

        row = row.push(
            Row::new()
                .width(Length::Units(2 * self.size))
                .height(Length::Units(self.size)),
        );

        row.layout(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: Layout<'_>,
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
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let mouse_over = layout.bounds().contains(cursor_position);
                if mouse_over {
                    shell.publish((self.on_toggle)(!self.is_active));

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {    
                if !state.is_focused() {
                    return event::Status::Ignored;
                }

                match key_code {
                    keyboard::KeyCode::Enter
                    | keyboard::KeyCode::NumpadEnter 
                    | keyboard::KeyCode::Space => {
                        shell.publish((self.on_toggle)(!self.is_active));
                        return event::Status::Captured;
                    }
                    _ => {}
                }    
            }
            _ => {},
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
        /// Makes sure that the border radius of the toggler looks good at every size.
        const BORDER_RADIUS_RATIO: f32 = 32.0 / 13.0;

        /// The space ratio between the background Quad and the Toggler bounds, and
        /// between the background Quad and foreground Quad.
        const SPACE_RATIO: f32 = 0.05;

        let state = tree.state.downcast_ref::<State>();
        let mut children = layout.children();
        let toggler_layout = children.next().unwrap();
        let bounds = toggler_layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);
        let is_focused = is_mouse_over | state.is_focused();

        if let Some(label) = &self.label {
            let label_layout = children.next().unwrap();

            crate::widget::text::draw(
                renderer,
                style,
                label_layout,
                label,
                self.text_size,
                self.font.clone(),
                Default::default(),
                self.text_alignment,
                alignment::Vertical::Center,
            );
        }

        let style = if is_focused {
            theme.hovered(&self.style, self.is_active)
        } else {
            theme.active(&self.style, self.is_active)
        };

        let border_radius = bounds.height as f32 / BORDER_RADIUS_RATIO;
        let space = SPACE_RATIO * bounds.height as f32;

        let toggler_background_bounds = Rectangle {
            x: bounds.x + space,
            y: bounds.y + space,
            width: bounds.width - (2.0 * space),
            height: bounds.height - (2.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_background_bounds,
                border_radius,
                border_width: 1.0,
                border_color: style
                    .background_border
                    .unwrap_or(style.background),
            },
            style.background,
        );

        let toggler_foreground_bounds = Rectangle {
            x: bounds.x
                + if self.is_active {
                    bounds.width - 2.0 * space - (bounds.height - (4.0 * space))
                } else {
                    2.0 * space
                },
            y: bounds.y + (2.0 * space),
            width: bounds.height - (4.0 * space),
            height: bounds.height - (4.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_foreground_bounds,
                border_radius,
                border_width: 1.0,
                border_color: style
                    .foreground_border
                    .unwrap_or(style.foreground),
            },
            style.foreground,
        );
    }
}

impl<'a, Message, Renderer> From<Toggler<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet + widget::text::StyleSheet,
{
    fn from(
        toggler: Toggler<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(toggler)
    }
}
