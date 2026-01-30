//! Voice mode events from the compositor.
//!
//! This module provides events related to voice input mode, including
//! start, stop, cancel events and orb attachment state changes.

/// Voice mode orb display state from the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbState {
    /// Orb is not visible.
    Hidden,
    /// Orb is floating (default receiver active).
    Floating,
    /// Orb is attached to a window.
    Attached,
    /// Orb is frozen in place (processing).
    Frozen,
    /// Orb is transitioning to attached (non-interruptible).
    Transitioning,
}

/// Voice mode events sent to registered windows.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Voice input has started. The window should begin recording/listening.
    Started {
        /// Where the orb is displayed.
        orb_state: OrbState,
    },
    /// Voice input stopped normally. Process the recorded audio.
    Stopped,
    /// Voice input was cancelled. Discard any recorded audio.
    Cancelled,
    /// The orb has attached to this window.
    OrbAttached {
        /// Surface x position.
        x: i32,
        /// Surface y position.
        y: i32,
        /// Surface width.
        width: i32,
        /// Surface height.
        height: i32,
    },
    /// The orb has detached from this window.
    OrbDetached,
    /// Voice input is about to stop. Client must respond with ack_stop.
    /// 
    /// The serial must be echoed back in the ack_stop call along with
    /// whether to freeze the orb (transcription processing) or proceed with hiding.
    WillStop {
        /// Serial to echo back in ack_stop.
        serial: u32,
    },
    /// Focus the input field.
    ///
    /// Sent when the user tapped the voice key (short press without holding).
    /// The client should focus its text input field so the user can start typing.
    /// If the surface is not currently visible, bring it to focus first.
    FocusInput,
}
