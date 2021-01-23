use iced::{
    button, executor, window, Align, Application, Button, Column, Command,
    Container, Element, Length, Settings, Text,
};

pub fn main() -> iced::Result {
    WinCtl::run(Settings::default())
}

#[derive(Debug, Default)]
struct WinCtl {
    mode: Option<window::Mode>,
    windowd_button: button::State,
    fullscreen_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    WindowedPress,
    FullscreenPress,
}

impl Application for WinCtl {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (WinCtl, Command<Message>) {
        (WinCtl::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Window controller - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let mode = match message {
            Message::WindowedPress => window::Mode::Windowed,
            Message::FullscreenPress => window::Mode::Fullscreen,
        };

        self.mode = Some(mode);

        Command::none()
    }

    fn mode(&mut self) -> Option<window::Mode> {
        self.mode.take()
    }

    fn view(&mut self) -> Element<Message> {
        Container::new(
            Column::new()
                .align_items(Align::Center)
                .spacing(20)
                .push(
                    Button::new(
                        &mut self.windowd_button,
                        Text::new("Windowed"),
                    )
                    .on_press(Message::WindowedPress),
                )
                .push(
                    Button::new(
                        &mut self.fullscreen_button,
                        Text::new("Fullscreen"),
                    )
                    .on_press(Message::FullscreenPress),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
