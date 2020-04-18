use crate::{layout, Clipboard, Element, Event, Layout, Overlay, Point, Size};

use std::hash::Hasher;

/// A set of interactive graphical elements with a specific [`Layout`].
///
/// It can be updated and drawn.
///
/// Iced tries to avoid dictating how to write your event loop. You are in
/// charge of using this type in your system in any way you want.
///
/// [`Layout`]: struct.Layout.html
///
/// # Example
/// The [`integration` example] uses a [`UserInterface`] to integrate Iced in
/// an existing graphical application.
///
/// [`integration` example]: https://github.com/hecrj/iced/tree/0.1/examples/integration
/// [`UserInterface`]: struct.UserInterface.html
#[allow(missing_debug_implementations)]
pub struct UserInterface<'a, Message, Renderer> {
    base: Layer<Element<'a, Message, Renderer>>,
    overlay: Option<Layer<Overlay<'a, Message, Renderer>>>,
    bounds: Size,
}

struct Layer<T> {
    root: T,
    layout: layout::Node,
    hash: u64,
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
    /// [`Element`]: struct.Element.html
    /// [`Cache`]: struct.Cache.html
    /// [`UserInterface`]: struct.UserInterface.html
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
    /// # use iced_native::Column;
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
        let mut root = root.into();

        let (base, overlay) = {
            let hash = {
                let hasher = &mut crate::Hasher::default();
                root.hash_layout(hasher);

                hasher.finish()
            };

            let layout_is_cached = hash == cache.hash && bounds == cache.bounds;

            let layout = if layout_is_cached {
                cache.layout
            } else {
                renderer.layout(&root, &layout::Limits::new(Size::ZERO, bounds))
            };

            let overlay = root.overlay(Layout::new(&layout));

            (Layer { root, layout, hash }, overlay)
        };

        let overlay = overlay.map(|root| {
            let hash = {
                let hasher = &mut crate::Hasher::default();
                root.hash_layout(hasher);

                hasher.finish()
            };

            let layout = root.layout(&renderer, bounds);

            Layer { root, layout, hash }
        });

        UserInterface {
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
    /// [`UserInterface`]: struct.UserInterface.html
    /// [`Event`]: enum.Event.html
    ///
    /// # Example
    /// Let's allow our [counter](index.html#usage) to change state by
    /// completing [the previous example](#example):
    ///
    /// ```no_run
    /// use iced_native::{UserInterface, Cache, Size, Point};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub use iced_native::renderer::Null as Renderer;
    /// # }
    /// #
    /// # use iced_native::Column;
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
    ///
    /// // Initialize our event storage
    /// let mut events = Vec::new();
    ///
    /// loop {
    ///     // Process system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     // Update the user interface
    ///     let messages = user_interface.update(
    ///         events.drain(..),
    ///         cursor_position,
    ///         None,
    ///         &renderer,
    ///     );
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     // Process the produced messages
    ///     for message in messages {
    ///         counter.update(message);
    ///     }
    /// }
    /// ```
    pub fn update(
        &mut self,
        events: impl IntoIterator<Item = Event>,
        cursor_position: Point,
        clipboard: Option<&dyn Clipboard>,
        renderer: &Renderer,
    ) -> Vec<Message> {
        let mut messages = Vec::new();

        for event in events {
            if let Some(overlay) = &mut self.overlay {
                let base_cursor =
                    if overlay.layout.bounds().contains(cursor_position) {
                        // TODO: Encode cursor availability
                        Point::new(-1.0, -1.0)
                    } else {
                        cursor_position
                    };

                overlay.root.on_event(
                    event.clone(),
                    Layout::new(&overlay.layout),
                    cursor_position,
                    &mut messages,
                    renderer,
                    clipboard,
                );

                self.base.root.widget.on_event(
                    event,
                    Layout::new(&self.base.layout),
                    base_cursor,
                    &mut messages,
                    renderer,
                    clipboard,
                );
            } else {
                self.base.root.widget.on_event(
                    event,
                    Layout::new(&self.base.layout),
                    cursor_position,
                    &mut messages,
                    renderer,
                    clipboard,
                );
            }
        }

        messages
    }

    /// Draws the [`UserInterface`] with the provided [`Renderer`].
    ///
    /// It returns the current state of the [`MouseCursor`]. You should update
    /// the icon of the mouse cursor accordingly in your system.
    ///
    /// [`UserInterface`]: struct.UserInterface.html
    /// [`Renderer`]: trait.Renderer.html
    /// [`MouseCursor`]: enum.MouseCursor.html
    ///
    /// # Example
    /// We can finally draw our [counter](index.html#usage) by
    /// [completing the last example](#example-1):
    ///
    /// ```no_run
    /// use iced_native::{UserInterface, Cache, Size, Point};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub use iced_native::renderer::Null as Renderer;
    /// # }
    /// #
    /// # use iced_native::Column;
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
    /// let mut events = Vec::new();
    ///
    /// loop {
    ///     // Process system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         window_size,
    ///         cache,
    ///         &mut renderer,
    ///     );
    ///
    ///     let messages = user_interface.update(
    ///         events.drain(..),
    ///         cursor_position,
    ///         None,
    ///         &renderer,
    ///     );
    ///
    ///     // Draw the user interface
    ///     let mouse_cursor = user_interface.draw(&mut renderer, cursor_position);
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     for message in messages {
    ///         counter.update(message);
    ///     }
    ///
    ///     // Update mouse cursor icon...
    ///     // Flush rendering operations...
    /// }
    /// ```
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        cursor_position: Point,
    ) -> Renderer::Output {
        if let Some(overlay) = &self.overlay {
            let overlay_bounds = overlay.layout.bounds();

            let base_cursor = if overlay_bounds.contains(cursor_position) {
                Point::new(-1.0, -1.0)
            } else {
                cursor_position
            };

            let base_primitives = self.base.root.widget.draw(
                renderer,
                &Renderer::Defaults::default(),
                Layout::new(&self.base.layout),
                base_cursor,
            );

            let overlay_primitives = overlay.root.draw(
                renderer,
                &Renderer::Defaults::default(),
                Layout::new(&overlay.layout),
                cursor_position,
            );

            renderer.overlay(
                base_primitives,
                overlay_primitives,
                overlay_bounds,
            )
        } else {
            self.base.root.widget.draw(
                renderer,
                &Renderer::Defaults::default(),
                Layout::new(&self.base.layout),
                cursor_position,
            )
        }
    }

    /// Extract the [`Cache`] of the [`UserInterface`], consuming it in the
    /// process.
    ///
    /// [`Cache`]: struct.Cache.html
    /// [`UserInterface`]: struct.UserInterface.html
    pub fn into_cache(self) -> Cache {
        Cache {
            hash: self.base.hash,
            layout: self.base.layout,
            bounds: self.bounds,
        }
    }
}

/// Reusable data of a specific [`UserInterface`].
///
/// [`UserInterface`]: struct.UserInterface.html
#[derive(Debug, Clone)]
pub struct Cache {
    hash: u64,
    layout: layout::Node,
    bounds: Size,
}

impl Cache {
    /// Creates an empty [`Cache`].
    ///
    /// You should use this to initialize a [`Cache`] before building your first
    /// [`UserInterface`].
    ///
    /// [`Cache`]: struct.Cache.html
    /// [`UserInterface`]: struct.UserInterface.html
    pub fn new() -> Cache {
        Cache {
            hash: 0,
            layout: layout::Node::new(Size::new(0.0, 0.0)),
            bounds: Size::ZERO,
        }
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache::new()
    }
}

impl PartialEq for Cache {
    fn eq(&self, other: &Cache) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Cache {}
