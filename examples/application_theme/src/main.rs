mod themes;

use themes::{Styling, Theme, ThemeChoice};

// use iced::{
//     button, scrollable, slider, text_input, Alignment, Button, Checkbox, Color,
//     Column, Container, Element, Length, ProgressBar, Radio, Row, Rule, Sandbox,
//     Scrollable, Settings, Slider, Space, Styling, Text, TextInput, Toggler,
// };
use iced::button::{Style, StyleSheet};
use iced::{
    button, Alignment, Button, Color, Column, Container, Element, Length,
    Radio, Row, Sandbox, Settings, Space, Text, Vector,
};

pub fn main() -> iced::Result {
    ApplicationTheme::run(Settings::default())
}

#[derive(Default)]
struct ApplicationTheme {
    theme: ThemeChoice,
    // scroll: scrollable::State,
    // input: text_input::State,
    // input_value: String,
    submit_button: button::State,
    custom_style_button: button::State,
    // slider: slider::State,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(ThemeChoice),
    InputChanged(String),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
}

impl Sandbox for ApplicationTheme {
    type Message = Message;
    type Styling = Styling;

    fn new() -> Self {
        ApplicationTheme::default()
    }

    fn title(&self) -> String {
        String::from("Application Theme - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
            // Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => {}
            // Message::SliderChanged(value) => self.slider_value = value,
            // Message::CheckboxToggled(value) => self.checkbox_value = value,
            // Message::TogglerToggled(value) => self.toggler_value = value,
            _ => {}
        }
    }

    fn view(&mut self) -> Element<Self::Message, Self::Styling> {
        let choose_theme = ThemeChoice::ALL.iter().fold(
            Column::new().spacing(10).push(Text::new("Choose a theme:")),
            |column, theme| {
                column.push(Radio::new(
                    *theme,
                    format!("{:?}", theme),
                    Some(self.theme),
                    Message::ThemeChanged,
                ))
            },
        );

        // let text_input = TextInput::new(
        //     &mut self.input,
        //     "Type something...",
        //     &self.input_value,
        //     Message::InputChanged,
        // )
        // .padding(10)
        // .size(20);

        let submit_button =
            Button::new(&mut self.submit_button, Text::new("Submit"))
                .padding(10)
                .on_press(Message::ButtonPressed);

        // let slider = Slider::new(
        //     &mut self.slider,
        //     0.0..=100.0,
        //     self.slider_value,
        //     Message::SliderChanged,
        // );
        //
        // let progress_bar = ProgressBar::new(0.0..=100.0, self.slider_value);
        //
        // let scrollable = Scrollable::new(&mut self.scroll)
        //     .width(Length::Fill)
        //     .height(Length::Units(100))
        //     .push(Text::new("Scroll me!"))
        //     .push(Space::with_height(Length::Units(800)))
        //     .push(Text::new("You did it!"));
        //
        // let checkbox = Checkbox::new(
        //     self.checkbox_value,
        //     "Check me!",
        //     Message::CheckboxToggled,
        // );
        //
        // let toggler = Toggler::new(
        //     self.toggler_value,
        //     String::from("Toggle me!"),
        //     Message::TogglerToggled,
        // )
        // .width(Length::Shrink)
        // .spacing(10);

        let custom_style_button = Button::new(
            &mut self.custom_style_button,
            Text::new("This button has custom style and will look the same regardless of theme").color(Color::WHITE)
        )
        .padding(10)
        .on_press(Message::ButtonPressed).style(CustomButtonStyling);

        let content = Column::new()
            .spacing(20)
            .padding(20)
            .max_width(600)
            .push(choose_theme)
            // .push(Rule::horizontal(38))
            // .push(Row::new().spacing(10).push(text_input).push(submit_button))
            .push(Row::new().spacing(10).push(submit_button))
            // .push(slider)
            // .push(progress_bar)
            // .push(
            //     Row::new()
            //         .spacing(10)
            //         .height(Length::Units(100))
            //         .align_items(Alignment::Center)
            //         .push(scrollable)
            //         .push(Rule::vertical(38))
            //         .push(
            //             Column::new()
            //                 .width(Length::Shrink)
            //                 .spacing(20)
            //                 .push(checkbox)
            //                 .push(toggler),
            //         ),
            // )
            .push(custom_style_button);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.into()
    }
}

pub struct CustomButtonStyling;

impl button::StyleSheet for CustomButtonStyling {
    type Theme = Theme;

    fn active(&self, theme: &Theme) -> button::Style {
        button::Style {
            background: Color::from_rgb(0.02, 0.54, 0.14).into(),
            border_radius: 50.0,
            shadow_offset: Vector::default(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    fn hovered(&self, theme: &Theme) -> button::Style {
        button::Style {
            background: Color::from_rgb(0.03, 0.64, 0.16).into(),
            ..self.active(theme)
        }
    }

    fn pressed(&self, theme: &Theme) -> button::Style {
        button::Style {
            border_width: 1.0,
            border_color: Color::WHITE,
            ..self.hovered(theme)
        }
    }

    fn disabled(&self, theme: &Theme) -> Style {
        self.pressed(theme)
    }
}
