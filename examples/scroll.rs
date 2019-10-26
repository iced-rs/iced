use iced::{
    button, scrollable, Align, Application, Button, Color, Column, Element,
    Image, Justify, Length, Scrollable, Text,
};

pub fn main() {
    Example::default().run()
}

#[derive(Default)]
struct Example {
    paragraph_count: u16,

    scroll: scrollable::State,
    add_button: button::State,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    AddParagraph,
}

impl Application for Example {
    type Message = Message;

    fn update(&mut self, message: Message) {
        match message {
            Message::AddParagraph => {
                self.paragraph_count += 1;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content = Scrollable::new(&mut self.scroll).spacing(20).padding(20);

        //let content = (0..self.paragraph_count)
        //    .fold(content, |column, _| column.push(lorem_ipsum()))
        //    .push(
        //        Button::new(&mut self.add_button, Text::new("Add paragraph"))
        //            .on_press(Message::AddParagraph)
        //            .padding(20)
        //            .border_radius(5)
        //            .align_self(Align::Center),
        //    );

        Column::new()
            .height(Length::Fill)
            .max_width(Length::Units(600))
            .align_self(Align::Center)
            .justify_content(Justify::Center)
            .push((0..3).fold(content, |content, _| {
                content.push(
                    Image::new(format!(
                        "{}/examples/resources/ferris.png",
                        env!("CARGO_MANIFEST_DIR")
                    ))
                    .width(Length::Units(400))
                    .align_self(Align::Center),
                )
            }))
            .into()
    }
}

fn lorem_ipsum() -> Text {
    Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi in dui vel massa blandit interdum. Quisque placerat, odio ut vulputate sagittis, augue est facilisis ex, eget euismod felis magna in sapien. Nullam luctus consequat massa, ac interdum mauris blandit pellentesque. Nullam in est urna. Aliquam tristique lectus ac luctus feugiat. Aenean libero diam, euismod facilisis consequat quis, pellentesque luctus erat. Praesent vel tincidunt elit.")
}
