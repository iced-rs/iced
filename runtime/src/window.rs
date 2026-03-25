//! Build window-based GUI applications.
use crate::core::time::Instant;
use crate::core::window::{
    Direction, Event, Icon, Id, Level, Mode, Screenshot, Settings, UserAttention,
};
use crate::core::{Point, Size};
use crate::futures::Subscription;
use crate::futures::event;
use crate::futures::futures::channel::oneshot;
use crate::task::{self, Task};

pub use raw_window_handle;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// An operation to be performed on some window.
pub enum Action {
    /// Open a new window with some [`Settings`].
    Open(Id, Settings, oneshot::Sender<Id>),

    /// Close the window and exits the application.
    Close(Id),

    /// Gets the [`Id`] of the oldest window.
    GetOldest(oneshot::Sender<Option<Id>>),

    /// Gets the [`Id`] of the latest window.
    GetLatest(oneshot::Sender<Option<Id>>),

    /// Move the window with the left mouse button until the button is
    /// released.
    ///
    /// There's no guarantee that this will work unless the left mouse
    /// button was pressed immediately before this function is called.
    Drag(Id),

    /// Resize the window with the left mouse button until the button is
    /// released.
    ///
    /// There's no guarantee that this will work unless the left mouse
    /// button was pressed immediately before this function is called.
    DragResize(Id, Direction),

    /// Resize the window to the given logical dimensions.
    Resize(Id, Size),

    /// Resize the window with animation (COSMIC compositor protocol).
    /// Parameters are (id, width, height, duration_ms).
    AnimatedResize(Id, u32, u32, u32),

    /// Resize the window with animation and explicit position (COSMIC compositor protocol).
    /// Parameters are (id, x, y, width, height, duration_ms).
    /// If the window is maximized, the position and size will be stored and used when restored.
    AnimatedResizeWithPosition(Id, i32, i32, u32, u32, u32),

    /// Embed a toplevel by process ID into the window's surface (COSMIC compositor protocol).
    /// Parameters are (window_id, pid, app_id, x, y, width, height, interactive, result_sender).
    /// Returns the embed ID via the sender, or None if the protocol is not available.
    EmbedToplevelByPid(
        Id,
        u32,
        String,
        i32,
        i32,
        i32,
        i32,
        bool,
        oneshot::Sender<Option<u64>>,
    ),

    /// Update the geometry of an embedded surface (COSMIC compositor protocol).
    /// Parameters are (window_id, embed_id, x, y, width, height).
    SetEmbedGeometry(Id, u64, i32, i32, i32, i32),

    /// Set anchor-based positioning for an embedded surface (COSMIC compositor protocol).
    /// Parameters are (window_id, embed_id, anchor, margin_top, margin_right, margin_bottom, margin_left, width, height).
    /// Anchor is a bitfield: 0=none, 1=top, 2=bottom, 4=left, 8=right.
    SetEmbedAnchor(Id, u64, u32, i32, i32, i32, i32, i32, i32),

    /// Set corner radius for an embedded surface (COSMIC compositor protocol).
    /// Parameters are (window_id, embed_id, top_left, top_right, bottom_right, bottom_left).
    SetEmbedCornerRadius(Id, u64, u32, u32, u32, u32),

    /// Set interactivity for an embedded surface (COSMIC compositor protocol).
    /// Parameters are (window_id, embed_id, interactive).
    SetEmbedInteractive(Id, u64, bool),

    /// Remove an embedded surface (COSMIC compositor protocol).
    /// Parameters are (window_id, embed_id).
    RemoveEmbed(Id, u64),

    /// Get the current logical dimensions of the window.
    GetSize(Id, oneshot::Sender<Size>),

    /// Get if the current window is maximized or not.
    GetMaximized(Id, oneshot::Sender<bool>),

    /// Set the window to maximized or back
    Maximize(Id, bool),

    /// Get if the current window is minimized or not.
    ///
    /// ## Platform-specific
    /// - **Wayland:** Always `None`.
    GetMinimized(Id, oneshot::Sender<Option<bool>>),

