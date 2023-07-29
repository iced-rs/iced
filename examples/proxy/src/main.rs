use std::path::PathBuf;

use explorer::Explorer;
use iced::{
    proxy,
    executor,
    widget::{Column, Text, TextInput},
    Application, Command, Length, Settings,
};
use notify::Event;

mod explorer;

fn main() {
    State::run(Settings::default()).unwrap()
}

enum State {
    Loading,
    Loaded(App),
}
struct App {
    path: String,
    explorer: Option<Explorer>,
    proxy: proxy::Proxy
}

impl App {
    fn new(proxy: proxy::Proxy) -> Self {
        App {
            path: String::new(),
            explorer: None,
            proxy
        }
    }
}

#[derive(Debug, Clone)]
enum AppMsg {
    OnInput(String),
    OnSubmit,
    ProxyReceived(proxy::Proxy),
    Event(Event),
}

impl Application for State {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            State::Loading,
            proxy::fetch_proxy(Message::ProxyReceived)
        )
    }

    fn title(&self) -> String {
        String::from("App")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match self {
            State::Loading => {}
            State::Loaded(app) => match message {
                AppMsg::OnInput(input) => {
                    app.path = input;
                }
                AppMsg::OnSubmit => {
                    app.explorer = Explorer::new(PathBuf::from(app.path.clone()));
                }
                AppMsg::ProxyReceived(proxy) => *self = State::Loaded(App::new(proxy)),
                AppMsg::Event(e) => {
                    if let Some(explorer) = &mut app.explorer {
                        explorer.process_event(e);
                    }
                }
            },
        }

        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        match self {
            State::Loading => Text::new("loading").into(),
            State::Loaded(app) => {
                let input = TextInput::new("path", &app.path)
                    .on_input(AppMsg::OnInput)
                    .on_submit(AppMsg::OnSubmit)
                    .into();

                let mut childs = Vec::new();

                childs.push(input);

                if let Some(explorer) = &app.explorer {
                    let text = Text::new(explorer.nb_deleted.to_string()).into();
                    childs.push(text);
                }

                Column::with_children(childs)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(iced::Alignment::Center)
                    .into()
            }
        }
    }
}
