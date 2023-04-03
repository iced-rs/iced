//! Configure a renderer.
use std::fmt;

pub use crate::Antialiasing;

/// The settings of a [`Backend`].
///
/// [`Backend`]: crate::Backend
#[derive(Clone, Copy, PartialEq)]
pub struct Settings {
    /// The present mode of the [`Backend`].
    ///
    /// [`Backend`]: crate::Backend
    pub present_mode: wgpu::PresentMode,

    /// The internal graphics backend to use.
    pub internal_backend: wgpu::Backends,

    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    pub default_font: Option<&'static [u8]>,

    /// The default size of text.
    ///
    /// By default, it will be set to `16.0`.
    pub default_text_size: f32,

    /// If enabled, spread text workload in multiple threads when multiple cores
    /// are available.
    ///
    /// By default, it is disabled.
    pub text_multithreading: bool,

    /// The antialiasing strategy that will be used for triangle primitives.
    ///
    /// By default, it is `None`.
    pub antialiasing: Option<Antialiasing>,
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("present_mode", &self.present_mode)
            .field("internal_backend", &self.internal_backend)
            // Instead of printing the font bytes, we simply show a `bool` indicating if using a default font or not.
            .field("default_font", &self.default_font.is_some())
            .field("default_text_size", &self.default_text_size)
            .field("text_multithreading", &self.text_multithreading)
            .field("antialiasing", &self.antialiasing)
            .finish()
    }
}

impl Settings {
    /// Creates new [`Settings`] using environment configuration.
    ///
    /// Specifically:
    ///
    /// - The `internal_backend` can be configured using the `WGPU_BACKEND`
    /// environment variable. If the variable is not set, the primary backend
    /// will be used. The following values are allowed:
    ///     - `vulkan`
    ///     - `metal`
    ///     - `dx12`
    ///     - `dx11`
    ///     - `gl`
    ///     - `webgpu`
    ///     - `primary`
    pub fn from_env() -> Self {
        Settings {
            internal_backend: backend_from_env()
                .unwrap_or(wgpu::Backends::all()),
            ..Self::default()
        }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            present_mode: wgpu::PresentMode::AutoVsync,
            internal_backend: wgpu::Backends::all(),
            default_font: None,
            default_text_size: 20.0,
            text_multithreading: false,
            antialiasing: None,
        }
    }
}

fn backend_from_env() -> Option<wgpu::Backends> {
    std::env::var("WGPU_BACKEND").ok().map(|backend| {
        match backend.to_lowercase().as_str() {
            "vulkan" => wgpu::Backends::VULKAN,
            "metal" => wgpu::Backends::METAL,
            "dx12" => wgpu::Backends::DX12,
            "dx11" => wgpu::Backends::DX11,
            "gl" => wgpu::Backends::GL,
            "webgpu" => wgpu::Backends::BROWSER_WEBGPU,
            "primary" => wgpu::Backends::PRIMARY,
            other => panic!("Unknown backend: {other}"),
        }
    })
}
