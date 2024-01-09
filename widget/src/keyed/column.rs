//! Distribute content vertically.
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::Operation;
use crate::core::{
    Alignment, Clipboard, Element, Layout, Length, Padding, Pixels, Rectangle,
    Shell, Size, Widget,
};

/// A container that distributes its contents vertically.
#[allow(missing_debug_implementations)]
pub struct Column<'a, Key, Message, Renderer = crate::Renderer>
where
    Key: Copy + PartialEq,
{
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    align_items: Alignment,
    keys: Vec<Key>,
    children: Vec<Element<'a, Message, Renderer>>,
}

impl<'a, Key, Message, Renderer> Column<'a, Key, Message, Renderer>
where
    Key: Copy + PartialEq,
    Renderer: crate::core::Renderer,
{
    /// Creates an empty [`Column`].
    pub fn new() -> Self {
        Column {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::INFINITY,
            align_items: Alignment::Start,
            keys: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Creates a [`Column`] with the given elements.
    pub fn with_children(
        children: impl IntoIterator<Item = (Key, Element<'a, Message, Renderer>)>,
    ) -> Self {
        children
            .into_iter()
            .fold(Self::new(), |column, (key, child)| column.push(key, child))
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`Column`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [`Column`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Column`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Column`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Column`] .
    pub fn align_items(mut self, align: Alignment) -> Self {
        self.align_items = align;
        self
    }

    /// Adds an element to the [`Column`].
    pub fn push(
        mut self,
        key: Key,
        child: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        let child = child.into();
        let size = child.as_widget().size_hint();

        if size.width.is_fill() {
            self.width = Length::Fill;
        }

        if size.height.is_fill() {
            self.height = Length::Fill;
        }

        self.keys.push(key);
        self.children.push(child);
        self
    }
}

impl<'a, Key, Message, Renderer> Default for Column<'a, Key, Message, Renderer>
where
    Key: Copy + PartialEq,
    Renderer: crate::core::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

struct State<Key>
where
    Key: Copy + PartialEq,
{
    keys: Vec<Key>,
}

impl<'a, Key, Message, Renderer> Widget<Message, Renderer>
    for Column<'a, Key, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Key: Copy + PartialEq + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Key>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            keys: self.keys.clone(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let Tree {
            state, children, ..
        } = tree;

        let state = state.downcast_mut::<State<Key>>();

        tree::diff_children_custom_with_search(
            children,
            &self.children,
            |tree, child| child.as_widget().diff(tree),
            |index| {
                self.keys.get(index).or_else(|| self.keys.last()).copied()
                    != Some(state.keys[index])
            },
            |child| Tree::new(child.as_widget()),
        );

        if state.keys != self.keys {
            state.keys = self.keys.clone();
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .max_width(self.max_width)
            .width(self.width)
            .height(self.height);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.width,
            self.height,
            self.padding,
            self.spacing,
            self.align_items,
            &self.children,
            &mut tree.children,
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state, layout, cursor, viewport, renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer)
    }
}

impl<'a, Key, Message, Renderer> From<Column<'a, Key, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Key: Copy + PartialEq + 'static,
    Message: 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(column: Column<'a, Key, Message, Renderer>) -> Self {
        Self::new(column)
    }
}
