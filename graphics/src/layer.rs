//! Draw and stack layers of graphical primitives.
use crate::core::renderer::GroupEffect;
use crate::core::{Rectangle, Transformation};

/// A layer of graphical primitives.
///
/// Layers normally dictate a set of primitives that are
/// rendered in a specific order.
pub trait Layer: Default {
    /// Creates a new [`Layer`] with the given bounds.
    fn with_bounds(bounds: Rectangle) -> Self;

    /// Returns the current bounds of the [`Layer`].
    fn bounds(&self) -> Rectangle;

    /// Flushes and settles any pending group of primitives in the [`Layer`].
    ///
    /// This will be called when a [`Layer`] is finished. It allows layers to efficiently
    /// record primitives together and defer grouping until the end.
    fn flush(&mut self);

    /// Resizes the [`Layer`] to the given bounds.
    fn resize(&mut self, bounds: Rectangle);

    /// Clears all the layers contents and resets its bounds.
    fn reset(&mut self);

    /// Returns the start level of the [`Layer`].
    ///
    /// A level is a "sublayer" index inside of a [`Layer`].
    ///
    /// A [`Layer`] may draw multiple primitive types in a certain order.
    /// The level represents the lowest index of the primitive types it
    /// contains.
    ///
    /// Two layers A and B can therefore be merged if they have the same bounds,
    /// and the end level of A is lower or equal than the start level of B.
    fn start(&self) -> usize;

    /// Returns the end level of the [`Layer`].
    fn end(&self) -> usize;

    /// Merges a [`Layer`] with the current one.
    fn merge(&mut self, _layer: &mut Self);

    /// Sets the effective opacity of the [`Layer`] (the product of the opacity
    /// groups it belongs to).
    ///
    /// Renderers that composite opacity groups can use this to include opacity
    /// in their damage tracking, so a changing opacity triggers a redraw even
    /// when the underlying primitives are unchanged. The default is a no-op.
    fn set_opacity(&mut self, _opacity: f32) {}
}

/// A composable group of layers recorded in a [`Stack`].
///
/// Its layers are isolated into an offscreen target and composited back with
/// [`effect`](Self::effect), enabling composable effects (opacity today; blur,
/// color filters, etc. in the future) on top of the same machinery.
#[derive(Debug, Clone, Copy)]
pub struct LayerGroup {
    /// The effect applied when compositing the group back onto its target.
    pub effect: GroupEffect,
    /// The bounds of the group, already transformed by the current
    /// [`Transformation`] at the time the group was created.
    pub bounds: Rectangle,
    /// The parent group this group is nested in, if any.
    pub parent: Option<usize>,
}

/// A plan describing when layer groups open and close while iterating the
/// layers of a [`Stack`] in order.
///
/// A renderer walks its layers together with [`GroupPlan::steps`]; when a group
/// opens it must redirect drawing to an isolated target, and when it closes it
/// must composite that target with the group's effect.
#[derive(Debug, Default)]
pub struct GroupPlan {
    /// One entry per active layer, in layer order.
    pub steps: Vec<GroupStep>,
    /// Groups still open after the last layer, to be closed in this order
    /// (innermost first).
    pub trailing: Vec<usize>,
}

/// The layer groups that open and close right before a given layer is drawn.
#[derive(Debug, Default, Clone)]
pub struct GroupStep {
    /// Groups to close before this layer, innermost first.
    pub closes: Vec<usize>,
    /// Groups to open before this layer, outermost first.
    pub opens: Vec<usize>,
}

