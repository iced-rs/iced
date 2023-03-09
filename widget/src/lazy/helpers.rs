use crate::core::{self, Element, Size};
use crate::lazy::component::{self, Component};
use crate::lazy::{Lazy, Responsive};

use std::hash::Hash;

pub fn lazy<'a, Message, Renderer, Dependency, View>(
    dependency: Dependency,
    view: impl Fn(&Dependency) -> View + 'a,
) -> Lazy<'a, Message, Renderer, Dependency, View>
where
    Dependency: Hash + 'a,
    View: Into<Element<'static, Message, Renderer>>,
{
    Lazy::new(dependency, view)
}

/// Turns an implementor of [`Component`] into an [`Element`] that can be
/// embedded in any application.
pub fn component<'a, C, Message, Renderer>(
    component: C,
) -> Element<'a, Message, Renderer>
where
    C: Component<Message, Renderer> + 'a,
    C::State: 'static,
    Message: 'a,
    Renderer: core::Renderer + 'a,
{
    component::view(component)
}

pub fn responsive<'a, Message, Renderer>(
    f: impl Fn(Size) -> Element<'a, Message, Renderer> + 'a,
) -> Responsive<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    Responsive::new(f)
}
