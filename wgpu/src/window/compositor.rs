//! Connect a window with a renderer.
use crate::core::Color;
use crate::graphics::color;
use crate::graphics::compositor;
use crate::graphics::error;
use crate::graphics::{self, Shell, Viewport};
use crate::settings::{self, Settings};
use crate::{Engine, Renderer};

/// A window graphics backend for iced powered by `wgpu`.
pub struct Compositor {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    format: wgpu::TextureFormat,
    alpha_mode: wgpu::CompositeAlphaMode,
    engine: Engine,
    settings: Settings,
}

/// A compositor error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// The surface creation failed.
    #[error("the surface creation failed: {0}")]
    SurfaceCreationFailed(#[from] wgpu::CreateSurfaceError),
    /// The surface is not compatible.
    #[error("the surface is not compatible")]
    IncompatibleSurface,
    /// No adapter was found for the options requested.
    #[error("no adapter was found for the options requested: {0:?}")]
    NoAdapterFound(String),
    /// No device request succeeded.
    #[error("no device request succeeded: {0:?}")]
    RequestDeviceFailed(Vec<(wgpu::Limits, wgpu::RequestDeviceError)>),
}

impl From<Error> for graphics::Error {
    fn from(error: Error) -> Self {
        Self::GraphicsAdapterNotFound {
            backend: "wgpu",
            reason: error::Reason::RequestFailed(error.to_string()),
        }
    }
}

