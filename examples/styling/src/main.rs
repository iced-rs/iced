use iced::theme::Palette;
use iced::theme::palette::Extended;
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, progress_bar, radio,
    row, scrollable, slider, text, text_input, toggler, vertical_rule,
    vertical_space,
};
use iced::{Alignment, Element, Length, Sandbox, Settings, Theme, Color};
use once_cell::sync::OnceCell;

pub fn main() -> iced::Result {
    let palette = Palette {
        background: Color::from_rgb(1.0, 0.9, 1.0),
        text: Color::BLACK,
        primary: Color::from_rgb(0.5, 0.5, 0.0),
        success: Color::from_rgb(0.0, 1.0, 0.0),
        danger: Color::from_rgb(1.0, 0.0, 0.0),
    };
    let extended = Extended::generate(palette);
    CUSTOM_THEME.set(Theme::Custom { palette, extended }).unwrap();
    Styling::run(Settings::default())
}

static CUSTOM_THEME: OnceCell<Theme> = OnceCell::new();

#[derive(Default)]
struct Styling {
    theme: Theme,
    input_value: String,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
    InputChanged(String),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
}

impl Sandbox for Styling {
    type Message = Message;

    fn new() -> Self {
        Styling::default()
    }

    fn title(&self) -> String {
        String::from("Styling - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::TogglerToggled(value) => self.toggler_value = value,
        }
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = [Theme::Light, Theme::Dark, *CUSTOM_THEME.get().unwrap()].iter().fold(
            column![text("Choose a theme:")].spacing(10),
            |column, theme| {
                column.push(radio(
                    match theme {
                        Theme::Light => "Light",
                        Theme::Dark => "Dark",
                        Theme::Custom { .. } => "Custom",
                    },
                    *theme,
                    Some(self.theme),
                    Message::ThemeChanged,
                ))
            },
        );

        let text_input = text_input(
            "Type something...",
            &self.input_value,
            Message::InputChanged,
        )
        .padding(10)
        .size(20);

        let button = button("Submit")
            .padding(10)
            .on_press(Message::ButtonPressed);

        let slider =
            slider(0.0..=100.0, self.slider_value, Message::SliderChanged);

        let progress_bar = progress_bar(0.0..=100.0, self.slider_value);

        let scrollable = scrollable(
            column![
                "Scroll me!",
                vertical_space(Length::Units(800)),
                "You did it!"
            ]
            .width(Length::Fill),
        )
        .height(Length::Units(100));

        let checkbox = checkbox(
            "Check me!",
            self.checkbox_value,
            Message::CheckboxToggled,
        );

        let toggler = toggler(
            String::from("Toggle me!"),
            self.toggler_value,
            Message::TogglerToggled,
        )
        .width(Length::Shrink)
        .spacing(10);

        let content = column![
            choose_theme,
            horizontal_rule(38),
            row![text_input, button].spacing(10),
            slider,
            progress_bar,
            row![
                scrollable,
                vertical_rule(38),
                column![checkbox, toggler].spacing(20)
            ]
            .spacing(10)
            .height(Length::Units(100))
            .align_items(Alignment::Center),
        ]
        .spacing(20)
        .padding(20)
        .max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
