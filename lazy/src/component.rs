//! Build and reuse custom widgets using The Elm Architecture.
use iced_native::event;
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget;
use iced_native::widget::tree::{self, Tree};
use iced_native::{
    Clipboard, Element, Length, Point, Rectangle, Shell, Size, Widget,
};

use ouroboros::self_referencing;
use std::cell::{Ref, RefCell};
use std::marker::PhantomData;

/// A reusable, custom widget that uses The Elm Architecture.
///
/// A [`Component`] allows you to implement custom widgets as if they were
/// `iced` applications with encapsulated state.
///
/// In other words, a [`Component`] allows you to turn `iced` applications into
/// custom widgets and embed them without cumbersome wiring.
///
/// A [`Component`] produces widgets that may fire an [`Event`](Component::Event)
/// and update the internal state of the [`Component`].
///
/// Additionally, a [`Component`] is capable of producing a `Message` to notify
/// the parent application of any relevant interactions.
pub trait Component<Message, Renderer> {
    /// The internal state of this [`Component`].
    type State: Default;

    /// The type of event this [`Component`] handles internally.
    type Event;

    /// Processes an [`Event`](Component::Event) and updates the [`Component`] state accordingly.
    ///
    /// It can produce a `Message` for the parent application.
    fn update(
        &mut self,
        state: &mut Self::State,
        event: Self::Event,
    ) -> Option<Message>;

    /// Produces the widgets of the [`Component`], which may trigger an [`Event`](Component::Event)
    /// on user interaction.
    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Renderer>;
}

/// Turns an implementor of [`Component`] into an [`Element`] that can be
/// embedded in any application.
pub fn view<'a, C, Message, Renderer>(
    component: C,
) -> Element<'a, Message, Renderer>
where
    C: Component<Message, Renderer> + 'a,
    C::State: 'static,
    Message: 'a,
    Renderer: iced_native::Renderer + 'a,
{
    Element::new(Instance {
        state: RefCell::new(Some(
            StateBuilder {
                component: Box::new(component),
                message: PhantomData,
                state: PhantomData,
                element_builder: |_| None,
            }
            .build(),
        )),
    })
}

struct Instance<'a, Message, Renderer, Event, S> {
    state: RefCell<Option<State<'a, Message, Renderer, Event, S>>>,
}

#[self_referencing]
struct State<'a, Message: 'a, Renderer: 'a, Event: 'a, S: 'a> {
    component:
        Box<dyn Component<Message, Renderer, Event = Event, State = S> + 'a>,
    message: PhantomData<Message>,
    state: PhantomData<S>,

    #[borrows(component)]
    #[covariant]
    element: Option<Element<'this, Event, Renderer>>,
}

