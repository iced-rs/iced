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
use crate::backend::{self, Backend};
use crate::defaults;
use crate::{Primitive, Renderer};
use iced_native::mouse;
use iced_native::pane_grid;
use iced_native::text;
use iced_native::{
    Element, HorizontalAlignment, Layout, Point, Rectangle, Vector,
    VerticalAlignment,
};

pub use iced_native::pane_grid::{
    Axis, Configuration, Content, Direction, DragEvent, Focus, Pane,
    ResizeEvent, Split, State, TitleBar,
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
    B: Backend + backend::Text,
{
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Pane, Content<'_, Message, Self>)],
        dragging: Option<(Pane, Point)>,
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

                if let Some((dragging, origin)) = dragging {
                    if *id == dragging {
                        dragged_pane = Some((i, layout, origin));
                    }
                }

                primitive
            })
            .collect();

        let primitives = if let Some((index, layout, origin)) = dragged_pane {
            let pane = panes.remove(index);
            let bounds = layout.bounds();

            // TODO: Fix once proper layering is implemented.
            // This is a pretty hacky way to achieve layering.
            let clip = Primitive::Clip {
                bounds: Rectangle {
                    x: cursor_position.x - origin.x,
                    y: cursor_position.y - origin.y,
                    width: bounds.width + 0.5,
                    height: bounds.height + 0.5,
                },
                offset: Vector::new(0, 0),
                content: Box::new(Primitive::Translate {
                    translation: Vector::new(
                        cursor_position.x - bounds.x - origin.x,
                        cursor_position.y - bounds.y - origin.y,
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

    fn draw_pane<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style_sheet: &Self::Style,
        title_bar: Option<(&TitleBar<'_, Message, Self>, Layout<'_>)>,
        body: (&Element<'_, Message, Self>, Layout<'_>),
        cursor_position: Point,
    ) -> Self::Output {
        let style = style_sheet.style();
        let (body, body_layout) = body;

        let (body_primitive, body_interaction) =
            body.draw(self, defaults, body_layout, cursor_position, &bounds);

        let background = crate::widget::container::background(bounds, &style);

        if let Some((title_bar, title_bar_layout)) = title_bar {
            let show_controls = bounds.contains(cursor_position);
            let is_over_pick_area =
                title_bar.is_over_pick_area(title_bar_layout, cursor_position);

            let (title_bar_primitive, title_bar_interaction) = title_bar.draw(
                self,
                defaults,
                title_bar_layout,
                cursor_position,
                show_controls,
            );

            (
                Primitive::Group {
                    primitives: vec![
                        background.unwrap_or(Primitive::None),
                        title_bar_primitive,
                        body_primitive,
                    ],
                },
                if is_over_pick_area {
                    mouse::Interaction::Grab
                } else if title_bar_interaction > body_interaction {
                    title_bar_interaction
                } else {
                    body_interaction
                },
            )
        } else {
            (
                if let Some(background) = background {
                    Primitive::Group {
                        primitives: vec![background, body_primitive],
                    }
                } else {
                    body_primitive
                },
                body_interaction,
            )
        }
    }

    fn draw_title_bar<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style_sheet: &Self::Style,
        title: &str,
        title_size: u16,
        title_font: Self::Font,
        title_bounds: Rectangle,
        controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        cursor_position: Point,
    ) -> Self::Output {
        let style = style_sheet.style();

        let defaults = Self::Defaults {
            text: defaults::Text {
                color: style.text_color.unwrap_or(defaults.text.color),
            },
        };

        let background = crate::widget::container::background(bounds, &style);

        let (title_primitive, _) = text::Renderer::draw(
            self,
            &defaults,
            title_bounds,
            title,
            title_size,
            title_font,
            None,
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
        );

        if let Some((controls, controls_layout)) = controls {
            let (controls_primitive, controls_interaction) = controls.draw(
                self,
                &defaults,
                controls_layout,
                cursor_position,
                &bounds,
            );

            (
                Primitive::Group {
                    primitives: vec![
                        background.unwrap_or(Primitive::None),
                        title_primitive,
                        controls_primitive,
                    ],
                },
                controls_interaction,
            )
        } else {
            (
                if let Some(background) = background {
                    Primitive::Group {
                        primitives: vec![background, title_primitive],
                    }
                } else {
                    title_primitive
                },
                mouse::Interaction::default(),
            )
        }
    }
}
