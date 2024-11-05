//! Implement your own event loop to drive a user interface.
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget;
use crate::core::window;
use crate::core::{Clipboard, Element, Layout, Rectangle, Shell, Size, Vector};
use crate::overlay;

/// A set of interactive graphical elements with a specific [`Layout`].
///
/// It can be updated and drawn.
///
/// Iced tries to avoid dictating how to write your event loop. You are in
/// charge of using this type in your system in any way you want.
///
/// # Example
/// The [`integration`] example uses a [`UserInterface`] to integrate Iced in an
/// existing graphical application.
///
/// [`integration`]: https://github.com/iced-rs/iced/tree/0.13/examples/integration
#[allow(missing_debug_implementations)]
pub struct UserInterface<'a, Message, Theme, Renderer> {
    root: Element<'a, Message, Theme, Renderer>,
    base: layout::Node,
    state: widget::Tree,
    overlay: Option<layout::Node>,
    bounds: Size,
}

impl<'a, Message, Theme, Renderer> UserInterface<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
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
    /// # mod iced_wgpu {
    /// #     pub type Renderer = ();
    /// # }
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> iced_core::Element<(), (), Renderer> { unimplemented!() }
    /// #     pub fn update(&mut self, _: ()) {}
    /// # }
    /// use iced_runtime::core::Size;
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// // Initialization
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
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
    pub fn build<E: Into<Element<'a, Message, Theme, Renderer>>>(
        root: E,
        bounds: Size,
        cache: Cache,
        renderer: &mut Renderer,
    ) -> Self {
        let root = root.into();

        let Cache { mut state } = cache;
        state.diff(root.as_widget());

        let base = root.as_widget().layout(
            &mut state,
            renderer,
            &layout::Limits::new(Size::ZERO, bounds),
        );

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
    /// # mod iced_wgpu {
    /// #     pub type Renderer = ();
    /// # }
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> iced_core::Element<(), (), Renderer> { unimplemented!() }
    /// #     pub fn update(&mut self, _: ()) {}
    /// # }
    /// use iced_runtime::core::clipboard;
    /// use iced_runtime::core::mouse;
    /// use iced_runtime::core::Size;
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor = mouse::Cursor::default();
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
    ///         cursor,
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
        cursor: mouse::Cursor,
        renderer: &mut Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> (State, Vec<event::Status>) {
        use std::mem::ManuallyDrop;

        let mut outdated = false;
        let mut redraw_request = None;

        let mut manual_overlay = ManuallyDrop::new(
            self.root
                .as_widget_mut()
                .overlay(
                    &mut self.state,
                    Layout::new(&self.base),
                    renderer,
                    Vector::ZERO,
                )
                .map(overlay::Nested::new),
        );

        let (base_cursor, overlay_statuses) = if manual_overlay.is_some() {
            let bounds = self.bounds;

            let mut overlay = manual_overlay.as_mut().unwrap();
            let mut layout = overlay.layout(renderer, bounds);
            let mut event_statuses = Vec::new();

            for event in events.iter().cloned() {
                let mut shell = Shell::new(messages);

                overlay.update(
                    event,
                    Layout::new(&layout),
                    cursor,
                    renderer,
                    clipboard,
                    &mut shell,
                );

                event_statuses.push(shell.event_status());

                match (redraw_request, shell.redraw_request()) {
                    (None, Some(at)) => {
                        redraw_request = Some(at);
                    }
                    (Some(current), Some(new)) if new < current => {
                        redraw_request = Some(new);
                    }
                    _ => {}
                }

                if shell.is_layout_invalid() {
                    let _ = ManuallyDrop::into_inner(manual_overlay);

                    self.base = self.root.as_widget().layout(
                        &mut self.state,
                        renderer,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    manual_overlay = ManuallyDrop::new(
                        self.root
                            .as_widget_mut()
                            .overlay(
                                &mut self.state,
                                Layout::new(&self.base),
                                renderer,
                                Vector::ZERO,
                            )
                            .map(overlay::Nested::new),
                    );

                    if manual_overlay.is_none() {
                        break;
                    }

                    overlay = manual_overlay.as_mut().unwrap();

                    shell.revalidate_layout(|| {
                        layout = overlay.layout(renderer, bounds);
                    });
                }

                if shell.are_widgets_invalid() {
                    outdated = true;
                }
            }

            let base_cursor = if manual_overlay
                .as_mut()
                .and_then(|overlay| {
                    cursor.position().map(|cursor_position| {
                        overlay.is_over(
                            Layout::new(&layout),
                            renderer,
                            cursor_position,
                        )
                    })
                })
                .unwrap_or_default()
            {
                mouse::Cursor::Unavailable
            } else {
                cursor
            };

            self.overlay = Some(layout);

            (base_cursor, event_statuses)
        } else {
            (cursor, vec![event::Status::Ignored; events.len()])
        };

        let viewport = Rectangle::with_size(self.bounds);

        let _ = ManuallyDrop::into_inner(manual_overlay);

        let event_statuses = events
            .iter()
            .cloned()
            .zip(overlay_statuses)
            .map(|(event, overlay_status)| {
                if matches!(overlay_status, event::Status::Captured) {
                    return overlay_status;
                }

                let mut shell = Shell::new(messages);

                self.root.as_widget_mut().update(
                    &mut self.state,
                    event,
                    Layout::new(&self.base),
                    base_cursor,
                    renderer,
                    clipboard,
                    &mut shell,
                    &viewport,
                );

                if shell.event_status() == event::Status::Captured {
                    self.overlay = None;
                }

                match (redraw_request, shell.redraw_request()) {
                    (None, Some(at)) => {
                        redraw_request = Some(at);
                    }
                    (Some(current), Some(new)) if new < current => {
                        redraw_request = Some(new);
                    }
                    _ => {}
                }

                shell.revalidate_layout(|| {
                    self.base = self.root.as_widget().layout(
                        &mut self.state,
                        renderer,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    self.overlay = None;
                });

                if shell.are_widgets_invalid() {
                    outdated = true;
                }

                shell.event_status().merge(overlay_status)
            })
            .collect();

        (
            if outdated {
                State::Outdated
            } else {
                State::Updated { redraw_request }
            },
            event_statuses,
        )
    }

    /// Draws the [`UserInterface`] with the provided [`Renderer`].
    ///
    /// It returns the current [`mouse::Interaction`]. You should update the
    /// icon of the mouse cursor accordingly in your system.
    ///
    /// [`Renderer`]: crate::core::Renderer
    ///
    /// # Example
    /// We can finally draw our [counter](index.html#usage) by
    /// [completing the last example](#example-1):
    ///
    /// ```no_run
    /// # mod iced_wgpu {
    /// #     pub type Renderer = ();
    /// #     pub type Theme = ();
    /// # }
    /// #
    /// # pub struct Counter;
    /// #
    /// # impl Counter {
    /// #     pub fn new() -> Self { Counter }
    /// #     pub fn view(&self) -> Element<(), (), Renderer> { unimplemented!() }
    /// #     pub fn update(&mut self, _: ()) {}
    /// # }
    /// use iced_runtime::core::clipboard;
    /// use iced_runtime::core::mouse;
    /// use iced_runtime::core::renderer;
    /// use iced_runtime::core::{Element, Size};
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::{Renderer, Theme};
    ///
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor = mouse::Cursor::default();
    /// let mut clipboard = clipboard::Null;
    /// let mut events = Vec::new();
    /// let mut messages = Vec::new();
    /// let mut theme = Theme::default();
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
    ///         cursor,
    ///         &mut renderer,
    ///         &mut clipboard,
    ///         &mut messages
    ///     );
    ///
    ///     // Draw the user interface
    ///     let mouse_interaction = user_interface.draw(&mut renderer, &theme, &renderer::Style::default(), cursor);
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
        theme: &Theme,
        style: &renderer::Style,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        // TODO: Move to shell level (?)
        renderer.clear();

        let viewport = Rectangle::with_size(self.bounds);

        let base_cursor = if let Some(mut overlay) = self
            .root
            .as_widget_mut()
            .overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
                Vector::ZERO,
            )
            .map(overlay::Nested::new)
        {
            let overlay_layout = self
                .overlay
                .take()
                .unwrap_or_else(|| overlay.layout(renderer, self.bounds));

            let cursor = if cursor
                .position()
                .map(|cursor_position| {
                    overlay.is_over(
                        Layout::new(&overlay_layout),
                        renderer,
                        cursor_position,
                    )
                })
                .unwrap_or_default()
            {
                mouse::Cursor::Unavailable
            } else {
                cursor
            };

            self.overlay = Some(overlay_layout);

            cursor
        } else {
            cursor
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
            base_cursor,
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
                root.as_widget_mut()
                    .overlay(
                        &mut self.state,
                        Layout::new(base),
                        renderer,
                        Vector::ZERO,
                    )
                    .map(overlay::Nested::new)
                    .map(|mut overlay| {
                        let overlay_interaction = overlay.mouse_interaction(
                            Layout::new(layout),
                            cursor,
                            &viewport,
                            renderer,
                        );

                        overlay.draw(
                            renderer,
                            theme,
                            style,
                            Layout::new(layout),
                            cursor,
                        );

                        if cursor
                            .position()
                            .map(|cursor_position| {
                                overlay.is_over(
                                    Layout::new(layout),
                                    renderer,
                                    cursor_position,
                                )
                            })
                            .unwrap_or_default()
                        {
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
        operation: &mut dyn widget::Operation,
    ) {
        self.root.as_widget().operate(
            &mut self.state,
            Layout::new(&self.base),
            renderer,
            operation,
        );

        if let Some(mut overlay) = self
            .root
            .as_widget_mut()
            .overlay(
                &mut self.state,
                Layout::new(&self.base),
                renderer,
                Vector::ZERO,
            )
            .map(overlay::Nested::new)
        {
            if self.overlay.is_none() {
                self.overlay = Some(overlay.layout(renderer, self.bounds));
            }

            overlay.operate(
                Layout::new(self.overlay.as_ref().unwrap()),
                renderer,
                operation,
            );
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
    Updated {
        /// The [`window::RedrawRequest`] when a redraw should be performed.
        redraw_request: Option<window::RedrawRequest>,
    },
}
