use iced::theme::Mode;
use iced::widget::text::Wrapping;
use iced::widget::{
    Space, button, column, container, pick_list, radio, row, rule, slider, text_editor,
};
use iced::{Center, Element, Fill, Font, Pixels, Shrink, Task, color};
use std::collections::HashSet;

pub fn main() -> iced::Result {
    iced::application(Example::new, Example::update, Example::view)
        .theme(Example::theme)
        .window_size((820.0, 820.0))
        .settings(iced::Settings {
            default_font: Font::with_name("IBM Plex Sans"),
            default_text_size: 14.into(),
            ..Default::default()
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    LetterSpacing(f32),
    FontSize(FontSize),
    FontChoice(FontChoice),
    RememberMe(bool),
    ThemeChoice(ThemeChoice),
    Density(Density),
    UsernameChanged(String),
    PasswordChanged(String),
    Editor(text_editor::Action),
    FontLoaded(FontChoice),
}

struct Example {
    letter_spacing: f32,
    font_size: FontSize,
    font_choice: FontChoice,
    available_fonts: HashSet<FontChoice>,
    remember_me: bool,
    theme_choice: ThemeChoice,
    display_density: Density,
    username: String,
    password: String,
    message: text_editor::Content,
}

impl Example {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                letter_spacing: -0.05,
                font_size: FontSize::Medium,
                font_choice: FontChoice::IBMPlexSans,
                available_fonts: HashSet::new(),
                remember_me: false,
                theme_choice: ThemeChoice::Dark,
                display_density: Density::Comfortable,
                username: String::new(),
                password: String::new(),
                message: text_editor::Content::new(),
            },
            Task::batch(
                FontChoice::ALL
                    .iter()
                    .flat_map(|f| f.urls().iter().copied().map(|url| fetch_font(*f, url)))
                    .chain([iced::system::theme().map(|t| Message::ThemeChoice(t.into()))]),
            ),
        )
    }

    fn spacing(&self) -> f32 {
        match self.display_density {
            Density::Compact => 8.0,
            Density::Comfortable => 12.0,
            Density::Spacious => 16.0,
        }
    }

    fn theme(&self) -> iced::Theme {
        let palette = match self.theme_choice {
            ThemeChoice::Light => iced::theme::Palette {
                background: color!(0xFFFFFF),
                text: color!(0x737373),
                primary: color!(0x000000),
                success: color!(0x22c55e),
                danger: color!(0xef4444),
                warning: color!(0xf59e0b),
            },
            ThemeChoice::Dark => iced::theme::Palette {
                background: color!(0x09090b),
                text: color!(0xa1a1aa),
                primary: color!(0xfafafa),
                success: color!(0x22c55e),
                danger: color!(0xef4444),
                warning: color!(0xf59e0b),
            },
        };

        iced::Theme::custom("Custom".to_string(), palette)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LetterSpacing(value) => {
                self.letter_spacing = value;
            }
            Message::FontSize(size) => {
                self.font_size = size;
            }
            Message::FontChoice(choice) => {
                self.font_choice = choice;
            }
            Message::RememberMe(enabled) => {
                self.remember_me = enabled;
            }
            Message::ThemeChoice(choice) => {
                self.theme_choice = choice;
            }
            Message::Density(density) => {
                self.display_density = density;
            }
            Message::UsernameChanged(username) => {
                self.username = username;
            }
            Message::PasswordChanged(password) => {
                self.password = password;
            }
            Message::Editor(action) => {
                self.message.perform(action);
            }
            Message::FontLoaded(choice) => {
                self.available_fonts.insert(choice);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let spacing = self.spacing();
        let font_size = self.font_size.to_pixels();
        let title_size = self.font_size.title_size();
        let font = self.font_choice.to_font();

        column![
            header(self.letter_spacing, font),
            card(
                controls(
                    self.theme_choice,
                    self.letter_spacing,
                    self.font_size,
                    self.font_choice,
                    self.display_density,
                    &self.available_fonts,
                    spacing,
                    title_size,
                    font_size,
                    font,
                ),
                spacing,
            ),
            card(
                login(
                    &self.username,
                    &self.password,
                    self.remember_me,
                    self.letter_spacing,
                    spacing,
                    title_size,
                    font_size,
                    font,
                ),
                spacing,
            ),
            card(
                form(
                    &self.message,
                    self.letter_spacing,
                    spacing,
                    title_size,
                    font_size,
                    font,
                ),
                spacing,
            ),
        ]
        .spacing(12)
        .padding(24)
        .align_x(Center)
        .into()
    }
}

