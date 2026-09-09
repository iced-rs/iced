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
    /// anchored to, `popup` is the size of the popup itself, and `gap` and
    /// `padding` are the amounts of space to leave around it.
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
        padding: f32,
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
                let available = |position: Position| match position {
                    Position::Top => base.y - viewport.y,
                    Position::Bottom => (viewport.y + viewport.height) - (base.y + base.height),
                    Position::Left => base.x - viewport.x,
                    Position::Right => (viewport.x + viewport.width) - (base.x + base.width),
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
            Position::Top => Point::new(x_center, position.y - popup.height - gap - padding),
            Position::Bottom => {
                Point::new(x_center, position.y + content_bounds.height + gap + padding)
            }
            Position::Left => Point::new(position.x - popup.width - gap - padding, y_center),
            Position::Right => {
                Point::new(position.x + content_bounds.width + gap + padding, y_center)
            }
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
            x: offset.x - padding,
            y: offset.y - padding,
            width: popup.width + 2.0 * padding,
            height: popup.height + 2.0 * padding,
        }
    }
}
