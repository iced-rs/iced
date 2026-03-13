//! Wayland drag-and-drop (DnD) types.
//!
//! This module provides the event types and request structures for
//! Wayland-native drag-and-drop.  The design mirrors the `wl_data_device`
//! protocol: a **source** initiates a drag with offered MIME types and
//! supported actions, while one or more **destinations** receive enter /
//! motion / leave / drop events and can accept or reject the operation.
//!
//! # Event flow
//!
//! 1. User starts dragging → widget publishes [`Request::StartDrag`] via
//!    [`Shell::start_dnd`].
//! 2. Runtime calls `wl_data_device.start_drag(…)` on the Wayland
//!    connection.
//! 3. The compositor delivers DnD protocol events which the runtime
//!    converts to [`Event`] variants and pushes them into the iced event
//!    queue.
//! 4. Widgets (or the application) handle [`Event::Enter`],
//!    [`Event::Motion`], [`Event::Leave`], [`Event::Drop`], and
//!    [`Event::SourceEvent`] to update visual feedback and perform the
//!    drop action.

use bitflags::bitflags;

bitflags! {
    /// The set of actions supported or accepted for a drag-and-drop
    /// operation.
    ///
    /// Maps directly to `wl_data_device_manager::dnd_action` in the
    /// Wayland protocol.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DndAction: u32 {
        /// No action (reject).
        const NONE = 0;
        /// Copy the dragged data.
        const COPY = 1;
        /// Move the dragged data (source should delete after drop).
        const MOVE = 2;
        /// Ask the user which action to perform.
        const ASK  = 4;
    }
}

/// A drag-and-drop event delivered to widgets.
///
/// These events are produced by the runtime when the Wayland compositor
/// sends `wl_data_device` / `wl_data_offer` / `wl_data_source` events.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A drag offer entered the application surface.
    ///
    /// `mime_types` lists the MIME types offered by the drag source.
    /// `x` / `y` are surface-local coordinates.
    Enter {
        /// Surface-local X coordinate.
        x: f64,
        /// Surface-local Y coordinate.
        y: f64,
        /// MIME types offered by the source.
        mime_types: Vec<String>,
    },

    /// The drag cursor moved within our surface.
    Motion {
        /// Surface-local X coordinate.
        x: f64,
        /// Surface-local Y coordinate.
        y: f64,
    },

    /// The drag offer left our surface without a drop.
    Leave,

    /// The user dropped the data on our surface.
    ///
    /// After receiving this, the destination should request the data
    /// for the desired MIME type via [`Request::RequestData`] and then
    /// call [`Request::FinishDnd`].
    Drop,

    /// Data was received for a previously requested MIME type.
    ///
    /// This is the response to [`Request::RequestData`].
    DataReceived {
        /// The MIME type of the received data.
        mime_type: String,
        /// The raw bytes of the data.
        data: Vec<u8>,
    },

    /// The selected (negotiated) action for the current drag.
    SelectedAction(DndAction),

    /// An event for the **drag source** side of the operation.
    SourceEvent(SourceEvent),
}

/// Events delivered to the drag **source** (the initiator of the drag).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEvent {
    /// The compositor is requesting data for the given MIME type.
    ///
    /// The source should respond by writing the data.  For internal
    /// (same-app) drags this is handled automatically by the runtime.
    SendRequested,

    /// The drag was cancelled (e.g. user pressed Escape).
    Cancelled,

    /// The drop was performed by the destination.
    DropPerformed,

    /// The destination has finished processing the dropped data.
    /// The source may now delete the original data if the action was
    /// `Move`.
    Finished,

    /// The negotiated action changed.
    Action(DndAction),
}

/// Icon for the drag visual.
///
/// The runtime uses this to create the `wl_surface` that the compositor
/// displays under the cursor during a Wayland drag.
pub enum DndIcon {
    /// Pre-rendered pixel data in **pre-multiplied ARGB** format
    /// (little-endian), which is the native Wayland `wl_shm` pixel
    /// format (`Argb8888`).
    Pixels {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// Pre-multiplied ARGB pixel data (4 bytes per pixel, row-major).
        pixels: Vec<u8>,
        /// Buffer scale for HiDPI support (e.g., 2 for 2x rendering).
        scale: i32,
    },

    /// A type-erased iced `Element` that the runtime will render
    /// offscreen to produce the icon pixels.
    ///
    /// The element is stored as `Box<dyn Any>` to avoid generic
    /// parameters leaking into the core DnD types.  The runtime
    /// downcasts to the concrete `(Element<'static, (), Theme, Renderer>,
    /// widget::tree::State)` tuple when rendering.
    Element(DndIconSurface),
}

