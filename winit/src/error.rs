use iced_futures::futures;

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The futures executor could not be created.
    #[error("the futures executor could not be created")]
    ExecutorCreationFailed(futures::io::Error),

    /// The application window could not be created.
    #[error("the application window could not be created")]
    WindowCreationFailed(winit::error::OsError),

    /// A suitable graphics adapter or device could not be found.
    #[error("a suitable graphics adapter or device could not be found")]
    GraphicsAdapterNotFound,
}

impl From<iced_graphics::Error> for Error {
    fn from(error: iced_graphics::Error) -> Error {
        match error {
            iced_graphics::Error::AdapterNotFound => {
                Error::GraphicsAdapterNotFound
            }
        }
    }
}
