//! Select data from the widget tree.
use iced_core as core;

mod find;
mod target;

pub use find::{Find, FindAll};
pub use target::{Bounded, Candidate, Target, Text};

use crate::core::Point;
use crate::core::widget;
use crate::core::widget::metadata::Role;

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

/// Creates a new [`Selector`] that matches widgets with the given semantic [`Role`].
pub fn by_role(role: Role) -> impl Selector<Output = Target> {
    struct ByRole {
        role: Role,
    }

    impl Selector for ByRole {
        type Output = Target;

        fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
            candidate
                .metadata()
                .is_some_and(|metadata| metadata.role == self.role)
                .then(|| Target::from(candidate))
        }

        fn description(&self) -> String {
            format!("role == {:?}", self.role)
        }
    }

    ByRole { role }
}

/// Creates a new [`Selector`] that matches widgets with the given semantic label.
pub fn by_label(label: impl Into<core::SmolStr>) -> impl Selector<Output = Target> {
    struct ByLabel {
        label: core::SmolStr,
    }

    impl Selector for ByLabel {
        type Output = Target;

        fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
            candidate
                .metadata()
                .and_then(|metadata| metadata.label.as_ref())
                .is_some_and(|label| label == &self.label)
                .then(|| Target::from(candidate))
        }

        fn description(&self) -> String {
            format!("label == {:?}", self.label)
        }
    }

    ByLabel {
        label: label.into(),
    }
}

/// Creates a new [`Selector`] that matches widgets with the given semantic [`Role`] and label.
pub fn by_role_and_label(
    role: Role,
    label: impl Into<core::SmolStr>,
) -> impl Selector<Output = Target> {
    struct ByRoleAndLabel {
        role: Role,
        label: core::SmolStr,
    }

    impl Selector for ByRoleAndLabel {
        type Output = Target;

        fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
            candidate
                .metadata()
                .is_some_and(|metadata| {
                    metadata.role == self.role
                        && metadata
                            .label
                            .as_ref()
                            .is_some_and(|label| label == &self.label)
                })
                .then(|| Target::from(candidate))
        }

        fn description(&self) -> String {
            format!("role == {:?} && label == {:?}", self.role, self.label)
        }
    }

    ByRoleAndLabel {
        role,
        label: label.into(),
    }
}

/// Creates a new [`Selector`] that matches widgets with the given semantic test id.
pub fn by_test_id(test_id: impl Into<core::SmolStr>) -> impl Selector<Output = Target> {
    struct ByTestId {
        test_id: core::SmolStr,
    }

    impl Selector for ByTestId {
        type Output = Target;

        fn select(&mut self, candidate: Candidate<'_>) -> Option<Self::Output> {
            candidate
                .metadata()
                .and_then(|metadata| metadata.test_id.as_ref())
                .is_some_and(|test_id| test_id == &self.test_id)
                .then(|| Target::from(candidate))
        }

        fn description(&self) -> String {
            format!("test_id == {:?}", self.test_id)
        }
    }

    ByTestId {
        test_id: test_id.into(),
    }
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
    use super::{Selector, by_label, by_role, by_role_and_label, by_test_id};
    use crate::core::Rectangle;
    use crate::core::widget::Operation;
    use crate::core::widget::metadata::{Metadata, Role};
    use crate::core::widget::operation::Outcome;
    use crate::target::{Candidate, Target};

    #[test]
    fn selects_by_role() {
        let metadata = Metadata::new(Role::Button);
        let candidate = Candidate::Metadata {
            id: None,
            bounds: Rectangle::new([1.0, 2.0].into(), [3.0, 4.0].into()),
            visible_bounds: None,
            metadata: &metadata,
        };

        assert!(matches!(
            by_role(Role::Button).select(candidate),
            Some(Target::Metadata { metadata, .. })
                if metadata.role == Role::Button
        ));
    }

    #[test]
    fn selects_by_label() {
        let metadata = Metadata::new(Role::Button).label("Create");
        let candidate = Candidate::Metadata {
            id: None,
            bounds: Rectangle::default(),
            visible_bounds: None,
            metadata: &metadata,
        };

        assert!(by_label("Create").select(candidate).is_some());
    }

    #[test]
    fn selects_by_role_and_label() {
        let metadata = Metadata::new(Role::Button).label("Create");
        let candidate = Candidate::Metadata {
            id: None,
            bounds: Rectangle::default(),
            visible_bounds: None,
            metadata: &metadata,
        };

        assert!(
            by_role_and_label(Role::Button, "Create")
                .select(candidate)
                .is_some()
        );
    }

    #[test]
    fn selects_by_test_id() {
        let metadata = Metadata::new(Role::TextInput).test_id("profile-name");
        let candidate = Candidate::Metadata {
            id: None,
            bounds: Rectangle::default(),
            visible_bounds: None,
            metadata: &metadata,
        };

        assert!(by_test_id("profile-name").select(candidate).is_some());
    }

    #[test]
    fn label_does_not_match_test_id() {
        let metadata = Metadata::new(Role::Button).test_id("create");
        let candidate = Candidate::Metadata {
            id: None,
            bounds: Rectangle::default(),
            visible_bounds: None,
            metadata: &metadata,
        };

        assert!(by_label("create").select(candidate).is_none());
    }

    #[test]
    fn find_selects_metadata_emitted_by_operation() {
        let metadata = Metadata::new(Role::Button).test_id("create");
        let mut operation = by_test_id("create").find();
        let bounds = Rectangle::new([1.0, 2.0].into(), [3.0, 4.0].into());

        operation.metadata(None, bounds, &metadata);

        assert!(matches!(
            operation.finish(),
            Outcome::Some(Some(Target::Metadata { bounds: found, .. }))
                if found == bounds
        ));
    }
}
