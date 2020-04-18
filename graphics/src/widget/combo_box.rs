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

        (
            if let Some(label) = selected {
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

                Primitive::Group {
                    primitives: vec![background, label],
                }
            } else {
                background
            },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
