//! Navigate an endless amount of content with a scrollbar.
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::scrollable;
use iced_native::{layout::flex::Axis, Background, Color, Point, Rectangle, Size, Vector};

pub use iced_native::scrollable::State;
pub use iced_style::scrollable::{Scrollbar, Scroller, StyleSheet};

/// A widget that can vertically display an infinite amount of content
/// with a scrollbar.
///
/// This is an alias of an `iced_native` scrollable with a default
/// `Renderer`.
pub type Scrollable<'a, Message, Backend> =
    iced_native::Scrollable<'a, Message, Renderer<Backend>>;

impl<B> scrollable::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn iced_style::scrollable::StyleSheet>;

    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
        axis: Axis,
    ) -> Option<scrollable::Scrollbar> {
        let scrollbar_necessary = match axis {
            Axis::Horizontal => content_bounds.width > bounds.width,
            Axis::Vertical => content_bounds.height > bounds.height,
        };
        if scrollbar_necessary {
            let outer_width =
                scrollbar_width.max(scroller_width) + 2 * scrollbar_margin;

            let outer_bounds_top_left = match axis {
                Axis::Horizontal => Point {
                    x: bounds.x,
                    y: bounds.y + bounds.height - outer_width as f32,
                },
                Axis::Vertical => Point {
                    x: bounds.x + bounds.width - outer_width as f32,
                    y: bounds.y,
                },
            };

            let outer_bounds_size = match axis {
                Axis::Horizontal => Size {
                    width: bounds.width,
                    height: outer_width as f32,
                },
                Axis::Vertical => Size {
                    width: outer_width as f32,
                    height: bounds.height,
                },
            };

            let outer_bounds =
                Rectangle::new(outer_bounds_top_left, outer_bounds_size);

            let scrollbar_bounds_top_left = match axis {
                Axis::Horizontal => Point {
                    x: bounds.x,
                    y: bounds.y + bounds.height
                    - f32::from(outer_width / 2 + scrollbar_width / 2),
                },
                Axis::Vertical => Point {
                    x: bounds.x + bounds.width
                            - f32::from(outer_width / 2 + scrollbar_width / 2),
                    y: bounds.y,
                },
            };

            let scrollbar_bounds_size = match axis {
                Axis::Horizontal => Size {
                    width: bounds.width,
                    height: scrollbar_width as f32,
                },
                Axis::Vertical => Size {
                    width: scrollbar_width as f32,
                    height: bounds.height,
                },
            };
            let scrollbar_bounds = Rectangle::new(
                scrollbar_bounds_top_left,
                scrollbar_bounds_size,
            );

            let ratio = match axis {
                Axis::Horizontal => bounds.width / content_bounds.width,
                Axis::Vertical => bounds.height / content_bounds.height,
            };

            let scroller_height = match axis {
                Axis::Horizontal => bounds.width * ratio,
                Axis::Vertical => bounds.height * ratio,
            };

            let offset = offset as f32 * ratio;

            let scroller_bounds_top_left = match axis {
                Axis::Horizontal => Point {
                    x: scrollbar_bounds.x + offset,
                    y: bounds.y + bounds.height
                    - f32::from(outer_width / 2 + scroller_width / 2),
                },
                Axis::Vertical => Point {
                    x: bounds.x + bounds.width
                        - f32::from(outer_width / 2 + scroller_width / 2),
                    y: scrollbar_bounds.y + offset,
                },
            };

            let scroller_bounds_size = match axis {
                Axis::Horizontal => Size {
                    width: scroller_height,
                    height: scroller_width as f32,
                },
                Axis::Vertical => Size {
                    width: scroller_width as f32,
                    height: scroller_height,
                },
            };
            let scroller_bounds =
                Rectangle::new(scroller_bounds_top_left, scroller_bounds_size);

            Some(scrollable::Scrollbar {
                outer_bounds,
                bounds: scrollbar_bounds,
                margin: scrollbar_margin,
                scroller: scrollable::Scroller {
                    bounds: scroller_bounds,
                },
                axis,
            })
        } else {
            None
        }
    }

    fn draw(
        &mut self,
        state: &scrollable::State,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        scrollbar: Option<scrollable::Scrollbar>,
        offset: u32,
        style_sheet: &Self::Style,
        (content, mouse_interaction): Self::Output,
    ) -> Self::Output {
        (
            if let Some(scrollbar) = scrollbar {
                let clip = Primitive::Clip {
                    bounds,
                    offset: match scrollbar.axis {
                        Axis::Vertical => Vector::new(0, offset),
                        Axis::Horizontal => Vector::new(offset, 0),
                    },
                    content: Box::new(content),
                };

                let style = if state.is_scroller_grabbed() {
                    style_sheet.dragging()
                } else if is_mouse_over_scrollbar {
                    style_sheet.hovered()
                } else {
                    style_sheet.active()
                };

                let is_scrollbar_visible =
                    style.background.is_some() || style.border_width > 0.0;

                let scroller = if is_mouse_over
                    || state.is_scroller_grabbed()
                    || is_scrollbar_visible
                {
                    Primitive::Quad {
                        bounds: scrollbar.scroller.bounds,
                        background: Background::Color(style.scroller.color),
                        border_radius: style.scroller.border_radius,
                        border_width: style.scroller.border_width,
                        border_color: style.scroller.border_color,
                    }
                } else {
                    Primitive::None
                };

                let scrollbar = if is_scrollbar_visible {
                    Primitive::Quad {
                        bounds: scrollbar.bounds,
                        background: style
                            .background
                            .unwrap_or(Background::Color(Color::TRANSPARENT)),
                        border_radius: style.border_radius,
                        border_width: style.border_width,
                        border_color: style.border_color,
                    }
                } else {
                    Primitive::None
                };

                Primitive::Group {
                    primitives: vec![clip, scrollbar, scroller],
                }
            } else {
                content
            },
            if is_mouse_over_scrollbar || state.is_scroller_grabbed() {
                mouse::Interaction::Idle
            } else {
                mouse_interaction
            },
        )
    }
}
