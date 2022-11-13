//! Implement your own event loop to drive a user interface.
use crate::application;
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::{Clipboard, Element, Layout, Point, Rectangle, Shell, Size};

/// A set of interactive graphical elements with a specific [`Layout`].
///
/// It can be updated and drawn.
///
/// Iced tries to avoid dictating how to write your event loop. You are in
/// charge of using this type in your system in any way you want.
///
/// # Example
/// The [`integration_opengl`] & [`integration_wgpu`] examples use a
/// [`UserInterface`] to integrate Iced in an existing graphical application.
///
/// [`integration_opengl`]: https://github.com/iced-rs/iced/tree/0.5/examples/integration_opengl
/// [`integration_wgpu`]: https://github.com/iced-rs/iced/tree/0.5/examples/integration_wgpu
#[allow(missing_debug_implementations)]
pub struct UserInterface<'a, Message, Renderer> {
    root: Element<'a, Message, Renderer>,
    base: layout::Node,
    state: widget::Tree,
    overlay: Option<layout::Node>,
    bounds: Size,
}

impl<'a, Message, Renderer> UserInterface<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: application::StyleSheet,
{
    /// Builds a user interface for an [`Element`].
    ///
    /// It is able to avoid expensive computations when using a [`Cache`]
    /// obtained from a previous instance of a [`UserInterface`].
    ///
    /// # Example
    /// Imagine we want to build a [`UserInterface`] for
    /// [the counter example that we previously wrote](index.html#usage). Here
    /// is naive way to set up our application loop:
    ///
    /// ```no_run
    /// use iced_native::Size;
    /// use iced_native::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub use iced_native::renderer::Null as Renderer;
    /// # }
    /// #
    /// # use iced_native::widget::Column;
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> Column<(), Renderer> {
    /// #         Column::new()
    /// #     }
    /// # }
    /// // Initialization
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::new();
    /// let mut window_size = Size::new(1024.0, 768.0);
    ///
    /// // Application loop
    /// loop {
    ///     // Process system events here...
    ///
    ///     // Build the user interface
    ///     let user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     // Update and draw the user interface here...
    ///     // ...
    ///
    ///     // Obtain the cache for the next iteration
    ///     cache = user_interface.into_cache();
    /// }
    /// ```
    pub fn build<E: Into<Element<'a, Message, Renderer>>>(
        root: E,
        bounds: Size,
        cache: Cache,
        renderer: &mut Renderer,
    ) -> Self {
        let root = root.into();

        let Cache { mut state } = cache;
        state.diff(root.as_widget());

        let base =
            renderer.layout(&root, &layout::Limits::new(Size::ZERO, bounds));

        UserInterface {
            root,
            base,
            state,
            overlay: None,
            bounds,
        }
    }