impl Compositor {
    /// Requests a new [`Compositor`] with the given [`Settings`].
    ///
    /// Returns `None` if no compatible graphics adapter could be found.
    pub async fn request<W: compositor::Window>(
        settings: Settings,
        compatible_window: Option<W>,
        shell: Shell,
    ) -> Result<Self, Error> {
        let instance = wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
            backends: settings.backends,
            flags: if cfg!(feature = "strict-assertions") {
                wgpu::InstanceFlags::debugging()
            } else {
                wgpu::InstanceFlags::empty()
            },
            ..Default::default()
        })
        .await;

        log::info!("{settings:#?}");

        #[cfg(not(target_arch = "wasm32"))]
        if log::max_level() >= log::LevelFilter::Info {
            let available_adapters: Vec<_> = instance
                .enumerate_adapters(settings.backends)
                .await
                .iter()
                .map(wgpu::Adapter::get_info)
                .collect();
            log::info!("Available adapters: {available_adapters:#?}");
        }

        #[allow(unsafe_code)]
        let compatible_surface =
            compatible_window.and_then(|window| instance.create_surface(window).ok());

        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::from_env()
                .unwrap_or(wgpu::PowerPreference::HighPerformance),
            compatible_surface: compatible_surface.as_ref(),
            force_fallback_adapter: false,
        };

        let adapter = instance
            .request_adapter(&adapter_options)
            .await
            .map_err(|_error| Error::NoAdapterFound(format!("{adapter_options:?}")))?;

        log::info!("Selected: {:#?}", adapter.get_info());

        let (format, alpha_mode) = compatible_surface
            .as_ref()
            .and_then(|surface| {
                let capabilities = surface.get_capabilities(&adapter);

                let formats = capabilities.formats.iter().copied();

                log::info!("Available formats: {formats:#?}");

                const BLACKLIST: &[wgpu::TextureFormat] = &[
                    wgpu::TextureFormat::Rgb10a2Unorm,
                    wgpu::TextureFormat::Rgb10a2Uint,
                ];

                let formats = formats.filter(|format| {
                    format.required_features() == wgpu::Features::empty()
                        && !BLACKLIST.contains(format)
                });

                let formats_vec: Vec<_> = formats.collect();

                log::info!("Filtered formats (no special features): {formats_vec:#?}");

                // TODO: Explore how to make this better on AMD
                // Prefer 8-bit sRGB formats over 10-bit HDR for better alpha blending compatibility
                // especially on AMD GPUs where Rgb10a2Unorm can cause visual artifacts
                let format = if color::GAMMA_CORRECTION {
                    // First try 8-bit sRGB formats explicitly
                    formats_vec
                        .iter()
                        .copied()
                        .find(|f| {
                            matches!(
                                f,
                                wgpu::TextureFormat::Bgra8UnormSrgb
                                    | wgpu::TextureFormat::Rgba8UnormSrgb
                            )
                        })
                        .or_else(|| {
                            // Fall back to any sRGB format
                            formats_vec
                                .iter()
                                .copied()
                                .find(wgpu::TextureFormat::is_srgb)
                        })
                } else {
                    // For non-gamma-corrected, prefer 8-bit over 10-bit
                    formats_vec
                        .iter()
                        .copied()
                        .find(|f| {
                            matches!(
                                f,
                                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                            )
                        })
                        .or_else(|| {
                            formats_vec
                                .iter()
                                .copied()
                                .find(wgpu::TextureFormat::is_srgb)
                        })
                };

                let format = format.or_else(|| {
                    log::warn!("No format found!");

                    capabilities.formats.first().copied()
                });

                let alpha_modes = capabilities.alpha_modes;

                log::info!("Available alpha modes: {alpha_modes:#?}");

                let preferred_alpha =
                    if alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
                        wgpu::CompositeAlphaMode::PreMultiplied
                    } else {
                        wgpu::CompositeAlphaMode::Auto
                    };

                format.zip(Some(preferred_alpha))
            })
            .ok_or(Error::IncompatibleSurface)?;

        log::info!("Selected format: {format:?} with alpha mode: {alpha_mode:?}");

        #[cfg(target_arch = "wasm32")]
        let limits = [wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())];

        #[cfg(not(target_arch = "wasm32"))]
        let limits = [wgpu::Limits::default(), wgpu::Limits::downlevel_defaults()];

        let limits = limits.into_iter().map(|limits| wgpu::Limits {
            max_bind_groups: 2,
            max_non_sampler_bindings: 2048,
            ..limits
        });

        // Request SHADER_F16 only if the adapter supports it (e.g., not available in WebGL2)
        let required_features = if adapter.features().contains(wgpu::Features::SHADER_F16) {
            wgpu::Features::SHADER_F16
        } else {
            wgpu::Features::empty()
        };

        let mut errors = Vec::new();

        for required_limits in limits {
            let result = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("iced_wgpu::window::compositor device descriptor"),
                    required_features,
                    required_limits: required_limits.clone(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    trace: wgpu::Trace::Off,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                })
                .await;

            match result {
                Ok((device, queue)) => {
                    let engine = Engine::new(
                        &adapter,
                        device,
                        queue,
                        format,
                        settings.antialiasing,
                        shell,
                    );

                    return Ok(Compositor {
                        instance,
                        adapter,
                        format,
                        alpha_mode,
                        engine,
                        settings,
                    });
                }
                Err(error) => {
                    errors.push((required_limits, error));
                }
            }
        }

        Err(Error::RequestDeviceFailed(errors))
    }
}

/// Creates a [`Compositor`] with the given [`Settings`] and window.
pub async fn new<W: compositor::Window>(
    settings: Settings,
    compatible_window: W,
    shell: Shell,
) -> Result<Compositor, Error> {
    Compositor::request(settings, Some(compatible_window), shell).await
}

/// Presents the given primitives with the given [`Compositor`].
pub fn present(
    renderer: &mut Renderer,
    surface: &mut wgpu::Surface<'static>,
    viewport: &Viewport,
    background_color: Color,
    on_pre_present: impl FnOnce(),
) -> Result<(), compositor::SurfaceError> {
    // Wrap the entire present flow in catch_unwind to handle Wayland
    // broken pipe panics (EPIPE / os error 32) that surface operations
    // can trigger during rapid maximize/unmaximize transitions.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match surface.get_current_texture() {
            Ok(frame) => {
                let view = &frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Use present_with_texture to enable backdrop blur effects
                let _submission = renderer.present_with_texture(
                    Some(background_color),
                    frame.texture.format(),
                    view,
                    &frame.texture,
                    viewport,
                );

                // Present the frame
                on_pre_present();
                frame.present();

                Ok(())
            }
            Err(error) => match error {
                wgpu::SurfaceError::Timeout => Err(compositor::SurfaceError::Timeout),
                wgpu::SurfaceError::Outdated => Err(compositor::SurfaceError::Outdated),
                wgpu::SurfaceError::Lost => Err(compositor::SurfaceError::Lost),
                wgpu::SurfaceError::OutOfMemory => Err(compositor::SurfaceError::OutOfMemory),
                wgpu::SurfaceError::Other => Err(compositor::SurfaceError::Other),
            },
        }
    }));

    match result {
        Ok(inner) => inner,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            log::error!("Present panicked (treating as surface lost): {msg}");
            Err(compositor::SurfaceError::Lost)
        }
    }
}

