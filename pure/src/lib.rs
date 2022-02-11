pub mod widget;

pub(crate) mod flex;

pub use widget::*;

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::renderer;
use iced_native::{Clipboard, Hasher, Length, Point, Rectangle, Shell};

pub struct Pure<'a, Message, Renderer> {
    state: &'a mut State,
    element: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Pure<'a, Message, Renderer>
where
    Message: 'static,
    Renderer: iced_native::Renderer + 'static,
{
    pub fn new(
        state: &'a mut State,
        content: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        let element = content.into();
        let _ = state.diff(&element);

        Self { state, element }
    }
}

pub struct State {
    state_tree: widget::Tree,
}

impl State {
    pub fn new() -> Self {
        Self {
            state_tree: widget::Tree::empty(),
        }
    }

    fn diff<Message, Renderer>(
        &mut self,
        new_element: &Element<Message, Renderer>,
    ) {
        self.state_tree.diff(new_element);
    }
}

impl<'a, Message, Renderer> iced_native::Widget<Message, Renderer>
    for Pure<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.element.as_widget().width()
    }

    fn height(&self) -> Length {
        self.element.as_widget().height()
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.element.as_widget().hash_layout(state)
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.element.as_widget().layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.element.as_widget_mut().on_event(
            &mut self.state.state_tree,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.element.as_widget().draw(
            &self.state.state_tree,
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element.as_widget().mouse_interaction(
            &self.state.state_tree,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message, Renderer> Into<iced_native::Element<'a, Message, Renderer>>
    for Pure<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
{
    fn into(self) -> iced_native::Element<'a, Message, Renderer> {
        iced_native::Element::new(self)
    }
}
