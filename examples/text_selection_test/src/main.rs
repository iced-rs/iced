//! Manual test harness for the text-selection PR. Exercises the three
//! pieces of reviewer feedback that aren't covered by `cargo test`:
//!
//! 1. Up/Down keyboard nav crossing into siblings (the `Line` /
//!    `LineEdge` filter fix).
//! 2. `Ctrl+A` not leaking across separate `selectable_group`s nor
//!    into a focused `text_editor`.
//! 3. Single / Double / Triple click dispatch (drag / word / line).
//!
//! The example mixes markdown views, a custom `selectable_group` of
//! plain `text` + `rich_text`, standalone selectable widgets, and a
//! `text_editor`, so the keyboard / click flows can be checked across
//! every selection-aware widget at once.
//!
//! Run with `cargo run -p text_selection_test`.

use iced::widget::{
    SelectableGroup, column, container, markdown, rich_text, row,
    scrollable, selectable_group, span, text, text_editor,
};
use iced::{Element, Fill, Never, Theme};

fn never<T>(never: Never) -> T {
    match never {}
}

pub fn main() -> iced::Result {
    iced::application(Test::new, Test::update, Test::view)
        .theme(Test::theme)
        .run()
}

struct Test {
    md_a: markdown::Content,
    md_b: markdown::Content,
    editor: text_editor::Content,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    EditorAction(text_editor::Action),
    #[allow(dead_code)]
    LinkClicked(markdown::Uri),
}

impl Test {
    fn new() -> Self {
        Self {
            md_a: markdown::Content::parse(MD_A),
            md_b: markdown::Content::parse(MD_B),
            editor: text_editor::Content::with_text(EDITOR_TEXT),
            theme: Theme::TokyoNight,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::EditorAction(action) => self.editor.perform(action),
            Message::LinkClicked(_) => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let settings = markdown::Settings {
            selectable: true,
            group_selection: true,
            ..markdown::Settings::with_style(&self.theme)
        };

        let md_a: Element<'_, _> = markdown::view(self.md_a.items(), settings)
            .map(Message::LinkClicked);
        let md_b: Element<'_, _> = markdown::view(self.md_b.items(), settings)
            .map(Message::LinkClicked);

        // Custom selectable_group mixing plain text and rich_text in a
        // column, so coordination across heterogeneous children gets
        // exercised outside the markdown pipeline too.
        let mixed_rich = rich_text![
            span("Then "),
            span("a rich_text![...] "),
            span("with multiple spans on the same line."),
        ]
        .on_link_click(never)
        .selectable(true);

        let mixed_group: SelectableGroup<'_, Never, _> = selectable_group(
            column![
                text("Mixed group · this paragraph is a plain text(...) widget.")
                    .selectable(true),
                mixed_rich,
                text(
                    "And one more text(...) line — try ArrowDown from the \
                     last visual line of the rich_text above and the caret \
                     should land here, not stick.",
                )
                .selectable(true),
            ]
            .spacing(8),
        );

        let standalone_text = text(
            "Standalone plain text(...). Single-click places caret. \
             Double-click selects a word. Triple-click selects the line. \
             Ctrl+A selects all of this once focused.",
        )
        .selectable(true);

        let standalone_rich = rich_text![
            span("Standalone rich_text![...] — "),
            span("triple-click here selects this whole line; "),
            span("double-click selects a word."),
        ]
        .on_link_click(never)
        .selectable(true);

        let editor = text_editor(&self.editor)
            .placeholder("text_editor — Ctrl+A here must NOT spill into the views above")
            .on_action(Message::EditorAction)
            .height(120)
            .padding(8);

        let header = text(
            "Try: drag-select inside one view, then click the other — \
             the previous selection should clear. Ctrl+A only selects \
             the most-recently-clicked widget. Double / triple click \
             a word / line in any selectable widget.",
        )
        .size(13);

        let panel = |title, inner| {
            container(
                column![
                    text(title).size(13),
                    container(scrollable(inner).height(Fill))
                        .padding(8)
                        .style(container::bordered_box),
                ]
                .spacing(6),
            )
            .width(Fill)
            .height(Fill)
        };

        column![
            header,
            row![
                panel("markdown view A (selectable_group via markdown)", md_a),
                panel("markdown view B (selectable_group via markdown)", md_b),
            ]
            .spacing(12)
            .height(Fill),
            panel(
                "custom selectable_group(column![text + rich_text + text])",
                mixed_group.into(),
            ),
            row![
                panel("standalone text(...).selectable(true)", standalone_text.into()),
                panel("standalone rich_text![...].selectable(true)", standalone_rich.into()),
            ]
            .spacing(12)
            .height(140),
            text("text_editor (focus-stealer test):").size(13),
            editor,
        ]
        .spacing(12)
        .padding(16)
        .into()
    }
}

const MD_A: &str = "\
# Markdown A

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod
tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim
veniam, quis nostrud exercitation ullamco laboris.

## Second paragraph

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum
dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

- list item one with enough text to wrap onto a second visual line
- list item two
- list item three

```rust
fn drag_select() {
    println!(\"triple-click selects this whole line\");
}
```
";

const MD_B: &str = "\
# Markdown B

Sit on a separate paragraph chain so cross-group `Ctrl+A` does **not**
fire across both views — only the most-recently-clicked group should
react.

> Quote: pressing `ArrowDown` at the end of this paragraph should jump
> into the next sibling, not stick on the current line.

1. Numbered item one
2. Numbered item two
3. Numbered item three with a [link](https://iced.rs) inside

The end.
";

const EDITOR_TEXT: &str = "\
text_editor content. Click into me and press Ctrl+A — none of the \
markdown views, the mixed group, or the standalone widgets above \
should react.";