impl graphics::Compositor for Compositor {
    type Renderer = Renderer;
    type Surface = wgpu::Surface<'static>;

    async fn with_backend(
        settings: graphics::Settings,
        _display: impl compositor::Display,
        compatible_window: impl compositor::Window,
        shell: Shell,
        backend: Option<&str>,
    ) -> Result<Self, graphics::Error> {
        match backend {
            None | Some("wgpu") => {
                let mut settings = Settings::from(settings);

                if let Some(backends) = wgpu::Backends::from_env() {
                    settings.backends = backends;
                }

                if let Some(present_mode) = settings::present_mode_from_env() {
                    settings.present_mode = present_mode;
                }

                Ok(new(settings, compatible_window, shell).await?)
            }
            Some(backend) => Err(graphics::Error::GraphicsAdapterNotFound {
                backend: "wgpu",
                reason: error::Reason::DidNotMatch {
                    preferred_backend: backend.to_owned(),
                },
            }),
        }
    }

    fn create_renderer(&self) -> Self::Renderer {
        Renderer::new(
            self.engine.clone(),
            self.settings.default_font,
            self.settings.default_text_size,
        )
    }

    fn create_surface<W: compositor::Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface {
        let mut surface = match self.instance.create_surface(window.clone()) {
            Ok(s) => s,
            Err(e) => {
                // On Wayland, create_surface can fail with a broken pipe
                // during rapid maximize/unmaximize transitions.
                // Retry once after a brief yield, then panic if still failing.
                log::warn!("Surface creation failed: {e}. Retrying...");
                match self.instance.create_surface(window) {
                    Ok(s) => s,
                    Err(e2) => {
                        log::error!("Surface creation retry also failed: {e2}");
                        panic!("Cannot recover from surface creation failure: {e2}");
                    }
                }
            }
        };

        if width > 0 && height > 0 {
            self.configure_surface(&mut surface, width, height);
        }

        surface
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32) {
        // Always need RENDER_ATTACHMENT; request COPY_SRC for backdrop blur only if supported
        let mut usage = wgpu::TextureUsages::RENDER_ATTACHMENT;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let caps = surface.get_capabilities(&self.adapter);
            if caps.usages.contains(wgpu::TextureUsages::COPY_SRC) {
                usage |= wgpu::TextureUsages::COPY_SRC;
            } else {
                log::warn!("Surface does not support COPY_SRC; backdrop blur will be unavailable");
            }
        }

        let config = wgpu::SurfaceConfiguration {
            usage,
            format: self.format,
            present_mode: self.settings.present_mode,
            width,
            height,
            alpha_mode: self.alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        // Wrap surface.configure in catch_unwind to handle Wayland broken pipe
        // errors (EPIPE / os error 32) that can occur during rapid window
        // resize or maximize/unmaximize transitions.
        let device = &self.engine.device;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            surface.configure(device, &config);
        }));

        if let Err(e) = result {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            log::error!("Surface configure failed (will retry next frame): {msg}");
        }
    }

    fn information(&self) -> compositor::Information {
        let information = self.adapter.get_info();

        compositor::Information {
            adapter: information.name,
            backend: format!("{:?}", information.backend),
        }
    }

    fn present(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        on_pre_present: impl FnOnce(),
    ) -> Result<(), compositor::SurfaceError> {
        present(
            renderer,
            surface,
            viewport,
            background_color,
            on_pre_present,
        )
    }

    fn screenshot(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
    ) -> Vec<u8> {
        renderer.screenshot(viewport, background_color)
    }
}
