use crate::container;
use crate::layout;
use crate::overlay;
use crate::pane_grid::{self, TitleBar};
use crate::{Clipboard, Element, Event, Hasher, Layout, Point, Size};

/// The content of a [`Pane`].
///
/// [`Pane`]: struct.Pane.html
#[allow(missing_debug_implementations)]
pub struct Content<'a, Message, Renderer: pane_grid::Renderer> {
    title_bar: Option<TitleBar<'a, Message, Renderer>>,
    body: Element<'a, Message, Renderer>,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> Content<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    /// Creates a new [`Content`] with the provided body.
    ///
    /// [`Content`]: struct.Content.html
    pub fn new(body: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            title_bar: None,
            body: body.into(),
            style: Renderer::Style::default(),
        }
    }

    /// Sets the [`TitleBar`] of this [`Content`].
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    /// [`Content`]: struct.Content.html
    pub fn title_bar(
        mut self,
        title_bar: TitleBar<'a, Message, Renderer>,
    ) -> Self {
        self.title_bar = Some(title_bar);
        self
    }

    /// Sets the style of the [`Content`].
    ///
    /// [`Content`]: struct.Content.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
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
    /// [`Content`]: struct.Content.html
    /// [`Renderer`]: trait.Renderer.html
    /// [`Layout`]: ../layout/struct.Layout.html
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
                layout.bounds(),
                &self.style,
                Some((title_bar, title_bar_layout)),
                (&self.body, body_layout),
                cursor_position,
            )
        } else {
            renderer.draw_pane(
                defaults,
                layout.bounds(),
                &self.style,
                None,
                (&self.body, layout),
                cursor_position,
            )
        }
    }

    /// Returns whether the [`Content`] with the given [`Layout`] can be picked
    /// at the provided cursor position.
    ///
    /// [`Content`]: struct.Content.html
    /// [`Layout`]: ../layout/struct.Layout.html
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
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        let body_layout = if let Some(title_bar) = &mut self.title_bar {
            let mut children = layout.children();

            title_bar.on_event(
                event.clone(),
                children.next().unwrap(),
                cursor_position,
                messages,
                renderer,
                clipboard,
            );

            children.next().unwrap()
        } else {
            layout
        };

        self.body.on_event(
            event,
            body_layout,
            cursor_position,
            messages,
            renderer,
            clipboard,
        );
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
        let body_layout = if self.title_bar.is_some() {
            let mut children = layout.children();

            // Overlays only allowed in the pane body, for now at least.
            let _title_bar_layout = children.next();

            children.next()?
        } else {
            layout
        };

        self.body.overlay(body_layout)
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
