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

    /// Returns a [`MenuEntry`] iterator.
    pub fn iter(&self) -> impl Iterator<Item = &Entry<Message>> {
        self.entries.iter()
    }

    /// Adds an [`Entry`] to the [`Menu`].
    pub fn push(mut self, entry: Entry<Message>) -> Self {
        self.entries.push(entry);
        self
    }

    /// Maps the `Message` of the [`Menu`] using the provided function.
    ///
    /// This is useful to compose menus and split them into different
    /// abstraction levels.
    pub fn map<B>(self, f: impl Fn(Message) -> B + Copy) -> Menu<B> {
        // TODO: Use a boxed trait to avoid reallocation of entries
        Menu {
            entries: self
                .entries
                .into_iter()
                .map(|entry| entry.map(f))
                .collect(),
        }
    }
}

/// Represents one of the possible entries used to build a [`Menu`].
#[derive(Debug, Clone)]
pub enum Entry<Message> {
    /// Item for a [`Menu`]
    Item {
        /// The title of the item
        title: String,
        /// The [`Hotkey`] to activate the item, if any
        hotkey: Option<Hotkey>,
        /// The message generated when the item is activated
        on_activation: Message,
    },
    /// Dropdown for a [`Menu`]
    Dropdown {
        /// Title of the dropdown
        title: String,
        /// The submenu of the dropdown
        submenu: Menu<Message>,
    },
    /// Separator for a [`Menu`]
    Separator,
}

impl<Message> Entry<Message> {
    /// Creates an [`Entry::Item`].
    pub fn item<S: Into<String>>(
        title: S,
        hotkey: impl Into<Option<Hotkey>>,
        on_activation: Message,
    ) -> Self {
        let title = title.into();
        let hotkey = hotkey.into();

        Self::Item {
            title,
            hotkey,
            on_activation,
        }
    }

    /// Creates an [`Entry::Dropdown`].
    pub fn dropdown<S: Into<String>>(title: S, submenu: Menu<Message>) -> Self {
        let title = title.into();

        Self::Dropdown { title, submenu }
    }

    fn map<B>(self, f: impl Fn(Message) -> B + Copy) -> Entry<B> {
        match self {
            Self::Item {
                title,
                hotkey,
                on_activation,
            } => Entry::Item {
                title,
                hotkey,
                on_activation: f(on_activation),
            },
            Self::Dropdown { title, submenu } => Entry::Dropdown {
                title,
                submenu: submenu.map(f),
            },
            Self::Separator => Entry::Separator,
        }
    }
}

impl<Message> PartialEq for Entry<Message> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Entry::Item { title, hotkey, .. },
                Entry::Item {
                    title: other_title,
                    hotkey: other_hotkey,
                    ..
                },
            ) => title == other_title && hotkey == other_hotkey,
            (
                Entry::Dropdown { title, submenu },
                Entry::Dropdown {
                    title: other_title,
                    submenu: other_submenu,
                },
            ) => title == other_title && submenu == other_submenu,
            (Entry::Separator, Entry::Separator) => true,
            _ => false,
        }
    }
}
