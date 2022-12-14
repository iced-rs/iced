/// Set by the user callback given to the [`EventLoop::run`] method.
///
/// Indicates the desired behavior of the event loop after [`Event::RedrawEventsCleared`] is emitted.
///
/// Defaults to [`Poll`].
///
/// ## Persistency
///
/// Almost every change is persistent between multiple calls to the event loop closure within a
/// given run loop. The only exception to this is [`ExitWithCode`] which, once set, cannot be unset.
/// Changes are **not** persistent between multiple calls to `run_return` - issuing a new call will
/// reset the control flow to [`Poll`].
///
/// [`ExitWithCode`]: Self::ExitWithCode
/// [`Poll`]: Self::Poll
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ControlFlow {
    /// When the current loop iteration finishes, immediately begin a new iteration regardless of
    /// whether or not new events are available to process.
    ///
    /// ## Platform-specific
    ///
    /// - **Web:** Events are queued and usually sent when `requestAnimationFrame` fires but sometimes
    ///   the events in the queue may be sent before the next `requestAnimationFrame` callback, for
    ///   example when the scaling of the page has changed. This should be treated as an implementation
    ///   detail which should not be relied on.
    Poll,
    /// When the current loop iteration finishes, suspend the thread until another event arrives.
    Wait,
    /// When the current loop iteration finishes, suspend the thread until either another event
    /// arrives or the given time is reached.
    ///
    /// Useful for implementing efficient timers. Applications which want to render at the display's
    /// native refresh rate should instead use [`Poll`] and the VSync functionality of a graphics API
    /// to reduce odds of missed frames.
    ///
    /// [`Poll`]: Self::Poll
    WaitUntil(std::time::Instant),
    /// Send a [`LoopDestroyed`] event and stop the event loop. This variant is *sticky* - once set,
    /// `control_flow` cannot be changed from `ExitWithCode`, and any future attempts to do so will
    /// result in the `control_flow` parameter being reset to `ExitWithCode`.
    ///
    /// The contained number will be used as exit code. The [`Exit`] constant is a shortcut for this
    /// with exit code 0.
    ///
    /// ## Platform-specific
    ///
    /// - **Android / iOS / WASM:** The supplied exit code is unused.
    /// - **Unix:** On most Unix-like platforms, only the 8 least significant bits will be used,
    ///   which can cause surprises with negative exit values (`-42` would end up as `214`). See
    ///   [`std::process::exit`].
    ///
    /// [`LoopDestroyed`]: Event::LoopDestroyed
    /// [`Exit`]: ControlFlow::Exit
    ExitWithCode(i32),
}
