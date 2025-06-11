use crate::container;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{self, Tree};
use crate::core::{
    self, Clipboard, Element, Event, Layout, Padding, Point, Rectangle, Shell,
    Size, Vector,
};
use crate::pane_grid::controls::Controls;

/// The title bar of a [`Pane`].
///
/// [`Pane`]: super::Pane
#[allow(missing_debug_implementations)]
pub struct TitleBar<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
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
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            controls: None,
            padding: Padding::ZERO,
            always_show_controls: false,
            class: Theme::default(),
        }
    }

    /// Sets the controls of the [`TitleBar`].
    pub fn controls(
        mut self,
        controls: impl Into<Controls<'a, Message, Theme, Renderer>>,
    ) -> Self {
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
    pub fn style(
        mut self,
        style: impl Fn(&Theme) -> container::Style + 'a,
    ) -> Self
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

    pub(super) fn diff(&self, tree: &mut Tree) {
        if tree.children.len() == 3 {
            if let Some(controls) = self.controls.as_ref() {
                if let Some(compact) = controls.compact.as_ref() {
                    tree.children[2].diff(compact);
                }

                tree.children[1].diff(&controls.full);
            }

            tree.children[0].diff(&self.content);
        } else {
            *tree = self.state();
        }
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
        layout: Layout<'_>,
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

        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();
        let mut show_title = true;

        if let Some(controls) = &self.controls {
            if show_controls || self.always_show_controls {
                let controls_layout = children.next().unwrap();
                if title_layout.bounds().width + controls_layout.bounds().width
                    > padded.bounds().width
                {
                    if let Some(compact) = controls.compact.as_ref() {
                        let compact_layout = children.next().unwrap();

                        compact.as_widget().draw(
                            &tree.children[2],
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
                            &tree.children[1],
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
                        &tree.children[1],
                        renderer,
                        theme,
                        &inherited_style,
                        controls_layout,
                        cursor,
                        viewport,
                    );
                }
            }
        }

        if show_title {
            self.content.as_widget().draw(
                &tree.children[0],
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

            if let Some(controls) = self.controls.as_ref() {
                let controls_layout = children.next().unwrap();

                if title_layout.bounds().width + controls_layout.bounds().width
                    > padded.bounds().width
                {
                    if controls.compact.is_some() {
                        let compact_layout = children.next().unwrap();

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

    pub(crate) fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.shrink(self.padding);
        let max_size = limits.max();

        let title_layout = self.content.as_widget().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, max_size),
        );

        let title_size = title_layout.size();

        let node = if let Some(controls) = &self.controls {
            let controls_layout = controls.full.as_widget().layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(Size::ZERO, max_size),
            );

            if title_layout.bounds().width + controls_layout.bounds().width
                > max_size.width
            {
                if let Some(compact) = controls.compact.as_ref() {
                    let compact_layout = compact.as_widget().layout(
                        &mut tree.children[2],
                        renderer,
                        &layout::Limits::new(Size::ZERO, max_size),
                    );

                    let compact_size = compact_layout.size();
                    let space_before_controls =
                        max_size.width - compact_size.width;

                    let height = title_size.height.max(compact_size.height);

                    layout::Node::with_children(
                        Size::new(max_size.width, height),
                        vec![
                            title_layout,
                            controls_layout,
                            compact_layout.move_to(Point::new(
                                space_before_controls,
                                0.0,
                            )),
                        ],
                    )
                } else {
                    let controls_size = controls_layout.size();
                    let space_before_controls =
                        max_size.width - controls_size.width;

                    let height = title_size.height.max(controls_size.height);

                    layout::Node::with_children(
                        Size::new(max_size.width, height),
                        vec![
                            title_layout,
                            controls_layout.move_to(Point::new(
                                space_before_controls,
                                0.0,
                            )),
                        ],
                    )
                }
            } else {
                let controls_size = controls_layout.size();
                let space_before_controls =
                    max_size.width - controls_size.width;

                let height = title_size.height.max(controls_size.height);

                layout::Node::with_children(
                    Size::new(max_size.width, height),
                    vec![
                        title_layout,
                        controls_layout
                            .move_to(Point::new(space_before_controls, 0.0)),
                    ],
                )
            }
        } else {
            layout::Node::with_children(
                Size::new(max_size.width, title_size.height),
                vec![title_layout],
            )
        };

        layout::Node::container(node, self.padding)
    }

    pub(crate) fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();
        let mut show_title = true;

        if let Some(controls) = &self.controls {
            let controls_layout = children.next().unwrap();

            if title_layout.bounds().width + controls_layout.bounds().width
                > padded.bounds().width
            {
                if let Some(compact) = controls.compact.as_ref() {
                    let compact_layout = children.next().unwrap();

                    compact.as_widget().operate(
                        &mut tree.children[2],
                        compact_layout,
                        renderer,
                        operation,
                    );
                } else {
                    show_title = false;

                    controls.full.as_widget().operate(
                        &mut tree.children[1],
                        controls_layout,
                        renderer,
                        operation,
                    );
                }
            } else {
                controls.full.as_widget().operate(
                    &mut tree.children[1],
                    controls_layout,
                    renderer,
                    operation,
                );
            }
        };

        if show_title {
            self.content.as_widget().operate(
                &mut tree.children[0],
                title_layout,
                renderer,
                operation,
            );
        }
    }

    pub(crate) fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        let padded = children.next().unwrap();

        let mut children = padded.children();
        let title_layout = children.next().unwrap();
        let mut show_title = true;

        if let Some(controls) = &mut self.controls {
            let controls_layout = children.next().unwrap();

            if title_layout.bounds().width + controls_layout.bounds().width
                > padded.bounds().width
            {
                if let Some(compact) = controls.compact.as_mut() {
                    let compact_layout = children.next().unwrap();

                    compact.as_widget_mut().update(
                        &mut tree.children[2],
                        event,
                        compact_layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                } else {
                    show_title = false;

                    controls.full.as_widget_mut().update(
                        &mut tree.children[1],
                        event,
                        controls_layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
            } else {
                controls.full.as_widget_mut().update(
                    &mut tree.children[1],
                    event,
                    controls_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }

        if show_title {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                title_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    pub(crate) fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
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
            cursor,
            viewport,
            renderer,
        );

        if let Some(controls) = &self.controls {
            let controls_layout = children.next().unwrap();
            let controls_interaction =
                controls.full.as_widget().mouse_interaction(
                    &tree.children[1],
                    controls_layout,
                    cursor,
                    viewport,
                    renderer,
                );

            if title_layout.bounds().width + controls_layout.bounds().width
                > padded.bounds().width
            {
                if let Some(compact) = controls.compact.as_ref() {
                    let compact_layout = children.next().unwrap();
                    let compact_interaction =
                        compact.as_widget().mouse_interaction(
                            &tree.children[2],
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
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
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
            .as_widget_mut()
            .overlay(title_state, title_layout, renderer, viewport, translation)
            .or_else(move || {
                controls.as_mut().and_then(|controls| {
                    let controls_layout = children.next()?;

                    if title_layout.bounds().width
                        + controls_layout.bounds().width
                        > padded.bounds().width
                    {
                        if let Some(compact) = controls.compact.as_mut() {
                            let compact_state = states.next().unwrap();
                            let compact_layout = children.next()?;

                            compact.as_widget_mut().overlay(
                                compact_state,
                                compact_layout,
                                renderer,
                                viewport,
                                translation,
                            )
                        } else {
                            controls.full.as_widget_mut().overlay(
                                controls_state,
                                controls_layout,
                                renderer,
                                viewport,
                                translation,
                            )
                        }
                    } else {
                        controls.full.as_widget_mut().overlay(
                            controls_state,
                            controls_layout,
                            renderer,
                            viewport,
                            translation,
                        )
                    }
                })
            })
    }
}