fn header(letter_spacing: f32, font: Font) -> Element<'static, Message> {
    column![
        text("Letter Spacing", font)
            .size(36)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..font
            })
            .letter_spacing(letter_spacing)
            .style(theme::text::header),
        row![
            Space::new().width(Fill),
            slider(-0.10..=0.10, letter_spacing, Message::LetterSpacing)
                .step(0.01)
                .width(200)
                .height(16)
                .style(theme::slider::thin),
            text(format!("{:.2} em", letter_spacing), font)
                .size(13)
                .width(60),
        ]
        .spacing(8)
        .align_y(Center),
    ]
    .spacing(8)
    .align_x(Center)
    .into()
}

fn card<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    padding: f32,
) -> Element<'a, Message> {
    container(content)
        .padding(padding)
        .style(theme::container::card)
        .into()
}

fn controls(
    theme_choice: ThemeChoice,
    letter_spacing: f32,
    font_size_choice: FontSize,
    font_choice: FontChoice,
    display_density: Density,
    available_fonts: &HashSet<FontChoice>,
    spacing: f32,
    title_size: Pixels,
    font_size: Pixels,
    font: Font,
) -> Element<'static, Message> {
    column![
        text("Controls", font)
            .size(title_size)
            .letter_spacing(letter_spacing)
            .style(theme::text::header),
        row![
            column![
                text("Theme", font)
                    .size(font_size)
                    .letter_spacing(letter_spacing)
                    .wrapping(Wrapping::None),
                column(ThemeChoice::ALL.iter().map(|choice| {
                    radio(
                        choice.to_string(),
                        *choice,
                        Some(theme_choice),
                        Message::ThemeChoice,
                    )
                    .font(font)
                    .size(font_size)
                    .text_size(font_size)
                    .letter_spacing(letter_spacing)
                    .into()
                }))
                .spacing(6),
            ]
            .padding(iced::padding::right(20))
            .spacing(4),
            rule::vertical(1),
            column![
                text("Font Size", font)
                    .size(font_size)
                    .letter_spacing(letter_spacing)
                    .wrapping(Wrapping::None),
                segmented_button(font_size_choice, letter_spacing, font_size, font),
            ]
            .spacing(4)
            .width(Fill),
            rule::vertical(1),
            column![
                text("Density", font)
                    .size(font_size)
                    .letter_spacing(letter_spacing)
                    .wrapping(Wrapping::None),
                pick_list(Some(display_density), Density::ALL, |d: &Density| d
                    .to_string(),)
                .on_select(Message::Density)
                .text_size(font_size)
                .font(font)
                .letter_spacing(letter_spacing),
            ]
            .spacing(4),
            rule::vertical(1),
            column![
                text("Font", font)
                    .size(font_size)
                    .letter_spacing(letter_spacing)
                    .wrapping(Wrapping::None),
                pick_list(
                    available_fonts
                        .contains(&font_choice)
                        .then_some(font_choice),
                    FontChoice::ALL
                        .iter()
                        .copied()
                        .filter(|f| available_fonts.contains(f))
                        .collect::<Vec<_>>(),
                    |f: &FontChoice| f.to_string(),
                )
                .on_select(Message::FontChoice)
                .text_size(font_size)
                .font(font)
                .letter_spacing(letter_spacing),
            ]
            .spacing(4),
        ]
        .height(Shrink)
        .spacing(spacing)
        .align_y(Center),
    ]
    .spacing(spacing)
    .width(Fill)
    .into()
}

fn segmented_button(
    current: FontSize,
    letter_spacing: f32,
    font_size: Pixels,
    font: Font,
) -> Element<'static, Message> {
    container(
        row![
            button(
                text("Small", font)
                    .size(font_size)
                    .letter_spacing(letter_spacing)
            )
            .padding([3, 8])
            .style(move |theme, status| {
                theme::button::segmented(theme, status, current == FontSize::Small)
            })
            .on_press(Message::FontSize(FontSize::Small)),
            button(
                text("Medium", font)
                    .size(font_size)
                    .letter_spacing(letter_spacing)
            )
            .padding([3, 8])
            .style(move |theme, status| {
                theme::button::segmented(theme, status, current == FontSize::Medium)
            })
            .on_press(Message::FontSize(FontSize::Medium)),
            button(
                text("Large", font)
                    .size(font_size)
                    .letter_spacing(letter_spacing)
            )
            .padding([3, 8])
            .style(move |theme, status| {
                theme::button::segmented(theme, status, current == FontSize::Large)
            })
            .on_press(Message::FontSize(FontSize::Large)),
        ]
        .spacing(3),
    )
    .clip(true)
    .padding(3)
    .style(theme::container::segmented_picker)
    .into()
}

