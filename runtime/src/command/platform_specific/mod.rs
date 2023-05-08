use std::{fmt, marker::PhantomData};

use iced_futures::MaybeSend;

#[cfg(feature = "wayland")]
pub mod wayland;

/// Platform specific actions defined for wayland
pub enum Action<T> {
    /// phantom data variant in case the platform has not specific actions implemented
    Phantom(PhantomData<T>),
    /// Wayland Specific Actions
    #[cfg(feature = "wayland")]
    Wayland(wayland::Action<T>),
}

impl<T> Action<T> {
    /// Maps the output of an [`Action`] using the given function.
    pub fn map<A>(
        self,
        _f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
        A: 'static,
    {
        match self {
            Action::Phantom(_) => unimplemented!(),
            #[cfg(feature = "wayland")]
            Action::Wayland(action) => Action::Wayland(action.map(_f)),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Phantom(_) => unimplemented!(),
            #[cfg(feature = "wayland")]
            Action::Wayland(action) => action.fmt(_f),
        }
    }
}
