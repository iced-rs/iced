//! Compute the damage between frames.
use crate::Viewport;
use crate::core::{Point, Rectangle, Size};

/// Diffs the damage regions given some previous and current primitives.
pub fn diff<T>(
    previous: &[T],
    current: &[T],
    bounds: impl Fn(&T) -> Vec<Rectangle>,
    diff: impl Fn(&T, &T) -> Vec<Rectangle>,
) -> Vec<Rectangle> {
    let damage = previous.iter().zip(current).flat_map(|(a, b)| diff(a, b));

    if previous.len() == current.len() {
        damage.collect()
    } else {
        let (smaller, bigger) = if previous.len() < current.len() {
            (previous, current)
        } else {
            (current, previous)
        };

        // Extend damage by the added/removed primitives
        damage
            .chain(bigger[smaller.len()..].iter().flat_map(bounds))
            .collect()
    }
}

/// Computes the damage regions given some previous and current primitives.
pub fn list<T>(
    previous: &[T],
    current: &[T],
    bounds: impl Fn(&T) -> Vec<Rectangle>,
    are_equal: impl Fn(&T, &T) -> bool,
) -> Vec<Rectangle> {
    diff(previous, current, &bounds, |a, b| {
        if are_equal(a, b) {
            vec![]
        } else {
            bounds(a).into_iter().chain(bounds(b)).collect()
        }
    })
}

/// Groups the given damage regions that are close together inside the given
/// bounds.
pub fn group(mut damage: Vec<Rectangle>, bounds: Rectangle) -> impl Iterator<Item = Rectangle> {
    const AREA_THRESHOLD: f32 = 20_000.0;

    damage.sort_by(|a, b| {
        a.center()
            .distance(Point::ORIGIN)
            .total_cmp(&b.center().distance(Point::ORIGIN))
    });

    let mut scaled = damage
        .into_iter()
        .filter_map(move |region| region.intersection(&bounds))
        .filter(|region| region.width >= 1.0 && region.height >= 1.0)
        .peekable();

    let mut group = scaled.next();

    std::iter::from_fn(move || {
        loop {
            let Some(region) = scaled.next() else {
                return group.take();
            };

            let current = group?;
            let union = current.union(&region);

            if union.area() - current.area() - region.area() <= AREA_THRESHOLD {
                group = Some(union);
            } else {
                group = Some(region);
                return Some(current);
            }
        }
    })
}

/// Converts logical damage into physical screen damage.
pub fn physical(
    damage: impl IntoIterator<Item = Rectangle>,
    viewport: &Viewport,
) -> impl Iterator<Item = Rectangle<u32>> {
    let scale = viewport.scale_factor();
    let viewport = Rectangle::with_size(Size::new(
        viewport.physical_width() as f32,
        viewport.physical_width() as f32,
    ));

    damage
        .into_iter()
        .map(move |region| region * scale)
        .filter_map(move |region| region.expand(2).intersection(&viewport))
        .filter_map(Rectangle::snap)
}
