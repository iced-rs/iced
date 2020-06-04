use crate::layout;
use crate::pane_grid::{self, TitleBar};
use crate::{Clipboard, Element, Event, Hasher, Layout, Point};

/// The content of a [`Pane`].
///
/// [`Pane`]: struct.Pane.html
pub struct Content<'a, Message, Renderer> {
    title_bar: Option<TitleBar<'a, Message, Renderer>>,
    body: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer> {
    pub fn new(body: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            title_bar: None,
            body: body.into(),
        }
    }

    pub fn title_bar(
        mut self,
        title_bar: TitleBar<'a, Message, Renderer>,
    ) -> Self {
        self.title_bar = Some(title_bar);
        self
    }
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw_pane(
            defaults,
            self.title_bar.as_ref(),
            &self.body,
            layout,
            cursor_position,
        )
    }

    pub(crate) fn is_over_drag_target(
        &self,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> bool {
        false
    }

    pub(crate) fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.body.layout(renderer, limits)
    }

    pub(crate) fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        self.body.on_event(
            event,
            layout,
            cursor_position,
            messages,
            renderer,
            clipboard,
        )
    }

    pub(crate) fn hash_layout(&self, state: &mut Hasher) {
        self.body.hash_layout(state);
    }
}
