use std::{fmt, marker::PhantomData};

use iced_futures::MaybeSend;

/// wayland platform specific actions
#[cfg(feature = "wayland")]
pub mod wayland;

/// Platform specific actions defined for wayland
pub enum Action<T> {
    /// LayerSurface Actions
    #[cfg(feature = "wayland")]
    Wayland(wayland::Action<T>),
    /// phantom data variant in case the platform has not specific actions implemented
    Phantom(PhantomData<T>),
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
            #[cfg(feature = "wayland")]
            Action::Wayland(a) => Action::Wayland(a.map(f)),
            Action::Phantom(_) => unimplemented!(),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "wayland")]
            Self::Wayland(arg0) => {
                f.debug_tuple("LayerSurface").field(arg0).finish()
            }
            Action::Phantom(_) => unimplemented!(),
        }
    }
}
