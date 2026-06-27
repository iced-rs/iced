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
use crate::Element;

use crate::layout::{Limits, Node};
use crate::length;
use crate::widget;
use crate::{Alignment, Length, Padding, Size};

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

    fn pack<T>(&self, main: T, cross: T) -> (T, T) {
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
pub fn resolve<Message, Theme, Renderer>(
    axis: Axis,
    renderer: &Renderer,
    limits: &Limits,
    width: Length,
    height: Length,
    padding: Padding,
    spacing: f32,
    align_items: Alignment,
    items: &mut [Element<'_, Message, Theme, Renderer>],
    trees: &mut [widget::Tree],
) -> Node
where
    Renderer: crate::Renderer,
{
    let limits = limits.width(width).height(height).shrink(padding);
    let total_spacing = spacing * items.len().saturating_sub(1) as f32;
    let max_cross = axis.cross(limits.max());

    let (main_compress, cross_compress) = {
        let compression = limits.compression();
        axis.pack(compression.width, compression.height)
    };

    let compression = {
        let (compress_x, compress_y) = axis.pack(main_compress, false);
        Size::new(compress_x, compress_y)
    };

    let mut fill_main_sum = 0;
    let mut some_fill_cross = false;
    let mut some_fill_max = false;
    let mut some_fill_min = false;
    let mut min_total = 0.0;
    let mut min_factors = 0;
    let mut cross = 0.0;
    let mut available = axis.main(limits.max()) - total_spacing;

    let mut nodes: Vec<Node> = Vec::with_capacity(items.len());
    nodes.resize(items.len(), Node::default());

    #[derive(Debug, Clone, Copy)]
    struct Meta {
        main: Length,
        cross: Length,
        category: Category,
        resolved: bool,
    }

    #[derive(Debug, Clone, Copy)]
    enum Category {
        Static,
        CrossFluid,
        CrossFluidDeferred(f32),
        MainFluid,
    }

    let mut metas = Vec::with_capacity(items.len());

    // STATIC PASS
    // We lay out non-fluid elements in the main axis.
    // If we need to compress the cross axis, then we skip any of these elements
    // that are also fluid in the cross axis.
    for (i, child) in items.iter_mut().enumerate() {
        let size = child.as_widget().size();
        let (size_main, size_cross) = axis.pack(size.width, size.height);

        let fill_main_factor = size_main.fill_factor();
        let fill_cross_factor = size_cross.fill_factor();
        let main_is_static = main_compress || fill_main_factor == 0;

        let category = match (main_is_static, cross_compress, fill_cross_factor == 0) {
            (true, false, _) | (true, _, true) => Category::Static,
            (true, true, false) => {
                if let Length::Fixed(main) = size_main {
                    available -= main;
                    Category::CrossFluidDeferred(main)
                } else {
                    Category::CrossFluid
                }
            }
            (false, _, _) => Category::MainFluid,
        };

        let meta = Meta {
            main: size_main,
            cross: size_cross,
            resolved: false,
            category,
        };

        metas.push(meta);

        match meta.main {
            Length::Bounded {
                sizing: length::Sizing::Fill(_),
                bounds: length::Bounds::Min(min),
            }
            | Length::Fluid(length::Constraint::Min(min)) => {
                min_total += min;
                min_factors += fill_main_factor;
                some_fill_min = true;
            }
            Length::Bounded {
                sizing: length::Sizing::Fill(_),
                bounds: length::Bounds::Max(_),
            }
            | Length::Fluid(length::Constraint::Max) => {
                some_fill_max = true;
            }
            Length::Bounded {
                sizing: length::Sizing::Fill(_),
                bounds: length::Bounds::Both { .. },
            } => {
                some_fill_max = true;
                some_fill_min = true;
            }
            _ => {}
        }

        let Category::Static = meta.category else {
            fill_main_sum += fill_main_factor;
            some_fill_cross = some_fill_cross || fill_cross_factor != 0;
            continue;
        };

        let (max_width, max_height) = axis.pack(
            available,
            if !cross_compress || fill_cross_factor == 0 {
                max_cross
            } else {
                cross
            },
        );

        let child_limits =
            Limits::with_compression(Size::ZERO, Size::new(max_width, max_height), compression);

        let layout = child
            .as_widget_mut()
            .layout(&mut trees[i], renderer, &child_limits);

        let size = layout.size();

        available -= axis.main(size);
        cross = cross.max(axis.cross(size));
        nodes[i] = layout;
    }

    // CROSS FLUID PASS
    // If we must compress the cross axis and there are fluid elements in the
    // cross axis, we lay out any of these elements that are also non-fluid in
    // the main axis (i.e. the ones we deliberately skipped in the first pass).
    //
    // We use the maximum cross length obtained in the first pass as the maximum
    // cross limit.
    //
    // We can defer the layout of any elements that have a fixed size in the main axis,
    // allowing them to use the cross calculations of the next pass.
    if cross_compress && some_fill_cross {
        for (i, child) in items.iter_mut().enumerate() {
            let meta = metas[i];

            let Category::CrossFluid = meta.category else {
                continue;
            };

            let (max_width, max_height) =
                axis.pack(available, if cross_compress { cross } else { max_cross });

            let child_limits =
                Limits::with_compression(Size::ZERO, Size::new(max_width, max_height), compression);

            let layout = child
                .as_widget_mut()
                .layout(&mut trees[i], renderer, &child_limits);

            let size = layout.size();

            available -= axis.main(size);
            cross = cross.max(axis.cross(size));
            nodes[i] = layout;
        }
    }

    let mut remaining = available.max(0.0);

    // MIN / MAX PASSES
    // We lay out any elements that are either bounded or fluid with some min / max constraints.
    //
    // First, we lay out any elements with `Max` bounds to potentially free up space; then
    // another pass lays out elements with `Min` bounds.
    //
    // Since the remaining space may change mid-iteration, we need to revisit previous elements
    // until they stabilize. Still, `layout` is only called once per element.
    #[derive(Debug, Clone, Copy)]
    enum Stage {
        Max,
        Min,
    }

    let mut step = if main_compress {
        None
    } else if some_fill_max {
        Some(Stage::Max)
    } else if some_fill_min {
        Some(Stage::Min)
    } else {
        None
    };

    while let Some(stage) = step {
        let current = remaining;
        let (reserved_space, reserved_factors) = match stage {
            Stage::Max => (min_total, min_factors),
            Stage::Min => (0.0, 0),
        };

        for (i, child) in items.iter_mut().enumerate() {
            let meta = &mut metas[i];

            if meta.resolved {
                continue;
            }

            let fill_main_factor = meta.main.fill_factor();

            if fill_main_factor == 0 {
                continue;
            }

            let bounds = match stage {
                Stage::Max => match meta.main {
                    Length::Bounded {
                        bounds: bounds @ (length::Bounds::Max(_) | length::Bounds::Both { .. }),
                        ..
                    } => bounds,
                    Length::Fluid(length::Constraint::Max) => length::Bounds::Min(0.0),
                    _ => continue,
                },
                Stage::Min => match meta.main {
                    Length::Bounded {
                        bounds: bounds @ (length::Bounds::Min(_) | length::Bounds::Both { .. }),
                        ..
                    } => bounds,
                    Length::Fluid(length::Constraint::Min(min)) => length::Bounds::Min(min),
                    _ => continue,
                },
            };

            let max_available = (remaining - reserved_space) * fill_main_factor as f32
                / (fill_main_sum - reserved_factors) as f32;

            let max_available = if max_available.is_nan() {
                f32::INFINITY
            } else {
                max_available
            };

            let (min, max) = match bounds {
                length::Bounds::Max(max) => (0.0, max),
                length::Bounds::Both { min, max } => (min, max),
                length::Bounds::Min(min) => (min, max_available),
            };

            match stage {
                Stage::Max if max > max_available => continue,
                Stage::Min if min < max_available => continue,
                _ => {}
            }

            let min = min.min(remaining);
            let max = max.min(max_available).max(min);

            let (min_width, min_height) = axis.pack(min, 0.0);
            let (max_width, max_height) = axis.pack(
                max,
                if !cross_compress || meta.cross.fill_factor() == 0 {
                    max_cross
                } else {
                    cross
                },
            );

            let child_limits = Limits::with_compression(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
                compression,
            );

            let layout = child
                .as_widget_mut()
                .layout(&mut trees[i], renderer, &child_limits);

            cross = cross.max(axis.cross(layout.size()));
            remaining -= axis.main(layout.size());
            fill_main_sum -= fill_main_factor;
            nodes[i] = layout;
            meta.resolved = true;
        }

        if remaining == current {
            step = match stage {
                Stage::Max if some_fill_min => Some(Stage::Min),
                _ => None,
            };
        }
    }

    // MAIN FLUID PASS (conditional)
    // We lay out the elements that are fluid in the main axis.
    // We use the remaining space to evenly allocate space based on fill factors.
    if !main_compress {
        for (i, child) in items.iter_mut().enumerate() {
            let meta = &mut metas[i];

            if meta.resolved {
                continue;
            }

            let Category::MainFluid = meta.category else {
                continue;
            };

            let max_main = remaining * meta.main.fill_factor() as f32 / fill_main_sum as f32;

            let max_main = if max_main.is_nan() {
                f32::INFINITY
            } else {
                max_main
            };

            let min_main = if max_main.is_infinite() {
                0.0
            } else {
                max_main
            };

            let (min_width, min_height) = axis.pack(min_main, 0.0);
            let (max_width, max_height) = axis.pack(
                max_main,
                if !cross_compress || meta.cross.fill_factor() == 0 {
                    max_cross
                } else {
                    cross
                },
            );

            let child_limits = Limits::with_compression(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
                compression,
            );

            let layout = child
                .as_widget_mut()
                .layout(&mut trees[i], renderer, &child_limits);

            cross = cross.max(axis.cross(layout.size()));
            nodes[i] = layout;
        }
    }

    // CROSS FLUID DEFERRED PASS (conditional)
    // We lay out any elements that were deferred in the second pass.
    // These are elements that must be compressed in their cross axis and have
    // a fixed length in the main axis.
    if cross_compress && some_fill_cross {
        for (i, child) in items.iter_mut().enumerate() {
            let meta = metas[i];

            let Category::CrossFluidDeferred(main) = meta.category else {
                continue;
            };

            let (max_width, max_height) = axis.pack(main, cross);
            let child_limits = Limits::new(Size::ZERO, Size::new(max_width, max_height));

            let layout = child
                .as_widget_mut()
                .layout(&mut trees[i], renderer, &child_limits);

            let size = layout.size();

            cross = cross.max(axis.cross(size));
            nodes[i] = layout;
        }
    }

    let pad = axis.pack(padding.left, padding.top);
    let mut main = pad.0;

    let cross = match axis {
        Axis::Horizontal => limits.resolve_height(height, cross),
        Axis::Vertical => limits.resolve_width(width, cross),
    };

    // ALIGNMENT PASS
    // We align all the laid out nodes in the cross axis, if needed.
    for (i, node) in nodes.iter_mut().enumerate() {
        if i > 0 {
            main += spacing;
        }

        node.move_to_mut(axis.pack(main, pad.1));

        match axis {
            Axis::Horizontal => {
                node.align_mut(Alignment::Start, align_items, Size::new(0.0, cross));
            }
            Axis::Vertical => {
                node.align_mut(align_items, Alignment::Start, Size::new(cross, 0.0));
            }
        }

        main += axis.main(node.size());
    }

    let main = match axis {
        Axis::Horizontal => limits.resolve_width(width, main - pad.0),
        Axis::Vertical => limits.resolve_height(height, main - pad.0),
    };

    let size = Size::from(axis.pack(main, cross));

    Node::with_children(size.expand(padding), nodes)
}
