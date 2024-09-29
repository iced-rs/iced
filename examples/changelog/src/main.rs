mod changelog;

use crate::changelog::Changelog;

use iced::font;
use iced::widget::{
    button, center, column, container, markdown, pick_list, progress_bar,
    rich_text, row, scrollable, span, stack, text, text_input,
};
use iced::{Center, Element, Fill, FillPortion, Font, Task, Theme};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Changelog Generator", Generator::update, Generator::view)
        .theme(Generator::theme)
        .run_with(Generator::new)
}

enum Generator {
    Loading,
    Reviewing {
        changelog: Changelog,
        pending: Vec<changelog::Contribution>,
        state: State,
        preview: Vec<markdown::Item>,
    },
    Done,
}

enum State {
    Loading(changelog::Contribution),
    Loaded {
        pull_request: changelog::PullRequest,
        description: Vec<markdown::Item>,
        title: String,
        category: changelog::Category,
    },
}

#[derive(Debug, Clone)]
enum Message {
    ChangelogListed(
        Result<(Changelog, Vec<changelog::Contribution>), changelog::Error>,
    ),
    PullRequestFetched(Result<changelog::PullRequest, changelog::Error>),
    UrlClicked(markdown::Url),
    TitleChanged(String),
    CategorySelected(changelog::Category),
    Next,
    OpenPullRequest(u64),
    ChangelogSaved(Result<(), changelog::Error>),
    Quit,
}

