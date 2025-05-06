mod controls;
mod scene;

use controls::Controls;
use scene::Scene;

use iced_wgpu::graphics::Viewport;
use iced_wgpu::{Engine, Renderer, wgpu};
use iced_winit::Clipboard;
use iced_winit::conversion;
use iced_winit::core::mouse;
use iced_winit::core::renderer;
use iced_winit::core::time::Instant;
use iced_winit::core::window;
use iced_winit::core::{Event, Font, Pixels, Size, Theme};
use iced_winit::futures;
use iced_winit::runtime::user_interface::{self, UserInterface};
use iced_winit::winit;

use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::ModifiersState,
};

use std::sync::Arc;

pub fn main() -> Result<(), winit::error::EventLoopError> {
    tracing_subscriber::fmt::init();

    // Initialize winit
    let event_loop = EventLoop::new()?;

    #[allow(clippy::large_enum_variant)]
    enum Runner {
        Loading,
        Ready {
            window: Arc<winit::window::Window>,
            queue: wgpu::Queue,
            device: wgpu::Device,
            surface: wgpu::Surface<'static>,
            format: wgpu::TextureFormat,
            renderer: Renderer,
            scene: Scene,
            controls: Controls,
            events: Vec<Event>,
            cursor: mouse::Cursor,
            cache: user_interface::Cache,
            clipboard: Clipboard,
            viewport: Viewport,
            modifiers: ModifiersState,
            resized: bool,
        },
    }

    impl winit::application::ApplicationHandler for Runner {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            if let Self::Loading = self {
                let window = Arc::new(
                    event_loop
                        .create_window(
                            winit::window::WindowAttributes::default(),
                        )
                        .expect("Create window"),
                );

                let physical_size = window.inner_size();
                let viewport = Viewport::with_physical_size(
                    Size::new(physical_size.width, physical_size.height),
                    window.scale_factor(),
                );
                let clipboard = Clipboard::connect(window.clone());

                let backend = wgpu::Backends::from_env().unwrap_or_default();

                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: backend,
                    ..Default::default()
                });
                let surface = instance
                    .create_surface(window.clone())
                    .expect("Create window surface");

                let (format, adapter, device, queue) =
                    futures::futures::executor::block_on(async {
                        let adapter =
                            wgpu::util::initialize_adapter_from_env_or_default(
                                &instance,
                                Some(&surface),
                            )
                            .await
                            .expect("Create adapter");

                        let adapter_features = adapter.features();

                        let capabilities = surface.get_capabilities(&adapter);

                        let (device, queue) = adapter
                            .request_device(
                                &wgpu::DeviceDescriptor {
                                    label: None,
                                    required_features: adapter_features
                                        & wgpu::Features::default(),
                                    required_limits: wgpu::Limits::default(),
                                    memory_hints:
                                        wgpu::MemoryHints::MemoryUsage,
                                },
                                None,
                            )
                            .await
                            .expect("Request device");

                        (
                            capabilities
                                .formats
                                .iter()
                                .copied()
                                .find(wgpu::TextureFormat::is_srgb)
                                .or_else(|| {
                                    capabilities.formats.first().copied()
                                })
                                .expect("Get preferred format"),
                            adapter,
                            device,
                            queue,
                        )
                    });

                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format,
                        width: physical_size.width,
                        height: physical_size.height,
                        present_mode: wgpu::PresentMode::AutoVsync,
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                        view_formats: vec![],
                        desired_maximum_frame_latency: 2,
                    },
                );

                // Initialize scene and GUI controls
                let scene = Scene::new(&device, format);
                let controls = Controls::new();

                // Initialize iced

                let renderer = {
                    let engine = Engine::new(
                        &adapter,
                        device.clone(),
                        queue.clone(),
                        format,
                        None,
                    );

                    Renderer::new(engine, Font::default(), Pixels::from(16))
                };

                // You should change this if you want to render continuously
                event_loop.set_control_flow(ControlFlow::Wait);

                *self = Self::Ready {
                    window,
                    device,
                    queue,
                    renderer,
                    surface,
                    format,
                    scene,
                    controls,
                    events: Vec::new(),
                    cursor: mouse::Cursor::Unavailable,
                    modifiers: ModifiersState::default(),
                    cache: user_interface::Cache::new(),
                    clipboard,
                    viewport,
                    resized: false,
                };
            }
        }

        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
            let Self::Ready {
                window,
                device,
                queue,
                surface,
                format,
                renderer,
                scene,
                controls,
                events,
                viewport,
                cursor,
                modifiers,
                clipboard,
                cache,
                resized,
            } = self
            else {
                return;
            };

            match event {
                WindowEvent::RedrawRequested => {
                    if *resized {
                        let size = window.inner_size();

                        *viewport = Viewport::with_physical_size(
                            Size::new(size.width, size.height),
                            window.scale_factor(),
                        );

                        surface.configure(
                            device,
                            &wgpu::SurfaceConfiguration {
                                format: *format,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                width: size.width,
                                height: size.height,
                                present_mode: wgpu::PresentMode::AutoVsync,
                                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                                view_formats: vec![],
                                desired_maximum_frame_latency: 2,
                            },
                        );

                        *resized = false;
                    }

                    match surface.get_current_texture() {
                        Ok(frame) => {
                            let view = frame.texture.create_view(
                                &wgpu::TextureViewDescriptor::default(),
                            );

                            let mut encoder = device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor { label: None },
                            );

                            {
                                // Clear the frame
                                let mut render_pass = Scene::clear(
                                    &view,
                                    &mut encoder,
                                    controls.background_color(),
                                );

                                // Draw the scene
                                scene.draw(&mut render_pass);
                            }

                            // Submit the scene
                            queue.submit([encoder.finish()]);

                            // Draw iced on top
                            let mut interface = UserInterface::build(
                                controls.view(),
                                viewport.logical_size(),
                                std::mem::take(cache),
                                renderer,
                            );

                            let (state, _) = interface.update(
                                &[Event::Window(
                                    window::Event::RedrawRequested(
                                        Instant::now(),
                                    ),
                                )],
                                *cursor,
                                renderer,
                                clipboard,
                                &mut Vec::new(),
                            );

                            // Update the mouse cursor
                            if let user_interface::State::Updated {
                                mouse_interaction,
                                ..
                            } = state
                            {
                                window.set_cursor(
                                    conversion::mouse_interaction(
                                        mouse_interaction,
                                    ),
                                );
                            }

                            // Draw the interface
                            interface.draw(
                                renderer,
                                &Theme::Dark,
                                &renderer::Style::default(),
                                *cursor,
                            );
                            *cache = interface.into_cache();

                            renderer.present(
                                None,
                                frame.texture.format(),
                                &view,
                                viewport,
                            );

                            // Present the frame
                            frame.present();
                        }
                        Err(error) => match error {
                            wgpu::SurfaceError::OutOfMemory => {
                                panic!(
                                    "Swapchain error: {error}. \
                                        Rendering cannot continue."
                                )
                            }
                            _ => {
                                // Try rendering again next frame.
                                window.request_redraw();
                            }
                        },
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    *cursor =
                        mouse::Cursor::Available(conversion::cursor_position(
                            position,
                            viewport.scale_factor(),
                        ));
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    *modifiers = new_modifiers.state();
                }
                WindowEvent::Resized(_) => {
                    *resized = true;
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                _ => {}
            }

            // Map window event to iced event
            if let Some(event) = conversion::window_event(
                event,
                window.scale_factor(),
                *modifiers,
            ) {
                events.push(event);
            }

            // If there are events pending
            if !events.is_empty() {
                // We process them
                let mut interface = UserInterface::build(
                    controls.view(),
                    viewport.logical_size(),
                    std::mem::take(cache),
                    renderer,
                );

                let mut messages = Vec::new();

                let _ = interface.update(
                    events,
                    *cursor,
                    renderer,
                    clipboard,
                    &mut messages,
                );

                events.clear();
                *cache = interface.into_cache();

                // update our UI with any messages
                for message in messages {
                    controls.update(message);
                }

                // and request a redraw
                window.request_redraw();
            }
        }
    }

    let mut runner = Runner::Loading;
    event_loop.run_app(&mut runner)
}
