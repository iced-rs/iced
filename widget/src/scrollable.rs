//! Navigate an endless amount of content with a scrollbar.
use iced_runtime::core::widget::Id;
use std::borrow::Cow;

use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Background, Clipboard, Color, Element, Layout, Length, Pixels, Point,
    Rectangle, Shell, Size, Vector, Widget,
};
use crate::runtime::Command;

pub use crate::style::scrollable::{Scrollbar, Scroller, StyleSheet};
use iced_renderer::core::widget::OperationOutputWrapper;
pub use operation::scrollable::RelativeOffset;

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Scrollable<'a, Message, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Id,
    scrollbar_id: Id,
    #[cfg(feature = "a11y")]
    name: Option<Cow<'a, str>>,
    #[cfg(feature = "a11y")]
    description: Option<iced_accessibility::Description<'a>>,
    #[cfg(feature = "a11y")]
    label: Option<Vec<iced_accessibility::accesskit::NodeId>>,
    width: Length,
    height: Length,
    vertical: Properties,
    horizontal: Option<Properties>,
    content: Element<'a, Message, Renderer>,
    on_scroll: Option<Box<dyn Fn(RelativeOffset) -> Message + 'a>>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Scrollable<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Scrollable`].
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Scrollable {
            id: Id::unique(),
            scrollbar_id: Id::unique(),
            #[cfg(feature = "a11y")]
            name: None,
            #[cfg(feature = "a11y")]
            description: None,
            #[cfg(feature = "a11y")]
            label: None,
            width: Length::Shrink,
            height: Length::Shrink,
            vertical: Properties::default(),
            horizontal: None,
            content: content.into(),
            on_scroll: None,
            style: Default::default(),
        }
    }

    /// Sets the [`Id`] of the [`Scrollable`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Sets the width of the [`Scrollable`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Configures the vertical scrollbar of the [`Scrollable`] .
    pub fn vertical_scroll(mut self, properties: Properties) -> Self {
        self.vertical = properties;
        self
    }

    /// Configures the horizontal scrollbar of the [`Scrollable`] .
    pub fn horizontal_scroll(mut self, properties: Properties) -> Self {
        self.horizontal = Some(properties);
        self
    }

    /// Sets a function to call when the [`Scrollable`] is scrolled.
    ///
    /// The function takes the new relative x & y offset of the [`Scrollable`]
    /// (e.g. `0` means beginning, while `1` means end).
    pub fn on_scroll(
        mut self,
        f: impl Fn(RelativeOffset) -> Message + 'a,
    ) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Sets the style of the [`Scrollable`] .
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the name of the [`Button`].
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description_widget(
        mut self,
        description: &impl iced_accessibility::Describes,
    ) -> Self {
        self.description = Some(iced_accessibility::Description::Id(
            description.description(),
        ));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description =
            Some(iced_accessibility::Description::Text(description.into()));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the label of the [`Button`].
    pub fn label(mut self, label: &dyn iced_accessibility::Labels) -> Self {
        self.label =
            Some(label.label().into_iter().map(|l| l.into()).collect());
        self
    }
}

/// Properties of a scrollbar within a [`Scrollable`].
#[derive(Debug)]
pub struct Properties {
    width: f32,
    margin: f32,
    scroller_width: f32,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            width: 10.0,
            margin: 0.0,
            scroller_width: 10.0,
        }
    }
}

