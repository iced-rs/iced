//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
//!
//! # Example
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/iced-rs/iced/tree/0.10/examples/pane_grid
mod axis;
mod configuration;
mod content;
mod direction;
mod draggable;
mod node;
mod pane;
mod split;
mod title_bar;

pub mod state;

pub use axis::Axis;
pub use configuration::Configuration;
pub use content::Content;
pub use direction::Direction;
pub use draggable::Draggable;
pub use node::Node;
pub use pane::Pane;
pub use split::Split;
pub use state::State;
pub use title_bar::TitleBar;

pub use crate::style::pane_grid::{Appearance, Line, StyleSheet};

use crate::container;
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay::{self, Group};
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Clipboard, Color, Element, Layout, Length, Pixels, Point, Rectangle, Shell,
    Size, Vector, Widget,
};

/// A collection of panes distributed using either vertical or horizontal splits
/// to completely fill the space available.
///
/// [![Pane grid - Iced](https://thumbs.gfycat.com/FrailFreshAiredaleterrier-small.gif)](https://gfycat.com/frailfreshairedaleterrier)
///
/// This distribution of space is common in tiling window managers (like
/// [`awesome`](https://awesomewm.org/), [`i3`](https://i3wm.org/), or even
/// [`tmux`](https://github.com/tmux/tmux)).
///
/// A [`PaneGrid`] supports:
///
/// * Vertical and horizontal splits
/// * Tracking of the last active pane
/// * Mouse-based resizing
/// * Drag and drop to reorganize panes
/// * Hotkey support
/// * Configurable modifier keys
/// * [`State`] API to perform actions programmatically (`split`, `swap`, `resize`, etc.)
///
/// ## Example
///
/// ```no_run
/// # use iced_widget::{pane_grid, text};
/// #
/// # type PaneGrid<'a, Message> =
/// #     iced_widget::PaneGrid<'a, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
/// #
/// enum PaneState {
///     SomePane,
///     AnotherKindOfPane,
/// }
///
/// enum Message {
///     PaneDragged(pane_grid::DragEvent),
///     PaneResized(pane_grid::ResizeEvent),
/// }
///
/// let (mut state, _) = pane_grid::State::new(PaneState::SomePane);
///
/// let pane_grid =
///     PaneGrid::new(&state, |pane, state, is_maximized| {
///         pane_grid::Content::new(match state {
///             PaneState::SomePane => text("This is some pane"),
///             PaneState::AnotherKindOfPane => text("This is another kind of pane"),
///         })
///     })
///     .on_drag(Message::PaneDragged)
///     .on_resize(10, Message::PaneResized);
/// ```
#[allow(missing_debug_implementations)]
pub struct PaneGrid<'a, Message, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet + container::StyleSheet,
{
    contents: Contents<'a, Content<'a, Message, Renderer>>,
    width: Length,
    height: Length,
    spacing: f32,
    on_click: Option<Box<dyn Fn(Pane) -> Message + 'a>>,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    on_resize: Option<(f32, Box<dyn Fn(ResizeEvent) -> Message + 'a>)>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> PaneGrid<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet + container::StyleSheet,
{
    /// Creates a [`PaneGrid`] with the given [`State`] and view function.
    ///
    /// The view function will be called to display each [`Pane`] present in the
    /// [`State`]. [`bool`] is set if the pane is maximized.
    pub fn new<T>(
        state: &'a State<T>,
        view: impl Fn(Pane, &'a T, bool) -> Content<'a, Message, Renderer>,
    ) -> Self {
        let contents = if let Some((pane, pane_state)) =
            state.maximized.and_then(|pane| {
                state.panes.get(&pane).map(|pane_state| (pane, pane_state))
            }) {
            Contents::Maximized(
                pane,
                view(pane, pane_state, true),
                Node::Pane(pane),
            )
        } else {
            Contents::All(
                state
                    .panes
                    .iter()
                    .map(|(pane, pane_state)| {
                        (*pane, view(*pane, pane_state, false))
                    })
                    .collect(),
                &state.internal,
            )
        };

        Self {
            contents,
            width: Length::Fill,
            height: Length::Fill,
            spacing: 0.0,
            on_click: None,
            on_drag: None,
            on_resize: None,
            style: Default::default(),
        }
    }

    /// Sets the width of the [`PaneGrid`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`PaneGrid`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the spacing _between_ the panes of the [`PaneGrid`].
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the message that will be produced when a [`Pane`] of the
    /// [`PaneGrid`] is clicked.
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(Pane) -> Message,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    /// Enables the drag and drop interactions of the [`PaneGrid`], which will
    /// use the provided function to produce messages.
    pub fn on_drag<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(DragEvent) -> Message,
    {
        self.on_drag = Some(Box::new(f));
        self
    }

    /// Enables the resize interactions of the [`PaneGrid`], which will
    /// use the provided function to produce messages.
    ///
    /// The `leeway` describes the amount of space around a split that can be
    /// used to grab it.
    ///
    /// The grabbable area of a split will have a length of `spacing + leeway`,
    /// properly centered. In other words, a length of
    /// `(spacing + leeway) / 2.0` on either side of the split line.
    pub fn on_resize<F>(mut self, leeway: impl Into<Pixels>, f: F) -> Self
    where
        F: 'a + Fn(ResizeEvent) -> Message,
    {
        self.on_resize = Some((leeway.into().0, Box::new(f)));
        self
    }

    /// Sets the style of the [`PaneGrid`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    fn drag_enabled(&self) -> bool {
        (!self.contents.is_maximized())
            .then(|| self.on_drag.is_some())
            .unwrap_or_default()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for PaneGrid<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet + container::StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<state::Action>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(state::Action::Idle)
    }

    fn children(&self) -> Vec<Tree> {
        self.contents
            .iter()
            .map(|(_, content)| content.state())
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        match &self.contents {
            Contents::All(contents, _) => tree.diff_children_custom(
                contents,
                |state, (_, content)| content.diff(state),
                |(_, content)| content.state(),
            ),
            Contents::Maximized(_, content, _) => tree.diff_children_custom(
                &[content],
                |state, content| content.diff(state),
                |content| content.state(),
            ),
        }
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            self.contents.layout(),
            self.width,
            self.height,
            self.spacing,
            self.contents.iter(),
            |content, renderer, limits| content.layout(renderer, limits),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.contents
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|(((_pane, content), state), layout)| {
                    content.operate(state, layout, renderer, operation);
                })
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let action = tree.state.downcast_mut::<state::Action>();

        let on_drag = if self.drag_enabled() {
            &self.on_drag
        } else {
            &None
        };

        let event_status = update(
            action,
            self.contents.layout(),
            &event,
            layout,
            cursor,
            shell,
            self.spacing,
            self.contents.iter(),
            &self.on_click,
            on_drag,
            &self.on_resize,
        );

        let picked_pane = action.picked_pane().map(|(pane, _)| pane);

        self.contents
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|(((pane, content), tree), layout)| {
                let is_picked = picked_pane == Some(pane);

                content.on_event(
                    tree,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                    is_picked,
                )
            })
            .fold(event_status, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(
            tree.state.downcast_ref(),
            self.contents.layout(),
            layout,
            cursor,
            self.spacing,
            self.on_resize.as_ref().map(|(leeway, _)| *leeway),
        )
        .unwrap_or_else(|| {
            self.contents
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .map(|(((_pane, content), tree), layout)| {
                    content.mouse_interaction(
                        tree,
                        layout,
                        cursor,
                        viewport,
                        renderer,
                        self.drag_enabled(),
                    )
                })
                .max()
                .unwrap_or_default()
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        draw(
            tree.state.downcast_ref(),
            self.contents.layout(),
            layout,
            cursor,
            renderer,
            theme,
            style,
            viewport,
            self.spacing,
            self.on_resize.as_ref().map(|(leeway, _)| *leeway),
            &self.style,
            self.contents
                .iter()
                .zip(&tree.children)
                .map(|((pane, content), tree)| (pane, (content, tree))),
            |(content, tree), renderer, style, layout, cursor, rectangle| {
                content.draw(
                    tree, renderer, theme, style, layout, cursor, rectangle,
                );
            },
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let children = self
            .contents
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|(((_, content), state), layout)| {
                content.overlay(state, layout, renderer)
            })
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Renderer> From<PaneGrid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + crate::core::Renderer,
    Renderer::Theme: StyleSheet + container::StyleSheet,
{
    fn from(
        pane_grid: PaneGrid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(pane_grid)
    }
}

/// Calculates the [`Layout`] of a [`PaneGrid`].
pub fn layout<Renderer, T>(
    renderer: &Renderer,
    limits: &layout::Limits,
    node: &Node,
    width: Length,
    height: Length,
    spacing: f32,
    contents: impl Iterator<Item = (Pane, T)>,
    layout_content: impl Fn(T, &Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.width(width).height(height);
    let size = limits.resolve(Size::ZERO);

    let regions = node.pane_regions(spacing, size);
    let children = contents
        .filter_map(|(pane, content)| {
            let region = regions.get(&pane)?;
            let size = Size::new(region.width, region.height);

            let mut node = layout_content(
                content,
                renderer,
                &layout::Limits::new(size, size),
            );

            node.move_to(Point::new(region.x, region.y));

            Some(node)
        })
        .collect();

    layout::Node::with_children(size, children)
}

/// Processes an [`Event`] and updates the [`state`] of a [`PaneGrid`]
/// accordingly.
pub fn update<'a, Message, T: Draggable>(
    action: &mut state::Action,
    node: &Node,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    spacing: f32,
    contents: impl Iterator<Item = (Pane, T)>,
    on_click: &Option<Box<dyn Fn(Pane) -> Message + 'a>>,
    on_drag: &Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    on_resize: &Option<(f32, Box<dyn Fn(ResizeEvent) -> Message + 'a>)>,
) -> event::Status {
    let mut event_status = event::Status::Ignored;

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let bounds = layout.bounds();

            if let Some(cursor_position) = cursor.position_over(bounds) {
                event_status = event::Status::Captured;

                match on_resize {
                    Some((leeway, _)) => {
                        let relative_cursor = Point::new(
                            cursor_position.x - bounds.x,
                            cursor_position.y - bounds.y,
                        );

                        let splits = node.split_regions(
                            spacing,
                            Size::new(bounds.width, bounds.height),
                        );

                        let clicked_split = hovered_split(
                            splits.iter(),
                            spacing + leeway,
                            relative_cursor,
                        );

                        if let Some((split, axis, _)) = clicked_split {
                            if action.picked_pane().is_none() {
                                *action =
                                    state::Action::Resizing { split, axis };
                            }
                        } else {
                            click_pane(
                                action,
                                layout,
                                cursor_position,
                                shell,
                                contents,
                                on_click,
                                on_drag,
                            );
                        }
                    }
                    None => {
                        click_pane(
                            action,
                            layout,
                            cursor_position,
                            shell,
                            contents,
                            on_click,
                            on_drag,
                        );
                    }
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. })
        | Event::Touch(touch::Event::FingerLost { .. }) => {
            if let Some((pane, _)) = action.picked_pane() {
                if let Some(on_drag) = on_drag {
                    if let Some(cursor_position) = cursor.position() {
                        let event = if let Some(edge) =
                            in_edge(layout, cursor_position)
                        {
                            DragEvent::Dropped {
                                pane,
                                target: Target::Edge(edge),
                            }
                        } else {
                            let dropped_region = contents
                                .zip(layout.children())
                                .filter_map(|(target, layout)| {
                                    layout_region(layout, cursor_position)
                                        .map(|region| (target, region))
                                })
                                .next();

                            match dropped_region {
                                Some(((target, _), region))
                                    if pane != target =>
                                {
                                    DragEvent::Dropped {
                                        pane,
                                        target: Target::Pane(target, region),
                                    }
                                }
                                _ => DragEvent::Canceled { pane },
                            }
                        };

                        shell.publish(on_drag(event));
                    }
                }

                event_status = event::Status::Captured;
            } else if action.picked_split().is_some() {
                event_status = event::Status::Captured;
            }

            *action = state::Action::Idle;
        }
        Event::Mouse(mouse::Event::CursorMoved { .. })
        | Event::Touch(touch::Event::FingerMoved { .. }) => {
            if let Some((_, on_resize)) = on_resize {
                if let Some((split, _)) = action.picked_split() {
                    let bounds = layout.bounds();

                    let splits = node.split_regions(
                        spacing,
                        Size::new(bounds.width, bounds.height),
                    );

                    if let Some((axis, rectangle, _)) = splits.get(&split) {
                        if let Some(cursor_position) = cursor.position() {
                            let ratio = match axis {
                                Axis::Horizontal => {
                                    let position = cursor_position.y
                                        - bounds.y
                                        - rectangle.y;

                                    (position / rectangle.height)
                                        .clamp(0.1, 0.9)
                                }
                                Axis::Vertical => {
                                    let position = cursor_position.x
                                        - bounds.x
                                        - rectangle.x;

                                    (position / rectangle.width).clamp(0.1, 0.9)
                                }
                            };

                            shell.publish(on_resize(ResizeEvent {
                                split,
                                ratio,
                            }));

                            event_status = event::Status::Captured;
                        }
                    }
                }
            }
        }
        _ => {}
    }

    event_status
}

fn layout_region(layout: Layout<'_>, cursor_position: Point) -> Option<Region> {
    let bounds = layout.bounds();

    if !bounds.contains(cursor_position) {
        return None;
    }

    let region = if cursor_position.x < (bounds.x + bounds.width / 3.0) {
        Region::Edge(Edge::Left)
    } else if cursor_position.x > (bounds.x + 2.0 * bounds.width / 3.0) {
        Region::Edge(Edge::Right)
    } else if cursor_position.y < (bounds.y + bounds.height / 3.0) {
        Region::Edge(Edge::Top)
    } else if cursor_position.y > (bounds.y + 2.0 * bounds.height / 3.0) {
        Region::Edge(Edge::Bottom)
    } else {
        Region::Center
    };

    Some(region)
}

fn click_pane<'a, Message, T>(
    action: &mut state::Action,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    contents: impl Iterator<Item = (Pane, T)>,
    on_click: &Option<Box<dyn Fn(Pane) -> Message + 'a>>,
    on_drag: &Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
) where
    T: Draggable,
{
    let mut clicked_region = contents
        .zip(layout.children())
        .filter(|(_, layout)| layout.bounds().contains(cursor_position));

    if let Some(((pane, content), layout)) = clicked_region.next() {
        if let Some(on_click) = &on_click {
            shell.publish(on_click(pane));
        }

        if let Some(on_drag) = &on_drag {
            if content.can_be_dragged_at(layout, cursor_position) {
                let pane_position = layout.position();

                let origin = cursor_position
                    - Vector::new(pane_position.x, pane_position.y);

                *action = state::Action::Dragging { pane, origin };

                shell.publish(on_drag(DragEvent::Picked { pane }));
            }
        }
    }
}

/// Returns the current [`mouse::Interaction`] of a [`PaneGrid`].
pub fn mouse_interaction(
    action: &state::Action,
    node: &Node,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    spacing: f32,
    resize_leeway: Option<f32>,
) -> Option<mouse::Interaction> {
    if action.picked_pane().is_some() {
        return Some(mouse::Interaction::Grabbing);
    }

    let resize_axis =
        action.picked_split().map(|(_, axis)| axis).or_else(|| {
            resize_leeway.and_then(|leeway| {
                let cursor_position = cursor.position()?;
                let bounds = layout.bounds();

                let splits = node.split_regions(spacing, bounds.size());

                let relative_cursor = Point::new(
                    cursor_position.x - bounds.x,
                    cursor_position.y - bounds.y,
                );

                hovered_split(splits.iter(), spacing + leeway, relative_cursor)
                    .map(|(_, axis, _)| axis)
            })
        });

    if let Some(resize_axis) = resize_axis {
        return Some(match resize_axis {
            Axis::Horizontal => mouse::Interaction::ResizingVertically,
            Axis::Vertical => mouse::Interaction::ResizingHorizontally,
        });
    }

    None
}

/// Draws a [`PaneGrid`].
pub fn draw<Renderer, T>(
    action: &state::Action,
    node: &Node,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    default_style: &renderer::Style,
    viewport: &Rectangle,
    spacing: f32,
    resize_leeway: Option<f32>,
    style: &<Renderer::Theme as StyleSheet>::Style,
    contents: impl Iterator<Item = (Pane, T)>,
    draw_pane: impl Fn(
        T,
        &mut Renderer,
        &renderer::Style,
        Layout<'_>,
        mouse::Cursor,
        &Rectangle,
    ),
) where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    let picked_pane = action.picked_pane();

    let picked_split = action
        .picked_split()
        .and_then(|(split, axis)| {
            let bounds = layout.bounds();

            let splits = node.split_regions(spacing, bounds.size());

            let (_axis, region, ratio) = splits.get(&split)?;

            let region = axis.split_line_bounds(*region, *ratio, spacing);

            Some((axis, region + Vector::new(bounds.x, bounds.y), true))
        })
        .or_else(|| match resize_leeway {
            Some(leeway) => {
                let cursor_position = cursor.position()?;
                let bounds = layout.bounds();

                let relative_cursor = Point::new(
                    cursor_position.x - bounds.x,
                    cursor_position.y - bounds.y,
                );

                let splits = node.split_regions(spacing, bounds.size());

                let (_split, axis, region) = hovered_split(
                    splits.iter(),
                    spacing + leeway,
                    relative_cursor,
                )?;

                Some((axis, region + Vector::new(bounds.x, bounds.y), false))
            }
            None => None,
        });

    let pane_cursor = if picked_pane.is_some() {
        mouse::Cursor::Unavailable
    } else {
        cursor
    };

    let mut render_picked_pane = None;

    let pane_in_edge = if picked_pane.is_some() {
        cursor
            .position()
            .and_then(|cursor_position| in_edge(layout, cursor_position))
    } else {
        None
    };

    for ((id, pane), pane_layout) in contents.zip(layout.children()) {
        match picked_pane {
            Some((dragging, origin)) if id == dragging => {
                render_picked_pane = Some((pane, origin, pane_layout));
            }
            Some((dragging, _)) if id != dragging => {
                draw_pane(
                    pane,
                    renderer,
                    default_style,
                    pane_layout,
                    pane_cursor,
                    viewport,
                );

                if picked_pane.is_some() && pane_in_edge.is_none() {
                    if let Some(region) =
                        cursor.position().and_then(|cursor_position| {
                            layout_region(pane_layout, cursor_position)
                        })
                    {
                        let bounds = layout_region_bounds(pane_layout, region);
                        let hovered_region_style = theme.hovered_region(style);

                        renderer.fill_quad(
                            renderer::Quad {
                                bounds,
                                border_radius: hovered_region_style
                                    .border_radius,
                                border_width: hovered_region_style.border_width,
                                border_color: hovered_region_style.border_color,
                            },
                            theme.hovered_region(style).background,
                        );
                    }
                }
            }
            _ => {
                draw_pane(
                    pane,
                    renderer,
                    default_style,
                    pane_layout,
                    pane_cursor,
                    viewport,
                );
            }
        }
    }

    if let Some(edge) = pane_in_edge {
        let hovered_region_style = theme.hovered_region(style);
        let bounds = edge_bounds(layout, edge);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: hovered_region_style.border_radius,
                border_width: hovered_region_style.border_width,
                border_color: hovered_region_style.border_color,
            },
            theme.hovered_region(style).background,
        );
    }

    // Render picked pane last
    if let Some((pane, origin, layout)) = render_picked_pane {
        if let Some(cursor_position) = cursor.position() {
            let bounds = layout.bounds();

            renderer.with_translation(
                cursor_position
                    - Point::new(bounds.x + origin.x, bounds.y + origin.y),
                |renderer| {
                    renderer.with_layer(bounds, |renderer| {
                        draw_pane(
                            pane,
                            renderer,
                            default_style,
                            layout,
                            pane_cursor,
                            viewport,
                        );
                    });
                },
            );
        }
    }

    if picked_pane.is_none() {
        if let Some((axis, split_region, is_picked)) = picked_split {
            let highlight = if is_picked {
                theme.picked_split(style)
            } else {
                theme.hovered_split(style)
            };

            if let Some(highlight) = highlight {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: match axis {
                            Axis::Horizontal => Rectangle {
                                x: split_region.x,
                                y: (split_region.y
                                    + (split_region.height - highlight.width)
                                        / 2.0)
                                    .round(),
                                width: split_region.width,
                                height: highlight.width,
                            },
                            Axis::Vertical => Rectangle {
                                x: (split_region.x
                                    + (split_region.width - highlight.width)
                                        / 2.0)
                                    .round(),
                                y: split_region.y,
                                width: highlight.width,
                                height: split_region.height,
                            },
                        },
                        border_radius: 0.0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    highlight.color,
                );
            }
        }
    }
}