fn login<'a>(
    username: &'a str,
    password: &'a str,
    remember_me: bool,
    letter_spacing: f32,
    spacing: f32,
    title_size: Pixels,
    font_size: Pixels,
    font: Font,
) -> Element<'a, Message> {
    column![
        text("Login", font)
            .size(title_size)
            .letter_spacing(letter_spacing)
            .style(theme::text::header),
        column![
            text("Username", font)
                .size(font_size)
                .letter_spacing(letter_spacing),
            text_input("Enter your username", username, font)
                .on_input(Message::UsernameChanged)
                .size(font_size)
                .letter_spacing(letter_spacing)
                .padding(8),
        ]
        .spacing(4),
        column![
            text("Password", font)
                .size(font_size)
                .letter_spacing(letter_spacing),
            text_input("Enter your password", password, font)
                .on_input(Message::PasswordChanged)
                .secure(true)
                .size(font_size)
                .letter_spacing(letter_spacing)
                .padding(8),
        ]
        .spacing(4),
        checkbox(remember_me, font)
            .label("Remember me")
            .on_toggle(Message::RememberMe)
            .size(font_size)
            .text_size(font_size)
            .letter_spacing(letter_spacing),
    ]
    .spacing(spacing)
    .into()
}

fn form<'a>(
    content: &'a text_editor::Content,
    letter_spacing: f32,
    spacing: f32,
    title_size: Pixels,
    font_size: Pixels,
    font: Font,
) -> Element<'a, Message> {
    column![
        text("Message", font)
            .size(title_size)
            .letter_spacing(letter_spacing)
            .style(theme::text::header),
        text("Write your message here", font)
            .size(font_size)
            .letter_spacing(letter_spacing),
        container(
            text_editor(content)
                .on_action(Message::Editor)
                .placeholder("Type your message...")
                .height(120)
                .font(Font::MONOSPACE)
                .size(font_size)
                .letter_spacing(letter_spacing)
        )
        .style(theme::container::text_editor),
    ]
    .spacing(spacing)
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    Light,
    Dark,
}

impl ThemeChoice {
    const ALL: &[Self] = &[Self::Light, Self::Dark];
}

impl Into<ThemeChoice> for Mode {
    fn into(self) -> ThemeChoice {
        match self {
            Mode::Light => ThemeChoice::Light,
            Mode::Dark | Mode::None => ThemeChoice::Dark,
        }
    }
}

impl std::fmt::Display for ThemeChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Dark => write!(f, "Dark"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FontSize {
    Small,
    Medium,
    Large,
}

impl FontSize {
    fn to_pixels(self) -> Pixels {
        match self {
            Self::Small => 12.into(),
            Self::Medium => 14.into(),
            Self::Large => 16.into(),
        }
    }

    fn title_size(self) -> Pixels {
        match self {
            Self::Small => 15.into(),
            Self::Medium => 18.into(),
            Self::Large => 21.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FontChoice {
    IBMPlexSans,
    Lato,
}

impl FontChoice {
    const ALL: &[Self] = &[Self::IBMPlexSans, Self::Lato];

    fn urls(self) -> &'static [&'static str] {
        match self {
            Self::IBMPlexSans => &[
                "https://raw.githubusercontent.com/IBM/plex/master/packages/plex-sans/fonts/complete/ttf/IBMPlexSans-Regular.ttf",
                "https://raw.githubusercontent.com/IBM/plex/master/packages/plex-sans/fonts/complete/ttf/IBMPlexSans-Bold.ttf",
            ],
            Self::Lato => &[
                "https://raw.githubusercontent.com/google/fonts/main/ofl/lato/Lato-Regular.ttf",
                "https://raw.githubusercontent.com/google/fonts/main/ofl/lato/Lato-Bold.ttf",
            ],
        }
    }

    fn to_font(self) -> Font {
        match self {
            Self::IBMPlexSans => Font::with_name("IBM Plex Sans"),
            Self::Lato => Font::with_name("Lato"),
        }
    }
}

async fn fetch_bytes(url: &'static str) -> Result<Vec<u8>, String> {
    use http_body_util::{BodyExt, Empty};
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let request = hyper::Request::get(url)
        .body(Empty::<Bytes>::new())
        .map_err(|e| format!("{e}"))?;
    let client = Client::builder(TokioExecutor::new()).build(hyper_tls::HttpsConnector::new());
    let response = client.request(request).await.map_err(|e| format!("{e}"))?;
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("{e}"))?;
    Ok(body.to_bytes().to_vec())
}

impl std::fmt::Display for FontChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IBMPlexSans => write!(f, "IBM Plex Sans"),
            Self::Lato => write!(f, "Lato"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Density {
    Compact,
    Comfortable,
    Spacious,
}

impl Density {
    const ALL: &[Self] = &[Self::Compact, Self::Comfortable, Self::Spacious];
}

impl std::fmt::Display for Density {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compact => write!(f, "Compact"),
            Self::Comfortable => write!(f, "Comfortable"),
            Self::Spacious => write!(f, "Spacious"),
        }
    }
}

fn text<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    font: Font,
) -> iced::widget::Text<'a> {
    iced::widget::Text::new(content).font(font)
}

