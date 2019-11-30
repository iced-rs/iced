use iced::{
    button, image, Align, Application, Background, Button, Color, Column,
    Command, Container, Element, HorizontalAlignment, Image, Length, Row,
    Settings, Text,
};
use serde::Deserialize;

pub fn main() {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    cats_button: button::State,
    dogs_button: button::State,
    image: Option<image::Handle>,
    state: State,
}

enum State {
    Idle,
    Loading(Pet),
    Error(LoadError),
}

impl Default for State {
    fn default() -> State {
        State::Idle
    }
}

#[derive(Debug, Clone)]
enum Message {
    PetChosen(Pet),
    ImageLoaded(Result<image::Handle, LoadError>),
}

#[derive(Debug, Clone, Copy)]
enum Pet {
    Cat,
    Dog,
}

impl Application for Example {
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Image viewer - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PetChosen(pet) => match self.state {
                State::Loading(_) => Command::none(),
                _ => {
                    self.state = State::Loading(pet);

                    Command::perform(get_pet_image(pet), Message::ImageLoaded)
                }
            },
            Message::ImageLoaded(Ok(image)) => {
                self.image = Some(image);
                self.state = State::Idle;

                Command::none()
            }
            Message::ImageLoaded(Err(error)) => {
                self.state = State::Error(error);

                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let Example {
            cats_button,
            dogs_button,
            state,
            image,
        } = self;

        let choose: Element<_> = match state {
            State::Loading(pet) => Text::new(format!(
                "Getting your {} ready...",
                match pet {
                    Pet::Cat => "cat",
                    Pet::Dog => "dog",
                }
            ))
            .width(Length::Shrink)
            .color([0.4, 0.4, 0.4])
            .into(),
            _ => Row::new()
                .width(Length::Shrink)
                .spacing(20)
                .push(
                    button(
                        cats_button,
                        "Cats",
                        Color::from_rgb8(0x89, 0x80, 0xF5),
                    )
                    .on_press(Message::PetChosen(Pet::Cat)),
                )
                .push(
                    button(
                        dogs_button,
                        "Dogs",
                        Color::from_rgb8(0x21, 0xD1, 0x9F),
                    )
                    .on_press(Message::PetChosen(Pet::Dog)),
                )
                .into(),
        };

        let content = Column::new()
            .width(Length::Shrink)
            .padding(20)
            .spacing(20)
            .align_items(Align::Center)
            .push(
                Text::new("What do you want to see?")
                    .width(Length::Shrink)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .size(40),
            )
            .push(choose);

        let content = if let Some(image) = image {
            content.push(Image::new(image.clone()).height(Length::Fill))
        } else {
            content
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn button<'a, Message>(
    state: &'a mut button::State,
    label: &str,
    color: Color,
) -> Button<'a, Message> {
    Button::new(
        state,
        Text::new(label)
            .horizontal_alignment(HorizontalAlignment::Center)
            .color(Color::WHITE)
            .size(30),
    )
    .padding(10)
    .min_width(100)
    .border_radius(10)
    .background(Background::Color(color))
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    url: String,
}

#[derive(Debug, Clone)]
enum LoadError {
    RequestError,
}

async fn get_pet_image(pet: Pet) -> Result<image::Handle, LoadError> {
    use std::io::Read;

    let search = match pet {
        Pet::Cat => "https://api.thecatapi.com/v1/images/search?limit=1&mime_types=jpg,png",
        Pet::Dog => "https://api.thedogapi.com/v1/images/search?limit=1&mime_types=jpg,png",
    };

    let results: Vec<SearchResult> = reqwest::get(search)?.json()?;
    let url = &results.first().unwrap().url;

    let mut image = reqwest::get(url)?;
    let mut bytes = Vec::new();

    image
        .read_to_end(&mut bytes)
        .map_err(|_| LoadError::RequestError)?;

    Ok(image::Handle::from_bytes(bytes))
}

impl From<reqwest::Error> for LoadError {
    fn from(error: reqwest::Error) -> LoadError {
        dbg!(&error);
        LoadError::RequestError
    }
}
