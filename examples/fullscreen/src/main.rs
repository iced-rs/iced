struct App {}

impl iced::Application for App {
    type Executor = iced::executor::Default;
    type Message = ();
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (Self {}, iced::window::set_mode(iced::window::Mode::Fullscreen))
    }

    fn title(&self) -> String {
        "Iced Full-Screen Example".into()
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        iced::widget::text("HELLO").size(50).into()
    }
}

fn main() -> iced::Result {
    use iced::Application;
    App::run(iced::Settings::default())
}
