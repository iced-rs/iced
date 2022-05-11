/// An error that occurred while creating an application's graphical context.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested backend version is not supported.
    #[error("the requested backend version is not supported")]
    VersionNotSupported,

    /// Failed to find any pixel format that matches the criteria.
    #[error("failed to find any pixel format that matches the criteria")]
    NoAvailablePixelFormat,

    /// A suitable graphics adapter or device could not be found.
    #[error("a suitable graphics adapter or device could not be found")]
    GraphicsAdapterNotFound,

    /// An error occured in the context's internal backend
    #[error("an error occured in the context's internal backend")]
    BackendError(String),
}
