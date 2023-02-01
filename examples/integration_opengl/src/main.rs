mod controls;
mod scene;

use std::{ffi::CString, num::NonZeroU32};

use controls::Controls;
use scene::Scene;

use glow::*;
use glutin::{
    context::NotCurrentGlContextSurfaceAccessor,
    display::{GetGlDisplay, GlDisplay},
    surface::GlSurface,
};
use iced_glow::glow;
use iced_glow::{Backend, Renderer, Settings, Viewport};
use iced_native::renderer;
use iced_native::{program, Color, Debug, Point, Size};
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::PhysicalPosition;
use winit::event::{Event, ModifiersState, WindowEvent};
use winit::event_loop::ControlFlow;

pub fn main() {
    env_logger::init();
    let (gl, event_loop, window, context, surface, shader_version) = {
        let el = winit::event_loop::EventLoop::new();

        let wb = winit::window::WindowBuilder::new()
            .with_title("OpenGL integration example")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

        // Just using first config
        let tb = glutin::config::ConfigTemplateBuilder::new()
            .with_transparency(true);
        let (window, config) = glutin_winit::DisplayBuilder::new()
            .with_window_builder(Some(wb))
            .build(&el, tb, |mut iter| iter.next().unwrap())
            .unwrap();
        let window = window.unwrap();

        let attributes = glutin::context::ContextAttributesBuilder::new()
            .build(Some(window.raw_window_handle()));
        let context =
            unsafe { config.display().create_context(&config, &attributes) }
                .unwrap();

        let physical_size = window.inner_size();

        let surface_attributes = glutin::surface::SurfaceAttributesBuilder::<
            glutin::surface::WindowSurface,
        >::new()
        .build(
            window.raw_window_handle(),
            NonZeroU32::new(physical_size.width).unwrap(),
            NonZeroU32::new(physical_size.height).unwrap(),
        );
        let surface = unsafe {
            config
                .display()
                .create_window_surface(&config, &surface_attributes)
                .unwrap()
        };

        unsafe {
            let context = context.make_current(&surface).unwrap();

            // Enable vsync
            surface
                .set_swap_interval(
                    &context,
                    glutin::surface::SwapInterval::Wait(
                        NonZeroU32::new(1).unwrap(),
                    ),
                )
                .unwrap();

            let gl = glow::Context::from_loader_function(|s| {
                config.display().get_proc_address(&CString::new(s).unwrap())
            });

            // Enable auto-conversion from/to sRGB
            gl.enable(glow::FRAMEBUFFER_SRGB);

            // Enable alpha blending
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            // Disable multisampling by default
            gl.disable(glow::MULTISAMPLE);

            (gl, el, window, context, surface, "#version 410")
        }
    };

    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );

    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = iced_native::clipboard::Null;

    let mut renderer = Renderer::new(Backend::new(&gl, Settings::default()));

    let mut debug = Debug::new();

    let controls = Controls::new();
    let mut state = program::State::new(
        controls,
        viewport.logical_size(),
        &mut renderer,
        &mut debug,
    );
    let mut resized = false;

    let scene = Scene::new(&gl, shader_version);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = position;
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(physical_size) => {
                        viewport = Viewport::with_physical_size(
                            Size::new(
                                physical_size.width,
                                physical_size.height,
                            ),
                            window.scale_factor(),
                        );

                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        scene.cleanup(&gl);
                        *control_flow = ControlFlow::Exit
                    }
                    _ => (),
                }

                if let Some(event) =
                    window_event(&event, window.scale_factor(), modifiers)
                {
                    state.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                // If there are events pending
                let logical_position =
                    cursor_position.to_logical(viewport.scale_factor());
                if !state.is_queue_empty() {
                    // We update iced
                    let _ = state.update(
                        viewport.logical_size(),
                        Point::new(logical_position.x, logical_position.y),
                        &mut renderer,
                        &iced_glow::Theme::Dark,
                        &renderer::Style {
                            text_color: Color::WHITE,
                        },
                        &mut clipboard,
                        &mut debug,
                    );

                    // and request a redraw
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    unsafe {
                        gl.viewport(
                            0,
                            0,
                            size.width as i32,
                            size.height as i32,
                        );
                    }

                    resized = false;
                }

                let program = state.program();
                {
                    // We clear the frame
                    scene.clear(&gl, program.background_color());

                    // Draw the scene
                    scene.draw(&gl);
                }

                // And then iced on top
                renderer.with_primitives(|backend, primitive| {
                    backend.present(
                        &gl,
                        primitive,
                        &viewport,
                        &debug.overlay(),
                    );
                });

                // Update the mouse cursor
                window.set_cursor_icon(mouse_interaction(
                    state.mouse_interaction(),
                ));

                surface.swap_buffers(&context).unwrap();
            }
            _ => (),
        }
    });
}

