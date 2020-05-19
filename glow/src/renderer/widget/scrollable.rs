use crate::{Primitive, Renderer};
use iced_native::{mouse, scrollable, Background, Color, Rectangle, Vector};

const SCROLLBAR_WIDTH: u16 = 10;
const SCROLLBAR_MARGIN: u16 = 2;

impl scrollable::Renderer for Renderer {
    type Style = Box<dyn iced_style::scrollable::StyleSheet>;

    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
    ) -> Option<scrollable::Scrollbar> {
        if content_bounds.height > bounds.height {
            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(SCROLLBAR_WIDTH + 2 * SCROLLBAR_MARGIN),
                y: bounds.y,
                width: f32::from(SCROLLBAR_WIDTH + 2 * SCROLLBAR_MARGIN),
                height: bounds.height,
            };

            let ratio = bounds.height / content_bounds.height;
            let scrollbar_height = bounds.height * ratio;
            let y_offset = offset as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: scrollbar_bounds.x + f32::from(SCROLLBAR_MARGIN),
                y: scrollbar_bounds.y + y_offset,
                width: scrollbar_bounds.width - f32::from(2 * SCROLLBAR_MARGIN),
                height: scrollbar_height,
            };

            Some(scrollable::Scrollbar {
                bounds: scrollbar_bounds,
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
                    style.background.is_some() || style.border_width > 0;

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
                        bounds: Rectangle {
                            x: scrollbar.bounds.x + f32::from(SCROLLBAR_MARGIN),
                            width: scrollbar.bounds.width
                                - f32::from(2 * SCROLLBAR_MARGIN),
                            ..scrollbar.bounds
                        },
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
