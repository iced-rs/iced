mod changelog;
mod icon;

use crate::changelog::Changelog;

use iced::clipboard;
use iced::font;
use iced::widget::{
    button, center, column, container, markdown, pick_list, progress_bar,
    rich_text, row, scrollable, span, stack, text, text_input,
};
use iced::{Element, Fill, FillPortion, Font, Task, Theme};

pub fn main() -> iced::Result {
    iced::application("Changelog Generator", Generator::update, Generator::view)
        .font(icon::FONT_BYTES)
        .theme(Generator::theme)
        .run_with(Generator::new)
}

enum Generator {
    Loading,
    Empty,
    Reviewing {
        changelog: Changelog,
        pending: Vec<changelog::Candidate>,
        state: State,
        preview: Vec<markdown::Item>,
    },
}

enum State {
    Loading(changelog::Candidate),
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
        Result<(Changelog, Vec<changelog::Candidate>), changelog::Error>,
    ),
    PullRequestFetched(Result<changelog::PullRequest, changelog::Error>),
    UrlClicked(markdown::Url),
    TitleChanged(String),
    CategorySelected(changelog::Category),
    Next,
    OpenPullRequest(u64),
    CopyPreview,
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
                if let Some(candidate) = pending.pop() {
                    let preview =
                        markdown::parse(&changelog.to_string()).collect();

                    *self = Self::Reviewing {
                        changelog,
                        pending,
                        state: State::Loading(candidate.clone()),
                        preview,
                    };

                    Task::perform(
                        candidate.fetch(),
                        Message::PullRequestFetched,
                    )
                } else {
                    *self = Self::Empty;

                    Task::none()
                }
            }
            Message::PullRequestFetched(Ok(pull_request)) => {
                let Self::Reviewing { state, .. } = self else {
                    return Task::none();
                };

                let description =
                    markdown::parse(&pull_request.description).collect();

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
            Message::ChangelogListed(Err(error))
            | Message::PullRequestFetched(Err(error)) => {
                log::error!("{error}");

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

                    *preview =
                        markdown::parse(&changelog.to_string()).collect();

                    if let Some(candidate) = pending.pop() {
                        *state = State::Loading(candidate.clone());

                        Task::perform(
                            candidate.fetch(),
                            Message::PullRequestFetched,
                        )
                    } else {
                        // TODO: We are done!
                        Task::none()
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
            Message::CopyPreview => {
                let Self::Reviewing { changelog, .. } = self else {
                    return Task::none();
                };

                clipboard::write(changelog.to_string())
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Loading => center("Loading...").into(),
            Self::Empty => center("No changes found!").into(),
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
                    State::Loading(candidate) => {
                        text!("Loading #{}...", candidate.id).into()
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

                let preview: Element<_> = if preview.is_empty() {
                    center(
                        container(
                            text("The changelog is empty... so far!").size(12),
                        )
                        .padding(10)
                        .style(container::rounded_box),
                    )
                    .into()
                } else {
                    let content = container(
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
                    .style(container::rounded_box);

                    let copy = button(icon::copy().size(12))
                        .on_press(Message::CopyPreview)
                        .style(button::text);

                    center(stack![content, container(copy).align_right(Fill)])
                        .into()
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
