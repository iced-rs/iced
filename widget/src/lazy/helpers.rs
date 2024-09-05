use crate::core::{self, Element, Size};
use crate::lazy::component;

use std::hash::Hash;

#[allow(deprecated)]
pub use crate::lazy::{Component, Lazy, Responsive};

/// Creates a new [`Lazy`] widget with the given data `Dependency` and a
/// closure that can turn this data into a widget tree.
#[cfg(feature = "lazy")]
pub fn lazy<'a, Message, Theme, Renderer, Dependency, View>(
    dependency: Dependency,
    view: impl Fn(&Dependency) -> View + 'a,
) -> Lazy<'a, Message, Theme, Renderer, Dependency, View>
where
    Dependency: Hash + 'a,
    View: Into<Element<'static, Message, Theme, Renderer>>,
{
    Lazy::new(dependency, view)
}

/// Turns an implementor of [`Component`] into an [`Element`] that can be
/// embedded in any application.
#[cfg(feature = "lazy")]
#[deprecated(
    since = "0.13.0",
    note = "components introduce encapsulated state and hamper the use of a single source of truth. \
    Instead, leverage the Elm Architecture directly, or implement a custom widget"
)]
#[allow(deprecated)]
pub fn component<'a, C, Message, Theme, Renderer>(
    component: C,
) -> Element<'a, Message, Theme, Renderer>
where
    C: Component<Message, Theme, Renderer> + 'a,
    C::State: 'static,
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    component::view(component)
}

/// Creates a new [`Responsive`] widget with a closure that produces its
/// contents.
///
/// The `view` closure will be provided with the current [`Size`] of
/// the [`Responsive`] widget and, therefore, can be used to build the
/// contents of the widget in a responsive way.
#[cfg(feature = "lazy")]
pub fn responsive<'a, Message, Theme, Renderer>(
    f: impl Fn(Size) -> Element<'a, Message, Theme, Renderer> + 'a,
) -> Responsive<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    Responsive::new(f)
}
