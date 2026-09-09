use crate::core::{Point, Rectangle, Size};

/// The position of a popup (a popover or a tooltip) relative to the element
/// it is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The popup will appear on the side of the widget with the most
    /// available space.
    #[default]
    Auto,
    /// The popup will appear on the top of the widget.
    Top,
    /// The popup will appear on the bottom of the widget.
    Bottom,
    /// The popup will appear on the left of the widget.
    Left,
    /// The popup will appear on the right of the widget.
    Right,
    /// The popup will follow the cursor.
    FollowCursor,
}

impl Position {
    /// Computes the [`Rectangle`] that a popup occupies for a [`Position`].
    ///
    /// All the arguments are expressed in the same coordinate space: the
    /// space in which the popup is laid out. The `content_bounds` describe
    /// the element the popup is anchored to, `popup` is the size of the
    /// popup itself, and `gap` is the amount of space to leave around it.
    ///
    /// The `cursor_position` is only used when the [`Position`] is
    /// [`Position::FollowCursor`], and the `viewport` is only used when it is
    /// [`Position::Auto`] or when `snap_within_viewport` is `true`, which
    /// clamps the resolved rectangle into the `viewport`.
    pub fn resolve(
        &self,
        content_bounds: Rectangle,
        popup: Size,
        gap: f32,
        cursor_position: Point,
        viewport: Rectangle,
        snap_within_viewport: bool,
    ) -> Rectangle {
        let offset = match self {
            // The popup follows the cursor.
            Position::FollowCursor => {
                Point::new(cursor_position.x, cursor_position.y - popup.height)
            }
            // The popup is placed on a side of the base; `Auto` resolves to
            // the side with the most available space.
            Position::Auto => Side::with_most_available_space(content_bounds, viewport, popup, gap)
                .offset(content_bounds, popup, gap),
            Position::Top => Side::Top.offset(content_bounds, popup, gap),
            Position::Bottom => Side::Bottom.offset(content_bounds, popup, gap),
            Position::Left => Side::Left.offset(content_bounds, popup, gap),
            Position::Right => Side::Right.offset(content_bounds, popup, gap),
        };

        let mut rectangle = Rectangle {
            x: offset.x,
            y: offset.y,
            width: popup.width,
            height: popup.height,
        };

        if snap_within_viewport {
            if rectangle.x < viewport.x {
                rectangle.x = viewport.x;
            } else if viewport.x + viewport.width < rectangle.x + rectangle.width {
                rectangle.x = viewport.x + viewport.width - rectangle.width;
            }

            if rectangle.y < viewport.y {
                rectangle.y = viewport.y;
            } else if viewport.y + viewport.height < rectangle.y + rectangle.height {
                rectangle.y = viewport.y + viewport.height - rectangle.height;
            }
        }

        rectangle
    }
}

#[derive(Clone, Copy)]
enum Side {
    Top,
    Left,
    Right,
    Bottom,
}

impl Side {
    fn with_most_available_space(
        content_bounds: Rectangle,
        viewport: Rectangle,
        popup: Size,
        gap: f32,
    ) -> Side {
        [Side::Top, Side::Left, Side::Right, Side::Bottom]
            .into_iter()
            .max_by(|a, b| {
                a.available_space(content_bounds, viewport, popup, gap)
                    .total_cmp(&b.available_space(content_bounds, viewport, popup, gap))
            })
            .unwrap_or(Side::Bottom)
    }

    fn available_space(
        self,
        content_bounds: Rectangle,
        viewport: Rectangle,
        popup: Size,
        gap: f32,
    ) -> f32 {
        match self {
            Side::Top => content_bounds.y - viewport.y - popup.height - gap,
            Side::Bottom => {
                (viewport.y + viewport.height)
                    - (content_bounds.y + content_bounds.height)
                    - popup.height
                    - gap
            }
            Side::Left => content_bounds.x - viewport.x - popup.width - gap,
            Side::Right => {
                (viewport.x + viewport.width)
                    - (content_bounds.x + content_bounds.width)
                    - popup.width
                    - gap
            }
        }
    }

