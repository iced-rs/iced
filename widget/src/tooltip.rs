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
//!         tooltip::Position::Bottom,
//!     )
//! }
//! ```
use crate::container;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::time::{Duration, Instant};
use crate::core::widget::{self, Widget};
use crate::core::window;
use crate::core::{Event, Length, Pixels, Point, Rectangle, Shell, Size, Vector};

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
///         tooltip::Position::Bottom,
///     )
/// }
/// ```
pub struct Tooltip<'a, W, V, Theme = crate::Theme>
where
    Theme: container::Catalog,
{
    content: W,
    tooltip: V,
    position: Position,
    gap: f32,
    snap_within_viewport: bool,
    delay: Duration,
    class: Theme::Class<'a>,
}

impl<'a, W, V, Theme> Tooltip<'a, W, V, Theme>
where
    Theme: container::Catalog,
{
    /// Creates a new [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    pub fn new(content: W, tooltip: V, position: Position) -> Self {
        Tooltip {
            content,
            tooltip,
            position,
            gap: 0.0,
            snap_within_viewport: true,
            delay: Duration::ZERO,
            class: Theme::default(),
        }
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

    /// Sets the style of the [`Tooltip`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<container::StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as container::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Tooltip`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<W, V, Theme> widget::Node for Tooltip<'_, W, V, Theme> where Theme: container::Catalog {}

impl<W, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Tooltip<'_, W, V, Theme>
where
    Theme: container::Catalog,
    Renderer: text::Renderer,
    W: Widget<Message, Theme, Renderer>,
    V: Widget<Message, Theme, Renderer>,
{
    fn diff(&mut self, tree: &mut widget::Tree) {
        let state = tree.state.downcast_mut::<State>();

        // The tooltip's contents may have changed, so the cached node
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

        // (Re)compute the tooltip's node if it was cleared by
        // `Widget::diff` or `Widget::update`
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
        } = state
        {
            let position = layout.position() + translation;
            let content_bounds = layout.bounds();
            let tooltip_size = tooltip_tree.size;

            let viewport = Rectangle::with_size(window);

            let x_center = position.x + (content_bounds.width - tooltip_size.width) / 2.0;
            let y_center = position.y + (content_bounds.height - tooltip_size.height) / 2.0;

            let mut tooltip_bounds = {
                let position = match self.position {
                    Position::Top => {
                        Point::new(x_center, position.y - tooltip_size.height - self.gap)
                    }
                    Position::Bottom => {
                        Point::new(x_center, position.y + content_bounds.height + self.gap)
                    }
                    Position::Left => {
                        Point::new(position.x - tooltip_size.width - self.gap, y_center)
                    }
                    Position::Right => {
                        Point::new(position.x + content_bounds.width + self.gap, y_center)
                    }
                    Position::FollowCursor => {
                        let translation = position - content_bounds.position();

                        Point::new(cursor_position.x, cursor_position.y - tooltip_size.height)
                            + translation
                    }
                };

                Rectangle::new(position, tooltip_size)
            };

            if self.snap_within_viewport {
                if tooltip_bounds.x < viewport.x {
                    tooltip_bounds.x = viewport.x;
                } else if viewport.x + viewport.width < tooltip_bounds.x + tooltip_bounds.width {
                    tooltip_bounds.x = viewport.x + viewport.width - tooltip_bounds.width;
                }

                if tooltip_bounds.y < viewport.y {
                    tooltip_bounds.y = viewport.y;
                } else if viewport.y + viewport.height < tooltip_bounds.y + tooltip_bounds.height {
                    tooltip_bounds.y = viewport.y + viewport.height - tooltip_bounds.height;
                }
            }

            Some(overlay::Element::new(Box::new(Overlay {
                layout: Layout::new(tooltip_tree.size).move_to(tooltip_bounds.position()),
                tooltip: &mut self.tooltip,
                tree: tooltip_tree,
                class: &self.class,
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

/// The position of the tooltip. Defaults to following the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The tooltip will appear on the top of the widget.
    #[default]
    Top,
    /// The tooltip will appear on the bottom of the widget.
    Bottom,
    /// The tooltip will appear on the left of the widget.
    Left,
    /// The tooltip will appear on the right of the widget.
    Right,
    /// The tooltip will follow the cursor.
    FollowCursor,
}

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

struct Overlay<'a, 'b, V, Theme>
where
    Theme: container::Catalog,
{
    layout: Layout,
    tooltip: &'b mut V,
    tree: &'b mut widget::Tree,
    class: &'b Theme::Class<'a>,
    window: Size,
}

impl<V, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, V, Theme>
where
    Theme: container::Catalog,
    Renderer: text::Renderer,
    V: Widget<Message, Theme, Renderer>,
{
    fn operate(&mut self, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        operation.container(None, self.layout.bounds(), &self.layout.bounds());

        operation.traverse(&mut |operation| {
            self.tooltip.operate(
                self.tree,
                self.layout,
                &Rectangle::with_size(self.window),
                renderer,
                operation,
            );
        });
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        cursor_position: mouse::Cursor,
    ) {
        let viewport = Rectangle::with_size(self.window);
        let bounds = self.layout.bounds();
        let style = theme.style(self.class);

        renderer.with_layer(viewport, |renderer| {
            container::draw_background(renderer, &style, bounds);

            let defaults = renderer::Style {
                text_color: style.text_color.unwrap_or(inherited_style.text_color),
            };

            self.tooltip.draw(
                self.tree,
                renderer,
                theme,
                &defaults,
                self.layout,
                cursor_position,
                &viewport,
            );
        });
    }
}
