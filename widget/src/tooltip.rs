//! Tooltips display a hint of information over some element when hovered.
//!
//! By default, the tooltip is displayed immediately, however, this can be adjusted
//! with [`Tooltip::delay`].
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # use iced::widget::Widget;
//! # pub type State = ();
//! use iced::widget::{container, tooltip};
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(_state: &State) -> impl Widget<Message> {
//!     tooltip(
//!         "Hover me to display the tooltip!",
//!         container("This is the tooltip contents!")
//!             .padding(10)
//!             .style(container::rounded_box),
//!     )
//!     .position(tooltip::Position::Bottom)
//! }
//! ```
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::time::{Duration, Instant};
use crate::core::widget::{self, Widget};
use crate::core::window;
use crate::core::{Element, Event, Length, Pixels, Point, Rectangle, Shell, Size, Vector};

/// An element to display a widget over another.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # use iced::widget::Widget;
/// # pub type State = ();
/// use iced::widget::{container, tooltip};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(_state: &State) -> impl Widget<Message> {
///     tooltip(
///         "Hover me to display the tooltip!",
///         container("This is the tooltip contents!")
///             .padding(10)
///             .style(container::rounded_box),
///     )
///     .position(tooltip::Position::Bottom)
/// }
/// ```
pub struct Tooltip<W, V> {
    content: W,
    tooltip: V,
    position: Position,
    gap: f32,
    snap_within_viewport: bool,
    delay: Duration,
}

impl<W, V> Tooltip<W, V> {
    /// Creates a new [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    ///
    /// By default, the [`Tooltip`] is positioned [`Position::Auto`], which
    /// places it on the side of the hovered element with the most available
    /// space.
    pub fn new(content: W, tooltip: V) -> Self {
        Tooltip {
            content,
            tooltip,
            position: Position::default(),
            gap: 0.0,
            snap_within_viewport: true,
            delay: Duration::ZERO,
        }
    }

    /// Sets the [`Position`] of the [`Tooltip`].
    ///
    /// By default, the [`Tooltip`] is positioned [`Position::Auto`], which
    /// places it on the side of the hovered element with the most available
    /// space.
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Sets the gap between the content and its [`Tooltip`].
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    /// Sets the delay before the [`Tooltip`] is shown.
    ///
    /// Set to [`Duration::ZERO`] to be shown immediately.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets whether the [`Tooltip`] is snapped within the viewport.
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }
}

impl<W, V> widget::Meta for Tooltip<W, V> {}

impl<W, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Tooltip<W, V>
where
    Renderer: text::Renderer,
    W: Widget<Message, Theme, Renderer>,
    V: Widget<Message, Theme, Renderer>,
{
    fn diff(&mut self, tree: &mut widget::Tree) {
        let state = tree.state.downcast_mut::<State>();

        // The tooltip's contents may have changed, so the cached layout
        // (if any) is no longer valid
        if let State::Open { needs_relayout, .. } = state {
            *needs_relayout = true;
        }

        if tree.children.len() != 2 {
            tree.children = vec![
                widget::Tree::new(&self.content),
                widget::Tree::new(&self.tooltip),
            ];
        }

        tree.children[0].diff(&mut self.content);
        tree.children[1].diff(&mut self.tooltip);
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn size(&self) -> Size<Length> {
        self.content.size()
    }

    fn layout(&mut self, tree: &mut widget::Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.content.layout(&mut tree.children[0], renderer, limits);

        tree.size = tree.children[0].size;
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Mouse(_) | Event::Window(window::Event::RedrawRequested(_)) = event {
            let state = tree.state.downcast_mut::<State>();
            let now = Instant::now();
            let cursor_position = cursor.position_over(layout.bounds());

            match (&*state, cursor_position) {
                (State::Idle, Some(cursor_position)) => {
                    if self.delay == Duration::ZERO {
                        *state = State::Open {
                            cursor_position,
                            needs_relayout: true,
                        };
                        shell.invalidate_overlay();
                    } else {
                        *state = State::Hovered { at: now };
                    }

                    shell.request_redraw_at(now + self.delay);
                }
                (State::Hovered { .. }, None) => {
                    *state = State::Idle;
                }
                (State::Hovered { at, .. }, _) if at.elapsed() < self.delay => {
                    shell.request_redraw_at(now + self.delay - at.elapsed());
                }
                (State::Hovered { .. }, Some(cursor_position)) => {
                    *state = State::Open {
                        cursor_position,
                        needs_relayout: true,
                    };
                    shell.invalidate_overlay();
                }
                (
                    &State::Open {
                        cursor_position: last_position,
                        ..
                    },
                    Some(cursor_position),
                ) if self.position == Position::FollowCursor
                    && last_position != cursor_position =>
                {
                    if let State::Open {
                        cursor_position: ref mut position,
                        ..
                    } = *state
                    {
                        *position = cursor_position;
                    }

                    shell.request_redraw();
                }
                (State::Open { .. }, None) => {
                    *state = State::Idle;
                    shell.invalidate_overlay();

                    if !matches!(event, Event::Window(window::Event::RedrawRequested(_)),) {
                        shell.request_redraw();
                    }
                }
                (State::Open { .. }, Some(_)) | (&State::Idle, None) => (),
            }
        }

        self.content.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();

        let mut children = tree.children.iter_mut();

        let content = self.content.overlay(
            children.next().unwrap(),
            layout,
            renderer,
            viewport,
            translation,
            window,
        );

        let tooltip_tree = children.next().unwrap();

        // (Re)compute the tooltip's layout if it was invalidated by
        // `Widget::diff`
        if let State::Open { needs_relayout, .. } = state
            && *needs_relayout
        {
            self.tooltip.layout(
                tooltip_tree,
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    if self.snap_within_viewport {
                        window
                    } else {
                        Size::INFINITE
                    },
                ),
            );

            *needs_relayout = false;
        }

        let tooltip = if let State::Open {
            cursor_position, ..
        } = &*state
        {
            let content_bounds = layout.bounds() + translation;
            let cursor_position = *cursor_position + translation;
            let tooltip_size = tooltip_tree.size;

            let viewport = Rectangle::with_size(window);

            let tooltip_bounds = self.position.resolve(
                content_bounds,
                tooltip_size,
                self.gap,
                cursor_position,
                viewport,
                self.snap_within_viewport,
            );

            Some(overlay::Element::new(Box::new(Overlay {
                layout: Layout::new(tooltip_size).move_to(tooltip_bounds.position()),
                tooltip: &mut self.tooltip,
                tree: tooltip_tree,
                window,
            })))
        } else {
            None
        };

        content.into_iter().chain(tooltip).collect()
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds(), viewport);
        operation.traverse(&mut |operation| {
            self.content
                .operate(&mut tree.children[0], layout, viewport, renderer, operation);
        });
    }
}

