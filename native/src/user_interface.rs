use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::{Clipboard, Element, Layout, Point, Rectangle, Shell, Size};

use std::hash::Hasher;

/// A set of interactive graphical elements with a specific [`Layout`].
///
/// It can be updated and drawn.
///
/// Iced tries to avoid dictating how to write your event loop. You are in
/// charge of using this type in your system in any way you want.
///
/// # Example
/// The [`integration` example] uses a [`UserInterface`] to integrate Iced in
/// an existing graphical application.
///
/// [`integration` example]: https://github.com/hecrj/iced/tree/0.3/examples/integration
#[allow(missing_debug_implementations)]
pub struct UserInterface<'a, Message, Renderer> {
    root: Element<'a, Message, Renderer>,
    base: Layer,
    overlay: Option<Layer>,
    bounds: Size,
}

impl<'a, Message, Renderer> UserInterface<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
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
    /// use iced_native::{UserInterface, Cache, Size};
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
    /// let mut cache = Cache::new();
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

        let (base, overlay) = {
            let hash = {
                let hasher = &mut crate::Hasher::default();
                root.hash_layout(hasher);

                hasher.finish()
            };

            let layout_is_cached =
                hash == cache.base.hash && bounds == cache.bounds;

            let (layout, overlay) = if layout_is_cached {
                (cache.base.layout, cache.overlay)
            } else {
                (
                    renderer.layout(
                        &root,
                        &layout::Limits::new(Size::ZERO, bounds),
                    ),
                    None,
                )
            };

            (Layer { layout, hash }, overlay)
        };

        UserInterface {
            root,
            base,
            overlay,
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
    /// use iced_native::{clipboard, UserInterface, Cache, Size, Point};
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
    /// let mut cache = Cache::new();
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
    ///     let event_statuses = user_interface.update(
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
    ) -> Vec<event::Status> {
        let (base_cursor, overlay_statuses) = if let Some(mut overlay) =
            self.root.overlay(Layout::new(&self.base.layout))
        {
            let bounds = self.bounds;

            let mut layer = Self::overlay_layer(
                self.overlay.take(),
                bounds,
                &mut overlay,
                renderer,
            );

            let event_statuses = events
                .iter()
                .cloned()
                .map(|event| {
                    let mut shell = Shell::new(messages);

                    let event_status = overlay.on_event(
                        event,
                        Layout::new(&layer.layout),
                        cursor_position,
                        renderer,
                        clipboard,
                        &mut shell,
                    );

                    shell.with_invalid_layout(|| {
                        layer = Self::overlay_layer(
                            None,
                            bounds,
                            &mut overlay,
                            renderer,
                        );
                    });

                    event_status
                })
                .collect();

            let base_cursor = if layer.layout.bounds().contains(cursor_position)
            {
                // TODO: Type-safe cursor availability
                Point::new(-1.0, -1.0)
            } else {
                cursor_position
            };

            self.overlay = Some(layer);

            (base_cursor, event_statuses)
        } else {
            (cursor_position, vec![event::Status::Ignored; events.len()])
        };

        events
            .iter()
            .cloned()
            .zip(overlay_statuses.into_iter())
            .map(|(event, overlay_status)| {
                let mut shell = Shell::new(messages);

                let event_status = self.root.widget.on_event(
                    event,
                    Layout::new(&self.base.layout),
                    base_cursor,
                    renderer,
                    clipboard,
                    &mut shell,
                );

                shell.with_invalid_layout(|| {
                    let hash = {
                        let hasher = &mut crate::Hasher::default();
                        self.root.hash_layout(hasher);

                        hasher.finish()
                    };

                    let layout = renderer.layout(
                        &self.root,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    self.base = Layer { layout, hash };
                    self.overlay = None;
                });

                event_status.merge(overlay_status)
            })
            .collect()
    }

    /// Draws the [`UserInterface`] with the provided [`Renderer`].
    ///
    /// It returns the some [`Renderer::Output`]. You should update the icon of
    /// the mouse cursor accordingly in your system.
    ///
    /// [`Renderer`]: crate::Renderer
    /// [`Renderer::Output`]: crate::Renderer::Output
    ///
    /// # Example
    /// We can finally draw our [counter](index.html#usage) by
    /// [completing the last example](#example-1):
    ///
    /// ```no_run
    /// use iced_native::{clipboard, UserInterface, Cache, Size, Point};
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
    /// let mut cache = Cache::new();
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
    ///     let mouse_cursor = user_interface.draw(&mut renderer, cursor_position);
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
        cursor_position: Point,
    ) -> mouse::Interaction {
        // TODO: Move to shell level (?)
        renderer.clear();

        let viewport = Rectangle::with_size(self.bounds);

        if let Some(mut overlay) =
            self.root.overlay(Layout::new(&self.base.layout))
        {
            let layer = Self::overlay_layer(
                self.overlay.take(),
                self.bounds,
                &mut overlay,
                renderer,
            );

            self.overlay = Some(layer);
        };

        if let Some(layer) = &self.overlay {
            let base_cursor = if layer.layout.bounds().contains(cursor_position)
            {
                Point::new(-1.0, -1.0)
            } else {
                cursor_position
            };

            self.root.widget.draw(
                renderer,
                &renderer::Style::default(),
                Layout::new(&self.base.layout),
                base_cursor,
                &viewport,
            );
        } else {
            self.root.widget.draw(
                renderer,
                &renderer::Style::default(),
                Layout::new(&self.base.layout),
                cursor_position,
                &viewport,
            );
        };

        let base_interaction = self.root.widget.mouse_interaction(
            Layout::new(&self.base.layout),
            cursor_position,
            &viewport,
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
            .and_then(|layer| {
                root.overlay(Layout::new(&base.layout)).map(|overlay| {
                    let overlay_interaction = overlay.mouse_interaction(
                        Layout::new(&layer.layout),
                        cursor_position,
                        &viewport,
                    );

                    let overlay_bounds = layer.layout.bounds();

                    renderer.with_layer(overlay_bounds, |renderer| {
                        overlay.draw(
                            renderer,
                            &renderer::Style::default(),
                            Layout::new(&layer.layout),
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

    /// Relayouts and returns a new  [`UserInterface`] using the provided
    /// bounds.
    pub fn relayout(self, bounds: Size, renderer: &mut Renderer) -> Self {
        Self::build(
            self.root,
            bounds,
            Cache {
                base: self.base,
                overlay: self.overlay,
                bounds: self.bounds,
            },
            renderer,
        )
    }

    /// Extract the [`Cache`] of the [`UserInterface`], consuming it in the
    /// process.
    pub fn into_cache(self) -> Cache {
        Cache {
            base: self.base,
            overlay: self.overlay,
            bounds: self.bounds,
        }
    }

    fn overlay_layer(
        cache: Option<Layer>,
        bounds: Size,
        overlay: &mut overlay::Element<'_, Message, Renderer>,
        renderer: &Renderer,
    ) -> Layer {
        let new_hash = {
            let hasher = &mut crate::Hasher::default();
            overlay.hash_layout(hasher);

            hasher.finish()
        };

        let layout = match cache {
            Some(Layer { hash, layout }) if new_hash == hash => layout,
            _ => overlay.layout(renderer, bounds),
        };

        Layer {
            layout,
            hash: new_hash,
        }
    }
}

#[derive(Debug, Clone)]
struct Layer {
    layout: layout::Node,
    hash: u64,
}

/// Reusable data of a specific [`UserInterface`].
#[derive(Debug, Clone)]
pub struct Cache {
    base: Layer,
    overlay: Option<Layer>,
    bounds: Size,
}

impl Cache {
    /// Creates an empty [`Cache`].
    ///
    /// You should use this to initialize a [`Cache`] before building your first
    /// [`UserInterface`].
    pub fn new() -> Cache {
        Cache {
            base: Layer {
                layout: layout::Node::new(Size::new(0.0, 0.0)),
                hash: 0,
            },
            overlay: None,
            bounds: Size::ZERO,
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache::new()
    }
}
