//! The state of a [`PaneGrid`].
//!
//! [`PaneGrid`]: super::PaneGrid
use crate::core::{Point, Size};
use crate::pane_grid::{
    Axis, Configuration, Direction, Edge, Node, Pane, Region, Split, Target,
};

use std::borrow::Cow;
use std::collections::BTreeMap;

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
/// [`PaneGrid`]: super::PaneGrid
/// [`PaneGrid::new`]: super::PaneGrid::new
#[derive(Debug, Clone)]
pub struct State<T> {
    /// The panes of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: super::PaneGrid
    pub panes: BTreeMap<Pane, T>,

    /// The internal state of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: super::PaneGrid
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
        let mut panes = BTreeMap::default();

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
    pub fn get(&self, pane: Pane) -> Option<&T> {
        self.panes.get(&pane)
    }

    /// Returns the internal state of the given [`Pane`] with mutability, if it
    /// exists.
    pub fn get_mut(&mut self, pane: Pane) -> Option<&mut T> {
        self.panes.get_mut(&pane)
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
    pub fn adjacent(&self, pane: Pane, direction: Direction) -> Option<Pane> {
        let regions = self.internal.layout.pane_regions(
            0.0,
            0.0,
            Size::new(4096.0, 4096.0),
        );

        let current_region = regions.get(&pane)?;

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
        pane: Pane,
        state: T,
    ) -> Option<(Pane, Split)> {
        self.split_node(axis, Some(pane), state, false)
    }

    /// Split a target [`Pane`] with a given [`Pane`] on a given [`Region`].
    ///
    /// Panes will be swapped by default for [`Region::Center`].
    pub fn split_with(&mut self, target: Pane, pane: Pane, region: Region) {
        match region {
            Region::Center => self.swap(pane, target),
            Region::Edge(edge) => match edge {
                Edge::Top => {
                    self.split_and_swap(Axis::Horizontal, target, pane, true);
                }
                Edge::Bottom => {
                    self.split_and_swap(Axis::Horizontal, target, pane, false);
                }
                Edge::Left => {
                    self.split_and_swap(Axis::Vertical, target, pane, true);
                }
                Edge::Right => {
                    self.split_and_swap(Axis::Vertical, target, pane, false);
                }
            },
        }
    }

    /// Drops the given [`Pane`] into the provided [`Target`].
    pub fn drop(&mut self, pane: Pane, target: Target) {
        match target {
            Target::Edge(edge) => self.move_to_edge(pane, edge),
            Target::Pane(target, region) => {
                self.split_with(target, pane, region);
            }
        }
    }

    fn split_node(
        &mut self,
        axis: Axis,
        pane: Option<Pane>,
        state: T,
        inverse: bool,
    ) -> Option<(Pane, Split)> {
        let node = if let Some(pane) = pane {
            self.internal.layout.find(pane)?
        } else {
            // Major node
            &mut self.internal.layout
        };

        let new_pane = {
            self.internal.last_id = self.internal.last_id.checked_add(1)?;

            Pane(self.internal.last_id)
        };

        let new_split = {
            self.internal.last_id = self.internal.last_id.checked_add(1)?;

            Split(self.internal.last_id)
        };

        if inverse {
            node.split_inverse(new_split, axis, new_pane);
        } else {
            node.split(new_split, axis, new_pane);
        }

        let _ = self.panes.insert(new_pane, state);
        let _ = self.internal.maximized.take();

        Some((new_pane, new_split))
    }

    fn split_and_swap(
        &mut self,
        axis: Axis,
        target: Pane,
        pane: Pane,
        swap: bool,
    ) {
        if let Some((state, _)) = self.close(pane) {
            if let Some((new_pane, _)) = self.split(axis, target, state) {
                // Ensure new node corresponds to original closed `Pane` for state continuity
                self.relabel(new_pane, pane);

                if swap {
                    self.swap(target, pane);
                }
            }
        }
    }

    /// Move [`Pane`] to an [`Edge`] of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: super::PaneGrid
    pub fn move_to_edge(&mut self, pane: Pane, edge: Edge) {
        match edge {
            Edge::Top => {
                self.split_major_node_and_swap(Axis::Horizontal, pane, true);
            }
            Edge::Bottom => {
                self.split_major_node_and_swap(Axis::Horizontal, pane, false);
            }
            Edge::Left => {
                self.split_major_node_and_swap(Axis::Vertical, pane, true);
            }
            Edge::Right => {
                self.split_major_node_and_swap(Axis::Vertical, pane, false);
            }
        }
    }

    fn split_major_node_and_swap(
        &mut self,
        axis: Axis,
        pane: Pane,
        inverse: bool,
    ) {
        if let Some((state, _)) = self.close(pane) {
            if let Some((new_pane, _)) =
                self.split_node(axis, None, state, inverse)
            {
                // Ensure new node corresponds to original closed `Pane` for state continuity
                self.relabel(new_pane, pane);
            }
        }
    }

    fn relabel(&mut self, target: Pane, label: Pane) {
        self.swap(target, label);

        let _ = self
            .panes
            .remove(&target)
            .and_then(|state| self.panes.insert(label, state));
    }

    /// Swaps the position of the provided panes in the [`State`].
    ///
    /// If you want to swap panes on drag and drop in your [`PaneGrid`], you
    /// will need to call this method when handling a [`DragEvent`].
    ///
    /// [`PaneGrid`]: super::PaneGrid
    /// [`DragEvent`]: super::DragEvent
    pub fn swap(&mut self, a: Pane, b: Pane) {
        self.internal.layout.update(&|node| match node {
            Node::Split { .. } => {}
            Node::Pane(pane) => {
                if *pane == a {
                    *node = Node::Pane(b);
                } else if *pane == b {
                    *node = Node::Pane(a);
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
    /// [`PaneGrid`]: super::PaneGrid
    /// [`ResizeEvent`]: super::ResizeEvent
    pub fn resize(&mut self, split: Split, ratio: f32) {
        let _ = self.internal.layout.resize(split, ratio);
    }

    /// Closes the given [`Pane`] and returns its internal state and its closest
    /// sibling, if it exists.
    pub fn close(&mut self, pane: Pane) -> Option<(T, Pane)> {
        if self.internal.maximized == Some(pane) {
            let _ = self.internal.maximized.take();
        }

        if let Some(sibling) = self.internal.layout.remove(pane) {
            self.panes.remove(&pane).map(|state| (state, sibling))
        } else {
            None
        }
    }

    /// Maximize the given [`Pane`]. Only this pane will be rendered by the
    /// [`PaneGrid`] until [`Self::restore()`] is called.
    ///
    /// [`PaneGrid`]: super::PaneGrid
    pub fn maximize(&mut self, pane: Pane) {
        self.internal.maximized = Some(pane);
    }

    /// Restore the currently maximized [`Pane`] to it's normal size. All panes
    /// will be rendered by the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: super::PaneGrid
    pub fn restore(&mut self) {
        let _ = self.internal.maximized.take();
    }

    /// Returns the maximized [`Pane`] of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: super::PaneGrid
    pub fn maximized(&self) -> Option<Pane> {
        self.internal.maximized
    }
}

/// The internal state of a [`PaneGrid`].
///
/// [`PaneGrid`]: super::PaneGrid
#[derive(Debug, Clone)]
pub struct Internal {
    layout: Node,
    last_id: usize,
    maximized: Option<Pane>,
}

impl Internal {
    /// Initializes the [`Internal`] state of a [`PaneGrid`] from a
    /// [`Configuration`].
    ///
    /// [`PaneGrid`]: super::PaneGrid
    pub fn from_configuration<T>(
        panes: &mut BTreeMap<Pane, T>,
        content: Configuration<T>,
        next_id: usize,
    ) -> Self {
        let (layout, last_id) = match content {
            Configuration::Split { axis, ratio, a, b } => {
                let Internal {
                    layout: a,
                    last_id: next_id,
                    ..
                } = Self::from_configuration(panes, *a, next_id);

                let Internal {
                    layout: b,
                    last_id: next_id,
                    ..
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

        Self {
            layout,
            last_id,
            maximized: None,
        }
    }

    pub(super) fn layout(&self) -> Cow<'_, Node> {
        match self.maximized {
            Some(pane) => Cow::Owned(Node::Pane(pane)),
            None => Cow::Borrowed(&self.layout),
        }
    }

    pub(super) fn maximized(&self) -> Option<Pane> {
        self.maximized
    }
}

/// The current action of a [`PaneGrid`].
///
/// [`PaneGrid`]: super::PaneGrid
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Action {
    /// The [`PaneGrid`] is idle.
    ///
    /// [`PaneGrid`]: super::PaneGrid
    #[default]
    Idle,
    /// A [`Pane`] in the [`PaneGrid`] is being dragged.
    ///
    /// [`PaneGrid`]: super::PaneGrid
    Dragging {
        /// The [`Pane`] being dragged.
        pane: Pane,
        /// The starting [`Point`] of the drag interaction.
        origin: Point,
    },
    /// A [`Split`] in the [`PaneGrid`] is being dragged.
    ///
    /// [`PaneGrid`]: super::PaneGrid
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
