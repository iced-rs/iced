//! Navigate an endless amount of content with a scrollbar.
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::scrollable;
use iced_native::{Background, Color, Rectangle, Vector};

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
    ) -> Option<scrollable::Scrollbar> {
        if content_bounds.height > bounds.height {
            let outer_width =
                scrollbar_width.max(scroller_width) + 2 * scrollbar_margin;

            let outer_bounds = Rectangle {
                x: bounds.x + bounds.width - outer_width as f32,
                y: bounds.y,
                width: outer_width as f32,
                height: bounds.height,
            };

            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(outer_width / 2 + scrollbar_width / 2),
                y: bounds.y,
                width: scrollbar_width as f32,
                height: bounds.height,
            };

            let ratio = bounds.height / content_bounds.height;
            let scroller_height = bounds.height * ratio;
            let y_offset = offset as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(outer_width / 2 + scroller_width / 2),
                y: scrollbar_bounds.y + y_offset,
                width: scroller_width as f32,
                height: scroller_height,
            };

            Some(scrollable::Scrollbar {
                outer_bounds,
                bounds: scrollbar_bounds,
                margin: scrollbar_margin,
                scroller: scrollable::Scroller {
                    bounds: scroller_bounds,
                },
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
                    offset: Vector::new(0, offset),
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

                let scroll = Primitive::Clip {
                    bounds,
                    offset: Vector::new(0, 0),
                    content: Box::new(Primitive::Group {
                        primitives: vec![scrollbar, scroller],
                    }),
                };

                Primitive::Group {
                    primitives: vec![clip, scroll],
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
