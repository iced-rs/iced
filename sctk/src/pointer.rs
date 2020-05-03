use smithay_client_toolkit::{reexports::client::protocol::{wl_pointer::{self, ButtonState}, wl_surface::WlSurface}, seat::pointer::ThemedPointer};
use {crate::{Event::Mouse, input::{self, mouse}}, super::conversion};

// Track focus and reconstruct scroll events
#[derive(Default)] pub struct Pointer {
    focus : Option<WlSurface>,
    axis_buffer: Option<(f32, f32)>,
    axis_discrete_buffer: Option<(i32, i32)>,
}

impl Pointer {
    pub fn handle(&mut self, event : wl_pointer::Event, pointer: ThemedPointer, events: &mut Vec<crate::Event>, /*surface: &WlSurface,*/ current_cursor: &'static str) {
        let Self{focus, axis_buffer, axis_discrete_buffer} = self;
        use wl_pointer::Event::*;
        match event {
            Enter { surface, surface_x:x,surface_y:y, .. } /*if surface == *window.surface()*/ => {
                *focus = Some(surface);
                pointer.set_cursor(current_cursor, None).expect("Unknown cursor");
                events.push(Mouse(mouse::Event::CursorEntered));
                events.push(Mouse(mouse::Event::CursorMoved{x: x as f32, y: y as f32}));
            }
            Leave { .. } => {
                *focus = None;
                events.push(Mouse(mouse::Event::CursorLeft));
            }
            Motion { surface_x: x, surface_y: y, .. } if focus.is_some() => {
                events.push(Mouse(mouse::Event::CursorMoved{x: x as f32, y: y as f32}));
            }
            Button { button, state, .. } if focus.is_some() => {
                let state = if let ButtonState::Pressed = state { input::ButtonState::Pressed } else { input::ButtonState::Released };
                events.push(Mouse(mouse::Event::Input{button: conversion::button(button), state}));
            }
            Axis { axis, value, .. } if focus.is_some() => {
                let (mut x, mut y) = axis_buffer.unwrap_or((0.0, 0.0));
                use wl_pointer::Axis::*;
                match axis {
                    // wayland vertical sign convention is the inverse of iced
                    VerticalScroll => y -= value as f32,
                    HorizontalScroll => x += value as f32,
                    _ => unreachable!(),
                }
                *axis_buffer = Some((x, y));
            }
            Frame if focus.is_some() => {
                let delta =
                    if let Some((x,y)) = axis_buffer.take() { mouse::ScrollDelta::Pixels{x:x as f32, y:y as f32} }
                    else if let Some((x,y)) = axis_discrete_buffer.take() { mouse::ScrollDelta::Lines{x:x as f32, y:y as f32} }
                    else { debug_assert!(false); mouse::ScrollDelta::Pixels{x:0.,y:0.} };
                events.push(Mouse(mouse::Event::WheelScrolled{delta}));
            }
            AxisSource { .. } => (),
            AxisStop { .. } => (),
            AxisDiscrete { axis, discrete } if focus.is_some() => {
                let (mut x, mut y) = axis_discrete_buffer.unwrap_or((0, 0));
                use wl_pointer::Axis::*;
                match axis {
                    // wayland vertical sign convention is the inverse of iced
                    VerticalScroll => y -= discrete,
                    HorizontalScroll => x += discrete,
                    _ => unreachable!(),
                }
                *axis_discrete_buffer = Some((x, y));
            }
            _ => panic!("Out of focus"),
        }
    }
}