    /// Set the window to minimized or back
    Minimize(Id, bool),

    /// Get the current logical coordinates of the window.
    GetPosition(Id, oneshot::Sender<Option<Point>>),

    /// Get the current scale factor (DPI) of the window.
    GetScaleFactor(Id, oneshot::Sender<f32>),

    /// Move the window to the given logical coordinates.
    ///
    /// Unsupported on Wayland.
    Move(Id, Point),

    /// Change the [`Mode`] of the window.
    SetMode(Id, Mode),

    /// Get the current [`Mode`] of the window.
    GetMode(Id, oneshot::Sender<Mode>),

    /// Toggle the window to maximized or back
    ToggleMaximize(Id),

    /// Toggle whether window has decorations.
    ///
    /// ## Platform-specific
    /// - **X11:** Not implemented.
    /// - **Web:** Unsupported.
    ToggleDecorations(Id),

    /// Request user attention to the window, this has no effect if the application
    /// is already focused. How requesting for user attention manifests is platform dependent,
    /// see [`UserAttention`] for details.
    ///
    /// Providing `None` will unset the request for user attention. Unsetting the request for
    /// user attention might not be done automatically by the WM when the window receives input.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    /// - **macOS:** `None` has no effect.
    /// - **X11:** Requests for user attention must be manually cleared.
    /// - **Wayland:** Requires `xdg_activation_v1` protocol, `None` has no effect.
    RequestUserAttention(Id, Option<UserAttention>),

    /// Bring the window to the front and sets input focus. Has no effect if the window is
    /// already in focus, minimized, or not visible.
    ///
    /// This method steals input focus from other applications. Do not use this method unless
    /// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
    /// user experience.
    ///
    /// ## Platform-specific
    ///
    /// - **Web / Wayland:** Unsupported.
    GainFocus(Id),

    /// Change the window [`Level`].
    SetLevel(Id, Level),

    /// Show the system menu at cursor position.
    ///
    /// ## Platform-specific
    /// Android / iOS / macOS / Orbital / Web / X11: Unsupported.
    ShowSystemMenu(Id),

    /// Get the raw identifier unique to the window.
    GetRawId(Id, oneshot::Sender<u64>),

    /// Change the window [`Icon`].
    ///
    /// On Windows and X11, this is typically the small icon in the top-left
    /// corner of the titlebar.
    ///
    /// ## Platform-specific
    ///
    /// - **Web / Wayland / macOS:** Unsupported.
    ///
    /// - **Windows:** Sets `ICON_SMALL`. The base size for a window icon is 16x16, but it's
    ///   recommended to account for screen scaling and pick a multiple of that, i.e. 32x32.
    ///
    /// - **X11:** Has no universal guidelines for icon sizes, so you're at the whims of the WM. That
    ///   said, it's usually in the same ballpark as on Windows.
    SetIcon(Id, Icon),

    /// Runs the closure with a reference to the [`Window`] with the given [`Id`].
    Run(Id, Box<dyn FnOnce(&dyn Window) + Send>),

    /// Screenshot the viewport of the window.
    Screenshot(Id, oneshot::Sender<Screenshot>),

    /// Enable mouse passthrough for the given window.
    ///
    /// This disables mouse events for the window and passes mouse events
    /// through to whatever window is underneath.
    EnableMousePassthrough(Id),

    /// Disable mouse passthrough for the given window.
    ///
    /// This enables mouse events for the window and stops mouse events
    /// from being passed to whatever is underneath.
    DisableMousePassthrough(Id),

    /// Set the minimum inner window size.
    SetMinSize(Id, Option<Size>),

    /// Set the maximum inner window size.
    SetMaxSize(Id, Option<Size>),

    /// Set the window to be resizable or not.
    SetResizable(Id, bool),

    /// Set the window size increment.
    SetResizeIncrements(Id, Option<Size>),

    /// Get the logical dimensions of the monitor containing the window with the given [`Id`].
    GetMonitorSize(Id, oneshot::Sender<Option<Size>>),