impl Properties {
    /// Creates new [`Properties`] for use in a [`Scrollable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the scrollbar width of the [`Scrollable`] .
    /// Silently enforces a minimum width of 1.
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into().0.max(1.0);
        self
    }

    /// Sets the scrollbar margin of the [`Scrollable`] .
    pub fn margin(mut self, margin: impl Into<Pixels>) -> Self {
        self.margin = margin.into().0;
        self
    }

    /// Sets the scroller width of the [`Scrollable`] .
    /// Silently enforces a minimum width of 1.
    pub fn scroller_width(mut self, scroller_width: impl Into<Pixels>) -> Self {
        self.scroller_width = scroller_width.into().0.max(1.0);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Scrollable<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

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
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.horizontal.is_some(),
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.scrollable(state, Some(&self.id));

        operation.container(Some(&self.id), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        update(
            tree.state.downcast_mut::<State>(),
            event,
            layout,
            cursor_position,
            clipboard,
            shell,
            &self.vertical,
            self.horizontal.as_ref(),
            &self.on_scroll,
            |event, layout, cursor_position, clipboard, shell| {
                self.content.as_widget_mut().on_event(
                    &mut tree.children[0],
                    event,
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            },
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            tree.state.downcast_ref::<State>(),
            renderer,
            theme,
            layout,
            cursor_position,
            &self.vertical,
            self.horizontal.as_ref(),
            &self.style,
            |renderer, layout, cursor_position, viewport| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor_position,
                    viewport,
                )
            },
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(
            tree.state.downcast_ref::<State>(),
            layout,
            cursor_position,
            &self.vertical,
            self.horizontal.as_ref(),
            |layout, cursor_position, viewport| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            },
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
            )
            .map(|overlay| {
                let bounds = layout.bounds();
                let content_layout = layout.children().next().unwrap();
                let content_bounds = content_layout.bounds();
                let offset = tree
                    .state
                    .downcast_ref::<State>()
                    .offset(bounds, content_bounds);

                overlay.translate(Vector::new(-offset.x, -offset.y))
            })
    }

    #[cfg(feature = "a11y")]
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        state: &Tree,
        cursor_position: Point,
    ) -> iced_accessibility::A11yTree {
        use iced_accessibility::{
            accesskit::{NodeBuilder, NodeId, Rect, Role},
            A11yId, A11yNode, A11yTree,
        };

        let child_layout = layout.children().next().unwrap();
        let child_tree = &state.children[0];
        let child_tree = self.content.as_widget().a11y_nodes(
            child_layout,
            &child_tree,
            cursor_position,
        );

        let window = layout.bounds();
        let is_hovered = window.contains(cursor_position);
        let Rectangle {
            x,
            y,
            width,
            height,
        } = window;
        let bounds = Rect::new(
            x as f64,
            y as f64,
            (x + width) as f64,
            (y + height) as f64,
        );
        let mut node = NodeBuilder::new(Role::ScrollView);
        node.set_bounds(bounds);
        if let Some(name) = self.name.as_ref() {
            node.set_name(name.clone());
        }
        match self.description.as_ref() {
            Some(iced_accessibility::Description::Id(id)) => {
                node.set_described_by(
                    id.iter()
                        .cloned()
                        .map(|id| NodeId::from(id))
                        .collect::<Vec<_>>(),
                );
            }
            Some(iced_accessibility::Description::Text(text)) => {
                node.set_description(text.clone());
            }
            None => {}
        }

        if is_hovered {
            node.set_hovered();
        }

        if let Some(label) = self.label.as_ref() {
            node.set_labelled_by(label.clone());
        }

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

        let mut scrollbar_node = NodeBuilder::new(Role::ScrollBar);
        if matches!(state.state, tree::State::Some(_)) {
            let state = state.state.downcast_ref::<State>();
            let scrollbars = Scrollbars::new(
                state,
                &self.vertical,
                self.horizontal.as_ref(),
                window,
                content_bounds,
            );
            for (window, content, offset, scrollbar) in scrollbars
                .x
                .iter()
                .map(|s| {
                    (window.width, content_bounds.width, state.offset_x, s)
                })
                .chain(scrollbars.y.iter().map(|s| {
                    (window.height, content_bounds.height, state.offset_y, s)
                }))
            {
                let scrollbar_bounds = scrollbar.total_bounds;
                let is_hovered = scrollbar_bounds.contains(cursor_position);
                let Rectangle {
                    x,
                    y,
                    width,
                    height,
                } = scrollbar_bounds;
                let bounds = Rect::new(
                    x as f64,
                    y as f64,
                    (x + width) as f64,
                    (y + height) as f64,
                );
                scrollbar_node.set_bounds(bounds);
                if is_hovered {
                    scrollbar_node.set_hovered();
                }
                scrollbar_node
                    .set_controls(vec![A11yId::Widget(self.id.clone()).into()]);
                scrollbar_node.set_numeric_value(
                    100.0 * offset.absolute(window, content) as f64
                        / scrollbar_bounds.height as f64,
                );
            }
        }

        let child_tree = A11yTree::join(
            [
                child_tree,
                A11yTree::leaf(scrollbar_node, self.scrollbar_id.clone()),
            ]
            .into_iter(),
        );
        A11yTree::node_with_child_tree(
            A11yNode::new(node, self.id.clone()),
            child_tree,
        )
    }

    fn id(&self) -> Option<Id> {
        use iced_accessibility::Internal;

        Some(Id(Internal::Set(vec![
            self.id.0.clone(),
            self.scrollbar_id.0.clone(),
        ])))
    }
}

