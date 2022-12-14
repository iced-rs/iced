use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::{collections::hash_map::DefaultHasher, fmt};

use iced_futures::MaybeSend;
use sctk::shell::layer::{Anchor, KeyboardInteractivity, Layer};

use crate::window;

/// output for layer surface
#[derive(Debug, Clone)]
pub enum IcedOutput {
    /// show on all outputs
    All,
    /// show on active output
    Active,
    /// show on a specific output
    Output {
        /// make
        make: String,
        /// model
        model: String,
    },
}

impl Default for IcedOutput {
    fn default() -> Self {
        Self::Active
    }
}

/// margins of the layer surface
#[derive(Debug, Clone, Copy, Default)]
pub struct IcedMargin {
    /// top
    pub top: i32,
    /// right
    pub right: i32,
    /// bottom
    pub bottom: i32,
    /// left
    pub left: i32,
}

/// layer surface
#[derive(Debug, Clone)]
pub struct SctkLayerSurfaceSettings {
    /// XXX id must be unique for every surface, window, and popup
    pub id: window::Id,
    /// layer
    pub layer: Layer,
    /// interactivity
    pub keyboard_interactivity: KeyboardInteractivity,
    /// anchor
    pub anchor: Anchor,
    /// output
    pub output: IcedOutput,
    /// namespace
    pub namespace: String,
    /// margin
    pub margin: IcedMargin,
    /// size, None in a given dimension lets the compositor decide, usually this would be done with a layer surface that is anchored to left & right or top & bottom
    pub size: (Option<u32>, Option<u32>),
    /// exclusive zone
    pub exclusive_zone: i32,
}

impl Default for SctkLayerSurfaceSettings {
    fn default() -> Self {
        Self {
            id: window::Id::new(0),
            layer: Layer::Top,
            keyboard_interactivity: Default::default(),
            anchor: Anchor::empty(),
            output: Default::default(),
            namespace: Default::default(),
            margin: Default::default(),
            size: (Some(200), Some(200)),
            exclusive_zone: Default::default(),
        }
    }
}

#[derive(Clone)]
/// LayerSurface Action
pub enum Action<T> {
    /// create a layer surface and receive a message with its Id
    LayerSurface {
        /// surface builder
        builder: SctkLayerSurfaceSettings,
        /// phantom
        _phantom: PhantomData<T>,
    },
    /// Set size of the layer surface.
    Size {
        /// id of the layer surface
        id: window::Id,
        /// The new logical width of the window
        width: Option<u32>,
        /// The new logical height of the window
        height: Option<u32>,
    },
    /// Destroy the layer surface
    Destroy(window::Id),
    /// The edges which the layer surface is anchored to
    Anchor {
        /// id of the layer surface
        id: window::Id,
        /// anchor of the layer surface
        anchor: Anchor,
    },
    /// exclusive zone of the layer surface
    ExclusiveZone {
        /// id of the layer surface
        id: window::Id,
        /// exclusive zone of the layer surface
        exclusive_zone: i32,
    },
    /// margin of the layer surface, ignored for un-anchored edges
    Margin {
        /// id of the layer surface
        id: window::Id,
        /// margins of the layer surface
        margin: IcedMargin,
    },
    /// keyboard interactivity of the layer surface
    KeyboardInteractivity {
        /// id of the layer surface
        id: window::Id,
        /// keyboard interactivity of the layer surface
        keyboard_interactivity: KeyboardInteractivity,
    },
    /// layer of the layer surface
    Layer {
        /// id of the layer surface
        id: window::Id,
        /// layer of the layer surface
        layer: Layer,
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
            Action::LayerSurface { builder, .. } => Action::LayerSurface {
                builder,
                _phantom: PhantomData::default(),
            },
            Action::Size { id, width, height } => {
                Action::Size { id, width, height }
            }
            Action::Destroy(id) => Action::Destroy(id),
            Action::Anchor { id, anchor } => Action::Anchor { id, anchor },
            Action::ExclusiveZone { id, exclusive_zone } => {
                Action::ExclusiveZone { id, exclusive_zone }
            }
            Action::Margin { id, margin } => Action::Margin { id, margin },
            Action::KeyboardInteractivity {
                id,
                keyboard_interactivity,
            } => Action::KeyboardInteractivity {
                id,
                keyboard_interactivity,
            },
            Action::Layer { id, layer } => Action::Layer { id, layer },
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::LayerSurface { builder, .. } => write!(
                f,
                "Action::LayerSurfaceAction::LayerSurface {{ builder: {:?} }}",
                builder
            ),
            Action::Size { id, width, height } => write!(
                f,
                "Action::LayerSurfaceAction::Size {{ id: {:#?}, width: {:?}, height: {:?} }}", id, width, height
            ),
            Action::Destroy(id) => write!(
                f,
                "Action::LayerSurfaceAction::Destroy {{ id: {:#?} }}", id
            ),
            Action::Anchor { id, anchor } => write!(
                f,
                "Action::LayerSurfaceAction::Anchor {{ id: {:#?}, anchor: {:?} }}", id, anchor
            ),
            Action::ExclusiveZone { id, exclusive_zone } => write!(
                f,
                "Action::LayerSurfaceAction::ExclusiveZone {{ id: {:#?}, exclusive_zone: {exclusive_zone} }}", id
            ),
            Action::Margin { id, margin } => write!(
                f,
                "Action::LayerSurfaceAction::Margin {{ id: {:#?}, margin: {:?} }}", id, margin
            ),
            Action::KeyboardInteractivity { id, keyboard_interactivity } => write!(
                f,
                "Action::LayerSurfaceAction::Margin {{ id: {:#?}, keyboard_interactivity: {:?} }}", id, keyboard_interactivity
            ),
            Action::Layer { id, layer } => write!(
                f,
                "Action::LayerSurfaceAction::Margin {{ id: {:#?}, layer: {:?} }}", id, layer
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
