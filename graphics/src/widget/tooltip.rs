//! Decorate content and apply alignment.
use crate::backend::{self, Backend};
use crate::defaults::{self, Defaults};
use crate::{Primitive, Renderer, Vector};

use iced_native::container;
use iced_native::layout::{self, Layout};
use iced_native::{Element, Point, Rectangle, Size, Text};

/// An element decorating some content.
///
/// This is an alias of an `iced_native` tooltip with a default
/// `Renderer`.
pub type Tooltip<'a, Message, Backend> =
    iced_native::Tooltip<'a, Message, Renderer<Backend>>;

pub use iced_native::tooltip::Position;

impl<B> iced_native::tooltip::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    const DEFAULT_PADDING: u16 = 5;

    fn draw<Message>(
        &mut self,
        defaults: &Defaults,
        cursor_position: Point,
        content_layout: Layout<'_>,
        viewport: &Rectangle,
        content: &Element<'_, Message, Self>,
        tooltip: &Text<Self>,
        position: Position,
        style_sheet: &<Self as container::Renderer>::Style,
        gap: u16,
        padding: u16,
    ) -> Self::Output {
        let (content, mouse_interaction) = content.draw(
            self,
            &defaults,
            content_layout,
            cursor_position,
            viewport,
        );

        let bounds = content_layout.bounds();

        if bounds.contains(cursor_position) {
            use iced_native::Widget;

            let gap = f32::from(gap);
            let padding = f32::from(padding);
            let style = style_sheet.style();

            let defaults = Defaults {
                text: defaults::Text {
                    color: style.text_color.unwrap_or(defaults.text.color),
                },
            };

            let tooltip_layout = Widget::<(), Self>::layout(
                tooltip,
                self,
                &layout::Limits::new(Size::ZERO, viewport.size())
                    .pad(f32::from(padding)),
            );

            let tooltip_bounds = tooltip_layout.bounds();

            let x_center =
                bounds.x + (bounds.width - tooltip_bounds.width) / 2.0;

            let y_center =
                bounds.y + (bounds.height - tooltip_bounds.height) / 2.0;

            let offset = match position {
                Position::Top => Vector::new(
                    x_center,
                    bounds.y - tooltip_bounds.height - gap - padding,
                ),
                Position::Bottom => Vector::new(
                    x_center,
                    bounds.y + bounds.height + gap + padding,
                ),
                Position::Left => Vector::new(
                    bounds.x - tooltip_bounds.width - gap - padding,
                    y_center,
                ),
                Position::Right => Vector::new(
                    bounds.x + bounds.width + gap + padding,
                    y_center,
                ),
                Position::FollowCursor => Vector::new(
                    cursor_position.x,
                    cursor_position.y - tooltip_bounds.height,
                ),
            };

            let (tooltip, _) = Widget::<(), Self>::draw(
                tooltip,
                self,
                &defaults,
                Layout::with_offset(offset, &tooltip_layout),
                cursor_position,
                viewport,
            );

            let tooltip_bounds = Rectangle {
                x: offset.x - padding,
                y: offset.y - padding,
                width: tooltip_bounds.width + padding * 2.0,
                height: tooltip_bounds.height + padding * 2.0,
            };

            (
                Primitive::Group {
                    primitives: vec![
                        content,
                        Primitive::Clip {
                            bounds: *viewport,
                            offset: Vector::new(0, 0),
                            content: Box::new(
                                if let Some(background) =
                                    crate::container::background(
                                        tooltip_bounds,
                                        &style,
                                    )
                                {
                                    Primitive::Group {
                                        primitives: vec![background, tooltip],
                                    }
                                } else {
                                    tooltip
                                },
                            ),
                        },
                    ],
                },
                mouse_interaction,
            )
        } else {
            (content, mouse_interaction)
        }
    }
}