fn text_input<'a>(
    placeholder: &str,
    value: &str,
    font: Font,
) -> iced::widget::TextInput<'a, Message> {
    iced::widget::TextInput::new(placeholder, value).font(font)
}

fn checkbox<'a>(is_checked: bool, font: Font) -> iced::widget::Checkbox<'a, Message> {
    iced::widget::Checkbox::new(is_checked).font(font)
}

fn fetch_font(choice: FontChoice, url: &'static str) -> Task<Message> {
    Task::future(fetch_bytes(url)).then(move |result| match result {
        Ok(bytes) => iced::font::load(bytes).map(move |_| Message::FontLoaded(choice)),
        Err(_) => Task::none(),
    })
}

mod theme {
    use iced::{Border, widget};

    pub mod button {
        use super::*;

        pub fn segmented(
            theme: &iced::Theme,
            status: widget::button::Status,
            selected: bool,
        ) -> widget::button::Style {
            let palette = theme.extended_palette();

            if selected {
                widget::button::Style {
                    background: Some(palette.primary.base.color.into()),
                    text_color: palette.background.base.color,
                    border: iced::border::rounded(10.0),
                    ..Default::default()
                }
            } else {
                let background = match status {
                    widget::button::Status::Hovered => Some(palette.background.weak.color.into()),
                    _ => None,
                };

                widget::button::Style {
                    background,
                    text_color: palette.background.strong.text,
                    border: iced::border::rounded(10.0),
                    ..Default::default()
                }
            }
        }
    }

    pub mod text {
        use super::*;

        pub fn header(theme: &iced::Theme) -> widget::text::Style {
            widget::text::Style {
                color: Some(theme.extended_palette().primary.base.color),
            }
        }
    }

    pub mod slider {
        use super::*;

        pub fn thin(theme: &iced::Theme, status: widget::slider::Status) -> widget::slider::Style {
            let palette = theme.extended_palette();

            let handle_color = match status {
                widget::slider::Status::Active => palette.primary.base.color,
                widget::slider::Status::Hovered => palette.primary.strong.color,
                widget::slider::Status::Dragged => palette.primary.weak.color,
            };

            widget::slider::Style {
                rail: widget::slider::Rail {
                    backgrounds: (handle_color.into(), palette.background.strong.color.into()),
                    width: 1.0,
                    border: iced::border::rounded(1.0),
                },
                handle: widget::slider::Handle {
                    shape: widget::slider::HandleShape::Circle { radius: 7.0 },
                    background: palette.background.base.color.into(),
                    border_width: 1.0,
                    border_color: handle_color,
                },
            }
        }
    }

    pub mod container {
        use super::*;

        pub fn card(theme: &iced::Theme) -> widget::container::Style {
            let palette = theme.extended_palette();
            widget::container::Style {
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }

        pub fn segmented_picker(theme: &iced::Theme) -> widget::container::Style {
            widget::container::Style {
                background: Some(theme.extended_palette().background.strong.color.into()),
                border: iced::border::rounded(12.0),
                ..Default::default()
            }
        }

        pub fn text_editor(theme: &iced::Theme) -> widget::container::Style {
            let palette = theme.extended_palette();
            widget::container::Style {
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }
    }
}
