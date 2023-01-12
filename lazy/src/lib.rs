#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    missing_debug_implementations,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![forbid(unsafe_code)]
#![allow(
    clippy::await_holding_refcell_ref,
    clippy::inherent_to_string,
    clippy::type_complexity
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
mod lazy;

pub mod component;
pub mod responsive;

pub use component::Component;
pub use lazy::Lazy;
pub use responsive::Responsive;

mod cache;

use iced_native::{Element, Size};
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
    Renderer: iced_native::Renderer + 'a,
{
    component::view(component)
}

pub fn responsive<'a, Message, Renderer>(
    f: impl Fn(Size) -> Element<'a, Message, Renderer> + 'a,
) -> Responsive<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    Responsive::new(f)
}
