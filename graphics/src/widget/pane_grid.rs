//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
//!
//! # Example
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/hecrj/iced/tree/0.2/examples/pane_grid
use crate::defaults;
use crate::{Backend, Color, Primitive, Renderer};
use iced_native::container;
use iced_native::mouse;
use iced_native::pane_grid;
use iced_native::{Element, Layout, Point, Rectangle, Vector};

pub use iced_native::pane_grid::{
    Axis, Configuration, Content, Direction, DragEvent, Node, Pane,
    ResizeEvent, Split, State, TitleBar,
};

pub use iced_style::pane_grid::{Line, StyleSheet};

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
    type Style = Box<dyn StyleSheet>;

    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Pane, Content<'_, Message, Self>)],
        dragging: Option<(Pane, Point)>,
        resizing: Option<(Axis, Rectangle, bool)>,
        layout: Layout<'_>,
        style_sheet: &<Self as pane_grid::Renderer>::Style,
        cursor_position: Point,
        viewport: &Rectangle,
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
                let (primitive, new_mouse_interaction) = pane.draw(
                    self,
                    defaults,
                    layout,
                    pane_cursor_position,
                    viewport,
                );

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

        let mut primitives = if let Some((index, layout, origin)) = dragged_pane
        {
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

        let (primitives, mouse_interaction) =
            if let Some((axis, split_region, is_picked)) = resizing {
                let highlight = if is_picked {
                    style_sheet.picked_split()
                } else {
                    style_sheet.hovered_split()
                };

                if let Some(highlight) = highlight {
                    primitives.push(Primitive::Quad {
                        bounds: match axis {
                            Axis::Horizontal => Rectangle {
                                x: split_region.x,
                                y: (split_region.y
                                    + (split_region.height - highlight.width)
                                        / 2.0)
                                    .round(),
                                width: split_region.width,
                                height: highlight.width,
                            },
                            Axis::Vertical => Rectangle {
                                x: (split_region.x
                                    + (split_region.width - highlight.width)
                                        / 2.0)
                                    .round(),
                                y: split_region.y,
                                width: highlight.width,
                                height: split_region.height,
                            },
                        },
                        background: highlight.color.into(),
                        border_radius: 0.0,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    });
                }

                (
                    primitives,
                    match axis {
                        Axis::Horizontal => {
                            mouse::Interaction::ResizingVertically
                        }
                        Axis::Vertical => {
                            mouse::Interaction::ResizingHorizontally
                        }
                    },
                )
            } else {
                (primitives, mouse_interaction)
            };

        (
            Primitive::Group { primitives },
            if dragging.is_some() {
                mouse::Interaction::Grabbing
            } else {
                mouse_interaction
            },
        )
    }

    fn draw_pane<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style_sheet: &<Self as container::Renderer>::Style,
        title_bar: Option<(&TitleBar<'_, Message, Self>, Layout<'_>)>,
        body: (&Element<'_, Message, Self>, Layout<'_>),
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output {
        let style = style_sheet.style();
        let (body, body_layout) = body;

        let (body_primitive, body_interaction) =
            body.draw(self, defaults, body_layout, cursor_position, viewport);

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
                viewport,
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
        style_sheet: &<Self as container::Renderer>::Style,
        content: (&Element<'_, Message, Self>, Layout<'_>),
        controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Self::Output {
        let style = style_sheet.style();
        let (title_content, title_layout) = content;

        let defaults = Self::Defaults {
            text: defaults::Text {
                color: style.text_color.unwrap_or(defaults.text.color),
            },
        };

        let background = crate::widget::container::background(bounds, &style);

        let (title_primitive, title_interaction) = title_content.draw(
            self,
            &defaults,
            title_layout,
            cursor_position,
            viewport,
        );

        if let Some((controls, controls_layout)) = controls {
            let (controls_primitive, controls_interaction) = controls.draw(
                self,
                &defaults,
                controls_layout,
                cursor_position,
                viewport,
            );

            (
                Primitive::Group {
                    primitives: vec![
                        background.unwrap_or(Primitive::None),
                        title_primitive,
                        controls_primitive,
                    ],
                },
                controls_interaction.max(title_interaction),
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
                title_interaction,
            )
        }
    }
}
