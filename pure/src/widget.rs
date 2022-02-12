mod button;
mod column;
mod container;
mod element;
mod row;
mod text;
mod tree;

pub use button::Button;
pub use column::Column;
pub use container::Container;
pub use element::Element;
pub use row::Row;
pub use text::Text;
pub use tree::Tree;

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::renderer;
use iced_native::{Clipboard, Hasher, Length, Point, Rectangle, Shell};

use std::any::{self, Any};

pub trait Widget<Message, Renderer> {
    fn tag(&self) -> any::TypeId;

    fn state(&self) -> Box<dyn Any>;

    fn children(&self) -> &[Element<Message, Renderer>];

    fn width(&self) -> Length;

    fn height(&self) -> Length;

    fn hash_layout(&self, state: &mut Hasher);

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node;

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    );

    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Idle
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        event::Status::Ignored
    }
}

pub fn container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Container<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    Container::new(content)
}

pub fn column<'a, Message, Renderer>() -> Column<'a, Message, Renderer> {
    Column::new()
}

pub fn row<'a, Message, Renderer>() -> Row<'a, Message, Renderer> {
    Row::new()
}

pub fn button<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Button<'a, Message, Renderer> {
    Button::new(content)
}

pub fn text<Renderer>(text: impl Into<String>) -> Text<Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    Text::new(text)
}
