use crate::widget::tree::{self, Tree};
use crate::widget::Widget;

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::renderer;
use iced_native::{Clipboard, Length, Point, Rectangle, Shell};

pub struct Element<'a, Message, Renderer> {
    widget: Box<dyn Widget<Message, Renderer> + 'a>,
}

impl<'a, Message, Renderer> Element<'a, Message, Renderer> {
    pub fn new(widget: impl Widget<Message, Renderer> + 'a) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub fn as_widget(&self) -> &dyn Widget<Message, Renderer> {
        self.widget.as_ref()
    }

    pub fn as_widget_mut(&mut self) -> &mut dyn Widget<Message, Renderer> {
        self.widget.as_mut()
    }

    pub fn map<B>(
        self,
        f: impl Fn(Message) -> B + 'a,
    ) -> Element<'a, B, Renderer>
    where
        Message: 'a,
        Renderer: iced_native::Renderer + 'a,
        B: 'a,
    {
        Element::new(Map::new(self.widget, f))
    }
}

struct Map<'a, A, B, Renderer> {
    widget: Box<dyn Widget<A, Renderer> + 'a>,
    mapper: Box<dyn Fn(A) -> B + 'a>,
}

impl<'a, A, B, Renderer> Map<'a, A, B, Renderer> {
    pub fn new<F>(
        widget: Box<dyn Widget<A, Renderer> + 'a>,
        mapper: F,
    ) -> Map<'a, A, B, Renderer>
    where
        F: 'a + Fn(A) -> B,
    {
        Map {
            widget,
            mapper: Box::new(mapper),
        }
    }
}

impl<'a, A, B, Renderer> Widget<B, Renderer> for Map<'a, A, B, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
    A: 'a,
    B: 'a,
{
    fn tag(&self) -> tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.widget.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.widget.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.widget.diff(tree)
    }

    fn width(&self) -> Length {
        self.widget.width()
    }

    fn height(&self) -> Length {
        self.widget.height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.widget.layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, B>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let status = self.widget.on_event(
            tree,
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            &mut local_shell,
        );

        shell.merge(local_shell, &self.mapper);

        status
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.widget.draw(
            tree,
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.widget.mouse_interaction(
            tree,
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }
}
