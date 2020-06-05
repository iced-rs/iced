use crate::container;
use crate::layout;
use crate::pane_grid::{self, TitleBar};
use crate::{Clipboard, Element, Event, Hasher, Layout, Point, Size};

/// The content of a [`Pane`].
///
/// [`Pane`]: struct.Pane.html
pub struct Content<'a, Message, Renderer: container::Renderer> {
    title_bar: Option<TitleBar<'a, Message, Renderer>>,
    body: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: container::Renderer,
{
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
    Renderer: pane_grid::Renderer + container::Renderer,
{
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();
            let body_layout = children.next().unwrap();

            renderer.draw_pane(
                defaults,
                Some((title_bar, title_bar_layout)),
                (&self.body, body_layout),
                cursor_position,
            )
        } else {
            renderer.draw_pane(
                defaults,
                None,
                (&self.body, layout),
                cursor_position,
            )
        }
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
        if let Some(title_bar) = &self.title_bar {
            let max_size = limits.max();

            let title_bar_layout = title_bar
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let title_bar_size = title_bar_layout.size();

            let mut body_layout = self.body.layout(
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    Size::new(
                        max_size.width,
                        max_size.height - title_bar_size.height,
                    ),
                ),
            );

            body_layout.move_to(Point::new(0.0, title_bar_size.height));

            layout::Node::with_children(
                max_size,
                vec![title_bar_layout, body_layout],
            )
        } else {
            self.body.layout(renderer, limits)
        }
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

impl<'a, T, Message, Renderer> From<T> for Content<'a, Message, Renderer>
where
    T: Into<Element<'a, Message, Renderer>>,
    Renderer: pane_grid::Renderer + container::Renderer,
{
    fn from(element: T) -> Self {
        Self::new(element)
    }
}
