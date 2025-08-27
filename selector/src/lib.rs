#![allow(missing_docs)]
use iced_core as core;

pub mod target;

mod find;

pub use find::{Find, FindAll};
pub use target::Target;

use crate::core::Point;
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

impl Selector for String {
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
        format!("id == {:?}", self)
    }
}

impl Selector for Point {
    type Output = target::Match;

    fn select(&mut self, target: Target<'_>) -> Option<Self::Output> {
        target
            .visible_bounds()
            .is_some_and(|visible_bounds| visible_bounds.contains(*self))
            .then(|| target::Match::from_target(target))
    }

    fn description(&self) -> String {
        format!("bounds contains {:?}", self)
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
