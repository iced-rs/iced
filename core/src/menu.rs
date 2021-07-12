//! Build menus for your application.
use crate::keyboard::Hotkey;

/// Menu representation.
///
/// This can be used by `shell` implementations to create a menu.
#[derive(Debug, Clone)]
pub struct Menu<Message> {
    entries: Vec<Entry<Message>>,
}

impl<Message> PartialEq for Menu<Message> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<Message> Menu<Message> {
    /// Creates an empty [`Menu`].
    pub fn new() -> Self {
        Self::with_entries(Vec::new())
    }

    /// Creates a new [`Menu`] with the given entries.
    pub fn with_entries(entries: Vec<Entry<Message>>) -> Self {
        Self { entries }
    }

    /// Adds an [`Entry`] to the [`Menu`].
    pub fn push(mut self, entry: Entry<Message>) -> Self {
        self.entries.push(entry);
        self
    }

    /// Returns a [`MenuEntry`] iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Entry<Message>> {
        self.entries.iter()
    }
}

/// Represents one of the possible entries used to build a [`Menu`].
#[derive(Debug, Clone)]
pub enum Entry<Message> {
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

impl<Message> Entry<Message> {
    /// Creates an [`Entry::Item`].
    pub fn item<S: Into<String>>(
        content: S,
        hotkey: impl Into<Option<Hotkey>>,
        on_activation: Message,
    ) -> Self {
        let content = content.into();
        let hotkey = hotkey.into();

        Entry::Item {
            content,
            hotkey,
            on_activation,
        }
    }

    /// Creates an [`Entry::Dropdown`].
    pub fn dropdown<S: Into<String>>(
        content: S,
        submenu: Menu<Message>,
    ) -> Self {
        let content = content.into();

        Entry::Dropdown { content, submenu }
    }
}

impl<Message> PartialEq for Entry<Message> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Entry::Item {
                    content, hotkey, ..
                },
                Entry::Item {
                    content: other_content,
                    hotkey: other_hotkey,
                    ..
                },
            ) => content == other_content && hotkey == other_hotkey,
            (
                Entry::Dropdown { content, submenu },
                Entry::Dropdown {
                    content: other_content,
                    submenu: other_submenu,
                },
            ) => content == other_content && submenu == other_submenu,
            (Entry::Separator, Entry::Separator) => true,
            _ => false,
        }
    }
}
