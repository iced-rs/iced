use crate::executor;
use crate::runtime::Task;

use std::process;

pub const COMPATIBLE_REVISION: &str =
    "20f9c9a897fecac5dce0977bbb5639fdce1f54b9";

pub fn launch() -> Task<launch::Result> {
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
                return Err(launch::Error::Outdated { revision: None });
            };

            let Some((revision, _)) = revision.rsplit_once("#") else {
                return Err(launch::Error::Outdated { revision: None });
            };

            if revision != COMPATIBLE_REVISION {
                return Err(launch::Error::Outdated {
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

        Err(launch::Error::NotFound)
    })
}

pub fn install() -> Task<install::Result> {
    executor::try_spawn_blocking(|mut sender| {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let mut install = Command::new("cargo")
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

        let mut stderr = BufReader::new(
            install.stderr.take().expect("stderr must be piped"),
        );

        let mut log = String::new();

        while let Ok(n) = stderr.read_line(&mut log) {
            if n == 0 {
                let status = install.wait()?;

                if status.success() {
                    break;
                } else {
                    return Err(install::Error::ProcessFailed(status));
                }
            }

            let _ = sender.try_send(install::Event::Logged(log.clone()));
            log.clear();
        }

        let _ = sender.try_send(install::Event::Finished);

        Ok(())
    })
}

pub mod launch {
    use std::io;
    use std::sync::Arc;

    pub type Result = std::result::Result<(), Error>;

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
}

pub mod install {
    use std::io;
    use std::process;
    use std::sync::Arc;

    pub type Result = std::result::Result<Event, Error>;

    #[derive(Debug, Clone)]
    pub enum Event {
        Logged(String),
        Finished,
    }

    #[derive(Debug, Clone)]
    pub enum Error {
        ProcessFailed(process::ExitStatus),
        IoFailed(Arc<io::Error>),
    }

    impl From<io::Error> for Error {
        fn from(error: io::Error) -> Self {
            Self::IoFailed(Arc::new(error))
        }
    }
}
