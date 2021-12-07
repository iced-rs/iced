use iced::{
    button, futures, image, Alignment, Application, Button, Column, Command,
    Container, Element, Length, Row, Settings, Text,
};

pub fn main() -> iced::Result {
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
        try_again: button::State,
    },
}

#[derive(Debug, Clone)]
enum Message {
    PokemonFound(Result<Pokemon, Error>),
    Search,
}

impl Application for Pokedex {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Pokedex, Command<Message>) {
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
            Message::PokemonFound(Err(_error)) => {
                *self = Pokedex::Errored {
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
            Pokedex::Loading => Column::new()
                .width(Length::Shrink)
                .push(Text::new("Searching for Pokémon...").size(40)),
            Pokedex::Loaded { pokemon, search } => Column::new()
                .max_width(500)
                .spacing(20)
                .align_items(Alignment::End)
                .push(pokemon.view())
                .push(
                    button(search, "Keep searching!").on_press(Message::Search),
                ),
            Pokedex::Errored { try_again, .. } => Column::new()
                .spacing(20)
                .align_items(Alignment::End)
                .push(Text::new("Whoops! Something went wrong...").size(40))
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
    image_viewer: image::viewer::State,
}

impl Pokemon {
    const TOTAL: u16 = 807;

    fn view(&mut self) -> Element<Message> {
        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(image::Viewer::new(
                &mut self.image_viewer,
                self.image.clone(),
            ))
            .push(
                Column::new()
                    .spacing(20)
                    .push(
                        Row::new()
                            .align_items(Alignment::Center)
                            .spacing(20)
                            .push(
                                Text::new(&self.name)
                                    .size(30)
                                    .width(Length::Fill),
                            )
                            .push(
                                Text::new(format!("#{}", self.number))
                                    .size(20)
                                    .color([0.5, 0.5, 0.5]),
                            ),
                    )
                    .push(Text::new(&self.description)),
            )
            .into()
    }

    async fn search() -> Result<Pokemon, Error> {
        use rand::Rng;
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        struct Entry {
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
            let mut rng = rand::rngs::OsRng::default();

            rng.gen_range(0, Pokemon::TOTAL)
        };

        let fetch_entry = async {
            let url =
                format!("https://pokeapi.co/api/v2/pokemon-species/{}", id);

            reqwest::get(&url).await?.json().await
        };

        let (entry, image): (Entry, _) =
            futures::future::try_join(fetch_entry, Self::fetch_image(id))
                .await?;

        let description = entry
            .flavor_text_entries
            .iter()
            .filter(|text| text.language.name == "en")
            .next()
            .ok_or(Error::LanguageError)?;

        Ok(Pokemon {
            number: id,
            name: entry.name.to_uppercase(),
            description: description
                .flavor_text
                .chars()
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect(),
            image,
            image_viewer: image::viewer::State::new(),
        })
    }

    async fn fetch_image(id: u16) -> Result<image::Handle, reqwest::Error> {
        let url = format!(
            "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/{}.png",
            id
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = reqwest::get(&url).await?.bytes().await?;

            Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
        }

        #[cfg(target_arch = "wasm32")]
        Ok(image::Handle::from_path(url))
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        dbg!(error);

        Error::APIError
    }
}

fn button<'a>(state: &'a mut button::State, text: &str) -> Button<'a, Message> {
    Button::new(state, Text::new(text))
        .padding(10)
        .style(style::Button::Primary)
}

mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Primary,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
                })),
                border_radius: 12.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }
    }
}