    /// Updates the [`UserInterface`] by processing each provided [`Event`].
    ///
    /// It returns __messages__ that may have been produced as a result of user
    /// interactions. You should feed these to your __update logic__.
    ///
    /// # Example
    /// Let's allow our [counter](index.html#usage) to change state by
    /// completing [the previous example](#example):
    ///
    /// ```no_run
    /// use iced_native::{clipboard, Size, Point};
    /// use iced_native::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub use iced_native::renderer::Null as Renderer;
    /// # }
    /// #
    /// # use iced_native::widget::Column;
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> Column<(), Renderer> {
    /// #         Column::new()
    /// #     }
    /// #     pub fn update(&mut self, message: ()) {}
    /// # }
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::new();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor_position = Point::default();
    /// let mut clipboard = clipboard::Null;
    ///
    /// // Initialize our event storage
    /// let mut events = Vec::new();
    /// let mut messages = Vec::new();
    ///
    /// loop {
    ///     // Obtain system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     // Update the user interface
    ///     let (state, event_statuses) = user_interface.update(
    ///         &events,
    ///         cursor_position,
    ///         &mut renderer,
    ///         &mut clipboard,
    ///         &mut messages
    ///     );
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     // Process the produced messages
    ///     for message in messages.drain(..) {
    ///         counter.update(message);
    ///     }
    /// }
    /// ```
    pub fn update(
        &mut self,
        events: &[Event],
        cursor_position: Point,
        renderer: &mut Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> (State, Vec<event::Status>) {
        use std::mem::ManuallyDrop;

        let mut state = State::Updated;
        let mut manual_overlay =
            ManuallyDrop::new(self.root.as_widget().overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
            ));

        let (base_cursor, overlay_statuses) = if manual_overlay.is_some() {
            let bounds = self.bounds;

            let mut overlay = manual_overlay.as_mut().unwrap();
            let mut layout = overlay.layout(renderer, bounds);
            let mut event_statuses = Vec::new();

            for event in events.iter().cloned() {
                let mut shell = Shell::new(messages);

                let event_status = overlay.on_event(
                    event,
                    Layout::new(&layout),
                    cursor_position,
                    renderer,
                    clipboard,
                    &mut shell,
                );

                event_statuses.push(event_status);

                if shell.is_layout_invalid() {
                    let _ = ManuallyDrop::into_inner(manual_overlay);

                    self.base = renderer.layout(
                        &self.root,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    manual_overlay =
                        ManuallyDrop::new(self.root.as_widget().overlay(
                            &mut self.state,
                            Layout::new(&self.base),
                            renderer,
                        ));

                    if manual_overlay.is_none() {
                        break;
                    }

                    overlay = manual_overlay.as_mut().unwrap();

                    shell.revalidate_layout(|| {
                        layout = overlay.layout(renderer, bounds);
                    });
                }

                if shell.are_widgets_invalid() {
                    state = State::Outdated;
                }
            }

            let base_cursor = if layout.bounds().contains(cursor_position) {
                // TODO: Type-safe cursor availability
                Point::new(-1.0, -1.0)
            } else {
                cursor_position
            };

            self.overlay = Some(layout);

            (base_cursor, event_statuses)
        } else {
            (cursor_position, vec![event::Status::Ignored; events.len()])
        };

        let _ = ManuallyDrop::into_inner(manual_overlay);

        let event_statuses = events
            .iter()
            .cloned()
            .zip(overlay_statuses.into_iter())
            .map(|(event, overlay_status)| {
                if matches!(overlay_status, event::Status::Captured) {
                    return overlay_status;
                }

                let mut shell = Shell::new(messages);

                let event_status = self.root.as_widget_mut().on_event(
                    &mut self.state,
                    event,
                    Layout::new(&self.base),
                    base_cursor,
                    renderer,
                    clipboard,
                    &mut shell,
                );

                if matches!(event_status, event::Status::Captured) {
                    self.overlay = None;
                }

                shell.revalidate_layout(|| {
                    self.base = renderer.layout(
                        &self.root,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    self.overlay = None;
                });

                if shell.are_widgets_invalid() {
                    state = State::Outdated;
                }

                event_status.merge(overlay_status)
            })
            .collect();

        (state, event_statuses)
    }

    /// Draws the [`UserInterface`] with the provided [`Renderer`].
    ///
    /// It returns the current [`mouse::Interaction`]. You should update the
    /// icon of the mouse cursor accordingly in your system.
    ///
    /// [`Renderer`]: crate::Renderer
    ///
    /// # Example
    /// We can finally draw our [counter](index.html#usage) by
    /// [completing the last example](#example-1):
    ///
    /// ```no_run
    /// use iced_native::clipboard;
    /// use iced_native::renderer;
    /// use iced_native::user_interface::{self, UserInterface};
    /// use iced_native::{Size, Point, Theme};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub use iced_native::renderer::Null as Renderer;
    /// # }
    /// #
    /// # use iced_native::widget::Column;
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> Column<(), Renderer> {
    /// #         Column::new()
    /// #     }
    /// #     pub fn update(&mut self, message: ()) {}
    /// # }
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::new();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor_position = Point::default();
    /// let mut clipboard = clipboard::Null;
    /// let mut events = Vec::new();
    /// let mut messages = Vec::new();
    ///
    /// loop {
    ///     // Obtain system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     // Update the user interface
    ///     let event_statuses = user_interface.update(
    ///         &events,
    ///         cursor_position,
    ///         &mut renderer,
    ///         &mut clipboard,
    ///         &mut messages
    ///     );
    ///
    ///     // Draw the user interface
    ///     let mouse_cursor = user_interface.draw(&mut renderer, &Theme::default(), &renderer::Style::default(), cursor_position);
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     for message in messages.drain(..) {
    ///         counter.update(message);
    ///     }
    ///
    ///     // Update mouse cursor icon...
    ///     // Flush rendering operations...
    /// }
    /// ```
    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        cursor_position: Point,
    ) -> mouse::Interaction {
        // TODO: Move to shell level (?)
        renderer.clear();

        let viewport = Rectangle::with_size(self.bounds);

        let base_cursor = if let Some(overlay) = self.root.as_widget().overlay(
            &mut self.state,
            Layout::new(&self.base),
            renderer,
        ) {
            let overlay_layout = self
                .overlay
                .take()
                .unwrap_or_else(|| overlay.layout(renderer, self.bounds));

            let new_cursor_position =
                if overlay_layout.bounds().contains(cursor_position) {
                    Point::new(-1.0, -1.0)
                } else {
                    cursor_position
                };

            self.overlay = Some(overlay_layout);

            new_cursor_position
        } else {
            cursor_position
        };

        self.root.as_widget().draw(
            &self.state,
            renderer,
            theme,
            style,
            Layout::new(&self.base),
            base_cursor,
            &viewport,
        );

        let base_interaction = self.root.as_widget().mouse_interaction(
            &self.state,
            Layout::new(&self.base),
            cursor_position,
            &viewport,
            renderer,
        );

        let Self {
            overlay,
            root,
            base,
            ..
        } = self;

        // TODO: Currently, we need to call Widget::overlay twice to
        // implement the painter's algorithm properly.
        //
        // Once we have a proper persistent widget tree, we should be able to
        // avoid this additional call.
        overlay
            .as_ref()
            .and_then(|layout| {
                root.as_widget()
                    .overlay(&mut self.state, Layout::new(base), renderer)
                    .map(|overlay| {
                        let overlay_interaction = overlay.mouse_interaction(
                            Layout::new(layout),
                            cursor_position,
                            &viewport,
                            renderer,
                        );

                        let overlay_bounds = layout.bounds();

                        renderer.with_layer(overlay_bounds, |renderer| {
                            overlay.draw(
                                renderer,
                                theme,
                                style,
                                Layout::new(layout),
                                cursor_position,
                            );
                        });

                        if overlay_bounds.contains(cursor_position) {
                            overlay_interaction
                        } else {
                            base_interaction
                        }
                    })
            })
            .unwrap_or(base_interaction)
    }

    /// Applies a [`widget::Operation`] to the [`UserInterface`].
    pub fn operate(
        &mut self,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.root.as_widget().operate(
            &mut self.state,
            Layout::new(&self.base),
            operation,
        );

        if let Some(layout) = self.overlay.as_ref() {
            if let Some(overlay) = self.root.as_widget().overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
            ) {
                overlay.operate(Layout::new(layout), operation);
            }
        }
    }

    /// Relayouts and returns a new  [`UserInterface`] using the provided
    /// bounds.
    pub fn relayout(self, bounds: Size, renderer: &mut Renderer) -> Self {
        Self::build(self.root, bounds, Cache { state: self.state }, renderer)
    }

    /// Extract the [`Cache`] of the [`UserInterface`], consuming it in the
    /// process.
    pub fn into_cache(self) -> Cache {
        Cache { state: self.state }
    }
}

/// Reusable data of a specific [`UserInterface`].
#[derive(Debug)]
pub struct Cache {
    state: widget::Tree,
}

impl Cache {
    /// Creates an empty [`Cache`].
    ///
    /// You should use this to initialize a [`Cache`] before building your first
    /// [`UserInterface`].
    pub fn new() -> Cache {
        Cache {
            state: widget::Tree::empty(),
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache::new()
    }
}

/// The resulting state after updating a [`UserInterface`].
#[derive(Debug, Clone, Copy)]
pub enum State {
    /// The [`UserInterface`] is outdated and needs to be rebuilt.
    Outdated,

    /// The [`UserInterface`] is up-to-date and can be reused without
    /// rebuilding.
    Updated,
}
