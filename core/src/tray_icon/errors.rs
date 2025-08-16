//! Tray Icon creation errors

/// An error that occurred during Tray Icon creation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    ///Failed to create icon
    #[error("icon could not be parsed")]
    BadIcon(#[from] tray_icon::BadIcon),
    ///Failed to create the tray icon
    #[error("tray icon could not be created")]
    CreationError(#[from] tray_icon::Error),
    ///Failed to create the tray icon menu
    #[error("tray icon menu could not be created")]
    MenuError(#[from] tray_icon::menu::Error),
    ///Failed to create menu icon
    #[error("menu icon could not be parsed")]
    BadMenuIcon(#[from] tray_icon::menu::BadIcon),
    ///Failed to create menu icon
    #[error("accelerator could not be parsed")]
    BadAccelerator(#[from] tray_icon::menu::AcceleratorParseError),
}
