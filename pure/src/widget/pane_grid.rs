//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
//!
//! # Example
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/iced-rs/iced/tree/0.4/examples/pane_grid
mod content;
mod title_bar;

pub use content::Content;
pub use title_bar::TitleBar;

pub use iced_native::widget::pane_grid::{
    Axis, Configuration, Direction, DragEvent, Node, Pane, ResizeEvent, Split,
    State,
};

use crate::overlay;
use crate::widget::tree::{self, Tree};
use crate::{Element, Widget};

use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::renderer;
use iced_native::widget::pane_grid;
use iced_native::widget::pane_grid::state;
use iced_native::{Clipboard, Layout, Length, Point, Rectangle, Shell};

pub use iced_style::pane_grid::{Line, StyleSheet};

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
/// ```
/// # use iced_pure::widget::pane_grid;
/// # use iced_pure::text;
/// #
/// # type PaneGrid<'a, Message> =
/// #     iced_pure::widget::PaneGrid<'a, Message, iced_native::renderer::Null>;
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
///     PaneGrid::new(&state, |pane, state| {
///         pane_grid::Content::new(match state {
///             PaneState::SomePane => text("This is some pane"),
///             PaneState::AnotherKindOfPane => text("This is another kind of pane"),
///         })
///     })
///     .on_drag(Message::PaneDragged)
///     .on_resize(10, Message::PaneResized);
/// ```
#[allow(missing_debug_implementations)]
pub struct PaneGrid<'a, Message, Renderer> {
    state: &'a state::Internal,
    elements: Vec<(Pane, Content<'a, Message, Renderer>)>,
    width: Length,
    height: Length,
    spacing: u16,
    on_click: Option<Box<dyn Fn(Pane) -> Message + 'a>>,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    on_resize: Option<(u16, Box<dyn Fn(ResizeEvent) -> Message + 'a>)>,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer> PaneGrid<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    /// Creates a [`PaneGrid`] with the given [`State`] and view function.
    ///
    /// The view function will be called to display each [`Pane`] present in the
    /// [`State`].
    pub fn new<T>(
        state: &'a State<T>,
        view: impl Fn(Pane, &'a T) -> Content<'a, Message, Renderer>,
    ) -> Self {
        let elements = {
            state
                .panes
                .iter()
                .map(|(pane, pane_state)| (*pane, view(*pane, pane_state)))
                .collect()
        };

        Self {
            elements,
            state: &state.internal,
            width: Length::Fill,
            height: Length::Fill,
            spacing: 0,
            on_click: None,
            on_drag: None,
            on_resize: None,
            style_sheet: Default::default(),
        }
    }

    /// Sets the width of the [`PaneGrid`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`PaneGrid`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the spacing _between_ the panes of the [`PaneGrid`].
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
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
    pub fn on_resize<F>(mut self, leeway: u16, f: F) -> Self
    where
        F: 'a + Fn(ResizeEvent) -> Message,
    {
        self.on_resize = Some((leeway, Box::new(f)));
        self
    }

    /// Sets the style of the [`PaneGrid`].
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet + 'a>>) -> Self {
        self.style_sheet = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for PaneGrid<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<state::Action>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(state::Action::Idle)
    }

    fn children(&self) -> Vec<Tree> {
        self.elements
            .iter()
            .map(|(_, content)| content.state())
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children_custom(
            &self.elements,
            |state, (_, content)| content.diff(state),
            |(_, content)| content.state(),
        )
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
        pane_grid::layout(
            renderer,
            limits,
            self.state,
            self.width,
            self.height,
            self.spacing,
            self.elements.iter().map(|(pane, content)| (*pane, content)),
            |element, renderer, limits| element.layout(renderer, limits),
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let action = tree.state.downcast_mut::<state::Action>();

        let event_status = pane_grid::update(
            action,
            self.state,
            &event,
            layout,
            cursor_position,
            shell,
            self.spacing,
            self.elements.iter().map(|(pane, content)| (*pane, content)),
            &self.on_click,
            &self.on_drag,
            &self.on_resize,
        );

        let picked_pane = action.picked_pane().map(|(pane, _)| pane);

        self.elements
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|(((pane, content), tree), layout)| {
                let is_picked = picked_pane == Some(*pane);

                content.on_event(
                    tree,
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                    is_picked,
                )
            })
            .fold(event_status, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        pane_grid::mouse_interaction(
            tree.state.downcast_ref(),
            self.state,
            layout,
            cursor_position,
            self.spacing,
            self.on_resize.as_ref().map(|(leeway, _)| *leeway),
        )
        .unwrap_or_else(|| {
            self.elements
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .map(|(((_pane, content), tree), layout)| {
                    content.mouse_interaction(
                        tree,
                        layout,
                        cursor_position,
                        viewport,
                        renderer,
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        pane_grid::draw(
            tree.state.downcast_ref(),
            self.state,
            layout,
            cursor_position,
            renderer,
            style,
            viewport,
            self.spacing,
            self.on_resize.as_ref().map(|(leeway, _)| *leeway),
            self.style_sheet.as_ref(),
            self.elements
                .iter()
                .zip(&tree.children)
                .map(|((pane, content), tree)| (*pane, (content, tree))),
            |(content, tree),
             renderer,
             style,
             layout,
             cursor_position,
             rectangle| {
                content.draw(
                    tree,
                    renderer,
                    style,
                    layout,
                    cursor_position,
                    rectangle,
                );
            },
        )
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.elements
            .iter()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|(((_, pane), tree), layout)| {
                pane.overlay(tree, layout, renderer)
            })
            .next()
    }
}

impl<'a, Message, Renderer> From<PaneGrid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + iced_native::Renderer,
    Message: 'a,
{
    fn from(
        pane_grid: PaneGrid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(pane_grid)
    }
}
