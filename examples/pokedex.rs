use iced::{
    button, image, Align, Application, Button, Column, Command, Container,
    Element, Image, Length, Row, Settings, Text,
};

pub fn main() {
    Pokedex::run(Settings::default())
}

#[derive(Debug)]
enum Pokedex {
    Loading,
    Loaded {
        pokemon: Pokemon,
        search: button::State,
    },
    Errored {
        error: Error,
        try_again: button::State,
    },
}

#[derive(Debug, Clone)]
enum Message {
    PokemonFound(Result<Pokemon, Error>),
    Search,
}

impl Application for Pokedex {
    type Message = Message;

    fn new() -> (Pokedex, Command<Message>) {
        (
            Pokedex::Loading,
            Command::perform(Pokemon::search(), Message::PokemonFound),
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
            Pokedex::Loading => "Loading",
            Pokedex::Loaded { pokemon, .. } => &pokemon.name,
            Pokedex::Errored { .. } => "Whoops!",
        };

        format!("{} - Pokédex", subtitle)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PokemonFound(Ok(pokemon)) => {
                *self = Pokedex::Loaded {
                    pokemon,
                    search: button::State::new(),
                };

                Command::none()
            }
            Message::PokemonFound(Err(error)) => {
                *self = Pokedex::Errored {
                    error,
                    try_again: button::State::new(),
                };

                Command::none()
            }
            Message::Search => match self {
                Pokedex::Loading => Command::none(),
                _ => {
                    *self = Pokedex::Loading;

                    Command::perform(Pokemon::search(), Message::PokemonFound)
                }
            },
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content = match self {
            Pokedex::Loading => Column::new().width(Length::Shrink).push(
                Text::new("Searching for Pokémon...")
                    .width(Length::Shrink)
                    .size(40),
            ),
            Pokedex::Loaded { pokemon, search } => Column::new()
                .max_width(500)
                .spacing(20)
                .align_items(Align::End)
                .push(pokemon.view())
                .push(
                    button(search, "Keep searching!").on_press(Message::Search),
                ),
            Pokedex::Errored { try_again, .. } => Column::new()
                .width(Length::Shrink)
                .spacing(20)
                .align_items(Align::End)
                .push(
                    Text::new("Whoops! Something went wrong...")
                        .width(Length::Shrink)
                        .size(40),
                )
                .push(button(try_again, "Try again").on_press(Message::Search)),
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone)]
struct Pokemon {
    number: u16,
    name: String,
    description: String,
    image: image::Handle,
}

impl Pokemon {
    const TOTAL: u16 = 807;

    fn view(&self) -> Element<Message> {
        Row::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(Image::new(self.image.clone()))
            .push(
                Column::new()
                    .spacing(20)
                    .push(
                        Row::new()
                            .align_items(Align::Center)
                            .spacing(20)
                            .push(Text::new(&self.name).size(30))
                            .push(
                                Text::new(format!("#{}", self.number))
                                    .width(Length::Shrink)
                                    .size(20)
                                    .change_style(|s| {
                                        s.text_color = [0.5, 0.5, 0.5].into()
                                    }),
                            ),
                    )
                    .push(Text::new(&self.description)),
            )
            .into()
    }

    async fn search() -> Result<Pokemon, Error> {
        use rand::Rng;
        use serde::Deserialize;
        use std::io::Read;

        #[derive(Debug, Deserialize)]
        struct Entry {
            id: u32,
            name: String,
            flavor_text_entries: Vec<FlavorText>,
        }

        #[derive(Debug, Deserialize)]
        struct FlavorText {
            flavor_text: String,
            language: Language,
        }

        #[derive(Debug, Deserialize)]
        struct Language {
            name: String,
        }

        let id = {
            let mut rng = rand::thread_rng();

            rng.gen_range(0, Pokemon::TOTAL)
        };

        let url = format!("https://pokeapi.co/api/v2/pokemon-species/{}", id);
        let sprite = format!("https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/{}.png", id);

        let entry: Entry = reqwest::get(&url)?.json()?;

        let description = entry
            .flavor_text_entries
            .iter()
            .filter(|text| text.language.name == "en")
            .next()
            .ok_or(Error::LanguageError)?;

        let mut sprite = reqwest::get(&sprite)?;
        let mut bytes = Vec::new();

        sprite
            .read_to_end(&mut bytes)
            .map_err(|_| Error::ImageError)?;

        Ok(Pokemon {
            number: id,
            name: entry.name.to_uppercase(),
            description: description
                .flavor_text
                .chars()
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect(),
            image: image::Handle::from_memory(bytes),
        })
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    ImageError,
    LanguageError,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        dbg!(&error);

        Error::APIError
    }
}

fn button<'a>(state: &'a mut button::State, text: &str) -> Button<'a, Message> {
    Button::new(state, Text::new(text))
        .change_style(|s| s.border_radius = 10)
        .padding(10)
}