    /// Set whether the system can automatically organize windows into tabs.
    ///
    /// See <https://developer.apple.com/documentation/appkit/nswindow/1646657-allowsautomaticwindowtabbing>
    SetAllowAutomaticTabbing(bool),

    /// Redraw all the windows.
    RedrawAll,

    /// Recompute the layouts of all the windows.
    RelayoutAll,

    /// Set exclusive mode for the window (COSMIC compositor protocol).
    /// When enabled, all other windows are hidden. When disabled, other windows are restored.
    /// Parameters are (window_id, exclusive).
    SetExclusiveMode(Id, bool),

    /// Set corner radius for the window (COSMIC compositor protocol).
    /// Communicates corner radius hints to the compositor for blur outlines and rounded corners.
    /// Parameters are (window_id, top_left, top_right, bottom_right, bottom_left).
    SetCornerRadius(Id, u32, u32, u32, u32),

    /// Set a compositor-rendered backdrop color for the window.
    /// The compositor renders a colored rectangle behind the window content,
    /// respecting the window's corner radius.
    /// Parameters are (window_id, r, g, b, a) where each component is 0-255.
    SetBackdropColor(Id, u32, u32, u32, u32),

    /// Register the window to receive voice mode events (COSMIC compositor protocol).
    /// Parameters are (window_id, is_default_receiver).
    /// When is_default_receiver is true, this window receives events when no other receiver is active.
    RegisterVoiceMode(Id, bool),

    /// Unregister the window from receiving voice mode events (COSMIC compositor protocol).
    UnregisterVoiceMode(Id),

    /// Acknowledge a will_stop event from the compositor (COSMIC compositor protocol).
    /// If freeze is true, the orb will freeze in place for processing.
    /// If freeze is false, the orb will proceed with hiding.
    VoiceAckStop(Id, u32, bool),

    /// Dismiss the frozen voice orb (COSMIC compositor protocol).
    /// Used when transcription completes without spawning a new window.
    VoiceDismiss(Id),
}

/// A window managed by iced.
///
/// It implements both [`HasWindowHandle`] and [`HasDisplayHandle`].
pub trait Window: HasWindowHandle + HasDisplayHandle {}

impl<T> Window for T where T: HasWindowHandle + HasDisplayHandle {}

/// Subscribes to the frames of the window of the running application.
///
/// The resulting [`Subscription`] will produce items at a rate equal to the
/// refresh rate of the first application window. Note that this rate may be variable, as it is
/// normally managed by the graphics driver and/or the OS.
///
/// In any case, this [`Subscription`] is useful to smoothly draw application-driven
/// animations without missing any frames.
pub fn frames() -> Subscription<Instant> {
    event::listen_raw(|event, _status, _window| match event {
        crate::core::Event::Window(Event::RedrawRequested(at)) => Some(at),
        _ => None,
    })
}

/// Subscribes to all window events of the running application.
pub fn events() -> Subscription<(Id, Event)> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(event) = event {
            Some((id, event))
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::Opened`] occurrences in the running application.
pub fn open_events() -> Subscription<Id> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::Opened { .. }) = event {
            Some(id)
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::Closed`] occurrences in the running application.
pub fn close_events() -> Subscription<Id> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::Closed) = event {
            Some(id)
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::Resized`] occurrences in the running application.
pub fn resize_events() -> Subscription<(Id, Size)> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::Resized(size)) = event {
            Some((id, size))
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::CloseRequested`] occurrences in the running application.
pub fn close_requests() -> Subscription<Id> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::CloseRequested) = event {
            Some(id)
        } else {
            None
        }
    })
}

/// Subscribes to all voice mode events in the running application.
///
/// Returns the window ID and the voice mode event. You must call
/// [`register_voice_mode`] on a window before it will receive these events.
pub fn voice_mode_events() -> Subscription<(Id, crate::core::voice_mode::Event)> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::VoiceMode(voice_event) = event {
            Some((id, voice_event))
        } else {
            None
        }
    })
}

/// Opens a new window with the given [`Settings`]; producing the [`Id`]
/// of the new window on completion.
pub fn open(settings: Settings) -> (Id, Task<Id>) {
    let id = Id::unique();

    (
        id,
        task::oneshot(|channel| crate::Action::Window(Action::Open(id, settings, channel))),
    )
}

