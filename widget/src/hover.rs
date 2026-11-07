//! Displays a widget on top of another one, only when the base widget is
//! hovered.

use crate::core;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{Meta, Operation};
use crate::core::window;
use crate::core::{Event, Length, Rectangle, Shell, Size, Vector, Widget};

/// A widget that displays another widget on top of it.
///
/// This works analogously to a [`stack`](crate::Stack), but it will only
/// display the layer on top when the cursor is over the base. It can be
/// useful for removing visual clutter.
pub struct Hover<W, V> {
    base: W,
    top: V,
    is_top_focused: bool,
    is_top_overlay_active: bool,
    is_hovered: bool,
}

impl<W, V> Hover<W, V> {
    /// Creates a new [`Hover`] widget with the given base and top widgets.
    pub fn new(base: W, top: V) -> Self {
        Self {
            base,
            top,
            is_top_focused: false,
            is_top_overlay_active: false,
            is_hovered: false,
        }
    }
}

impl<W, V> Meta for Hover<W, V> {}

impl<W, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Hover<W, V>
where
    Renderer: core::Renderer,
    W: Widget<Message, Theme, Renderer>,
    V: Widget<Message, Theme, Renderer>,
{
    fn tag(&self) -> tree::Tag {
        struct Tag;
        tree::Tag::of::<Tag>()
    }

    fn diff(&mut self, tree: &mut Tree) {
        if tree.children.len() != 2 {
            tree.children = vec![Tree::new(&self.base), Tree::new(&self.top)];
        }

        tree.children[0].diff(&mut self.base);
        tree.children[1].diff(&mut self.top);
    }

    fn size(&self) -> Size<Length> {
        self.base.size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.base.layout(&mut tree.children[0], renderer, limits);

        let base_size = tree.children[0].size;

        self.top.layout(
            &mut tree.children[1],
            renderer,
            &layout::Limits::new(Size::ZERO, base_size),
        );

        tree.size = base_size;
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(bounds) = layout.bounds().intersection(viewport) {
            let mut children = layout.iter(&tree.children);

            let (base_layout, base_tree) = children.next().unwrap();

            self.base.draw(
                base_tree,
                renderer,
                theme,
                style,
                base_layout,
                cursor,
                viewport,
            );

            if cursor.is_over(layout.bounds()) || self.is_top_focused || self.is_top_overlay_active
            {
                let (top_layout, top_tree) = children.next().unwrap();

                renderer.with_layer(bounds, |renderer| {
                    self.top.draw(
                        top_tree, renderer, theme, style, top_layout, cursor, viewport,
                    );
                });
            }
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut children = layout.iter_mut(&mut tree.children);

        let (base_layout, base_tree) = children.next().unwrap();
        let (top_layout, top_tree) = children.next().unwrap();

        self.base
            .operate(base_tree, base_layout, viewport, renderer, operation);
        self.top
            .operate(top_tree, top_layout, viewport, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut children = layout.iter_mut(&mut tree.children);

        let (base_layout, base_tree) = children.next().unwrap();
        let (top_layout, top_tree) = children.next().unwrap();

        let is_hovered = cursor.is_over(layout.bounds());

        if matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
            let mut count_focused = operation::focusable::count();

            self.top.operate(
                top_tree,
                top_layout,
                viewport,
                renderer,
                &mut operation::black_box(&mut count_focused),
            );

            self.is_top_focused = match count_focused.finish() {
                operation::Outcome::Some(count) => count.focused.is_some(),
                _ => false,
            };

            self.is_hovered = is_hovered;
        } else if is_hovered != self.is_hovered {
            shell.request_redraw();
        }

        let is_visible = is_hovered || self.is_top_focused || self.is_top_overlay_active;

        if matches!(
            event,
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::ButtonReleased(_))
        ) || is_visible
        {
            let redraw_request = shell.redraw_request();

            self.top.update(
                top_tree, event, top_layout, cursor, renderer, shell, viewport,
            );

            // Ignore redraw requests of invisible content
            if !is_visible {
                Shell::replace_redraw_request(shell, redraw_request);
            }

            if shell.is_event_captured() {
                return;
            }
        };

        self.base.update(
            base_tree,
            event,
            base_layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.iter(&tree.children).rev();

        let (top_layout, top_tree) = children.next().unwrap();
        let interaction = self
            .top
            .mouse_interaction(top_tree, top_layout, cursor, viewport, renderer);

        if interaction != mouse::Interaction::None {
            return interaction;
        }

        let (base_layout, base_tree) = children.next().unwrap();
        self.base
            .mouse_interaction(base_tree, base_layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.iter_mut(&mut tree.children);

        let (base_layout, base_tree) = children.next().unwrap();
        let (top_layout, top_tree) = children.next().unwrap();

        let base_overlays = self.base.overlay(
            base_tree,
            base_layout,
            renderer,
            viewport,
            translation,
            window,
        );
        let top_overlays = self.top.overlay(
            top_tree,
            top_layout,
            renderer,
            viewport,
            translation,
            window,
        );

        self.is_top_overlay_active = !top_overlays.is_empty();

        base_overlays.into_iter().chain(top_overlays).collect()
    }
}
