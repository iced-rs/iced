//! Build responsive widgets.
use crate::{Cache, CacheBuilder};

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::{
    Clipboard, Element, Hasher, Length, Point, Rectangle, Shell, Size, Widget,
};

use std::cell::RefCell;
use std::hash::{Hash, Hasher as _};
use std::ops::Deref;

/// The state of a [`Responsive`] widget.
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

    fn layout(&self, parent: Layout<'_>) -> Layout<'_> {
        Layout::with_offset(
            parent.position() - Point::ORIGIN,
            &self.last_layout,
        )
    }
}

/// A widget that is aware of its dimensions.
///
/// A [`Responsive`] widget will always try to fill all the available space of
/// its parent.
#[allow(missing_debug_implementations)]
pub struct Responsive<'a, Message, Renderer>(
    RefCell<Internal<'a, Message, Renderer>>,
);

impl<'a, Message, Renderer> Responsive<'a, Message, Renderer> {
    /// Creates a new [`Responsive`] widget with the given [`State`] and a
    /// closure that produces its contents.
    ///
    /// The `view` closure will be provided with the current [`Size`] of
    /// the [`Responsive`] widget and, therefore, can be used to build the
    /// contents of the widget in a responsive way.
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
    Renderer: iced_native::Renderer,
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
        let mut internal = self.0.borrow_mut();

        if internal.state.last_size != Some(internal.state.last_layout.size()) {
            shell.invalidate_widgets();
        }

        internal.resolve(renderer, |state, renderer, content| {
            content.on_event(
                event,
                state.layout(layout),
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        })
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let mut internal = self.0.borrow_mut();

        internal.resolve(renderer, |state, renderer, content| {
            content.draw(
                renderer,
                style,
                state.layout(layout),
                cursor_position,
                viewport,
            )
        })
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut internal = self.0.borrow_mut();

        internal.resolve(renderer, |state, renderer, content| {
            content.mouse_interaction(
                state.layout(layout),
                cursor_position,
                viewport,
                renderer,
            )
        })
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let has_overlay = {
            use std::ops::DerefMut;

            let mut internal = self.0.borrow_mut();

            let _ =
                internal.resolve(renderer, |_state, _renderer, _content| {});

            let Internal { content, state } = internal.deref_mut();

            let content_layout = state.layout(layout);

            match content {
                Content::Pending(_) => false,
                Content::Ready(cache) => {
                    *cache = Some(
                        CacheBuilder {
                            element: cache.take().unwrap().into_heads().element,
                            overlay_builder: |element| {
                                element.overlay(content_layout, renderer)
                            },
                        }
                        .build(),
                    );

                    cache.as_ref().unwrap().borrow_overlay().is_some()
                }
            }
        };

        has_overlay.then(|| {
            overlay::Element::new(
                layout.position(),
                Box::new(Overlay { instance: self }),
            )
        })
    }
}

struct Internal<'a, Message, Renderer> {
    state: &'a mut State,
    content: Content<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Internal<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn resolve<R, T>(
        &mut self,
        renderer: R,
        f: impl FnOnce(&State, R, &mut Element<'a, Message, Renderer>) -> T,
    ) -> T
    where
        R: Deref<Target = Renderer>,
    {
        self.content.resolve(&mut self.state, renderer, f)
    }
}

enum Content<'a, Message, Renderer> {
    Pending(
        Option<Box<dyn FnOnce(Size) -> Element<'a, Message, Renderer> + 'a>>,
    ),
    Ready(Option<Cache<'a, Message, Renderer>>),
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn resolve<R, T>(
        &mut self,
        state: &mut State,
        renderer: R,
        f: impl FnOnce(&State, R, &mut Element<'a, Message, Renderer>) -> T,
    ) -> T
    where
        R: Deref<Target = Renderer>,
    {
        match self {
            Content::Ready(cache) => {
                let mut heads = cache.take().unwrap().into_heads();

                let result = f(state, renderer, &mut heads.element);

                *cache = Some(
                    CacheBuilder {
                        element: heads.element,
                        overlay_builder: |_| None,
                    }
                    .build(),
                );

                result
            }
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
                        renderer.deref(),
                        &layout::Limits::new(
                            Size::ZERO,
                            state.last_size.unwrap_or(Size::ZERO),
                        ),
                    );

                    state.last_layout_hash = new_layout_hash;
                }

                *self = Content::Ready(Some(
                    CacheBuilder {
                        element,
                        overlay_builder: |_| None,
                    }
                    .build(),
                ));

                self.resolve(state, renderer, f)
            }
        }
    }
}

impl<'a, Message, Renderer> From<Responsive<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + 'a,
    Message: 'a,
{
    fn from(responsive: Responsive<'a, Message, Renderer>) -> Self {
        Self::new(responsive)
    }
}

struct Overlay<'a, 'b, Message, Renderer> {
    instance: &'b mut Responsive<'a, Message, Renderer>,
}

impl<'a, 'b, Message, Renderer> Overlay<'a, 'b, Message, Renderer> {
    fn with_overlay_maybe<T>(
        &self,
        f: impl FnOnce(&overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        let internal = self.instance.0.borrow();

        match &internal.content {
            Content::Pending(_) => None,
            Content::Ready(cache) => {
                cache.as_ref().unwrap().borrow_overlay().as_ref().map(f)
            }
        }
    }

    fn with_overlay_mut_maybe<T>(
        &self,
        f: impl FnOnce(&mut overlay::Element<'_, Message, Renderer>) -> T,
    ) -> Option<T> {
        let mut internal = self.instance.0.borrow_mut();

        match &mut internal.content {
            Content::Pending(_) => None,
            Content::Ready(cache) => cache
                .as_mut()
                .unwrap()
                .with_overlay_mut(|overlay| overlay.as_mut().map(f)),
        }
    }
}

impl<'a, 'b, Message, Renderer> overlay::Overlay<Message, Renderer>
    for Overlay<'a, 'b, Message, Renderer>
where
    Renderer: iced_native::Renderer,
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        self.with_overlay_maybe(|overlay| {
            overlay.draw(renderer, style, layout, cursor_position);
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

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        (position.x as u32).hash(state);
        (position.y as u32).hash(state);

        self.with_overlay_maybe(|overlay| {
            overlay.hash_layout(state);
        });
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
        self.with_overlay_mut_maybe(|overlay| {
            overlay.on_event(
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        })
        .unwrap_or_else(|| iced_native::event::Status::Ignored)
    }
}
