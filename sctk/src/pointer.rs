use smithay_client_toolkit::{
    reexports::client::protocol::{wl_pointer::{ButtonState, Event, Axis}, wl_surface::WlSurface},
    seat::pointer::ThemedPointer,
    window
};
type SCTKWindow = window::Window<window::ConceptFrame>;
use {crate::{Event::Mouse, input::{self, mouse}}, super::conversion};

// Track focus and reconstruct scroll events
#[derive(Default)] pub struct Pointer {
    focus : Option<WlSurface>,
    axis_buffer: Option<(f32, f32)>,
    axis_discrete_buffer: Option<(i32, i32)>,
}

impl Pointer {
    fn handle(&mut self, event : Event, pointer: ThemedPointer, events: &mut Vec<Event>, window: &SCTKWindow, current_cursor: &'static str) {
        let Self{focus, axis_buffer, axis_discrete_buffer} = self;
        match event {
            Event::Enter { surface, surface_x:x,surface_y:y, .. } if surface == *window.surface() => {
                focus = Some(surface);
                pointer.set_cursor(current_cursor, None).expect("Unknown cursor");
                events.push(Mouse(mouse::Event::CursorEntered));
                events.push(Mouse(mouse::Event::CursorMoved{x: x as f32, y: y as f32}));
            }
            Event::Leave { .. } => {
                focus = None;
                events.push(Event::Mouse(mouse::Event::CursorLeft));
            }
            Event::Motion { surface_x: x, surface_y: y, .. } if focus.is_some() => {
                events.push(Event::Mouse(mouse::Event::CursorMoved{x: x as f32, y: y as f32}));
            }
            Event::Button { button, state, .. } if focus.is_some() => {
                let state = match state {
                    ButtonState::Pressed => input::ButtonState::Pressed,
                    ButtonState::Released => input::ButtonState::Released,
                    _ => unreachable!(),
                };
                events.push(Event::Mouse(mouse::Event::Input{button: conversion::button(button), state}));
            }
            Event::Axis { axis, value, .. } if focus.is_some() => {
                let (mut x, mut y) = axis_buffer.unwrap_or((0.0, 0.0));
                match axis {
                    // wayland vertical sign convention is the inverse of iced
                    Axis::VerticalScroll => y -= value as f32,
                    Axis::HorizontalScroll => x += value as f32,
                    _ => unreachable!(),
                }
                axis_buffer = Some((x, y));
            }
            Event::Frame if focus.is_some() => {
                let axis_buffer = axis_buffer.take();
                let axis_discrete_buffer = axis_discrete_buffer.take();
                if let Some((x, y)) = axis_discrete_buffer {
                    events.push(Event::Mouse(mouse::Event::WheelScrolled {delta: mouse::ScrollDelta::Lines {x: x as f32, y: y as f32}}));
                }
                else if let Some((x, y)) = axis_buffer {
                    events.push(Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Pixels {x,y}}));
                }
            }
            Event::AxisSource { .. } => (),
            Event::AxisStop { .. } => (),
            Event::AxisDiscrete { axis, discrete } if focus.is_some() => {
                let (mut x, mut y) = axis_discrete_buffer.unwrap_or((0, 0));
                match axis {
                    // wayland vertical sign convention is the inverse of iced
                    Axis::VerticalScroll => y -= discrete,
                    Axis::HorizontalScroll => x += discrete,
                    _ => unreachable!(),
                }
                axis_discrete_buffer = Some((x, y));
            }
            _ => panic!("Out of focus"),
        }
    }
}