impl Generator {
    fn new() -> (Self, Task<Message>) {
        (
            Self::Loading,
            Task::perform(Changelog::list(), Message::ChangelogListed),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangelogListed(Ok((changelog, mut pending))) => {
                if let Some(contribution) = pending.pop() {
                    let preview =
                        markdown::parse(&changelog.to_string()).collect();

                    *self = Self::Reviewing {
                        changelog,
                        pending,
                        state: State::Loading(contribution.clone()),
                        preview,
                    };

                    Task::perform(
                        changelog::PullRequest::fetch(contribution),
                        Message::PullRequestFetched,
                    )
                } else {
                    *self = Self::Done;

                    Task::none()
                }
            }
            Message::PullRequestFetched(Ok(pull_request)) => {
                let Self::Reviewing { state, .. } = self else {
                    return Task::none();
                };

                let description = markdown::parse(
                    pull_request
                        .description
                        .as_deref()
                        .unwrap_or("*No description provided*"),
                )
                .collect();

                *state = State::Loaded {
                    title: pull_request.title.clone(),
                    category: pull_request
                        .labels
                        .iter()
                        .map(String::as_str)
                        .filter_map(changelog::Category::guess)
                        .next()
                        .unwrap_or(changelog::Category::Added),
                    pull_request,
                    description,
                };

                Task::none()
            }
            Message::UrlClicked(url) => {
                let _ = webbrowser::open(url.as_str());

                Task::none()
            }
            Message::TitleChanged(new_title) => {
                let Self::Reviewing { state, .. } = self else {
                    return Task::none();
                };

                let State::Loaded { title, .. } = state else {
                    return Task::none();
                };

                *title = new_title;

                Task::none()
            }
            Message::CategorySelected(new_category) => {
                let Self::Reviewing { state, .. } = self else {
                    return Task::none();
                };

                let State::Loaded { category, .. } = state else {
                    return Task::none();
                };

                *category = new_category;

                Task::none()
            }
            Message::Next => {
                let Self::Reviewing {
                    changelog,
                    pending,
                    state,
                    preview,
                    ..
                } = self
                else {
                    return Task::none();
                };

                let State::Loaded {
                    title,
                    category,
                    pull_request,
                    ..
                } = state
                else {
                    return Task::none();
                };

                if let Some(entry) =
                    changelog::Entry::new(title, *category, pull_request)
                {
                    changelog.push(entry);

                    let save = Task::perform(
                        changelog.clone().save(),
                        Message::ChangelogSaved,
                    );

                    *preview =
                        markdown::parse(&changelog.to_string()).collect();

                    if let Some(contribution) = pending.pop() {
                        *state = State::Loading(contribution.clone());

                        Task::batch([
                            save,
                            Task::perform(
                                changelog::PullRequest::fetch(contribution),
                                Message::PullRequestFetched,
                            ),
                        ])
                    } else {
                        *self = Self::Done;
                        save
                    }
                } else {
                    Task::none()
                }
            }
            Message::OpenPullRequest(id) => {
                let _ = webbrowser::open(&format!(
                    "https://github.com/iced-rs/iced/pull/{id}"
                ));

                Task::none()
            }
            Message::ChangelogSaved(Ok(())) => Task::none(),

            Message::ChangelogListed(Err(error))
            | Message::PullRequestFetched(Err(error))
            | Message::ChangelogSaved(Err(error)) => {
                log::error!("{error}");

                Task::none()
            }
            Message::Quit => iced::exit(),
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Loading => center("Loading...").into(),
            Self::Done => center(
                column![
                    text("Changelog is up-to-date! 🎉")
                        .shaping(text::Shaping::Advanced),
                    button("Quit").on_press(Message::Quit),
                ]
                .spacing(10)
                .align_x(Center),
            )
            .into(),
            Self::Reviewing {
                changelog,
                pending,
                state,
                preview,
            } => {
                let progress = {
                    let total = pending.len() + changelog.len();

                    let bar = progress_bar(
                        0.0..=1.0,
                        changelog.len() as f32 / total as f32,
                    )
                    .style(progress_bar::secondary);

                    let label = text!(
                        "{amount_reviewed} / {total}",
                        amount_reviewed = changelog.len()
                    )
                    .font(Font::MONOSPACE)
                    .size(12);

                    stack![bar, center(label)]
                };

                let form: Element<_> = match state {
                    State::Loading(contribution) => {
                        text!("Loading #{}...", contribution.id).into()
                    }
                    State::Loaded {
                        pull_request,
                        description,
                        title,
                        category,
                    } => {
                        let details = {
                            let title = rich_text![
                                span(&pull_request.title).size(24).link(
                                    Message::OpenPullRequest(pull_request.id)
                                ),
                                span(format!(" by {}", pull_request.author))
                                    .font(Font {
                                        style: font::Style::Italic,
                                        ..Font::default()
                                    }),
                            ]
                            .font(Font::MONOSPACE);

                            let description = markdown::view(
                                description,
                                markdown::Settings::default(),
                                markdown::Style::from_palette(
                                    self.theme().palette(),
                                ),
                            )
                            .map(Message::UrlClicked);

                            let labels =
                                row(pull_request.labels.iter().map(|label| {
                                    container(
                                        text(label)
                                            .size(10)
                                            .font(Font::MONOSPACE),
                                    )
                                    .padding(5)
                                    .style(container::rounded_box)
                                    .into()
                                }))
                                .spacing(10)
                                .wrap();

                            column![
                                title,
                                labels,
                                scrollable(description)
                                    .spacing(10)
                                    .width(Fill)
                                    .height(Fill)
                            ]
                            .spacing(10)
                        };

                        let title = text_input(
                            "Type a changelog entry title...",
                            title,
                        )
                        .on_input(Message::TitleChanged);

                        let category = pick_list(
                            changelog::Category::ALL,
                            Some(category),
                            Message::CategorySelected,
                        );

                        let next = button("Next →")
                            .on_press(Message::Next)
                            .style(button::success);

                        column![
                            details,
                            row![title, category, next].spacing(10)
                        ]
                        .spacing(10)
                        .into()
                    }
                };

                let preview = if preview.is_empty() {
                    center(
                        container(
                            text("The changelog is empty... so far!").size(12),
                        )
                        .padding(10)
                        .style(container::rounded_box),
                    )
                } else {
                    container(
                        scrollable(
                            markdown::view(
                                preview,
                                markdown::Settings::with_text_size(12),
                                markdown::Style::from_palette(
                                    self.theme().palette(),
                                ),
                            )
                            .map(Message::UrlClicked),
                        )
                        .spacing(10),
                    )
                    .width(Fill)
                    .padding(10)
                    .style(container::rounded_box)
                };

                let review = column![container(form).height(Fill), progress]
                    .spacing(10)
                    .width(FillPortion(2));

                row![review, preview].spacing(10).padding(10).into()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }
}
