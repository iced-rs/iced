use iced::widget::{container, text_editor};
use iced::{Element, Font, Sandbox, Settings};

pub fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Edit(text_editor::Action),
}

impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self {
            content: text_editor::Content::with(include_str!(
                "../../../README.md"
            )),
        }
    }

    fn title(&self) -> String {
        String::from("Editor - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        container(
            text_editor(&self.content)
                .on_edit(Message::Edit)
                .font(Font::with_name("Hasklug Nerd Font Mono")),
        )
        .padding(20)
        .into()
    }
}