impl<'a, Message, Renderer> From<Scrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        text_input: Scrollable<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}

/// Produces a [`Command`] that snaps the [`Scrollable`] with the given [`Id`]
/// to the provided `percentage` along the x & y axis.
pub fn snap_to<Message: 'static>(
    id: Id,
    offset: RelativeOffset,
) -> Command<Message> {
    Command::widget(operation::scrollable::snap_to(id, offset))
}

/// Computes the layout of a [`Scrollable`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    horizontal_enabled: bool,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.width(width).height(height);

    let child_limits = layout::Limits::new(
        Size::new(limits.min().width, 0.0),
        Size::new(
            if horizontal_enabled {
                f32::INFINITY
            } else {
                limits.max().width
            },
            f32::MAX,
        ),
    );

    let content = layout_content(renderer, &child_limits);
    let size = limits.resolve(content.size());

    layout::Node::with_children(size, vec![content])
}

/// Processes an [`Event`] and updates the [`State`] of a [`Scrollable`]
/// accordingly.
pub fn update<Message>(
    state: &mut State,
    event: Event,
    layout: Layout<'_>,
    cursor_position: Point,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    vertical: &Properties,
    horizontal: Option<&Properties>,
    on_scroll: &Option<Box<dyn Fn(RelativeOffset) -> Message + '_>>,
    update_content: impl FnOnce(
        Event,
        Layout<'_>,
        Point,
        &mut dyn Clipboard,
        &mut Shell<'_, Message>,
    ) -> event::Status,
) -> event::Status {
    let bounds = layout.bounds();
    let mouse_over_scrollable = bounds.contains(cursor_position);

    let content = layout.children().next().unwrap();
    let content_bounds = content.bounds();

    let scrollbars =
        Scrollbars::new(state, vertical, horizontal, bounds, content_bounds);

    let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) =
        scrollbars.is_mouse_over(cursor_position);

    let event_status = {
        let cursor_position = if mouse_over_scrollable
            && !(mouse_over_y_scrollbar || mouse_over_x_scrollbar)
        {
            cursor_position + state.offset(bounds, content_bounds)
        } else {
            // TODO: Make `cursor_position` an `Option<Point>` so we can encode
            // cursor availability.
            // This will probably happen naturally once we add multi-window
            // support.
            Point::new(-1.0, -1.0)
        };

        update_content(
            event.clone(),
            content,
            cursor_position,
            clipboard,
            shell,
        )
    };

    if let event::Status::Captured = event_status {
        return event::Status::Captured;
    }

    if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event
    {
        state.keyboard_modifiers = modifiers;

        return event::Status::Ignored;
    }

    if mouse_over_scrollable {
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        // TODO: Configurable speed/friction (?)
                        let movement = if state.keyboard_modifiers.shift() {
                            Vector::new(y, x)
                        } else {
                            Vector::new(x, y)
                        };

                        movement * 60.0
                    }
                    mouse::ScrollDelta::Pixels { x, y } => Vector::new(x, y),
                };

                state.scroll(delta, bounds, content_bounds);

                notify_on_scroll(
                    state,
                    on_scroll,
                    bounds,
                    content_bounds,
                    shell,
                );

                return event::Status::Captured;
            }
            Event::Touch(event)
                if state.scroll_area_touched_at.is_some()
                    || !mouse_over_y_scrollbar && !mouse_over_x_scrollbar =>
            {
                match event {
                    touch::Event::FingerPressed { .. } => {
                        state.scroll_area_touched_at = Some(cursor_position);
                    }
                    touch::Event::FingerMoved { .. } => {
                        if let Some(scroll_box_touched_at) =
                            state.scroll_area_touched_at
                        {
                            let delta = Vector::new(
                                cursor_position.x - scroll_box_touched_at.x,
                                cursor_position.y - scroll_box_touched_at.y,
                            );

                            state.scroll(delta, bounds, content_bounds);

                            state.scroll_area_touched_at =
                                Some(cursor_position);

                            notify_on_scroll(
                                state,
                                on_scroll,
                                bounds,
                                content_bounds,
                                shell,
                            );
                        }
                    }
                    touch::Event::FingerLifted { .. }
                    | touch::Event::FingerLost { .. } => {
                        state.scroll_area_touched_at = None;
                    }
                }

                return event::Status::Captured;
            }
            _ => {}
        }
    }

    if let Some(scroller_grabbed_at) = state.y_scroller_grabbed_at {
        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.y_scroller_grabbed_at = None;

                return event::Status::Captured;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(scrollbar) = scrollbars.y {
                    state.scroll_y_to(
                        scrollbar.scroll_percentage_y(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );

                    return event::Status::Captured;
                }
            }
            _ => {}
        }
    } else if mouse_over_y_scrollbar {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                    (scrollbars.grab_y_scroller(cursor_position), scrollbars.y)
                {
                    state.scroll_y_to(
                        scrollbar.scroll_percentage_y(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    state.y_scroller_grabbed_at = Some(scroller_grabbed_at);

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );
                }

                return event::Status::Captured;
            }
            _ => {}
        }
    }

    if let Some(scroller_grabbed_at) = state.x_scroller_grabbed_at {
        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.x_scroller_grabbed_at = None;

                return event::Status::Captured;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(scrollbar) = scrollbars.x {
                    state.scroll_x_to(
                        scrollbar.scroll_percentage_x(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );
                }

                return event::Status::Captured;
            }
            _ => {}
        }
    } else if mouse_over_x_scrollbar {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                    (scrollbars.grab_x_scroller(cursor_position), scrollbars.x)
                {
                    state.scroll_x_to(
                        scrollbar.scroll_percentage_x(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    state.x_scroller_grabbed_at = Some(scroller_grabbed_at);

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );

                    return event::Status::Captured;
                }
            }
            _ => {}
        }
    }

    event::Status::Ignored
}

