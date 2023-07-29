use std::path::PathBuf;

use explorer::Explorer;
use iced::{Application, executor, Command, widget::{Text, Column, Button}, Settings, Length};

mod explorer;


fn main() {
    App::run(Settings::default()).unwrap()
}
struct App {
    explorer: Explorer
}

#[derive(Debug, Clone)]
enum AppMsg {
    Notify(explorer::Message)
}

impl Application for App {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let path = PathBuf::from("/home/lenaic/Documents/iced/examples/proxy/test");
        let app =  App {
            explorer: Explorer::new(path)
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("App")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        
        match message {
            AppMsg::Notify(msg) => self.explorer.update(msg),
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let path2 = PathBuf::from("/home/lenaic/Documents/iced/examples/proxy/test/test2");

        Column::new()
            .push(Text::new(self.explorer.nb_deleted.to_string()))
            .push(
                Button::new("watch")
                    .on_press(AppMsg::Notify(explorer::Message::Watch(path2)))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(iced::Alignment::Center)
        
        .into()
    }
}


