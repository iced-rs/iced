use super::Renderer;

use ggez::graphics::{DrawParam, Rect};
use iced_native::{
    checkbox, text, Align, Checkbox, Column, Layout, Length, MouseCursor, Node,
    Row, Text, Widget,
};

const SPRITE: Rect = Rect {
    x: 98.0,
    y: 0.0,
    w: 28.0,
    h: 28.0,
};

impl checkbox::Renderer for Renderer<'_>
where
    Self: text::Renderer,
{
    fn node<Message>(&mut self, checkbox: &Checkbox<Message>) -> Node {
        Row::<(), Self>::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(
                Column::new()
                    .width(Length::Units(SPRITE.w as u16))
                    .height(Length::Units(SPRITE.h as u16)),
            )
            .push(Text::new(&checkbox.label))
            .node(self)
    }

    fn draw<Message>(
        &mut self,
        checkbox: &Checkbox<Message>,
        layout: Layout<'_>,
        cursor_position: iced_native::Point,
    ) -> MouseCursor {
        let bounds = layout.bounds();
        let children: Vec<_> = layout.children().collect();
        let text_bounds = children[1].bounds();

        let mut text = Text::new(&checkbox.label);

        if let Some(label_color) = checkbox.label_color {
            text = text.color(label_color);
        }

        text::Renderer::draw(self, &text, children[1]);

        let mouse_over = bounds.contains(cursor_position)
            || text_bounds.contains(cursor_position);

        let width = self.spritesheet.width() as f32;
        let height = self.spritesheet.height() as f32;

        self.sprites.add(DrawParam {
            src: Rect {
                x: (SPRITE.x + (if mouse_over { SPRITE.w } else { 0.0 }))
                    / width,
                y: SPRITE.y / height,
                w: SPRITE.w / width,
                h: SPRITE.h / height,
            },
            dest: ggez::mint::Point2 {
                x: bounds.x,
                y: bounds.y,
            },
            ..DrawParam::default()
        });

        if checkbox.is_checked {
            self.sprites.add(DrawParam {
                src: Rect {
                    x: (SPRITE.x + SPRITE.w * 2.0) / width,
                    y: SPRITE.y / height,
                    w: SPRITE.w / width,
                    h: SPRITE.h / height,
                },
                dest: ggez::mint::Point2 {
                    x: bounds.x,
                    y: bounds.y,
                },
                ..DrawParam::default()
            });
        }

        if mouse_over {
            MouseCursor::Pointer
        } else {
            MouseCursor::OutOfBounds
        }
    }
}
