use super::Renderer;

use ggez::graphics::{DrawParam, Rect};
use iced_native::{
    radio, text, Align, Column, Layout, Length, MouseCursor, Node, Point,
    Radio, Row, Text, Widget,
};

const SPRITE: Rect = Rect {
    x: 98.0,
    y: 28.0,
    w: 28.0,
    h: 28.0,
};

impl radio::Renderer for Renderer<'_>
where
    Self: text::Renderer,
{
    fn node<Message>(&mut self, radio: &Radio<Message>) -> Node {
        Row::<(), Self>::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(
                Column::new()
                    .width(Length::Units(SPRITE.w as u16))
                    .height(Length::Units(SPRITE.h as u16)),
            )
            .push(Text::new(&radio.label))
            .node(self)
    }

    fn draw<Message>(
        &mut self,
        radio: &Radio<Message>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> MouseCursor {
        let children: Vec<_> = layout.children().collect();

        let mut text = Text::new(&radio.label);

        if let Some(label_color) = radio.label_color {
            text = text.color(label_color);
        }

        text::Renderer::draw(self, &text, children[1]);

        let bounds = layout.bounds();
        let mouse_over = bounds.contains(cursor_position);

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

        if radio.is_selected {
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
