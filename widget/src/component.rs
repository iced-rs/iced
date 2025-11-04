//! Build and reuse custom widgets using The Elm Architecture.
use crate::Action;
use crate::core::event;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Clipboard, Element, Event, Length, Point, Rectangle, Shell, Size,
    Vector, Widget,
};

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
///
/// # State
/// A component can store its state in one of two ways: either as data within the
/// implementor of the trait, or in a type [`State`][Component::State] that is managed
/// by the runtime and provided to the trait methods. These two approaches are not
/// mutually exclusive and have opposite pros and cons.
///
/// For instance, if a piece of state is needed by multiple components that reside
/// in different branches of the tree, then it's more convenient to let a common
/// ancestor store it and pass it down.
///
/// On the other hand, if a piece of state is only needed by the component itself,
/// you can store it as part of its internal [`State`][Component::State].
pub trait Component<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
>
{
    /// The internal state of this [`Component`].
    type State: Default + 'static;

    /// The type of event this [`Component`] handles internally.
    type Event: 'static;

    /// Processes an [`Event`](Component::Event) and updates the [`Component`] state accordingly.
    ///
    /// It can produce a `Message` for the parent application.
    fn update(
        &mut self,
        state: &mut Self::State,
        event: Self::Event,
        renderer: &Renderer,
    ) -> Option<Message>;

    /// Produces the widgets of the [`Component`], which may trigger an [`Event`](Component::Event)
    /// on user interaction.
    fn view(
        &self,
        state: &Self::State,
    ) -> Element<'a, Self::Event, Theme, Renderer>;

    /// Listens to a runtime [`Event`] and performs an [`Action`] as a result.
    ///
    /// If the [`Action`] publishes a [`Component::Event`], it will be immediately fed
    /// to [`update`](Self::update).
    ///
    /// By default, it returns [`Action::none`].
    fn listen(
        &self,
        _state: &Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Action<Self::Event> {
        Action::none()
    }

    /// Returns the current [`mouse::Interaction`] of the [`Component`].
    ///
    /// This interaction will override any interaction produced by the [`view`](Self::view)
    /// of the [`Component`].
    ///
    /// By default, it returns [`mouse::Interaction::None`].
    fn mouse_interaction(&self, _state: &Self::State) -> mouse::Interaction {
        mouse::Interaction::None
    }

    /// Reconciles the current [`Component`] with its internal [`State`](Self::State) persisted
    /// in the widget tree.
    ///
    /// This method will be called every time the widget tree changes. You can leverage it to
    /// detect and react to changes in the [`Component`].
    ///
    /// By default, it does nothing.
    fn diff(&self, _state: &mut Self::State) {}

    /// Run the provided [`widget::Operation`] on the [`Component`].
    ///
    /// By default, it does nothing.
    fn operate(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _operation: &mut dyn widget::Operation,
    ) {
    }

    /// Returns a [`Size`] hint for laying out the [`Component`].
    ///
    /// This hint may be used by some widget containers to adjust their sizing strategy
    /// during construction.
    ///
    /// By default, it returns a [`Size`] with both dimensions set to [`Length::Shrink`].
    fn size_hint(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }
}

/// Turns an implementor of [`Component`] into an [`Element`] that can be
/// embedded in any application.
pub fn component<'a, C, Message, Theme, Renderer>(
    component: C,
) -> Element<'a, Message, Theme, Renderer>
where
    C: Component<'a, Message, Theme, Renderer> + 'a,
    C::State: 'static,
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    let size_hint = component.size_hint();

    Element::new(Instance {
        component,
        size_hint,
        view: crate::space().into(),
        limits: layout::Limits::new(Size::ZERO, Size::INFINITE),
        layout: layout::Node::new(Size::ZERO),
        is_outdated: true,
        has_overlay: false,
    })
}

struct Instance<'a, C, Message, Theme, Renderer>
where
    C: Component<'a, Message, Theme, Renderer> + 'a,
{
    component: C,
    view: Element<'a, C::Event, Theme, Renderer>,
    limits: layout::Limits,
    layout: layout::Node,
    size_hint: Size<Length>,
    is_outdated: bool,
    has_overlay: bool,
}

