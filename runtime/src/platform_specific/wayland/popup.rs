//! Popup actions and types for Wayland xdg_popup surfaces.
//!
//! This module provides types for creating and managing popup surfaces
//! that can appear outside the parent window bounds.

use iced_core::Rectangle;
use iced_core::layout::Limits;
use iced_core::window::Id;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Anchor position on the parent surface for popup positioning.
///
/// Based on `xdg_positioner::anchor` from the xdg-shell protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum Anchor {
    /// No anchor (use anchor_rect center)
    #[default]
    None = 0,
    /// Top edge of anchor_rect
    Top = 1,
    /// Bottom edge of anchor_rect
    Bottom = 2,
    /// Left edge of anchor_rect
    Left = 3,
    /// Right edge of anchor_rect
    Right = 4,
    /// Top-left corner of anchor_rect
    TopLeft = 5,
    /// Bottom-left corner of anchor_rect
    BottomLeft = 6,
    /// Top-right corner of anchor_rect
    TopRight = 7,
    /// Bottom-right corner of anchor_rect
    BottomRight = 8,
}

/// Gravity direction for the popup surface.
///
/// Based on `xdg_positioner::gravity` from the xdg-shell protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum Gravity {
    /// No gravity (centered on anchor point)
    #[default]
    None = 0,
    /// Popup above anchor point
    Top = 1,
    /// Popup below anchor point
    Bottom = 2,
    /// Popup left of anchor point
    Left = 3,
    /// Popup right of anchor point
    Right = 4,
    /// Popup above-left of anchor point
    TopLeft = 5,
    /// Popup below-left of anchor point
    BottomLeft = 6,
    /// Popup above-right of anchor point
    TopRight = 7,
    /// Popup below-right of anchor point
    BottomRight = 8,
}

/// Constraint adjustment flags for popup repositioning.
///
/// When the popup would be constrained (partially outside screen bounds),
/// the compositor can adjust its position based on these flags.
pub mod constraint_adjustment {
    /// No adjustment, the popup is clipped or fails
    pub const NONE: u32 = 0;
    /// Slide the popup along the X axis
    pub const SLIDE_X: u32 = 1;
    /// Slide the popup along the Y axis
    pub const SLIDE_Y: u32 = 2;
    /// Flip the anchor and gravity on the X axis
    pub const FLIP_X: u32 = 4;
    /// Flip the anchor and gravity on the Y axis
    pub const FLIP_Y: u32 = 8;
    /// Resize the popup on the X axis
    pub const RESIZE_X: u32 = 16;
    /// Resize the popup on the Y axis
    pub const RESIZE_Y: u32 = 32;
}

/// Settings for creating a popup surface.
#[derive(Debug, Clone)]
pub struct PopupSettings {
    /// ID of the parent surface (must exist)
    pub parent: Id,
    /// Unique ID for this popup
    pub id: Id,
    /// Positioning information for the popup
    pub positioner: Positioner,
    /// Whether to grab keyboard and pointer input
    pub grab: bool,
    /// When true, the popup surface will have an empty input region so
    /// pointer events pass through to the surface below. Useful for tooltips.
    pub input_passthrough: bool,
}

impl Hash for PopupSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Positioner configuration for popup placement.
#[derive(Debug, Clone)]
pub struct Positioner {
    /// Size of the popup (if None, will be auto-sized)
    pub size: Option<(u32, u32)>,
    /// Size limits for auto-sizing
    pub size_limits: Limits,
    /// Rectangle on the parent surface to anchor to
    pub anchor_rect: Rectangle<i32>,
    /// Where on the anchor_rect to attach
    pub anchor: Anchor,
    /// Direction the popup extends from the anchor point
    pub gravity: Gravity,
    /// How to adjust if popup would be constrained
    pub constraint_adjustment: u32,
    /// Offset from the calculated position
    pub offset: (i32, i32),
    /// Whether popup should reposition when parent moves
    pub reactive: bool,
    /// Window geometry (x, y, width, height) in logical pixels.
    ///
    /// Tells the compositor which part of the surface is visible content
    /// (excluding shadows/decorations). The compositor uses this for
    /// constraint adjustment (SLIDE/FLIP) — only the geometry rect is
    /// kept on-screen, not the full surface.
    pub window_geometry: Option<(i32, i32, i32, i32)>,
    /// Shadow padding in logical pixels.
    ///
    /// When set and `size` is None (auto-sizing), iced will automatically
    /// compute `window_geometry` from the measured content size minus this
    /// padding on each side. This tells the compositor to only constrain
    /// the visible content area, not the shadow.
    pub shadow_padding: u32,
}