/// A stack of layers used for drawing.
#[derive(Debug)]
pub struct Stack<T: Layer> {
    layers: Vec<T>,
    transformations: Vec<Transformation>,
    previous: Vec<usize>,
    current: usize,
    active_count: usize,
    /// All layer groups recorded this frame, indexed by group id.
    groups: Vec<LayerGroup>,
    /// The innermost group each layer slot belongs to, parallel to `layers` by
    /// slot index.
    layer_groups: Vec<Option<usize>>,
    /// The groups currently open while recording, as a stack of ids.
    active_groups: Vec<usize>,
    /// For each open group scope, whether it actually created an isolated group
    /// (no-op effects are elided). Keeps `push_group` and `pop_group` balanced.
    group_isolated: Vec<bool>,
}

impl<T: Layer> Stack<T> {
    /// Creates a new empty [`Stack`].
    pub fn new() -> Self {
        Self {
            layers: vec![T::default()],
            transformations: vec![Transformation::IDENTITY],
            previous: vec![],
            current: 0,
            active_count: 1,
            groups: Vec::new(),
            layer_groups: vec![None],
            active_groups: Vec::new(),
            group_isolated: Vec::new(),
        }
    }

    /// Returns a mutable reference to the current [`Layer`] of the [`Stack`], together with
    /// the current [`Transformation`].
    #[inline]
    pub fn current_mut(&mut self) -> (&mut T, Transformation) {
        let transformation = self.transformation();

        (&mut self.layers[self.current], transformation)
    }

    /// Returns the current [`Transformation`] of the [`Stack`].
    #[inline]
    pub fn transformation(&self) -> Transformation {
        self.transformations.last().copied().unwrap()
    }

    /// Pushes a new clipping region in the [`Stack`]; creating a new layer in the
    /// process.
    pub fn push_clip(&mut self, bounds: Rectangle) {
        self.previous.push(self.current);

        self.current = self.active_count;
        self.active_count += 1;

        let group = self.active_groups.last().copied();
        let opacity = self.current_opacity();
        let bounds = bounds * self.transformation();

        if self.current == self.layers.len() {
            self.layers.push(T::with_bounds(bounds));
            self.layer_groups.push(group);
        } else {
            self.layers[self.current].resize(bounds);
            self.layer_groups[self.current] = group;
        }

        self.layers[self.current].set_opacity(opacity);
    }

    /// Returns the effective opacity at the current point in recording: the
    /// product of every open opacity group's opacity.
    ///
    /// Only [`GroupEffect::Opacity`] groups contribute; other effects are
    /// transparent to opacity. Used to track opacity in a [`Layer`]'s damage.
    fn current_opacity(&self) -> f32 {
        self.active_groups
            .iter()
            .map(|id| match self.groups[*id].effect {
                GroupEffect::Opacity(opacity) => opacity,
            })
            .product()
    }

    /// Pushes a new layer group in the [`Stack`].
    ///
    /// A new layer is created for the group (like [`push_clip`]) and every layer
    /// drawn until the matching [`pop_group`] belongs to it. The renderer is
    /// expected to composite all of those layers into an isolated target and
    /// blend the result with `effect`, so that overlapping primitives are
    /// affected as a single group instead of independently.
    ///
    /// [`push_clip`]: Self::push_clip
    /// [`pop_group`]: Self::pop_group
    pub fn push_group(&mut self, effect: GroupEffect, bounds: Rectangle) {
        // An effect with no visible impact is skipped entirely, avoiding the cost
        // of an offscreen target.
        if effect.is_noop() {
            self.group_isolated.push(false);
            return;
        }

        let parent = self.active_groups.last().copied();
        let id = self.groups.len();

        self.groups.push(LayerGroup {
            effect,
            bounds: bounds * self.transformation(),
            parent,
        });

        // The group must be active before `push_clip` so the freshly created
        // layer is tagged as belonging to it.
        self.active_groups.push(id);
        self.push_clip(bounds);
        self.group_isolated.push(true);
    }

    /// Pops the current layer group from the [`Stack`].
    pub fn pop_group(&mut self) {
        if self.group_isolated.pop() == Some(true) {
            self.pop_clip();
            let _ = self.active_groups.pop();
        }
    }

