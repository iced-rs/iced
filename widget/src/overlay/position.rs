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
    /// The `position` and `content_bounds` describe the element the popup is
    /// anchored to, `popup` is the size of the popup itself, and `gap` is the
    /// amount of space to leave around it.
    ///
    /// The `cursor_position` is only used when the [`Position`] is
    /// [`Position::FollowCursor`], and the `viewport` is only used when it is
    /// [`Position::Auto`].
    pub fn resolve(
        &self,
        position: Point,
        content_bounds: Rectangle,
        popup: Size,
        gap: f32,
        cursor_position: Point,
        viewport: Rectangle,
    ) -> Rectangle {
        let x_center = position.x + (content_bounds.width - popup.width) / 2.0;
        let y_center = position.y + (content_bounds.height - popup.height) / 2.0;

        // Resolve the positioning, choosing the side with the most available
        // space when `Position::Auto` is used.
        let positioning = match self {
            Position::Auto => {
                let base = content_bounds;

                // The space left on each side of the base once the popup
                // (and the gap) is placed there. Accounting for the popup's
                // own size makes `Auto` prefer a side where the popup
                // actually fits, so it is not snapped back over the base.
                let available = |position: Position| match position {
                    Position::Top => base.y - viewport.y - popup.height - gap,
                    Position::Bottom => {
                        (viewport.y + viewport.height) - (base.y + base.height) - popup.height - gap
                    }
                    Position::Left => base.x - viewport.x - popup.width - gap,
                    Position::Right => {
                        (viewport.x + viewport.width) - (base.x + base.width) - popup.width - gap
                    }
                    Position::Auto | Position::FollowCursor => unreachable!(),
                };

                // `Bottom` is listed last so it is preferred on a tie.
                [
                    Position::Top,
                    Position::Left,
                    Position::Right,
                    Position::Bottom,
                ]
                .into_iter()
                .max_by(|a, b| available(*a).total_cmp(&available(*b)))
                .unwrap()
            }
            other => *other,
        };

        let offset = match positioning {
            Position::Top => Point::new(x_center, position.y - popup.height - gap),
            Position::Bottom => Point::new(x_center, position.y + content_bounds.height + gap),
            Position::Left => Point::new(position.x - popup.width - gap, y_center),
            Position::Right => Point::new(position.x + content_bounds.width + gap, y_center),
            Position::FollowCursor => {
                let translation = position - content_bounds.position();
                Point::new(
                    cursor_position.x + translation.x,
                    cursor_position.y - popup.height + translation.y,
                )
            }
            Position::Auto => unreachable!("`positioning` is resolved above"),
        };

        Rectangle {
            x: offset.x,
            y: offset.y,
            width: popup.width,
            height: popup.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Position;
    use crate::core::{Point, Rectangle, Size};

    /// Common inputs: a 50x50 base at the origin, an 80x80 popup, no gap, and a
    /// 1000x1000 viewport.
    fn inputs() -> (Point, Rectangle, Size, f32, Point, Rectangle) {
        (
            Point::new(0.0, 0.0),
            Rectangle::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0)),
            Size::new(80.0, 80.0),
            0.0,
            Point::new(0.0, 0.0),
            Rectangle::new(Point::new(0.0, 0.0), Size::new(1000.0, 1000.0)),
        )
    }

    #[test]
    fn auto_prefers_the_side_with_the_most_space() {
        let (position, base, popup, gap, cursor, viewport) = inputs();

        // There is far more space below (950) than above (0); the tie between
        // "below" and "right" is broken in favour of `Bottom`.
        let rect = Position::Auto.resolve(position, base, popup, gap, cursor, viewport);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, 50.0), Size::new(80.0, 80.0)),
            "Auto should resolve to the side with the most space"
        );
    }

    #[test]
    fn auto_avoids_a_side_where_the_popup_does_not_fit() {
        let (position, base, _, gap, cursor, viewport) = inputs();
        // A popup taller than the space below the base (950), but one that
        // fits on the right.
        let popup = Size::new(80.0, 960.0);

        let rect = Position::Auto.resolve(position, base, popup, gap, cursor, viewport);

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
        let (position, base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Top.resolve(position, base, popup, gap, cursor, viewport);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, -80.0), Size::new(80.0, 80.0)),
            "Top should place the popup above the base"
        );
    }

    #[test]
    fn bottom_places_the_popup_below_the_base() {
        let (position, base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Bottom.resolve(position, base, popup, gap, cursor, viewport);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, 50.0), Size::new(80.0, 80.0)),
            "Bottom should place the popup below the base"
        );
    }

    #[test]
    fn left_places_the_popup_left_of_the_base() {
        let (position, base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Left.resolve(position, base, popup, gap, cursor, viewport);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-80.0, -15.0), Size::new(80.0, 80.0)),
            "Left should place the popup to the left of the base"
        );
    }

    #[test]
    fn right_places_the_popup_right_of_the_base() {
        let (position, base, popup, gap, cursor, viewport) = inputs();

        let rect = Position::Right.resolve(position, base, popup, gap, cursor, viewport);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(50.0, -15.0), Size::new(80.0, 80.0)),
            "Right should place the popup to the right of the base"
        );
    }

    #[test]
    fn follow_cursor_places_the_popup_at_the_cursor() {
        let (position, base, popup, gap, _, viewport) = inputs();
        let cursor = Point::new(200.0, 300.0);

        // The popup's bottom edge is placed at the cursor.
        let rect = Position::FollowCursor.resolve(position, base, popup, gap, cursor, viewport);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(200.0, 220.0), Size::new(80.0, 80.0)),
            "FollowCursor should place the popup at the cursor"
        );
    }
}
