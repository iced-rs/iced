use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{collections::hash_map::DefaultHasher, fmt};

use iced_core::Rectangle;
use iced_futures::MaybeSend;
use sctk::reexports::protocols::xdg::shell::client::xdg_positioner::{
    Anchor, Gravity,
};

use crate::window;
/// Popup creation details
#[derive(Debug, Clone)]
pub struct SctkPopupSettings {
    /// XXX must be unique, id of the parent
    pub parent: window::Id,
    /// XXX must be unique, id of the popup
    pub id: window::Id,
    /// positioner of the popup
    pub positioner: SctkPositioner,
    /// optional parent size, must be correct if specified or the behavior is undefined
    pub parent_size: Option<(u32, u32)>,
    /// whether a grab should be requested for the popup after creation
    pub grab: bool,
}

impl Hash for SctkPopupSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Positioner of a popup
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SctkPositioner {
    /// size of the popup
    pub size: (u32, u32),
    /// the rectangle which the popup will be anchored to
    pub anchor_rect: Rectangle<i32>,
    /// the anchor location on the popup
    pub anchor: Anchor,
    /// the gravity of the popup
    pub gravity: Gravity,
    /// the constraint adjustment,
    /// Specify how the window should be positioned if the originally intended position caused the surface to be constrained, meaning at least partially outside positioning boundaries set by the compositor. The adjustment is set by constructing a bitmask describing the adjustment to be made when the surface is constrained on that axis.
    /// If no bit for one axis is set, the compositor will assume that the child surface should not change its position on that axis when constrained.
    ///
    /// If more than one bit for one axis is set, the order of how adjustments are applied is specified in the corresponding adjustment descriptions.
    ///
    /// The default adjustment is none.
    pub constraint_adjustment: u32,
    /// offset of the popup
    pub offset: (i32, i32),
    /// whether the popup is reactive
    pub reactive: bool,
}

impl Hash for SctkPositioner {
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
    }
}

impl Default for SctkPositioner {
    fn default() -> Self {
        Self {
            size: (200, 100),
            anchor_rect: Rectangle {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            anchor: Anchor::None,
            gravity: Gravity::None,
            constraint_adjustment: Default::default(),
            offset: Default::default(),
            reactive: true,
        }
    }
}

#[derive(Clone)]
/// Window Action
pub enum Action<T> {
    /// create a window and receive a message with its Id
    Popup {
        /// popup
        popup: SctkPopupSettings,
        /// phantom
        _phantom: PhantomData<T>,
    },
    /// destroy the popup
    Destroy {
        /// id of the popup
        id: window::Id,
    },
    /// request that the popup be repositioned
    Reposition {
        /// id of the popup
        id: window::Id,
        /// the positioner
        positioner: SctkPositioner,
    },
    /// request that the popup make an explicit grab
    Grab {
        /// id of the popup
        id: window::Id,
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
            Action::Popup { popup, .. } => Action::Popup {
                popup,
                _phantom: PhantomData::default(),
            },
            Action::Destroy { id } => Action::Destroy { id },
            Action::Reposition { id, positioner } => {
                Action::Reposition { id, positioner }
            }
            Action::Grab { id } => Action::Grab { id },
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Popup { popup, .. } => write!(
                f,
                "Action::PopupAction::Popup {{ popup: {:?} }}",
                popup
            ),
            Action::Destroy { id } => write!(
                f,
                "Action::PopupAction::Destroy {{ id: {:?} }}",
                id
            ),
            Action::Reposition { id, positioner } => write!(
                f,
                "Action::PopupAction::Reposition {{ id: {:?}, positioner: {:?} }}",
                id, positioner
            ),
            Action::Grab { id } => write!(
                f,
                "Action::PopupAction::Grab {{ id: {:?} }}",
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