const THICKNESS_RATIO: f32 = 25.0;

fn in_edge(layout: Layout<'_>, cursor: Point) -> Option<Edge> {
    let bounds = layout.bounds();

    let height_thickness = bounds.height / THICKNESS_RATIO;
    let width_thickness = bounds.width / THICKNESS_RATIO;
    let thickness = height_thickness.min(width_thickness);

    if cursor.x > bounds.x && cursor.x < bounds.x + thickness {
        Some(Edge::Left)
    } else if cursor.x > bounds.x + bounds.width - thickness
        && cursor.x < bounds.x + bounds.width
    {
        Some(Edge::Right)
    } else if cursor.y > bounds.y && cursor.y < bounds.y + thickness {
        Some(Edge::Top)
    } else if cursor.y > bounds.y + bounds.height - thickness
        && cursor.y < bounds.y + bounds.height
    {
        Some(Edge::Bottom)
    } else {
        None
    }
}

fn edge_bounds(layout: Layout<'_>, edge: Edge) -> Rectangle {
    let bounds = layout.bounds();

    let height_thickness = bounds.height / THICKNESS_RATIO;
    let width_thickness = bounds.width / THICKNESS_RATIO;
    let thickness = height_thickness.min(width_thickness);

    match edge {
        Edge::Top => Rectangle {
            height: thickness,
            ..bounds
        },
        Edge::Left => Rectangle {
            width: thickness,
            ..bounds
        },
        Edge::Right => Rectangle {
            x: bounds.x + bounds.width - thickness,
            width: thickness,
            ..bounds
        },
        Edge::Bottom => Rectangle {
            y: bounds.y + bounds.height - thickness,
            height: thickness,
            ..bounds
        },
    }
}

