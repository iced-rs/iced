use crate::container;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{self, Tree};
use crate::core::{self, Element, Event, Layout, Padding, Point, Rectangle, Shell, Size, Vector};
use crate::pane_grid::controls::Controls;

/// The title bar of a [`Pane`].
///
/// [`Pane`]: super::Pane
pub struct TitleBar<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    controls: Option<Controls<'a, Message, Theme, Renderer>>,
    padding: Padding,
    always_show_controls: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> TitleBar<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    /// Creates a new [`TitleBar`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            controls: None,
            padding: Padding::ZERO,
            always_show_controls: false,
            class: Theme::default(),
        }
    }

    /// Sets the controls of the [`TitleBar`].
    pub fn controls(mut self, controls: impl Into<Controls<'a, Message, Theme, Renderer>>) -> Self {
        self.controls = Some(controls.into());
        self
    }

    /// Sets the [`Padding`] of the [`TitleBar`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets whether or not the [`controls`] attached to this [`TitleBar`] are
    /// always visible.
    ///
    /// By default, the controls are only visible when the [`Pane`] of this
    /// [`TitleBar`] is hovered.
    ///
    /// [`controls`]: Self::controls
    /// [`Pane`]: super::Pane
    pub fn always_show_controls(mut self) -> Self {
        self.always_show_controls = true;
        self
    }

    /// Sets the style of the [`TitleBar`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<container::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as container::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`TitleBar`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> TitleBar<'_, Message, Theme, Renderer>
