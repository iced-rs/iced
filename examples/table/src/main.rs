use iced::font;
use iced::time::{Duration, hours, minutes};
use iced::widget::{center, scrollable, table, text};
use iced::{Element, Fill, Font};

pub fn main() -> iced::Result {
    iced::application(Table::new, Table::update, Table::view).run()
}

struct Table {
    events: Vec<Event>,
}

#[derive(Debug, Clone)]
enum Message {}

impl Table {
    fn new() -> Self {
        Self {
            events: Event::list(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<'_, Message> {
        let table = {
            let bold = |header| {
                text(header).font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
            };

            let columns = table::definition()
                .column(bold("Name"), |event: &Event| {
                    text(&event.name).width(Fill)
                })
                .column(bold("Time"), |event| text!("{:?}", event.duration))
                .column(bold("Price"), |event| text!("{:.2}", event.price))
                .column(bold("Rating"), |event| text!("{:.2}", event.rating));

            table(columns, &self.events).width(640).spacing_y(5)
        };

        center(scrollable(table).spacing(10)).padding(10).into()
    }
}

struct Event {
    name: String,
    duration: Duration,
    price: f32,
    rating: f32,
}

impl Event {
    fn list() -> Vec<Self> {
        vec![
            Event {
                name: "Get lost in a hacker bookstore".to_owned(),
                duration: hours(2),
                price: 0.0,
                rating: 4.9,
            },
            Event {
                name: "Buy vintage synth at Noisebridge flea market".to_owned(),
                duration: hours(1),
                price: 150.0,
                rating: 4.8,
            },
            Event {
                name: "Eat a questionable hot dog at 2AM".to_owned(),
                duration: minutes(20),
                price: 5.0,
                rating: 1.7,
            },
            Event {
                name: "Ride the MUNI for the story".to_owned(),
                duration: minutes(60),
                price: 3.0,
                rating: 4.1,
            },
            Event {
                name: "Scream into the void from Twin Peaks".to_owned(),
                duration: minutes(40),
                price: 0.0,
                rating: 4.9,
            },
            Event {
                name: "Buy overpriced coffee and feel things".to_owned(),
                duration: minutes(25),
                price: 6.5,
                rating: 4.5,
            },
            Event {
                name: "Attend an underground robot poetry slam".to_owned(),
                duration: hours(1),
                price: 12.0,
                rating: 4.8,
            },
            Event {
                name: "Browse cursed tech at a retro computer fair".to_owned(),
                duration: hours(2),
                price: 10.0,
                rating: 4.7,
            },
            Event {
                name: "Try to order at a secret ramen place with no sign"
                    .to_owned(),
                duration: minutes(50),
                price: 14.0,
                rating: 4.6,
            },
            Event {
                name: "Join a spontaneous rooftop drone rave".to_owned(),
                duration: hours(3),
                price: 0.0,
                rating: 4.9,
            },
            Event {
                name: "Sketch a stranger at Dolores Park".to_owned(),
                duration: minutes(45),
                price: 0.0,
                rating: 4.4,
            },
            Event {
                name: "Visit the Museum of Obsolete APIs".to_owned(),
                duration: hours(1),
                price: 9.99,
                rating: 4.2,
            },
            Event {
                name: "Chase the last working payphone".to_owned(),
                duration: minutes(35),
                price: 0.25,
                rating: 4.0,
            },
            Event {
                name: "Trade zines with a punk on BART".to_owned(),
                duration: minutes(30),
                price: 3.5,
                rating: 4.7,
            },
            Event {
                name: "Get a tattoo of the Git logo".to_owned(),
                duration: hours(1),
                price: 200.0,
                rating: 4.6,
            },
        ]
    }
}
