use iced_native::event;
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::{
    Clipboard, Element, Hasher, Length, Point, Rectangle, Shell, Size, Widget,
};

use ouroboros::self_referencing;
use std::cell::RefCell;
use std::hash::Hash;
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
        state: RefCell::new(Some(
            StateBuilder {
                component: Box::new(component),
                cache_builder: |state| {
                    Some(
                        CacheBuilder {
                            element: state.view(),
                            message: PhantomData,
                            overlay_builder: |_| None,
                        }
                        .build(),
                    )
                },
            }
            .build(),
        )),
    })
}

pub trait Component<Message, Renderer> {
    type Event;

    fn update(&mut self, event: Self::Event) -> Option<Message>;

    fn view(&mut self) -> Element<Self::Event, Renderer>;
}

struct Instance<'a, Message, Renderer, Event> {
    state: RefCell<Option<State<'a, Message, Renderer, Event>>>,
}

#[self_referencing]
struct State<'a, Message: 'a, Renderer: 'a, Event: 'a> {
    component: Box<dyn Component<Message, Renderer, Event = Event> + 'a>,

    #[borrows(mut component)]
    #[covariant]
    cache: Option<Cache<'this, Message, Renderer, Event>>,
}

#[self_referencing]
struct Cache<'a, Message, Renderer: 'a, Event: 'a> {
    element: Element<'a, Event, Renderer>,
    message: PhantomData<Message>,

    #[borrows(mut element)]
    #[covariant]
    overlay: Option<overlay::Element<'this, Event, Renderer>>,
}

impl<'a, Message, Renderer, Event> Widget<Message, Renderer>
    for Instance<'a, Message, Renderer, Event>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let element = cache.take().unwrap().into_heads().element;
                let width = element.width();

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );

                width
            })
    }

    fn height(&self) -> Length {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let element = cache.take().unwrap().into_heads().element;
                let height = element.height();

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );

                height
            })
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let element = cache.take().unwrap().into_heads().element;
                let layout = element.layout(renderer, limits);

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );

                layout
            })
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

        let event_status = self
            .state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let mut element = cache.take().unwrap().into_heads().element;
                let event_status = element.on_event(
                    event,
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    &mut local_shell,
                );

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );

                event_status
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

            *self.state.borrow_mut() = Some(
                StateBuilder {
                    component,
                    cache_builder: |state| {
                        Some(
                            CacheBuilder {
                                element: state.view(),
                                message: PhantomData,
                                overlay_builder: |_| None,
                            }
                            .build(),
                        )
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
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let element = cache.take().unwrap().into_heads().element;
                element.draw(
                    renderer,
                    style,
                    layout,
                    cursor_position,
                    viewport,
                );

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );
            })
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let element = cache.take().unwrap().into_heads().element;
                element.hash_layout(state);

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );
            })
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> mouse::Interaction {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let element = cache.take().unwrap().into_heads().element;
                let mouse_interaction = element.mouse_interaction(
                    layout,
                    cursor_position,
                    viewport,
                );

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );

                mouse_interaction
            })
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let has_overlay = self
            .state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                let element = cache.take().unwrap().into_heads().element;

                *cache = Some(
                    CacheBuilder {
                        element,
                        message: PhantomData,
                        overlay_builder: |element| element.overlay(layout),
                    }
                    .build(),
                );

                cache.as_ref().unwrap().borrow_overlay().is_some()
            });

        let Self { state, .. } = self;

        has_overlay.then(|| {
            overlay::Element::new(
                layout.position(),
                Box::new(Overlay { state }),
            )
        })
    }
}

struct Overlay<'a, 'b, Message, Event, Renderer> {
    state: &'b RefCell<Option<State<'a, Message, Renderer, Event>>>,
}

impl<'a, 'b, Message, Event, Renderer> overlay::Overlay<Message, Renderer>
    for Overlay<'a, 'b, Message, Event, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                cache.as_mut().unwrap().with_overlay_mut(|overlay| {
                    *overlay = overlay.take().map(|x| {
                        let vector = position - x.position();
                        x.translate(vector)
                    });
                    overlay
                        .as_mut()
                        .map(|overlay| overlay.layout(renderer, bounds))
                        .unwrap_or_else(|| layout::Node::new(Size::ZERO))
                })
            })
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        self.state.borrow().as_ref().unwrap().with_cache(|cache| {
            if let Some(overlay) =
                cache.as_ref().unwrap().borrow_overlay().as_ref()
            {
                overlay.draw(renderer, style, layout, cursor_position);
            }
        })
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> mouse::Interaction {
        self.state.borrow().as_ref().unwrap().with_cache(|cache| {
            cache
                .as_ref()
                .unwrap()
                .borrow_overlay()
                .as_ref()
                .map(|overlay| {
                    overlay.mouse_interaction(layout, cursor_position, viewport)
                })
                .unwrap_or(mouse::Interaction::default())
        })
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        (position.x as u32).hash(state);
        (position.y as u32).hash(state);

        self.state.borrow().as_ref().unwrap().with_cache(|cache| {
            if let Some(overlay) =
                cache.as_ref().unwrap().borrow_overlay().as_ref()
            {
                overlay.hash_layout(state);
            }
        })
    }

    fn on_event(
        &mut self,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> iced_native::event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let event_status = self
            .state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_cache_mut(|cache| {
                cache.as_mut().unwrap().with_overlay_mut(|overlay| {
                    overlay
                        .as_mut()
                        .map(|overlay| {
                            overlay.on_event(
                                event,
                                layout,
                                cursor_position,
                                renderer,
                                clipboard,
                                &mut local_shell,
                            )
                        })
                        .unwrap_or(iced_native::event::Status::Ignored)
                })
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

            *self.state.borrow_mut() = Some(
                StateBuilder {
                    component,
                    cache_builder: |state| {
                        Some(
                            CacheBuilder {
                                element: state.view(),
                                message: PhantomData,
                                overlay_builder: |element| {
                                    element.overlay(layout)
                                },
                            }
                            .build(),
                        )
                    },
                }
                .build(),
            );

            shell.invalidate_layout();
        }

        event_status
    }
}
