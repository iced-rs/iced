//! Platform specific settings for unix-like systems (except macOS).

#[cfg(feature = "x11")]
use winit::platform::x11::XWindowType;

/// The platform specific window settings of an application.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlatformSpecific {
    /// Build window with `_NET_WM_WINDOW_TYPE` hints; defaults to `Normal`. Only relevant on X11.
    #[cfg(feature = "x11")]
    pub x11_window_type: Vec<XWindowType>,
}
