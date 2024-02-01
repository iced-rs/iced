use iced::executor;
use iced::font::{self, Font};
use iced::theme;
use iced::widget::{checkbox, column, container, row, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

const ICON_FONT: Font = Font::with_name("icons");

pub fn main() -> iced::Result {
    Example::run(Settings::default())
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
    FontLoaded(Result<(), font::Error>),
}

impl Application for Example {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self::default(),
            font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                .map(Message::FontLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("Checkbox - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
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
            Message::FontLoaded(_) => (),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let default_checkbox = checkbox("Default", self.default)
            .on_toggle(Message::DefaultToggled);

        let styled_checkbox = |label, style| {
            checkbox(label, self.styled)
                .on_toggle_maybe(self.default.then(|| Message::StyledToggled))
                .style(style)
        };

        let checkboxes = row![
            styled_checkbox("Primary", theme::Checkbox::Primary),
            styled_checkbox("Secondary", theme::Checkbox::Secondary),
            styled_checkbox("Success", theme::Checkbox::Success),
            styled_checkbox("Danger", theme::Checkbox::Danger),
        ]
        .spacing(10);

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
            column![default_checkbox, checkboxes, custom_checkbox,].spacing(22);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