fn layout_region_bounds(layout: Layout<'_>, region: Region) -> Rectangle {
    let bounds = layout.bounds();

    match region {
        Region::Center => bounds,
        Region::Edge(edge) => match edge {
            Edge::Top => Rectangle {
                height: bounds.height / 2.0,
                ..bounds
            },
            Edge::Left => Rectangle {
                width: bounds.width / 2.0,
                ..bounds
            },
            Edge::Right => Rectangle {
                x: bounds.x + bounds.width / 2.0,
                width: bounds.width / 2.0,
                ..bounds
            },
            Edge::Bottom => Rectangle {
                y: bounds.y + bounds.height / 2.0,
                height: bounds.height / 2.0,
                ..bounds
            },
        },
    }
}

/// An event produced during a drag and drop interaction of a [`PaneGrid`].
#[derive(Debug, Clone, Copy)]
pub enum DragEvent {
    /// A [`Pane`] was picked for dragging.
    Picked {
        /// The picked [`Pane`].
        pane: Pane,
    },

    /// A [`Pane`] was dropped on top of another [`Pane`].
    Dropped {
        /// The picked [`Pane`].
        pane: Pane,

        /// The [`Target`] where the picked [`Pane`] was dropped on.
        target: Target,
    },

    /// A [`Pane`] was picked and then dropped outside of other [`Pane`]
    /// boundaries.
    Canceled {
        /// The picked [`Pane`].
        pane: Pane,
    },
}

