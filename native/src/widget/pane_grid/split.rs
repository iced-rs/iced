use crate::Rectangle;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Split {
    Horizontal,
    Vertical,
}

impl Split {
    pub(super) fn apply(
        &self,
        rectangle: &Rectangle,
        ratio: f32,
        halved_spacing: f32,
    ) -> (Rectangle, Rectangle) {
        match self {
            Split::Horizontal => {
                let width_left =
                    (rectangle.width * ratio).round() - halved_spacing;
                let width_right = rectangle.width - width_left - halved_spacing;

                (
                    Rectangle {
                        width: width_left,
                        ..*rectangle
                    },
                    Rectangle {
                        x: rectangle.x + width_left + halved_spacing,
                        width: width_right,
                        ..*rectangle
                    },
                )
            }
            Split::Vertical => {
                let height_top =
                    (rectangle.height * ratio).round() - halved_spacing;
                let height_bottom =
                    rectangle.height - height_top - halved_spacing;

                (
                    Rectangle {
                        height: height_top,
                        ..*rectangle
                    },
                    Rectangle {
                        y: rectangle.y + height_top + halved_spacing,
                        height: height_bottom,
                        ..*rectangle
                    },
                )
            }
        }
    }
}
