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
    /// Hardware accelerated graphics backend with the given [`Api`].
    Hardware(Api),
    /// Sofware graphics backend; quite slower than hardware-based backends, but more compatible.
    Software,
    /// A custom rendering backend with the given name.
    Custom(String),
}

impl Backend {
    /// All the possible known combinations of [`Backend`].
    pub const ALL: &[Self] = &[
        Self::Best,
        Self::Hardware(Api::Best),
        Self::Hardware(Api::Vulkan),
        Self::Hardware(Api::Metal),
        Self::Hardware(Api::DirectX12),
        Self::Hardware(Api::OpenGL),
        Self::Hardware(Api::WebGPU),
        Self::Software,
    ];

    /// Returns true if the [`Backend`] is [`Backend::Hardware`].
    pub fn hardware(&self) -> Option<Api> {
        match self {
            Backend::Hardware(api) => Some(*api),
            _ => None,
        }
    }

    /// Returns true if the [`Backend`] is [`Backend::Software`].
    pub fn is_software(&self) -> bool {
        matches!(self, Self::Software)
    }

    /// Returns true if the [`Backend`] is [`Backend::Best`] or matches the given name.
    pub fn matches(&self, target: &str) -> bool {
        match self {
            Backend::Best => true,
            Backend::Custom(name) => name == target || name == &target.replace("-", "_"),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Api {
    /// Auto-detect and choose the best available graphics [`Api`].
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

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Api::Best => "Best",
            Api::Vulkan => "Vulkan",
            Api::Metal => "Metal",
            Api::DirectX12 => "DirectX 12",
            Api::OpenGL => "OpenGL",
            Api::WebGPU => "WebGPU",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Default)]
/// The power-usage preference for graphics adapters.
pub enum PowerPreference {
    /// No power preference in which adapter should be chosen.
    ///
    /// This is the default.
    #[default]
    None,

    /// The backend will prefer low power adapters over high performance ones.
    LowPower,

    /// The backend will prefer adapters that are high performance.
    HighPerformance,
}

/// The settings usted to configure a [`Backend`].
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The graphical backend to use.
    ///
    /// By default, it is [`Backend::Best`].
    pub backend: Backend,

    /// The [`PowerPreference`] of the backend.
    ///
    /// By default, it is [`PowerPreference::None`].
    pub power_preference: PowerPreference,

    /// If set to true, the renderer will try to perform antialiasing for some
    /// primitives.
    ///
    /// Enabling it can produce a smoother result in some widgets, like the
    /// `Canvas`, at a performance cost.
    ///
    /// By default, it is `true`.
    pub antialiasing: bool,

    /// Whether or not to synchronize frames.
    ///
    /// By default, it is `true`.
    pub vsync: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            backend: Backend::Best,
            antialiasing: true,
            vsync: true,
            power_preference: PowerPreference::None,
        }
    }
}

impl From<&crate::Settings> for Settings {
    fn from(settings: &crate::Settings) -> Self {
        Self {
            backend: settings.backend.clone(),
            antialiasing: settings.antialiasing,
            vsync: settings.vsync,
            power_preference: settings.power_preference,
        }
    }
}

/// An error that occurred while creating an application's graphical context.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The requested backend version is not supported.
    #[error("the requested backend version is not supported")]
    VersionNotSupported,

    /// Failed to find any pixel format that matches the criteria.
    #[error("failed to find any pixel format that matches the criteria")]
    NoAvailablePixelFormat,

    /// A suitable graphics adapter or device could not be found.
    #[error("a suitable graphics adapter could not be found: {reason}")]
    GraphicsAdapterNotFound {
        /// The name of the backend where the error happened
        backend: &'static str,
        /// The reason why this backend could not be used
        reason: Reason,
    },

    /// An error occurred in the context's internal backend
    #[error("an error occurred in the context's internal backend")]
    BackendError(String),

    /// Multiple errors occurred
    #[error("multiple errors occurred:\n{}", error_list(.0))]
    List(Vec<Self>),
}

/// The reason why a graphics adapter could not be found
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// The backend did not match the preference
    DidNotMatch {
        /// The preferred backend
        preferred_backend: Backend,
    },
    /// The request to create the backend failed
    RequestFailed(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reason::DidNotMatch { preferred_backend } => {
                write!(
                    f,
                    "the backend did not match the preference: {preferred_backend}"
                )
            }
            Reason::RequestFailed(error) => f.write_str(error),
        }
    }
}

fn error_list(errors: &Vec<Error>) -> String {
    let mut list = String::new();

    for error in errors {
        list.push_str("- ");
        list.push_str(&error.to_string());
        list.push('\n');
    }

    list
}
