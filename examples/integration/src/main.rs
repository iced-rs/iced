mod controls;
mod scene;

use controls::Controls;
use scene::Scene;

use iced_wgpu::{
    wgpu, window::SwapChain, Primitive, Renderer, Settings, Target,
};
use iced_winit::{
    futures, mouse, winit, Cache, Clipboard, Size, UserInterface,
};

use winit::{
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub fn main() {
    env_logger::init();

    // Initialize winit
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let mut logical_size =
        window.inner_size().to_logical(window.scale_factor());
    let mut modifiers = ModifiersState::default();

    // Initialize WGPU

    let surface = wgpu::Surface::create(&window);
    let (mut device, queue) = futures::executor::block_on(async {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .expect("Request adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await
    });

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let mut swap_chain = {
        let size = window.inner_size();

        SwapChain::new(&device, &surface, format, size.width, size.height)
    };
    let mut resized = false;

    // Initialize iced
    let mut events = Vec::new();
    let mut cache = Some(Cache::default());
    let mut renderer = Renderer::new(&mut device, Settings::default());
    let mut output = (Primitive::None, mouse::Interaction::default());
    let clipboard = Clipboard::new(&window);

    // Initialize scene and GUI controls
    let mut scene = Scene::new(&device);
    let mut controls = Controls::new();

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        // You should change this if you want to render continuosly
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(new_size) => {
                        logical_size =
                            new_size.to_logical(window.scale_factor());
                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }

                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) = iced_winit::conversion::window_event(
                    &event,
                    window.scale_factor(),
                    modifiers,
                ) {
                    events.push(event);
                }
            }
            Event::MainEventsCleared => {
                // If no relevant events happened, we can simply skip this
                if events.is_empty() {
                    return;
                }

                // We need to:
                // 1. Process events of our user interface.
                // 2. Update state as a result of any interaction.
                // 3. Generate a new output for our renderer.

                // First, we build our user interface.
                let mut user_interface = UserInterface::build(
                    controls.view(&scene),
                    Size::new(logical_size.width, logical_size.height),
                    cache.take().unwrap(),
                    &mut renderer,
                );

                // Then, we process the events, obtaining messages in return.
                let messages = user_interface.update(
                    events.drain(..),
                    clipboard.as_ref().map(|c| c as _),
                    &renderer,
                );

                let user_interface = if messages.is_empty() {
                    // If there are no messages, no interactions we care about have
                    // happened. We can simply leave our user interface as it is.
                    user_interface
                } else {
                    // If there are messages, we need to update our state
                    // accordingly and rebuild our user interface.
                    // We can only do this if we drop our user interface first
                    // by turning it into its cache.
                    cache = Some(user_interface.into_cache());

                    // In this example, `Controls` is the only part that cares
                    // about messages, so updating our state is pretty
                    // straightforward.
                    for message in messages {
                        controls.update(message, &mut scene);
                    }

                    // Once the state has been changed, we rebuild our updated
                    // user interface.
                    UserInterface::build(
                        controls.view(&scene),
                        Size::new(logical_size.width, logical_size.height),
                        cache.take().unwrap(),
                        &mut renderer,
                    )
                };

                // Finally, we just need to draw a new output for our renderer,
                output = user_interface.draw(&mut renderer);

                // update our cache,
                cache = Some(user_interface.into_cache());

                // and request a redraw
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    swap_chain = SwapChain::new(
                        &device,
                        &surface,
                        format,
                        size.width,
                        size.height,
                    );
                }

                let (frame, viewport) =
                    swap_chain.next_frame().expect("Next frame");

                let mut encoder = device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor { label: None },
                );

                // We draw the scene first
                scene.draw(&mut encoder, &frame.view);

                // And then iced on top
                let mouse_interaction = renderer.draw(
                    &mut device,
                    &mut encoder,
                    Target {
                        texture: &frame.view,
                        viewport,
                    },
                    &output,
                    window.scale_factor(),
                    &["Some debug information!"],
                );

                // Then we submit the work
                queue.submit(&[encoder.finish()]);

                // And update the mouse cursor
                window.set_cursor_icon(
                    iced_winit::conversion::mouse_interaction(
                        mouse_interaction,
                    ),
                );
            }
            _ => {}
        }
    })
}