    /// Pops the current clipping region from the [`Stack`] and restores the previous one.
    ///
    /// The current layer will be recorded for drawing.
    pub fn pop_clip(&mut self) {
        self.flush();

        self.current = self.previous.pop().unwrap();
    }

    /// Pushes a new [`Transformation`] in the [`Stack`].
    ///
    /// Future drawing operations will be affected by this new [`Transformation`] until
    /// it is popped using [`pop_transformation`].
    ///
    /// [`pop_transformation`]: Self::pop_transformation
    pub fn push_transformation(&mut self, transformation: Transformation) {
        self.transformations
            .push(self.transformation() * transformation);
    }

    /// Pops the current [`Transformation`] in the [`Stack`].
    pub fn pop_transformation(&mut self) {
        let _ = self.transformations.pop();
    }

    /// Returns an iterator over immutable references to the layers in the [`Stack`].
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.layers[..self.active_count].iter()
    }

    /// Returns the slice of layers in the [`Stack`].
    pub fn as_slice(&self) -> &[T] {
        &self.layers[..self.active_count]
    }

    /// Flushes and settles any primitives in the [`Stack`].
    pub fn flush(&mut self) {
        self.layers[self.current].flush();
    }

    /// Performs layer merging wherever possible.
    ///
    /// Flushes and settles any primitives in the [`Stack`].
    pub fn merge(&mut self) {
        self.flush();

        // These are the layers left to process
        let mut left = self.active_count;

        // There must be at least 2 or more layers to merge
        while left > 1 {
            // We set our target as the topmost layer left to process
            let mut current = left - 1;
            let mut target = &self.layers[current];
            let mut target_start = target.start();
            let mut target_index = current;

            // We scan downwards for a contiguous block of mergeable layer candidates
            while current > 0 {
                current -= 1;

                let candidate = &self.layers[current];
                let start = candidate.start();
                let end = candidate.end();

                // We skip empty layers
                if end == 0 {
                    continue;
                }

                // Candidate can be merged if primitive sublayers do not overlap with
                // previous targets, the clipping bounds match, and both layers
                // belong to the same opacity group (so isolated groups never leak
                // primitives in or out).
                if end > target_start
                    || candidate.bounds() != target.bounds()
                    || self.layer_groups[current] != self.layer_groups[target_index]
                {
                    break;
                }

                // Candidate is not empty and can be merged into
                target = candidate;
                target_start = start;
                target_index = current;
            }

            // We merge all the layers scanned into the target
            //
            // Since we use `target_index` instead of `current`, we
            // deliberately avoid merging into empty layers.
            //
            // If no candidates were mergeable, this is a no-op.
            let (head, tail) = self.layers.split_at_mut(target_index + 1);
            let layer = &mut head[target_index];

            for middle in &mut tail[0..left - target_index - 1] {
                layer.merge(middle);
            }

            // Empty layers found after the target can be skipped
            left = current;
        }
    }

    /// Clears the layers of the [`Stack`], allowing reuse.
    ///
    /// It resizes the base layer bounds to the `new_bounds`.
    ///
    /// This will normally keep layer allocations for future drawing operations.
    pub fn reset(&mut self, new_bounds: Rectangle) {
        for layer in self.layers[..self.active_count].iter_mut() {
            layer.reset();
        }

        self.layers[0].resize(new_bounds);
        self.current = 0;
        self.active_count = 1;
        self.previous.clear();

        self.groups.clear();
        self.active_groups.clear();
        self.group_isolated.clear();
        if let Some(first) = self.layer_groups.first_mut() {
            *first = None;
        }
    }

    /// Returns the layer groups recorded in the [`Stack`], indexed by group id.
    pub fn groups(&self) -> &[LayerGroup] {
        &self.groups
    }

    /// Returns whether any layer group was recorded in the [`Stack`].
    pub fn has_groups(&self) -> bool {
        !self.groups.is_empty()
    }

    /// Fuses adjacent sibling groups that can share a single isolated target and
    /// composite: same parent, equal and [batchable](GroupEffect::is_batchable)
    /// effect, adjacent layer ranges, and non-overlapping bounds.
    ///
    /// The later groups' layers are reassigned to the first, whose bounds grow to
    /// the union, so the renderer isolates and composites the whole run once.
    pub fn batch_groups(&mut self) {
        let count = self.groups.len();

        if count < 2 {
            return;
        }

        // A group that is some other group's parent is not a leaf.
        let mut is_parent = vec![false; count];
        for group in &self.groups {
            if let Some(parent) = group.parent {
                is_parent[parent] = true;
            }
        }

        // The (contiguous) layer range occupied by each group.
        let mut first = vec![usize::MAX; count];
        let mut last = vec![0usize; count];
        for index in 0..self.active_count {
            if let Some(group) = self.layer_groups[index] {
                if first[group] == usize::MAX {
                    first[group] = index;
                }
                last[group] = index;
            }
        }

        // Visit leaf groups in layer order so adjacency can be checked.
        let mut leaves: Vec<usize> = (0..count)
            .filter(|&group| !is_parent[group] && first[group] != usize::MAX)
            .collect();
        leaves.sort_by_key(|&group| first[group]);

        // The current run being accumulated: its representative group, the
        // bounds of its members (to test overlap), and the last layer consumed.
        let mut representative: Option<usize> = None;
        let mut members: Vec<Rectangle> = Vec::new();
        let mut run_last = 0;

        for group in leaves {
            let can_extend = representative.is_some_and(|rep| {
                let a = &self.groups[group];
                let b = &self.groups[rep];

                a.parent == b.parent
                    && a.effect == b.effect
                    && a.effect.is_batchable()
                    && first[group] == run_last + 1
                    && members
                        .iter()
                        .all(|bounds| bounds.intersection(&a.bounds).is_none())
            });

            if can_extend {
                let rep = representative.expect("Run has a representative");
                let bounds = self.groups[group].bounds;

                for index in first[group]..=last[group] {
                    if self.layer_groups[index] == Some(group) {
                        self.layer_groups[index] = Some(rep);
                    }
                }

                self.groups[rep].bounds = self.groups[rep].bounds.union(&bounds);
                members.push(bounds);
                run_last = last[group];
            } else {
                representative = Some(group);
                members.clear();
                members.push(self.groups[group].bounds);
                run_last = last[group];
            }
        }
    }

    /// Returns the chain of groups a group belongs to, outermost first.
    fn group_chain(&self, group: usize) -> Vec<usize> {
        let mut chain = Vec::new();
        let mut current = Some(group);

        while let Some(id) = current {
            chain.push(id);
            current = self.groups[id].parent;
        }

        chain.reverse();
        chain
    }

    /// Builds the [`GroupPlan`] describing when layer groups open and close as
    /// the active layers are iterated in order.
    ///
    /// Groups occupy contiguous layer ranges, so a group opens right before its
    /// first layer and closes once the walk leaves it.
    pub fn group_plan(&self) -> GroupPlan {
        let mut plan = GroupPlan::default();

        if self.groups.is_empty() {
            plan.steps
                .resize_with(self.active_count, GroupStep::default);
            return plan;
        }

        let mut previous: Vec<usize> = Vec::new();

        for index in 0..self.active_count {
            let chain = match self.layer_groups[index] {
                Some(group) => self.group_chain(group),
                None => Vec::new(),
            };

            let common = previous
                .iter()
                .zip(chain.iter())
                .take_while(|(a, b)| a == b)
                .count();

            plan.steps.push(GroupStep {
                closes: previous[common..].iter().rev().copied().collect(),
                opens: chain[common..].to_vec(),
            });

            previous = chain;
        }

        plan.trailing = previous.iter().rev().copied().collect();
        plan
    }
}

impl<T: Layer> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}
