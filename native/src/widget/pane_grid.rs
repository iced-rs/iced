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
mod direction;
mod node;
mod pane;
mod split;
mod state;

pub use axis::Axis;
pub use direction::Direction;
pub use pane::Pane;
pub use split::Split;
pub use state::{Focus, State};

use crate::{
    input::{keyboard, mouse, ButtonState},
    layout, Clipboard, Element, Event, Hasher, Layout, Length, Point, Size,
    Widget,
};

/// A collection of panes distributed using either vertical or horizontal splits
/// to completely fill the space available.
///
/// [![Pane grid - Iced](https://thumbs.gfycat.com/MixedFlatJellyfish-small.gif)](https://gfycat.com/mixedflatjellyfish)
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
///         match state {
///             PaneState::SomePane => Text::new("This is some pane"),
///             PaneState::AnotherKindOfPane => Text::new("This is another kind of pane"),
///         }.into()
///     })
///     .on_drag(Message::PaneDragged)
///     .on_resize(Message::PaneResized);
/// ```
///
/// [`PaneGrid`]: struct.PaneGrid.html
/// [`State`]: struct.State.html
#[allow(missing_debug_implementations)]
pub struct PaneGrid<'a, Message, Renderer> {
    state: &'a mut state::Internal,
    pressed_modifiers: &'a mut keyboard::ModifiersState,
    elements: Vec<(Pane, Element<'a, Message, Renderer>)>,
    width: Length,
    height: Length,
    spacing: u16,
    modifier_keys: keyboard::ModifiersState,
    on_drag: Option<Box<dyn Fn(DragEvent) -> Message + 'a>>,
    on_resize: Option<Box<dyn Fn(ResizeEvent) -> Message + 'a>>,
    on_key_press: Option<Box<dyn Fn(KeyPressEvent) -> Option<Message> + 'a>>,
}

impl<'a, Message, Renderer> PaneGrid<'a, Message, Renderer> {
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
        ) -> Element<'a, Message, Renderer>,
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
            pressed_modifiers: &mut state.modifiers,
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
    /// The modifier keys will need to be pressed to trigger dragging, resizing,
    /// and key events.
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
    /// Panes can be dragged using `Modifier keys + Left click`.
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
    /// Panes can be resized using `Modifier keys + Right click`.
    ///
    /// [`PaneGrid`]: struct.PaneGrid.html
    pub fn on_resize<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(ResizeEvent) -> Message,
    {
        self.on_resize = Some(Box::new(f));
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

    fn trigger_resize(
        &mut self,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        if let Some(on_resize) = &self.on_resize {
            if let Some((split, _)) = self.state.picked_split() {
                let bounds = layout.bounds();

                let splits = self.state.splits(
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
    Renderer: 'static + self::Renderer,
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

        let regions = self.state.regions(f32::from(self.spacing), size);

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
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state,
            }) => match state {
                ButtonState::Pressed => {
                    let mut clicked_region =
                        self.elements.iter().zip(layout.children()).filter(
                            |(_, layout)| {
                                layout.bounds().contains(cursor_position)
                            },
                        );

                    if let Some(((pane, _), _)) = clicked_region.next() {
                        match &self.on_drag {
                            Some(on_drag)
                                if self
                                    .pressed_modifiers
                                    .matches(self.modifier_keys) =>
                            {
                                self.state.pick_pane(pane);

                                messages.push(on_drag(DragEvent::Picked {
                                    pane: *pane,
                                }));
                            }
                            _ => {
                                self.state.focus(pane);
                            }
                        }
                    } else {
                        self.state.unfocus();
                    }
                }
                ButtonState::Released => {
                    if let Some(pane) = self.state.picked_pane() {
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
                    }
                }
            },
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Right,
                state: ButtonState::Pressed,
            }) if self.on_resize.is_some()
                && self.state.picked_pane().is_none()
                && self.pressed_modifiers.matches(self.modifier_keys) =>
            {
                let bounds = layout.bounds();

                if bounds.contains(cursor_position) {
                    let relative_cursor = Point::new(
                        cursor_position.x - bounds.x,
                        cursor_position.y - bounds.y,
                    );

                    let splits = self.state.splits(
                        f32::from(self.spacing),
                        Size::new(bounds.width, bounds.height),
                    );

                    let mut sorted_splits: Vec<_> = splits
                        .iter()
                        .filter(|(_, (axis, rectangle, _))| match axis {
                            Axis::Horizontal => {
                                relative_cursor.x > rectangle.x
                                    && relative_cursor.x
                                        < rectangle.x + rectangle.width
                            }
                            Axis::Vertical => {
                                relative_cursor.y > rectangle.y
                                    && relative_cursor.y
                                        < rectangle.y + rectangle.height
                            }
                        })
                        .collect();

                    sorted_splits.sort_by_key(
                        |(_, (axis, rectangle, ratio))| {
                            let distance = match axis {
                                Axis::Horizontal => (relative_cursor.y
                                    - (rectangle.y + rectangle.height * ratio))
                                    .abs(),
                                Axis::Vertical => (relative_cursor.x
                                    - (rectangle.x + rectangle.width * ratio))
                                    .abs(),
                            };

                            distance.round() as u32
                        },
                    );

                    if let Some((split, (axis, _, _))) = sorted_splits.first() {
                        self.state.pick_split(split, *axis);
                        self.trigger_resize(layout, cursor_position, messages);
                    }
                }
            }
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Right,
                state: ButtonState::Released,
            }) if self.state.picked_split().is_some() => {
                self.state.drop_split();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                self.trigger_resize(layout, cursor_position, messages);
            }
            Event::Keyboard(keyboard::Event::Input {
                modifiers,
                key_code,
                state,
            }) => {
                if let Some(on_key_press) = &self.on_key_press {
                    // TODO: Discard when event is captured
                    if state == ButtonState::Pressed {
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

                *self.pressed_modifiers = modifiers;
            }
            _ => {}
        }

        if self.state.picked_pane().is_none() {
            {
                self.elements.iter_mut().zip(layout.children()).for_each(
                    |((_, pane), layout)| {
                        pane.widget.on_event(
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
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            &self.elements,
            self.state.picked_pane(),
            self.state.picked_split().map(|(_, axis)| axis),
            layout,
            cursor_position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        std::any::TypeId::of::<PaneGrid<'_, (), Renderer>>().hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.state.hash_layout(state);

        for (_, element) in &self.elements {
            element.hash_layout(state);
        }
    }
}

/// The renderer of a [`PaneGrid`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`PaneGrid`] in your user interface.
///
/// [`PaneGrid`]: struct.PaneGrid.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
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
        content: &[(Pane, Element<'_, Message, Self>)],
        dragging: Option<Pane>,
        resizing: Option<Axis>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<PaneGrid<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
    Message: 'a,
{
    fn from(
        pane_grid: PaneGrid<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(pane_grid)
    }
}
