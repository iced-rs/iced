use crate::futures;
use crate::graphics;
use crate::shell;

use crate::core::tray_icon;

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The futures executor could not be created.
    #[error("the futures executor could not be created")]
    ExecutorCreationFailed(futures::io::Error),

    /// The application window could not be created.
    #[error("the application window could not be created")]
    WindowCreationFailed(Box<dyn std::error::Error + Send + Sync>),

    /// The application graphics context could not be created.
    #[error("the application graphics context could not be created")]
    GraphicsCreationFailed(graphics::Error),

    /// The application tray icon could not be created
    #[error("the application tray icon could not be created")]
    TrayIconCreationFailed(tray_icon::Error),
}

impl From<shell::Error> for Error {
    fn from(error: shell::Error) -> Error {
        match error {
            shell::Error::ExecutorCreationFailed(error) => {
                Error::ExecutorCreationFailed(error)
            }
            shell::Error::WindowCreationFailed(error) => {
                Error::WindowCreationFailed(Box::new(error))
            }
            shell::Error::GraphicsCreationFailed(error) => {
                Error::GraphicsCreationFailed(error)
            }
            shell::Error::TrayIconCreationFailed(error) => {
                Error::TrayIconCreationFailed(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_send_sync() {
        fn _assert<T: Send + Sync>() {}
        _assert::<Error>();
    }
}
