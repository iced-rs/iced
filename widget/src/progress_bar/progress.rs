use std::ops::RangeInclusive;

use crate::core::Rectangle;

pub fn get_progress_offset(
    bounds: Rectangle,
    value: f32,
    range: RangeInclusive<f32>,
    vertical: bool,
) -> f32 {
    let (start, end) = range.into_inner();

    let percent = if value < start {
        0.
    } else if value > end {
        1.
    } else {
        (value - start) / (end - start)
    };

    let (size, _offset) = if vertical {
        (bounds.height, bounds.y)
    } else {
        (bounds.width, bounds.x)
    };

    size * percent
}

pub fn get_progress_rect(
    bounds: Rectangle,
    value: f32,
    range: RangeInclusive<f32>,
    vertical: bool,
    reverse: bool,
) -> Rectangle {
    let bar_size = get_progress_offset(bounds, value, range, vertical);

    match (vertical, reverse) {
        (false, false) => Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bar_size,
            height: bounds.height,
        },
        (false, true) => Rectangle {
            x: bounds.x + bounds.width - bar_size,
            y: bounds.y,
            width: bar_size,
            height: bounds.height,
        },
        (true, false) => Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - bar_size,
            width: bounds.width,
            height: bar_size,
        },
        (true, true) => Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bar_size,
        },
    }
}