/// Computes the current [`mouse::Interaction`] of a [`Scrollable`].
pub fn mouse_interaction(
    state: &State,
    layout: Layout<'_>,
    cursor_position: Point,
    vertical: &Properties,
    horizontal: Option<&Properties>,
    content_interaction: impl FnOnce(
        Layout<'_>,
        Point,
        &Rectangle,
    ) -> mouse::Interaction,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let mouse_over_scrollable = bounds.contains(cursor_position);

    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    let scrollbars =
        Scrollbars::new(state, vertical, horizontal, bounds, content_bounds);

    let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) =
        scrollbars.is_mouse_over(cursor_position);

    if (mouse_over_x_scrollbar || mouse_over_y_scrollbar)
        || state.scrollers_grabbed()
    {
        mouse::Interaction::Idle
    } else {
        let offset = state.offset(bounds, content_bounds);

        let cursor_position = if mouse_over_scrollable
            && !(mouse_over_y_scrollbar || mouse_over_x_scrollbar)
        {
            cursor_position + offset
        } else {
            Point::new(-1.0, -1.0)
        };

        content_interaction(
            content_layout,
            cursor_position,
            &Rectangle {
                y: bounds.y + offset.y,
                x: bounds.x + offset.x,
                ..bounds
            },
        )
    }
}

