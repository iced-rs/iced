use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    self, Clipboard, Element, Event, Length, Rectangle, Shell, Size, Vector,
    Widget,
};
use crate::horizontal_space;

/// A widget that is aware of its dimensions.
///
/// A [`Responsive`] widget will always try to fill all the available space of
/// its parent.
#[allow(missing_debug_implementations)]
pub struct Responsive<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> {
    view: Box<dyn Fn(Size) -> Element<'a, Message, Theme, Renderer> + 'a>,
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Responsive<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    /// Creates a new [`Responsive`] widget with a closure that produces its
    /// contents.
    ///
    /// The `view` closure will be provided with the current [`Size`] of
    /// the [`Responsive`] widget and, therefore, can be used to build the
    /// contents of the widget in a responsive way.
    pub fn new(
        view: impl Fn(Size) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        Self {
            view: Box::new(view),
            content: Element::new(horizontal_space().width(0)),
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Responsive<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        struct Marker;
        tree::Tag::of::<Marker>()
    }

    fn diff(&mut self, _tree: &mut Tree) {
        // Diff is deferred to layout
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.max();

        self.content = (self.view)(size);
        tree.diff_children(std::slice::from_mut(&mut self.content));

        self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &limits.loose(),
        )
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

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
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

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer>
    From<Responsive<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(responsive: Responsive<'a, Message, Theme, Renderer>) -> Self {
        Self::new(responsive)
    }
}
