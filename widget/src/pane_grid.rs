//! Pane grids let your users split regions of your application and organize layout dynamically.
//!
//! ![Pane grid - Iced](https://iced.rs/examples/pane_grid.gif)
//!
//! This distribution of space is common in tiling window managers (like
//! [`awesome`](https://awesomewm.org/), [`i3`](https://i3wm.org/), or even
//! [`tmux`](https://github.com/tmux/tmux)).
//!
//! A [`PaneGrid`] supports:
//!
//! * Vertical and horizontal splits
//! * Tracking of the last active pane
//! * Mouse-based resizing
//! * Drag and drop to reorganize panes
//! * Hotkey support
//! * Configurable modifier keys
//! * [`State`] API to perform actions programmatically (`split`, `swap`, `resize`, etc.)
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::{pane_grid, text};
//!
//! struct State {
//!     panes: pane_grid::State<Pane>,
//! }
//!
//! enum Pane {
//!     SomePane,
//!     AnotherKindOfPane,
//! }
//!
//! enum Message {
//!     PaneDragged(pane_grid::DragEvent),
//!     PaneResized(pane_grid::ResizeEvent),
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     pane_grid(&state.panes, |pane, state, is_maximized| {
//!         pane_grid::Content::new(match state {
//!             Pane::SomePane => text("This is some pane"),
//!             Pane::AnotherKindOfPane => text("This is another kind of pane"),
//!         })
//!     })
//!     .on_drag(Message::PaneDragged)
//!     .on_resize(10, Message::PaneResized)
//!     .into()
//! }
//! ```
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/iced-rs/iced/tree/0.13/examples/pane_grid
mod axis;
mod configuration;
mod content;
mod controls;
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
pub use controls::Controls;
pub use direction::Direction;
pub use draggable::Draggable;
pub use node::Node;
pub use pane::Pane;
pub use split::Split;
pub use state::State;
pub use title_bar::TitleBar;

use crate::container;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay::{self, Group};
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    self, Background, Border, Clipboard, Color, Element, Event, Layout, Length,
    Pixels, Point, Rectangle, Shell, Size, Theme, Vector, Widget,
};

const DRAG_DEADBAND_DISTANCE: f32 = 10.0;
const THICKNESS_RATIO: f32 = 25.0;

/// A collection of panes distributed using either vertical or horizontal splits
/// to completely fill the space available.
///
/// ![Pane grid - Iced](https://iced.rs/examples/pane_grid.gif)
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
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::{pane_grid, text};
///
/// struct State {
///     panes: pane_grid::State<Pane>,
/// }
///
/// enum Pane {
///     SomePane,
///     AnotherKindOfPane,
/// }
///
/// enum Message {
///     PaneDragged(pane_grid::DragEvent),
///     PaneResized(pane_grid::ResizeEvent),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     pane_grid(&state.panes, |pane, state, is_maximized| {
///         pane_grid::Content::new(match state {
///             Pane::SomePane => text("This is some pane"),
///             Pane::AnotherKindOfPane => text("This is another kind of pane"),
///         })
///     })
///     .on_drag(Message::PaneDragged)
///     .on_resize(10, Message::PaneResized)
///     .into()
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct PaneGrid<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    internal: &'a state::Internal,
    panes: Vec<Pane>,
    contents: Vec<Content<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
    spacing: f32,
    min_size: f32,
    on_click: Option<Box<dyn Fn(Pane) -> Message + 'a>>,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    on_resize: Option<(f32, Box<dyn Fn(ResizeEvent) -> Message + 'a>)>,
    class: <Theme as Catalog>::Class<'a>,
    last_mouse_interaction: Option<mouse::Interaction>,
}

