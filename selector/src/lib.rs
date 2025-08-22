#![allow(missing_docs)]
use iced_core as core;

pub mod target;

mod find;

pub use find::{Find, FindAll};
pub use target::Target;

use crate::core::widget::Id;

pub trait Selector {
    type Output;

    fn select(&mut self, target: Target<'_>) -> Option<Self::Output>;

    fn description(&self) -> String;

    fn find(self) -> Find<Self>
    where
        Self: Sized,
    {
        Find::new(find::One::new(self))
    }

    fn find_all(self) -> FindAll<Self>
    where
        Self: Sized,
    {
        FindAll::new(find::All::new(self))
    }
}

impl Selector for &str {
    type Output = target::Text;

    fn select(&mut self, target: Target<'_>) -> Option<Self::Output> {
        match target {
            Target::TextInput {
                id,
                bounds,
                visible_bounds,
                state,
            } if state.text() == *self => Some(target::Text::Input {
                id: id.cloned(),
                bounds,
                visible_bounds,
            }),
            Target::Text {
                id,
                bounds,
                visible_bounds,
                content,
            } if content == *self => Some(target::Text::Raw {
                id: id.cloned(),
                bounds,
                visible_bounds,
            }),
            _ => None,
        }
    }

    fn description(&self) -> String {
        format!("text == \"{}\"", self.escape_default())
    }
}

impl Selector for Id {
    type Output = target::Match;

    fn select(&mut self, target: Target<'_>) -> Option<Self::Output> {
        if target.id() != Some(self) {
            return None;
        }

        Some(target::Match::from_target(target))
    }

    fn description(&self) -> String {
        format!("id == \"{:?}\"", self)
    }
}

impl<F, T> Selector for F
where
    F: FnMut(Target<'_>) -> Option<T>,
{
    type Output = T;

    fn select(&mut self, target: Target<'_>) -> Option<Self::Output> {
        (self)(target)
    }

    fn description(&self) -> String {
        format!("custom selector: {}", std::any::type_name_of_val(self))
    }
}

// pub fn inspect(position: Point) -> impl Selector<Output = (Match, Rectangle)> {
//     visible(move |target: Target<'_>, visible_bounds: Rectangle| {
//         visible_bounds
//             .contains(position)
//             .then(|| Match::from_target(target))
//     })
// }

// pub fn visible<T>(
//     f: impl Fn(Target<'_>, Rectangle) -> Option<T>,
// ) -> impl Selector<Output = (T, Rectangle)> {
//     todo!()
// }
//
