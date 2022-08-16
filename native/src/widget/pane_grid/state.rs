//! The state of a [`PaneGrid`].
//!
//! [`PaneGrid`]: crate::widget::PaneGrid
use crate::widget::pane_grid::{
    Axis, Configuration, Direction, Node, Pane, Split,
};
use crate::{Point, Rectangle, Size};

use std::collections::{BTreeMap, HashMap};

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
/// [`PaneGrid`]: crate::widget::PaneGrid
/// [`PaneGrid::new`]: crate::widget::PaneGrid::new
#[derive(Debug, Clone)]
pub struct State<T> {
    /// The panes of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    pub panes: HashMap<Pane, T>,

    /// The internal state of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    pub internal: Internal,
}

impl<T> State<T> {
    /// Creates a new [`State`], initializing the first pane with the provided
    /// state.
    ///
    /// Alongside the [`State`], it returns the first [`Pane`] identifier.
    pub fn new(first_pane_state: T) -> (Self, Pane) {
        (
            Self::with_configuration(Configuration::Pane(first_pane_state)),
            Pane(0),
        )
    }

    /// Creates a new [`State`] with the given [`Configuration`].
    pub fn with_configuration(config: impl Into<Configuration<T>>) -> Self {
        let mut panes = HashMap::new();

        let internal =
            Internal::from_configuration(&mut panes, config.into(), 0);

        State { panes, internal }
    }

    /// Returns the total amount of panes in the [`State`].
    pub fn len(&self) -> usize {
        self.panes.len()
    }

    /// Returns `true` if the amount of panes in the [`State`] is 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the internal state of the given [`Pane`], if it exists.
    pub fn get(&self, pane: &Pane) -> Option<&T> {
        self.panes.get(pane)
    }

    /// Returns the internal state of the given [`Pane`] with mutability, if it
    /// exists.
    pub fn get_mut(&mut self, pane: &Pane) -> Option<&mut T> {
        self.panes.get_mut(pane)
    }

    /// Returns an iterator over all the panes of the [`State`], alongside its
    /// internal state.
    pub fn iter(&self) -> impl Iterator<Item = (&Pane, &T)> {
        self.panes.iter()
    }

    /// Returns a mutable iterator over all the panes of the [`State`],
    /// alongside its internal state.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Pane, &mut T)> {
        self.panes.iter_mut()
    }

    /// Returns the layout of the [`State`].
    pub fn layout(&self) -> &Node {
        &self.internal.layout
    }

    /// Returns the adjacent [`Pane`] of another [`Pane`] in the given
    /// direction, if there is one.
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
    /// [`PaneGrid`]: crate::widget::PaneGrid
    /// [`DragEvent`]: crate::widget::pane_grid::DragEvent
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
    /// [`PaneGrid`]: crate::widget::PaneGrid
    /// [`ResizeEvent`]: crate::widget::pane_grid::ResizeEvent
    pub fn resize(&mut self, split: &Split, ratio: f32) {
        let _ = self.internal.layout.resize(split, ratio);
    }

    /// Closes the given [`Pane`] and returns its internal state and its closest
    /// sibling, if it exists.
    pub fn close(&mut self, pane: &Pane) -> Option<(T, Pane)> {
        if let Some(sibling) = self.internal.layout.remove(pane) {
            self.panes.remove(pane).map(|state| (state, sibling))
        } else {
            None
        }
    }
}

/// The internal state of a [`PaneGrid`].
///
/// [`PaneGrid`]: crate::widget::PaneGrid
#[derive(Debug, Clone)]
pub struct Internal {
    layout: Node,
    last_id: usize,
}

impl Internal {
    /// Initializes the [`Internal`] state of a [`PaneGrid`] from a
    /// [`Configuration`].
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    pub fn from_configuration<T>(
        panes: &mut HashMap<Pane, T>,
        content: Configuration<T>,
        next_id: usize,
    ) -> Self {
        let (layout, last_id) = match content {
            Configuration::Split { axis, ratio, a, b } => {
                let Internal {
                    layout: a,
                    last_id: next_id,
                } = Self::from_configuration(panes, *a, next_id);

                let Internal {
                    layout: b,
                    last_id: next_id,
                } = Self::from_configuration(panes, *b, next_id);

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
        };

        Self { layout, last_id }
    }
}

/// The current action of a [`PaneGrid`].
///
/// [`PaneGrid`]: crate::widget::PaneGrid
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// The [`PaneGrid`] is idle.
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    Idle,
    /// A [`Pane`] in the [`PaneGrid`] is being dragged.
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    Dragging {
        /// The [`Pane`] being dragged.
        pane: Pane,
        /// The starting [`Point`] of the drag interaction.
        origin: Point,
    },
    /// A [`Split`] in the [`PaneGrid`] is being dragged.
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    Resizing {
        /// The [`Split`] being dragged.
        split: Split,
        /// The [`Axis`] of the [`Split`].
        axis: Axis,
    },
}

impl Action {
    /// Returns the current [`Pane`] that is being dragged, if any.
    pub fn picked_pane(&self) -> Option<(Pane, Point)> {
        match *self {
            Action::Dragging { pane, origin, .. } => Some((pane, origin)),
            _ => None,
        }
    }

    /// Returns the current [`Split`] that is being dragged, if any.
    pub fn picked_split(&self) -> Option<(Split, Axis)> {
        match *self {
            Action::Resizing { split, axis, .. } => Some((split, axis)),
            _ => None,
        }
    }
}

impl Internal {
    /// Calculates the current [`Pane`] regions from the [`PaneGrid`] layout.
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    pub fn pane_regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> BTreeMap<Pane, Rectangle> {
        self.layout.pane_regions(spacing, size)
    }

    /// Calculates the current [`Split`] regions from the [`PaneGrid`] layout.
    ///
    /// [`PaneGrid`]: crate::widget::PaneGrid
    pub fn split_regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> BTreeMap<Split, (Axis, Rectangle, f32)> {
        self.layout.split_regions(spacing, size)
    }
}