/// Draws a [`Scrollable`].
pub fn draw<Renderer>(
    state: &State,
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor_position: Point,
    vertical: &Properties,
    horizontal: Option<&Properties>,
    style: &<Renderer::Theme as StyleSheet>::Style,
    draw_content: impl FnOnce(&mut Renderer, Layout<'_>, Point, &Rectangle),
) where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    let bounds = layout.bounds();
    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    let scrollbars =
        Scrollbars::new(state, vertical, horizontal, bounds, content_bounds);

    let mouse_over_scrollable = bounds.contains(cursor_position);
    let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) =
        scrollbars.is_mouse_over(cursor_position);

    let offset = state.offset(bounds, content_bounds);

    let cursor_position = if mouse_over_scrollable
        && !(mouse_over_x_scrollbar || mouse_over_y_scrollbar)
    {
        cursor_position + offset
    } else {
        Point::new(-1.0, -1.0)
    };

    // Draw inner content
    if scrollbars.active() {
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(
                Vector::new(-offset.x, -offset.y),
                |renderer| {
                    draw_content(
                        renderer,
                        content_layout,
                        cursor_position,
                        &Rectangle {
                            y: bounds.y + offset.y,
                            x: bounds.x + offset.x,
                            ..bounds
                        },
                    );
                },
            );
        });

        let draw_scrollbar =
            |renderer: &mut Renderer,
             style: Scrollbar,
             scrollbar: &internals::Scrollbar| {
                //track
                if style.background.is_some()
                    || (style.border_color != Color::TRANSPARENT
                        && style.border_width > 0.0)
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: scrollbar.bounds,
                            border_radius: style.border_radius.into(),
                            border_width: style.border_width,
                            border_color: style.border_color,
                        },
                        style
                            .background
                            .unwrap_or(Background::Color(Color::TRANSPARENT)),
                    );
                }

                //thumb
                if style.scroller.color != Color::TRANSPARENT
                    || (style.scroller.border_color != Color::TRANSPARENT
                        && style.scroller.border_width > 0.0)
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: scrollbar.scroller.bounds,
                            border_radius: style.scroller.border_radius.into(),
                            border_width: style.scroller.border_width,
                            border_color: style.scroller.border_color,
                        },
                        style.scroller.color,
                    );
                }
            };

        renderer.with_layer(
            Rectangle {
                width: bounds.width + 2.0,
                height: bounds.height + 2.0,
                ..bounds
            },
            |renderer| {
                //draw y scrollbar
                if let Some(scrollbar) = scrollbars.y {
                    let style = if state.y_scroller_grabbed_at.is_some() {
                        theme.dragging(style)
                    } else if mouse_over_scrollable {
                        theme.hovered(style, mouse_over_y_scrollbar)
                    } else {
                        theme.active(style)
                    };

                    draw_scrollbar(renderer, style, &scrollbar);
                }

                //draw x scrollbar
                if let Some(scrollbar) = scrollbars.x {
                    let style = if state.x_scroller_grabbed_at.is_some() {
                        theme.dragging_horizontal(style)
                    } else if mouse_over_scrollable {
                        theme.hovered_horizontal(style, mouse_over_x_scrollbar)
                    } else {
                        theme.active_horizontal(style)
                    };

                    draw_scrollbar(renderer, style, &scrollbar);
                }
            },
        );
    } else {
        draw_content(
            renderer,
            content_layout,
            cursor_position,
            &Rectangle {
                x: bounds.x + offset.x,
                y: bounds.y + offset.y,
                ..bounds
            },
        );
    }
}

fn notify_on_scroll<Message>(
    state: &mut State,
    on_scroll: &Option<Box<dyn Fn(RelativeOffset) -> Message + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(on_scroll) = on_scroll {
        if content_bounds.width <= bounds.width
            && content_bounds.height <= bounds.height
        {
            return;
        }

        let x = state.offset_x.absolute(bounds.width, content_bounds.width)
            / (content_bounds.width - bounds.width);

        let y = state
            .offset_y
            .absolute(bounds.height, content_bounds.height)
            / (content_bounds.height - bounds.height);

        let new_offset = RelativeOffset { x, y };

        // Don't publish redundant offsets to shell
        if let Some(prev_offset) = state.last_notified {
            let unchanged = |a: f32, b: f32| {
                (a - b).abs() <= f32::EPSILON || (a.is_nan() && b.is_nan())
            };

            if unchanged(prev_offset.x, new_offset.x)
                && unchanged(prev_offset.y, new_offset.y)
            {
                return;
            }
        }

        shell.publish(on_scroll(new_offset));
        state.last_notified = Some(new_offset);
    }
}

/// The local state of a [`Scrollable`].
#[derive(Debug, Clone, Copy)]
pub struct State {
    scroll_area_touched_at: Option<Point>,
    offset_y: Offset,
    y_scroller_grabbed_at: Option<f32>,
    offset_x: Offset,
    x_scroller_grabbed_at: Option<f32>,
    keyboard_modifiers: keyboard::Modifiers,
    last_notified: Option<RelativeOffset>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            scroll_area_touched_at: None,
            offset_y: Offset::Absolute(0.0),
            y_scroller_grabbed_at: None,
            offset_x: Offset::Absolute(0.0),
            x_scroller_grabbed_at: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            last_notified: None,
        }
    }
}

impl operation::Scrollable for State {
    fn snap_to(&mut self, offset: RelativeOffset) {
        State::snap_to(self, offset);
    }
}

#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(self, window: f32, content: f32) -> f32 {
        match self {
            Offset::Absolute(absolute) => {
                absolute.min((content - window).max(0.0))
            }
            Offset::Relative(percentage) => {
                ((content - window) * percentage).max(0.0)
            }
        }
    }
}