/// The [`Target`] area a pane can be dropped on.
#[derive(Debug, Clone, Copy)]
pub enum Target {
    /// An [`Edge`] of the full [`PaneGrid`].
    Edge(Edge),
    /// A single [`Pane`] of the [`PaneGrid`].
    Pane(Pane, Region),
}

/// The region of a [`Pane`].
#[derive(Debug, Clone, Copy, Default)]
pub enum Region {
    /// Center region.
    #[default]
    Center,
    /// Edge region.
    Edge(Edge),
}

/// The edges of an area.
#[derive(Debug, Clone, Copy)]
pub enum Edge {
    /// Top edge.
    Top,
    /// Left edge.
    Left,
    /// Right edge.
    Right,
    /// Bottom edge.
    Bottom,
}

/// An event produced during a resize interaction of a [`PaneGrid`].
#[derive(Debug, Clone, Copy)]
pub struct ResizeEvent {
    /// The [`Split`] that is being dragged for resizing.
    pub split: Split,

    /// The new ratio of the [`Split`].
    ///
    /// The ratio is a value in [0, 1], representing the exact position of a
    /// [`Split`] between two panes.
    pub ratio: f32,
}

/*
 * Helpers
 */
fn hovered_split<'a>(
    splits: impl Iterator<Item = (&'a Split, &'a (Axis, Rectangle, f32))>,
    spacing: f32,
    cursor_position: Point,
) -> Option<(Split, Axis, Rectangle)> {
    splits
        .filter_map(|(split, (axis, region, ratio))| {
            let bounds = axis.split_line_bounds(*region, *ratio, spacing);

            if bounds.contains(cursor_position) {
                Some((*split, *axis, bounds))
            } else {
                None
            }
        })
        .next()
}