// Duplicates logic from `iced_winit`, but only relevant events
fn window_event(
    event: &winit::event::WindowEvent<'_>,
    scale_factor: f64,
    _modifiers: winit::event::ModifiersState,
) -> Option<iced_native::Event> {
    use iced_native::{mouse, window};

    match event {
        WindowEvent::Resized(new_size) => {
            let logical_size = new_size.to_logical(scale_factor);

            Some(iced_native::Event::Window(window::Event::Resized {
                width: logical_size.width,
                height: logical_size.height,
            }))
        }
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
            let logical_size = new_inner_size.to_logical(scale_factor);

            Some(iced_native::Event::Window(window::Event::Resized {
                width: logical_size.width,
                height: logical_size.height,
            }))
        }
        WindowEvent::CloseRequested => {
            Some(iced_native::Event::Window(window::Event::CloseRequested))
        }
        WindowEvent::CursorMoved { position, .. } => {
            let position = position.to_logical::<f64>(scale_factor);

            Some(iced_native::Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(position.x as f32, position.y as f32),
            }))
        }
        WindowEvent::CursorEntered { .. } => {
            Some(iced_native::Event::Mouse(mouse::Event::CursorEntered))
        }
        WindowEvent::CursorLeft { .. } => {
            Some(iced_native::Event::Mouse(mouse::Event::CursorLeft))
        }
        WindowEvent::MouseInput { button, state, .. } => {
            let button = mouse_button(*button);

            Some(iced_native::Event::Mouse(match state {
                winit::event::ElementState::Pressed => {
                    mouse::Event::ButtonPressed(button)
                }
                winit::event::ElementState::Released => {
                    mouse::Event::ButtonReleased(button)
                }
            }))
        }
        WindowEvent::Focused(focused) => {
            Some(iced_native::Event::Window(if *focused {
                window::Event::Focused
            } else {
                window::Event::Unfocused
            }))
        }
        WindowEvent::Moved(position) => {
            let winit::dpi::LogicalPosition { x, y } =
                position.to_logical(scale_factor);

            Some(iced_native::Event::Window(window::Event::Moved { x, y }))
        }
        _ => None,
    }
}

// Duplicates logic from `iced_winit`
fn mouse_button(
    mouse_button: winit::event::MouseButton,
) -> iced_native::mouse::Button {
    use iced_native::mouse;

    match mouse_button {
        winit::event::MouseButton::Left => mouse::Button::Left,
        winit::event::MouseButton::Right => mouse::Button::Right,
        winit::event::MouseButton::Middle => mouse::Button::Middle,
        winit::event::MouseButton::Other(other) => {
            mouse::Button::Other(other as u8)
        }
    }
}

// Duplicates logic from `iced_winit`
fn mouse_interaction(
    interaction: iced_native::mouse::Interaction,
) -> winit::window::CursorIcon {
    use iced_native::mouse::Interaction;

    match interaction {
        Interaction::Idle => winit::window::CursorIcon::Default,
        Interaction::Pointer => winit::window::CursorIcon::Hand,
        Interaction::Working => winit::window::CursorIcon::Progress,
        Interaction::Grab => winit::window::CursorIcon::Grab,
        Interaction::Grabbing => winit::window::CursorIcon::Grabbing,
        Interaction::Crosshair => winit::window::CursorIcon::Crosshair,
        Interaction::Text => winit::window::CursorIcon::Text,
        Interaction::ResizingHorizontally => {
            winit::window::CursorIcon::EwResize
        }
        Interaction::ResizingVertically => winit::window::CursorIcon::NsResize,
    }
}
