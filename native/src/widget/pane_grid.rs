use crate::{
    input::{keyboard, mouse, ButtonState},
    layout, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use std::collections::HashMap;

#[allow(missing_debug_implementations)]
pub struct PaneGrid<'a, Message, Renderer> {
    state: &'a mut Internal,
    elements: Vec<(Pane, Element<'a, Message, Renderer>)>,
    width: Length,
    height: Length,
    on_drop: Option<Box<dyn Fn(Drop) -> Message>>,
}

impl<'a, Message, Renderer> PaneGrid<'a, Message, Renderer> {
    pub fn new<T>(
        state: &'a mut State<T>,
        view: impl Fn(
            Pane,
            &'a mut T,
            Option<Focus>,
        ) -> Element<'a, Message, Renderer>,
    ) -> Self {
        let elements = {
            let focused_pane = state.internal.focused_pane;

            state
                .panes
                .iter_mut()
                .map(move |(pane, pane_state)| {
                    let focus = match focused_pane {
                        FocusedPane::Some {
                            pane: focused_pane,
                            focus,
                        } if *pane == focused_pane => Some(focus),
                        _ => None,
                    };

                    (*pane, view(*pane, pane_state, focus))
                })
                .collect()
        };

        Self {
            state: &mut state.internal,
            elements,
            width: Length::Fill,
            height: Length::Fill,
            on_drop: None,
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

    pub fn on_drop(mut self, f: impl Fn(Drop) -> Message + 'static) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Drop {
    pub pane: Pane,
    pub target: Pane,
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for PaneGrid<'a, Message, Renderer>
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

        let regions = self.state.layout.regions(size);

        let children = self
            .elements
            .iter()
            .filter_map(|(pane, element)| {
                let region = regions.get(pane)?;
                let size = Size::new(region.width, region.height);

                let mut node =
                    element.layout(renderer, &layout::Limits::new(size, size));

                node.move_to(Point::new(region.x, region.y));

                Some(node)
            })
            .collect();

        layout::Node::with_children(size, children)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state,
            }) => match state {
                ButtonState::Pressed => {
                    let mut clicked_region =
                        self.elements.iter().zip(layout.children()).filter(
                            |(_, layout)| {
                                layout.bounds().contains(cursor_position)
                            },
                        );

                    if let Some(((pane, _), _)) = clicked_region.next() {
                        self.state.focused_pane = if self.on_drop.is_some()
                            && self.state.modifiers.alt
                        {
                            FocusedPane::Some {
                                pane: *pane,
                                focus: Focus::Dragging,
                            }
                        } else {
                            FocusedPane::Some {
                                pane: *pane,
                                focus: Focus::Idle,
                            }
                        }
                    }
                }
                ButtonState::Released => {
                    if let Some(on_drop) = &self.on_drop {
                        if let FocusedPane::Some {
                            pane,
                            focus: Focus::Dragging,
                        } = self.state.focused_pane
                        {
                            let mut dropped_region = self
                                .elements
                                .iter()
                                .zip(layout.children())
                                .filter(|(_, layout)| {
                                    layout.bounds().contains(cursor_position)
                                });

                            if let Some(((target, _), _)) =
                                dropped_region.next()
                            {
                                if pane != *target {
                                    messages.push(on_drop(Drop {
                                        pane,
                                        target: *target,
                                    }));
                                }
                            }

                            self.state.focused_pane = FocusedPane::Some {
                                pane,
                                focus: Focus::Idle,
                            };
                        }
                    }
                }
            },
            Event::Keyboard(keyboard::Event::Input { modifiers, .. }) => {
                self.state.modifiers = modifiers;
            }
            _ => {}
        }

        match self.state.focused_pane {
            FocusedPane::Some {
                focus: Focus::Dragging,
                ..
            } => {}
            _ => {
                self.elements.iter_mut().zip(layout.children()).for_each(
                    |((_, pane), layout)| {
                        pane.widget.on_event(
                            event.clone(),
                            layout,
                            cursor_position,
                            messages,
                            renderer,
                            clipboard,
                        )
                    },
                );
            }
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let dragging = match self.state.focused_pane {
            FocusedPane::Some {
                pane,
                focus: Focus::Dragging,
            } => Some(pane),
            _ => None,
        };

        renderer.draw(
            defaults,
            &self.elements,
            dragging,
            layout,
            cursor_position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<PaneGrid<'_, Message, Renderer>>().hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.state.layout.hash(state);

        for (_, element) in &self.elements {
            element.hash_layout(state);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pane(usize);

impl Pane {
    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct State<T> {
    panes: HashMap<Pane, T>,
    internal: Internal,
}

#[derive(Debug)]
struct Internal {
    layout: Node,
    last_pane: usize,
    focused_pane: FocusedPane,
    modifiers: keyboard::ModifiersState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Idle,
    Dragging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedPane {
    None,
    Some { pane: Pane, focus: Focus },
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
                    focused_pane: FocusedPane::None,
                    modifiers: keyboard::ModifiersState::default(),
                },
            },
            first_pane,
        )
    }

    pub fn len(&self) -> usize {
        self.panes.len()
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
        match self.internal.focused_pane {
            FocusedPane::Some {
                pane,
                focus: Focus::Idle,
            } => Some(pane),
            FocusedPane::Some {
                focus: Focus::Dragging,
                ..
            } => None,
            FocusedPane::None => None,
        }
    }

    pub fn focus(&mut self, pane: Pane) {
        self.internal.focused_pane = FocusedPane::Some {
            pane,
            focus: Focus::Idle,
        };
    }

    pub fn split_vertically(&mut self, pane: &Pane, state: T) -> Option<Pane> {
        self.split(Split::Vertical, pane, state)
    }

    pub fn split_horizontally(
        &mut self,
        pane: &Pane,
        state: T,
    ) -> Option<Pane> {
        self.split(Split::Horizontal, pane, state)
    }

    pub fn split(
        &mut self,
        kind: Split,
        pane: &Pane,
        state: T,
    ) -> Option<Pane> {
        let node = self.internal.layout.find(pane)?;

        let new_pane = {
            self.internal.last_pane = self.internal.last_pane.checked_add(1)?;

            Pane(self.internal.last_pane)
        };

        node.split(kind, new_pane);

        let _ = self.panes.insert(new_pane, state);
        self.internal.focused_pane = FocusedPane::Some {
            pane: new_pane,
            focus: Focus::Idle,
        };

        Some(new_pane)
    }

    pub fn swap(&mut self, a: &Pane, b: &Pane) {
        self.internal.layout.update(&|node| match node {
            Node::Split { .. } => {}
            Node::Pane(pane) => {
                if pane == a {
                    *node = Node::Pane(*b);
                } else if pane == b {
                    *node = Node::Pane(*a);
                }
            }
        });
    }

    pub fn close(&mut self, pane: &Pane) -> Option<T> {
        if let Some(sibling) = self.internal.layout.remove(pane) {
            self.internal.focused_pane = FocusedPane::Some {
                pane: sibling,
                focus: Focus::Idle,
            };

            self.panes.remove(pane)
        } else {
            None
        }
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

impl Node {
    fn find(&mut self, pane: &Pane) -> Option<&mut Node> {
        match self {
            Node::Split { a, b, .. } => {
                if let Some(node) = a.find(pane) {
                    Some(node)
                } else {
                    b.find(pane)
                }
            }
            Node::Pane(p) => {
                if p == pane {
                    Some(self)
                } else {
                    None
                }
            }
        }
    }

    fn split(&mut self, kind: Split, new_pane: Pane) {
        *self = Node::Split {
            kind,
            ratio: 500_000,
            a: Box::new(self.clone()),
            b: Box::new(Node::Pane(new_pane)),
        };
    }

    fn update(&mut self, f: &impl Fn(&mut Node)) {
        match self {
            Node::Split { a, b, .. } => {
                a.update(f);
                b.update(f);
            }
            _ => {}
        }

        f(self);
    }

    fn remove(&mut self, pane: &Pane) -> Option<Pane> {
        match self {
            Node::Split { a, b, .. } => {
                if a.pane() == Some(*pane) {
                    *self = *b.clone();
                    Some(self.first_pane())
                } else if b.pane() == Some(*pane) {
                    *self = *a.clone();
                    Some(self.first_pane())
                } else {
                    a.remove(pane).or_else(|| b.remove(pane))
                }
            }
            Node::Pane(_) => None,
        }
    }

    pub fn regions(&self, size: Size) -> HashMap<Pane, Rectangle> {
        let mut regions = HashMap::new();

        self.compute_regions(
            &Rectangle {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
            &mut regions,
        );

        regions
    }

    fn pane(&self) -> Option<Pane> {
        match self {
            Node::Split { .. } => None,
            Node::Pane(pane) => Some(*pane),
        }
    }

    fn first_pane(&self) -> Pane {
        match self {
            Node::Split { a, .. } => a.first_pane(),
            Node::Pane(pane) => *pane,
        }
    }

    fn compute_regions(
        &self,
        current: &Rectangle,
        regions: &mut HashMap<Pane, Rectangle>,
    ) {
        match self {
            Node::Split { kind, ratio, a, b } => {
                let ratio = *ratio as f32 / 1_000_000.0;
                let (region_a, region_b) = kind.apply(current, ratio);

                a.compute_regions(&region_a, regions);
                b.compute_regions(&region_b, regions);
            }
            Node::Pane(pane) => {
                let _ = regions.insert(*pane, *current);
            }
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Split {
    Horizontal,
    Vertical,
}

impl Split {
    fn apply(
        &self,
        rectangle: &Rectangle,
        ratio: f32,
    ) -> (Rectangle, Rectangle) {
        match self {
            Split::Horizontal => {
                let width_left = rectangle.width * ratio;
                let width_right = rectangle.width - width_left;

                (
                    Rectangle {
                        width: width_left,
                        ..*rectangle
                    },
                    Rectangle {
                        x: rectangle.x + width_left,
                        width: width_right,
                        ..*rectangle
                    },
                )
            }
            Split::Vertical => {
                let height_top = rectangle.height * ratio;
                let height_bottom = rectangle.height - height_top;

                (
                    Rectangle {
                        height: height_top,
                        ..*rectangle
                    },
                    Rectangle {
                        y: rectangle.y + height_top,
                        height: height_bottom,
                        ..*rectangle
                    },
                )
            }
        }
    }
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
        content: &[(Pane, Element<'_, Message, Self>)],
        dragging: Option<Pane>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<PaneGrid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer + 'static,
    Message: 'static,
{
    fn from(
        panes: PaneGrid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(panes)
    }
}
