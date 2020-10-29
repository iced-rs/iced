//! Let your users split regions of your application and organize layout dynamically.
//!
//! [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
//!
//! # Example
//! The [`pane_grid` example] showcases how to use a [`PaneGrid`] with resizing,
//! drag and drop, and hotkey support.
//!
//! [`pane_grid` example]: https://github.com/hecrj/iced/tree/0.1/examples/pane_grid
//! [`PaneGrid`]: struct.PaneGrid.html
mod axis;
mod configuration;
mod content;
mod direction;
mod node;
mod pane;
mod split;
mod state;
mod title_bar;

pub use axis::Axis;
pub use configuration::Configuration;
pub use content::Content;
pub use direction::Direction;
pub use node::Node;
pub use pane::Pane;
pub use split::Split;
pub use state::{Focus, State};
pub use title_bar::TitleBar;

use crate::{
    container, keyboard, layout, mouse, overlay, row, text, Clipboard, Element,
    Event, Hasher, Layout, Length, Point, Rectangle, Size, Vector, Widget,
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
/// ```
/// # use iced_native::{pane_grid, Text};
/// #
/// # type PaneGrid<'a, Message> =
/// #     iced_native::PaneGrid<'a, Message, iced_native::renderer::Null>;
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
///     PaneGrid::new(&mut state, |pane, state, focus| {
///         pane_grid::Content::new(match state {
///             PaneState::SomePane => Text::new("This is some pane"),
///             PaneState::AnotherKindOfPane => Text::new("This is another kind of pane"),
///         })
///     })
///     .on_drag(Message::PaneDragged)
///     .on_resize(10, Message::PaneResized);
/// ```
///
/// [`PaneGrid`]: struct.PaneGrid.html
/// [`State`]: struct.State.html
#[allow(missing_debug_implementations)]
pub struct PaneGrid<'a, Message, Renderer: self::Renderer> {
    state: &'a mut state::Internal,
    elements: Vec<(Pane, Content<'a, Message, Renderer>)>,
    width: Length,
    height: Length,
    spacing: u16,
    modifier_keys: keyboard::ModifiersState,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    on_resize: Option<(u16, Box<dyn Fn(ResizeEvent) -> Message + 'a>)>,
    on_key_press: Option<Box<dyn Fn(KeyPressEvent) -> Option<Message> + 'a>>,
}

impl<'a, Message, Renderer> PaneGrid<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    /// Creates a [`PaneGrid`] with the given [`State`] and view function.
    ///
    /// The view function will be called to display each [`Pane`] present in the
    /// [`State`].
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    /// [`State`]: struct.State.html
    /// [`Pane`]: struct.Pane.html
    pub fn new<T>(
        state: &'a mut State<T>,
        view: impl Fn(
            Pane,
            &'a mut T,
            Option<Focus>,
        ) -> Content<'a, Message, Renderer>,
    ) -> Self {
        let elements = {
            let action = state.internal.action();
            let current_focus = action.focus();

            state
                .panes
                .iter_mut()
                .map(move |(pane, pane_state)| {
                    let focus = match current_focus {
                        Some((focused_pane, focus))
                            if *pane == focused_pane =>
                        {
                            Some(focus)
                        }
                        _ => None,
                    };

                    (*pane, view(*pane, pane_state, focus))
                })
                .collect()
        };

        Self {
            state: &mut state.internal,
            elements,
            width: Length::Fill,
            height: Length::Fill,
            spacing: 0,
            modifier_keys: keyboard::ModifiersState {
                control: true,
                ..Default::default()
            },
            on_drag: None,
            on_resize: None,
            on_key_press: None,
        }
    }

    /// Sets the width of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the spacing _between_ the panes of the [`PaneGrid`].
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the modifier keys of the [`PaneGrid`].
    ///
    /// The modifier keys will need to be pressed to trigger key events.
    ///
    /// The default modifier key is `Ctrl`.
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    pub fn modifier_keys(
        mut self,
        modifier_keys: keyboard::ModifiersState,
    ) -> Self {
        self.modifier_keys = modifier_keys;
        self
    }

    /// Enables the drag and drop interactions of the [`PaneGrid`], which will
    /// use the provided function to produce messages.
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
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
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    pub fn on_resize<F>(mut self, leeway: u16, f: F) -> Self
    where
        F: 'a + Fn(ResizeEvent) -> Message,
    {
        self.on_resize = Some((leeway, Box::new(f)));
        self
    }

    /// Captures hotkey interactions with the [`PaneGrid`], using the provided
    /// function to produce messages.
    ///
    /// The function will be called when:
    ///   - a [`Pane`] is focused
    ///   - a key is pressed
    ///   - all the modifier keys are pressed
    ///
    /// If the function returns `None`, the key press event will be discarded
    /// without producing any message.
    ///
    /// This method is particularly useful to implement hotkey interactions.
    /// For instance, you can use it to enable splitting, swapping, or resizing
    /// panes by pressing combinations of keys.
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    /// [`Pane`]: struct.Pane.html
    pub fn on_key_press<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(KeyPressEvent) -> Option<Message>,
    {
        self.on_key_press = Some(Box::new(f));
        self
    }
}

impl<'a, Message, Renderer> PaneGrid<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn click_pane(
        &mut self,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        let mut clicked_region =
            self.elements.iter().zip(layout.children()).filter(
                |(_, layout)| layout.bounds().contains(cursor_position),
            );

        if let Some(((pane, content), layout)) = clicked_region.next() {
            match &self.on_drag {
                Some(on_drag) => {
                    if content.can_be_picked_at(layout, cursor_position) {
                        let pane_position = layout.position();

                        let origin = cursor_position
                            - Vector::new(pane_position.x, pane_position.y);

                        self.state.pick_pane(pane, origin);

                        messages
                            .push(on_drag(DragEvent::Picked { pane: *pane }));
                    } else {
                        self.state.focus(pane);
                    }
                }
                None => {
                    self.state.focus(pane);
                }
            }
        }
    }

    fn trigger_resize(
        &mut self,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        if let Some((_, on_resize)) = &self.on_resize {
            if let Some((split, _)) = self.state.picked_split() {
                let bounds = layout.bounds();

                let splits = self.state.split_regions(
                    f32::from(self.spacing),
                    Size::new(bounds.width, bounds.height),
                );

                if let Some((axis, rectangle, _)) = splits.get(&split) {
                    let ratio = match axis {
                        Axis::Horizontal => {
                            let position =
                                cursor_position.y - bounds.y - rectangle.y;

                            (position / rectangle.height).max(0.1).min(0.9)
                        }
                        Axis::Vertical => {
                            let position =
                                cursor_position.x - bounds.x - rectangle.x;

                            (position / rectangle.width).max(0.1).min(0.9)
                        }
                    };

                    messages.push(on_resize(ResizeEvent { split, ratio }));
                }
            }
        }
    }
}