struct Internal<State, Event> {
    state: State,
    events: Vec<Event>,
}

impl<'a, C, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Instance<'a, C, Message, Theme, Renderer>
where
    C: Component<'a, Message, Theme, Renderer> + 'a,
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Internal<C::State, C::Event>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Internal {
            state: C::State::default(),
            events: Vec::<C::Event>::new(),
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let internal =
            tree.state.downcast_mut::<Internal<C::State, C::Event>>();

        self.component.diff(&mut internal.state);
    }

    fn size(&self) -> Size<Length> {
        self.size_hint
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let internal =
            tree.state.downcast_ref::<Internal<C::State, C::Event>>();

        if self.is_outdated {
            self.view = self.component.view(&internal.state);
            tree.diff_children(&[&self.view]);

            self.is_outdated = false;
        }

        if &self.limits != limits {
            self.limits = *limits;
            self.layout = self.view.as_widget_mut().layout(
                &mut tree.children[0],
                renderer,
                limits,
            );
        }

        layout::Node::new(self.layout.size())
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let internal =
            tree.state.downcast_mut::<Internal<C::State, C::Event>>();

        let action = self.component.listen(
            &internal.state,
            event,
            layout.bounds(),
            cursor,
        );

        let (publish, redraw_request, event_status) = action.into_inner();

        shell.request_redraw_at(redraw_request);

        if let event::Status::Captured = event_status {
            shell.capture_event();
        }

        internal.events.extend(publish);

        if !shell.is_event_captured() {
            let mut local_shell = Shell::new(&mut internal.events);

            self.view.as_widget_mut().update(
                &mut tree.children[0],
                event,
                Layout::with_offset(
                    layout.position() - Point::ORIGIN,
                    &self.layout,
                ),
                cursor,
                renderer,
                clipboard,
                &mut local_shell,
                viewport,
            );

            if local_shell.is_event_captured() {
                shell.capture_event();
            }

            if local_shell.is_layout_invalid() {
                shell.invalidate_layout();
            }

            if local_shell.are_widgets_invalid() {
                shell.invalidate_widgets();
            }

            shell.request_redraw_at(local_shell.redraw_request());
            shell.request_input_method(local_shell.input_method());
        }

        if internal.events.is_empty() {
            return;
        }

        for event in internal.events.drain(..) {
            if let Some(message) =
                self.component.update(&mut internal.state, event, renderer)
            {
                shell.publish(message);
            }
        }

        self.view = self.component.view(&internal.state);
        tree.diff_children(&[&self.view]);

        let previous_size = self.layout.size();
        self.layout = self.view.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &self.limits,
        );

        let new_size_hint = self.component.size_hint();

        // We must invalidate application layout in 3 instances:
        //
        // 1. The size hint of the component changes. Other widgets
        //    may change layout behavior.
        //
        // 2. The size hint of the component is `Shrink` for any axis
        //    and the component has changed size. The new size may
        //    push other widgets around.
        //
        // 3. The overlay status of the component changes. The
        //    runtime will only call `overlay` again if the layout
        //    is invalidated.
        if new_size_hint != self.size_hint {
            self.size_hint = new_size_hint;
            shell.invalidate_widgets();
        } else if (self.size_hint.width == Length::Shrink
            || self.size_hint.height == Length::Shrink)
            && previous_size != self.layout.size()
        {
            shell.invalidate_layout();
        } else {
            let has_overlay = self
                .view
                .as_widget_mut()
                .overlay(
                    &mut tree.children[0],
                    Layout::with_offset(
                        layout.position() - Point::ORIGIN,
                        &self.layout,
                    ),
                    renderer,
                    viewport,
                    Vector::ZERO,
                )
                .is_some();

            if self.has_overlay != has_overlay {
                self.has_overlay = has_overlay;
                shell.invalidate_layout();
            }
        }

        self.is_outdated = false;

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            let internal =
                tree.state.downcast_mut::<Internal<C::State, C::Event>>();

            let mut local_shell = Shell::new(&mut internal.events);

            self.view.as_widget_mut().update(
                &mut tree.children[0],
                event,
                Layout::with_offset(
                    layout.position() - Point::ORIGIN,
                    &self.layout,
                ),
                cursor,
                renderer,
                clipboard,
                &mut local_shell,
                viewport,
            );

            if internal.events.is_empty() {
                return;
            }
        }

        shell.request_redraw();
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.view.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            Layout::with_offset(
                layout.position() - Point::ORIGIN,
                &self.layout,
            ),
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let internal =
            tree.state.downcast_ref::<Internal<C::State, C::Event>>();

        let interaction = self.component.mouse_interaction(&internal.state);

        if interaction != mouse::Interaction::None {
            return interaction;
        }

        self.view.as_widget().mouse_interaction(
            &tree.children[0],
            Layout::with_offset(
                layout.position() - Point::ORIGIN,
                &self.layout,
            ),
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let internal =
            tree.state.downcast_ref::<Internal<C::State, C::Event>>();

        self.component
            .operate(&internal.state, layout.bounds(), operation);

        self.view.as_widget_mut().operate(
            &mut tree.children[0],
            Layout::with_offset(
                layout.position() - Point::ORIGIN,
                &self.layout,
            ),
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let overlay = self.view.as_widget_mut().overlay(
            &mut tree.children[0],
            Layout::with_offset(
                layout.position() - Point::ORIGIN,
                &self.layout,
            ),
            renderer,
            viewport,
            translation,
        )?;

        self.has_overlay = true;

        Some(overlay::Element::new(Box::new(Overlay {
            component: &mut self.component,
            internal: tree.state.downcast_mut(),
            raw: overlay,
            is_outdated: &mut self.is_outdated,
        })))
    }
}

