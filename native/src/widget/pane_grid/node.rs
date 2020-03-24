use crate::{
    pane_grid::{Axis, Pane, Split},
    Rectangle, Size,
};

use std::collections::HashMap;

#[derive(Debug, Clone, Hash)]
pub enum Node {
    Split {
        id: Split,
        axis: Axis,
        ratio: u32,
        a: Box<Node>,
        b: Box<Node>,
    },
    Pane(Pane),
}

impl Node {
    pub fn find(&mut self, pane: &Pane) -> Option<&mut Node> {
        match self {
            Node::Split { a, b, .. } => {
                a.find(pane).or_else(move || b.find(pane))
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

    pub fn split(&mut self, id: Split, axis: Axis, new_pane: Pane) {
        *self = Node::Split {
            id,
            axis,
            ratio: 500_000,
            a: Box::new(self.clone()),
            b: Box::new(Node::Pane(new_pane)),
        };
    }

    pub fn update(&mut self, f: &impl Fn(&mut Node)) {
        match self {
            Node::Split { a, b, .. } => {
                a.update(f);
                b.update(f);
            }
            _ => {}
        }

        f(self);
    }

    pub fn resize(&mut self, split: &Split, percentage: f32) -> bool {
        match self {
            Node::Split {
                id, ratio, a, b, ..
            } => {
                if id == split {
                    *ratio = (percentage * 1_000_000.0).round() as u32;

                    true
                } else if a.resize(split, percentage) {
                    true
                } else {
                    b.resize(split, percentage)
                }
            }
            Node::Pane(_) => false,
        }
    }

    pub fn remove(&mut self, pane: &Pane) -> Option<Pane> {
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

    pub fn regions(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Pane, Rectangle> {
        let mut regions = HashMap::new();

        self.compute_regions(
            spacing / 2.0,
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

    pub fn splits(
        &self,
        spacing: f32,
        size: Size,
    ) -> HashMap<Split, (Axis, Rectangle, f32)> {
        let mut splits = HashMap::new();

        self.compute_splits(
            spacing / 2.0,
            &Rectangle {
                x: 0.0,
                y: 0.0,
                width: size.width,
                height: size.height,
            },
            &mut splits,
        );

        splits
    }

    pub fn pane(&self) -> Option<Pane> {
        match self {
            Node::Split { .. } => None,
            Node::Pane(pane) => Some(*pane),
        }
    }

    pub fn first_pane(&self) -> Pane {
        match self {
            Node::Split { a, .. } => a.first_pane(),
            Node::Pane(pane) => *pane,
        }
    }

    fn compute_regions(
        &self,
        halved_spacing: f32,
        current: &Rectangle,
        regions: &mut HashMap<Pane, Rectangle>,
    ) {
        match self {
            Node::Split {
                axis, ratio, a, b, ..
            } => {
                let ratio = *ratio as f32 / 1_000_000.0;
                let (region_a, region_b) =
                    axis.split(current, ratio, halved_spacing);

                a.compute_regions(halved_spacing, &region_a, regions);
                b.compute_regions(halved_spacing, &region_b, regions);
            }
            Node::Pane(pane) => {
                let _ = regions.insert(*pane, *current);
            }
        }
    }

    fn compute_splits(
        &self,
        halved_spacing: f32,
        current: &Rectangle,
        splits: &mut HashMap<Split, (Axis, Rectangle, f32)>,
    ) {
        match self {
            Node::Split {
                axis,
                ratio,
                a,
                b,
                id,
            } => {
                let ratio = *ratio as f32 / 1_000_000.0;
                let (region_a, region_b) =
                    axis.split(current, ratio, halved_spacing);

                let _ = splits.insert(*id, (*axis, *current, ratio));

                a.compute_splits(halved_spacing, &region_a, splits);
                b.compute_splits(halved_spacing, &region_b, splits);
            }
            Node::Pane(_) => {}
        }
    }
}
