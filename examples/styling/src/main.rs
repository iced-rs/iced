use iced::keyboard;
use iced::widget::{
    button, center_x, center_y, checkbox, column, container, pick_list,
    progress_bar, row, rule, scrollable, slider, space, text, text_input,
    toggler,
};
use iced::{Center, Element, Fill, Shrink, Subscription, Theme};

pub fn main() -> iced::Result {
    iced::application(Styling::default, Styling::update, Styling::view)
        .subscription(Styling::subscription)
        .theme(Styling::theme)
        .run()
}

#[derive(Default)]
struct Styling {
    theme: Option<Theme>,
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
    PreviousTheme,
    NextTheme,
    ClearTheme,
}

impl Styling {
    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = Some(theme);
            }
            Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::TogglerToggled(value) => self.toggler_value = value,
            Message::PreviousTheme | Message::NextTheme => {
                let current = Theme::ALL.iter().position(|candidate| {
                    self.theme.as_ref() == Some(candidate)
                });

                self.theme = Some(if matches!(message, Message::NextTheme) {
                    Theme::ALL[current.map(|current| current + 1).unwrap_or(0)
                        % Theme::ALL.len()]
                    .clone()
                } else {
                    let current = current.unwrap_or(0);

                    if current == 0 {
                        Theme::ALL
                            .last()
                            .expect("Theme::ALL must not be empty")
                            .clone()
                    } else {
                        Theme::ALL[current - 1].clone()
                    }
                });
            }
            Message::ClearTheme => {
                self.theme = None;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let choose_theme = column![
            text("Theme:"),
            pick_list(Theme::ALL, self.theme.as_ref(), Message::ThemeChanged)
                .width(Fill)
                .placeholder("System"),
        ]
        .spacing(10);

        let text_input = text_input("Type something...", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(20);

        let buttons = {
            let styles = [
                ("Primary", button::primary as fn(&Theme, _) -> _),
                ("Secondary", button::secondary),
                ("Success", button::success),
                ("Warning", button::warning),
                ("Danger", button::danger),
            ];

            let styled_button =
                |label| button(text(label).width(Fill).center()).padding(10);

            column![
                row(styles.into_iter().map(|(name, style)| styled_button(
                    name
                )
                .on_press(Message::ButtonPressed)
                .style(style)
                .into()))
                .spacing(10)
                .align_y(Center),
                row(styles.into_iter().map(|(name, style)| styled_button(
                    name
                )
                .style(style)
                .into()))
                .spacing(10)
                .align_y(Center),
            ]
            .spacing(10)
        };

        let slider =
            || slider(0.0..=100.0, self.slider_value, Message::SliderChanged);

        let progress_bar = || progress_bar(0.0..=100.0, self.slider_value);

        let scroll_me = scrollable(column![
            "Scroll me!",
            space().height(800),
            "You did it!"
        ])
        .width(Fill)
        .height(Fill);

        let check = checkbox("Check me!", self.checkbox_value)
            .on_toggle(Message::CheckboxToggled);

        let check_disabled = checkbox("Disabled", self.checkbox_value);

        let toggle = toggler(self.toggler_value)
            .label("Toggle me!")
            .on_toggle(Message::TogglerToggled)
            .spacing(10);

        let disabled_toggle =
            toggler(self.toggler_value).label("Disabled").spacing(10);

        let card = {
            container(
                column![
                    text("Card Example").size(24),
                    slider(),
                    progress_bar(),
                ]
                .spacing(20),
            )
            .width(Fill)
            .padding(20)
            .style(container::bordered_box)
        };

        let content = column![
            choose_theme,
            rule::horizontal(1),
            text_input,
            buttons,
            slider(),
            progress_bar(),
            row![
                scroll_me,
                rule::vertical(1),
                column![check, check_disabled, toggle, disabled_toggle]
                    .spacing(10)
            ]
            .spacing(10)
            .height(Shrink)
            .align_y(Center),
            card
        ]
        .spacing(20)
        .padding(20)
        .max_width(600);

        center_y(scrollable(center_x(content)).spacing(10))
            .padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, _modifiers| match key {
            keyboard::Key::Named(
                keyboard::key::Named::ArrowUp | keyboard::key::Named::ArrowLeft,
            ) => Some(Message::PreviousTheme),
            keyboard::Key::Named(
                keyboard::key::Named::ArrowDown
                | keyboard::key::Named::ArrowRight,
            ) => Some(Message::NextTheme),
            keyboard::Key::Named(keyboard::key::Named::Space) => {
                Some(Message::ClearTheme)
            }
            _ => None,
        })
    }

    fn theme(&self) -> Option<Theme> {
        self.theme.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;

    use iced_test::{Error, simulator};

    #[test]
    #[ignore]
    fn it_showcases_every_theme() -> Result<(), Error> {
        Theme::ALL
            .par_iter()
            .cloned()
            .map(|theme| {
                let mut styling = Styling::default();
                styling.update(Message::ThemeChanged(theme.clone()));

                let mut ui = simulator(styling.view());
                let snapshot = ui.snapshot(&theme)?;

                assert!(
                    snapshot.matches_hash(format!(
                        "snapshots/{theme}",
                        theme = theme
                            .to_string()
                            .to_ascii_lowercase()
                            .replace(" ", "_")
                    ))?,
                    "snapshots for {theme} should match!"
                );

                Ok(())
            })
            .collect()
    }
}
