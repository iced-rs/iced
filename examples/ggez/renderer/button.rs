use super::Renderer;
use ggez::graphics::{
    self, Align, Color, DrawParam, Rect, Scale, Text, TextFragment, WHITE,
};
use iced::{button, MouseCursor};

const LEFT: Rect = Rect {
    x: 0.0,
    y: 34.0,
    w: 6.0,
    h: 49.0,
};

const BACKGROUND: Rect = Rect {
    x: LEFT.w,
    y: LEFT.y,
    w: 1.0,
    h: LEFT.h,
};

const RIGHT: Rect = Rect {
    x: LEFT.h - LEFT.w,
    y: LEFT.y,
    w: LEFT.w,
    h: LEFT.h,
};

impl button::Renderer for Renderer<'_> {
    fn draw(
        &mut self,
        cursor_position: iced::Point,
        mut bounds: iced::Rectangle<f32>,
        state: &button::State,
        label: &str,
        class: button::Class,
    ) -> MouseCursor {
        let mouse_over = bounds.contains(cursor_position);

        let mut state_offset = 0.0;

        if mouse_over {
            if state.is_pressed() {
                bounds.y += 4.0;
                state_offset = RIGHT.x + RIGHT.w;
            } else {
                bounds.y -= 1.0;
            }
        }

        let class_index = match class {
            button::Class::Primary => 0,
            button::Class::Secondary => 1,
            button::Class::Positive => 2,
        };

        let width = self.spritesheet.width() as f32;
        let height = self.spritesheet.height() as f32;

        self.sprites.add(DrawParam {
            src: Rect {
                x: (LEFT.x + state_offset) / width,
                y: (LEFT.y + class_index as f32 * LEFT.h) / height,
                w: LEFT.w / width,
                h: LEFT.h / height,
            },
            dest: ggez::mint::Point2 {
                x: bounds.x,
                y: bounds.y,
            },
            ..DrawParam::default()
        });

        self.sprites.add(DrawParam {
            src: Rect {
                x: (BACKGROUND.x + state_offset) / width,
                y: (BACKGROUND.y + class_index as f32 * BACKGROUND.h) / height,
                w: BACKGROUND.w / width,
                h: BACKGROUND.h / height,
            },
            dest: ggez::mint::Point2 {
                x: bounds.x + LEFT.w,
                y: bounds.y,
            },
            scale: ggez::mint::Vector2 {
                x: bounds.width - LEFT.w - RIGHT.w,
                y: 1.0,
            },
            ..DrawParam::default()
        });

        self.sprites.add(DrawParam {
            src: Rect {
                x: (RIGHT.x + state_offset) / width,
                y: (RIGHT.y + class_index as f32 * RIGHT.h) / height,
                w: RIGHT.w / width,
                h: RIGHT.h / height,
            },
            dest: ggez::mint::Point2 {
                x: bounds.x + bounds.width - RIGHT.w,
                y: bounds.y,
            },
            ..DrawParam::default()
        });

        let mut text = Text::new(TextFragment {
            text: String::from(label),
            scale: Some(Scale { x: 20.0, y: 20.0 }),
            ..Default::default()
        });

        text.set_bounds(
            ggez::mint::Point2 {
                x: bounds.width,
                y: bounds.height,
            },
            Align::Center,
        );

        graphics::queue_text(
            self.context,
            &text,
            ggez::mint::Point2 {
                x: bounds.x,
                y: bounds.y + BACKGROUND.h / 4.0,
            },
            Some(if mouse_over {
                WHITE
            } else {
                Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                }
            }),
        );

        if mouse_over {
            MouseCursor::Pointer
        } else {
            MouseCursor::OutOfBounds
        }
    }
}
