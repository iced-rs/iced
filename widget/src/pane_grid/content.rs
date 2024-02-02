use crate::container;
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{self, Tree};
use crate::core::{
    Clipboard, Element, Layout, Point, Rectangle, Shell, Size, Vector,
};
use crate::pane_grid::{Draggable, TitleBar};

/// The content of a [`Pane`].
///
/// [`Pane`]: super::Pane
#[allow(missing_debug_implementations)]
pub struct Content<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Theme: container::StyleSheet,
    Renderer: crate::core::Renderer,
{
    title_bar: Option<TitleBar<'a, Message, Theme, Renderer>>,
    body: Element<'a, Message, Theme, Renderer>,
    style: Theme::Style,
}

impl<'a, Message, Theme, Renderer> Content<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet,
    Renderer: crate::core::Renderer,
{
    /// Creates a new [`Content`] with the provided body.
    pub fn new(body: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            title_bar: None,
            body: body.into(),
            style: Default::default(),
        }
    }

    /// Sets the [`TitleBar`] of this [`Content`].
    pub fn title_bar(
        mut self,
        title_bar: TitleBar<'a, Message, Theme, Renderer>,
    ) -> Self {
        self.title_bar = Some(title_bar);
        self
    }

    /// Sets the style of the [`Content`].
    pub fn style(mut self, style: impl Into<Theme::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Content<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet,
    Renderer: crate::core::Renderer,
{
    pub(super) fn state(&self) -> Tree {
        let children = if let Some(title_bar) = self.title_bar.as_ref() {
            vec![Tree::new(&self.body), title_bar.state()]
        } else {
            vec![Tree::new(&self.body), Tree::empty()]
        };

        Tree {
            children,
            ..Tree::empty()
        }
    }

    pub(super) fn diff(&self, tree: &mut Tree) {
        if tree.children.len() == 2 {
            if let Some(title_bar) = self.title_bar.as_ref() {
                title_bar.diff(&mut tree.children[1]);
            }

            tree.children[0].diff(&self.body);
        } else {
            *tree = self.state();
        }
    }

    /// Draws the [`Content`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: crate::core::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        {
            let style = theme.appearance(&self.style);

            container::draw_background(renderer, &style, bounds);
        }

        if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();
            let body_layout = children.next().unwrap();

            let show_controls = cursor.is_over(bounds);

            self.body.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                body_layout,
                cursor,
                viewport,
            );

            title_bar.draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                title_bar_layout,
                cursor,
                viewport,
                show_controls,
            );
        } else {
            self.body.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
    }

    pub(crate) fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if let Some(title_bar) = &self.title_bar {
            let max_size = limits.max();

            let title_bar_layout = title_bar.layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(Size::ZERO, max_size),
            );

            let title_bar_size = title_bar_layout.size();

            let body_layout = self.body.as_widget().layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    Size::new(
                        max_size.width,
                        max_size.height - title_bar_size.height,
                    ),
                ),
            );

            layout::Node::with_children(
                max_size,
                vec![
                    title_bar_layout,
                    body_layout.move_to(Point::new(0.0, title_bar_size.height)),
                ],
            )
        } else {
            self.body.as_widget().layout(
                &mut tree.children[0],
                renderer,
                limits,
            )
        }
    }

    pub(crate) fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        let body_layout = if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children();

            title_bar.operate(
                &mut tree.children[1],
                children.next().unwrap(),
                renderer,
                operation,
            );

            children.next().unwrap()
        } else {
            layout
        };

        self.body.as_widget().operate(
            &mut tree.children[0],
            body_layout,
            renderer,
            operation,
        );
    }

    pub(crate) fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
        is_picked: bool,
    ) -> event::Status {
        let mut event_status = event::Status::Ignored;

        let body_layout = if let Some(title_bar) = &mut self.title_bar {
            let mut children = layout.children();

            event_status = title_bar.on_event(
                &mut tree.children[1],
                event.clone(),
                children.next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            children.next().unwrap()
        } else {
            layout
        };

        let body_status = if is_picked {
            event::Status::Ignored
        } else {
            self.body.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                body_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        };

        event_status.merge(body_status)
    }

    pub(crate) fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
        drag_enabled: bool,
    ) -> mouse::Interaction {
        let (body_layout, title_bar_interaction) = if let Some(title_bar) =
            &self.title_bar
        {
            let mut children = layout.children();
            let title_bar_layout = children.next().unwrap();

            let is_over_pick_area = cursor
                .position()
                .map(|cursor_position| {
                    title_bar
                        .is_over_pick_area(title_bar_layout, cursor_position)
                })
                .unwrap_or_default();

            if is_over_pick_area && drag_enabled {
                return mouse::Interaction::Grab;
            }

            let mouse_interaction = title_bar.mouse_interaction(
                &tree.children[1],
                title_bar_layout,
                cursor,
                viewport,
                renderer,
            );

            (children.next().unwrap(), mouse_interaction)
        } else {
            (layout, mouse::Interaction::default())
        };

        self.body
            .as_widget()
            .mouse_interaction(
                &tree.children[0],
                body_layout,
                cursor,
                viewport,
                renderer,
            )
            .max(title_bar_interaction)
    }

    pub(crate) fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if let Some(title_bar) = self.title_bar.as_mut() {
            let mut children = layout.children();
            let title_bar_layout = children.next()?;

            let mut states = tree.children.iter_mut();
            let body_state = states.next().unwrap();
            let title_bar_state = states.next().unwrap();

            match title_bar.overlay(
                title_bar_state,
                title_bar_layout,
                renderer,
                translation,
            ) {
                Some(overlay) => Some(overlay),
                None => self.body.as_widget_mut().overlay(
                    body_state,
                    children.next()?,
                    renderer,
                    translation,
                ),
            }
        } else {
            self.body.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                translation,
            )
        }
    }
}

impl<'a, Message, Theme, Renderer> Draggable
    for &Content<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet,
    Renderer: crate::core::Renderer,
{
    fn can_be_dragged_at(
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
}

impl<'a, T, Message, Theme, Renderer> From<T>
    for Content<'a, Message, Theme, Renderer>
where
    T: Into<Element<'a, Message, Theme, Renderer>>,
    Theme: container::StyleSheet,
    Renderer: crate::core::Renderer,
{
    fn from(element: T) -> Self {
        Self::new(element)
    }
}
