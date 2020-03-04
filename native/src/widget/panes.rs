use crate::{
    layout, Clipboard, Element, Event, Hasher, Layout, Length, Point, Size,
    Widget,
};

use std::collections::HashMap;

#[allow(missing_debug_implementations)]
pub struct Panes<'a, Message, Renderer> {
    state: &'a mut Internal,
    elements: Vec<Element<'a, Message, Renderer>>,
    width: Length,
    height: Length,
}

impl<'a, Message, Renderer> Panes<'a, Message, Renderer> {
    pub fn new<T>(
        state: &'a mut State<T>,
        view: impl Fn(Pane, &'a mut T) -> Element<'a, Message, Renderer>,
    ) -> Self {
        let elements = state
            .panes
            .iter_mut()
            .map(|(pane, state)| view(*pane, state))
            .collect();

        Self {
            state: &mut state.internal,
            elements,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    /// Sets the width of the [`Panes`].
    ///
    /// [`Panes`]: struct.Column.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Panes`].
    ///
    /// [`Panes`]: struct.Column.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Panes<'a, Message, Renderer>
where
    Renderer: self::Renderer + 'static,
    Message: 'static,
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
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        let children = self
            .elements
            .iter()
            .map(|element| element.layout(renderer, &limits))
            .collect();

        layout::Node::with_children(size, children)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(defaults, &self.elements, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<Panes<'_, Message, Renderer>>().hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.state.layout.hash(state);

        for element in &self.elements {
            element.hash_layout(state);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pane(usize);

#[derive(Debug)]
pub struct State<T> {
    panes: HashMap<Pane, T>,
    internal: Internal,
}

#[derive(Debug)]
struct Internal {
    layout: Node,
    last_pane: usize,
    focused_pane: Option<Pane>,
}

impl<T> State<T> {
    pub fn new(first_pane_state: T) -> (Self, Pane) {
        let first_pane = Pane(0);

        let mut panes = HashMap::new();
        let _ = panes.insert(first_pane, first_pane_state);

        (
            State {
                panes,
                internal: Internal {
                    layout: Node::Pane(first_pane),
                    last_pane: 0,
                    focused_pane: None,
                },
            },
            first_pane,
        )
    }

    pub fn get_mut(&mut self, pane: &Pane) -> Option<&mut T> {
        self.panes.get_mut(pane)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Pane, &T)> {
        self.panes.iter().map(|(pane, state)| (*pane, state))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Pane, &mut T)> {
        self.panes.iter_mut().map(|(pane, state)| (*pane, state))
    }

    pub fn focused_pane(&self) -> Option<Pane> {
        self.internal.focused_pane
    }

    pub fn focus(&mut self, pane: Pane) {
        self.internal.focused_pane = Some(pane);
    }

    pub fn split_vertically(&mut self, pane: &Pane, state: T) -> Option<Pane> {
        let new_pane = Pane(self.internal.last_pane.checked_add(1)?);

        // TODO

        Some(new_pane)
    }

    pub fn split_horizontally(
        &mut self,
        pane: &Pane,
        state: T,
    ) -> Option<Pane> {
        let new_pane = Pane(self.internal.last_pane.checked_add(1)?);

        // TODO

        Some(new_pane)
    }
}

#[derive(Debug, Clone, Hash)]
enum Node {
    Split {
        kind: Split,
        ratio: u32,
        a: Box<Node>,
        b: Box<Node>,
    },
    Pane(Pane),
}

#[derive(Debug, Clone, Copy, Hash)]
enum Split {
    Horizontal,
    Vertical,
}

/// The renderer of some [`Panes`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use [`Panes`] in your user interface.
///
/// [`Panes`]: struct.Panes.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Draws some [`Panes`].
    ///
    /// It receives:
    /// - the children of the [`Column`]
    /// - the [`Layout`] of the [`Column`] and its children
    /// - the cursor position
    ///
    /// [`Column`]: struct.Row.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[Element<'_, Message, Self>],
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Panes<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer + 'static,
    Message: 'static,
{
    fn from(
        panes: Panes<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(panes)
    }
}
