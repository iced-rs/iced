use serde::Deserialize;
use tokio::fs;
use tokio::process;

use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::io;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Changelog {
    ids: Vec<u64>,
    added: Vec<String>,
    changed: Vec<String>,
    fixed: Vec<String>,
    removed: Vec<String>,
    authors: Vec<String>,
}

impl Changelog {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            added: Vec::new(),
            changed: Vec::new(),
            fixed: Vec::new(),
            removed: Vec::new(),
            authors: Vec::new(),
        }
    }

    pub async fn list() -> Result<(Self, Vec<Contribution>), Error> {
        let mut changelog = Self::new();

        {
            let markdown = fs::read_to_string("CHANGELOG.md").await?;

            if let Some(unreleased) = markdown.split("\n## ").nth(1) {
                let sections = unreleased.split("\n\n");

                for section in sections {
                    if section.starts_with("Many thanks to...") {
                        for author in section.lines().skip(1) {
                            let author = author.trim_start_matches("- @");

                            if author.is_empty() {
                                continue;
                            }

                            changelog.authors.push(author.to_owned());
                        }

                        continue;
                    }

                    let Some((_, rest)) = section.split_once("### ") else {
                        continue;
                    };

                    let Some((name, rest)) = rest.split_once("\n") else {
                        continue;
                    };

                    let category = match name {
                        "Added" => Category::Added,
                        "Fixed" => Category::Fixed,
                        "Changed" => Category::Changed,
                        "Removed" => Category::Removed,
                        _ => continue,
                    };

                    for entry in rest.lines() {
                        let Some((_, id)) = entry.split_once("[#") else {
                            continue;
                        };

                        let Some((id, _)) = id.split_once(']') else {
                            continue;
                        };

                        let Ok(id): Result<u64, _> = id.parse() else {
                            continue;
                        };

                        changelog.ids.push(id);

                        let target = match category {
                            Category::Added => &mut changelog.added,
                            Category::Changed => &mut changelog.changed,
                            Category::Fixed => &mut changelog.fixed,
                            Category::Removed => &mut changelog.removed,
                        };

                        target.push(entry.to_owned());
                    }
                }
            }
        }

        let mut candidates = Contribution::list().await?;

        for reviewed_entry in changelog.entries() {
            candidates.retain(|candidate| candidate.id != reviewed_entry);
        }

        Ok((changelog, candidates))
    }

    pub async fn save(self) -> Result<(), Error> {
        let markdown = fs::read_to_string("CHANGELOG.md").await?;

        let Some((header, rest)) = markdown.split_once("\n## ") else {
            return Err(Error::InvalidFormat);
        };

        let Some((_unreleased, rest)) = rest.split_once("\n## ") else {
            return Err(Error::InvalidFormat);
        };

        let unreleased = format!("\n## [Unreleased]\n{self}");

        let rest = format!("\n## {rest}");

        let changelog = [header, &unreleased, &rest].concat();
        fs::write("CHANGELOG.md", changelog).await?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn entries(&self) -> impl Iterator<Item = u64> + '_ {
        self.ids.iter().copied()
    }

    pub fn push(&mut self, entry: Entry) {
        self.ids.push(entry.id);

        let item = format!(
            "- {title}. [#{id}](https://github.com/iced-rs/iced/pull/{id})",
            title = entry.title,
            id = entry.id
        );

        let target = match entry.category {
            Category::Added => &mut self.added,
            Category::Changed => &mut self.changed,
            Category::Fixed => &mut self.fixed,
            Category::Removed => &mut self.removed,
        };

        target.push(item);

        if entry.author != "hecrj" && !self.authors.contains(&entry.author) {
            self.authors.push(entry.author);
            self.authors.sort_by_key(|author| author.to_lowercase());
        }
    }
}

impl fmt::Display for Changelog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn section(category: Category, entries: &[String]) -> String {
            if entries.is_empty() {
                return String::new();
            }

