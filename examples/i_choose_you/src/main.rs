use iced::widget::combo_box::{self, ComboBox};
use iced::widget::{column, Container, Space, Text};
use iced::{Alignment, Length, Sandbox, Settings};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

struct PokemonPicker {
    pokemon: Pokemon,
    // We store the state of the combo box
    // that in turn contains the vector of choices
    // because displaying the combo box requires
    // the vector of choices to stay alive throughout the lifetime
    // of the combo box.
    choices: combo_box::State<Pokemon>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    PokemonSelected(Pokemon),
}

// `strum::EnumIter` derives `iter()` function for an enum, which is useful
// to create a vector of choices.
// `strum::IntoStaticStr` derives `Into<&'static str>` for an enum, which is useful
// to display the enum variant names.
#[derive(Debug, Default, Clone, Copy, EnumIter, IntoStaticStr)]
enum Pokemon {
    Bulbasaur,
    Charmander,
    Meowth,
    #[default]
    Pikachu,
    Squirtle,
}

impl Sandbox for PokemonPicker {
    type Message = Message;

    fn new() -> Self {
        PokemonPicker {
            pokemon: Pokemon::default(),
            choices: combo_box::State::new(Pokemon::iter().collect()),
        }
    }

    fn title(&self) -> String {
        String::from("I choose You!")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::PokemonSelected(pokemon) => self.pokemon = pokemon,
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let header = Text::new("Choose your Pokemon!").size(50);

        let prompt = Text::new(format!("Which one will you pick?"));

        let picker = {
            let selected = Some(&self.pokemon);
            ComboBox::new(
                &self.choices,
                "Select a pokemon!",
                selected,
                |selected_pokemon| Message::PokemonSelected(selected_pokemon),
            )
            .width(200)
        };

        let text = Text::new(format!("You chose: {}", self.pokemon));

        let col = column![
            header,
            Space::with_height(20),
            prompt,
            Space::with_height(20),
            picker,
            Space::with_height(20),
            text,
        ]
        .align_items(Alignment::Center);

        let container = Container::new(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();
        container.into()
    }
}

impl std::fmt::Display for Pokemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}

fn main() -> iced::Result {
    PokemonPicker::run(Settings::default())
}
