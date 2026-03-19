//! A virtualized list widget that only constructs and lays out visible items.
//!
//! Inspired by VS Code's Monaco editor approach: given a fixed item height and
//! total item count, only the items within the visible viewport are
//! instantiated. This keeps layout and draw costs at O(visible) instead of
//! O(total), enabling smooth scrolling through thousands of items.
//!
//! # Usage
//!
//! Place a [`VirtualList`] inside a [`Scrollable`]. The widget reports its
//! full virtual height (`item_count * item_height`) so the scrollbar works
//! correctly, but internally only builds [`Element`]s for the visible slice
//! plus a small overscan buffer.
//!
//! ```ignore
//! use iced::widget::{scrollable, virtual_list};
//!
//! scrollable(
//!     virtual_list(lines.len(), 20.0, |index| {
//!         text(&lines[index]).into()
//!     })
//! )
//! ```
//!
//! [`Scrollable`]: crate::Scrollable

use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Element, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Vector, Widget,
};

/// Number of extra items rendered above and below the viewport.
const OVERSCAN: usize = 3;

/// A virtualized vertical list.
///
/// Only items within (or near) the visible viewport are constructed and laid
/// out. The widget reports its full virtual height so a parent [`Scrollable`]
/// can display an accurate scrollbar.
///
/// [`Scrollable`]: crate::Scrollable
pub struct VirtualList<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    item_count: usize,
    item_height: f32,
    view_item: Box<dyn Fn(usize) -> Element<'a, Message, Theme, Renderer> + 'a>,
    width: Length,
    /// Visible children built during `layout()`.
    visible_children: Vec<Element<'a, Message, Theme, Renderer>>,
    /// Index of the first visible child.
    visible_start: usize,
}

impl<'a, Message, Theme, Renderer> VirtualList<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Creates a new [`VirtualList`].
    ///
    /// * `item_count` – total number of items.
    /// * `item_height` – fixed height of every item in logical pixels.
    /// * `view_item` – closure that produces the [`Element`] for a given index.
    pub fn new(
        item_count: usize,
        item_height: impl Into<Pixels>,
        view_item: impl Fn(usize) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        Self {
            item_count,
            item_height: item_height.into().0,
            view_item: Box::new(view_item),
            width: Length::Fill,
            visible_children: Vec::new(),
            visible_start: 0,
        }
    }

    /// Sets the width of the [`VirtualList`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Compute the visible range from the cached viewport state.
    fn visible_range(&self, state: &State) -> (usize, usize) {
        if self.item_count == 0 || self.item_height <= 0.0 {
            return (0, 0);
        }

        let start = (state.viewport_y / self.item_height).floor().max(0.0) as usize;
        let start = start.saturating_sub(OVERSCAN).min(self.item_count);

        let visible_count = (state.viewport_height.max(0.0) / self.item_height).ceil() as usize;
        let visible_count = visible_count.saturating_add(2 * OVERSCAN + 1);
        let end = start.saturating_add(visible_count).min(self.item_count);

        (start, end)
    }
}

/// Internal state persisted across frames via the widget tree.
#[derive(Debug, Clone)]
struct State {
    /// Top of the visible region in content-space (updated from the viewport
    /// rectangle the parent scrollable passes to `update` / `draw`).
    viewport_y: f32,
    /// Height of the visible region.
    viewport_height: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            viewport_y: 0.0,
            // A generous default so the first frame renders enough items even
            // before we know the actual viewport.
            viewport_height: 2000.0,
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VirtualList<'_, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    // Diff is deferred to `layout`, similar to `Responsive`.
    fn diff(&self, _tree: &mut Tree) {}

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<State>();
        let (start, end) = self.visible_range(state);

        // Build elements only for the visible slice.
        self.visible_start = start;
        self.visible_children = (start..end).map(|i| (self.view_item)(i)).collect();

        // Reconcile the widget tree with the new set of children.
        tree.diff_children(
            &self
                .visible_children
                .iter()
                .map(Element::as_widget)
                .collect::<Vec<_>>(),
        );

        // Layout each visible child with constrained height.
        let max_width = limits.max().width;
        let child_limits = layout::Limits::new(Size::ZERO, Size::new(max_width, self.item_height));

        let mut child_nodes = Vec::with_capacity(self.visible_children.len());
        for (i, (child, child_tree)) in self
            .visible_children
            .iter_mut()
            .zip(tree.children.iter_mut())
            .enumerate()
        {
            let mut node = child
                .as_widget_mut()
                .layout(child_tree, renderer, &child_limits);

            // Position the child at its absolute y offset within the virtual
            // content area.
            let y = (self.visible_start + i) as f32 * self.item_height;
            node = node.move_to(Point::new(0.0, y));

            child_nodes.push(node);
        }

        // The node's total height is the full virtual height so the parent
        // scrollable can compute the correct scrollbar.
        let total_height = self.item_count as f32 * self.item_height;
        let width = limits
            .resolve(self.width, Length::Shrink, Size::new(max_width, 0.0))
            .width;

        layout::Node::with_children(Size::new(width, total_height), child_nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();

        // The parent scrollable translates the viewport into content-space
        // coordinates: viewport.y is the top of the visible region relative
        // to our content origin.
        let new_viewport_y = (viewport.y - bounds.y).max(0.0);
        let new_viewport_height = viewport.height;

        // If the visible window changed, cache it and re-layout next frame.
        let old_start = (state.viewport_y / self.item_height).floor() as usize;
        let new_start = (new_viewport_y / self.item_height).floor() as usize;

        if old_start != new_start
            || (state.viewport_height - new_viewport_height).abs() > self.item_height * 0.5
        {
            state.viewport_y = new_viewport_y;
            state.viewport_height = new_viewport_height;
            shell.invalidate_layout();
        }

        // Forward events to visible children.
        for ((child, child_tree), child_layout) in self
            .visible_children
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                child_tree,
                event,
                child_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, child_tree), child_layout) in self
            .visible_children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, child_layout)| child_layout.bounds().intersects(viewport))
        {
            child.as_widget().draw(
                child_tree,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.visible_children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, child_tree), child_layout)| {
                child.as_widget().mouse_interaction(
                    child_tree,
                    child_layout,
                    cursor,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.visible_children
                .iter_mut()
                .zip(tree.children.iter_mut())
                .zip(layout.children())
                .for_each(|((child, child_tree), child_layout)| {
                    child
                        .as_widget_mut()
                        .operate(child_tree, child_layout, renderer, operation);
                });
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.visible_children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<VirtualList<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(list: VirtualList<'a, Message, Theme, Renderer>) -> Self {
        Self::new(list)
    }
}
