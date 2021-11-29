//! Distribute content horizontally.
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::{
    Alignment, Clipboard, Element, Hasher, Layout, Length, Padding, Point,
    Rectangle, Shell, Widget,
};

use std::hash::Hash;
use std::u32;

/// A container that distributes its contents horizontally.
#[allow(missing_debug_implementations)]
pub struct Row<'a, Message, Renderer> {
    spacing: u16,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    align_items: Alignment,
    children: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Row<'a, Message, Renderer> {
    /// Creates an empty [`Row`].
    pub fn new() -> Self {
        Self::with_children(Vec::new())
    }

    /// Creates a [`Row`] with the given elements.
    pub fn with_children(
        children: Vec<Element<'a, Message, Renderer>>,
    ) -> Self {
        Row {
            spacing: 0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            align_items: Alignment::Start,
            children,
        }
    }

    /// Sets the horizontal spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the [`Padding`] of the [`Row`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Row`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Row`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Row`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Row`].
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the vertical alignment of the contents of the [`Row`] .
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an [`Element`] to the [`Row`].
    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        self.children.push(child.into());
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Row<'a, Message, Renderer>
where
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
        let limits = limits
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        layout::flex::resolve(
            layout::flex::Axis::Horizontal,
            renderer,
            &limits,
            self.padding,
            self.spacing as f32,
            self.align_items,
            &self.children,
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
        self.children
            .iter_mut()
            .zip(layout.children())
            .map(|(child, layout)| {
                child.widget.on_event(
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(layout.children())
            .map(|(child, layout)| {
                child.widget.mouse_interaction(
                    layout,
                    cursor_position,
                    viewport,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        for (child, layout) in self.children.iter().zip(layout.children()) {
            child.draw(renderer, style, layout, cursor_position, viewport);
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.align_items.hash(state);
        self.spacing.hash(state);
        self.padding.hash(state);

        for child in &self.children {
            child.widget.hash_layout(state);
        }
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.children
            .iter_mut()
            .zip(layout.children())
            .filter_map(|(child, layout)| child.widget.overlay(layout))
            .next()
    }
}

impl<'a, Message, Renderer> From<Row<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + crate::Renderer,
    Message: 'a,
{
    fn from(row: Row<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(row)
    }
}
