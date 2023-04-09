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

    pub(super) fn split_line_bounds(
        &self,
        rectangle: Rectangle,
        ratio: f32,
        spacing: f32,
    ) -> Rectangle {
        match self {
            Axis::Horizontal => Rectangle {
                x: rectangle.x,
                y: (rectangle.y + rectangle.height * ratio - spacing / 2.0)
                    .round(),
                width: rectangle.width,
                height: spacing,
            },
            Axis::Vertical => Rectangle {
                x: (rectangle.x + rectangle.width * ratio - spacing / 2.0)
                    .round(),
                y: rectangle.y,
                width: spacing,
                height: rectangle.height,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    enum Case {
        Horizontal {
            overall_height: f32,
            spacing: f32,
            top_height: f32,
            bottom_y: f32,
            bottom_height: f32,
        },
        Vertical {
            overall_width: f32,
            spacing: f32,
            left_width: f32,
            right_x: f32,
            right_width: f32,
        },
    }

    #[test]
    fn split() {
        let cases = vec![
            // Even height, even spacing
            Case::Horizontal {
                overall_height: 10.0,
                spacing: 2.0,
                top_height: 4.0,
                bottom_y: 6.0,
                bottom_height: 4.0,
            },
            // Odd height, even spacing
            Case::Horizontal {
                overall_height: 9.0,
                spacing: 2.0,
                top_height: 4.0,
                bottom_y: 6.0,
                bottom_height: 3.0,
            },
            // Even height, odd spacing
            Case::Horizontal {
                overall_height: 10.0,
                spacing: 1.0,
                top_height: 5.0,
                bottom_y: 6.0,
                bottom_height: 4.0,
            },
            // Odd height, odd spacing
            Case::Horizontal {
                overall_height: 9.0,
                spacing: 1.0,
                top_height: 4.0,
                bottom_y: 5.0,
                bottom_height: 4.0,
            },
            // Even width, even spacing
            Case::Vertical {
                overall_width: 10.0,
                spacing: 2.0,
                left_width: 4.0,
                right_x: 6.0,
                right_width: 4.0,
            },
            // Odd width, even spacing
            Case::Vertical {
                overall_width: 9.0,
                spacing: 2.0,
                left_width: 4.0,
                right_x: 6.0,
                right_width: 3.0,
            },
            // Even width, odd spacing
            Case::Vertical {
                overall_width: 10.0,
                spacing: 1.0,
                left_width: 5.0,
                right_x: 6.0,
                right_width: 4.0,
            },
            // Odd width, odd spacing
            Case::Vertical {
                overall_width: 9.0,
                spacing: 1.0,
                left_width: 4.0,
                right_x: 5.0,
                right_width: 4.0,
            },
        ];
        for case in cases {
            match case {
                Case::Horizontal {
                    overall_height,
                    spacing,
                    top_height,
                    bottom_y,
                    bottom_height,
                } => {
                    let a = Axis::Horizontal;
                    let r = Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: 10.0,
                        height: overall_height,
                    };
                    let (top, bottom) = a.split(&r, 0.5, spacing);
                    assert_eq!(
                        top,
                        Rectangle {
                            height: top_height,
                            ..r
                        }
                    );
                    assert_eq!(
                        bottom,
                        Rectangle {
                            y: bottom_y,
                            height: bottom_height,
                            ..r
                        }
                    );
                }
                Case::Vertical {
                    overall_width,
                    spacing,
                    left_width,
                    right_x,
                    right_width,
                } => {
                    let a = Axis::Vertical;
                    let r = Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: overall_width,
                        height: 10.0,
                    };
                    let (left, right) = a.split(&r, 0.5, spacing);
                    assert_eq!(
                        left,
                        Rectangle {
                            width: left_width,
                            ..r
                        }
                    );
                    assert_eq!(
                        right,
                        Rectangle {
                            x: right_x,
                            width: right_width,
                            ..r
                        }
                    );
                }
            }
        }
    }
}
