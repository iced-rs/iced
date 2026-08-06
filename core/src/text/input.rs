#![allow(missing_docs)] // TODO
use crate::alignment;
use crate::clipboard;
use crate::layout;
use crate::mouse;
use crate::text::editor;
use crate::text::paragraph;
use crate::text::{self, Alignment, Editor, LineHeight, Position, Text, Wrapping};
use crate::widget::operation::{Focusable, TextInput};
use crate::{Color, Event, InputMethod, Length, Padding, Pixels, Point, Rectangle, Shell};

use std::sync::Arc;

pub struct Input<R: text::Renderer> {
    editor: R::Editor,
    state: editor::State,
    placeholder: paragraph::Plain<R::Paragraph>,
    padding: Padding,
    multiline: Option<Wrapping>,
}

pub struct Layout<'a, Font> {
    pub width: Length,
    pub height: Length,
    pub padding: Padding,
    pub placeholder: &'a str,
    pub font: Option<Font>,
    pub size: Option<Pixels>,
    pub line_height: LineHeight,
    pub alignment: Alignment,
    pub multiline: Option<Wrapping>,
}

impl<R: text::Renderer> Input<R> {
    pub fn new() -> Self {
        Self {
            editor: R::Editor::with_text(""),
            state: editor::State::new(),
            placeholder: paragraph::Plain::default(),
            padding: Padding::default(),
            multiline: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.editor.is_empty()
    }

    pub fn value(&self) -> String {
        Editor::text(&self.editor)
    }

    pub fn placeholder(&self) -> &str {
        self.placeholder.content()
    }

    pub fn overwrite(&mut self, value: &str) {
        self.editor.overwrite(value);
    }

    pub fn layout(
        &mut self,
        renderer: &R,
        limits: &layout::Limits,
        layout: Layout<'_, R::Font>,
    ) -> layout::Node {
        self.padding = layout.padding;
        self.multiline = layout.multiline;

        let limits = limits
            .width(layout.width)
            .height(layout.height)
            .shrink(layout.padding);

        let font = layout.font.unwrap_or_else(|| renderer.default_font());
        let size = layout.size.unwrap_or_else(|| renderer.default_size());
        let hint_factor = renderer.hint_factor();

        self.editor.update(
            limits.max(),
            font,
            size,
            layout.line_height,
            layout.multiline.unwrap_or(text::Wrapping::None),
            layout.alignment,
            hint_factor,
            &mut text::highlighter::PlainText,
        );

        let bounds = match layout.height {
            Length::Fill
            | Length::FillPortion(_)
            | Length::Fixed(_)
            | Length::Bounded { .. }
            | Length::Fluid(_) => limits.max(),
            Length::Shrink | Length::Fit => {
                limits.resolve(layout.width, layout.height, self.editor.min_bounds())
            }
        };

        let _ = self.placeholder.update(Text {
            content: layout.placeholder,
            font,
            line_height: layout.line_height,
            bounds,
            size,
            align_x: layout.alignment,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
            ellipsis: text::Ellipsis::None,
            hint_factor,
        });

        layout::Node::new(bounds.expand(layout.padding))
    }

    pub fn update<Message>(
        &mut self,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
        key_binding: impl Fn(editor::KeyPress) -> Option<editor::Binding<Message>>,
    ) -> Option<Edit> {
        fn apply<Message>(
            editor: &mut impl Editor,
            shell: &mut Shell<'_, Message>,
            update: editor::Update<Message>,
            is_multiline: bool,
        ) -> Option<Edit> {
            match update {
                editor::Update::Action(action) => {
                    let (action, is_paste) = match action {
                        editor::Action::Edit(editor::Edit::Enter)
                        | editor::Action::Move(editor::Motion::Up)
                        | editor::Action::Move(editor::Motion::Down)
                            if !is_multiline =>
                        {
                            return None;
                        }
                        editor::Action::Edit(editor::Edit::Paste(text)) => (
                            editor::Action::Edit(editor::Edit::Paste(if !is_multiline {
                                Arc::new(text.lines().collect())
                            } else {
                                text
                            })),
                            true,
                        ),
                        _ => (action, false),
                    };

                    let is_edit = action.is_edit();

                    editor.perform(action);
                    shell.capture_event();

                    if is_edit && is_multiline {
                        shell.invalidate_layout();
                    } else {
                        shell.request_redraw();
                    }

                    return is_edit.then_some(Edit { is_paste });
                }
                editor::Update::Focus | editor::Update::InputMethod => {
                    shell.request_redraw();
                    shell.capture_event();
                }
                editor::Update::Unfocus => {
                    shell.request_redraw();
                }
                editor::Update::Release => {}
                editor::Update::Copy(text) => {
                    shell.write_clipboard(text);
                    shell.capture_event();
                }
                editor::Update::Paste => {
                    shell.read_clipboard(clipboard::Kind::Text);
                    shell.capture_event();
                }
                editor::Update::RedrawAt(at) => {
                    shell.request_redraw_at(at);
                }
                editor::Update::Custom(message) => {
                    shell.publish(message);
                    shell.capture_event();
                }
                editor::Update::Sequence(updates) => {
                    let mut edit: Option<Edit> = None;

                    for update in updates {
                        if let Some(new_edit) = apply(editor, shell, update, is_multiline) {
                            edit = Some(Edit {
                                is_paste: edit.unwrap_or_default().is_paste || new_edit.is_paste,
                            });
                        }
                    }

                    return edit;
                }
            }

            None
        }

        let update = self.state.update(
            &self.editor,
            event,
            bounds,
            self.padding,
            cursor,
            key_binding,
        )?;

        apply(&mut self.editor, shell, update, self.multiline.is_some())
    }

    pub fn draw(&self, renderer: &mut R, bounds: Rectangle, viewport: Rectangle, style: Style) {
        let text_bounds = bounds.shrink(self.padding);

        let Some(clip_bounds) = text_bounds.intersection(&viewport) else {
            return;
        };

        if self.editor.is_empty() {
            let anchor = text_bounds.anchor(
                self.placeholder.min_bounds(),
                self.placeholder.align_x(),
                self.placeholder.align_y(),
            );

            renderer.fill_paragraph(
                self.placeholder.raw(),
                anchor,
                style.placeholder,
                clip_bounds,
            );
        }

        self.state.draw(
            &self.editor,
            renderer,
            text_bounds.position(),
            clip_bounds,
            editor::Style {
                value: style.value,
                selection: style.selection,
            },
        );
    }

    pub fn input_method(&self, position: Point) -> InputMethod<&str> {
        self.state.input_method(&self.editor, position)
    }
}

impl<R: text::Renderer> Default for Input<R> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Style {
    pub value: Color,
    pub selection: Color,
    pub placeholder: Color,
}

impl<R: text::Renderer> Focusable for Input<R> {
    fn is_focused(&self) -> bool {
        self.state.is_focused()
    }

    fn focus(&mut self) {
        self.state.focus();
    }

    fn unfocus(&mut self) {
        self.state.unfocus();
    }
}

impl<R: text::Renderer> TextInput for Input<R> {
    fn text(&self) -> text::Fragment<'_> {
        if self.editor.is_empty() {
            text::Fragment::Borrowed(self.placeholder.content())
        } else {
            TextInput::text(&self.editor)
        }
    }

    fn move_cursor_to(&mut self, position: Position) {
        self.editor.move_cursor_to(position);
    }

    fn move_cursor_to_front(&mut self) {
        self.editor.move_cursor_to_front();
    }

    fn move_cursor_to_end(&mut self) {
        self.editor.move_cursor_to_end();
    }

    fn select_all(&mut self) {
        self.editor.select_all();
    }

    fn select_range(&mut self, start: Position, end: Position) {
        self.editor.select_range(start, end);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Edit {
    pub is_paste: bool,
}
