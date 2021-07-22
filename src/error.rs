use iced_futures::futures;

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The futures executor could not be created.
    #[error("the futures executor could not be created")]
    ExecutorCreationFailed(futures::io::Error),

    /// The application window could not be created.
    #[error("the application window could not be created")]
    WindowCreationFailed(Box<dyn std::error::Error + Send + Sync>),

    /// A suitable graphics adapter or device could not be found.
    #[error("a suitable graphics adapter or device could not be found")]
    GraphicsAdapterNotFound,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<iced_winit::Error> for Error {
    fn from(error: iced_winit::Error) -> Error {
        match error {
            iced_winit::Error::ExecutorCreationFailed(error) => {
                Error::ExecutorCreationFailed(error)
            }
            iced_winit::Error::WindowCreationFailed(error) => {
                Error::WindowCreationFailed(Box::new(error))
            }
            iced_winit::Error::GraphicsAdapterNotFound => {
                Error::GraphicsAdapterNotFound
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