impl<'a, W, V, Message, Theme, Renderer> From<Tooltip<W, V>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: text::Renderer + 'a,
    W: Widget<Message, Theme, Renderer> + 'a,
    V: Widget<Message, Theme, Renderer> + 'a,
{
    fn from(tooltip: Tooltip<W, V>) -> Element<'a, Message, Theme, Renderer> {
        tooltip._boxed()
    }
}

/// The position of the tooltip.
pub use crate::overlay::Position;

#[derive(Debug, Clone, PartialEq, Default)]
enum State {
    #[default]
    Idle,
    Hovered {
        at: Instant,
    },
    Open {
        cursor_position: Point,
        needs_relayout: bool,
    },
}

struct Overlay<'b, V> {
    layout: Layout,
    tooltip: &'b mut V,
    tree: &'b mut widget::Tree,
    window: Size,
}

impl<V, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer> for Overlay<'_, V>
where
    Renderer: text::Renderer,
    V: Widget<Message, Theme, Renderer>,
{
    fn update(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        self.tooltip.update(
            self.tree,
            event,
            self.layout,
            cursor,
            renderer,
            shell,
            &Rectangle::with_size(self.window),
        );
    }

    fn mouse_interaction(&self, cursor: mouse::Cursor, renderer: &Renderer) -> mouse::Interaction {
        self.tooltip
            .mouse_interaction(self.tree, self.layout, cursor, &Rectangle::with_size(self.window), renderer)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        cursor_position: mouse::Cursor,
    ) {
        // The tooltip content is drawn directly; users are responsible for
        // styling it.
        let viewport = Rectangle::with_size(self.window);

        renderer.with_layer(viewport, |renderer| {
            self.tooltip.draw(
                self.tree,
                renderer,
                theme,
                inherited_style,
                self.layout,
                cursor_position,
                &viewport,
            );
        });
    }

    fn operate(&mut self, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        let viewport = Rectangle::with_size(self.window);

        operation.container(None, self.layout.bounds(), &viewport);

        operation.traverse(&mut |operation| {
            self.tooltip.operate(
                self.tree,
                self.layout,
                &viewport,
                renderer,
                operation,
            );
        });
    }

    fn overlay<'c>(
        &'c mut self,
        renderer: &Renderer,
    ) -> Vec<overlay::Element<'c, Message, Theme, Renderer>> {
        self.tooltip.overlay(
            self.tree,
            self.layout,
            renderer,
            &Rectangle::with_size(self.window),
            Vector::ZERO,
            self.window,
        )
    }
}
