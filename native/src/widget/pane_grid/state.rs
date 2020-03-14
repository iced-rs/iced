use crate::{
    input::keyboard,
    pane_grid::{node::Node, Direction, Pane, Split},
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
                    last_pane: 0,
                    focused_pane: FocusedPane::None,
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

    pub fn adjacent_pane(
        &self,
        pane: &Pane,
        direction: Direction,
    ) -> Option<Pane> {
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
    last_pane: usize,
    focused_pane: FocusedPane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    None,
    Some { pane: Pane, focus: Focus },
}

impl Internal {
    pub fn focused(&self) -> FocusedPane {
        self.focused_pane
    }
    pub fn dragged(&self) -> Option<Pane> {
        match self.focused_pane {
            FocusedPane::Some {
                pane,
                focus: Focus::Dragging,
            } => Some(pane),
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

    pub fn focus(&mut self, pane: &Pane) {
        self.focused_pane = FocusedPane::Some {
            pane: *pane,
            focus: Focus::Idle,
        };
    }

    pub fn drag(&mut self, pane: &Pane) {
        self.focused_pane = FocusedPane::Some {
            pane: *pane,
            focus: Focus::Dragging,
        };
    }

    pub fn unfocus(&mut self) {
        self.focused_pane = FocusedPane::None;
    }

    pub fn hash_layout(&self, hasher: &mut Hasher) {
        use std::hash::Hash;

        self.layout.hash(hasher);
    }
}
