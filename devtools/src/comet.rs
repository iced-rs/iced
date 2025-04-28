use crate::executor;
use crate::runtime::Task;

use std::io;
use std::process;
use std::sync::Arc;

pub const COMPATIBLE_REVISION: &str =
    "69dd2283886dccdaa1ee6e1c274af62f7250bc38";

pub fn launch() -> Task<Result<(), Error>> {
    executor::try_spawn_blocking(|mut sender| {
        let cargo_install = process::Command::new("cargo")
            .args(["install", "--list"])
            .output()?;

        let installed_packages = String::from_utf8_lossy(&cargo_install.stdout);

        for line in installed_packages.lines() {
            if !line.starts_with("iced_comet ") {
                continue;
            }

            let Some((_, revision)) = line.rsplit_once("?rev=") else {
                return Err(Error::Outdated { revision: None });
            };

            let Some((revision, _)) = revision.rsplit_once("#") else {
                return Err(Error::Outdated { revision: None });
            };

            if revision != COMPATIBLE_REVISION {
                return Err(Error::Outdated {
                    revision: Some(revision.to_owned()),
                });
            }

            let _ = process::Command::new("iced_comet")
                .stdin(process::Stdio::null())
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .spawn()?;

            let _ = sender.try_send(());
            return Ok(());
        }

        Err(Error::NotFound)
    })
}

pub fn install() -> Task<Result<Installation, Error>> {
    executor::try_spawn_blocking(|mut sender| {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let install = Command::new("cargo")
            .args([
                "install",
                "--locked",
                "--git",
                "https://github.com/iced-rs/comet.git",
                "--rev",
                COMPATIBLE_REVISION,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stderr =
            BufReader::new(install.stderr.expect("stderr must be piped"));

        let mut log = String::new();

        while let Ok(n) = stderr.read_line(&mut log) {
            if n == 0 {
                break;
            }

            let _ = sender.try_send(Installation::Logged(log.clone()));
            log.clear();
        }

        let _ = sender.try_send(Installation::Finished);

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum Installation {
    Logged(String),
    Finished,
}

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    Outdated { revision: Option<String> },
    IoFailed(Arc<io::Error>),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IoFailed(Arc::new(error))
    }
}
