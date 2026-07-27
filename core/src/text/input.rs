#![allow(missing_docs)] // TODO
use crate::alignment;
use crate::clipboard;
use crate::layout;
use crate::mouse;
use crate::text::editor;
use crate::text::paragraph;
use crate::text::{self, Alignment, Editor, LineHeight, Text, Wrapping};
use crate::widget::operation::Focusable;
use crate::{Color, Event, InputMethod, Length, Padding, Pixels, Point, Rectangle, Shell};

pub struct Input<R: text::Renderer> {
    editor: R::Editor,
    state: editor::State,
    placeholder: paragraph::Plain<R::Paragraph>,
    padding: Padding,
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
    pub wrapping: Wrapping,
}

impl<R: text::Renderer> Input<R> {
    pub fn new() -> Self {
        Self {
            editor: R::Editor::with_text(""),
            state: editor::State::new(),
            placeholder: paragraph::Plain::default(),
            padding: Padding::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.editor.is_empty()
    }

    pub fn value(&self) -> String {
        self.editor.text()
    }

    pub fn placeholder(&self) -> &str {
        self.placeholder.content()
    }

    pub fn is_focused(&self) -> bool {
        self.state.is_focused()
    }

    pub fn focus(&mut self) {
        self.state.focus();
    }

    pub fn unfocus(&mut self) {
        self.state.unfocus();
    }

    pub fn select_all(&mut self) {
        self.editor.perform(editor::Action::SelectAll);
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
            layout.wrapping,
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
    ) -> bool {
        fn apply<Message>(
            editor: &mut impl Editor,
            shell: &mut Shell<'_, Message>,
            update: editor::Update<Message>,
        ) -> bool {
            match update {
                editor::Update::Action(action) => {
                    let is_edit = action.is_edit();

                    editor.perform(action);

                    shell.request_redraw();
                    shell.capture_event();

                    return is_edit;
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
                    let mut is_edit = false;

                    for update in updates {
                        is_edit |= apply(editor, shell, update);
                    }

                    return is_edit;
                }
            }

            false
        }

        let Some(update) = self.state.update(
            &self.editor,
            event,
            bounds,
            self.padding,
            cursor,
            key_binding,
        ) else {
            return false;
        };

        apply(&mut self.editor, shell, update)
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