struct Overlay<'a, 'b, C, Message, Theme, Renderer>
where
    C: Component<'a, Message, Theme, Renderer>,
{
    component: &'b mut C,
    internal: &'b mut Internal<C::State, C::Event>,
    is_outdated: &'b mut bool,
    raw: overlay::Element<'b, C::Event, Theme, Renderer>,
}

impl<'a, 'b, C, Message, Theme, Renderer>
    overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'a, 'b, C, Message, Theme, Renderer>
where
    C: Component<'a, Message, Theme, Renderer>,
    Renderer: core::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.raw.as_overlay_mut().layout(renderer, bounds)
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let mut local_shell = Shell::new(&mut self.internal.events);

        self.raw.as_overlay_mut().update(
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
        );

        if local_shell.is_event_captured() {
            shell.capture_event();
        }

        if local_shell.is_layout_invalid() {
            shell.invalidate_layout();
        }

        if local_shell.are_widgets_invalid() {
            shell.invalidate_widgets();
        }

        shell.request_redraw_at(local_shell.redraw_request());
        shell.request_input_method(local_shell.input_method());

        if self.internal.events.is_empty() {
            return;
        }

        for event in self.internal.events.drain(..) {
            if let Some(message) =
                self.component
                    .update(&mut self.internal.state, event, renderer)
            {
                shell.publish(message);
            }
        }

        *self.is_outdated = true;

        shell.invalidate_layout();
        shell.request_redraw();
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.raw
            .as_overlay()
            .draw(renderer, theme, style, layout, cursor);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.raw
            .as_overlay()
            .mouse_interaction(layout, cursor, renderer)
    }

    fn index(&self) -> f32 {
        self.raw.as_overlay().index()
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.raw
            .as_overlay_mut()
            .operate(layout, renderer, operation);
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let overlay = self.raw.as_overlay_mut().overlay(layout, renderer)?;

        Some(overlay::Element::new(Box::new(Overlay {
            component: self.component,
            raw: overlay,
            internal: self.internal,
            is_outdated: self.is_outdated,
        })))
    }
}
