//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
//!
//! # Example
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/hecrj/iced/tree/0.1/examples/pane_grid
//! [`PaneGrid`]: type.PaneGrid.html
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::pane_grid;
use iced_native::{Element, Layout, Point, Rectangle, Vector};

pub use iced_native::pane_grid::{
    Axis, Direction, DragEvent, Focus, KeyPressEvent, Pane, ResizeEvent, Split,
    State,
};

/// A collection of panes distributed using either vertical or horizontal splits
/// to completely fill the space available.
///
/// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
///
/// This is an alias of an `iced_native` pane grid with an `iced_wgpu::Renderer`.
pub type PaneGrid<'a, Message, Backend> =
    iced_native::PaneGrid<'a, Message, Renderer<Backend>>;

impl<B> pane_grid::Renderer for Renderer<B>
where
    B: Backend,
{
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Pane, Element<'_, Message, Self>)],
        dragging: Option<Pane>,
        resizing: Option<Axis>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let pane_cursor_position = if dragging.is_some() {
            // TODO: Remove once cursor availability is encoded in the type
            // system
            Point::new(-1.0, -1.0)
        } else {
            cursor_position
        };

        let mut mouse_interaction = mouse::Interaction::default();
        let mut dragged_pane = None;

        let mut panes: Vec<_> = content
            .iter()
            .zip(layout.children())
            .enumerate()
            .map(|(i, ((id, pane), layout))| {
                let (primitive, new_mouse_interaction) =
                    pane.draw(self, defaults, layout, pane_cursor_position);

                if new_mouse_interaction > mouse_interaction {
                    mouse_interaction = new_mouse_interaction;
                }

                if Some(*id) == dragging {
                    dragged_pane = Some((i, layout));
                }

                primitive
            })
            .collect();

        let primitives = if let Some((index, layout)) = dragged_pane {
            let pane = panes.remove(index);
            let bounds = layout.bounds();

            // TODO: Fix once proper layering is implemented.
            // This is a pretty hacky way to achieve layering.
            let clip = Primitive::Clip {
                bounds: Rectangle {
                    x: cursor_position.x - bounds.width / 2.0,
                    y: cursor_position.y - bounds.height / 2.0,
                    width: bounds.width + 0.5,
                    height: bounds.height + 0.5,
                },
                offset: Vector::new(0, 0),
                content: Box::new(Primitive::Translate {
                    translation: Vector::new(
                        cursor_position.x - bounds.x - bounds.width / 2.0,
                        cursor_position.y - bounds.y - bounds.height / 2.0,
                    ),
                    content: Box::new(pane),
                }),
            };

            panes.push(clip);

            panes
        } else {
            panes
        };

        (
            Primitive::Group { primitives },
            if dragging.is_some() {
                mouse::Interaction::Grabbing
            } else if let Some(axis) = resizing {
                match axis {
                    Axis::Horizontal => mouse::Interaction::ResizingVertically,
                    Axis::Vertical => mouse::Interaction::ResizingHorizontally,
                }
            } else {
                mouse_interaction
            },
        )
    }
}
