use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget::operation::Operation;
use crate::widget::scrollable::Direction;
use crate::widget::tree::{self, Tree};
use crate::{
    Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Widget,
};

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Vertical<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: super::StyleSheet,
{
    id: Option<super::Id>,
    height: Length,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    content: Element<'a, Message, Renderer>,
    on_scroll: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    style: <Renderer::Theme as super::StyleSheet>::Style,
}

impl<'a, Message, Renderer> Vertical<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: super::StyleSheet,
{
    /// Creates a new [`Vertical`] scrollable.
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Vertical {
            id: None,
            height: Length::Shrink,
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            content: content.into(),
            on_scroll: None,
            style: Default::default(),
        }
    }

    /// Sets the [`Id`] of the [`Vertical`] scrollable.
    pub fn id(mut self, id: super::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the height of the [`Vertical`] scrollable.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the scrollbar width of the [`Vertical`] scrollable.
    /// Silently enforces a minimum value of 1.
    pub fn scrollbar_width(mut self, scrollbar_width: u16) -> Self {
        self.scrollbar_width = scrollbar_width.max(1);
        self
    }

    /// Sets the scrollbar margin of the [`Vertical`] scrollable.
    pub fn scrollbar_margin(mut self, scrollbar_margin: u16) -> Self {
        self.scrollbar_margin = scrollbar_margin;
        self
    }

    /// Sets the scroller width of the [`Vertical`] scrollable.
    ///
    /// It silently enforces a minimum value of 1.
    pub fn scroller_width(mut self, scroller_width: u16) -> Self {
        self.scroller_width = scroller_width.max(1);
        self
    }

    /// Sets a function to call when the [`Vertical`] scrollable is scrolled.
    ///
    /// The function takes the new relative offset of the [`Vertical`] scrollable,
    /// (e.g. `0` means top, while `1` means bottom).
    pub fn on_scroll(mut self, f: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Sets the style of the [`Vertical`] scrollable.
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as super::StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Vertical<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: super::StyleSheet,
{
    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        super::layout(
            &Direction::Vertical,
            renderer,
            limits,
            Widget::<Message, Renderer>::width(self),
            self.height,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        super::draw(
            &Direction::Vertical,
            tree.state.downcast_ref::<super::State>(),
            renderer,
            theme,
            layout,
            cursor_position,
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
            self.style,
            |renderer, layout, cursor_position, viewport| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor_position,
                    viewport,
                )
            },
        )
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<super::State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(super::State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        super::operate(tree, self.id.as_ref(), &self.content, layout, operation)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        super::update(
            &Direction::Vertical,
            tree.state.downcast_mut::<super::State>(),
            event,
            layout,
            cursor_position,
            clipboard,
            shell,
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
            &self.on_scroll,
            |event, layout, cursor_position, clipboard, shell| {
                self.content.as_widget_mut().on_event(
                    &mut tree.children[0],
                    event,
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            },
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        super::mouse_interaction(
            &Direction::Vertical,
            tree.state.downcast_ref::<super::State>(),
            layout,
            cursor_position,
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
            |layout, cursor_position, viewport| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            },
        )
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        super::overlay(
            &Direction::Vertical,
            tree,
            layout,
            renderer,
            &self.content,
        )
    }
}

impl<'a, Message, Renderer> From<Vertical<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + crate::Renderer,
    Renderer::Theme: super::StyleSheet,
{
    fn from(
        scrollable: Vertical<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(scrollable)
    }
}
