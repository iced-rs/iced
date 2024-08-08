use iced::Element;

fn main() -> iced::Result {
    iced::program("Hello - Iced", HelloIced::update, HelloIced::view).run()
}

#[derive(Default)]
struct HelloIced {}

#[derive(Debug)]
enum Message {}

impl HelloIced {
    fn update(&mut self, _message: Message) {}
    
    fn view(&self) -> Element<Message> {
        "Hello, Iced".into()
    }
}