impl State {
    /// Creates a new [`State`] with the scrollbar(s) at the beginning.
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the [`Scrollable`] and its contents.
    pub fn scroll(
        &mut self,
        delta: Vector<f32>,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        if bounds.height < content_bounds.height {
            self.offset_y = Offset::Absolute(
                (self.offset_y.absolute(bounds.height, content_bounds.height)
                    - delta.y)
                    .clamp(0.0, content_bounds.height - bounds.height),
            )
        }

        if bounds.width < content_bounds.width {
            self.offset_x = Offset::Absolute(
                (self.offset_x.absolute(bounds.width, content_bounds.width)
                    - delta.x)
                    .clamp(0.0, content_bounds.width - bounds.width),
            );
        }
    }

    /// Scrolls the [`Scrollable`] to a relative amount along the y axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_y_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset_y = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    /// Scrolls the [`Scrollable`] to a relative amount along the x axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_x_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset_x = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    /// Snaps the scroll position to a [`RelativeOffset`].
    pub fn snap_to(&mut self, offset: RelativeOffset) {
        self.offset_x = Offset::Relative(offset.x.clamp(0.0, 1.0));
        self.offset_y = Offset::Relative(offset.y.clamp(0.0, 1.0));
    }

    /// Unsnaps the current scroll position, if snapped, given the bounds of the
    /// [`Scrollable`] and its contents.
    pub fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_x = Offset::Absolute(
            self.offset_x.absolute(bounds.width, content_bounds.width),
        );
        self.offset_y = Offset::Absolute(
            self.offset_y.absolute(bounds.height, content_bounds.height),
        );
    }

    /// Returns the scrolling offset of the [`State`], given the bounds of the
    /// [`Scrollable`] and its contents.
    pub fn offset(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Vector {
        Vector::new(
            self.offset_x.absolute(bounds.width, content_bounds.width),
            self.offset_y.absolute(bounds.height, content_bounds.height),
        )
    }

    /// Returns whether any scroller is currently grabbed or not.
    pub fn scrollers_grabbed(&self) -> bool {
        self.x_scroller_grabbed_at.is_some()
            || self.y_scroller_grabbed_at.is_some()
    }
}

#[derive(Debug)]
/// State of both [`Scrollbar`]s.
struct Scrollbars {
    y: Option<internals::Scrollbar>,
    x: Option<internals::Scrollbar>,
}