impl<'a, Message, Theme, Renderer> PaneGrid<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    /// Creates a [`PaneGrid`] with the given [`State`] and view function.
    ///
    /// The view function will be called to display each [`Pane`] present in the
    /// [`State`]. [`bool`] is set if the pane is maximized.
    pub fn new<T>(
        state: &'a State<T>,
        view: impl Fn(Pane, &'a T, bool) -> Content<'a, Message, Theme, Renderer>,
    ) -> Self {
        let panes = state.panes.keys().copied().collect();
        let contents = state
            .panes
            .iter()
            .map(|(pane, pane_state)| match state.maximized() {
                Some(p) if *pane == p => view(*pane, pane_state, true),
                _ => view(*pane, pane_state, false),
            })
            .collect();

        Self {
            internal: &state.internal,
            panes,
            contents,
            width: Length::Fill,
            height: Length::Fill,
            spacing: 0.0,
            min_size: 50.0,
            on_click: None,
            on_drag: None,
            on_resize: None,
            class: <Theme as Catalog>::default(),
            last_mouse_interaction: None,
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

    /// Sets the minimum size of a [`Pane`] in the [`PaneGrid`] on both axes.
    pub fn min_size(mut self, min_size: impl Into<Pixels>) -> Self {
        self.min_size = min_size.into().0;
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
        if self.internal.maximized().is_none() {
            self.on_drag = Some(Box::new(f));
        }
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
        if self.internal.maximized().is_none() {
            self.on_resize = Some((leeway.into().0, Box::new(f)));
        }
        self
    }

    /// Sets the style of the [`PaneGrid`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`PaneGrid`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(
        mut self,
        class: impl Into<<Theme as Catalog>::Class<'a>>,
    ) -> Self {
        self.class = class.into();
        self
    }

    fn drag_enabled(&self) -> bool {
        self.internal
            .maximized()
            .is_none()
            .then(|| self.on_drag.is_some())
            .unwrap_or_default()
    }

    fn grid_interaction(
        &self,
        action: &state::Action,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) -> Option<mouse::Interaction> {
        if action.picked_pane().is_some() {
            return Some(mouse::Interaction::Grabbing);
        }

        let resize_leeway = self.on_resize.as_ref().map(|(leeway, _)| *leeway);
        let node = self.internal.layout();

        let resize_axis =
            action.picked_split().map(|(_, axis)| axis).or_else(|| {
                resize_leeway.and_then(|leeway| {
                    let cursor_position = cursor.position()?;
                    let bounds = layout.bounds();

                    let splits = node.split_regions(
                        self.spacing,
                        self.min_size,
                        bounds.size(),
                    );

                    let relative_cursor = Point::new(
                        cursor_position.x - bounds.x,
                        cursor_position.y - bounds.y,
                    );

                    hovered_split(
                        splits.iter(),
                        self.spacing + leeway,
                        relative_cursor,
                    )
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
}

#[derive(Default)]
struct Memory {
    action: state::Action,
    order: Vec<Pane>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for PaneGrid<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Memory>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Memory::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.contents.iter().map(Content::state).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let Memory { order, .. } = tree.state.downcast_ref();

        // `Pane` always increments and is iterated by Ord so new
        // states are always added at the end. We can simply remove
        // states which no longer exist and `diff_children` will
        // diff the remaining values in the correct order and
        // add new states at the end

        let mut i = 0;
        let mut j = 0;
        tree.children.retain(|_| {
            let retain = self.panes.get(i) == order.get(j);

            if retain {
                i += 1;
            }
            j += 1;

            retain
        });

        tree.diff_children_custom(
            &self.contents,
            |state, content| content.diff(state),
            Content::state,
        );

        let Memory { order, .. } = tree.state.downcast_mut();
        order.clone_from(&self.panes);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let bounds = limits.resolve(self.width, self.height, Size::ZERO);
        let regions = self.internal.layout().pane_regions(
            self.spacing,
            self.min_size,
            bounds,
        );

        let children = self
            .panes
            .iter()
            .copied()
            .zip(&self.contents)
            .zip(tree.children.iter_mut())
            .filter_map(|((pane, content), tree)| {
                if self
                    .internal
                    .maximized()
                    .is_some_and(|maximized| maximized != pane)
                {
                    return Some(layout::Node::new(Size::ZERO));
                }

                let region = regions.get(&pane)?;
                let size = Size::new(region.width, region.height);

                let node = content.layout(
                    tree,
                    renderer,
                    &layout::Limits::new(size, size),
                );

                Some(node.move_to(Point::new(region.x, region.y)))
            })
            .collect();

        layout::Node::with_children(bounds, children)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.panes
                .iter()
                .copied()
                .zip(&self.contents)
                .zip(&mut tree.children)
                .zip(layout.children())
                .filter(|(((pane, _), _), _)| {
                    self.internal
                        .maximized()
                        .is_none_or(|maximized| *pane == maximized)
                })
                .for_each(|(((_, content), state), layout)| {
                    content.operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
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
        let Memory { action, .. } = tree.state.downcast_mut();
        let node = self.internal.layout();

        let on_drag = if self.drag_enabled() {
            &self.on_drag
        } else {
            &None
        };

        let picked_pane = action.picked_pane().map(|(pane, _)| pane);

        for (((pane, content), tree), layout) in self
            .panes
            .iter()
            .copied()
            .zip(&mut self.contents)
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter(|(((pane, _), _), _)| {
                self.internal
                    .maximized()
                    .is_none_or(|maximized| *pane == maximized)
            })
        {
            let is_picked = picked_pane == Some(pane);

            content.update(
                tree, event, layout, cursor, renderer, clipboard, shell,
                viewport, is_picked,
            );
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let bounds = layout.bounds();

                if let Some(cursor_position) = cursor.position_over(bounds) {
                    shell.capture_event();

                    match &self.on_resize {
                        Some((leeway, _)) => {
                            let relative_cursor = Point::new(
                                cursor_position.x - bounds.x,
                                cursor_position.y - bounds.y,
                            );

                            let splits = node.split_regions(
                                self.spacing,
                                self.min_size,
                                bounds.size(),
                            );

                            let clicked_split = hovered_split(
                                splits.iter(),
                                self.spacing + leeway,
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
                                    self.panes
                                        .iter()
                                        .copied()
                                        .zip(&self.contents),
                                    &self.on_click,
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
                                self.panes.iter().copied().zip(&self.contents),
                                &self.on_click,
                                on_drag,
                            );
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if let Some((pane, origin)) = action.picked_pane() {
                    if let Some(on_drag) = on_drag {
                        if let Some(cursor_position) = cursor.position() {
                            if cursor_position.distance(origin)
                                > DRAG_DEADBAND_DISTANCE
                            {
                                let event = if let Some(edge) =
                                    in_edge(layout, cursor_position)
                                {
                                    DragEvent::Dropped {
                                        pane,
                                        target: Target::Edge(edge),
                                    }
                                } else {
                                    let dropped_region = self
                                        .panes
                                        .iter()
                                        .copied()
                                        .zip(&self.contents)
                                        .zip(layout.children())
                                        .find_map(|(target, layout)| {
                                            layout_region(
                                                layout,
                                                cursor_position,
                                            )
                                            .map(|region| (target, region))
                                        });

                                    match dropped_region {
                                        Some(((target, _), region))
                                            if pane != target =>
                                        {
                                            DragEvent::Dropped {
                                                pane,
                                                target: Target::Pane(
                                                    target, region,
                                                ),
                                            }
                                        }
                                        _ => DragEvent::Canceled { pane },
                                    }
                                };

                                shell.publish(on_drag(event));
                            } else {
                                shell.publish(on_drag(DragEvent::Canceled {
                                    pane,
                                }));
                            }
                        }
                    }
                }

                *action = state::Action::Idle;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some((_, on_resize)) = &self.on_resize {
                    if let Some((split, _)) = action.picked_split() {
                        let bounds = layout.bounds();

                        let splits = node.split_regions(
                            self.spacing,
                            self.min_size,
                            bounds.size(),
                        );

                        if let Some((axis, rectangle, _)) = splits.get(&split) {
                            if let Some(cursor_position) = cursor.position() {
                                let ratio = match axis {
                                    Axis::Horizontal => {
                                        let position = cursor_position.y
                                            - bounds.y
                                            - rectangle.y;

                                        (position / rectangle.height)
                                            .clamp(0.0, 1.0)
                                    }
                                    Axis::Vertical => {
                                        let position = cursor_position.x
                                            - bounds.x
                                            - rectangle.x;

                                        (position / rectangle.width)
                                            .clamp(0.0, 1.0)
                                    }
                                };

                                shell.publish(on_resize(ResizeEvent {
                                    split,
                                    ratio,
                                }));

                                shell.capture_event();
                            }
                        }
                    } else if action.picked_pane().is_some() {
                        shell.request_redraw();
                    }
                }
            }
            _ => {}
        }

        if shell.redraw_request() != window::RedrawRequest::NextFrame {
            let interaction = self
                .grid_interaction(action, layout, cursor)
                .or_else(|| {
                    self.panes
                        .iter()
                        .zip(&self.contents)
                        .zip(layout.children())
                        .filter(|((pane, _content), _layout)| {
                            self.internal
                                .maximized()
                                .is_none_or(|maximized| **pane == maximized)
                        })
                        .find_map(|((_pane, content), layout)| {
                            content.grid_interaction(
                                layout,
                                cursor,
                                on_drag.is_some(),
                            )
                        })
                })
                .unwrap_or(mouse::Interaction::None);

            if let Event::Window(window::Event::RedrawRequested(_now)) = event {
                self.last_mouse_interaction = Some(interaction);
            } else if self.last_mouse_interaction.is_some_and(
                |last_mouse_interaction| last_mouse_interaction != interaction,
            ) {
                shell.request_redraw();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Memory { action, .. } = tree.state.downcast_ref();

        if let Some(grid_interaction) =
            self.grid_interaction(action, layout, cursor)
        {
            return grid_interaction;
        }

        self.panes
            .iter()
            .copied()
            .zip(&self.contents)
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(((pane, _), _), _)| {
                self.internal
                    .maximized()
                    .is_none_or(|maximized| *pane == maximized)
            })
            .map(|(((_, content), tree), layout)| {
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
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Memory { action, .. } = tree.state.downcast_ref();
        let node = self.internal.layout();
        let resize_leeway = self.on_resize.as_ref().map(|(leeway, _)| *leeway);

        let picked_pane = action.picked_pane().filter(|(_, origin)| {
            cursor
                .position()
                .map(|position| position.distance(*origin))
                .unwrap_or_default()
                > DRAG_DEADBAND_DISTANCE
        });

        let picked_split = action
            .picked_split()
            .and_then(|(split, axis)| {
                let bounds = layout.bounds();

                let splits = node.split_regions(
                    self.spacing,
                    self.min_size,
                    bounds.size(),
                );

                let (_axis, region, ratio) = splits.get(&split)?;

                let region =
                    axis.split_line_bounds(*region, *ratio, self.spacing);

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

                    let splits = node.split_regions(
                        self.spacing,
                        self.min_size,
                        bounds.size(),
                    );

                    let (_split, axis, region) = hovered_split(
                        splits.iter(),
                        self.spacing + leeway,
                        relative_cursor,
                    )?;

                    Some((
                        axis,
                        region + Vector::new(bounds.x, bounds.y),
                        false,
                    ))
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

        let style = Catalog::style(theme, &self.class);

        for (((id, content), tree), pane_layout) in self
            .panes
            .iter()
            .copied()
            .zip(&self.contents)
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(((pane, _), _), _)| {
                self.internal
                    .maximized()
                    .is_none_or(|maximized| maximized == *pane)
            })
        {
            match picked_pane {
                Some((dragging, origin)) if id == dragging => {
                    render_picked_pane =
                        Some(((content, tree), origin, pane_layout));
                }
                Some((dragging, _)) if id != dragging => {
                    content.draw(
                        tree,
                        renderer,
                        theme,
                        defaults,
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
                            let bounds =
                                layout_region_bounds(pane_layout, region);

                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds,
                                    border: style.hovered_region.border,
                                    ..renderer::Quad::default()
                                },
                                style.hovered_region.background,
                            );
                        }
                    }
                }
                _ => {
                    content.draw(
                        tree,
                        renderer,
                        theme,
                        defaults,
                        pane_layout,
                        pane_cursor,
                        viewport,
                    );
                }
            }
        }

        if let Some(edge) = pane_in_edge {
            let bounds = edge_bounds(layout, edge);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.hovered_region.border,
                    ..renderer::Quad::default()
                },
                style.hovered_region.background,
            );
        }

        // Render picked pane last
        if let Some(((content, tree), origin, layout)) = render_picked_pane {
            if let Some(cursor_position) = cursor.position() {
                let bounds = layout.bounds();

                let translation =
                    cursor_position - Point::new(origin.x, origin.y);

                renderer.with_translation(translation, |renderer| {
                    renderer.with_layer(bounds, |renderer| {
                        content.draw(
                            tree,
                            renderer,
                            theme,
                            defaults,
                            layout,
                            pane_cursor,
                            viewport,
                        );
                    });
                });
            }
        }

        if picked_pane.is_none() {
            if let Some((axis, split_region, is_picked)) = picked_split {
                let highlight = if is_picked {
                    style.picked_split
                } else {
                    style.hovered_split
                };

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
                        ..renderer::Quad::default()
                    },
                    highlight.color,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let children = self
            .panes
            .iter()
            .copied()
            .zip(&mut self.contents)
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|(((pane, content), state), layout)| {
                if self
                    .internal
                    .maximized()
                    .is_some_and(|maximized| maximized != pane)
                {
                    return None;
                }

                content.overlay(state, layout, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<PaneGrid<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(
        pane_grid: PaneGrid<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(pane_grid)
    }
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
                *action = state::Action::Dragging {
                    pane,
                    origin: cursor_position,
                };

                shell.publish(on_drag(DragEvent::Picked { pane }));
            }
        }
    }
}

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
    mut splits: impl Iterator<Item = (&'a Split, &'a (Axis, Rectangle, f32))>,
    spacing: f32,
    cursor_position: Point,
) -> Option<(Split, Axis, Rectangle)> {
    splits.find_map(|(split, (axis, region, ratio))| {
        let bounds = axis.split_line_bounds(*region, *ratio, spacing);

        if bounds.contains(cursor_position) {
            Some((*split, *axis, bounds))
        } else {
            None
        }
    })
}

/// The appearance of a [`PaneGrid`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The appearance of a hovered region highlight.
    pub hovered_region: Highlight,
    /// The appearance of a picked split.
    pub picked_split: Line,
    /// The appearance of a hovered split.
    pub hovered_split: Line,
}

/// The appearance of a highlight of the [`PaneGrid`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Highlight {
    /// The [`Background`] of the pane region.
    pub background: Background,
    /// The [`Border`] of the pane region.
    pub border: Border,
}

/// A line.
///
/// It is normally used to define the highlight of something, like a split.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// The [`Color`] of the [`Line`].
    pub color: Color,
    /// The width of the [`Line`].
    pub width: f32,
}

/// The theme catalog of a [`PaneGrid`].
pub trait Catalog: container::Catalog {
    /// The item class of this [`Catalog`].
    type Class<'a>;

    /// The default class produced by this [`Catalog`].
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style;
}

/// A styling function for a [`PaneGrid`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

/// The default style of a [`PaneGrid`].
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        hovered_region: Highlight {
            background: Background::Color(Color {
                a: 0.5,
                ..palette.primary.base.color
            }),
            border: Border {
                width: 2.0,
                color: palette.primary.strong.color,
                radius: 0.0.into(),
            },
        },
        hovered_split: Line {
            color: palette.primary.base.color,
            width: 2.0,
        },
        picked_split: Line {
            color: palette.primary.strong.color,
            width: 2.0,
        },
    }
}
