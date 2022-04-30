//! Display a dropdown list of selectable values.
use crate::widget::tree::{self, Tree};
use crate::{Element, Widget};

use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::text;
use iced_native::widget::pick_list;
use iced_native::{
    Clipboard, Layout, Length, Padding, Point, Rectangle, Shell,
};

use std::borrow::Cow;

pub use iced_style::pick_list::{Style, StyleSheet};

/// A widget for selecting a single value from a list of options.
#[allow(missing_debug_implementations)]
pub struct PickList<'a, T, Message, Renderer: text::Renderer>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    on_selected: Box<dyn Fn(T) -> Message + 'a>,
    options: Cow<'a, [T]>,
    placeholder: Option<String>,
    selected: Option<T>,
    width: Length,
    padding: Padding,
    text_size: Option<u16>,
    font: Renderer::Font,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, T: 'a, Message, Renderer: text::Renderer>
    PickList<'a, T, Message, Renderer>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
{
    /// The default padding of a [`PickList`].
    pub const DEFAULT_PADDING: Padding = Padding::new(5);

    /// Creates a new [`PickList`] with the given list of options, the current
    /// selected value, and the message to produce when an option is selected.
    pub fn new(
        options: impl Into<Cow<'a, [T]>>,
        selected: Option<T>,
        on_selected: impl Fn(T) -> Message + 'a,
    ) -> Self {
        Self {
            on_selected: Box::new(on_selected),
            options: options.into(),
            placeholder: None,
            selected,
            width: Length::Shrink,
            text_size: None,
            padding: Self::DEFAULT_PADDING,
            font: Default::default(),
            style_sheet: Default::default(),
        }
    }

    /// Sets the placeholder of the [`PickList`].
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the width of the [`PickList`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the [`Padding`] of the [`PickList`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`PickList`].
    pub fn text_size(mut self, size: u16) -> Self {
        self.text_size = Some(size);
        self
    }

    /// Sets the font of the [`PickList`].
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`PickList`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }
}

impl<'a, T: 'a, Message, Renderer> Widget<Message, Renderer>
    for PickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Message: 'a,
    Renderer: text::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<pick_list::State<T>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(pick_list::State::<T>::new())
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
        pick_list::layout(
            renderer,
            limits,
            self.width,
            self.padding,
            self.text_size,
            &self.font,
            self.placeholder.as_ref().map(String::as_str),
            &self.options,
        )
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
        pick_list::update(
            event,
            layout,
            cursor_position,
            shell,
            self.on_selected.as_ref(),
            self.selected.as_ref(),
            &self.options,
            || tree.state.downcast_mut::<pick_list::State<T>>(),
        )
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        pick_list::mouse_interaction(layout, cursor_position)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        pick_list::draw(
            renderer,
            layout,
            cursor_position,
            self.padding,
            self.text_size,
            &self.font,
            self.placeholder.as_ref().map(String::as_str),
            self.selected.as_ref(),
            self.style_sheet.as_ref(),
        )
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let state = tree.state.downcast_mut::<pick_list::State<T>>();

        pick_list::overlay(
            layout,
            state,
            self.padding,
            self.text_size,
            self.font.clone(),
            &self.options,
            self.style_sheet.as_ref(),
        )
    }
}

impl<'a, T: 'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for PickList<'a, T, Message, Renderer>
where
    T: Clone + ToString + Eq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: text::Renderer + 'a,
    Message: 'a,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
