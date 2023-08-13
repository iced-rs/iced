/// Sets a specific theme for the window.
///
/// If `None` is provided, the window will use the system theme.
///
/// The default is `None`.
///
/// ## Platform-specific
///
/// - **macOS:** This is an app-wide setting.
/// - **Wayland:** This control only CSD. You can also use `WINIT_WAYLAND_CSD_THEME` env variable to set the theme.
///   Possible values for env variable are: "dark" and light".
/// - **x11:** Build window with `_GTK_THEME_VARIANT` hint set to `dark` or `light`.
/// - **iOS / Android / Web / x11 / Orbital:** Ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowTheme {
    /// Use the light variant.
    Light,

    /// Use the dark variant.
    Dark,
}
