use crate::layout;
use crate::pane_grid;
use crate::{Clipboard, Element, Event, Layout, Point, Rectangle, Size};

pub struct TitleBar<'a, Message, Renderer: pane_grid::Renderer> {
    title: String,
    title_size: Option<u16>,
    controls: Option<Element<'a, Message, Renderer>>,
    padding: u16,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            title_size: None,
            controls: None,
            padding: 0,
            style: Renderer::Style::default(),
        }
    }

    /// Sets the size of the title of the [`TitleBar`].
    ///
    /// [`TitleBar`]: struct.Text.html
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
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: pane_grid::Renderer,
{
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

            let (title_bounds, controls) = if show_controls {
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
                self.title_size.unwrap_or(Renderer::DEFAULT_SIZE),
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
                self.title_size.unwrap_or(Renderer::DEFAULT_SIZE),
                Renderer::Font::default(),
                padded.bounds(),
                None,
                cursor_position,
            )
        }
    }

    pub fn is_over_draggable(
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

    pub(crate) fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(self.padding);
        let limits = limits.pad(padding);
        let max_size = limits.max();

        let title_size = self.title_size.unwrap_or(Renderer::DEFAULT_SIZE);
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
            layout::Node::new(Size::new(title_width, title_height))
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