            format!("### {category}\n{list}\n", list = entries.join("\n"))
        }

        fn thank_you<'a>(authors: impl IntoIterator<Item = &'a str>) -> String {
            let mut list = String::new();

            for author in authors {
                list.push_str(&format!("- @{author}\n"));
            }

            format!("Many thanks to...\n{list}")
        }

        let changelog = [
            section(Category::Added, &self.added),
            section(Category::Changed, &self.changed),
            section(Category::Fixed, &self.fixed),
            section(Category::Removed, &self.removed),
            thank_you(self.authors.iter().map(String::as_str)),
        ]
        .into_iter()
        .filter(|section| !section.is_empty())
        .collect::<Vec<String>>()
        .join("\n");

        f.write_str(&changelog)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: u64,
    pub title: String,
    pub category: Category,
    pub author: String,
}

impl Entry {
    pub fn new(
        title: &str,
        category: Category,
        pull_request: &PullRequest,
    ) -> Option<Self> {
        let title = title.strip_suffix(".").unwrap_or(title);

        if title.is_empty() {
            return None;
        };

        Some(Self {
            id: pull_request.id,
            title: title.to_owned(),
            category,
            author: pull_request.author.clone(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Added,
    Changed,
    Fixed,
    Removed,
}

impl Category {
    pub const ALL: &'static [Self] =
        &[Self::Added, Self::Changed, Self::Fixed, Self::Removed];

    pub fn guess(label: &str) -> Option<Self> {
        Some(match label {
            "feature" | "addition" => Self::Added,
            "change" => Self::Changed,
            "bug" | "fix" => Self::Fixed,
            _ => None?,
        })
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Category::Added => "Added",
            Category::Changed => "Changed",
            Category::Fixed => "Fixed",
            Category::Removed => "Removed",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Contribution {
    pub id: u64,
}

impl Contribution {
    pub async fn list() -> Result<Vec<Contribution>, Error> {
        let output = process::Command::new("git")
            .args([
                "log",
                "--oneline",
                "--grep",
                "#[0-9]*",
                "origin/latest..HEAD",
            ])
            .output()
            .await?;

        let log = String::from_utf8_lossy(&output.stdout);

        let mut contributions: Vec<_> = log
            .lines()
            .filter(|title| !title.is_empty())
            .filter_map(|title| {
                let (_, pull_request) = title.split_once("#")?;
                let (pull_request, _) = pull_request.split_once([')', ' '])?;

                Some(Contribution {
                    id: pull_request.parse().ok()?,
                })
            })
            .collect();

        let mut unique = BTreeSet::from_iter(contributions.clone());
        contributions.retain_mut(|contribution| unique.remove(contribution));

        Ok(contributions)
    }
}

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub labels: Vec<String>,
    pub author: String,
}

impl PullRequest {
    pub async fn fetch(contribution: Contribution) -> Result<Self, Error> {
        let request = reqwest::Client::new()
            .request(
                reqwest::Method::GET,
                format!(
                    "https://api.github.com/repos/iced-rs/iced/pulls/{}",
                    contribution.id
                ),
            )
            .header("User-Agent", "iced changelog generator")
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    env::var("GITHUB_TOKEN")
                        .map_err(|_| Error::GitHubTokenNotFound)?
                ),
            );

        #[derive(Deserialize)]
        struct Schema {
            title: String,
            body: Option<String>,
            user: User,
            labels: Vec<Label>,
        }

        #[derive(Deserialize)]
        struct User {
            login: String,
        }

        #[derive(Deserialize)]
        struct Label {
            name: String,
        }

        let schema: Schema = request.send().await?.json().await?;

        Ok(Self {
            id: contribution.id,
            title: schema.title,
            description: schema.body,
            labels: schema.labels.into_iter().map(|label| label.name).collect(),
            author: schema.user.login,
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("io operation failed: {0}")]
    IOFailed(Arc<io::Error>),

    #[error("http request failed: {0}")]
    RequestFailed(Arc<reqwest::Error>),

    #[error("no GITHUB_TOKEN variable was set")]
    GitHubTokenNotFound,

    #[error("the changelog format is not valid")]
    InvalidFormat,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOFailed(Arc::new(error))
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RequestFailed(Arc::new(error))
    }
}
