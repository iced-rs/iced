//! Configure a renderer.
pub use crate::Antialiasing;

/// The settings of a [`Backend`].
///
/// [`Backend`]: crate::Backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The output format of the [`Backend`].
    ///
    /// [`Backend`]: crate::Backend
    pub format: wgpu::TextureFormat,

    /// The present mode of the [`Backend`].
    ///
    /// [`Backend`]: crate::Backend
    pub present_mode: wgpu::PresentMode,

    /// The internal graphics backend to use.
    pub internal_backend: wgpu::BackendBit,

    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    pub default_font: Option<&'static [u8]>,

    /// The default size of text.
    ///
    /// By default, it will be set to 20.
    pub default_text_size: u16,

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
                .unwrap_or(wgpu::BackendBit::PRIMARY),
            ..Self::default()
        }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Mailbox,
            internal_backend: wgpu::BackendBit::PRIMARY,
            default_font: None,
            default_text_size: 20,
            text_multithreading: false,
            antialiasing: None,
        }
    }
}

fn backend_from_env() -> Option<wgpu::BackendBit> {
    std::env::var("WGPU_BACKEND").ok().map(|backend| {
        match backend.to_lowercase().as_str() {
            "vulkan" => wgpu::BackendBit::VULKAN,
            "metal" => wgpu::BackendBit::METAL,
            "dx12" => wgpu::BackendBit::DX12,
            "dx11" => wgpu::BackendBit::DX11,
            "gl" => wgpu::BackendBit::GL,
            "webgpu" => wgpu::BackendBit::BROWSER_WEBGPU,
            "primary" => wgpu::BackendBit::PRIMARY,
            other => panic!("Unknown backend: {}", other),
        }
    })
}
