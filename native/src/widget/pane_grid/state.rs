use crate::{
    pane_grid::{Axis, Configuration, Direction, Node, Pane, Split},
    Hasher, Point, Rectangle, Size,
};

use std::collections::HashMap;

/// The state of a [`PaneGrid`].
///
/// It keeps track of the state of each [`Pane`] and the position of each
/// [`Split`].
///
/// The [`State`] needs to own any mutable contents a [`Pane`] may need. This is
/// why this struct is generic over the type `T`. Values of this type are
/// provided to the view function of [`PaneGrid::new`] for displaying each
/// [`Pane`].
///
/// [`PaneGrid`]: struct.PaneGrid.html
/// [`PaneGrid::new`]: struct.PaneGrid.html#method.new
/// [`Pane`]: struct.Pane.html
/// [`Split`]: struct.Split.html
/// [`State`]: struct.State.html
#[derive(Debug, Clone)]
pub struct State<T> {
    pub(super) panes: HashMap<Pane, T>,
    pub(super) internal: Internal,
}

/// The current focus of a [`Pane`].
///
/// [`Pane`]: struct.Pane.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The [`Pane`] is just focused.
    ///
    /// [`Pane`]: struct.Pane.html
    Idle,

    /// The [`Pane`] is being dragged.
    ///
    /// [`Pane`]: struct.Pane.html
    Dragging,
}

impl<T> State<T> {
    /// Creates a new [`State`], initializing the first pane with the provided
    /// state.
    ///
    /// Alongside the [`State`], it returns the first [`Pane`] identifier.
    ///
    /// [`State`]: struct.State.html
    /// [`Pane`]: struct.Pane.html
    pub fn new(first_pane_state: T) -> (Self, Pane) {
        (
            Self::with_configuration(Configuration::Pane(first_pane_state)),
            Pane(0),
        )
    }

    /// Creates a new [`State`] with the given [`Configuration`].
    ///
    /// [`State`]: struct.State.html
    /// [`Configuration`]: enum.Configuration.html
    pub fn with_configuration(config: impl Into<Configuration<T>>) -> Self {
        let mut panes = HashMap::new();

        let (layout, last_id) =
            Self::distribute_content(&mut panes, config.into(), 0);

        State {
            panes,
            internal: Internal {
                layout,
                last_id,
                action: Action::Idle,
            },
        }
    }

