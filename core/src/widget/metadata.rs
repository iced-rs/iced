//! Semantic widget metadata.

use crate::SmolStr;

/// Semantic information about a widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// The semantic role of the widget.
    pub role: Role,
    /// A user-facing label that identifies the widget.
    pub label: Option<SmolStr>,
    /// A stable identifier intended for tests.
    pub test_id: Option<SmolStr>,
}

impl Metadata {
    /// Creates new [`Metadata`] with the given [`Role`].
    pub fn new(role: Role) -> Self {
        Self {
            role,
            label: None,
            test_id: None,
        }
    }

    /// Sets the user-facing label of the [`Metadata`].
    pub fn label(mut self, label: impl Into<SmolStr>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the test identifier of the [`Metadata`].
    pub fn test_id(mut self, test_id: impl Into<SmolStr>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }
}

/// The semantic role of a widget.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Role {
    Button,
    Checkbox,
    TextInput,
    PickList,
    Scrollable,
    Text,
    Dialog,
    Tab,
    List,
    ListItem,
    Custom(SmolStr),
}
