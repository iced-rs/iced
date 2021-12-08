use iced_native::event;
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::{
    Clipboard, Element, Hasher, Length, Point, Rectangle, Shell, Widget,
};

use ouroboros::self_referencing;
use std::marker::PhantomData;

pub fn view<'a, C, Message, Renderer>(
    component: C,
) -> Element<'a, Message, Renderer>
where
    C: Component<Message, Renderer> + 'a,
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
{
    Element::new(Instance {
        state: Some(
            StateBuilder {
                component: Box::new(component),
                cache_builder: |state| Cache {
                    element: state.view(),
                    message: PhantomData,
                },
            }
            .build(),
        ),
    })
}

pub trait Component<Message, Renderer> {
    type Event;

    fn update(&mut self, event: Self::Event) -> Option<Message>;

    fn view(&mut self) -> Element<Self::Event, Renderer>;
}

struct Instance<'a, Message, Renderer, Event> {
    state: Option<State<'a, Message, Renderer, Event>>,
}

#[self_referencing]
struct State<'a, Message: 'a, Renderer: 'a, Event: 'a> {
    component: Box<dyn Component<Message, Renderer, Event = Event> + 'a>,

    #[borrows(mut component)]
    #[covariant]
    cache: Cache<'this, Message, Renderer, Event>,
}

struct Cache<'a, Message, Renderer, Event> {
    element: Element<'a, Event, Renderer>,
    message: PhantomData<Message>,
}

impl<'a, Message, Renderer, Event> Widget<Message, Renderer>
    for Instance<'a, Message, Renderer, Event>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.state.as_ref().unwrap().borrow_cache().element.width()
    }

    fn height(&self) -> Length {
        self.state.as_ref().unwrap().borrow_cache().element.height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.state
            .as_ref()
            .unwrap()
            .borrow_cache()
            .element
            .layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let event_status =
            self.state.as_mut().unwrap().with_cache_mut(|cache| {
                cache.element.on_event(
                    event,
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    &mut local_shell,
                )
            });

        if !local_messages.is_empty() {
            let mut component =
                self.state.take().unwrap().into_heads().component;

            for message in local_messages
                .into_iter()
                .filter_map(|message| component.update(message))
            {
                shell.publish(message);
            }

            self.state = Some(
                StateBuilder {
                    component,
                    cache_builder: |state| Cache {
                        element: state.view(),
                        message: PhantomData,
                    },
                }
                .build(),
            );

            shell.invalidate_layout();
        }

        event_status
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.state.as_ref().unwrap().borrow_cache().element.draw(
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.state
            .as_ref()
            .unwrap()
            .borrow_cache()
            .element
            .hash_layout(state)
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> mouse::Interaction {
        self.state
            .as_ref()
            .unwrap()
            .borrow_cache()
            .element
            .mouse_interaction(layout, cursor_position, viewport)
    }

    fn overlay(
        &mut self,
        _layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        // TODO: Rethink overlay composability
        None
    }
}