/// Closes the window with `id`.
pub fn close<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::Close(id)))
}

/// Gets the window [`Id`] of the oldest window.
pub fn oldest() -> Task<Option<Id>> {
    task::oneshot(|channel| crate::Action::Window(Action::GetOldest(channel)))
}

/// Gets the window [`Id`] of the latest window.
pub fn latest() -> Task<Option<Id>> {
    task::oneshot(|channel| crate::Action::Window(Action::GetLatest(channel)))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::Drag(id)))
}

/// Begins resizing the window while the left mouse button is held.
pub fn drag_resize<T>(id: Id, direction: Direction) -> Task<T> {
    task::effect(crate::Action::Window(Action::DragResize(id, direction)))
}

/// Resizes the window to the given logical dimensions.
pub fn resize<T>(id: Id, new_size: Size) -> Task<T> {
    task::effect(crate::Action::Window(Action::Resize(id, new_size)))
}

/// Resizes the window with animation using the COSMIC animated resize protocol.
///
/// This requests the compositor to smoothly animate the window from its current
/// size to the target size over the specified duration.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_animated_resize_v1` protocol.
/// - **Other platforms:** Falls back to instant resize.
pub fn animated_resize<T>(id: Id, width: u32, height: u32, duration_ms: u32) -> Task<T> {
    task::effect(crate::Action::Window(Action::AnimatedResize(
        id,
        width,
        height,
        duration_ms,
    )))
}

/// Resizes the window with animation and explicit position using the COSMIC animated resize protocol.
///
/// This requests the compositor to smoothly animate the window from its current
/// geometry to the target position and size over the specified duration.
///
/// If the window is maximized, the position and size will be stored and used
/// when the window is restored to normal state.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_animated_resize_v1` protocol.
/// - **Other platforms:** Falls back to instant resize (position ignored).
pub fn animated_resize_with_position<T>(
    id: Id,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    duration_ms: u32,
) -> Task<T> {
    task::effect(crate::Action::Window(Action::AnimatedResizeWithPosition(
        id,
        x,
        y,
        width,
        height,
        duration_ms,
    )))
}

/// Embed a toplevel by process ID into this window's surface.
///
/// This uses the `zcosmic_surface_embed_manager_v1` protocol to embed a
/// foreign toplevel window within this window's surface. The compositor
/// will monitor for new toplevels from the specified PID and embed the
/// first matching one.
///
/// Returns a [`Task`] that resolves to the embed ID (which can be used to
/// update geometry or remove the embed), or `None` if the protocol is not
/// available.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_surface_embed_manager_v1` protocol.
/// - **Other platforms:** Always returns `None`.
///
/// # Arguments
/// * `id` - Window ID to embed into
/// * `pid` - Process ID of the application to embed
/// * `app_id` - Optional app_id hint for verification (can be empty)
/// * `x` - X position within this window's surface
/// * `y` - Y position within this window's surface
/// * `width` - Width of the embed region
/// * `height` - Height of the embed region
/// * `interactive` - Whether input should be routed to the embedded surface
#[allow(clippy::too_many_arguments)]
pub fn embed_toplevel_by_pid(
    id: Id,
    pid: u32,
    app_id: impl Into<String>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    interactive: bool,
) -> Task<Option<u64>> {
    task::oneshot(move |channel| {
        crate::Action::Window(Action::EmbedToplevelByPid(
            id,
            pid,
            app_id.into(),
            x,
            y,
            width,
            height,
            interactive,
            channel,
        ))
    })
}

/// Update the geometry of an embedded surface.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_surface_embed_manager_v1` protocol.
/// - **Other platforms:** No-op.
pub fn set_embed_geometry<T>(
    id: Id,
    embed_id: u64,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetEmbedGeometry(
        id, embed_id, x, y, width, height,
    )))
}

