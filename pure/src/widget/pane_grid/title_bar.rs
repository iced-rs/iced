use crate::widget::Tree;
use crate::Element;

use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget::container;
use iced_native::{Clipboard, Layout, Padding, Point, Rectangle, Shell, Size};

/// The title bar of a [`Pane`].
///
/// [`Pane`]: crate::widget::pane_grid::Pane
#[allow(missing_debug_implementations)]
pub struct TitleBar<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
    controls: Option<Element<'a, Message, Renderer>>,
    padding: Padding,
    always_show_controls: bool,
    style_sheet: Box<dyn container::StyleSheet + 'a>,
}

impl<'a, Message, Renderer> TitleBar<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    /// Creates a new [`TitleBar`] with the given content.
    pub fn new<E>(content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            content: content.into(),
            controls: None,
            padding: Padding::ZERO,
            always_show_controls: false,
            style_sheet: Default::default(),
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

    /// Sets the [`Padding`] of the [`TitleBar`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the style of the [`TitleBar`].
    pub fn style(
        mut self,
        style: impl Into<Box<dyn container::StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style.into();
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
    Renderer: iced_native::Renderer,
{
    pub(super) fn state(&self) -> Tree {
        let children = if let Some(controls) = self.controls.as_ref() {
            vec![Tree::new(&self.content), Tree::new(controls)]
        } else {
            vec![Tree::new(&self.content), Tree::empty()]
        };

        Tree {
            children,
            ..Tree::empty()
        }
    }

    pub(super) fn diff(&self, tree: &mut Tree) {
        if tree.children.len() == 2 {
            if let Some(controls) = self.controls.as_ref() {
                tree.children[1].diff(controls);
            }

            tree.children[0].diff(&self.content);
        } else {
            *tree = self.state();
        }
    }

    /// Draws the [`TitleBar`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: iced_native::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        show_controls: bool,
    ) {
        let bounds = layout.bounds();
        let style = self.style_sheet.style();
        let inherited_style = renderer::Style {
            text_color: style.text_color.unwrap_or(inherited_style.text_color),
        };

        container::draw_background(renderer, &style, bounds);

        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            &inherited_style,
            title_layout,
            cursor_position,
            viewport,
        );

        if let Some(controls) = &self.controls {
            let controls_layout = children.next().unwrap();

            if show_controls || self.always_show_controls {
                controls.as_widget().draw(
                    &tree.children[1],
                    renderer,
                    &inherited_style,
                    controls_layout,
                    cursor_position,
                    viewport,
                );
            }
        }
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
            let mut children = padded.children();
            let title_layout = children.next().unwrap();

            if self.controls.is_some() {
                let controls_layout = children.next().unwrap();

                !controls_layout.bounds().contains(cursor_position)
                    && !title_layout.bounds().contains(cursor_position)
            } else {
                !title_layout.bounds().contains(cursor_position)
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
        let limits = limits.pad(self.padding);
        let max_size = limits.max();

        let title_layout = self
            .content
            .as_widget()
            .layout(renderer, &layout::Limits::new(Size::ZERO, max_size));

        let title_size = title_layout.size();

        let mut node = if let Some(controls) = &self.controls {
            let mut controls_layout = controls
                .as_widget()
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

        node.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        layout::Node::with_children(node.size().pad(self.padding), vec![node])
    }

    pub(crate) fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();

        let control_status = if let Some(controls) = &mut self.controls {
            let controls_layout = children.next().unwrap();

            controls.as_widget_mut().on_event(
                &mut tree.children[1],
                event.clone(),
                controls_layout,
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        } else {
            event::Status::Ignored
        };

        let title_status = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            title_layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        );

        control_status.merge(title_status)
    }

    pub(crate) fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();

        let title_interaction = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            title_layout,
            cursor_position,
            viewport,
            renderer,
        );

        if let Some(controls) = &self.controls {
            let controls_layout = children.next().unwrap();

            controls
                .as_widget()
                .mouse_interaction(
                    &tree.children[1],
                    controls_layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
                .max(title_interaction)
        } else {
            title_interaction
        }
    }

    pub(crate) fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        let mut children = layout.children();
        let padded = children.next()?;

        let mut children = padded.children();
        let title_layout = children.next()?;

        let Self {
            content, controls, ..
        } = self;

        let mut states = tree.children.iter_mut();
        let title_state = states.next().unwrap();
        let controls_state = states.next().unwrap();

        content
            .as_widget()
            .overlay(title_state, title_layout, renderer)
            .or_else(move || {
                controls.as_ref().and_then(|controls| {
                    let controls_layout = children.next()?;

                    controls.as_widget().overlay(
                        controls_state,
                        controls_layout,
                        renderer,
                    )
                })
            })
    }
}
