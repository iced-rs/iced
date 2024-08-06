use iced::widget::{center, checkbox, column, row, text};
use iced::{Element, Font};

const ICON_FONT: Font = Font::with_name("icons");

pub fn main() -> iced::Result {
    iced::application("Checkbox - Iced", Example::update, Example::view)
        .font(include_bytes!("../fonts/icons.ttf").as_slice())
        .run()
}

#[derive(Default)]
struct Example {
    default: bool,
    styled: bool,
    custom: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    DefaultToggled(bool),
    CustomToggled(bool),
    StyledToggled(bool),
}

impl Example {
    fn update(&mut self, message: Message) {
        match message {
            Message::DefaultToggled(default) => {
                self.default = default;
            }
            Message::StyledToggled(styled) => {
                self.styled = styled;
            }
            Message::CustomToggled(custom) => {
                self.custom = custom;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let default_checkbox = checkbox("Default", self.default)
            .on_toggle(Message::DefaultToggled);

        let styled_checkbox = |label| {
            checkbox(label, self.styled)
                .on_toggle_maybe(self.default.then_some(Message::StyledToggled))
        };

        let checkboxes = row![
            styled_checkbox("Primary").style(checkbox::primary),
            styled_checkbox("Secondary").style(checkbox::secondary),
            styled_checkbox("Success").style(checkbox::success),
            styled_checkbox("Danger").style(checkbox::danger),
        ]
        .spacing(20);

        let custom_checkbox = checkbox("Custom", self.custom)
            .on_toggle(Message::CustomToggled)
            .icon(checkbox::Icon {
                font: ICON_FONT,
                code_point: '\u{e901}',
                size: None,
                line_height: text::LineHeight::Relative(1.0),
                shaping: text::Shaping::Basic,
            });

        let content =
            column![default_checkbox, checkboxes, custom_checkbox].spacing(20);

        center(content).into()
    }
}
