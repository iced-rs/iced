mod controls;
mod scene;

use controls::Controls;
use scene::Scene;

use glow;
use glow::*;
use iced_glow::{Backend, Renderer, Settings, Viewport};
use iced_glutin::glutin;
use iced_glutin::glutin::event::{Event, WindowEvent};
use iced_glutin::glutin::event_loop::ControlFlow;
use iced_glutin::{program, Clipboard, Debug, Size};
use iced_winit::conversion;
use iced_winit::winit;
use winit::{dpi::PhysicalPosition, event::ModifiersState};

pub fn main() {
    env_logger::init();
    let (gl, event_loop, windowed_context, shader_version) = {
        let el = glutin::event_loop::EventLoop::new();

        let wb = glutin::window::WindowBuilder::new()
            .with_title("OpenGL integration example")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));

        let windowed_context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(wb, &el)
            .unwrap();

        unsafe {
            let windowed_context = windowed_context.make_current().unwrap();

            let gl = glow::Context::from_loader_function(|s| {
                windowed_context.get_proc_address(s) as *const _
            });

            // Enable auto-conversion from/to sRGB
            gl.enable(glow::FRAMEBUFFER_SRGB);

            // Enable alpha blending
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            // Disable multisampling by default
            gl.disable(glow::MULTISAMPLE);

            (gl, el, windowed_context, "#version 410")
        }
    };

    let physical_size = windowed_context.window().inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        windowed_context.window().scale_factor(),
    );

    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&windowed_context.window());

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

    event_loop.run(move |event, _, control_flow| {
        let scene = Scene::new(&gl, &shader_version);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
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
                            windowed_context.window().scale_factor(),
                        );

                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        scene.cleanup(&gl);
                        *control_flow = ControlFlow::Exit
                    }
                    _ => (),
                }

                // Map window event to iced event
                if let Some(event) = iced_winit::conversion::window_event(
                    &event,
                    windowed_context.window().scale_factor(),
                    modifiers,
                ) {
                    state.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                // If there are events pending
                if !state.is_queue_empty() {
                    // We update iced
                    let _ = state.update(
                        viewport.logical_size(),
                        conversion::cursor_position(
                            cursor_position,
                            viewport.scale_factor(),
                        ),
                        &mut renderer,
                        &mut clipboard,
                        &mut debug,
                    );

                    // and request a redraw
                    windowed_context.window().request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = windowed_context.window().inner_size();

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
                windowed_context.window().set_cursor_icon(
                    iced_winit::conversion::mouse_interaction(
                        state.mouse_interaction(),
                    ),
                );

                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
