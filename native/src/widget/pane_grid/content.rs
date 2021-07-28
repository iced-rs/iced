use crate::container;
use crate::event::{self, Event};
use crate::layout;
use crate::overlay;
use crate::pane_grid::{self, TitleBar};
use crate::{Clipboard, Element, Hasher, Layout, Point, Rectangle, Size};

/// The content of a [`Pane`].
///
/// [`Pane`]: crate::widget::pane_grid::Pane
#[allow(missing_debug_implementations)]
pub struct Content<'a, Message, Renderer: pane_grid::Renderer> {
    title_bar: Option<TitleBar<'a, Message, Renderer>>,
    body: Element<'a, Message, Renderer>,
    style: <Renderer as container::Renderer>::Style,
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    /// Creates a new [`Content`] with the provided body.
    pub fn new(body: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            title_bar: None,
            body: body.into(),
            style: Default::default(),
        }
    }

    /// Sets the [`TitleBar`] of this [`Content`].
    pub fn title_bar(
        mut self,
        title_bar: TitleBar<'a, Message, Renderer>,
    ) -> Self {
        self.title_bar = Some(title_bar);
        self
    }

    /// Sets the style of the [`Content`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as container::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    /// Draws the [`Content`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: crate::widget::pane_grid::Renderer
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();
            let body_layout = children.next().unwrap();

            renderer.draw_pane(
                defaults,
                layout.bounds(),
                &self.style,
                Some((title_bar, title_bar_layout)),
                (&self.body, body_layout),
                cursor_position,
                viewport,
            )
        } else {
            renderer.draw_pane(
                defaults,
                layout.bounds(),
                &self.style,
                None,
                (&self.body, layout),
                cursor_position,
                viewport,
            )
        }
    }

    /// Returns whether the [`Content`] with the given [`Layout`] can be picked
    /// at the provided cursor position.
    pub fn can_be_picked_at(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> bool {
        if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();

            title_bar.is_over_pick_area(title_bar_layout, cursor_position)
        } else {
            false
        }
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
        is_picked: bool,
    ) -> event::Status {
        let mut event_status = event::Status::Ignored;

        let body_layout = if let Some(title_bar) = &mut self.title_bar {
            let mut children = layout.children();

            event_status = title_bar.on_event(
                event.clone(),
                children.next().unwrap(),
                cursor_position,
                renderer,
                clipboard,
                messages,
            );

            children.next().unwrap()
        } else {
            layout
        };

        let body_status = if is_picked {
            event::Status::Ignored
        } else {
            self.body.on_event(
                event,
                body_layout,
                cursor_position,
                renderer,
                clipboard,
                messages,
            )
        };

        event_status.merge(body_status)
    }

    pub(crate) fn hash_layout(&self, state: &mut Hasher) {
        if let Some(title_bar) = &self.title_bar {
            title_bar.hash_layout(state);
        }

        self.body.hash_layout(state);
    }

    pub(crate) fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        if let Some(title_bar) = self.title_bar.as_mut() {
            let mut children = layout.children();
            let title_bar_layout = children.next()?;

            match title_bar.overlay(title_bar_layout) {
                Some(overlay) => Some(overlay),
                None => self.body.overlay(children.next()?),
            }
        } else {
            self.body.overlay(layout)
        }
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