    /// Returns the total amount of panes in the [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn len(&self) -> usize {
        self.panes.len()
    }

    /// Returns the internal state of the given [`Pane`], if it exists.
    ///
    /// [`Pane`]: struct.Pane.html
    pub fn get(&self, pane: &Pane) -> Option<&T> {
        self.panes.get(pane)
    }

    /// Returns the internal state of the given [`Pane`] with mutability, if it
    /// exists.
    ///
    /// [`Pane`]: struct.Pane.html
    pub fn get_mut(&mut self, pane: &Pane) -> Option<&mut T> {
        self.panes.get_mut(pane)
    }

    /// Returns an iterator over all the panes of the [`State`], alongside its
    /// internal state.
    ///
    /// [`State`]: struct.State.html
    pub fn iter(&self) -> impl Iterator<Item = (&Pane, &T)> {
        self.panes.iter()
    }

    /// Returns a mutable iterator over all the panes of the [`State`],
    /// alongside its internal state.
    ///
    /// [`State`]: struct.State.html
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Pane, &mut T)> {
        self.panes.iter_mut()
    }

    /// Returns the layout of the [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn layout(&self) -> &Node {
        &self.internal.layout
    }

    /// Returns the adjacent [`Pane`] of another [`Pane`] in the given
    /// direction, if there is one.
    ///
    /// [`Pane`]: struct.Pane.html
    pub fn adjacent(&self, pane: &Pane, direction: Direction) -> Option<Pane> {
        let regions = self
            .internal
            .layout
            .pane_regions(0.0, Size::new(4096.0, 4096.0));

        let current_region = regions.get(pane)?;

        let target = match direction {
            Direction::Left => {
                Point::new(current_region.x - 1.0, current_region.y + 1.0)
            }
            Direction::Right => Point::new(
                current_region.x + current_region.width + 1.0,
                current_region.y + 1.0,
            ),
            Direction::Up => {
                Point::new(current_region.x + 1.0, current_region.y - 1.0)
            }
            Direction::Down => Point::new(
                current_region.x + 1.0,
                current_region.y + current_region.height + 1.0,
            ),
        };

        let mut colliding_regions =
            regions.iter().filter(|(_, region)| region.contains(target));

        let (pane, _) = colliding_regions.next()?;

        Some(*pane)
    }

    /// Splits the given [`Pane`] into two in the given [`Axis`] and
    /// initializing the new [`Pane`] with the provided internal state.
    ///
    /// [`Pane`]: struct.Pane.html
    /// [`Axis`]: enum.Axis.html
    pub fn split(
        &mut self,
        axis: Axis,
        pane: &Pane,
        state: T,
    ) -> Option<(Pane, Split)> {
        let node = self.internal.layout.find(pane)?;

        let new_pane = {
            self.internal.last_id = self.internal.last_id.checked_add(1)?;

            Pane(self.internal.last_id)
        };

        let new_split = {
            self.internal.last_id = self.internal.last_id.checked_add(1)?;

            Split(self.internal.last_id)
        };

        node.split(new_split, axis, new_pane);

        let _ = self.panes.insert(new_pane, state);

        Some((new_pane, new_split))
    }

    /// Swaps the position of the provided panes in the [`State`].
    ///
    /// If you want to swap panes on drag and drop in your [`PaneGrid`], you
    /// will need to call this method when handling a [`DragEvent`].
    ///
    /// [`State`]: struct.State.html
    /// [`PaneGrid`]: struct.PaneGrid.html
    /// [`DragEvent`]: struct.DragEvent.html
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

    /// Resizes two panes by setting the position of the provided [`Split`].
    ///
    /// The ratio is a value in [0, 1], representing the exact position of a
    /// [`Split`] between two panes.
    ///
    /// If you want to enable resize interactions in your [`PaneGrid`], you will
    /// need to call this method when handling a [`ResizeEvent`].
    ///
    /// [`Split`]: struct.Split.html
    /// [`PaneGrid`]: struct.PaneGrid.html
    /// [`ResizeEvent`]: struct.ResizeEvent.html
    pub fn resize(&mut self, split: &Split, ratio: f32) {
        let _ = self.internal.layout.resize(split, ratio);
    }

    /// Closes the given [`Pane`] and returns its internal state and its closest
    /// sibling, if it exists.
    ///
    /// [`Pane`]: struct.Pane.html
    pub fn close(&mut self, pane: &Pane) -> Option<(T, Pane)> {
        if let Some(sibling) = self.internal.layout.remove(pane) {
            self.panes.remove(pane).map(|state| (state, sibling))
        } else {
            None
        }
    }

    fn distribute_content(
        panes: &mut HashMap<Pane, T>,
        content: Configuration<T>,
        next_id: usize,
    ) -> (Node, usize) {
        match content {
            Configuration::Split { axis, ratio, a, b } => {
                let (a, next_id) = Self::distribute_content(panes, *a, next_id);
                let (b, next_id) = Self::distribute_content(panes, *b, next_id);

                (
                    Node::Split {
                        id: Split(next_id),
                        axis,
                        ratio,
                        a: Box::new(a),
                        b: Box::new(b),
                    },
                    next_id + 1,
                )
            }
            Configuration::Pane(state) => {
                let id = Pane(next_id);
                let _ = panes.insert(id, state);

                (Node::Pane(id), next_id + 1)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Internal {
    layout: Node,
    last_id: usize,
    action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Idle,
    Dragging { pane: Pane, origin: Point },
    Resizing { split: Split, axis: Axis },
}

impl Internal {
    pub fn picked_pane(&self) -> Option<(Pane, Point)> {
        match self.action {
            Action::Dragging { pane, origin, .. } => Some((pane, origin)),
            _ => None,
        }
    }

    pub fn picked_split(&self) -> Option<(Split, Axis)> {
        match self.action {
            Action::Resizing { split, axis, .. } => Some((split, axis)),
            _ => None,
        }
    }

    pub fn pane_regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Pane, Rectangle> {
        self.layout.pane_regions(spacing, size)
    }

    pub fn split_regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Split, (Axis, Rectangle, f32)> {
        self.layout.split_regions(spacing, size)
    }

    pub fn pick_pane(&mut self, pane: &Pane, origin: Point) {
        self.action = Action::Dragging {
            pane: *pane,
            origin,
        };
    }

    pub fn pick_split(&mut self, split: &Split, axis: Axis) {
        // TODO: Obtain `axis` from layout itself. Maybe we should implement
        // `Node::find_split`
        if self.picked_pane().is_some() {
            return;
        }

        self.action = Action::Resizing {
            split: *split,
            axis,
        };
    }

    pub fn idle(&mut self) {
        self.action = Action::Idle;
    }

    pub fn hash_layout(&self, hasher: &mut Hasher) {
        use std::hash::Hash;

        self.layout.hash(hasher);
    }
}
