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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_horizontal() {
        let a = Axis::Horizontal;
        // rectangle height, spacing, top height, bottom y, bottom height
        let cases = vec![
            // Even height, even spacing
            (10.0, 2.0, 4.0, 6.0, 4.0),
            // Odd height, even spacing
            (9.0, 2.0, 4.0, 6.0, 3.0),
            // Even height, odd spacing
            (10.0, 1.0, 5.0, 6.0, 4.0),
            // Odd height, odd spacing
            (9.0, 1.0, 4.0, 5.0, 4.0),
        ];
        for case in cases {
            let (h0, spacing, h1_top, y_bottom, h1_bottom) = case;
            let r = Rectangle {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: h0,
            };
            let (top, bottom) = a.split(&r, 0.5, spacing);
            assert_eq!(
                top,
                Rectangle {
                    height: h1_top,
                    ..r
                }
            );
            assert_eq!(
                bottom,
                Rectangle {
                    y: y_bottom,
                    height: h1_bottom,
                    ..r
                }
            );
        }
    }

    #[test]
    fn split_vertical() {
        let a = Axis::Vertical;
        // rectangle width, spacing, left width, right x, right width
        let cases = vec![
            // Even width, even spacing
            (10.0, 2.0, 4.0, 6.0, 4.0),
            // Odd width, even spacing
            (9.0, 2.0, 4.0, 6.0, 3.0),
            // Even width, odd spacing
            (10.0, 1.0, 5.0, 6.0, 4.0),
            // Odd width, odd spacing
            (9.0, 1.0, 4.0, 5.0, 4.0),
        ];
        for case in cases {
            let (w0, spacing, w1_left, x_right, w1_right) = case;
            let r = Rectangle {
                x: 0.0,
                y: 0.0,
                width: w0,
                height: 10.0,
            };
            let (left, right) = a.split(&r, 0.5, spacing);
            assert_eq!(
                left,
                Rectangle {
                    width: w1_left,
                    ..r
                }
            );
            assert_eq!(
                right,
                Rectangle {
                    x: x_right,
                    width: w1_right,
                    ..r
                }
            );
        }
    }
}
