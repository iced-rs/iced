use crate::Rectangle;

/// A fixed reference line for the measurement of coordinates.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Axis {
    /// The horizontal axis: —
    Horizontal,
    /// The vertical axis: |
    Vertical,
}

impl Axis {
    pub(super) fn split(
        &self,
        rectangle: &Rectangle,
        ratio: f32,
        spacing: f32,
    ) -> (Rectangle, Rectangle) {
        match self {
            Axis::Horizontal => {
                let height_top =
                    (rectangle.height * ratio - spacing / 2.0).round();
                let height_bottom = rectangle.height - height_top - spacing;

                (
                    Rectangle {
                        height: height_top,
                        ..*rectangle
                    },
                    Rectangle {
                        y: rectangle.y + height_top + spacing,
                        height: height_bottom,
                        ..*rectangle
                    },
                )
            }
            Axis::Vertical => {
                let width_left =
                    (rectangle.width * ratio - spacing / 2.0).round();
                let width_right = rectangle.width - width_left - spacing;

                (
                    Rectangle {
                        width: width_left,
                        ..*rectangle
                    },
                    Rectangle {
                        x: rectangle.x + width_left + spacing,
                        width: width_right,
                        ..*rectangle
                    },
                )
            }
        }
    }
}