/// An event produced during a drag and drop interaction of a [`PaneGrid`].
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone, Copy)]
pub enum DragEvent {
    /// A [`Pane`] was picked for dragging.
    ///
    /// [`Pane`]: struct.Pane.html
    Picked {
        /// The picked [`Pane`].
        ///
        /// [`Pane`]: struct.Pane.html
        pane: Pane,
    },

    /// A [`Pane`] was dropped on top of another [`Pane`].
    ///
    /// [`Pane`]: struct.Pane.html
    Dropped {
        /// The picked [`Pane`].
        ///
        /// [`Pane`]: struct.Pane.html
        pane: Pane,

        /// The [`Pane`] where the picked one was dropped on.
        ///
        /// [`Pane`]: struct.Pane.html
        target: Pane,
    },

    /// A [`Pane`] was picked and then dropped outside of other [`Pane`]
    /// boundaries.
    ///
    /// [`Pane`]: struct.Pane.html
    Canceled {
        /// The picked [`Pane`].
        ///
        /// [`Pane`]: struct.Pane.html
        pane: Pane,
    },
}

/// An event produced during a resize interaction of a [`PaneGrid`].
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone, Copy)]
pub struct ResizeEvent {
    /// The [`Split`] that is being dragged for resizing.
    ///
    /// [`Split`]: struct.Split.html
    pub split: Split,