impl std::fmt::Debug for DndIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pixels {
                width,
                height,
                scale,
                ..
            } => f
                .debug_struct("DndIcon::Pixels")
                .field("width", width)
                .field("height", height)
                .field("scale", scale)
                .finish(),
            Self::Element(_) => f
                .debug_tuple("DndIcon::Element")
                .field(&"<opaque>")
                .finish(),
        }
    }
}

/// A type-erased iced [`Element`] that can be rendered offscreen to
/// produce DnD icon pixels.
///
/// Created via [`DndIconSurface::new`] on the application/widget side
/// and downcast back via [`DndIconSurface::downcast`] in the runtime.
pub struct DndIconSurface {
    inner: Box<dyn std::any::Any>,
}

impl DndIconSurface {
    /// Wrap an iced Element (mapped to message type `()`) together with
    /// its initial widget tree state.
    pub fn new<Theme: 'static, Renderer: 'static>(
        element: crate::Element<'static, (), Theme, Renderer>,
        state: crate::widget::tree::State,
    ) -> Self {
        Self {
            inner: Box::new(DndIconSurfaceInner::<Theme, Renderer> { element, state }),
        }
    }

    /// Downcast back to the concrete types.  Returns `None` if the
    /// type parameters don't match.
    pub fn downcast<Theme: 'static, Renderer: 'static>(
        self,
    ) -> Option<(
        crate::Element<'static, (), Theme, Renderer>,
        crate::widget::tree::State,
    )> {
        self.inner
            .downcast::<DndIconSurfaceInner<Theme, Renderer>>()
            .ok()
            .map(|inner| (inner.element, inner.state))
    }
}

struct DndIconSurfaceInner<Theme, Renderer> {
    element: crate::Element<'static, (), Theme, Renderer>,
    state: crate::widget::tree::State,
}

/// A drag-and-drop request that widgets can issue through [`Shell`].
///
/// [`Shell`]: crate::Shell
pub enum Request {
    /// Start a Wayland drag-and-drop operation.
    ///
    /// The runtime will create a `wl_data_source`, offer the given
    /// MIME types, and call `wl_data_device.start_drag(…)`.
    StartDrag {
        /// Whether this is an internal-only drag (same application).
        /// When `true`, the runtime may short-circuit data transfer.
        internal: bool,
        /// MIME types the source is willing to provide.
        mime_types: Vec<String>,
        /// Supported DnD actions.
        actions: DndAction,
        /// Opaque drag data, pre-serialized per MIME type.
        ///
        /// The outer `Vec` parallels `mime_types` — entry `i` is the
        /// data for `mime_types[i]`.  If empty, the runtime will send
        /// a `SourceEvent::SendRequested` when the destination asks
        /// for data.
        data: Vec<Vec<u8>>,
        /// Optional icon for the drag visual.
        ///
        /// If `None`, the runtime provides a default generic icon.
        icon: Option<DndIcon>,
    },

    /// Accept the current drag offer for a specific MIME type.
    ///
    /// This tells the compositor (and the drag source) which MIME type
    /// the destination wants.  Pass `None` to reject the current
    /// serial.
    AcceptMimeType(Option<String>),

    /// Set the accepted DnD actions for the current offer.
    SetActions {
        /// Bitfield of accepted actions.
        actions: DndAction,
        /// The preferred action.
        preferred: DndAction,
    },

    /// Request the data for a given MIME type from the current offer.
    ///
    /// The runtime will call `wl_data_offer.receive(mime_type, fd)`
    /// and deliver the result as [`Event::DataReceived`].
    RequestData {
        /// The MIME type to request.
        mime_type: String,
    },

    /// Signal that the destination has finished processing the drop.
    ///
    /// Must be called after [`Event::Drop`] once all data has been
    /// read and applied.
    FinishDnd,

    /// Cancel / end the current DnD operation from the source side.
    EndDnd,
}

impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartDrag {
                internal,
                mime_types,
                actions,
                icon,
                ..
            } => f
                .debug_struct("StartDrag")
                .field("internal", internal)
                .field("mime_types", mime_types)
                .field("actions", actions)
                .field("icon", icon)
                .finish(),
            Self::AcceptMimeType(m) => f.debug_tuple("AcceptMimeType").field(m).finish(),
            Self::SetActions { actions, preferred } => f
                .debug_struct("SetActions")
                .field("actions", actions)
                .field("preferred", preferred)
                .finish(),
            Self::RequestData { mime_type } => f
                .debug_struct("RequestData")
                .field("mime_type", mime_type)
                .finish(),
            Self::FinishDnd => write!(f, "FinishDnd"),
            Self::EndDnd => write!(f, "EndDnd"),
        }
    }
}
