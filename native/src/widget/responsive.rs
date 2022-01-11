use crate::event::{self, Event};
use crate::layout::{self, Layout};
use crate::renderer;
use crate::{
    Clipboard, Element, Hasher, Length, Point, Rectangle, Shell, Size, Widget,
};

use std::cell::RefCell;
use std::hash::Hasher as _;

#[derive(Debug, Clone, Default)]
pub struct State {
    last_size: Option<Size>,
    last_layout: layout::Node,
    last_layout_hash: u64,
}

impl State {
    pub fn new() -> State {
        State::default()
    }
}

#[allow(missing_debug_implementations)]
pub struct Responsive<'a, Message, Renderer>(
    RefCell<Internal<'a, Message, Renderer>>,
);

impl<'a, Message, Renderer> Responsive<'a, Message, Renderer> {
    pub fn new(
        state: &'a mut State,
        view: impl FnOnce(Size) -> Element<'a, Message, Renderer> + 'a,
    ) -> Self {
        Self(RefCell::new(Internal {
            state,
            content: Content::Pending(Some(Box::new(view))),
        }))
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Responsive<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn hash_layout(&self, _hasher: &mut Hasher) {}

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.max();

        self.0.borrow_mut().state.last_size = Some(size);

        layout::Node::new(size)
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
        use std::ops::DerefMut;

        let mut internal = self.0.borrow_mut();

        let Internal { content, state } = internal.deref_mut();

        let content = content.resolve(state, renderer);

        let content_layout = Layout::with_offset(
            layout.position() - Point::ORIGIN,
            &state.last_layout,
        );

        content.on_event(
            event,
            content_layout,
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
        use std::ops::DerefMut;

        let mut internal = self.0.borrow_mut();

        let Internal { content, state } = internal.deref_mut();

        let content = content.resolve(state, renderer);

        let content_layout = Layout::with_offset(
            layout.position() - Point::ORIGIN,
            &state.last_layout,
        );

        content.draw(renderer, style, content_layout, cursor_position, viewport)
    }
}

struct Internal<'a, Message, Renderer> {
    state: &'a mut State,
    content: Content<'a, Message, Renderer>,
}

enum Content<'a, Message, Renderer> {
    Pending(
        Option<Box<dyn FnOnce(Size) -> Element<'a, Message, Renderer> + 'a>>,
    ),
    Ready(Element<'a, Message, Renderer>),
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn resolve(
        &mut self,
        state: &mut State,
        renderer: &Renderer,
    ) -> &mut Element<'a, Message, Renderer> {
        match self {
            Content::Ready(element) => element,
            Content::Pending(view) => {
                let element =
                    view.take().unwrap()(state.last_size.unwrap_or(Size::ZERO));

                let new_layout_hash = {
                    let mut hasher = Hasher::default();
                    element.hash_layout(&mut hasher);

                    hasher.finish()
                };

                if new_layout_hash != state.last_layout_hash {
                    state.last_layout = element.layout(
                        renderer,
                        &layout::Limits::new(
                            Size::ZERO,
                            state.last_size.unwrap_or(Size::ZERO),
                        ),
                    );

                    state.last_layout_hash = new_layout_hash;
                }

                *self = Content::Ready(element);

                self.resolve(state, renderer)
            }
        }
    }
}

impl<'a, Message, Renderer> From<Responsive<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: crate::Renderer + 'a,
    Message: 'a,
{
    fn from(responsive: Responsive<'a, Message, Renderer>) -> Self {
        Self::new(responsive)
    }
}