/// The visible contents of the [`PaneGrid`]
#[derive(Debug)]
pub enum Contents<'a, T> {
    /// All panes are visible
    All(Vec<(Pane, T)>, &'a state::Internal),
    /// A maximized pane is visible
    Maximized(Pane, T, Node),
}

impl<'a, T> Contents<'a, T> {
    /// Returns the layout [`Node`] of the [`Contents`]
    pub fn layout(&self) -> &Node {
        match self {
            Contents::All(_, state) => state.layout(),
            Contents::Maximized(_, _, layout) => layout,
        }
    }

    /// Returns an iterator over the values of the [`Contents`]
    pub fn iter(&self) -> Box<dyn Iterator<Item = (Pane, &T)> + '_> {
        match self {
            Contents::All(contents, _) => Box::new(
                contents.iter().map(|(pane, content)| (*pane, content)),
            ),
            Contents::Maximized(pane, content, _) => {
                Box::new(std::iter::once((*pane, content)))
            }
        }
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (Pane, &mut T)> + '_> {
        match self {
            Contents::All(contents, _) => Box::new(
                contents.iter_mut().map(|(pane, content)| (*pane, content)),
            ),
            Contents::Maximized(pane, content, _) => {
                Box::new(std::iter::once((*pane, content)))
            }
        }
    }

    fn is_maximized(&self) -> bool {
        matches!(self, Self::Maximized(..))
    }
}