impl<'a, Message, Renderer, Event, S> Instance<'a, Message, Renderer, Event, S>
where
    S: Default,
{
    fn rebuild_element(&self, state: &S) {
        let heads = self.state.borrow_mut().take().unwrap().into_heads();

        *self.state.borrow_mut() = Some(
            StateBuilder {
                component: heads.component,
                message: PhantomData,
                state: PhantomData,
                element_builder: |component| Some(component.view(state)),
            }
            .build(),
        );
    }

    fn with_element<T>(
        &self,
        f: impl FnOnce(&Element<'_, Event, Renderer>) -> T,
    ) -> T {
        self.with_element_mut(|element| f(element))
    }

    fn with_element_mut<T>(
        &self,
        f: impl FnOnce(&mut Element<'_, Event, Renderer>) -> T,
    ) -> T {
        self.state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .with_element_mut(|element| f(element.as_mut().unwrap()))
    }
}

impl<'a, Message, Renderer, Event, S> Widget<Message, Renderer>
    for Instance<'a, Message, Renderer, Event, S>
where
    S: 'static + Default,
    Renderer: iced_native::Renderer,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<S>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(S::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.rebuild_element(&S::default());
        self.with_element(|element| vec![Tree::new(element)])
    }

    fn diff(&self, tree: &mut Tree) {
        self.rebuild_element(tree.state.downcast_ref());
        self.with_element(|element| {
            tree.diff_children(std::slice::from_ref(&element))
        })
    }

    fn width(&self) -> Length {
        self.with_element(|element| element.as_widget().width())
    }

    fn height(&self) -> Length {
        self.with_element(|element| element.as_widget().height())
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.with_element(|element| {
            element.as_widget().layout(renderer, limits)
        })
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced_native::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let event_status = self.with_element_mut(|element| {
            element.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                &mut local_shell,
            )
        });

        local_shell.revalidate_layout(|| shell.invalidate_layout());

        if !local_messages.is_empty() {
            let mut heads = self.state.take().unwrap().into_heads();

            for message in local_messages.into_iter().filter_map(|message| {
                heads
                    .component
                    .update(tree.state.downcast_mut::<S>(), message)
            }) {
                shell.publish(message);
            }

            self.state = RefCell::new(Some(
                StateBuilder {
                    component: heads.component,
                    message: PhantomData,
                    state: PhantomData,
                    element_builder: |state| {
                        Some(state.view(tree.state.downcast_ref::<S>()))
                    },
                }
                .build(),
            ));

            self.with_element(|element| {
                tree.diff_children(std::slice::from_ref(&element))
            });

            shell.invalidate_layout();
        }

        event_status
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        struct MapOperation<'a, B> {
            operation: &'a mut dyn widget::Operation<B>,
        }

        impl<'a, T, B> widget::Operation<T> for MapOperation<'a, B> {
            fn container(
                &mut self,
                id: Option<&widget::Id>,
                operate_on_children: &mut dyn FnMut(
                    &mut dyn widget::Operation<T>,
                ),
            ) {
                self.operation.container(id, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }

            fn focusable(
                &mut self,
                state: &mut dyn widget::operation::Focusable,
                id: Option<&widget::Id>,
            ) {
                self.operation.focusable(state, id);
            }

            fn text_input(
                &mut self,
                state: &mut dyn widget::operation::TextInput,
                id: Option<&widget::Id>,
            ) {
                self.operation.text_input(state, id);
            }
        }

        self.with_element(|element| {
            element.as_widget().operate(
                &mut tree.children[0],
                layout,
                &mut MapOperation { operation },
            );
        });
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.with_element(|element| {
            element.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            );
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_element(|element| {
            element.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        })
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let overlay = OverlayBuilder {
            instance: self,
            instance_ref_builder: |instance| instance.state.borrow(),
            tree,
            types: PhantomData,
            overlay_builder: |instance, tree| {
                instance
                    .as_ref()
                    .unwrap()
                    .borrow_element()
                    .as_ref()
                    .unwrap()
                    .as_widget()
                    .overlay(&mut tree.children[0], layout, renderer)
            },
        }
        .build();

        let has_overlay = overlay.with_overlay(|overlay| {
            overlay.as_ref().map(overlay::Element::position)
        });

        has_overlay.map(|position| {
            overlay::Element::new(
                position,
                Box::new(OverlayInstance {
                    overlay: Some(overlay),
                }),
            )
        })
    }
}

#[self_referencing]
struct Overlay<'a, 'b, Message, Renderer, Event, S> {
    instance: &'a Instance<'b, Message, Renderer, Event, S>,
    tree: &'a mut Tree,
    types: PhantomData<(Message, Event, S)>,

    #[borrows(instance)]
    #[covariant]
    instance_ref: Ref<'this, Option<State<'a, Message, Renderer, Event, S>>>,

    #[borrows(instance_ref, mut tree)]
    #[covariant]
    overlay: Option<overlay::Element<'this, Event, Renderer>>,
}

struct OverlayInstance<'a, 'b, Message, Renderer, Event, S> {
    overlay: Option<Overlay<'a, 'b, Message, Renderer, Event, S>>,
}

impl<'a, 'b, Message, Renderer, Event, S>
    OverlayInstance<'a, 'b, Message, Renderer, Event, S>
{
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&overlay::Element<'_, Event, Renderer>) -> T,
    ) -> Option<T> {
        self.overlay
            .as_ref()
            .unwrap()
            .borrow_overlay()
            .as_ref()
            .map(f)
    }

    fn with_overlay_mut_maybe<T>(
        &mut self,
        f: impl FnOnce(&mut overlay::Element<'_, Event, Renderer>) -> T,
    ) -> Option<T> {
        self.overlay
            .as_mut()
            .unwrap()
            .with_overlay_mut(|overlay| overlay.as_mut().map(f))
    }
}

impl<'a, 'b, Message, Renderer, Event, S> overlay::Overlay<Message, Renderer>
    for OverlayInstance<'a, 'b, Message, Renderer, Event, S>
where
    Renderer: iced_native::Renderer,
    S: 'static + Default,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        self.with_overlay_maybe(|overlay| {
            let vector = position - overlay.position();

            overlay.layout(renderer, bounds).translate(vector)
        })
        .unwrap_or_default()
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        let _ = self.with_overlay_maybe(|overlay| {
            overlay.draw(renderer, theme, style, layout, cursor_position);
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.with_overlay_maybe(|overlay| {
            overlay.mouse_interaction(
                layout,
                cursor_position,
                viewport,
                renderer,
            )
        })
        .unwrap_or_default()
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
            .with_overlay_mut_maybe(|overlay| {
                overlay.on_event(
                    event,
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    &mut local_shell,
                )
            })
            .unwrap_or(iced_native::event::Status::Ignored);

        local_shell.revalidate_layout(|| shell.invalidate_layout());

        if !local_messages.is_empty() {
            let overlay = self.overlay.take().unwrap().into_heads();
            let mut heads = overlay.instance.state.take().unwrap().into_heads();

            for message in local_messages.into_iter().filter_map(|message| {
                heads
                    .component
                    .update(overlay.tree.state.downcast_mut::<S>(), message)
            }) {
                shell.publish(message);
            }

            *overlay.instance.state.borrow_mut() = Some(
                StateBuilder {
                    component: heads.component,
                    message: PhantomData,
                    state: PhantomData,
                    element_builder: |state| {
                        Some(state.view(overlay.tree.state.downcast_ref::<S>()))
                    },
                }
                .build(),
            );

            overlay.instance.with_element(|element| {
                overlay.tree.diff_children(std::slice::from_ref(&element))
            });

            self.overlay = Some(
                OverlayBuilder {
                    instance: overlay.instance,
                    instance_ref_builder: |instance| instance.state.borrow(),
                    tree: overlay.tree,
                    types: PhantomData,
                    overlay_builder: |_, _| None,
                }
                .build(),
            );

            shell.invalidate_layout();
        }

        event_status
    }
}
