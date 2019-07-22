use super::Renderer;
use ggez::graphics::{DrawParam, Rect};
use iced::{checkbox, MouseCursor};

const SPRITE: Rect = Rect {
    x: 98.0,
    y: 0.0,
    w: 28.0,
    h: 28.0,
};

impl checkbox::Renderer for Renderer<'_> {
    fn draw(
        &mut self,
        cursor_position: iced::Point,
        bounds: iced::Rectangle<f32>,
        text_bounds: iced::Rectangle<f32>,
        is_checked: bool,
    ) -> MouseCursor {
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

        if is_checked {
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
