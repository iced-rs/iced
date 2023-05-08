use iced_core::window::Id;
use iced_futures::MaybeSend;
use sctk::reexports::client::protocol::wl_data_device_manager::DndAction;
use std::{any::Any, fmt, marker::PhantomData};

/// DataDevice Action
pub struct Action<T> {
    /// The inner action
    pub inner: ActionInner,
    /// The phantom data
    _phantom: PhantomData<T>,
}

impl<T> From<ActionInner> for Action<T> {
    fn from(inner: ActionInner) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

/// A trait for converting to data given a mime type.
pub trait DataFromMimeType {
    /// Convert to data given a mime type.
    fn from_mime_type(&self, mime_type: &str) -> Option<Vec<u8>>;
}

/// DataDevice Action
pub enum ActionInner {
    /// Indicate that you are setting the selection and will respond to events
    /// with data of the advertised mime types.
    SetSelection {
        /// The mime types that the selection can be converted to.
        mime_types: Vec<String>,
        /// The data to send.
        data: Box<dyn DataFromMimeType + Send + Sync>,
    },
    /// Unset the selection.
    UnsetSelection,
    /// Request the selection data from the clipboard.
    RequestSelectionData {
        /// The mime type that the selection should be converted to.
        mime_type: String,
    },
    /// Start a drag and drop operation. When a client asks for the selection, an event will be delivered
    /// This is used for internal drags, where the client is the source of the drag.
    /// The client will be resposible for data transfer.
    StartInternalDnd {
        /// The window id of the window that is the source of the drag.
        origin_id: Id,
        /// An optional window id for the cursor icon surface.
        icon_id: Option<Id>,
    },
    /// Start a drag and drop operation. When a client asks for the selection, an event will be delivered
    StartDnd {
        /// The mime types that the dnd data can be converted to.
        mime_types: Vec<String>,
        /// The actions that the client supports.
        actions: DndAction,
        /// The window id of the window that is the source of the drag.
        origin_id: Id,
        /// An optional window id for the cursor icon surface.
        icon_id: Option<DndIcon>,
        /// The data to send.
        data: Box<dyn DataFromMimeType + Send + Sync>,
    },
    /// Set the accepted drag and drop mime type.
    Accept(Option<String>),
    /// Set accepted and preferred drag and drop actions.
    SetActions {
        /// The preferred action.
        preferred: DndAction,
        /// The accepted actions.
        accepted: DndAction,
    },
    /// Read the Drag and Drop data with a mime type. An event will be delivered with a pipe to read the data from.
    RequestDndData(String),
    /// The drag and drop operation has finished.
    DndFinished,
    /// The drag and drop operation has been cancelled.
    DndCancelled,
}

/// DndIcon
#[derive(Debug)]
pub enum DndIcon {
    /// The id of the widget which will draw the dnd icon.
    Widget(Id, Box<dyn Send + Any>),
    /// A custom icon.
    Custom(Id),
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
        Action::from(self.inner)
    }
}

impl fmt::Debug for ActionInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accept(mime_type) => {
                f.debug_tuple("Accept").field(mime_type).finish()
            }
            Self::SetSelection { mime_types, .. } => {
                f.debug_tuple("SetSelection").field(mime_types).finish()
            }
            Self::UnsetSelection => f.debug_tuple("UnsetSelection").finish(),
            Self::RequestSelectionData { mime_type } => {
                f.debug_tuple("RequestSelection").field(mime_type).finish()
            }
            Self::StartInternalDnd { origin_id, icon_id } => f
                .debug_tuple("StartInternalDnd")
                .field(origin_id)
                .field(icon_id)
                .finish(),
            Self::StartDnd {
                mime_types,
                actions,
                origin_id,
                icon_id,
                ..
            } => f
                .debug_tuple("StartDnd")
                .field(mime_types)
                .field(actions)
                .field(origin_id)
                .field(icon_id)
                .finish(),
            Self::SetActions {
                preferred,
                accepted,
            } => f
                .debug_tuple("SetActions")
                .field(preferred)
                .field(accepted)
                .finish(),
            Self::RequestDndData(mime_type) => {
                f.debug_tuple("RequestDndData").field(mime_type).finish()
            }
            Self::DndFinished => f.debug_tuple("DndFinished").finish(),
            Self::DndCancelled => f.debug_tuple("DndCancelled").finish(),
        }
    }
}
