use crate::container;
use crate::layout;
use crate::pane_grid;
use crate::{Element, Layout, Point, Size};

pub struct TitleBar<'a, Message, Renderer: container::Renderer> {
    title: Element<'a, Message, Renderer>,
    controls: Option<Element<'a, Message, Renderer>>,
    padding: u16,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: container::Renderer,
{
    pub fn new(title: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Self {
            title: title.into(),
            controls: None,
            padding: 0,
            style: Renderer::Style::default(),
        }
    }

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
    ) -> Renderer::Output {
        if let Some(controls) = &self.controls {
            let mut children = layout.children();
            let title_layout = children.next().unwrap();
            let controls_layout = children.next().unwrap();

            renderer.draw_title_bar(
                defaults,
                &self.style,
                (&self.title, title_layout),
                Some((controls, controls_layout)),
                cursor_position,
            )
        } else {
            renderer.draw_title_bar(
                defaults,
                &self.style,
                (&self.title, layout),
                None,
                cursor_position,
            )
        }
    }

    pub(crate) fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(self.padding);
        let limits = limits.pad(padding);

        let mut node = if let Some(controls) = &self.controls {
            let max_size = limits.max();

            let title_layout = self
                .title
                .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

            let title_size = title_layout.size();

            let mut controls_layout = controls.layout(
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    Size::new(
                        max_size.width - title_size.width,
                        max_size.height,
                    ),
                ),
            );

            controls_layout.move_to(Point::new(title_size.width, 0.0));

            layout::Node::with_children(
                max_size,
                vec![title_layout, controls_layout],
            )
        } else {
            self.title.layout(renderer, &limits)
        };

        node.move_to(Point::new(padding, padding));

        node
    }
}
