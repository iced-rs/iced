//!

use iced::widget::text;

///
fn main() {
    iced::application(App::new, App::update, App::view)
        .run()
        .unwrap();
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

struct App {}

impl App {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, _: Message) {}

    fn view(&self) -> iced::Element<'_, Message> {
        text("Tao is working").into()
    }
}
