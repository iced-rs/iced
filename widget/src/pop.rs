//! Generate messages when content pops in and out of view.
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Clipboard, Element, Event, Layout, Length, Pixels, Rectangle, Shell,
    Size, Vector, Widget,
};

/// A widget that can generate messages when its content pops in and out of view.
///
/// It can even notify you with anticipation at a given distance!
#[allow(missing_debug_implementations)]
pub struct Pop<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_show: Option<Message>,
    on_hide: Option<Message>,
    anticipate: Pixels,
}

impl<'a, Message, Theme, Renderer> Pop<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
    Message: Clone,
{
    /// Creates a new [`Pop`] widget with the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            on_show: None,
            on_hide: None,
            anticipate: Pixels::ZERO,
        }
    }

    /// Sets the message to be produced when the content pops into view.
    pub fn on_show(mut self, on_show: Message) -> Self {
        self.on_show = Some(on_show);
        self
    }

    /// Sets the message to be produced when the content pops out of view.
    pub fn on_hide(mut self, on_hide: Message) -> Self {
        self.on_hide = Some(on_hide);
        self
    }

    /// Sets the distance in [`Pixels`] to use in anticipation of the
    /// content popping into view.
    ///
    /// This can be quite useful to lazily load items in a long scrollable
    /// behind the scenes before the user can notice it!
    pub fn anticipate(mut self, distance: impl Into<Pixels>) -> Self {
        self.anticipate = distance.into();
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    has_popped_in: bool,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Pop<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Window(window::Event::RedrawRequested(_)) = &event {
            let state = tree.state.downcast_mut::<State>();
            let bounds = layout.bounds();

            let top_left_distance = viewport.distance(bounds.position());

            let bottom_right_distance = viewport
                .distance(bounds.position() + Vector::from(bounds.size()));

            let distance = top_left_distance.min(bottom_right_distance);

            if state.has_popped_in {
                if let Some(on_hide) = &self.on_hide {
                    if distance > self.anticipate.0 {
                        state.has_popped_in = false;
                        shell.publish(on_hide.clone());
                    }
                }
            } else if let Some(on_show) = &self.on_show {
                if distance <= self.anticipate.0 {
                    state.has_popped_in = true;
                    shell.publish(on_show.clone());
                }
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: core::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: core::Layout<'_>,
        renderer: &Renderer,
        translation: core::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Pop<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer + 'a,
    Theme: 'a,
    Message: Clone + 'a,
{
    fn from(pop: Pop<'a, Message, Theme, Renderer>) -> Self {
        Element::new(pop)
    }
}
