use crate::container;
use crate::event::{self, Event};
use crate::layout;
use crate::pane_grid;
use crate::{Clipboard, Element, Hasher, Layout, Point, Size};

/// The title bar of a [`Pane`].
///
/// [`Pane`]: crate::widget::pane_grid::Pane
#[allow(missing_debug_implementations)]
pub struct TitleBar<'a, Message, Renderer: pane_grid::Renderer> {
    content: Element<'a, Message, Renderer>,
    controls: Option<Element<'a, Message, Renderer>>,
    padding: u16,
    always_show_controls: bool,
    style: <Renderer as container::Renderer>::Style,
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    /// Creates a new [`TitleBar`] with the given content.
    pub fn new<E>(content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            content: content.into(),
            controls: None,
            padding: 0,
            always_show_controls: false,
            style: Default::default(),
        }
    }

    /// Sets the controls of the [`TitleBar`].
    pub fn controls(
        mut self,
        controls: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.controls = Some(controls.into());
        self
    }

    /// Sets the padding of the [`TitleBar`].
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the style of the [`TitleBar`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer as container::Renderer>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Sets whether or not the [`controls`] attached to this [`TitleBar`] are
    /// always visible.
    ///
    /// By default, the controls are only visible when the [`Pane`] of this
    /// [`TitleBar`] is hovered.
    ///
    /// [`controls`]: Self::controls
    /// [`Pane`]: crate::widget::pane_grid::Pane
    pub fn always_show_controls(mut self) -> Self {
        self.always_show_controls = true;
        self
    }
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    /// Draws the [`TitleBar`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: crate::widget::pane_grid::Renderer
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        show_controls: bool,
    ) -> Renderer::Output {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();

        let controls = if let Some(controls) = &self.controls {
            let controls_layout = children.next().unwrap();

            if show_controls || self.always_show_controls {
                Some((controls, controls_layout))
            } else {
                None
            }
        } else {
            None
        };

        renderer.draw_title_bar(
            defaults,
            layout.bounds(),
            &self.style,
            (&self.content, title_layout),
            controls,
            cursor_position,
        )
    }

    /// Returns whether the mouse cursor is over the pick area of the
    /// [`TitleBar`] or not.
    ///
    /// The whole [`TitleBar`] is a pick area, except its controls.
    pub fn is_over_pick_area(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> bool {
        if layout.bounds().contains(cursor_position) {
            let mut children = layout.children();
            let padded = children.next().unwrap();

            if self.controls.is_some() {
                let mut children = padded.children();
                let _ = children.next().unwrap();
                let controls_layout = children.next().unwrap();

                !controls_layout.bounds().contains(cursor_position)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub(crate) fn hash_layout(&self, hasher: &mut Hasher) {
        use std::hash::Hash;

        self.content.hash_layout(hasher);
        self.padding.hash(hasher);
    }

    pub(crate) fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(self.padding);
        let limits = limits.pad(padding);
        let max_size = limits.max();

        let title_layout = self
            .content
            .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));
        let title_size = title_layout.size();

        let mut node = if let Some(controls) = &self.controls {
            let mut controls_layout = controls
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let controls_size = controls_layout.size();
            let space_before_controls = max_size.width - controls_size.width;

            let height = title_size.height.max(controls_size.height);

            controls_layout.move_to(Point::new(space_before_controls, 0.0));

            layout::Node::with_children(
                Size::new(max_size.width, height),
                vec![title_layout, controls_layout],
            )
        } else {
            layout::Node::with_children(
                Size::new(max_size.width, title_size.height),
                vec![title_layout],
            )
        };

        node.move_to(Point::new(padding, padding));

        layout::Node::with_children(node.size().pad(padding), vec![node])
    }

    pub(crate) fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        if let Some(controls) = &mut self.controls {
            let mut children = layout.children();
            let padded = children.next().unwrap();

            let mut children = padded.children();
            let _ = children.next();
            let controls_layout = children.next().unwrap();

            controls.on_event(
                event,
                controls_layout,
                cursor_position,
                messages,
                renderer,
                clipboard,
            )
        } else {
            event::Status::Ignored
        }
    }
}