    /// The new ratio of the [`Split`].
    ///
    /// The ratio is a value in [0, 1], representing the exact position of a
    /// [`Split`] between two panes.
    ///
    /// [`Split`]: struct.Split.html
    pub ratio: f32,
}

/// An event produced during a key press interaction of a [`PaneGrid`].
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone, Copy)]
pub struct KeyPressEvent {
    /// The key that was pressed.
    pub key_code: keyboard::KeyCode,

    /// The state of the modifier keys when the key was pressed.
    pub modifiers: keyboard::ModifiersState,
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for PaneGrid<'a, Message, Renderer>
where
    Renderer: self::Renderer + container::Renderer,
{
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
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        let regions = self.state.pane_regions(f32::from(self.spacing), size);

        let children = self
            .elements
            .iter()
            .filter_map(|(pane, element)| {
                let region = regions.get(pane)?;
                let size = Size::new(region.width, region.height);

                let mut node =
                    element.layout(renderer, &layout::Limits::new(size, size));

                node.move_to(Point::new(region.x, region.y));

                Some(node)
            })
            .collect();

        layout::Node::with_children(size, children)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        match self.on_resize {
                            Some((leeway, _)) => {
                                let relative_cursor = Point::new(
                                    cursor_position.x - bounds.x,
                                    cursor_position.y - bounds.y,
                                );

                                let splits = self.state.split_regions(
                                    f32::from(self.spacing),
                                    Size::new(bounds.width, bounds.height),
                                );

                                let clicked_split = hovered_split(
                                    splits.iter(),
                                    f32::from(self.spacing + leeway),
                                    relative_cursor,
                                );

                                if let Some((split, axis)) = clicked_split {
                                    self.state.pick_split(&split, axis);
                                } else {
                                    self.click_pane(
                                        layout,
                                        cursor_position,
                                        messages,
                                    );
                                }
                            }
                            None => {
                                self.click_pane(
                                    layout,
                                    cursor_position,
                                    messages,
                                );
                            }
                        }
                    } else {
                        // TODO: Encode cursor availability in the type system
                        if cursor_position.x > 0.0 && cursor_position.y > 0.0 {
                            self.state.unfocus();
                        }
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if let Some((pane, _)) = self.state.picked_pane() {
                        self.state.focus(&pane);

                        if let Some(on_drag) = &self.on_drag {
                            let mut dropped_region = self
                                .elements
                                .iter()
                                .zip(layout.children())
                                .filter(|(_, layout)| {
                                    layout.bounds().contains(cursor_position)
                                });

                            let event = match dropped_region.next() {
                                Some(((target, _), _)) if pane != *target => {
                                    DragEvent::Dropped {
                                        pane,
                                        target: *target,
                                    }
                                }
                                _ => DragEvent::Canceled { pane },
                            };

                            messages.push(on_drag(event));
                        }
                    } else if self.state.picked_split().is_some() {
                        self.state.drop_split();
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    self.trigger_resize(layout, cursor_position, messages);
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => {
                match keyboard_event {
                    keyboard::Event::KeyPressed {
                        modifiers,
                        key_code,
                    } => {
                        if let Some(on_key_press) = &self.on_key_press {
                            // TODO: Discard when event is captured
                            if let Some(_) = self.state.active_pane() {
                                if modifiers.matches(self.modifier_keys) {
                                    if let Some(message) =
                                        on_key_press(KeyPressEvent {
                                            key_code,
                                            modifiers,
                                        })
                                    {
                                        messages.push(message);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if self.state.picked_pane().is_none() {
            {
                self.elements.iter_mut().zip(layout.children()).for_each(
                    |((_, pane), layout)| {
                        pane.on_event(
                            event.clone(),
                            layout,
                            cursor_position,
                            messages,
                            renderer,
                            clipboard,
                        )
                    },
                );
            }
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        let picked_split = self
            .state
            .picked_split()
            .or_else(|| match self.on_resize {
                Some((leeway, _)) => {
                    let bounds = layout.bounds();

                    let relative_cursor = Point::new(
                        cursor_position.x - bounds.x,
                        cursor_position.y - bounds.y,
                    );

                    let splits = self
                        .state
                        .split_regions(f32::from(self.spacing), bounds.size());

                    hovered_split(
                        splits.iter(),
                        f32::from(self.spacing + leeway),
                        relative_cursor,
                    )
                }
                None => None,
            })
            .map(|(_, axis)| axis);

        self::Renderer::draw(
            renderer,
            defaults,
            &self.elements,
            self.state.picked_pane(),
            picked_split,
            layout,
            cursor_position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.state.hash_layout(state);

        for (_, element) in &self.elements {
            element.hash_layout(state);
        }
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.elements
            .iter_mut()
            .zip(layout.children())
            .filter_map(|((_, pane), layout)| pane.overlay(layout))
            .next()
    }
}

/// The renderer of a [`PaneGrid`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`PaneGrid`] in your user interface.
///
/// [`PaneGrid`]: struct.PaneGrid.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer:
    crate::Renderer + container::Renderer + text::Renderer + Sized
{
    /// Draws a [`PaneGrid`].
    ///
    /// It receives:
    /// - the elements of the [`PaneGrid`]
    /// - the [`Pane`] that is currently being dragged
    /// - the [`Axis`] that is currently being resized
    /// - the [`Layout`] of the [`PaneGrid`] and its elements
    /// - the cursor position
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    /// [`Pane`]: struct.Pane.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Pane, Content<'_, Message, Self>)],
        dragging: Option<(Pane, Point)>,
        resizing: Option<Axis>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output;

    /// Draws a [`Pane`].
    ///
    /// It receives:
    /// - the [`TitleBar`] of the [`Pane`], if any
    /// - the [`Content`] of the [`Pane`]
    /// - the [`Layout`] of the [`Pane`] and its elements
    /// - the cursor position
    ///
    /// [`Pane`]: struct.Pane.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw_pane<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style: &Self::Style,
        title_bar: Option<(&TitleBar<'_, Message, Self>, Layout<'_>)>,
        body: (&Element<'_, Message, Self>, Layout<'_>),
        cursor_position: Point,
    ) -> Self::Output;

    /// Draws a [`TitleBar`].
    ///
    /// It receives:
    /// - the bounds, style of the [`TitleBar`]
    /// - the style of the [`TitleBar`]
    /// - the title of the [`TitleBar`] with its size, font, and bounds
    /// - the controls of the [`TitleBar`] with their [`Layout`+, if any
    /// - the cursor position
    ///
    /// [`TitleBar`]: struct.TitleBar.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn draw_title_bar<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        style: &Self::Style,
        title: &str,
        title_size: u16,
        title_font: Self::Font,
        title_bounds: Rectangle,
        controls: Option<(&Element<'_, Message, Self>, Layout<'_>)>,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<PaneGrid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + row::Renderer,
    Message: 'a,
{
    fn from(
        pane_grid: PaneGrid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(pane_grid)
    }
}

/*
 * Helpers
 */
fn hovered_split<'a>(
    splits: impl Iterator<Item = (&'a Split, &'a (Axis, Rectangle, f32))>,
    spacing: f32,
    cursor_position: Point,
) -> Option<(Split, Axis)> {
    splits
        .filter_map(|(split, (axis, region, ratio))| {
            let bounds =
                axis.split_line_bounds(*region, *ratio, f32::from(spacing));

            if bounds.contains(cursor_position) {
                Some((*split, *axis))
            } else {
                None
            }
        })
        .next()
}
