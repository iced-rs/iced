//! Display a dropdown list of selectable values.
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::overlay::menu::{self, Menu};
use crate::scrollable;
use crate::text;
use crate::touch;
use crate::{
    Clipboard, Element, Hasher, Layout, Length, Padding, Point, Rectangle,
    Size, Widget,
};

/// A widget for selecting a single value from a dynamic scrollable list of options.
#[allow(missing_debug_implementations)]
pub struct SelectionList<'a, T, Message, Renderer: self::Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    menu: &'a mut menu::State,
    hovered_option: &'a mut Option<usize>,
    last_selection: &'a mut Option<T>,
    on_selected: Box<dyn Fn(T) -> Message>,
    selected: Option<T>,
    options: Vec<T>,
    width: Length,
    padding: Padding,
    text_size: Option<u16>,
    font: Renderer::Font,
    style: <Renderer as self::Renderer>::Style,
}

/// The local state of a [`SelectionList`].
#[derive(Debug, Clone)]
pub struct State<T> {
    menu: menu::State,
    hovered_option: Option<usize>,
    last_selection: Option<T>,
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self {
            menu: menu::State::default(),
            hovered_option: Option::default(),
            last_selection: Option::default(),
        }
    }
}

impl<'a, T: 'a, Message, Renderer: self::Renderer>
    SelectionList<'a, T, Message, Renderer>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
{
    /// Creates a new [`SelectionList`] with the given [`State`], a list of options,
    /// the current selected value, and the message to produce when an option is
    /// selected.
    pub fn new(
        state: &'a mut State<T>,
        options: Vec<T>,
        selected: Option<T>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        let State {
            menu,
            hovered_option,
            last_selection,
        } = state;

        Self {
            menu,
            hovered_option,
            last_selection,
            on_selected: Box::new(on_selected),
            options: options,
            selected,
            width: Length::Shrink,
            text_size: None,
            padding: Renderer::DEFAULT_PADDING,
            font: Default::default(),
            style: Default::default(),
        }
    }

    /// Sets the width of the [`SelectionList`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the [`Padding`] of the [`SelectionList`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`SelectionList`].
    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Sets the font of the [`SelectionList`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    fn update_hovered_option(&mut self) {
        let selected = self.selected.as_ref();

        *self.hovered_option = self
            .options
            .iter()
            .position(|option| Some(option) == selected);
    }

    /// Sets the style of the [`SelectionList`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as self::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Insert into the [`SelectionList`].
    pub fn insert(&mut self, index: usize, element: T) {
        self.options.insert(index, element);
        self.update_hovered_option();
    }

    /// push into the end of the [`SelectionList`].
    pub fn push(&mut self, element: T) {
        self.options.push(element);
    }
}

impl<'a, T: 'a, Message, Renderer> Widget<Message, Renderer>
    for SelectionList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: 'static,
    Renderer: self::Renderer + scrollable::Renderer + 'a,
{
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
        use std::f32;

        let limits = limits
            .width(self.width)
            .height(Length::Shrink)
            .pad(self.padding);

        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let max_width = match self.width {
            Length::Shrink => {
                let labels = self.options.iter().map(ToString::to_string);

                labels
                    .map(|label| {
                        let (width, _) = renderer.measure(
                            &label,
                            text_size,
                            self.font,
                            Size::new(f32::INFINITY, f32::INFINITY),
                        );

                        width.round() as u32
                    })
                    .max()
                    .unwrap_or(100)
            }
            _ => 0,
        };

        let size = {
            let intrinsic = Size::new(
                max_width as f32
                    + f32::from(text_size)
                    + f32::from(self.padding.left),
                f32::from(text_size),
            );

            limits.resolve(intrinsic).pad(self.padding)
        };

        layout::Node::new(size)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash as _;

        match self.width {
            Length::Shrink => {
                self.options
                    .iter()
                    .map(ToString::to_string)
                    .for_each(|label| label.hash(state));
            }
            _ => {
                self.width.hash(state);
            }
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let event_status = if layout.bounds().contains(cursor_position)
                {
                    self.update_hovered_option();

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                };

                if let Some(last_selection) = self.last_selection.take() {
                    self.selected = Some(last_selection.clone());
                    messages.push((self.on_selected)(last_selection));

                    event::Status::Captured
                } else {
                    event_status
                }
            }
            _ => event::Status::Ignored,
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        self::Renderer::draw(renderer)
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let bounds = layout.bounds();

        let mut menu = Menu::new(
            &mut self.menu,
            &self.options[..],
            &mut self.hovered_option,
            &mut self.last_selection,
        )
        .width(bounds.width.round() as u16)
        .padding(self.padding)
        .font(self.font)
        .style(Renderer::menu_style(&self.style));

        if let Some(text_size) = self.text_size {
            menu = menu.text_size(text_size);
        }

        Some(menu.overlay(layout.position(), bounds.height))
    }
}

/// The renderer of a [`SelectionList`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`SelectionList`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer: text::Renderer + menu::Renderer {
    /// The default padding of a [`SelectionList`].
    const DEFAULT_PADDING: Padding;

    /// The [`SelectionList`] style supported by this renderer.
    type Style: Default;

    /// Returns the style of the [`Menu`] of the [`SelectionList`].
    fn menu_style(
        style: &<Self as Renderer>::Style,
    ) -> <Self as menu::Renderer>::Style;

    /// Draws a [`SelectionList`].
    fn draw(&mut self) -> Self::Output;
}

impl<'a, T: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for SelectionList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: self::Renderer + 'a,
    Message: 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