impl Hash for Positioner {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.anchor_rect.x.hash(state);
        self.anchor_rect.y.hash(state);
        self.anchor_rect.width.hash(state);
        self.anchor_rect.height.hash(state);
        self.anchor.hash(state);
        self.gravity.hash(state);
        self.constraint_adjustment.hash(state);
        self.offset.hash(state);
        self.reactive.hash(state);
        self.window_geometry.hash(state);
        self.shadow_padding.hash(state);
    }
}

impl Default for Positioner {
    fn default() -> Self {
        Self {
            size: None,
            size_limits: Limits::NONE
                .min_height(1.0)
                .min_width(1.0)
                .max_width(400.0)
                .max_height(600.0),
            anchor_rect: Rectangle {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            anchor: Anchor::None,
            gravity: Gravity::None,
            constraint_adjustment: constraint_adjustment::SLIDE_X
                | constraint_adjustment::SLIDE_Y
                | constraint_adjustment::FLIP_X
                | constraint_adjustment::FLIP_Y,
            offset: (0, 0),
            reactive: true,
            window_geometry: None,
            shadow_padding: 0,
        }
    }
}

/// Actions for popup surfaces.
#[derive(Clone)]
pub enum Action {
    /// Create a popup surface
    Show {
        /// Popup configuration
        settings: PopupSettings,
    },
    /// Destroy a popup surface
    Hide {
        /// ID of the popup to destroy
        id: Id,
    },
    /// Resize a popup surface
    Resize {
        /// ID of the popup
        id: Id,
        /// New width
        width: u32,
        /// New height
        height: u32,
    },
    /// Reposition a popup surface using xdg_popup.reposition (v3).
    ///
    /// Updates the popup's positioner and asks the compositor to
    /// reposition it accordingly. Useful for tooltips that follow
    /// the cursor or need to adjust position dynamically.
    Reposition {
        /// ID of the popup to reposition
        id: Id,
        /// New positioner configuration
        positioner: Positioner,
    },
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Show { settings } => f
                .debug_struct("Action::Show")
                .field("settings", settings)
                .finish(),
            Action::Hide { id } => f.debug_struct("Action::Hide").field("id", id).finish(),
            Action::Resize { id, width, height } => f
                .debug_struct("Action::Resize")
                .field("id", id)
                .field("width", width)
                .field("height", height)
                .finish(),
            Action::Reposition { id, positioner } => f
                .debug_struct("Action::Reposition")
                .field("id", id)
                .field("positioner", positioner)
                .finish(),
        }
    }
}

// ============================================================================
// Task Helpers
// ============================================================================

use crate::Task;
use crate::platform_specific;
use crate::task;

/// Show a popup surface.
///
/// The popup will be positioned relative to the parent window using
/// the positioner settings.
pub fn show<Message>(settings: PopupSettings) -> Task<Message> {
    task::effect(crate::Action::PlatformSpecific(
        platform_specific::Action::Wayland(super::Action::Popup(Action::Show { settings })),
    ))
}

/// Hide (destroy) a popup surface.
pub fn hide<Message>(id: Id) -> Task<Message> {
    task::effect(crate::Action::PlatformSpecific(
        platform_specific::Action::Wayland(super::Action::Popup(Action::Hide { id })),
    ))
}

/// Resize a popup surface.
pub fn resize<Message>(id: Id, width: u32, height: u32) -> Task<Message> {
    task::effect(crate::Action::PlatformSpecific(
        platform_specific::Action::Wayland(super::Action::Popup(Action::Resize {
            id,
            width,
            height,
        })),
    ))
}

/// Reposition a popup surface.
///
/// Uses xdg_popup.reposition (protocol v3) to update the popup's
/// position without destroying and recreating it.
pub fn reposition<Message>(id: Id, positioner: Positioner) -> Task<Message> {
    task::effect(crate::Action::PlatformSpecific(
        platform_specific::Action::Wayland(super::Action::Popup(Action::Reposition {
            id,
            positioner,
        })),
    ))
}
