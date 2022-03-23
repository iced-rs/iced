mod component;
mod responsive;

pub use component::Component;
pub use responsive::Responsive;

use iced_native::Size;
use iced_pure::Element;

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
