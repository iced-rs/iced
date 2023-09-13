use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text::editor::{Cursor, Editor as _};
use crate::core::text::{self, LineHeight};
use crate::core::widget::{self, Widget};
use crate::core::{
    Clipboard, Color, Element, Length, Padding, Pixels, Point, Rectangle,
    Shell, Vector,
};

use std::cell::RefCell;

pub use crate::style::text_editor::{Appearance, StyleSheet};
pub use text::editor::{Action, Motion};

pub struct TextEditor<'a, Message, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: &'a Content<Renderer>,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    padding: Padding,
    style: <Renderer::Theme as StyleSheet>::Style,
    on_edit: Option<Box<dyn Fn(Action) -> Message + 'a>>,
}

impl<'a, Message, Renderer> TextEditor<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub fn new(content: &'a Content<Renderer>) -> Self {
        Self {
            content,
            font: None,
            text_size: None,
            line_height: LineHeight::default(),
            width: Length::Fill,
            height: Length::Fill,
            padding: Padding::new(5.0),
            style: Default::default(),
            on_edit: None,
        }
    }

    pub fn on_edit(mut self, on_edit: impl Fn(Action) -> Message + 'a) -> Self {
        self.on_edit = Some(Box::new(on_edit));
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
}

pub struct Content<R = crate::Renderer>(RefCell<Internal<R>>)
where
    R: text::Renderer;

struct Internal<R>
where
    R: text::Renderer,
{
    editor: R::Editor,
    is_dirty: bool,
}

impl<R> Content<R>
where
    R: text::Renderer,
{
    pub fn new() -> Self {
        Self::with("")
    }

    pub fn with(text: &str) -> Self {
        Self(RefCell::new(Internal {
            editor: R::Editor::with_text(text),
            is_dirty: true,
        }))
    }

    pub fn edit(&mut self, action: Action) {
        let internal = self.0.get_mut();

        internal.editor.perform(action);
        internal.is_dirty = true;
    }
}

impl<Renderer> Default for Content<Renderer>
where
    Renderer: text::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

