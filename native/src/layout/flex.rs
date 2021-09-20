//! Distribute elements using a flex-based layout.
// This code is heavily inspired by the [`druid`] codebase.
//
// [`druid`]: https://github.com/xi-editor/druid
//
// Copyright 2018 The xi-editor Authors, Héctor Ramón
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::layout::{Limits, Node};
use crate::{Alignment, Element, Padding, Point, Size};

/// The main axis of a flex layout.
#[derive(Debug)]
pub enum Axis {
    /// The horizontal axis
    Horizontal,

    /// The vertical axis
    Vertical,
}

impl Axis {
    fn main(&self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn cross(&self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

    fn pack(&self, main: f32, cross: f32) -> (f32, f32) {
        match self {
            Axis::Horizontal => (main, cross),
            Axis::Vertical => (cross, main),
        }
    }
}

/// Computes the flex layout with the given axis and limits, applying spacing,
/// padding and alignment to the items as needed.
///
/// It returns a new layout [`Node`].
pub fn resolve<Message, Renderer>(
    axis: Axis,
    renderer: &Renderer,
    limits: &Limits,
    padding: Padding,
    spacing: f32,
    align_items: Alignment,
    items: &[Element<'_, Message, Renderer>],
) -> Node
where
    Renderer: crate::Renderer,
{
    let limits = limits.pad(padding);
    let total_spacing = spacing * items.len().saturating_sub(1) as f32;
    let max_cross = axis.cross(limits.max());

    let mut fill_sum = 0;
    let mut cross = axis.cross(limits.min()).max(axis.cross(limits.fill()));
    let mut available = axis.main(limits.max()) - total_spacing;

    let mut nodes: Vec<Node> = Vec::with_capacity(items.len());
    nodes.resize(items.len(), Node::default());

    if align_items == Alignment::Fill {
        let mut fill_cross = axis.cross(limits.min());

        items.iter().for_each(|child| {
            let cross_fill_factor = match axis {
                Axis::Horizontal => child.height(),
                Axis::Vertical => child.width(),
            }
            .fill_factor();

            if cross_fill_factor == 0 {
                let (max_width, max_height) = axis.pack(available, max_cross);

                let child_limits =
                    Limits::new(Size::ZERO, Size::new(max_width, max_height));

                let layout = child.layout(renderer, &child_limits);
                let size = layout.size();

                fill_cross = fill_cross.max(axis.cross(size));
            }
        });

        cross = fill_cross;
    }

    for (i, child) in items.iter().enumerate() {
        let fill_factor = match axis {
            Axis::Horizontal => child.width(),
            Axis::Vertical => child.height(),
        }
        .fill_factor();

        if fill_factor == 0 {
            let (min_width, min_height) = if align_items == Alignment::Fill {
                axis.pack(0.0, cross)
            } else {
                axis.pack(0.0, 0.0)
            };

            let (max_width, max_height) = if align_items == Alignment::Fill {
                axis.pack(available, cross)
            } else {
                axis.pack(available, max_cross)
            };

            let child_limits = Limits::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let layout = child.layout(renderer, &child_limits);
            let size = layout.size();

            available -= axis.main(size);

            if align_items != Alignment::Fill {
                cross = cross.max(axis.cross(size));
            }

            nodes[i] = layout;
        } else {
            fill_sum += fill_factor;
        }
    }

    let remaining = available.max(0.0);

    for (i, child) in items.iter().enumerate() {
        let fill_factor = match axis {
            Axis::Horizontal => child.width(),
            Axis::Vertical => child.height(),
        }
        .fill_factor();

        if fill_factor != 0 {
            let max_main = remaining * fill_factor as f32 / fill_sum as f32;
            let min_main = if max_main.is_infinite() {
                0.0
            } else {
                max_main
            };

            let (min_width, min_height) = if align_items == Alignment::Fill {
                axis.pack(min_main, cross)
            } else {
                axis.pack(min_main, axis.cross(limits.min()))
            };

            let (max_width, max_height) = if align_items == Alignment::Fill {
                axis.pack(max_main, cross)
            } else {
                axis.pack(max_main, max_cross)
            };

            let child_limits = Limits::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let layout = child.layout(renderer, &child_limits);

            if align_items != Alignment::Fill {
                cross = cross.max(axis.cross(layout.size()));
            }

            nodes[i] = layout;
        }
    }

    let pad = axis.pack(padding.left as f32, padding.top as f32);
    let mut main = pad.0;

    for (i, node) in nodes.iter_mut().enumerate() {
        if i > 0 {
            main += spacing;
        }

        let (x, y) = axis.pack(main, pad.1);

        node.move_to(Point::new(x, y));

        match axis {
            Axis::Horizontal => {
                node.align(
                    Alignment::Start,
                    align_items,
                    Size::new(0.0, cross),
                );
            }
            Axis::Vertical => {
                node.align(
                    align_items,
                    Alignment::Start,
                    Size::new(cross, 0.0),
                );
            }
        }

        let size = node.size();

        main += axis.main(size);
    }

    let (width, height) = axis.pack(main - pad.0, cross);
    let size = limits.resolve(Size::new(width, height));

    Node::with_children(size.pad(padding), nodes)
}
