#![allow(missing_docs)]
use iced_core as core;

mod find;
mod target;

pub use find::{Find, FindAll};
pub use target::{Bounded, Candidate, Target, Text};

use crate::core::Point;
use crate::core::widget;

pub trait Selector {
    type Output;

    fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output>;

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

    fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
        match candidate {
            Candidate::TextInput {
                id,
                bounds,
                visible_bounds,
                state,
            } if state.text() == *self => Some(target::Text::Input {
                id: id.cloned(),
                bounds,
                visible_bounds,
            }),
            Candidate::Text {
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
        format!("text == {self:?}")
    }
}

impl Selector for String {
    type Output = target::Text;

    fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
        self.as_str().select(candidate)
    }

    fn description(&self) -> String {
        self.as_str().description()
    }
}

impl Selector for widget::Id {
    type Output = Target;

    fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
        if candidate.id() != Some(self) {
            return None;
        }

        Some(Target::from(candidate))
    }

    fn description(&self) -> String {
        format!("id == {self:?}")
    }
}

impl Selector for Point {
    type Output = Target;

    fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
        candidate
            .visible_bounds()
            .is_some_and(|visible_bounds| visible_bounds.contains(*self))
            .then(|| Target::from(candidate))
    }

    fn description(&self) -> String {
        format!("bounds contains {self:?}")
    }
}

impl<F, T> Selector for F
where
    F: FnMut(Candidate<'_>) -> Option<T>,
{
    type Output = T;

    fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
        (self)(candidate)
    }

    fn description(&self) -> String {
        format!("custom selector: {}", std::any::type_name_of_val(self))
    }
}

/// Creates a new [`Selector`] that matches widgets with the given [`widget::Id`].
pub fn id(id: impl Into<widget::Id>) -> impl Selector<Output = Target> {
    id.into()
}
