use crate::container;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{self, Tree};
use crate::core::{self, Element, Event, Layout, Point, Rectangle, Shell, Size, Vector};
use crate::pane_grid::{Draggable, TitleBar};

/// The content of a [`Pane`].
///
/// [`Pane`]: super::Pane
pub struct Content<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    title_bar: Option<TitleBar<'a, Message, Theme, Renderer>>,
    body: Element<'a, Message, Theme, Renderer>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Content<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    /// Creates a new [`Content`] with the provided body.
    pub fn new(body: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            title_bar: None,
            body: body.into(),
            class: Theme::default(),
        }
    }

    /// Sets the [`TitleBar`] of the [`Content`].
    pub fn title_bar(mut self, title_bar: TitleBar<'a, Message, Theme, Renderer>) -> Self {
        self.title_bar = Some(title_bar);
        self
    }

    /// Sets the style of the [`Content`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<container::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as container::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Content`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> Content<'_, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
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

    pub(super) fn diff(&mut self, tree: &mut Tree) {
        if tree.children.len() == 2 {
            if let Some(title_bar) = &mut self.title_bar {
                title_bar.diff(&mut tree.children[1]);
            }

            tree.children[0].diff(&mut self.body);
        } else {
            *tree = self.state();
        }
    }

    /// Draws the [`Content`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: core::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        {
            let style = theme.style(&self.class);

            container::draw_background(renderer, &style, bounds);
        }

        if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children(tree);
            let (body_layout, body_tree) = children.next().unwrap();
            let (title_bar_layout, title_bar_tree) = children.next().unwrap();

            let show_controls = cursor.is_over(bounds);

            self.body.as_widget().draw(
                body_tree,
                renderer,
                theme,
                style,
                body_layout,
                cursor,
                viewport,
            );

            title_bar.draw(
                title_bar_tree,
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

    pub(crate) fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        if let Some(title_bar) = &mut self.title_bar {
            let max_size = limits.max;

            title_bar.layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(Size::ZERO, max_size),
            );

            let title_bar_size = tree.children[1].size;

            self.body.as_widget_mut().layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    Size::new(max_size.width, max_size.height - title_bar_size.height),
                ),
            );

            tree.children[1].translation = Vector::ZERO;
            tree.children[0].translation = Vector::new(0.0, title_bar_size.height);

            tree.size = max_size;
        } else {
            self.body
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits);

            tree.size = tree.children[0].size;
        }
    }

    pub(crate) fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let (body_layout, body_tree) = if let Some(title_bar) = &mut self.title_bar {
            let mut children = layout.children_mut(tree);
            let (body_layout, body_tree) = children.next().unwrap();
            let (title_bar_layout, title_bar_tree) = children.next().unwrap();

            title_bar.operate(
                title_bar_tree,
                title_bar_layout,
                viewport,
                renderer,
                operation,
            );

            (body_layout, body_tree)
        } else {
            layout.children_mut(tree).next().unwrap()
        };

        self.body
            .as_widget_mut()
            .operate(body_tree, body_layout, viewport, renderer, operation);
    }

    pub(crate) fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
        is_picked: bool,
    ) {
        let (body_layout, body_tree) = if let Some(title_bar) = &mut self.title_bar {
            let mut children = layout.children_mut(tree);
            let (body_layout, body_tree) = children.next().unwrap();
            let (title_bar_layout, title_bar_tree) = children.next().unwrap();

            title_bar.update(
                title_bar_tree,
                event,
                title_bar_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );

            (body_layout, body_tree)
        } else {
            layout.children_mut(tree).next().unwrap()
        };

        if !is_picked {
            self.body.as_widget_mut().update(
                body_tree,
                event,
                body_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }
    }

    pub(crate) fn grid_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        drag_enabled: bool,
    ) -> Option<mouse::Interaction> {
        let title_bar = self.title_bar.as_ref()?;

        let title_bar_layout = layout.children(tree).nth(1).unwrap().0;

        let is_over_pick_area = cursor
            .position()
            .map(|cursor_position| {
                title_bar.is_over_pick_area(&tree.children[1], title_bar_layout, cursor_position)
            })
            .unwrap_or_default();

        if is_over_pick_area && drag_enabled {
            return Some(mouse::Interaction::Grab);
        }

        None
    }

    pub(crate) fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
        drag_enabled: bool,
    ) -> mouse::Interaction {
        let (body_layout, title_bar_interaction) = if let Some(title_bar) = &self.title_bar {
            let mut children = layout.children(tree);
            let (body_layout, _) = children.next().unwrap();
            let (title_bar_layout, title_bar_tree) = children.next().unwrap();

            let is_over_pick_area = cursor
                .position()
                .map(|cursor_position| {
                    title_bar.is_over_pick_area(title_bar_tree, title_bar_layout, cursor_position)
                })
                .unwrap_or_default();

            if is_over_pick_area && drag_enabled {
                return mouse::Interaction::Grab;
            }

            let mouse_interaction = title_bar.mouse_interaction(
                title_bar_tree,
                title_bar_layout,
                cursor,
                viewport,
                renderer,
            );

            (body_layout, mouse_interaction)
        } else {
            (layout, mouse::Interaction::default())
        };

        self.body
            .as_widget()
            .mouse_interaction(&tree.children[0], body_layout, cursor, viewport, renderer)
            .max(title_bar_interaction)
    }

    pub(crate) fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        if let Some(title_bar) = self.title_bar.as_mut() {
            let mut children = layout.children_mut(tree);
            let (body_layout, body_tree) = children.next().unwrap();
            let (title_bar_layout, title_bar_tree) = children.next().unwrap();

            let title_bar_overlays = title_bar.overlay(
                title_bar_tree,
                title_bar_layout,
                renderer,
                viewport,
                translation,
                window,
            );

            let body_overlays = self.body.as_widget_mut().overlay(
                body_tree,
                body_layout,
                renderer,
                viewport,
                translation,
                window,
            );

            title_bar_overlays
                .into_iter()
                .chain(body_overlays)
                .collect()
        } else {
            let (layout, body_tree) = layout.children_mut(tree).next().unwrap();

            self.body.as_widget_mut().overlay(
                body_tree,
                layout,
                renderer,
                viewport,
                translation,
                window,
            )
        }
    }
}

impl<Message, Theme, Renderer> Draggable for &Content<'_, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    fn can_be_dragged_at(&self, tree: &Tree, layout: Layout, cursor_position: Point) -> bool {
        if let Some(title_bar) = &self.title_bar {
            let title_bar_layout = layout.children(tree).nth(1).unwrap().0;

            title_bar.is_over_pick_area(&tree.children[1], title_bar_layout, cursor_position)
        } else {
            false
        }
    }
}

impl<'a, T, Message, Theme, Renderer> From<T> for Content<'a, Message, Theme, Renderer>
where
    T: Into<Element<'a, Message, Theme, Renderer>>,
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    fn from(element: T) -> Self {
        Self::new(element)
    }
}
