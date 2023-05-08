use std::fmt::Debug;

use iced_futures::MaybeSend;

/// data device Actions
pub mod data_device;
/// layer surface actions
pub mod layer_surface;
/// popup actions
pub mod popup;
/// window actions
pub mod window;

/// Platform specific actions defined for wayland
pub enum Action<T> {
    /// LayerSurface Actions
    LayerSurface(layer_surface::Action<T>),
    /// Window Actions
    Window(window::Action<T>),
    /// popup
    Popup(popup::Action<T>),
    /// data device
    DataDevice(data_device::Action<T>),
}

impl<T> Action<T> {
    /// Maps the output of an [`Action`] using the given function.
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
        A: 'static,
    {
        match self {
            Action::LayerSurface(a) => Action::LayerSurface(a.map(f)),
            Action::Window(a) => Action::Window(a.map(f)),
            Action::Popup(a) => Action::Popup(a.map(f)),
            Action::DataDevice(a) => Action::DataDevice(a.map(f)),
        }
    }
}

impl<T> Debug for Action<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LayerSurface(arg0) => {
                f.debug_tuple("LayerSurface").field(arg0).finish()
            }
            Self::Window(arg0) => f.debug_tuple("Window").field(arg0).finish(),
            Self::Popup(arg0) => f.debug_tuple("Popup").field(arg0).finish(),
            Self::DataDevice(arg0) => {
                f.debug_tuple("DataDevice").field(arg0).finish()
            }
        }
    }
}