/// Set anchor-based positioning for an embedded surface.
///
/// Instead of specifying absolute (x, y) coordinates, this allows
/// positioning relative to the parent window edges. The geometry is
/// automatically recalculated by the compositor when the parent window resizes.
///
/// # Arguments
/// * `id` - The parent window ID
/// * `embed_id` - The embedded surface ID
/// * `anchor` - Bitflags indicating which edges to anchor to:
///   - 0: none (use absolute positioning)
///   - 1: top
///   - 2: bottom
///   - 4: left
///   - 8: right
///   - Combinations like 9 (top | right) are valid
/// * `margin_top` - Margin from top edge (when anchored to top)
/// * `margin_right` - Margin from right edge (when anchored to right)
/// * `margin_bottom` - Margin from bottom edge (when anchored to bottom)
/// * `margin_left` - Margin from left edge (when anchored to left)
/// * `width` - Width of embed region (0 to stretch between left/right anchors)
/// * `height` - Height of embed region (0 to stretch between top/bottom anchors)
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_surface_embed_manager_v1` protocol.
/// - **Other platforms:** No-op.
pub fn set_embed_anchor<T>(
    id: Id,
    embed_id: u64,
    anchor: u32,
    margin_top: i32,
    margin_right: i32,
    margin_bottom: i32,
    margin_left: i32,
    width: i32,
    height: i32,
) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetEmbedAnchor(
        id,
        embed_id,
        anchor,
        margin_top,
        margin_right,
        margin_bottom,
        margin_left,
        width,
        height,
    )))
}

/// Set corner radius for an embedded surface.
///
/// This allows the parent to specify rounded corners that match its own UI.
/// Each corner can have a different radius. Values are in logical pixels.
/// A value of 0 means no rounding for that corner.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_surface_embed_manager_v1` protocol.
/// - **Other platforms:** No-op.
pub fn set_embed_corner_radius<T>(
    id: Id,
    embed_id: u64,
    top_left: u32,
    top_right: u32,
    bottom_right: u32,
    bottom_left: u32,
) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetEmbedCornerRadius(
        id,
        embed_id,
        top_left,
        top_right,
        bottom_right,
        bottom_left,
    )))
}

/// Set interactivity for an embedded surface.
///
/// When interactive, pointer/keyboard/touch events within the embed
/// region will be routed to the embedded toplevel.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_surface_embed_manager_v1` protocol.
/// - **Other platforms:** No-op.
pub fn set_embed_interactive<T>(id: Id, embed_id: u64, interactive: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetEmbedInteractive(
        id,
        embed_id,
        interactive,
    )))
}

/// Remove an embedded surface.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_surface_embed_manager_v1` protocol.
/// - **Other platforms:** No-op.
pub fn remove_embed<T>(id: Id, embed_id: u64) -> Task<T> {
    task::effect(crate::Action::Window(Action::RemoveEmbed(id, embed_id)))
}

/// Set the window to be resizable or not.
pub fn set_resizable<T>(id: Id, resizable: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetResizable(id, resizable)))
}

/// Set the inner maximum size of the window.
pub fn set_max_size<T>(id: Id, size: Option<Size>) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetMaxSize(id, size)))
}

/// Set the inner minimum size of the window.
pub fn set_min_size<T>(id: Id, size: Option<Size>) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetMinSize(id, size)))
}

/// Set the window size increment.
///
/// This is usually used by apps such as terminal emulators that need "blocky" resizing.
pub fn set_resize_increments<T>(id: Id, increments: Option<Size>) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetResizeIncrements(
        id, increments,
    )))
}

/// Gets the window size in logical dimensions.
pub fn size(id: Id) -> Task<Size> {
    task::oneshot(move |channel| crate::Action::Window(Action::GetSize(id, channel)))
}

/// Gets the maximized state of the window with the given [`Id`].
pub fn is_maximized(id: Id) -> Task<bool> {
    task::oneshot(move |channel| crate::Action::Window(Action::GetMaximized(id, channel)))
}

/// Maximizes the window.
pub fn maximize<T>(id: Id, maximized: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::Maximize(id, maximized)))
}

/// Gets the minimized state of the window with the given [`Id`].
pub fn is_minimized(id: Id) -> Task<Option<bool>> {
    task::oneshot(move |channel| crate::Action::Window(Action::GetMinimized(id, channel)))
}

