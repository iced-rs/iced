use iced::{
    button, Button, Column, Container, Element, Length, Sandbox, Settings, Text,
};

pub fn main() -> iced::Result {
    Styling::run(Settings::default())
}

#[derive(Default)]
struct Styling {
    button: button::State,
}

impl Sandbox for Styling {
    type Message = ();

    fn new() -> Self {
        Styling::default()
    }

    fn title(&self) -> String {
        String::from("Basic Styling - Iced")
    }

    fn update(&mut self, message: Self::Message) {
    }

    fn view(&mut self) -> Element<Self::Message> {
        let button = Button::new(&mut self.button, Text::new("Button Text"))
            .padding(10)
            .on_press(())
            .style(Styles);

        let content = Column::new()
            .spacing(20)
            .padding(20)
            .max_width(600)
            .push(button);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

struct Styles;

impl button::StyleSheet for Styles {
    fn active(&self) -> button::Style {
        button::Style {
            border_radius: 0.0,
            ..button::DefaultStyle.active()
        }
    }
}
