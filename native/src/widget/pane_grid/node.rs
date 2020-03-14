use crate::{
    pane_grid::{Axis, Pane},
    Rectangle, Size,
};

use std::collections::HashMap;

#[derive(Debug, Clone, Hash)]
pub enum Node {
    Split {
        id: usize,
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

    pub fn split(&mut self, id: usize, axis: Axis, new_pane: Pane) {
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
}
