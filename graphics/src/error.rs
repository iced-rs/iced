/// A graphical error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A suitable graphics adapter or device could not be found
    #[error("a suitable graphics adapter or device could not be found")]
    AdapterNotFound,
}