/// Minimizes the window.
pub fn minimize<T>(id: Id, minimized: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::Minimize(id, minimized)))
}

/// Gets the position in logical coordinates of the window with the given [`Id`].
pub fn position(id: Id) -> Task<Option<Point>> {
    task::oneshot(move |channel| crate::Action::Window(Action::GetPosition(id, channel)))
}

/// Gets the scale factor of the window with the given [`Id`].
pub fn scale_factor(id: Id) -> Task<f32> {
    task::oneshot(move |channel| crate::Action::Window(Action::GetScaleFactor(id, channel)))
}

/// Moves the window to the given logical coordinates.
pub fn move_to<T>(id: Id, position: Point) -> Task<T> {
    task::effect(crate::Action::Window(Action::Move(id, position)))
}

/// Gets the current [`Mode`] of the window.
pub fn mode(id: Id) -> Task<Mode> {
    task::oneshot(move |channel| crate::Action::Window(Action::GetMode(id, channel)))
}

/// Changes the [`Mode`] of the window.
pub fn set_mode<T>(id: Id, mode: Mode) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetMode(id, mode)))
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::ToggleMaximize(id)))
}

/// Toggles the window decorations.
pub fn toggle_decorations<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::ToggleDecorations(id)))
}

/// Requests user attention to the window. This has no effect if the application
/// is already focused. How requesting for user attention manifests is platform dependent,
/// see [`UserAttention`] for details.
///
/// Providing `None` will unset the request for user attention. Unsetting the request for
/// user attention might not be done automatically by the WM when the window receives input.
pub fn request_user_attention<T>(id: Id, user_attention: Option<UserAttention>) -> Task<T> {
    task::effect(crate::Action::Window(Action::RequestUserAttention(
        id,
        user_attention,
    )))
}

/// Brings the window to the front and sets input focus. Has no effect if the window is
/// already in focus, minimized, or not visible.
///
/// This [`Task`] steals input focus from other applications. Do not use this method unless
/// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
/// user experience.
pub fn gain_focus<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::GainFocus(id)))
}

/// Changes the window [`Level`].
pub fn set_level<T>(id: Id, level: Level) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetLevel(id, level)))
}

/// Shows the [system menu] at cursor position.
///
/// [system menu]: https://en.wikipedia.org/wiki/Common_menus_in_Microsoft_Windows#System_menu
pub fn show_system_menu<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::ShowSystemMenu(id)))
}

/// Gets an identifier unique to the window, provided by the underlying windowing system. This is
/// not to be confused with [`Id`].
pub fn raw_id<Message>(id: Id) -> Task<u64> {
    task::oneshot(|channel| crate::Action::Window(Action::GetRawId(id, channel)))
}

/// Changes the [`Icon`] of the window.
pub fn set_icon<T>(id: Id, icon: Icon) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetIcon(id, icon)))
}

/// Runs the given callback with a reference to the [`Window`] with the given [`Id`].
///
/// Note that if the window closes before this call is processed the callback will not be run.
pub fn run<T>(id: Id, f: impl FnOnce(&dyn Window) -> T + Send + 'static) -> Task<T>
where
    T: Send + 'static,
{
    task::oneshot(move |channel| {
        crate::Action::Window(Action::Run(
            id,
            Box::new(move |handle| {
                let _ = channel.send(f(handle));
            }),
        ))
    })
}

/// Captures a [`Screenshot`] from the window.
pub fn screenshot(id: Id) -> Task<Screenshot> {
    task::oneshot(move |channel| crate::Action::Window(Action::Screenshot(id, channel)))
}

/// Enables mouse passthrough for the given window.
///
/// This disables mouse events for the window and passes mouse events
/// through to whatever window is underneath.
pub fn enable_mouse_passthrough<Message>(id: Id) -> Task<Message> {
    task::effect(crate::Action::Window(Action::EnableMousePassthrough(id)))
}

/// Disables mouse passthrough for the given window.
///
/// This enables mouse events for the window and stops mouse events
/// from being passed to whatever is underneath.
pub fn disable_mouse_passthrough<Message>(id: Id) -> Task<Message> {
    task::effect(crate::Action::Window(Action::DisableMousePassthrough(id)))
}

/// Gets the logical dimensions of the monitor containing the window with the given [`Id`].
pub fn monitor_size(id: Id) -> Task<Option<Size>> {
    task::oneshot(move |channel| crate::Action::Window(Action::GetMonitorSize(id, channel)))
}

/// Sets whether the system can automatically organize windows into tabs.
///
/// See <https://developer.apple.com/documentation/appkit/nswindow/1646657-allowsautomaticwindowtabbing>
pub fn allow_automatic_tabbing<T>(enabled: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetAllowAutomaticTabbing(
        enabled,
    )))
}

