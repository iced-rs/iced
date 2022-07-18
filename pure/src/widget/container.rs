//! Decorate content and apply alignment.
use crate::widget::Tree;
use crate::{Element, Widget};

use iced_native::alignment;
use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget::container;
use iced_native::{
    Clipboard, Layout, Length, Padding, Point, Rectangle, Shell,
};

use std::u32;

pub use iced_style::container::{Appearance, StyleSheet};

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct Container<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: container::StyleSheet,
{
    padding: Padding,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    style: <Renderer::Theme as StyleSheet>::Style,
    content: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Container<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: container::StyleSheet,
{
    /// Creates an empty [`Container`].
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        Container {
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            style: Default::default(),
            content: content.into(),
        }
    }

    /// Sets the [`Padding`] of the [`Container`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Container`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Container`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Container`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = alignment::Horizontal::Center;
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = alignment::Vertical::Center;
        self
    }

    /// Sets the style of the [`Container`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Container<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

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
        container::layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let style = theme.appearance(self.style);

        container::draw_background(renderer, &style, layout.bounds());

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style
                    .text_color
                    .unwrap_or(renderer_style.text_color),
            },
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<Container<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        column: Container<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
