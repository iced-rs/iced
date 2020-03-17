use crate::Rectangle;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub(super) fn split(
        &self,
        rectangle: &Rectangle,
        ratio: f32,
        halved_spacing: f32,
    ) -> (Rectangle, Rectangle) {
        match self {
            Axis::Horizontal => {
                let height_top = (rectangle.height * ratio).round();
                let height_bottom = rectangle.height - height_top;

                (
                    Rectangle {
                        height: height_top - halved_spacing,
                        ..*rectangle
                    },
                    Rectangle {
                        y: rectangle.y + height_top + halved_spacing,
                        height: height_bottom - halved_spacing,
                        ..*rectangle
                    },
                )
            }
            Axis::Vertical => {
                let width_left = (rectangle.width * ratio).round();
                let width_right = rectangle.width - width_left;

                (
                    Rectangle {
                        width: width_left - halved_spacing,
                        ..*rectangle
                    },
                    Rectangle {
                        x: rectangle.x + width_left + halved_spacing,
                        width: width_right - halved_spacing,
                        ..*rectangle
                    },
                )
            }
        }
    }
}
