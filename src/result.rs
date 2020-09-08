use crate::Error;

/// The result of running an [`Application`].
///
/// [`Application`]: trait.Application.html
pub type Result = std::result::Result<(), Error>;
