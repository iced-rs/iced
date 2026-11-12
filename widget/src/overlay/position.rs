use crate::core::{Point, Rectangle, Size};

/// The position of a popup (a popover or a tooltip) relative to the element
/// it is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// The popup will appear on the preferred [`Side`] if it fits there, and
    /// on the side of the widget with the most available space otherwise.
    Auto {
        /// The [`Side`] to prefer when the popup fits there.
        preference: Side,
    },
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

impl Default for Position {
    fn default() -> Self {
        Position::Auto {
            preference: Side::default(),
        }
    }
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
            // the preferred side if the popup fits there, and to the side
            // with the most available space otherwise.
            Position::Auto { preference } => {
                Side::with_most_available_space(content_bounds, viewport, popup, gap, *preference)
                    .offset(content_bounds, popup, gap)
            }
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

/// A side that a popup can appear on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Side {
    /// The top side.
    Top,
    /// The left side.
    Left,
    /// The right side.
    Right,
    /// The bottom side.
    #[default]
    Bottom,
}

impl Side {
    fn with_most_available_space(
        content_bounds: Rectangle,
        viewport: Rectangle,
        popup: Size,
        gap: f32,
        preference: Side,
    ) -> Side {
        // The popup is placed on the preferred side if it fits there.
        if preference.available_space(content_bounds, viewport, popup, gap) >= 0.0 {
            return preference;
        }

        // Otherwise, it is placed on the side with the most available space.
        [Side::Top, Side::Left, Side::Right, Side::Bottom]
            .into_iter()
            .max_by(|a, b| {
                a.available_space(content_bounds, viewport, popup, gap)
                    .total_cmp(&b.available_space(content_bounds, viewport, popup, gap))
            })
            .unwrap_or_default()
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
    use super::{Position, Side};
    use crate::core::{Point, Rectangle, Size};

    /// Common inputs: a 50x50 base at the origin, an 80x80 popup, no gap, a
    /// cursor at the origin, and a 1000x1000 viewport.
    const BASE: Rectangle = Rectangle::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0));
    const POPUP: Size = Size::new(80.0, 80.0);
    const GAP: f32 = 0.0;
    const CURSOR: Point = Point::new(0.0, 0.0);
    const VIEWPORT: Rectangle = Rectangle::new(Point::new(0.0, 0.0), Size::new(1000.0, 1000.0));

    #[test]
    fn auto_uses_the_preferred_side_when_it_fits() {
        // There is more space above (520) than below (270); the preference
        // still wins because the popup fits below.
        let base = Rectangle::new(Point::new(0.0, 600.0), Size::new(50.0, 50.0));
        let position = Position::Auto {
            preference: Side::Bottom,
        };

        let rect = position.resolve(base, POPUP, GAP, CURSOR, VIEWPORT, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, 650.0), Size::new(80.0, 80.0)),
            "Auto should use the preferred side when the popup fits there"
        );
    }

    #[test]
    fn auto_falls_back_to_the_side_with_the_most_space() {
        // The preferred side (`Bottom`) does not fit: the popup (960) is
        // taller than the space below the base (950), so the popup is placed
        // on the side with the most available space (right) instead.
        let popup = Size::new(80.0, 960.0);
        let position = Position::Auto {
            preference: Side::Bottom,
        };

        let rect = position.resolve(BASE, popup, GAP, CURSOR, VIEWPORT, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(50.0, -455.0), Size::new(80.0, 960.0)),
            "Auto should fall back to the side with the most available space"
        );
    }

    #[test]
    fn top_places_the_popup_above_the_base() {
        let rect = Position::Top.resolve(BASE, POPUP, GAP, CURSOR, VIEWPORT, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, -80.0), Size::new(80.0, 80.0)),
            "Top should place the popup above the base"
        );
    }

    #[test]
    fn bottom_places_the_popup_below_the_base() {
        let rect = Position::Bottom.resolve(BASE, POPUP, GAP, CURSOR, VIEWPORT, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-15.0, 50.0), Size::new(80.0, 80.0)),
            "Bottom should place the popup below the base"
        );
    }

    #[test]
    fn left_places_the_popup_left_of_the_base() {
        let rect = Position::Left.resolve(BASE, POPUP, GAP, CURSOR, VIEWPORT, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(-80.0, -15.0), Size::new(80.0, 80.0)),
            "Left should place the popup to the left of the base"
        );
    }

    #[test]
    fn right_places_the_popup_right_of_the_base() {
        let rect = Position::Right.resolve(BASE, POPUP, GAP, CURSOR, VIEWPORT, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(50.0, -15.0), Size::new(80.0, 80.0)),
            "Right should place the popup to the right of the base"
        );
    }

    #[test]
    fn follow_cursor_places_the_popup_at_the_cursor() {
        let cursor = Point::new(200.0, 300.0);

        // The popup's bottom edge is placed at the cursor.
        let rect = Position::FollowCursor.resolve(BASE, POPUP, GAP, cursor, VIEWPORT, false);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(200.0, 220.0), Size::new(80.0, 80.0)),
            "FollowCursor should place the popup at the cursor"
        );
    }

    #[test]
    fn snap_clamps_the_popup_into_the_viewport() {
        // Without snapping, `Top` places the popup at (-15, -80), outside of
        // the viewport; with snapping, it is clamped to the viewport's
        // origin.
        let rect = Position::Top.resolve(BASE, POPUP, GAP, CURSOR, VIEWPORT, true);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(0.0, 0.0), Size::new(80.0, 80.0)),
            "Snap should clamp the popup into the viewport"
        );
    }

    #[test]
    fn snap_shifts_a_popup_that_overflows_the_viewport() {
        // A base flush with the right edge of the viewport (1000).
        let base = Rectangle::new(Point::new(950.0, 0.0), Size::new(50.0, 50.0));

        // Without snapping, `Right` places the popup at (1000, -15), beyond
        // the right edge of the viewport; with snapping, it is shifted back
        // so it fits.
        let rect = Position::Right.resolve(base, POPUP, GAP, CURSOR, VIEWPORT, true);

        assert_eq!(
            rect,
            Rectangle::new(Point::new(920.0, 0.0), Size::new(80.0, 80.0)),
            "Snap should shift a popup that overflows the viewport"
        );
    }
}
