//! Build menus for your application.
use crate::keyboard::Hotkey;

/// Menu representation.
///
/// This can be used by `shell` implementations to create a menu.
#[derive(Debug, Clone, PartialEq)]
pub struct Menu<Message> {
    items: Vec<MenuEntry<Message>>,
}

impl<Message> Menu<Message> {
    /// Creates an empty [`Menu`].
    pub fn new() -> Self {
        Menu { items: Vec::new() }
    }

    /// Adds an item to the [`Menu`].
    pub fn item<S: Into<String>>(
        mut self,
        content: S,
        hotkey: impl Into<Option<Hotkey>>,
        on_activation: Message,
    ) -> Self {
        let content = content.into();
        let hotkey = hotkey.into();

        self.items.push(MenuEntry::Item {
            on_activation,
            content,
            hotkey,
        });
        self
    }

    /// Adds a separator to the [`Menu`].
    pub fn separator(mut self) -> Self {
        self.items.push(MenuEntry::Separator);
        self
    }

    /// Adds a dropdown to the [`Menu`].
    pub fn dropdown<S: Into<String>>(
        mut self,
        content: S,
        submenu: Menu<Message>,
    ) -> Self {
        let content = content.into();

        self.items.push(MenuEntry::Dropdown { content, submenu });
        self
    }

    /// Returns a [`MenuEntry`] iterator.
    pub fn iter(self) -> std::vec::IntoIter<MenuEntry<Message>> {
        self.items.into_iter()
    }
}

/// Represents one of the possible entries used to build a [`Menu`].
#[derive(Debug, Clone, PartialEq)]
pub enum MenuEntry<Message> {
    /// Item for a [`Menu`]
    Item {
        /// The title of the item
        content: String,
        /// The [`Hotkey`] to activate the item, if any
        hotkey: Option<Hotkey>,
        /// The message generated when the item is activated
        on_activation: Message,
    },
    /// Dropdown for a [`Menu`]
    Dropdown {
        /// Title of the dropdown
        content: String,
        /// The submenu of the dropdown
        submenu: Menu<Message>,
    },
    /// Separator for a [`Menu`]
    Separator,
}
