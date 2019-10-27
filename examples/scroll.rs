use iced::{
    button, scrollable, Align, Application, Button, Column, Element, Image,
    Justify, Length, Scrollable, Text,
};

pub fn main() {
    Example::default().run()
}

#[derive(Default)]
struct Example {
    item_count: u16,

    scroll: scrollable::State,
    add_button: button::State,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    AddItem,
}

impl Application for Example {
    type Message = Message;

    fn update(&mut self, message: Message) {
        match message {
            Message::AddItem => {
                self.item_count += 1;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        //let content = (0..3).fold(
        //    Scrollable::new(&mut self.scroll).spacing(20).padding(20),
        //    |content, _| {
        //        content.push(
        //        )
        //    },
        //);

        let content = (0..self.item_count)
            .fold(
                Scrollable::new(&mut self.scroll)
                    .spacing(20)
                    .padding(20)
                    .align_items(Align::Center),
                |column, i| {
                    if i % 2 == 0 {
                        column.push(lorem_ipsum().width(Length::Units(600)))
                    } else {
                        column.push(
                            Image::new(format!(
                                "{}/examples/resources/ferris.png",
                                env!("CARGO_MANIFEST_DIR")
                            ))
                            .width(Length::Units(400)),
                        )
                    }
                },
            )
            .push(
                Button::new(&mut self.add_button, Text::new("Add item"))
                    .on_press(Message::AddItem)
                    .padding(20)
                    .border_radius(5),
            );

        Column::new()
            .height(Length::Fill)
            .justify_content(Justify::Center)
            .padding(20)
            .push(content)
            .into()
    }
}

fn lorem_ipsum() -> Text {
    Text::new("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi in dui vel massa blandit interdum. Quisque placerat, odio ut vulputate sagittis, augue est facilisis ex, eget euismod felis magna in sapien. Nullam luctus consequat massa, ac interdum mauris blandit pellentesque. Nullam in est urna. Aliquam tristique lectus ac luctus feugiat. Aenean libero diam, euismod facilisis consequat quis, pellentesque luctus erat. Praesent vel tincidunt elit.")
}