where
    Theme: container::Catalog,
    Renderer: core::Renderer,
{
    pub(super) fn state(&self) -> Tree {
        let children = match self.controls.as_ref() {
            Some(controls) => match controls.compact.as_ref() {
                Some(compact) => vec![
                    Tree::new(&self.content),
                    Tree::new(&controls.full),
                    Tree::new(compact),
                ],
                None => vec![
                    Tree::new(&self.content),
                    Tree::new(&controls.full),
                    Tree::empty(),
                ],
            },
            None => {
                vec![Tree::new(&self.content), Tree::empty(), Tree::empty()]
            }
        };

        Tree {
            children,
            ..Tree::empty()
        }
    }

    pub(super) fn diff(&mut self, tree: &mut Tree) {
        if tree.children.len() != 3 {
            *tree = self.state();
        }

        if let Some(controls) = &mut self.controls {
            if let Some(compact) = &mut controls.compact {
                tree.children[2].diff(compact);
            }

            tree.children[1].diff(&mut controls.full);
        }

        tree.children[0].diff(&mut self.content);
    }

    /// Draws the [`TitleBar`] with the provided [`Renderer`] and [`Layout`].
    ///
    /// [`Renderer`]: core::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        show_controls: bool,
    ) {
        let bounds = layout.bounds();
        let style = theme.style(&self.class);

        let inherited_style = renderer::Style {
            text_color: style.text_color.unwrap_or(inherited_style.text_color),
        };

        container::draw_background(renderer, &style, bounds);

        let mut children = layout.children(tree);
        let (title_layout, title_tree) = children.next().unwrap();
        let (controls_layout, controls_tree) = children.next().unwrap();
        let (compact_layout, compact_tree) = children.next().unwrap();
        let mut show_title = true;

        let padded_width = layout.bounds().width - self.padding.left + self.padding.right;

        if let Some(controls) = &self.controls
            && (show_controls || self.always_show_controls)
        {
            if title_layout.bounds().width + controls_layout.bounds().width > padded_width {
                if let Some(compact) = controls.compact.as_ref() {
                    compact.as_widget().draw(
                        compact_tree,
                        renderer,
                        theme,
                        &inherited_style,
                        compact_layout,
                        cursor,
                        viewport,
                    );
                } else {
                    show_title = false;

                    controls.full.as_widget().draw(
                        controls_tree,
                        renderer,
                        theme,
                        &inherited_style,
                        controls_layout,
                        cursor,
                        viewport,
                    );
                }
            } else {
                controls.full.as_widget().draw(
                    controls_tree,
                    renderer,
                    theme,
                    &inherited_style,
                    controls_layout,
                    cursor,
                    viewport,
                );
            }
        }

        if show_title {
            self.content.as_widget().draw(
                title_tree,
                renderer,
                theme,
                &inherited_style,
                title_layout,
                cursor,
                viewport,
            );
        }
    }

    /// Returns whether the mouse cursor is over the pick area of the
    /// [`TitleBar`] or not.
    ///
    /// The whole [`TitleBar`] is a pick area, except its controls.
    pub fn is_over_pick_area(&self, tree: &Tree, layout: Layout, cursor_position: Point) -> bool {
        if layout.bounds().contains(cursor_position) {
            let mut children = layout.children(tree);
            let (title_layout, _) = children.next().unwrap();
            let (controls_layout, _) = children.next().unwrap();
            let (compact_layout, _) = children.next().unwrap();

            let padded_width = layout.bounds().width - self.padding.left + self.padding.right;

            if let Some(controls) = self.controls.as_ref() {
                if title_layout.bounds().width + controls_layout.bounds().width > padded_width {
                    if controls.compact.is_some() {
                        !compact_layout.bounds().contains(cursor_position)
                            && !title_layout.bounds().contains(cursor_position)
                    } else {
                        !controls_layout.bounds().contains(cursor_position)
                    }
                } else {
                    !controls_layout.bounds().contains(cursor_position)
                        && !title_layout.bounds().contains(cursor_position)
                }
            } else {
                !title_layout.bounds().contains(cursor_position)
            }
        } else {
            false
        }
    }

    pub(crate) fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        let limits = limits.shrink(self.padding);
        let max_size = limits.max;

        self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, max_size),
        );

        let title_size = tree.children[0].size;

        let inner_size = if let Some(controls) = &mut self.controls {
            controls.full.as_widget_mut().layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(Size::ZERO, max_size),
            );

            let controls_size = tree.children[1].size;

            if title_size.width + controls_size.width > max_size.width {
                if let Some(compact) = controls.compact.as_mut() {
                    compact.as_widget_mut().layout(
                        &mut tree.children[2],
                        renderer,
                        &layout::Limits::new(Size::ZERO, max_size),
                    );

                    let compact_size = tree.children[2].size;
                    let space_before_controls = max_size.width - compact_size.width;

                    tree.children[1].translation = Vector::ZERO;
                    tree.children[2].translation = Vector::new(space_before_controls, 0.0);

                    Size::new(max_size.width, title_size.height.max(compact_size.height))
                } else {
                    let space_before_controls = max_size.width - controls_size.width;

                    tree.children[1].translation = Vector::new(space_before_controls, 0.0);

                    Size::new(max_size.width, title_size.height.max(controls_size.height))
                }
            } else {
                let space_before_controls = max_size.width - controls_size.width;

                tree.children[1].translation = Vector::new(space_before_controls, 0.0);

                Size::new(max_size.width, title_size.height.max(controls_size.height))
            }
        } else {
            Size::new(max_size.width, title_size.height)
        };

        tree.children[0].translation = Vector::ZERO;

        for child in &mut tree.children {
            child.translation += Vector::new(self.padding.left, self.padding.top);
        }

        tree.size = inner_size.expand(self.padding);
    }

    pub(crate) fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let mut children = layout.children_mut(tree);
        let (title_layout, title_tree) = children.next().unwrap();
        let (controls_layout, controls_tree) = children.next().unwrap();
        let (compact_layout, compact_tree) = children.next().unwrap();
        let padded_width = layout.bounds().width - self.padding.left + self.padding.right;
        let mut show_title = true;

        if let Some(controls) = &mut self.controls {
            if title_layout.bounds().width + controls_layout.bounds().width > padded_width {
                if let Some(compact) = controls.compact.as_mut() {
                    compact.as_widget_mut().operate(
                        compact_tree,
                        compact_layout,
                        viewport,
                        renderer,
                        operation,
                    );
                } else {
                    show_title = false;

                    controls.full.as_widget_mut().operate(
                        controls_tree,
                        controls_layout,
                        viewport,
                        renderer,
                        operation,
                    );
                }
            } else {
                controls.full.as_widget_mut().operate(
                    controls_tree,
                    controls_layout,
                    viewport,
                    renderer,
                    operation,
                );
            }
        };

        if show_title {
            self.content.as_widget_mut().operate(
                title_tree,
                title_layout,
                viewport,
                renderer,
                operation,
            );
        }
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
    ) {
        let mut children = layout.children_mut(tree);
        let (title_layout, title_tree) = children.next().unwrap();
        let (controls_layout, controls_tree) = children.next().unwrap();
        let (compact_layout, compact_tree) = children.next().unwrap();
        let padded_width = layout.bounds().width - self.padding.left + self.padding.right;
        let mut show_title = true;

        if let Some(controls) = &mut self.controls {
            if title_layout.bounds().width + controls_layout.bounds().width > padded_width {
                if let Some(compact) = controls.compact.as_mut() {
                    compact.as_widget_mut().update(
                        compact_tree,
                        event,
                        compact_layout,
                        cursor,
                        renderer,
                        shell,
                        viewport,
                    );
                } else {
                    show_title = false;

                    controls.full.as_widget_mut().update(
                        controls_tree,
                        event,
                        controls_layout,
                        cursor,
                        renderer,
                        shell,
                        viewport,
                    );
                }
            } else {
                controls.full.as_widget_mut().update(
                    controls_tree,
                    event,
                    controls_layout,
                    cursor,
                    renderer,
                    shell,
                    viewport,
                );
            }
        }

        if show_title {
            self.content.as_widget_mut().update(
                title_tree,
                event,
                title_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }
    }

    pub(crate) fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children(tree);
        let (title_layout, title_tree) = children.next().unwrap();
        let (controls_layout, controls_tree) = children.next().unwrap();
        let (compact_layout, compact_tree) = children.next().unwrap();
        let padded_width = layout.bounds().width - self.padding.left + self.padding.right;

        let title_interaction = self.content.as_widget().mouse_interaction(
            title_tree,
            title_layout,
            cursor,
            viewport,
            renderer,
        );

        if let Some(controls) = &self.controls {
            let controls_interaction = controls.full.as_widget().mouse_interaction(
                controls_tree,
                controls_layout,
                cursor,
                viewport,
                renderer,
            );

            if title_layout.bounds().width + controls_layout.bounds().width > padded_width {
                if let Some(compact) = controls.compact.as_ref() {
                    let compact_interaction = compact.as_widget().mouse_interaction(
                        compact_tree,
                        compact_layout,
                        cursor,
                        viewport,
                        renderer,
                    );

                    compact_interaction.max(title_interaction)
                } else {
                    controls_interaction
                }
            } else {
                controls_interaction.max(title_interaction)
            }
        } else {
            title_interaction
        }
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
        let mut children = layout.children_mut(tree);
        let (title_layout, title_tree) = children.next().unwrap();
        let (controls_layout, controls_tree) = children.next().unwrap();
        let (compact_layout, compact_tree) = children.next().unwrap();
        let padded_width = layout.bounds().width - self.padding.left + self.padding.right;

        let Self {
            content, controls, ..
        } = self;

        let mut overlays = content.as_widget_mut().overlay(
            title_tree,
            title_layout,
            renderer,
            viewport,
            translation,
            window,
        );

        if let Some(controls) = controls {
            if title_layout.bounds().width + controls_layout.bounds().width > padded_width {
                if let Some(compact) = &mut controls.compact {
                    overlays.extend(compact.as_widget_mut().overlay(
                        compact_tree,
                        compact_layout,
                        renderer,
                        viewport,
                        translation,
                        window,
                    ));
                } else {
                    overlays.extend(controls.full.as_widget_mut().overlay(
                        controls_tree,
                        controls_layout,
                        renderer,
                        viewport,
                        translation,
                        window,
                    ));
                }
            } else {
                overlays.extend(controls.full.as_widget_mut().overlay(
                    controls_tree,
                    controls_layout,
                    renderer,
                    viewport,
                    translation,
                    window,
                ));
            }
        }

        overlays
    }
}