/// Sets exclusive mode for the window using the COSMIC exclusive mode protocol.
///
/// When enabled, all other windows on the screen are hidden/minimized.
/// When disabled, previously hidden windows are restored.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_exclusive_mode_v1` protocol.
/// - **Other platforms:** No effect.
pub fn set_exclusive_mode<T>(id: Id, exclusive: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetExclusiveMode(
        id, exclusive,
    )))
}

/// Sets the corner radius for the window using the COSMIC corner radius protocol.
///
/// Communicates corner radius hints to the compositor so it can draw proper
/// blur outlines and apply rounded corners.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `layer_corner_radius_manager_v1` protocol.
/// - **Other platforms:** No effect.
pub fn set_corner_radius<T>(
    id: Id,
    top_left: u32,
    top_right: u32,
    bottom_right: u32,
    bottom_left: u32,
) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetCornerRadius(
        id,
        top_left,
        top_right,
        bottom_right,
        bottom_left,
    )))
}

/// Sets a compositor-rendered backdrop color for the window.
///
/// The compositor will render a colored rectangle behind the window content,
/// using the window's corner radius. RGBA components are in the range 0-255.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `backdrop_color_manager_v1` protocol.
/// - **Other platforms:** No effect.
pub fn set_backdrop_color<T>(id: Id, r: u32, g: u32, b: u32, a: u32) -> Task<T> {
    task::effect(crate::Action::Window(Action::SetBackdropColor(
        id, r, g, b, a,
    )))
}

/// Registers the window to receive voice mode events from the compositor.
///
/// When `is_default_receiver` is true, this window will receive voice mode events
/// even when it doesn't have focus. Only one window should be the default receiver.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_voice_mode_v1` protocol.
/// - **Other platforms:** No effect.
pub fn register_voice_mode<T>(id: Id, is_default_receiver: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::RegisterVoiceMode(
        id,
        is_default_receiver,
    )))
}

/// Unregisters the window from receiving voice mode events.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_voice_mode_v1` protocol.
/// - **Other platforms:** No effect.
pub fn unregister_voice_mode<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::UnregisterVoiceMode(id)))
}

/// Acknowledges a will_stop event from the compositor.
///
/// This responds to a will_stop event, telling the compositor whether to
/// freeze the orb (transcription processing) or proceed with hiding.
///
/// ## Arguments
/// * `id` - The window ID of the voice mode receiver
/// * `serial` - The serial from the will_stop event
/// * `freeze` - If true, freeze the orb in place. If false, proceed with hiding.
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_voice_mode_v1` protocol's `ack_stop` request.
/// - **Other platforms:** No effect.
pub fn voice_ack_stop<T>(id: Id, serial: u32, freeze: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::VoiceAckStop(
        id, serial, freeze,
    )))
}

/// Dismisses the frozen voice orb.
///
/// This tells the compositor to hide the orb when transcription completes
/// without spawning a new window (e.g., empty result or error).
///
/// ## Arguments
/// * `id` - The window ID of the voice mode receiver
///
/// ## Platform-specific
/// - **COSMIC/Wayland:** Uses `zcosmic_voice_mode_v1` protocol's `dismiss` request.
/// - **Other platforms:** No effect.
pub fn voice_dismiss<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::VoiceDismiss(id)))
}
