use crate::{
    input::keyboard,
    pane_grid::{node::Node, Axis, Direction, Pane, Split},
    Hasher, Point, Rectangle, Size,
};

use std::collections::HashMap;

#[derive(Debug)]
pub struct State<T> {
    pub(super) panes: HashMap<Pane, T>,
    pub(super) internal: Internal,
    pub(super) modifiers: keyboard::ModifiersState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Idle,
    Dragging,
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
                    last_id: 0,
                    action: Action::Idle { focus: None },
                },
                modifiers: keyboard::ModifiersState::default(),
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

    pub fn iter(&self) -> impl Iterator<Item = (&Pane, &T)> {
        self.panes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Pane, &mut T)> {
        self.panes.iter_mut()
    }

    pub fn active(&self) -> Option<Pane> {
        match self.internal.action {
            Action::Idle { focus } => focus,
            _ => None,
        }
    }

    pub fn adjacent(&self, pane: &Pane, direction: Direction) -> Option<Pane> {
        let regions =
            self.internal.layout.regions(0.0, Size::new(4096.0, 4096.0));

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

    pub fn focus(&mut self, pane: &Pane) {
        self.internal.focus(pane);
    }

    pub fn split(&mut self, axis: Axis, pane: &Pane, state: T) -> Option<Pane> {
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
        self.focus(&new_pane);

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

    pub fn resize(&mut self, split: &Split, percentage: f32) {
        let _ = self.internal.layout.resize(split, percentage);
    }

    pub fn close(&mut self, pane: &Pane) -> Option<T> {
        if let Some(sibling) = self.internal.layout.remove(pane) {
            self.focus(&sibling);
            self.panes.remove(pane)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Internal {
    layout: Node,
    last_id: usize,
    action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Idle {
        focus: Option<Pane>,
    },
    Dragging {
        pane: Pane,
    },
    Resizing {
        split: Split,
        axis: Axis,
        focus: Option<Pane>,
    },
}

impl Action {
    pub fn focus(&self) -> Option<(Pane, Focus)> {
        match self {
            Action::Idle { focus } | Action::Resizing { focus, .. } => {
                focus.map(|pane| (pane, Focus::Idle))
            }
            Action::Dragging { pane } => Some((*pane, Focus::Dragging)),
        }
    }
}

impl Internal {
    pub fn action(&self) -> Action {
        self.action
    }

    pub fn picked_pane(&self) -> Option<Pane> {
        match self.action {
            Action::Dragging { pane } => Some(pane),
            _ => None,
        }
    }

    pub fn picked_split(&self) -> Option<(Split, Axis)> {
        match self.action {
            Action::Resizing { split, axis, .. } => Some((split, axis)),
            _ => None,
        }
    }

    pub fn regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Pane, Rectangle> {
        self.layout.regions(spacing, size)
    }

    pub fn splits(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Split, (Axis, Rectangle, f32)> {
        self.layout.splits(spacing, size)
    }

    pub fn focus(&mut self, pane: &Pane) {
        self.action = Action::Idle { focus: Some(*pane) };
    }

    pub fn pick_pane(&mut self, pane: &Pane) {
        self.action = Action::Dragging { pane: *pane };
    }

    pub fn pick_split(&mut self, split: &Split, axis: Axis) {
        // TODO: Obtain `axis` from layout itself. Maybe we should implement
        // `Node::find_split`
        if self.picked_pane().is_some() {
            return;
        }

        let focus = self.action.focus().map(|(pane, _)| pane);

        self.action = Action::Resizing {
            split: *split,
            axis,
            focus,
        };
    }

    pub fn drop_split(&mut self) {
        match self.action {
            Action::Resizing { focus, .. } => {
                self.action = Action::Idle { focus };
            }
            _ => {}
        }
    }

    pub fn unfocus(&mut self) {
        self.action = Action::Idle { focus: None };
    }

    pub fn hash_layout(&self, hasher: &mut Hasher) {
        use std::hash::Hash;

        self.layout.hash(hasher);
    }
}
