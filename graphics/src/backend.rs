//! Graphical backends are designed to aid in rendering computer graphics to a monitor.
use std::env;
use std::fmt;

/// A graphical backend.
///
/// You can override the default strategy by setting the `ICED_BACKEND` environment variable.
/// If you are using the default renderer, the available options are:
///   - `wgpu` for the hardware accelerated backend.
///   - `tiny-skia` for the software-based backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// Auto-detect and choose the best available [`Backend`].
    Best,
    /// Hardware accelerated graphics backend with the given [`API`].
    Hardware(API),
    /// Sofware graphics backend; quite slower than hardware-based backends, but more compatible.
    Software,
    /// A custom rendering backend with the given name.
    Custom(String),
}

impl Backend {
    /// All the possible known combinations of [`Backend`].
    pub const ALL: &[Self] = &[
        Self::Best,
        Self::Hardware(API::Best),
        Self::Hardware(API::Vulkan),
        Self::Hardware(API::Metal),
        Self::Hardware(API::DirectX12),
        Self::Hardware(API::OpenGL),
        Self::Hardware(API::WebGPU),
        Self::Software,
    ];

    /// Returns true if the [`Backend`] is [`Backend::Best`] or matches the given name.
    pub fn matches(&self, target: &str) -> bool {
        match self {
            Backend::Best => true,
            Backend::Custom(name) => name == target,
            _ => false,
        }
    }
}

impl From<String> for Backend {
    fn from(backend: String) -> Self {
        Self::Custom(backend)
    }
}

impl From<&str> for Backend {
    fn from(backend: &str) -> Self {
        Self::Custom(backend.to_owned())
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Best => write!(f, "Best Backend"),
            Backend::Hardware(api) => write!(f, "Hardware Backend ({api})"),
            Backend::Software => write!(f, "Software Backend"),
            Backend::Custom(name) => write!(f, "Custom Backend ({name})"),
        }
    }
}

/// A hardware graphics API.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum API {
    /// Auto-detect and choose the best available graphics [`API`].
    #[default]
    Best,
    /// Vulkan API (Windows, Linux, Android, MacOS via vulkan-portability/MoltenVK)
    Vulkan,
    /// Metal API (Apple platforms)
    Metal,
    /// Direct3D-12 (Windows)
    DirectX12,
    /// OpenGL 3.3+ (Windows), OpenGL ES 3.0+ (Linux, Android, MacOS via Angle), and WebGL2
    OpenGL,
    /// WebGPU (Web Browser)
    WebGPU,
}

impl Default for Backend {
    fn default() -> Self {
        let Ok(backend) = env::var("ICED_BACKEND") else {
            return Self::Best;
        };

        Self::Custom(backend.to_owned())
    }
}

impl fmt::Display for API {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            API::Best => "Best",
            API::Vulkan => "Vulkan",
            API::Metal => "Metal",
            API::DirectX12 => "DirectX 12",
            API::OpenGL => "OpenGL",
            API::WebGPU => "WebGPU",
        })
    }
}
