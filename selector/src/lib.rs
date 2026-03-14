//! Select data from the widget tree.
use iced_core as core;

mod find;
mod target;

pub use find::{Find, FindAll};
pub use target::{AccessibleMatch, Bounded, Candidate, Target, Text};

use crate::core::Point;
use crate::core::widget;

/// A type that traverses the widget tree to "select" data and produce some output.
pub trait Selector {
    /// The output type of the [`Selector`].
    ///
    /// For most selectors, this will normally be a [`Target`]. However, some
    /// selectors may want to return a more limited type to encode the selection
    /// guarantees in the type system.
    ///
    /// For instance, the implementations of [`String`] and [`str`] of [`Selector`]
    /// return a [`target::Text`] instead of a generic [`Target`], since they are
    /// guaranteed to only select text.
    type Output;

    /// Performs a selection of the given [`Candidate`], if applicable.
    ///
    /// This method traverses the widget tree in depth-first order.
    fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output>;

    /// Returns a short description of the [`Selector`] for debugging purposes.
    fn description(&self) -> String;

    /// Returns a [`widget::Operation`] that runs the [`Selector`] and stops after
    /// the first [`Output`](Self::Output) is produced.
    fn find(self) -> Find<Self>
    where
        Self: Sized,
    {
        Find::new(find::One::new(self))
    }

    /// Returns a [`widget::Operation`] that runs the [`Selector`] for the entire
    /// widget tree and aggregates all of its [`Output`](Self::Output).
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

/// Returns a [`Selector`] that matches widgets with the given accessible
/// [`Role`](widget::operation::accessible::Role).
pub fn by_role(
    role: widget::operation::accessible::Role,
) -> impl Selector<Output = AccessibleMatch> {
    struct ByRole(widget::operation::accessible::Role);

    impl Selector for ByRole {
        type Output = AccessibleMatch;

        fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
            if let Candidate::Accessible {
                id,
                bounds,
                visible_bounds,
                accessible,
            } = candidate
                && accessible.role == self.0
            {
                return Some(AccessibleMatch {
                    id: id.cloned(),
                    bounds,
                    visible_bounds,
                    role: accessible.role,
                    label: accessible.label.map(str::to_owned),
                    description: accessible.description.map(str::to_owned),
                });
            }
            None
        }

        fn description(&self) -> String {
            format!("role == {:?}", self.0)
        }
    }

    ByRole(role)
}

/// Returns a [`Selector`] that matches widgets with the given accessible label (exact match).
pub fn by_label(label: &str) -> impl Selector<Output = AccessibleMatch> + '_ {
    struct ByLabel<'a>(&'a str);

    impl Selector for ByLabel<'_> {
        type Output = AccessibleMatch;

        fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
            if let Candidate::Accessible {
                id,
                bounds,
                visible_bounds,
                accessible,
            } = candidate
                && accessible.label == Some(self.0)
            {
                return Some(AccessibleMatch {
                    id: id.cloned(),
                    bounds,
                    visible_bounds,
                    role: accessible.role,
                    label: accessible.label.map(str::to_owned),
                    description: accessible.description.map(str::to_owned),
                });
            }
            None
        }

        fn description(&self) -> String {
            format!("label == {:?}", self.0)
        }
    }

    ByLabel(label)
}

/// Returns a [`Selector`] that matches widgets that are currently focused.
pub fn is_focused() -> impl Selector<Output = Target> {
    struct IsFocused;

    impl Selector for IsFocused {
        type Output = Target;

        fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
            if let Candidate::Focusable { state, .. } = candidate
                && state.is_focused()
            {
                Some(Target::from(candidate))
            } else {
                None
            }
        }

        fn description(&self) -> String {
            "is focused".to_owned()
        }
    }

    IsFocused
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Rectangle;
    use crate::core::widget::operation::accessible::{Accessible, Role as IcedRole};
    use crate::target::Candidate;

    const UNIT: Rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
    };

    fn make_accessible_candidate(
        role: IcedRole,
        label: Option<&str>,
    ) -> (Accessible<'_>, Rectangle) {
        (
            Accessible {
                role,
                label,
                ..Accessible::default()
            },
            UNIT,
        )
    }

    #[test]
    fn by_role_matches_accessible_candidate() {
        let (accessible, bounds) = make_accessible_candidate(IcedRole::Button, None);
        let candidate = Candidate::Accessible {
            id: None,
            bounds,
            visible_bounds: Some(bounds),
            accessible: &accessible,
        };

        let mut selector = by_role(IcedRole::Button);
        let result = selector.select(candidate);

        assert!(
            result.is_some(),
            "by_role(Button) should match a Button candidate"
        );
        let m = result.unwrap();
        assert_eq!(m.role, IcedRole::Button);
    }

    #[test]
    fn by_role_skips_wrong_role() {
        let (accessible, bounds) = make_accessible_candidate(IcedRole::Button, None);
        let candidate = Candidate::Accessible {
            id: None,
            bounds,
            visible_bounds: Some(bounds),
            accessible: &accessible,
        };

        let mut selector = by_role(IcedRole::Slider);
        let result = selector.select(candidate);

        assert!(
            result.is_none(),
            "by_role(Slider) should not match a Button candidate"
        );
    }

    #[test]
    fn by_label_matches_accessible_label() {
        let (accessible, bounds) = make_accessible_candidate(IcedRole::Button, Some("Submit"));
        let candidate = Candidate::Accessible {
            id: None,
            bounds,
            visible_bounds: Some(bounds),
            accessible: &accessible,
        };

        let mut selector = by_label("Submit");
        let result = selector.select(candidate);

        assert!(result.is_some(), "by_label(\"Submit\") should match");
        let m = result.unwrap();
        assert_eq!(m.label.as_deref(), Some("Submit"));
    }

    #[test]
    fn by_label_skips_wrong_label() {
        let (accessible, bounds) = make_accessible_candidate(IcedRole::Button, Some("Submit"));
        let candidate = Candidate::Accessible {
            id: None,
            bounds,
            visible_bounds: Some(bounds),
            accessible: &accessible,
        };

        let mut selector = by_label("Cancel");
        let result = selector.select(candidate);

        assert!(
            result.is_none(),
            "by_label(\"Cancel\") should not match \"Submit\""
        );
    }
}
