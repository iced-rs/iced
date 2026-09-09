//! Implement your own event loop to drive a user interface.
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::shell;
use crate::core::widget;
use crate::core::window;
use crate::core::{
    Clipboard, Element, InputMethod, Layout, Rectangle, Shell, Size, Vector, Window,
};

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
/// [`integration`]: https://github.com/iced-rs/iced/tree/master/examples/integration
pub struct UserInterface<'a, Message, Theme, Renderer> {
    root: Element<'a, Message, Theme, Renderer>,
    base: layout::Node,
    state: widget::Tree,
    overlay: Option<Overlay>,
    bounds: Size,
}

struct Overlay {
    layouts: Vec<layout::Node>,
    interaction: mouse::Interaction,
}

/// Sorts a flat list of overlays by their [`overlay::Overlay::index`], so that
/// overlays with a higher index are drawn on top.
///
/// The sort is stable: overlays with an equal index keep their document order.
fn sort_overlays<'a, Message, Theme, Renderer>(
    overlays: &mut Vec<overlay::Element<'a, Message, Theme, Renderer>>,
) where
    Renderer: crate::core::Renderer,
{
    use std::cmp::Ordering;

    overlays.sort_by(|a, b| {
        a.as_overlay()
            .index()
            .partial_cmp(&b.as_overlay().index())
            .unwrap_or(Ordering::Equal)
    });
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
    /// use iced_runtime::core::shell;
    /// use iced_runtime::core::window;
    /// use iced_runtime::core::Size;
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// // Initialization
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window = window::Headless; // This should be a proper window, like a `winit` one
    /// let mut waker = shell::Waker::noop();
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
        let mut root = root.into();

        let Cache { mut state } = cache;
        state.diff(root.as_widget_mut());

        let base = root.as_widget_mut().layout(
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
    /// use iced_runtime::core::mouse;
    /// use iced_runtime::core::shell;
    /// use iced_runtime::core::window;
    /// use iced_runtime::core::Size;
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::Renderer;
    ///
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window = window::Headless; // This should be a proper window, like a `winit` one
    /// let mut waker = shell::Waker::noop();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor = mouse::Cursor::default();
    ///
    /// // Initialize our event storage
    /// let mut events = Vec::new();
    /// let mut messages = shell::Bus::new();
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
    ///         &window,
    ///         &waker,
    ///         &events,
    ///         cursor,
    ///         &mut renderer,
    ///         &mut messages
    ///     );
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     // Process the produced messages
    ///     for (message, _receipt) in messages.drain() {
    ///         counter.update(message);
    ///     }
    /// }
    /// ```
    pub fn update(
        &mut self,
        window: &dyn Window,
        waker: &shell::Waker,
        events: &[Event],
        cursor: mouse::Cursor,
        renderer: &mut Renderer,
        messages: &mut shell::Bus<Message>,
    ) -> (State, Vec<event::Status>) {
        let mut outdated = false;
        let mut redraw_request = window::RedrawRequest::Wait;
        let mut input_method = InputMethod::Disabled;
        let mut clipboard = Clipboard::new();
        let mut has_layout_changed = false;
        let viewport = Rectangle::with_size(self.bounds);

        let mut overlays = self.root.as_widget_mut().overlay(
            &mut self.state,
            Layout::new(&self.base),
            renderer,
            &viewport,
            Vector::ZERO,
        );

        sort_overlays(&mut overlays);

        let (base_cursor, overlay_statuses, overlay_interaction) = if !overlays.is_empty() {
            let bounds = self.bounds;

            let mut layouts: Vec<layout::Node> = overlays
                .iter_mut()
                .map(|overlay| overlay.as_overlay_mut().layout(renderer, bounds))
                .collect();
            let mut event_statuses = Vec::new();

            for event in events {
                let mut shell = Shell::new(window, waker.clone(), messages);

                for (overlay, layout) in overlays.iter_mut().zip(&layouts) {
                    overlay.as_overlay_mut().update(
                        event,
                        Layout::new(layout),
                        cursor,
                        renderer,
                        &mut shell,
                    );
                }

                event_statuses.push(shell.event_status());
                redraw_request = redraw_request.min(shell.redraw_request());
                input_method.merge(shell.input_method());
                clipboard.merge(shell.clipboard_mut());

                if let Some(diff) = shell.is_layout_invalid() {
                    drop(overlays);

                    match diff {
                        shell::Diff::Perform => {
                            self.root.as_widget_mut().diff(&mut self.state);
                        }
                        shell::Diff::Skip => {}
                    }

                    self.base = self.root.as_widget_mut().layout(
                        &mut self.state,
                        renderer,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    overlays = self.root.as_widget_mut().overlay(
                        &mut self.state,
                        Layout::new(&self.base),
                        renderer,
                        &viewport,
                        Vector::ZERO,
                    );

                    sort_overlays(&mut overlays);

                    if overlays.is_empty() {
                        event_statuses.resize(events.len(), event::Status::Ignored);
                        break;
                    }

                    shell.revalidate_layout(|_diff| {
                        layouts = overlays
                            .iter_mut()
                            .map(|overlay| overlay.as_overlay_mut().layout(renderer, bounds))
                            .collect();
                        has_layout_changed = true;
                    });
                }

                if shell.are_widgets_invalid() {
                    outdated = true;
                }
            }

            let (base_cursor, interaction) = if !overlays.is_empty() {
                let interaction = cursor
                    .position()
                    .map(|cursor_position| {
                        overlays
                            .iter()
                            .zip(&layouts)
                            .map(|(overlay, layout)| {
                                overlay.as_overlay().mouse_interaction(
                                    Layout::new(layout),
                                    mouse::Cursor::Available(cursor_position),
                                    renderer,
                                )
                            })
                            .max()
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();

                if interaction == mouse::Interaction::None {
                    (cursor, mouse::Interaction::None)
                } else {
                    (mouse::Cursor::Unavailable, interaction)
                }
            } else {
                (cursor, mouse::Interaction::None)
            };

            self.overlay = (!overlays.is_empty()).then_some(Overlay {
                layouts,
                interaction,
            });

            (base_cursor, event_statuses, interaction)
        } else {
            self.overlay = None;

            (
                cursor,
                vec![event::Status::Ignored; events.len()],
                mouse::Interaction::None,
            )
        };

        drop(overlays);

        let event_statuses = events
            .iter()
            .zip(overlay_statuses)
            .map(|(event, overlay_status)| {
                if matches!(overlay_status, event::Status::Captured) {
                    return overlay_status;
                }

                let mut shell = Shell::new(window, waker.clone(), messages);

                self.root.as_widget_mut().update(
                    &mut self.state,
                    event,
                    Layout::new(&self.base),
                    base_cursor,
                    renderer,
                    &mut shell,
                    &viewport,
                );

                if shell.event_status() == event::Status::Captured {
                    self.overlay = None;
                }

                redraw_request = redraw_request.min(shell.redraw_request());
                input_method.merge(shell.input_method());
                clipboard.merge(shell.clipboard_mut());

                shell.revalidate_layout(|diff| {
                    has_layout_changed = true;

                    match diff {
                        shell::Diff::Perform => {
                            self.root.as_widget_mut().diff(&mut self.state);
                        }
                        shell::Diff::Skip => {}
                    }

                    self.base = self.root.as_widget_mut().layout(
                        &mut self.state,
                        renderer,
                        &layout::Limits::new(Size::ZERO, self.bounds),
                    );

                    let mut overlays = self.root.as_widget_mut().overlay(
                        &mut self.state,
                        Layout::new(&self.base),
                        renderer,
                        &viewport,
                        Vector::ZERO,
                    );

                    sort_overlays(&mut overlays);

                    if !overlays.is_empty() {
                        let layouts = overlays
                            .iter_mut()
                            .map(|overlay| overlay.as_overlay_mut().layout(renderer, self.bounds))
                            .collect();
                        let interaction = overlays
                            .iter()
                            .zip(&layouts)
                            .map(|(overlay, layout)| {
                                overlay.as_overlay().mouse_interaction(
                                    Layout::new(layout),
                                    cursor,
                                    renderer,
                                )
                            })
                            .max()
                            .unwrap_or_default();

                        self.overlay = Some(Overlay {
                            layouts,
                            interaction,
                        });
                    }
                });

                if shell.are_widgets_invalid() {
                    outdated = true;
                }

                shell.event_status().merge(overlay_status)
            })
            .collect();

        let mouse_interaction = if overlay_interaction == mouse::Interaction::None {
            self.root.as_widget().mouse_interaction(
                &self.state,
                Layout::new(&self.base),
                base_cursor,
                &viewport,
                renderer,
            )
        } else {
            overlay_interaction
        };

        (
            if outdated {
                State::Outdated
            } else {
                State::Updated {
                    mouse_interaction,
                    redraw_request,
                    input_method,
                    clipboard,
                    has_layout_changed,
                }
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
    /// use iced_runtime::core::mouse;
    /// use iced_runtime::core::renderer;
    /// use iced_runtime::core::shell;
    /// use iced_runtime::core::window;
    /// use iced_runtime::core::{Element, Size};
    /// use iced_runtime::user_interface::{self, UserInterface};
    /// use iced_wgpu::{Renderer, Theme};
    ///
    /// let mut counter = Counter::new();
    /// let mut cache = user_interface::Cache::new();
    /// let mut renderer = Renderer::default();
    /// let mut window = window::Headless; // This should be a proper window, like a `winit` one
    /// let mut waker = shell::Waker::noop();
    /// let mut window_size = Size::new(1024.0, 768.0);
    /// let mut cursor = mouse::Cursor::default();
    /// let mut events = Vec::new();
    /// let mut messages = shell::Bus::new();
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
    ///         &window,
    ///         &waker,
    ///         &events,
    ///         cursor,
    ///         &mut renderer,
    ///         &mut messages
    ///     );
    ///
    ///     // Draw the user interface
    ///     let mouse_interaction = user_interface.draw(&mut renderer, &theme, &renderer::Style::default(), cursor);
    ///
    ///     cache = user_interface.into_cache();
    ///
    ///     for (message, _receipt) in messages.drain() {
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
    ) {
        let viewport = Rectangle::with_size(self.bounds);
        renderer.reset(viewport);

        let base_cursor = match &self.overlay {
            None
            | Some(Overlay {
                interaction: mouse::Interaction::None,
                ..
            }) => cursor,
            _ => mouse::Cursor::Unavailable,
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

        let Self {
            overlay,
            root,
            base,
            ..
        } = self;

        let Some(Overlay { layouts, .. }) = overlay.as_ref() else {
            return;
        };

        let mut overlays = root.as_widget_mut().overlay(
            &mut self.state,
            Layout::new(base),
            renderer,
            &viewport,
            Vector::ZERO,
        );

        sort_overlays(&mut overlays);

        for (overlay, layout) in overlays.iter_mut().zip(layouts) {
            overlay
                .as_overlay()
                .draw(renderer, theme, style, Layout::new(layout), cursor);
        }
    }

    /// Applies a [`widget::Operation`] to the [`UserInterface`].
    pub fn operate(&mut self, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        let viewport = Rectangle::with_size(self.bounds);

        self.root.as_widget_mut().operate(
            &mut self.state,
            Layout::new(&self.base),
            renderer,
            operation,
        );

        let mut overlays = self.root.as_widget_mut().overlay(
            &mut self.state,
            Layout::new(&self.base),
            renderer,
            &viewport,
            Vector::ZERO,
        );

        sort_overlays(&mut overlays);

        if !overlays.is_empty() {
            if self.overlay.is_none() {
                let layouts = overlays
                    .iter_mut()
                    .map(|overlay| overlay.as_overlay_mut().layout(renderer, self.bounds))
                    .collect();

                self.overlay = Some(Overlay {
                    layouts,
                    interaction: mouse::Interaction::None,
                });
            }

            let layouts = self.overlay.as_ref().unwrap().layouts.clone();

            for (overlay, layout) in overlays.iter_mut().zip(&layouts) {
                overlay
                    .as_overlay_mut()
                    .operate(Layout::new(layout), renderer, operation);
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
#[derive(Debug)]
pub enum State {
    /// The [`UserInterface`] is outdated and needs to be rebuilt.
    Outdated,

    /// The [`UserInterface`] is up-to-date and can be reused without
    /// rebuilding.
    Updated {
        /// The current [`mouse::Interaction`] of the user interface.
        mouse_interaction: mouse::Interaction,
        /// The [`window::RedrawRequest`] describing when a redraw should be performed.
        redraw_request: window::RedrawRequest,
        /// The current [`InputMethod`] strategy of the user interface.
        input_method: InputMethod,
        /// The set of [`Clipboard`] requests that the user interface has produced.
        clipboard: Clipboard,
        /// Whether the layout of the [`UserInterface`] has changed.
        has_layout_changed: bool,
    },
}

impl State {
    /// Returns whether the layout of the [`UserInterface`] has changed.
    pub fn has_layout_changed(&self) -> bool {
        match self {
            State::Outdated => true,
            State::Updated {
                has_layout_changed, ..
            } => *has_layout_changed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Theme = crate::core::Theme;
    type Renderer = ();

    struct ByIndex(f32);

    impl overlay::Overlay<(), Theme, Renderer> for ByIndex {
        fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
            layout::Node::new(Size::ZERO)
        }

        fn draw(
            &self,
            _renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            _layout: Layout<'_>,
            _cursor: mouse::Cursor,
        ) {
        }

        fn index(&self) -> f32 {
            self.0
        }
    }

    #[test]
    fn sort_overlays_orders_by_index_and_keeps_document_order_on_ties() {
        let mut overlays = vec![
            overlay::Element::new(Box::new(ByIndex(1.0))),
            overlay::Element::new(Box::new(ByIndex(f32::MAX))),
            overlay::Element::new(Box::new(ByIndex(1.0))),
            overlay::Element::new(Box::new(ByIndex(2.0))),
        ];

        sort_overlays(&mut overlays);

        let indices: Vec<f32> = overlays
            .iter()
            .map(|overlay| overlay.as_overlay().index())
            .collect();

        // The two `1.0` overlays keep their relative document order (stable
        // sort), and the `MAX` overlay is placed last so it is drawn on top.
        assert_eq!(indices, vec![1.0, 1.0, 2.0, f32::MAX]);
    }
}
