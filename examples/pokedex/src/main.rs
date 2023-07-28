use iced::futures;
use iced::widget::{self, column, container, image, row, text};
use iced::{
    Alignment, Application, Color, Command, Element, Length, Settings, Theme,
};

pub fn main() -> iced::Result {
    Pokedex::run(Settings::default())
}

#[derive(Debug)]
enum Pokedex {
    Loading,
    Loaded { pokemon: Pokemon },
    Errored,
}

#[derive(Debug, Clone)]
enum Message {
    PokemonFound(Result<Pokemon, Error>),
    Search,
}

impl Application for Pokedex {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
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

        format!("{subtitle} - Pokédex")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PokemonFound(Ok(pokemon)) => {
                *self = Pokedex::Loaded { pokemon };

                Command::none()
            }
            Message::PokemonFound(Err(_error)) => {
                *self = Pokedex::Errored;

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

    fn view(&self) -> Element<Message> {
        let content = match self {
            Pokedex::Loading => {
                column![text("Searching for Pokémon...").size(40),]
                    .width(Length::Shrink)
            }
            Pokedex::Loaded { pokemon } => column![
                pokemon.view(),
                button("Keep searching!").on_press(Message::Search)
            ]
            .max_width(500)
            .spacing(20)
            .align_items(Alignment::End),
            Pokedex::Errored => column![
                text("Whoops! Something went wrong...").size(40),
                button("Try again").on_press(Message::Search)
            ]
            .spacing(20)
            .align_items(Alignment::End),
        };

        container(content)
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
        row![
            image::viewer(self.image.clone()),
            column![
                row![
                    text(&self.name).size(30).width(Length::Fill),
                    text(format!("#{}", self.number))
                        .size(20)
                        .style(Color::from([0.5, 0.5, 0.5])),
                ]
                .align_items(Alignment::Center)
                .spacing(20),
                self.description.as_ref(),
            ]
            .spacing(20),
        ]
        .spacing(20)
        .align_items(Alignment::Center)
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
            let mut rng = rand::rngs::OsRng;

            rng.gen_range(0, Pokemon::TOTAL)
        };

        let fetch_entry = async {
            let url = format!("https://pokeapi.co/api/v2/pokemon-species/{id}");

            reqwest::get(&url).await?.json().await
        };

        let (entry, image): (Entry, _) =
            futures::future::try_join(fetch_entry, Self::fetch_image(id))
                .await?;

        let description = entry
            .flavor_text_entries
            .iter()
            .find(|text| text.language.name == "en")
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
        })
    }

    async fn fetch_image(id: u16) -> Result<image::Handle, reqwest::Error> {
        let url = format!(
            "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/{id}.png"
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = reqwest::get(&url).await?.bytes().await?;

            Ok(image::Handle::from_memory(bytes))
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

fn button(text: &str) -> widget::Button<'_, Message> {
    widget::button(text).padding(10)
}
