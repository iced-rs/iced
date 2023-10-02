use iced::executor;
use iced::font::{self, Font};
use iced::theme::Checkbox;
use iced::widget::{checkbox, column, container, row, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

const ICON_FONT: Font = Font::with_name("icons");

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    default_checkbox: bool,
    custom_checkbox: bool,
    styled_checkbox_enabled: bool,
    styled_checkbox_checked: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    DefaultChecked(bool),
    CustomChecked(bool),
    StyledChecked(bool),
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
            Message::DefaultChecked(value) => {
                self.default_checkbox = value;
                self.styled_checkbox_enabled = value;
            }
            Message::CustomChecked(value) => self.custom_checkbox = value,
            Message::StyledChecked(value) => {
                self.styled_checkbox_checked = value
            }
            Message::FontLoaded(_) => (),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let default_checkbox = checkbox("Default", self.default_checkbox)
            .on_toggle(Message::DefaultChecked);

        let checkboxes = [
            ("Primary", Checkbox::Primary),
            ("Secondary", Checkbox::Secondary),
            ("Success", Checkbox::Success),
            ("Danger", Checkbox::Danger),
        ];

        let checkboxes = row(checkboxes
            .into_iter()
            .map(|(label, style)| {
                checkbox(label, self.styled_checkbox_checked)
                    .on_toggle_maybe(
                        self.default_checkbox.then(|| Message::StyledChecked),
                    )
                    .style(style)
            })
            .map(Element::from))
        .spacing(10);

        let custom_checkbox = checkbox("Custom", self.custom_checkbox)
            .on_toggle(Message::CustomChecked)
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
