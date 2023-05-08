//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use super::cursor;
pub use super::cursor::Cursor;
use super::editor::Editor;
pub use super::value::Value;

use crate::core::alignment;
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse::{self, click};
use crate::core::renderer;
use crate::core::text::{self, Text};
use crate::core::time::{Duration, Instant};
use crate::core::touch;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::Id;
use crate::core::window;
use crate::core::{
    Clipboard, Color, Element, Layout, Length, Padding, Pixels, Point,
    Rectangle, Shell, Size, Vector, Widget,
};
use crate::runtime::Command;
use iced_renderer::core::event::{wayland, PlatformSpecific};
use iced_renderer::core::widget::OperationOutputWrapper;

use iced_runtime::command::platform_specific;
use iced_runtime::command::platform_specific::wayland::data_device::{
    DataFromMimeType, DndIcon,
};
pub use iced_style::text_input::{Appearance, StyleSheet};
use sctk::reexports::client::protocol::wl_data_device_manager::DndAction;

const SUPPORTED_MIME_TYPES: &'static [&'static str; 6] = &[
    "text/plain;charset=utf-8",
    "text/plain;charset=UTF-8",
    "UTF8_STRING",
    "STRING",
    "text/plain",
    "TEXT",
];

/// A field that can be filled with text.
///
/// # Example
/// ```no_run
/// # pub type TextInput<'a, Message> =
/// #     iced_widget::TextInput<'a, Message, iced_widget::renderer::Renderer<iced_widget::style::Theme>>;
/// #
/// #[derive(Debug, Clone)]
/// enum Message {
///     TextInputChanged(String),
/// }
///
/// let value = "Some text";
///
/// let input = TextInput::new(
///     "This is the placeholder...",
///     value,
/// )
/// .on_input(Message::TextInputChanged)
/// .padding(10);
/// ```
/// ![Text input drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text_input.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    placeholder: String,
    value: Value,
    is_secure: bool,
    font: Option<Renderer::Font>,
    width: Length,
    padding: Padding,
    size: Option<f32>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    icon: Option<Icon<Renderer::Font>>,
    style: <Renderer::Theme as StyleSheet>::Style,
    // (text_input::State, mime_type, dnd_action) -> Message
    on_create_dnd_source: Option<Box<dyn Fn(State) -> Message + 'a>>,
    on_dnd_command_produced:
        Option<Box<dyn Fn(Box<dyn Send + Sync + Fn() -> platform_specific::wayland::data_device::ActionInner>) -> Message + 'a>>,
    surface_ids: Option<(window::Id, window::Id)>,
    dnd_icon: bool,
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - a placeholder,
    /// - the current value
    pub fn new(placeholder: &str, value: &str) -> Self {
        TextInput {
            id: None,
            placeholder: String::from(placeholder),
            value: Value::new(value),
            is_secure: false,
            font: None,
            width: Length::Fill,
            padding: Padding::new(5.0),
            size: None,
            on_input: None,
            on_paste: None,
            on_submit: None,
            icon: None,
            style: Default::default(),
            on_dnd_command_produced: None,
            on_create_dnd_source: None,
            surface_ids: None,
            dnd_icon: false,
        }
    }

    /// Sets the [`Id`] of the [`TextInput`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Converts the [`TextInput`] into a secure password input.
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`].
    ///
    /// If this method is not called, the [`TextInput`] will be disabled.
    pub fn on_input<F>(mut self, callback: F) -> Self
    where
        F: 'a + Fn(String) -> Message,
    {
        self.on_input = Some(Box::new(callback));
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`].
    pub fn on_paste(
        mut self,
        on_paste: impl Fn(String) -> Message + 'a,
    ) -> Self {
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the [`Font`] of the [`TextInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the [`Icon`] of the [`TextInput`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into().0);
        self
    }

    /// Sets the style of the [`TextInput`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        layout: Layout<'_>,
        cursor_position: Point,
        value: Option<&Value>,
    ) {
        draw(
            renderer,
            theme,
            layout,
            cursor_position,
            tree.state.downcast_ref::<State>(),
            value.unwrap_or(&self.value),
            &self.placeholder,
            self.size,
            self.font,
            self.on_input.is_none(),
            self.is_secure,
            self.icon.as_ref(),
            &self.style,
            self.dnd_icon,
        )
    }

    /// Sets the on_start_dnd handler of the [`TextInput`].
    pub fn on_start_dnd(
        mut self,
        on_start_dnd: impl Fn(State) -> Message + 'a,
    ) -> Self {
        self.on_create_dnd_source = Some(Box::new(on_start_dnd));
        self
    }

    /// Sets the on_dnd_command_produced handler of the [`TextInput`].
    /// Commands should be returned in the update function of the application.
    pub fn on_dnd_command_produced(
        mut self,
        on_dnd_command_produced: impl Fn(Box<dyn Send + Sync + Fn() -> platform_specific::wayland::data_device::ActionInner>) -> Message + 'a,
    ) -> Self {
        self.on_dnd_command_produced = Some(Box::new(on_dnd_command_produced));
        self
    }

    /// Sets the window id of the [`TextInput`] and the window_id of the drag icon.
    /// Both ids are required to be unique.
    /// This is required for the dnd to work.
    pub fn surface_ids(mut self, window_id: (window::Id, window::Id)) -> Self {
        self.surface_ids = Some(window_id);
        self
    }

    /// Sets the mode of this [`TextInput`] to be a drag and drop icon.
    pub fn dnd_icon(mut self, dnd_icon: bool) -> Self {
        self.dnd_icon = dnd_icon;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Unfocus text input if it becomes disabled
        if self.on_input.is_none() {
            state.last_click = None;
            state.is_focused = None;
            state.is_pasting = None;
            state.dragging_state = None;
        }
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if self.dnd_icon {
            let limits = limits.width(Length::Shrink).height(Length::Shrink);

            let size = self.size.unwrap_or_else(|| renderer.default_size());

            let bounds = limits.max();
            let font = self.font.unwrap_or_else(|| renderer.default_font());

            let (width, height) =
                renderer.measure(&self.value.to_string(), size, font, bounds);

            let size = limits.resolve(Size::new(width, height));
            layout::Node::with_children(size, vec![layout::Node::new(size)])
        } else {
            layout(
                renderer,
                limits,
                self.width,
                self.padding,
                self.size,
                self.icon.as_ref(),
            )
        }
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.focusable(state, self.id.as_ref());
        operation.text_input(state, self.id.as_ref());
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
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
            &mut self.value,
            self.size,
            self.font,
            self.is_secure,
            self.on_input.as_deref(),
            self.on_paste.as_deref(),
            &self.on_submit,
            || tree.state.downcast_mut::<State>(),
            self.on_create_dnd_source.as_deref(),
            self.dnd_icon,
            self.on_dnd_command_produced.as_deref(),
            self.surface_ids.clone(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            theme,
            layout,
            cursor_position,
            tree.state.downcast_ref::<State>(),
            &self.value,
            &self.placeholder,
            self.size,
            self.font,
            self.on_input.is_none(),
            self.is_secure,
            self.icon.as_ref(),
            &self.style,
            self.dnd_icon,
        )
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor_position, self.on_input.is_none())
    }
}

impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        text_input: TextInput<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}

/// The content of the [`Icon`].
#[derive(Debug, Clone)]
pub struct Icon<Font> {
    /// The font that will be used to display the `code_point`.
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// The font size of the content.
    pub size: Option<f32>,
    /// The spacing between the [`Icon`] and the text in a [`TextInput`].
    pub spacing: f32,
    /// The side of a [`TextInput`] where to display the [`Icon`].
    pub side: Side,
}

/// The side of a [`TextInput`].
#[derive(Debug, Clone)]
pub enum Side {
    /// The left side of a [`TextInput`].
    Left,
    /// The right side of a [`TextInput`].
    Right,
}

/// Produces a [`Command`] that focuses the [`TextInput`] with the given [`Id`].
pub fn focus<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::focusable::focus(id))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// end.
pub fn move_cursor_to_end<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to_end(id))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// front.
pub fn move_cursor_to_front<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to_front(id))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// provided position.
pub fn move_cursor_to<Message: 'static>(
    id: Id,
    position: usize,
) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to(id, position))
}

/// Produces a [`Command`] that selects all the content of the [`TextInput`] with the given [`Id`].
pub fn select_all<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::select_all(id))
}

/// Computes the layout of a [`TextInput`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    padding: Padding,
    size: Option<f32>,
    icon: Option<&Icon<Renderer::Font>>,
) -> layout::Node
where
    Renderer: text::Renderer,
{
    let text_size = size.unwrap_or_else(|| renderer.default_size());
    let padding = padding.fit(Size::ZERO, limits.max());
    let limits = limits.width(width).pad(padding).height(text_size * 1.2);

    let text_bounds = limits.resolve(Size::ZERO);

    if let Some(icon) = icon {
        let icon_width = renderer.measure_width(
            &icon.code_point.to_string(),
            icon.size.unwrap_or_else(|| renderer.default_size()),
            icon.font,
        );

        let mut text_node = layout::Node::new(
            text_bounds - Size::new(icon_width + icon.spacing, 0.0),
        );

        let mut icon_node =
            layout::Node::new(Size::new(icon_width, text_bounds.height));

        match icon.side {
            Side::Left => {
                text_node.move_to(Point::new(
                    padding.left + icon_width + icon.spacing,
                    padding.top,
                ));

                icon_node.move_to(Point::new(padding.left, padding.top));
            }
            Side::Right => {
                text_node.move_to(Point::new(padding.left, padding.top));

                icon_node.move_to(Point::new(
                    padding.left + text_bounds.width - icon_width,
                    padding.top,
                ));
            }
        };

        layout::Node::with_children(
            text_bounds.pad(padding),
            vec![text_node, icon_node],
        )
    } else {
        let mut text = layout::Node::new(text_bounds);
        text.move_to(Point::new(padding.left, padding.top));

        layout::Node::with_children(text_bounds.pad(padding), vec![text])
    }
}

/// Processes an [`Event`] and updates the [`State`] of a [`TextInput`]
/// accordingly.
pub fn update<'a, Message, Renderer>(
    event: Event,
    layout: Layout<'_>,
    cursor_position: Point,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    value: &mut Value,
    size: Option<f32>,
    font: Option<Renderer::Font>,
    is_secure: bool,
    on_input: Option<&dyn Fn(String) -> Message>,
    on_paste: Option<&dyn Fn(String) -> Message>,
    on_submit: &Option<Message>,
    state: impl FnOnce() -> &'a mut State,
    on_start_dnd_source: Option<&dyn Fn(State) -> Message>,
    dnd_icon: bool,
    on_dnd_command_produced: Option<
        &dyn Fn(
            Box<
                dyn Send
                    + Sync
                    + Fn() -> platform_specific::wayland::data_device::ActionInner,
            >,
        ) -> Message,
    >,
    surface_ids: Option<(window::Id, window::Id)>,
) -> event::Status
where
    Message: Clone,
    Renderer: text::Renderer,
{
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let state = state();
            let is_clicked =
                layout.bounds().contains(cursor_position) && on_input.is_some();

            state.is_focused = if is_clicked {
                state.is_focused.or_else(|| {
                    let now = Instant::now();
                    Some(Focus {
                        updated_at: now,
                        now,
                    })
                })
            } else {
                None
            };

            let font: <Renderer as text::Renderer>::Font =
                font.unwrap_or_else(|| renderer.default_font());
            if is_clicked {
                let text_layout = layout.children().next().unwrap();
                let target = cursor_position.x - text_layout.bounds().x;

                let click =
                    mouse::Click::new(cursor_position, state.last_click);

                match (
                    &state.dragging_state,
                    click.kind(),
                    state.cursor().state(value),
                ) {
                    (
                        None,
                        click::Kind::Single,
                        cursor::State::Selection { start, end },
                    ) => {
                        // if something is already selected, we can start a drag and drop for a
                        // single click that is on top of the selected text
                        // is the click on selected text?
                        if is_secure {
                            return event::Status::Ignored;
                        }
                        if let (
                            Some(on_start_dnd),
                            Some(on_dnd_command_produced),
                            Some((window_id, icon_id)),
                            Some(on_input),
                        ) = (
                            on_start_dnd_source,
                            on_dnd_command_produced,
                            surface_ids,
                            on_input,
                        ) {
                            let text_bounds =
                                layout.children().next().unwrap().bounds();
                            let actual_size =
                                size.unwrap_or_else(|| renderer.default_size());

                            let left = start.min(end);
                            let right = end.max(start);

                            let (left_position, _left_offset) =
                                measure_cursor_and_scroll_offset(
                                    renderer,
                                    text_bounds,
                                    value,
                                    actual_size,
                                    left,
                                    font.clone(),
                                );

                            let (right_position, _right_offset) =
                                measure_cursor_and_scroll_offset(
                                    renderer,
                                    text_bounds,
                                    value,
                                    actual_size,
                                    right,
                                    font.clone(),
                                );

                            let width = right_position - left_position;
                            let selection_bounds = Rectangle {
                                x: text_bounds.x + left_position,
                                y: text_bounds.y,
                                width,
                                height: text_bounds.height,
                            };

                            if selection_bounds.contains(cursor_position) {
                                let text = state
                                    .selected_text(&value.to_string())
                                    .unwrap_or_default();
                                state.dragging_state =
                                    Some(DraggingState::Dnd(
                                        DndAction::empty(),
                                        text.clone(),
                                    ));
                                let mut editor =
                                    Editor::new(value, &mut state.cursor);
                                editor.delete();

                                let message = (on_input)(editor.contents());
                                shell.publish(message);
                                shell.publish(on_start_dnd(state.clone()));
                                let state = state.clone();
                                shell.publish(on_dnd_command_produced(Box::new(move || {
                                    platform_specific::wayland::data_device::ActionInner::StartDnd {
                                        mime_types: SUPPORTED_MIME_TYPES.iter().map(|t| t.to_string()).collect(),
                                        actions: DndAction::Move,
                                        origin_id: window_id.clone(),
                                        icon_id: Some(
                                            DndIcon::Widget(
                                                icon_id.clone(),
                                                Box::new(state.clone())
                                                )),
                                        data: Box::new(TextInputString(text.clone()))
                                    }
                                    })));
                            } else {
                                // existing logic for setting the selection
                                let position = if target > 0.0 {
                                    let value = if is_secure {
                                        value.secure()
                                    } else {
                                        value.clone()
                                    };

                                    find_cursor_position(
                                        renderer,
                                        text_layout.bounds(),
                                        font.clone(),
                                        size,
                                        &value,
                                        state,
                                        target,
                                    )
                                } else {
                                    None
                                };

                                state.cursor.move_to(position.unwrap_or(0));
                                state.dragging_state =
                                    Some(DraggingState::Selection);
                            }
                        } else {
                            state.dragging_state = None;
                        }
                    }
                    (Some(DraggingState::Dnd(..)), _, _) => {
                        // TODO: should we cancel if this happens?
                        state.dragging_state = None;
                    }
                    (None, click::Kind::Single, _) => {
                        // existing logic for setting the selection
                        let position = if target > 0.0 {
                            let value = if is_secure {
                                value.secure()
                            } else {
                                value.clone()
                            };

                            find_cursor_position(
                                renderer,
                                text_layout.bounds(),
                                font.clone(),
                                size,
                                &value,
                                state,
                                target,
                            )
                        } else {
                            None
                        };

                        state.cursor.move_to(position.unwrap_or(0));
                        state.dragging_state = Some(DraggingState::Selection);
                    }
                    (None, click::Kind::Double, _)
                    | (
                        Some(DraggingState::Selection),
                        click::Kind::Double,
                        _,
                    ) => {
                        if is_secure {
                            state.cursor.select_all(value);
                        } else {
                            let position = find_cursor_position(
                                renderer,
                                text_layout.bounds(),
                                font.clone(),
                                size,
                                value,
                                state,
                                target,
                            )
                            .unwrap_or(0);

                            state.cursor.select_range(
                                value.previous_start_of_word(position),
                                value.next_end_of_word(position),
                            );
                        }
                        state.dragging_state = Some(DraggingState::Selection);
                    }
                    (None, click::Kind::Triple, _)
                    | (
                        Some(DraggingState::Selection),
                        click::Kind::Triple,
                        _,
                    ) => {
                        state.cursor.select_all(value);
                        state.dragging_state = Some(DraggingState::Selection);
                    }
                    _ => {
                        state.dragging_state = None;
                    }
                }

                state.last_click = Some(click);

                return event::Status::Captured;
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. })
        | Event::Touch(touch::Event::FingerLost { .. }) => {
            let state = state();
            state.dragging_state = None;
        }
        Event::Mouse(mouse::Event::CursorMoved { position })
        | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
            let state = state();

            if matches!(state.dragging_state, Some(DraggingState::Selection)) {
                let text_layout = layout.children().next().unwrap();
                let target = position.x - text_layout.bounds().x;

                let value = if is_secure {
                    value.secure()
                } else {
                    value.clone()
                };
                let font: <Renderer as text::Renderer>::Font =
                    font.unwrap_or_else(|| renderer.default_font());

                let position = find_cursor_position(
                    renderer,
                    text_layout.bounds(),
                    font,
                    size,
                    &value,
                    state,
                    target,
                )
                .unwrap_or(0);

                state
                    .cursor
                    .select_range(state.cursor.start(&value), position);

                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::CharacterReceived(c)) => {
            let state = state();

            if let Some(focus) = &mut state.is_focused {
                let Some(on_input) = on_input else { return event::Status::Ignored };

                if state.is_pasting.is_none()
                    && !state.keyboard_modifiers.command()
                    && !c.is_control()
                {
                    let mut editor = Editor::new(value, &mut state.cursor);

                    editor.insert(c);

                    let message = (on_input)(editor.contents());
                    shell.publish(message);

                    focus.updated_at = Instant::now();

                    return event::Status::Captured;
                }
            }
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
            let state = state();

            if let Some(focus) = &mut state.is_focused {
                let Some(on_input) = on_input else { return event::Status::Ignored };

                let modifiers = state.keyboard_modifiers;
                focus.updated_at = Instant::now();

                match key_code {
                    keyboard::KeyCode::Enter
                    | keyboard::KeyCode::NumpadEnter => {
                        if let Some(on_submit) = on_submit.clone() {
                            shell.publish(on_submit);
                        }
                    }
                    keyboard::KeyCode::Backspace => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && state.cursor.selection(value).is_none()
                        {
                            if is_secure {
                                let cursor_pos = state.cursor.end(value);
                                state.cursor.select_range(0, cursor_pos);
                            } else {
                                state.cursor.select_left_by_words(value);
                            }
                        }

                        let mut editor = Editor::new(value, &mut state.cursor);
                        editor.backspace();

                        let message = (on_input)(editor.contents());
                        shell.publish(message);
                    }
                    keyboard::KeyCode::Delete => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && state.cursor.selection(value).is_none()
                        {
                            if is_secure {
                                let cursor_pos = state.cursor.end(value);
                                state
                                    .cursor
                                    .select_range(cursor_pos, value.len());
                            } else {
                                state.cursor.select_right_by_words(value);
                            }
                        }

                        let mut editor = Editor::new(value, &mut state.cursor);
                        editor.delete();

                        let message = (on_input)(editor.contents());
                        shell.publish(message);
                    }
                    keyboard::KeyCode::Left => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && !is_secure
                        {
                            if modifiers.shift() {
                                state.cursor.select_left_by_words(value);
                            } else {
                                state.cursor.move_left_by_words(value);
                            }
                        } else if modifiers.shift() {
                            state.cursor.select_left(value)
                        } else {
                            state.cursor.move_left(value);
                        }
                    }
                    keyboard::KeyCode::Right => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && !is_secure
                        {
                            if modifiers.shift() {
                                state.cursor.select_right_by_words(value);
                            } else {
                                state.cursor.move_right_by_words(value);
                            }
                        } else if modifiers.shift() {
                            state.cursor.select_right(value)
                        } else {
                            state.cursor.move_right(value);
                        }
                    }
                    keyboard::KeyCode::Home => {
                        if modifiers.shift() {
                            state
                                .cursor
                                .select_range(state.cursor.start(value), 0);
                        } else {
                            state.cursor.move_to(0);
                        }
                    }
                    keyboard::KeyCode::End => {
                        if modifiers.shift() {
                            state.cursor.select_range(
                                state.cursor.start(value),
                                value.len(),
                            );
                        } else {
                            state.cursor.move_to(value.len());
                        }
                    }
                    keyboard::KeyCode::C
                        if state.keyboard_modifiers.command() =>
                    {
                        if let Some((start, end)) =
                            state.cursor.selection(value)
                        {
                            clipboard
                                .write(value.select(start, end).to_string());
                        }
                    }
                    keyboard::KeyCode::X
                        if state.keyboard_modifiers.command() =>
                    {
                        if let Some((start, end)) =
                            state.cursor.selection(value)
                        {
                            clipboard
                                .write(value.select(start, end).to_string());
                        }

                        let mut editor = Editor::new(value, &mut state.cursor);
                        editor.delete();

                        let message = (on_input)(editor.contents());
                        shell.publish(message);
                    }
                    keyboard::KeyCode::V => {
                        if state.keyboard_modifiers.command() {
                            let content = match state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .read()
                                        .unwrap_or_default()
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    Value::new(&content)
                                }
                            };

                            let mut editor =
                                Editor::new(value, &mut state.cursor);

                            editor.paste(content.clone());

                            let message = if let Some(paste) = &on_paste {
                                (paste)(editor.contents())
                            } else {
                                (on_input)(editor.contents())
                            };
                            shell.publish(message);

                            state.is_pasting = Some(content);
                        } else {
                            state.is_pasting = None;
                        }
                    }
                    keyboard::KeyCode::A
                        if state.keyboard_modifiers.command() =>
                    {
                        state.cursor.select_all(value);
                    }
                    keyboard::KeyCode::Escape => {
                        state.is_focused = None;
                        state.dragging_state = None;
                        state.is_pasting = None;

                        state.keyboard_modifiers =
                            keyboard::Modifiers::default();
                    }
                    keyboard::KeyCode::Tab
                    | keyboard::KeyCode::Up
                    | keyboard::KeyCode::Down => {
                        return event::Status::Ignored;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {
            let state = state();

            if state.is_focused.is_some() {
                match key_code {
                    keyboard::KeyCode::V => {
                        state.is_pasting = None;
                    }
                    keyboard::KeyCode::Tab
                    | keyboard::KeyCode::Up
                    | keyboard::KeyCode::Down => {
                        return event::Status::Ignored;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            } else {
                state.is_pasting = None;
            }
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            let state = state();

            state.keyboard_modifiers = modifiers;
        }
        Event::Window(_, window::Event::RedrawRequested(now)) => {
            let state = state();

            if let Some(focus) = &mut state.is_focused {
                focus.now = now;

                let millis_until_redraw = CURSOR_BLINK_INTERVAL_MILLIS
                    - (now - focus.updated_at).as_millis()
                        % CURSOR_BLINK_INTERVAL_MILLIS;

                shell.request_redraw(window::RedrawRequest::At(
                    now + Duration::from_millis(millis_until_redraw as u64),
                ));
            }
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DataSource(wayland::DataSourceEvent::DndFinished),
        )) => {
            let state = state();
            if matches!(state.dragging_state, Some(DraggingState::Dnd(..))) {
                state.dragging_state = None;
                return event::Status::Captured;
            }
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DataSource(wayland::DataSourceEvent::Cancelled),
        )) => {
            let state = state();
            if matches!(state.dragging_state, Some(DraggingState::Dnd(..))) {
                state.dragging_state = None;
                return event::Status::Captured;
            }
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DataSource(
                wayland::DataSourceEvent::DndActionAccepted(action),
            ),
        )) => {
            let state = state();
            if let Some(DraggingState::Dnd(_, text)) =
                state.dragging_state.as_ref()
            {
                state.dragging_state =
                    Some(DraggingState::Dnd(action, text.clone()));
                return event::Status::Captured;
            }
        }
        // TODO: handle dnd offer events
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DndOffer(wayland::DndOfferEvent::Enter {
                x,
                y,
                mime_types,
            }),
        )) => {
            let on_dnd_command_produced = match on_dnd_command_produced {
                Some(on_dnd_command_produced) => on_dnd_command_produced,
                None => return event::Status::Ignored,
            };

            let state = state();
            let bounds = layout.bounds();
            let is_clicked = bounds.contains(Point {
                x: x as f32,
                y: y as f32,
            });

            if !is_clicked
                && matches!(state.dnd_offer, DndOfferState::HandlingOffer(..))
            {
                state.dnd_offer =
                    DndOfferState::OutsideWidget(mime_types, DndAction::None);
                return event::Status::Captured;
            } else if !is_clicked {
                state.dnd_offer =
                    DndOfferState::OutsideWidget(mime_types, DndAction::None);
                return event::Status::Captured;
            }
            let mut accepted = false;
            for m in &mime_types {
                if SUPPORTED_MIME_TYPES.contains(&m.as_str()) {
                    let clone = m.clone();
                    accepted = true;
                    shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::Accept(Some(clone.clone())))));
                }
            }
            if accepted {
                shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::SetActions { preferred: DndAction::Move, accepted: DndAction::Move.union(DndAction::Copy) })));
                let text_layout = layout.children().next().unwrap();
                let target = x as f32 - text_layout.bounds().x;
                state.dnd_offer = DndOfferState::HandlingOffer(
                    mime_types.clone(),
                    DndAction::None,
                );
                // existing logic for setting the selection
                let position = if target > 0.0 {
                    let value = if is_secure {
                        value.secure()
                    } else {
                        value.clone()
                    };

                    let font = font.unwrap_or_else(|| renderer.default_font());

                    find_cursor_position(
                        renderer,
                        text_layout.bounds(),
                        font.clone(),
                        size,
                        &value,
                        state,
                        target,
                    )
                } else {
                    None
                };

                state.cursor.move_to(position.unwrap_or(0));
                return event::Status::Captured;
            }
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DndOffer(wayland::DndOfferEvent::Motion { x, y }),
        )) => {
            let on_dnd_command_produced = match on_dnd_command_produced {
                Some(on_dnd_command_produced) => on_dnd_command_produced,
                None => return event::Status::Ignored,
            };

            let state = state();
            let bounds = layout.bounds();
            let is_clicked = bounds.contains(Point {
                x: x as f32,
                y: y as f32,
            });

            if !is_clicked {
                if let DndOfferState::HandlingOffer(mime_types, action) =
                    state.dnd_offer.clone()
                {
                    state.dnd_offer =
                        DndOfferState::OutsideWidget(mime_types, action);
                    shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::SetActions { preferred: DndAction::None, accepted: DndAction::None })));
                    shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::Accept(None))));
                }
                return event::Status::Captured;
            } else if let DndOfferState::OutsideWidget(mime_types, action) =
                state.dnd_offer.clone()
            {
                let mut accepted = false;
                for m in &mime_types {
                    if SUPPORTED_MIME_TYPES.contains(&m.as_str()) {
                        accepted = true;
                        let clone = m.clone();
                        shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::Accept(Some(clone.clone())))));
                    }
                }
                if accepted {
                    shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::SetActions { preferred: DndAction::Move, accepted: DndAction::Move.union(DndAction::Copy) })));
                    state.dnd_offer = DndOfferState::HandlingOffer(
                        mime_types.clone(),
                        action,
                    );
                }
            };
            let text_layout = layout.children().next().unwrap();
            let target = x as f32 - text_layout.bounds().x;
            // existing logic for setting the selection
            let position = if target > 0.0 {
                let value = if is_secure {
                    value.secure()
                } else {
                    value.clone()
                };
                let font = font.unwrap_or_else(|| renderer.default_font());

                find_cursor_position(
                    renderer,
                    text_layout.bounds(),
                    font.clone(),
                    size,
                    &value,
                    state,
                    target,
                )
            } else {
                None
            };

            state.cursor.move_to(position.unwrap_or(0));
            return event::Status::Captured;
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DndOffer(wayland::DndOfferEvent::DropPerformed),
        )) => {
            let on_dnd_command_produced = match on_dnd_command_produced {
                Some(on_dnd_command_produced) => on_dnd_command_produced,
                None => return event::Status::Ignored,
            };

            let state = state();
            if let DndOfferState::HandlingOffer(mime_types, _action) =
                state.dnd_offer.clone()
            {
                let mime_type = match SUPPORTED_MIME_TYPES
                    .iter()
                    .find(|m| mime_types.contains(&m.to_string()))
                {
                    Some(m) => m.clone(),
                    None => {
                        state.dnd_offer = DndOfferState::None;
                        return event::Status::Captured;
                    }
                }
                .to_string();
                state.dnd_offer = DndOfferState::Dropped;
                shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::RequestDndData(mime_type.clone()))));
            } else if let DndOfferState::OutsideWidget(..) = &state.dnd_offer {
                state.dnd_offer = DndOfferState::None;
                return event::Status::Captured;
            }
            return event::Status::Ignored;
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DndOffer(wayland::DndOfferEvent::Leave),
        )) => {
            let state = state();
            // ASHLEY TODO we should be able to reset but for now we don't if we are handling a
            // drop
            match state.dnd_offer {
                DndOfferState::Dropped => {}
                _ => {
                    state.dnd_offer = DndOfferState::None;
                }
            };
            return event::Status::Captured;
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DndOffer(wayland::DndOfferEvent::DndData {
                mime_type,
                data,
            }),
        )) => {
            let on_dnd_command_produced = match on_dnd_command_produced {
                Some(on_dnd_command_produced) => on_dnd_command_produced,
                None => return event::Status::Ignored,
            };

            let state = state();
            if let DndOfferState::Dropped = state.dnd_offer.clone() {
                state.dnd_offer = DndOfferState::None;
                if !SUPPORTED_MIME_TYPES.contains(&mime_type.as_str())
                    || data.is_empty()
                {
                    return event::Status::Captured;
                }
                let content = match String::from_utf8(data) {
                    Ok(text) => text,
                    Err(_) => return event::Status::Captured,
                };

                let mut editor = Editor::new(value, &mut state.cursor);

                editor.paste(Value::new(content.as_str()));
                if let Some(on_paste) = on_paste.as_ref() {
                    let message = (on_paste)(editor.contents());
                    shell.publish(message);
                }
                if let Some(on_paste) = on_paste {
                    let message = (on_paste)(editor.contents());
                    shell.publish(message);
                }

                shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::DndFinished)));
                return event::Status::Captured;
            }
            return event::Status::Ignored;
        }
        Event::PlatformSpecific(PlatformSpecific::Wayland(
            wayland::Event::DndOffer(wayland::DndOfferEvent::SourceActions(
                actions,
            )),
        )) => {
            let on_dnd_command_produced = match on_dnd_command_produced {
                Some(on_dnd_command_produced) => on_dnd_command_produced,
                None => return event::Status::Ignored,
            };

            let state = state();
            if let DndOfferState::HandlingOffer(..) = state.dnd_offer.clone() {
                shell.publish(on_dnd_command_produced(Box::new(move || platform_specific::wayland::data_device::ActionInner::SetActions { preferred: actions.intersection(DndAction::Move), accepted: actions.clone() })));
                return event::Status::Captured;
            }
            return event::Status::Ignored;
        }
        _ => {}
    }

    event::Status::Ignored
}

/// Draws the [`TextInput`] with the given [`Renderer`], overriding its
/// [`Value`] if provided.
///
/// [`Renderer`]: text::Renderer
pub fn draw<Renderer>(
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor_position: Point,
    state: &State,
    value: &Value,
    placeholder: &str,
    size: Option<f32>,
    font: Option<Renderer::Font>,
    is_disabled: bool,
    is_secure: bool,
    icon: Option<&Icon<Renderer::Font>>,
    style: &<Renderer::Theme as StyleSheet>::Style,
    dnd_icon: bool,
) where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    let secure_value = is_secure.then(|| value.secure());
    let value = secure_value.as_ref().unwrap_or(value);

    let bounds = layout.bounds();

    let mut children_layout = layout.children();
    let text_bounds = children_layout.next().unwrap().bounds();

    let is_mouse_over = bounds.contains(cursor_position);

    let appearance = if is_disabled {
        theme.disabled(style)
    } else if state.is_focused() {
        theme.focused(style)
    } else if is_mouse_over {
        theme.hovered(style)
    } else {
        theme.active(style)
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border_radius: appearance.border_radius.into(),
            border_width: appearance.border_width,
            border_color: appearance.border_color,
        },
        appearance.background,
    );

    if let Some(icon) = icon {
        let icon_layout = children_layout.next().unwrap();

        renderer.fill_text(Text {
            content: &icon.code_point.to_string(),
            size: icon.size.unwrap_or_else(|| renderer.default_size()),
            font: icon.font,
            color: appearance.icon_color,
            bounds: icon_layout.bounds(),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
        });
    }

    let text = value.to_string();
    let font = font.unwrap_or_else(|| renderer.default_font());
    let size = size.unwrap_or_else(|| renderer.default_size());

    let (cursor, offset) = if let Some(focus) = &state.is_focused {
        match state.cursor.state(value) {
            cursor::State::Index(position) => {
                let (text_value_width, offset) =
                    measure_cursor_and_scroll_offset(
                        renderer,
                        text_bounds,
                        value,
                        size,
                        position,
                        font,
                    );

                let is_cursor_visible = ((focus.now - focus.updated_at)
                    .as_millis()
                    / CURSOR_BLINK_INTERVAL_MILLIS)
                    % 2
                    == 0;

                if is_cursor_visible {
                    if !dnd_icon {
                        (
                            Some((
                                renderer::Quad {
                                    bounds: Rectangle {
                                        x: text_bounds.x + text_value_width,
                                        y: text_bounds.y,
                                        width: 1.0,
                                        height: text_bounds.height,
                                    },
                                    border_radius: 0.0.into(),
                                    border_width: 0.0,
                                    border_color: Color::TRANSPARENT,
                                },
                                theme.value_color(style),
                            )),
                            offset,
                        )
                    } else {
                        (None, 0.0)
                    }
                } else {
                    (None, 0.0)
                }
            }
            cursor::State::Selection { start, end } => {
                let left = start.min(end);
                let right = end.max(start);

                let (left_position, left_offset) =
                    measure_cursor_and_scroll_offset(
                        renderer,
                        text_bounds,
                        value,
                        size,
                        left,
                        font,
                    );

                let (right_position, right_offset) =
                    measure_cursor_and_scroll_offset(
                        renderer,
                        text_bounds,
                        value,
                        size,
                        right,
                        font,
                    );

                let width = right_position - left_position;

                if !dnd_icon {
                    (
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + left_position,
                                    y: text_bounds.y,
                                    width,
                                    height: text_bounds.height,
                                },
                                border_radius: 0.0.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            theme.selection_color(style),
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                    )
                } else {
                    (None, 0.0)
                }
            }
        }
    } else {
        (None, 0.0)
    };

    let text_width = renderer.measure_width(
        if text.is_empty() { placeholder } else { &text },
        size,
        font,
    );

    let render = |renderer: &mut Renderer| {
        if let Some((cursor, color)) = cursor {
            renderer.fill_quad(cursor, color);
        } else {
            renderer.with_translation(Vector::ZERO, |_| {});
        }

        renderer.fill_text(Text {
            content: if text.is_empty() { placeholder } else { &text },
            color: if text.is_empty() {
                theme.placeholder_color(style)
            } else if is_disabled {
                theme.disabled_color(style)
            } else {
                theme.value_color(style)
            },
            font,
            bounds: Rectangle {
                y: text_bounds.center_y(),
                width: f32::INFINITY,
                ..text_bounds
            },
            size,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
        });
    };

    if text_width > text_bounds.width {
        renderer.with_layer(text_bounds, |renderer| {
            renderer.with_translation(Vector::new(-offset, 0.0), render)
        });
    } else {
        render(renderer);
    }
}

/// Computes the current [`mouse::Interaction`] of the [`TextInput`].
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor_position: Point,
    is_disabled: bool,
) -> mouse::Interaction {
    if layout.bounds().contains(cursor_position) {
        if is_disabled {
            mouse::Interaction::NotAllowed
        } else {
            mouse::Interaction::Text
        }
    } else {
        mouse::Interaction::default()
    }
}

/// A string which can be sent to the clipboard or drag-and-dropped.
#[derive(Debug, Clone)]
pub struct TextInputString(String);

impl DataFromMimeType for TextInputString {
    fn from_mime_type(&self, mime_type: &str) -> Option<Vec<u8>> {
        if SUPPORTED_MIME_TYPES.contains(&mime_type) {
            Some(self.0.as_bytes().to_vec())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DraggingState {
    Selection,
    Dnd(DndAction, String),
}

#[derive(Debug, Default, Clone)]
pub(crate) enum DndOfferState {
    #[default]
    None,
    OutsideWidget(Vec<String>, DndAction),
    HandlingOffer(Vec<String>, DndAction),
    Dropped,
}

/// The state of a [`TextInput`].
#[derive(Debug, Default, Clone)]
pub struct State {
    is_focused: Option<Focus>,
    dragging_state: Option<DraggingState>,
    dnd_offer: DndOfferState,
    is_pasting: Option<Value>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    // TODO: Add stateful horizontal scrolling offset
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current value of the selected text in the [`TextInput`].
    pub fn selected_text(&self, text: &str) -> Option<String> {
        let value = Value::new(text);
        match self.cursor.state(&value) {
            cursor::State::Index(_) => None,
            cursor::State::Selection { start, end } => {
                let left = start.min(end);
                let right = end.max(start);
                Some(text[left..right].to_string())
            }
        }
    }

    /// Returns the current value of the dragged text in the [`TextInput`].
    pub fn dragged_text(&self) -> Option<String> {
        match self.dragging_state.as_ref() {
            Some(DraggingState::Dnd(_, text)) => Some(text.clone()),
            _ => None,
        }
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    pub fn focused() -> Self {
        Self {
            is_focused: None,
            dragging_state: None,
            dnd_offer: DndOfferState::None,
            is_pasting: None,
            last_click: None,
            cursor: Cursor::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    /// Returns the [`Cursor`] of the [`TextInput`].
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`TextInput`].
    pub fn focus(&mut self) {
        let now = Instant::now();

        self.is_focused = Some(Focus {
            updated_at: now,
            now,
        });

        self.move_cursor_to_end();
    }

    /// Unfocuses the [`TextInput`].
    pub fn unfocus(&mut self) {
        self.is_focused = None;
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the front of the input text.
    pub fn move_cursor_to_front(&mut self) {
        self.cursor.move_to(0);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the end of the input text.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor.move_to(usize::MAX);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to an arbitrary location.
    pub fn move_cursor_to(&mut self, position: usize) {
        self.cursor.move_to(position);
    }

    /// Selects all the content of the [`TextInput`].
    pub fn select_all(&mut self) {
        self.cursor.select_range(0, usize::MAX);
    }
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        State::is_focused(self)
    }

    fn focus(&mut self) {
        State::focus(self)
    }

    fn unfocus(&mut self) {
        State::unfocus(self)
    }
}

impl operation::TextInput for State {
    fn move_cursor_to_front(&mut self) {
        State::move_cursor_to_front(self)
    }

    fn move_cursor_to_end(&mut self) {
        State::move_cursor_to_end(self)
    }

    fn move_cursor_to(&mut self, position: usize) {
        State::move_cursor_to(self, position)
    }

    fn select_all(&mut self) {
        State::select_all(self)
    }
}

mod platform {
    use crate::core::keyboard;

    pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt()
        } else {
            modifiers.control()
        }
    }
}

fn offset<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    font: Renderer::Font,
    size: f32,
    value: &Value,
    state: &State,
) -> f32
where
    Renderer: text::Renderer,
{
    if state.is_focused() {
        let cursor = state.cursor();

        let focus_position = match cursor.state(value) {
            cursor::State::Index(i) => i,
            cursor::State::Selection { end, .. } => end,
        };

        let (_, offset) = measure_cursor_and_scroll_offset(
            renderer,
            text_bounds,
            value,
            size,
            focus_position,
            font,
        );

        offset
    } else {
        0.0
    }
}

fn measure_cursor_and_scroll_offset<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    value: &Value,
    size: f32,
    cursor_index: usize,
    font: Renderer::Font,
) -> (f32, f32)
where
    Renderer: text::Renderer,
{
    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width =
        renderer.measure_width(&text_before_cursor, size, font);

    let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

    (text_value_width, offset)
}

/// Computes the position of the text cursor at the given X coordinate of
/// a [`TextInput`].
fn find_cursor_position<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    font: Renderer::Font,
    size: Option<f32>,
    value: &Value,
    state: &State,
    x: f32,
) -> Option<usize>
where
    Renderer: text::Renderer,
{
    let size = size.unwrap_or_else(|| renderer.default_size());

    let offset = offset(renderer, text_bounds, font, size, value, state);
    let value = value.to_string();

    let char_offset = renderer
        .hit_test(
            &value,
            size,
            font,
            Size::INFINITY,
            Point::new(x + offset, text_bounds.height / 2.0),
            true,
        )
        .map(text::Hit::cursor)?;

    Some(
        unicode_segmentation::UnicodeSegmentation::graphemes(
            &value[..char_offset],
            true,
        )
        .count(),
    )
}

const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;
