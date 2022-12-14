use crate::error::Error;

/// The result of running an [`Application`].
///
/// [`Application`]: crate::Application
pub type Result = std::result::Result<(), Error>;
