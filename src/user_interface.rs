use crate::{input::mouse, Column, Element, Event, Layout, MouseCursor, Point};

use std::hash::Hasher;
use stretch::result;

/// A set of interactive graphical elements with a specific [`Layout`].
///
/// It can be updated and drawn.
///
/// Iced tries to avoid dictating how to write your event loop. You are in
/// charge of using this type in your system in any way you want.
///
/// [`Layout`]: struct.Layout.html
#[derive(Debug)]
pub struct UserInterface<'a, Message, Renderer> {
    hash: u64,
    root: Element<'a, Message, Renderer>,
    layout: result::Layout,
    cursor_position: Point,
}

impl<'a, Message, Renderer> UserInterface<'a, Message, Renderer> {
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
    /// use iced::{UserInterface, Cache};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub struct Renderer;
    /// #
    /// #     impl Renderer {
    /// #         pub fn new() -> Self { Renderer }
    /// #     }
    /// # }
    /// #
    /// # use iced::Column;
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
    ///
    /// // Application loop
    /// loop {
    ///     // Process system events here...
    ///
    ///     // Build the user interface
    ///     let user_interface = UserInterface::build(
    ///         counter.view(),
    ///         cache,
    ///         &renderer,
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
        cache: Cache,
        renderer: &Renderer,
    ) -> Self {
        let root = root.into();

        let hasher = &mut crate::Hasher::default();
        root.hash_layout(hasher);

        let hash = hasher.finish();

        let layout = if hash == cache.hash {
            cache.layout
        } else {
            root.compute_layout(renderer)
        };

        UserInterface {
            hash,
            root,
            layout,
            cursor_position: cache.cursor_position,
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
    /// Let's allow our [counter](index.html#usage) to change state by completing
    /// [the previous example](#example):
    ///
    /// ```no_run
    /// use iced::{UserInterface, Cache};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub struct Renderer;
    /// #
    /// #     impl Renderer {
    /// #         pub fn new() -> Self { Renderer }
    /// #     }
    /// # }
    /// #
    /// # use iced::Column;
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
    ///
    /// // Initialize our event storage
    /// let mut events = Vec::new();
    ///
    /// loop {
    ///     // Process system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         cache,
    ///         &renderer,
    ///     );
    ///
    ///     // Update the user interface
    ///     let messages = user_interface.update(events.drain(..));
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
        events: impl Iterator<Item = Event>,
    ) -> Vec<Message> {
        let mut messages = Vec::new();

        for event in events {
            if let Event::Mouse(mouse::Event::CursorMoved { x, y }) = event {
                self.cursor_position = Point::new(x, y);
            }

            self.root.widget.on_event(
                event,
                Layout::new(&self.layout),
                self.cursor_position,
                &mut messages,
            );
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
    /// use iced::{UserInterface, Cache};
    /// use iced_wgpu::Renderer;
    ///
    /// # mod iced_wgpu {
    /// #     pub struct Renderer;
    /// #
    /// #     impl Renderer {
    /// #         pub fn new() -> Self { Renderer }
    /// #     }
    /// # }
    /// #
    /// # use iced::Column;
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
    /// let mut events = Vec::new();
    ///
    /// loop {
    ///     // Process system events...
    ///
    ///     let mut user_interface = UserInterface::build(
    ///         counter.view(),
    ///         cache,
    ///         &renderer,
    ///     );
    ///
    ///     let messages = user_interface.update(events.drain(..));
    ///
    ///     // Draw the user interface
    ///     let mouse_cursor = user_interface.draw(&mut renderer);
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
    pub fn draw(&self, renderer: &mut Renderer) -> MouseCursor {
        self.root.widget.draw(
            renderer,
            Layout::new(&self.layout),
            self.cursor_position,
        )
    }

    /// Extract the [`Cache`] of the [`UserInterface`], consuming it in the
    /// process.
    ///
    /// [`Cache`]: struct.Cache.html
    /// [`UserInterface`]: struct.UserInterface.html
    pub fn into_cache(self) -> Cache {
        Cache {
            hash: self.hash,
            layout: self.layout,
            cursor_position: self.cursor_position,
        }
    }
}

/// Reusable data of a specific [`UserInterface`].
///
/// [`UserInterface`]: struct.UserInterface.html
#[derive(Debug, Clone)]
pub struct Cache {
    hash: u64,
    layout: result::Layout,
    cursor_position: Point,
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
        let root: Element<'_, (), ()> = Column::new().into();

        let hasher = &mut crate::Hasher::default();
        root.hash_layout(hasher);

        Cache {
            hash: hasher.finish(),
            layout: root.compute_layout(&()),
            cursor_position: Point::new(0.0, 0.0),
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
        self.hash == other.hash && self.cursor_position == other.cursor_position
    }
}

impl Eq for Cache {}
