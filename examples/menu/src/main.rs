use iced::menu::{self, Menu};
use iced::{
    executor, Application, Clipboard, Command, Container, Element, Length,
    Settings, Text,
};
use iced_native::keyboard::{Hotkey, KeyCode, Modifiers};

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Debug, Default)]
struct App {
    selected: Option<Entry>,
}

#[derive(Debug, Clone)]
enum Entry {
    One,
    Two,
    Three,
    A,
    B,
    C,
}

#[derive(Debug, Clone)]
enum Message {
    MenuActivated(Entry),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Menu - Iced")
    }

    fn menu(&self) -> Menu<Message> {
        let alt = Modifiers::ALT;
        let ctrl_shift = Modifiers::CTRL | Modifiers::SHIFT;

        Menu::with_entries(vec![
            menu::Entry::dropdown(
                "First",
                Menu::with_entries(vec![
                    menu::Entry::item(
                        "One",
                        Hotkey::new(alt, KeyCode::F1),
                        Message::MenuActivated(Entry::One),
                    ),
                    menu::Entry::item(
                        "Two",
                        Hotkey::new(alt, KeyCode::F2),
                        Message::MenuActivated(Entry::Two),
                    ),
                    menu::Entry::Separator,
                    menu::Entry::item(
                        "Three",
                        Hotkey::new(alt, KeyCode::F3),
                        Message::MenuActivated(Entry::Three),
                    ),
                ]),
            ),
            menu::Entry::dropdown(
                "Second",
                Menu::with_entries(vec![
                    menu::Entry::item(
                        "A",
                        Hotkey::new(ctrl_shift, KeyCode::A),
                        Message::MenuActivated(Entry::A),
                    ),
                    menu::Entry::item(
                        "B",
                        Hotkey::new(ctrl_shift, KeyCode::B),
                        Message::MenuActivated(Entry::B),
                    ),
                    menu::Entry::Separator,
                    menu::Entry::item(
                        "C",
                        Hotkey::new(ctrl_shift, KeyCode::C),
                        Message::MenuActivated(Entry::C),
                    ),
                ]),
            ),
        ])
    }

    fn update(
        &mut self,
        message: Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Message> {
        match message {
            Message::MenuActivated(entry) => self.selected = Some(entry),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Container::new(
            Text::new(format!("Selected {:?}", self.selected)).size(48),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
