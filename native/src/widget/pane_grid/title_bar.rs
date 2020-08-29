use crate::layout;
use crate::pane_grid;
use crate::{
    Clipboard, Element, Event, Hasher, Layout, Point, Rectangle, Size,
};

/// The title bar of a [`Pane`].
///
/// [`Pane`]: struct.Pane.html
#[allow(missing_debug_implementations)]
pub struct TitleBar<'a, Message, Renderer: pane_grid::Renderer> {
    title: String,
    title_size: Option<u16>,
    controls: Option<Element<'a, Message, Renderer>>,
    padding: u16,
    always_show_controls: bool,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    /// Creates a new [`TitleBar`] with the given title.
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            title_size: None,
            controls: None,
            padding: 0,
            always_show_controls: false,
            style: Renderer::Style::default(),
        }
    }

    /// Sets the size of the title of the [`TitleBar`].
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    pub fn title_size(mut self, size: u16) -> Self {
        self.title_size = Some(size);
        self
    }

    /// Sets the controls of the [`TitleBar`].
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    pub fn controls(
        mut self,
        controls: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        self.controls = Some(controls.into());
        self
    }

    /// Sets the padding of the [`TitleBar`].
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the style of the [`TitleBar`].
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets whether or not the [`controls`] attached to this [`TitleBar`] are
    /// always visible.
    ///
    /// By default, the controls are only visible when the [`Pane`] of this
    /// [`TitleBar`] is hovered.
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    /// [`controls`]: struct.TitleBar.html#method.controls
    /// [`Pane`]: struct.Pane.html
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
    /// [`TitleBar`]: struct.TitleBar.html
    /// [`Renderer`]: trait.Renderer.html
    /// [`Layout`]: ../layout/struct.Layout.html
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

        if let Some(controls) = &self.controls {
            let mut children = padded.children();
            let title_layout = children.next().unwrap();
            let controls_layout = children.next().unwrap();

            let (title_bounds, controls) =
                if show_controls || self.always_show_controls {
                    (title_layout.bounds(), Some((controls, controls_layout)))
                } else {
                    (
                        Rectangle {
                            width: padded.bounds().width,
                            ..title_layout.bounds()
                        },
                        None,
                    )
                };

            renderer.draw_title_bar(
                defaults,
                layout.bounds(),
                &self.style,
                &self.title,
                self.title_size.unwrap_or(renderer.default_size()),
                Renderer::Font::default(),
                title_bounds,
                controls,
                cursor_position,
            )
        } else {
            renderer.draw_title_bar::<()>(
                defaults,
                layout.bounds(),
                &self.style,
                &self.title,
                self.title_size.unwrap_or(renderer.default_size()),
                Renderer::Font::default(),
                padded.bounds(),
                None,
                cursor_position,
            )
        }
    }

    /// Returns whether the mouse cursor is over the pick area of the
    /// [`TitleBar`] or not.
    ///
    /// The whole [`TitleBar`] is a pick area, except its controls.
    ///
    /// [`TitleBar`]: struct.TitleBar.html
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

        self.title.hash(hasher);
        self.title_size.hash(hasher);
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

        let title_size = self.title_size.unwrap_or(renderer.default_size());
        let title_font = Renderer::Font::default();

        let (title_width, title_height) = renderer.measure(
            &self.title,
            title_size,
            title_font,
            Size::new(f32::INFINITY, max_size.height),
        );

        let mut node = if let Some(controls) = &self.controls {
            let mut controls_layout = controls
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let controls_size = controls_layout.size();
            let space_before_controls = max_size.width - controls_size.width;

            let mut title_layout = layout::Node::new(Size::new(
                title_width.min(space_before_controls),
                title_height,
            ));

            let title_size = title_layout.size();
            let height = title_size.height.max(controls_size.height);

            title_layout
                .move_to(Point::new(0.0, (height - title_size.height) / 2.0));
            controls_layout.move_to(Point::new(space_before_controls, 0.0));

            layout::Node::with_children(
                Size::new(max_size.width, height),
                vec![title_layout, controls_layout],
            )
        } else {
            layout::Node::new(Size::new(max_size.width, title_height))
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
    ) {
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
            );
        }
    }
}
