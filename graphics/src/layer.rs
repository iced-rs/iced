//! Draw and stack layers of graphical primitives.
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

/// An opacity group recorded in a [`Stack`].
#[derive(Debug, Clone, Copy)]
pub struct OpacityGroup {
    /// The opacity of the group, in the range `0.0..=1.0`.
    pub opacity: f32,
    /// The bounds of the group, already transformed by the current
    /// [`Transformation`] at the time the group was created.
    pub bounds: Rectangle,
    /// The parent group this group is nested in, if any.
    pub parent: Option<usize>,
}

/// A plan describing when opacity groups open and close while iterating the
/// layers of a [`Stack`] in order.
///
/// A renderer walks its layers together with [`OpacityPlan::steps`]; when a
/// group opens it must redirect drawing to an isolated target, and when it
/// closes it must composite that target at the group's opacity.
#[derive(Debug, Default)]
pub struct OpacityPlan {
    /// One entry per active layer, in layer order.
    pub steps: Vec<OpacityStep>,
    /// Groups still open after the last layer, to be closed in this order
    /// (innermost first).
    pub trailing: Vec<usize>,
}

/// The opacity groups that open and close right before a given layer is drawn.
#[derive(Debug, Default, Clone)]
pub struct OpacityStep {
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
    /// All opacity groups recorded this frame, indexed by group id.
    opacity_groups: Vec<OpacityGroup>,
    /// The innermost opacity group each layer slot belongs to, parallel to
    /// `layers` by slot index.
    layer_groups: Vec<Option<usize>>,
    /// The opacity groups currently open while recording, as a stack of ids.
    active_groups: Vec<usize>,
    /// For each open opacity scope, whether it actually created an isolated
    /// group (fully-opaque scopes are elided). Keeps `push_opacity` and
    /// `pop_opacity` balanced.
    opacity_isolated: Vec<bool>,
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
            opacity_groups: Vec::new(),
            layer_groups: vec![None],
            active_groups: Vec::new(),
            opacity_isolated: Vec::new(),
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
    fn current_opacity(&self) -> f32 {
        self.active_groups
            .iter()
            .map(|id| self.opacity_groups[*id].opacity)
            .product()
    }

    /// Pushes a new opacity group in the [`Stack`].
    ///
    /// A new layer is created for the group (like [`push_clip`]) and every layer
    /// drawn until the matching [`pop_opacity`] belongs to it. The renderer is
    /// expected to composite all of those layers into an isolated target and
    /// blend the result with `opacity`, so that overlapping primitives fade as a
    /// single group instead of independently.
    ///
    /// [`push_clip`]: Self::push_clip
    /// [`pop_opacity`]: Self::pop_opacity
    pub fn push_opacity(&mut self, opacity: f32, bounds: Rectangle) {
        // A fully-opaque group has no visible effect, so we skip isolating it
        // entirely and avoid the cost of an offscreen target.
        if opacity >= 1.0 {
            self.opacity_isolated.push(false);
            return;
        }

        let parent = self.active_groups.last().copied();
        let id = self.opacity_groups.len();

        self.opacity_groups.push(OpacityGroup {
            opacity: opacity.clamp(0.0, 1.0),
            bounds: bounds * self.transformation(),
            parent,
        });

        // The group must be active before `push_clip` so the freshly created
        // layer is tagged as belonging to it.
        self.active_groups.push(id);
        self.push_clip(bounds);
        self.opacity_isolated.push(true);
    }

    /// Pops the current opacity group from the [`Stack`].
    pub fn pop_opacity(&mut self) {
        if self.opacity_isolated.pop() == Some(true) {
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

        self.opacity_groups.clear();
        self.active_groups.clear();
        self.opacity_isolated.clear();
        if let Some(first) = self.layer_groups.first_mut() {
            *first = None;
        }
    }

    /// Returns the opacity groups recorded in the [`Stack`], indexed by group id.
    pub fn opacity_groups(&self) -> &[OpacityGroup] {
        &self.opacity_groups
    }

    /// Returns whether any opacity group was recorded in the [`Stack`].
    pub fn has_opacity(&self) -> bool {
        !self.opacity_groups.is_empty()
    }

    /// Returns the chain of opacity groups a group belongs to, outermost first.
    fn opacity_chain(&self, group: usize) -> Vec<usize> {
        let mut chain = Vec::new();
        let mut current = Some(group);

        while let Some(id) = current {
            chain.push(id);
            current = self.opacity_groups[id].parent;
        }

        chain.reverse();
        chain
    }

    /// Builds the [`OpacityPlan`] describing when opacity groups open and close
    /// as the active layers are iterated in order.
    ///
    /// Groups occupy contiguous layer ranges, so a group opens right before its
    /// first layer and closes once the walk leaves it.
    pub fn opacity_plan(&self) -> OpacityPlan {
        let mut plan = OpacityPlan::default();

        if self.opacity_groups.is_empty() {
            plan.steps
                .resize_with(self.active_count, OpacityStep::default);
            return plan;
        }

        let mut previous: Vec<usize> = Vec::new();

        for index in 0..self.active_count {
            let chain = match self.layer_groups[index] {
                Some(group) => self.opacity_chain(group),
                None => Vec::new(),
            };

            let common = previous
                .iter()
                .zip(chain.iter())
                .take_while(|(a, b)| a == b)
                .count();

            plan.steps.push(OpacityStep {
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
