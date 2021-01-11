//! Navigate an endless amount of content with a scrollbar.
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::scrollable_hor;
use iced_native::{Background, Color, Rectangle, Vector};

pub use iced_native::scrollable_hor::State;
pub use iced_style::scrollable::{Scrollbar, Scroller, StyleSheet};

/// A widget that can horizontally display an infinite amount of content
/// with a scrollbar.
///
/// This is an alias of an `iced_native` scrollable with a default
/// `Renderer`.
pub type ScrollableHor<'a, Message, Backend> =
    iced_native::ScrollableHor<'a, Message, Renderer<Backend>>;

impl<B> scrollable_hor::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn iced_style::scrollable::StyleSheet>;

    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        scrollbar_height: u16,
        scrollbar_margin: u16,
        scroller_height: u16,
    ) -> Option<scrollable_hor::Scrollbar> {
        if content_bounds.width > bounds.width {
            let outer_height =
                scrollbar_height.max(scroller_height) + 2 * scrollbar_margin;

            let outer_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - outer_height as f32,
                width: bounds.width,
                height: outer_height as f32,
            };
//
            let scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height
                    - f32::from(outer_height / 2 + scrollbar_height / 2),
                width: bounds.width,
                height: scrollbar_height as f32,
            };

            let ratio = bounds.width / content_bounds.width;
            let scroller_width = bounds.width * ratio;
            let x_offset = offset as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: scrollbar_bounds.x + x_offset,
                y: bounds.y + bounds.height
                - f32::from(outer_height / 2 + scroller_height / 2),
                width: scroller_width,
                height: scroller_height as f32,
            };

            Some(scrollable_hor::Scrollbar {
                outer_bounds,
                bounds: scrollbar_bounds,
                margin: scrollbar_margin,
                scroller: scrollable_hor::Scroller {
                    bounds: scroller_bounds,
                },
            })
        } else {
            None
        }
    }

    fn draw(
        &mut self,
        state: &scrollable_hor::State,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        scrollbar: Option<scrollable_hor::Scrollbar>,
        offset: u32,
        style_sheet: &Self::Style,
        (content, mouse_interaction): Self::Output,
    ) -> Self::Output {
        (
            if let Some(scrollbar) = scrollbar {
                let clip = Primitive::Clip {
                    bounds,
                    offset: Vector::new(offset, 0),
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
