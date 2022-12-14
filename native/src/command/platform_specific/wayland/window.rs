use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{collections::hash_map::DefaultHasher, fmt};

use iced_futures::MaybeSend;
use sctk::reexports::protocols::xdg::shell::client::xdg_toplevel::ResizeEdge;

use crate::window;

/// window settings
#[derive(Debug, Clone)]
pub struct SctkWindowSettings {
    /// vanilla window settings
    pub iced_settings: window::Settings,
    /// window id
    pub window_id: window::Id,
    /// optional app id
    pub app_id: Option<String>,
    /// optional window title
    pub title: Option<String>,
    /// optional window parent
    pub parent: Option<window::Id>,
}

impl Default for SctkWindowSettings {
    fn default() -> Self {
        Self {
            iced_settings: Default::default(),
            window_id: window::Id::new(0),
            app_id: Default::default(),
            title: Default::default(),
            parent: Default::default(),
        }
    }
}

#[derive(Clone)]
/// Window Action
pub enum Action<T> {
    /// create a window and receive a message with its Id
    Window {
        /// window builder
        builder: SctkWindowSettings,
        /// phanton
        _phantom: PhantomData<T>,
    },
    /// Destroy the window
    Destroy(window::Id),
    /// Set size of the window.
    Size {
        /// id of the window
        id: window::Id,
        /// The new logical width of the window
        width: u32,
        /// The new logical height of the window
        height: u32,
    },
    /// Set min size of the window.
    MinSize {
        /// id of the window
        id: window::Id,
        /// optional size
        size: Option<(u32, u32)>,
    },
    /// Set max size of the window.
    MaxSize {
        /// id of the window
        id: window::Id,
        /// optional size
        size: Option<(u32, u32)>,
    },
    /// Set title of the window.
    Title {
        /// id of the window
        id: window::Id,
        /// The new logical width of the window
        title: String,
    },
    /// Minimize the window.
    Minimize {
        /// id of the window
        id: window::Id,
    },
    /// Maximize the window.
    Maximize {
        /// id of the window
        id: window::Id,
    },
    /// UnsetMaximize the window.
    UnsetMaximize {
        /// id of the window
        id: window::Id,
    },
    /// Fullscreen the window.
    Fullscreen {
        /// id of the window
        id: window::Id,
    },
    /// UnsetFullscreen the window.
    UnsetFullscreen {
        /// id of the window
        id: window::Id,
    },
    /// Start an interactive move of the window.
    InteractiveResize {
        /// id of the window
        id: window::Id,
        /// edge being resized
        edge: ResizeEdge,
    },
    /// Start an interactive move of the window.
    InteractiveMove {
        /// id of the window
        id: window::Id,
    },
    /// Show the window context menu
    ShowWindowMenu {
        /// id of the window
        id: window::Id,
        /// x location of popup
        x: i32,
        /// y location of popup
        y: i32,
    },
}

impl<T> Action<T> {
    /// Maps the output of a window [`Action`] using the provided closure.
    pub fn map<A>(
        self,
        _: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
    {
        match self {
            Action::Window { builder, .. } => Action::Window {
                builder,
                _phantom: PhantomData::default(),
            },
            Action::Size { id, width, height } => {
                Action::Size { id, width, height }
            }
            Action::MinSize { id, size } => Action::MinSize { id, size },
            Action::MaxSize { id, size } => Action::MaxSize { id, size },
            Action::Title { id, title } => Action::Title { id, title },
            Action::Minimize { id } => Action::Minimize { id },
            Action::Maximize { id } => Action::Maximize { id },
            Action::UnsetMaximize { id } => Action::UnsetMaximize { id },
            Action::Fullscreen { id } => Action::Fullscreen { id },
            Action::UnsetFullscreen { id } => Action::UnsetFullscreen { id },
            Action::InteractiveMove { id } => Action::InteractiveMove { id },
            Action::ShowWindowMenu { id, x, y } => {
                Action::ShowWindowMenu { id, x, y }
            }
            Action::InteractiveResize { id, edge } => {
                Action::InteractiveResize { id, edge }
            }
            Action::Destroy(id) => Action::Destroy(id),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Window { builder, .. } => write!(
                f,
                "Action::Window::LayerSurface {{ builder: {:?} }}",
                builder
            ),
            Action::Size { id, width, height } => write!(
                f,
                "Action::Window::Size {{ id: {:?}, width: {:?}, height: {:?} }}",
                id, width, height
            ),
            Action::MinSize { id, size } => write!(
                f,
                "Action::Window::MinSize {{ id: {:?}, size: {:?} }}",
                id, size
            ),
            Action::MaxSize { id, size } => write!(
                f,
                "Action::Window::MaxSize {{ id: {:?}, size: {:?} }}",
                id, size
            ),
            Action::Title { id, title } => write!(
                f,
                "Action::Window::Title {{ id: {:?}, title: {:?} }}",
                id, title
            ),
            Action::Minimize { id } => write!(
                f,
                "Action::Window::Minimize {{ id: {:?} }}",
                id
            ),
            Action::Maximize { id } => write!(
                f,
                "Action::Window::Maximize {{ id: {:?} }}",
                id
            ),
            Action::UnsetMaximize { id } => write!(
                f,
                "Action::Window::UnsetMaximize {{ id: {:?} }}",
                id
            ),
            Action::Fullscreen { id } => write!(
                f,
                "Action::Window::Fullscreen {{ id: {:?} }}",
                id
            ),
            Action::UnsetFullscreen { id } => write!(
                f,
                "Action::Window::UnsetFullscreen {{ id: {:?} }}",
                id
            ),
            Action::InteractiveMove { id } => write!(
                f,
                "Action::Window::InteractiveMove {{ id: {:?} }}",
                id
            ),
            Action::ShowWindowMenu { id, x, y } => write!(
                f,
                "Action::Window::ShowWindowMenu {{ id: {:?}, x: {x}, y: {y} }}",
                id
            ),
            Action::InteractiveResize { id, edge } => write!(
                f,
                "Action::Window::InteractiveResize {{ id: {:?}, edge: {:?} }}",
                id, edge
            ),
            Action::Destroy(id) => write!(
                f,
                "Action::Window::Destroy {{ id: {:?} }}",
                id
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// TODO(derezzedex)
pub struct Id(u64);

impl Id {
    /// TODO(derezzedex)
    pub fn new(id: impl Hash) -> Id {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        Id(hasher.finish())
    }
}