    fn offset(self, content_bounds: Rectangle, popup: Size, gap: f32) -> Point {
        let x_center = content_bounds.x + (content_bounds.width - popup.width) / 2.0;
        let y_center = content_bounds.y + (content_bounds.height - popup.height) / 2.0;

        match self {
            Side::Top => Point::new(x_center, content_bounds.y - popup.height - gap),
            Side::Bottom => Point::new(x_center, content_bounds.y + content_bounds.height + gap),
            Side::Left => Point::new(content_bounds.x - popup.width - gap, y_center),
            Side::Right => Point::new(content_bounds.x + content_bounds.width + gap, y_center),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Position;
    use crate::core::{Point, Rectangle, Size};

    /// Common inputs: a 50x50 base at the origin, an 80x80 popup, no gap, and a
    /// 1000x1000 viewport.
    fn inputs() -> (Rectangle, Size, f32, Point, Rectangle) {
        (
            Rectangle::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0)),
            Size::new(80.0, 80.0),
            0.0,
            Point::new(0.0, 0.0),
            Rectangle::new(Point::new(0.0, 0.0), Size::new(1000.0, 1000.0)),
        )
    }

    #[test]
    fn auto_prefers_the_side_with_the_most_space() {
        let (base, popup, gap, cursor, viewport) = inputs();

        // There is far more space below (950) than above (0); the tie between
        // "below" and "right" is broken in favour of `Bottom`.
        let rect = Position::Auto.resolve(base, popup, gap, cursor, viewport, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, 50.0), Size::new(80.0, 80.0)),
            "Auto should resolve to the side with the most space"
        );
    }

    #[test]
    fn auto_avoids_a_side_where_the_popup_does_not_fit() {
        let (base, _, gap, cursor, viewport) = inputs();
        // A popup taller than the space below the base (950), but one that
        // fits on the right.
        let popup = Size::new(80.0, 960.0);

        let rect = Position::Auto.resolve(base, popup, gap, cursor, viewport, false);

        // `Bottom` is avoided because the popup (960) does not fit below the
        // base (950); the popup is placed to the right instead, so it is not
        // snapped back over the base.
        assert_eq!(
            rect,
            Rectangle::new(Point::new(50.0, -455.0), Size::new(80.0, 960.0)),
            "Auto should avoid a side where the popup does not fit"
        );
    }

    #[test]
    fn top_places_the_popup_above_the_base() {
        let (base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Top.resolve(base, popup, gap, cursor, viewport, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, -80.0), Size::new(80.0, 80.0)),
            "Top should place the popup above the base"
        );
    }

    #[test]
    fn bottom_places_the_popup_below_the_base() {
        let (base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Bottom.resolve(base, popup, gap, cursor, viewport, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, 50.0), Size::new(80.0, 80.0)),
            "Bottom should place the popup below the base"
        );
    }

    #[test]
    fn left_places_the_popup_left_of_the_base() {
        let (base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Left.resolve(base, popup, gap, cursor, viewport, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-80.0, -15.0), Size::new(80.0, 80.0)),
            "Left should place the popup to the left of the base"
        );
    }

    #[test]
    fn right_places_the_popup_right_of_the_base() {
        let (base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Right.resolve(base, popup, gap, cursor, viewport, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(50.0, -15.0), Size::new(80.0, 80.0)),
            "Right should place the popup to the right of the base"
        );
    }

    #[test]
    fn follow_cursor_places_the_popup_at_the_cursor() {
        let (base, popup, gap, _, viewport) = inputs();
        let cursor = Point::new(200.0, 300.0);

        // The popup's bottom edge is placed at the cursor.
        let rect = Position::FollowCursor.resolve(base, popup, gap, cursor, viewport, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(200.0, 220.0), Size::new(80.0, 80.0)),
            "FollowCursor should place the popup at the cursor"
        );
    }

    #[test]
    fn snap_clamps_the_popup_into_the_viewport() {
        let (base, popup, gap, cursor, viewport) = inputs();

        // Without snapping, `Top` places the popup at (-15, -80), outside of
        // the viewport; with snapping, it is clamped to the viewport's
        // origin.
        let rect = Position::Top.resolve(base, popup, gap, cursor, viewport, true);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(0.0, 0.0), Size::new(80.0, 80.0)),
            "Snap should clamp the popup into the viewport"
        );
    }

    #[test]
    fn snap_shifts_a_popup_that_overflows_the_viewport() {
        let (_, popup, gap, cursor, viewport) = inputs();
        // A base flush with the right edge of the viewport (1000).
        let base = Rectangle::new(Point::new(950.0, 0.0), Size::new(50.0, 50.0));

        // Without snapping, `Right` places the popup at (1000, -15), beyond
        // the right edge of the viewport; with snapping, it is shifted back
        // so it fits.
        let rect = Position::Right.resolve(base, popup, gap, cursor, viewport, true);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(920.0, 0.0), Size::new(80.0, 80.0)),
            "Snap should shift a popup that overflows the viewport"
        );
    }
}
