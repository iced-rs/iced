use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::{
    mouse, Background, Color, Font, HorizontalAlignment, Point, Rectangle,
    VerticalAlignment,
};

pub use iced_native::ComboBox;

impl<B> iced_native::combo_box::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    const DEFAULT_PADDING: u16 = 5;

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        selected: Option<String>,
        text_size: u16,
        padding: u16,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let background = Primitive::Quad {
            bounds,
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_color: if is_mouse_over {
                Color::BLACK
            } else {
                [0.7, 0.7, 0.7].into()
            },
            border_width: 1,
            border_radius: 0,
        };

        let arrow_down = Primitive::Text {
            content: B::ARROW_DOWN_ICON.to_string(),
            font: B::ICON_FONT,
            size: bounds.height * 0.7,
            bounds: Rectangle {
                x: bounds.x + bounds.width - f32::from(padding) * 2.0,
                y: bounds.center_y(),
                ..bounds
            },
            color: Color::BLACK,
            horizontal_alignment: HorizontalAlignment::Right,
            vertical_alignment: VerticalAlignment::Center,
        };

        (
            Primitive::Group {
                primitives: if let Some(label) = selected {
                    let label = Primitive::Text {
                        content: label,
                        size: f32::from(text_size),
                        font: Font::Default,
                        color: Color::BLACK,
                        bounds: Rectangle {
                            x: bounds.x + f32::from(padding),
                            y: bounds.center_y(),
                            ..bounds
                        },
                        horizontal_alignment: HorizontalAlignment::Left,
                        vertical_alignment: VerticalAlignment::Center,
                    };

                    vec![background, label, arrow_down]
                } else {
                    vec![background, arrow_down]
                },
            },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