struct State {
    is_focused: bool,
    is_dragging: bool,
    last_click: Option<mouse::Click>,
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for TextEditor<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            is_focused: false,
            is_dragging: false,
            last_click: None,
        })
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> iced_renderer::core::layout::Node {
        let mut internal = self.content.0.borrow_mut();

        internal.editor.update(
            limits.pad(self.padding).max(),
            self.font.unwrap_or_else(|| renderer.default_font()),
            self.text_size.unwrap_or_else(|| renderer.default_size()),
            self.line_height,
        );

        layout::Node::new(limits.max())
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let Some(on_edit) = self.on_edit.as_ref() else {
            return event::Status::Ignored;
        };

        let state = tree.state.downcast_mut::<State>();

        let Some(update) = Update::from_event(
            event,
            state,
            layout.bounds(),
            self.padding,
            cursor,
        ) else {
            return event::Status::Ignored;
        };

        match update {
            Update::Click { click, action } => {
                state.is_focused = true;
                state.is_dragging = true;
                state.last_click = Some(click);
                shell.publish(on_edit(action));
            }
            Update::Unfocus => {
                state.is_focused = false;
                state.is_dragging = false;
            }
            Update::StopDragging => {
                state.is_dragging = false;
            }
            Update::Edit(action) => {
                shell.publish(on_edit(action));
            }
            Update::Copy => {}
            Update::Paste => if let Some(_contents) = clipboard.read() {},
        }

        event::Status::Captured
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as renderer::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let internal = self.content.0.borrow();
        let state = tree.state.downcast_ref::<State>();

        let is_disabled = self.on_edit.is_none();
        let is_mouse_over = cursor.is_over(bounds);

        let appearance = if is_disabled {
            theme.disabled(&self.style)
        } else if state.is_focused {
            theme.focused(&self.style)
        } else if is_mouse_over {
            theme.hovered(&self.style)
        } else {
            theme.active(&self.style)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: appearance.border_radius,
                border_width: appearance.border_width,
                border_color: appearance.border_color,
            },
            appearance.background,
        );

        renderer.fill_editor(
            &internal.editor,
            bounds.position()
                + Vector::new(self.padding.left, self.padding.top),
            style.text_color,
        );

        if state.is_focused {
            match internal.editor.cursor() {
                Cursor::Caret(position) => {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: position.x + bounds.x + self.padding.left,
                                y: position.y + bounds.y + self.padding.top,
                                width: 1.0,
                                height: self
                                    .line_height
                                    .to_absolute(self.text_size.unwrap_or_else(
                                        || renderer.default_size(),
                                    ))
                                    .into(),
                            },
                            border_radius: 0.0.into(),
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        },
                        theme.value_color(&self.style),
                    );
                }
                Cursor::Selection(ranges) => {
                    for range in ranges {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: range
                                    + Vector::new(
                                        bounds.x + self.padding.left,
                                        bounds.y + self.padding.top,
                                    ),
                                border_radius: 0.0.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            theme.selection_color(&self.style),
                        );
                    }
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        _state: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_disabled = self.on_edit.is_none();

        if cursor.is_over(layout.bounds()) {
            if is_disabled {
                mouse::Interaction::NotAllowed
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Renderer> From<TextEditor<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(text_editor: TextEditor<'a, Message, Renderer>) -> Self {
        Self::new(text_editor)
    }
}

enum Update {
    Click { click: mouse::Click, action: Action },
    Unfocus,
    StopDragging,
    Edit(Action),
    Copy,
    Paste,
}

impl Update {
    fn from_event(
        event: Event,
        state: &State,
        bounds: Rectangle,
        padding: Padding,
        cursor: mouse::Cursor,
    ) -> Option<Self> {
        let edit = |action| Some(Update::Edit(action));
        let move_ = |motion| Some(Update::Edit(Action::Move(motion)));

        match event {
            Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let cursor_position = cursor_position
                            - Vector::new(padding.top, padding.left);

                        let click = mouse::Click::new(
                            cursor_position,
                            state.last_click,
                        );

                        let action = match click.kind() {
                            mouse::click::Kind::Single => {
                                Action::Click(cursor_position)
                            }
                            mouse::click::Kind::Double => Action::SelectWord,
                            mouse::click::Kind::Triple => Action::SelectLine,
                        };

                        Some(Update::Click { click, action })
                    } else if state.is_focused {
                        Some(Update::Unfocus)
                    } else {
                        None
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    Some(Update::StopDragging)
                }
                mouse::Event::CursorMoved { .. } if state.is_dragging => {
                    let cursor_position = cursor.position_in(bounds)?
                        - Vector::new(padding.top, padding.left);

                    edit(Action::Drag(cursor_position))
                }
                _ => None,
            },
            Event::Keyboard(event) => match event {
                keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                } if state.is_focused => {
                    if let Some(motion) = motion(key_code) {
                        let motion = if modifiers.control() {
                            motion.widen()
                        } else {
                            motion
                        };

                        return edit(if modifiers.shift() {
                            Action::Select(motion)
                        } else {
                            Action::Move(motion)
                        });
                    }

                    match key_code {
                        keyboard::KeyCode::Enter => edit(Action::Enter),
                        keyboard::KeyCode::Backspace => edit(Action::Backspace),
                        keyboard::KeyCode::Delete => edit(Action::Delete),
                        keyboard::KeyCode::Escape => Some(Self::Unfocus),
                        _ => None,
                    }
                }
                keyboard::Event::CharacterReceived(c) if state.is_focused => {
                    edit(Action::Insert(c))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn motion(key_code: keyboard::KeyCode) -> Option<Motion> {
    match key_code {
        keyboard::KeyCode::Left => Some(Motion::Left),
        keyboard::KeyCode::Right => Some(Motion::Right),
        keyboard::KeyCode::Up => Some(Motion::Up),
        keyboard::KeyCode::Down => Some(Motion::Down),
        keyboard::KeyCode::Home => Some(Motion::Home),
        keyboard::KeyCode::End => Some(Motion::End),
        keyboard::KeyCode::PageUp => Some(Motion::PageUp),
        keyboard::KeyCode::PageDown => Some(Motion::PageDown),
        _ => None,
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