impl Scrollbars {
    /// Create y and/or x scrollbar(s) if content is overflowing the [`Scrollable`] bounds.
    fn new(
        state: &State,
        vertical: &Properties,
        horizontal: Option<&Properties>,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Self {
        let offset = state.offset(bounds, content_bounds);

        let show_scrollbar_x = horizontal.and_then(|h| {
            if content_bounds.width > bounds.width {
                Some(h)
            } else {
                None
            }
        });

        let y_scrollbar = if content_bounds.height > bounds.height {
            let Properties {
                width,
                margin,
                scroller_width,
            } = *vertical;

            // Adjust the height of the vertical scrollbar if the horizontal scrollbar
            // is present
            let x_scrollbar_height = show_scrollbar_x
                .map_or(0.0, |h| h.width.max(h.scroller_width) + h.margin);

            let total_scrollbar_width =
                width.max(scroller_width) + 2.0 * margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width,
                y: bounds.y,
                width: total_scrollbar_width,
                height: (bounds.height - x_scrollbar_height).max(0.0),
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - total_scrollbar_width / 2.0
                    - width / 2.0,
                y: bounds.y,
                width,
                height: (bounds.height - x_scrollbar_height).max(0.0),
            };

            let ratio = bounds.height / content_bounds.height;
            // min height for easier grabbing with super tall content
            let scroller_height = (bounds.height * ratio).max(2.0);
            let scroller_offset = offset.y * ratio;

            let scroller_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - total_scrollbar_width / 2.0
                    - scroller_width / 2.0,
                y: (scrollbar_bounds.y + scroller_offset - x_scrollbar_height)
                    .max(0.0),
                width: scroller_width,
                height: scroller_height,
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller: internals::Scroller {
                    bounds: scroller_bounds,
                },
            })
        } else {
            None
        };

        let x_scrollbar = if let Some(horizontal) = show_scrollbar_x {
            let Properties {
                width,
                margin,
                scroller_width,
            } = *horizontal;

            // Need to adjust the width of the horizontal scrollbar if the vertical scrollbar
            // is present
            let scrollbar_y_width = y_scrollbar.map_or(0.0, |_| {
                vertical.width.max(vertical.scroller_width) + vertical.margin
            });

            let total_scrollbar_height =
                width.max(scroller_width) + 2.0 * margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - total_scrollbar_height,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: total_scrollbar_height,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height
                    - total_scrollbar_height / 2.0
                    - width / 2.0,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: width,
            };

            let ratio = bounds.width / content_bounds.width;
            // min width for easier grabbing with extra wide content
            let scroller_length = (bounds.width * ratio).max(2.0);
            let scroller_offset = offset.x * ratio;

            let scroller_bounds = Rectangle {
                x: (scrollbar_bounds.x + scroller_offset - scrollbar_y_width)
                    .max(0.0),
                y: bounds.y + bounds.height
                    - total_scrollbar_height / 2.0
                    - scroller_width / 2.0,
                width: scroller_length,
                height: scroller_width,
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller: internals::Scroller {
                    bounds: scroller_bounds,
                },
            })
        } else {
            None
        };

        Self {
            y: y_scrollbar,
            x: x_scrollbar,
        }
    }

    fn is_mouse_over(&self, cursor_position: Point) -> (bool, bool) {
        (
            self.y
                .as_ref()
                .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                .unwrap_or(false),
            self.x
                .as_ref()
                .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                .unwrap_or(false),
        )
    }

    fn grab_y_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.y.and_then(|scrollbar| {
            if scrollbar.total_bounds.contains(cursor_position) {
                Some(if scrollbar.scroller.bounds.contains(cursor_position) {
                    (cursor_position.y - scrollbar.scroller.bounds.y)
                        / scrollbar.scroller.bounds.height
                } else {
                    0.5
                })
            } else {
                None
            }
        })
    }

    fn grab_x_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.x.and_then(|scrollbar| {
            if scrollbar.total_bounds.contains(cursor_position) {
                Some(if scrollbar.scroller.bounds.contains(cursor_position) {
                    (cursor_position.x - scrollbar.scroller.bounds.x)
                        / scrollbar.scroller.bounds.width
                } else {
                    0.5
                })
            } else {
                None
            }
        })
    }

    fn active(&self) -> bool {
        self.y.is_some() || self.x.is_some()
    }
}

pub(super) mod internals {
    use crate::core::{Point, Rectangle};

    /// The scrollbar of a [`Scrollable`].
    #[derive(Debug, Copy, Clone)]
    pub struct Scrollbar {
        /// The total bounds of the [`Scrollbar`], including the scrollbar, the scroller,
        /// and the scrollbar margin.
        pub total_bounds: Rectangle,

        /// The bounds of just the [`Scrollbar`].
        pub bounds: Rectangle,

        /// The state of this scrollbar's [`Scroller`].
        pub scroller: Scroller,
    }

    impl Scrollbar {
        /// Returns whether the mouse is over the scrollbar or not.
        pub fn is_mouse_over(&self, cursor_position: Point) -> bool {
            self.total_bounds.contains(cursor_position)
        }

        /// Returns the y-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_y(
            &self,
            grabbed_at: f32,
            cursor_position: Point,
        ) -> f32 {
            if cursor_position.x < 0.0 && cursor_position.y < 0.0 {
                // cursor position is unavailable! Set to either end or beginning of scrollbar depending
                // on where the thumb currently is in the track
                (self.scroller.bounds.y / self.total_bounds.height).round()
            } else {
                (cursor_position.y
                    - self.bounds.y
                    - self.scroller.bounds.height * grabbed_at)
                    / (self.bounds.height - self.scroller.bounds.height)
            }
        }

        /// Returns the x-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_x(
            &self,
            grabbed_at: f32,
            cursor_position: Point,
        ) -> f32 {
            if cursor_position.x < 0.0 && cursor_position.y < 0.0 {
                (self.scroller.bounds.x / self.total_bounds.width).round()
            } else {
                (cursor_position.x
                    - self.bounds.x
                    - self.scroller.bounds.width * grabbed_at)
                    / (self.bounds.width - self.scroller.bounds.width)
            }
        }
    }

    /// The handle of a [`Scrollbar`].
    #[derive(Debug, Clone, Copy)]
    pub struct Scroller {
        /// The bounds of the [`Scroller`].
        pub bounds: Rectangle,
    }
}
