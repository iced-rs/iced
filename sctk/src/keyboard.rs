use std::{rc::Rc, cell::Cell, time::{Instant, Duration}};
use futures::{future::FutureExt, stream::StreamExt};
pub use smithay_client_toolkit::seat::keyboard::{Event, KeyState};
use {super::{Update,Item}, crate::{input::{ButtonState, keyboard::{self, ModifiersState}}}, super::conversion};

// Track modifiers and key repetition
#[derive(Default)] pub struct Keyboard {
    modifiers : ModifiersState,
    repeat : Option<Rc<Cell<Event<'static>>>>,
}

impl Keyboard {
    pub fn handle<W>(&mut self, Update{streams, events, ..}: &mut Update<W>, event: Event) {
        let Self{modifiers, repeat} = self;
        match event {
            Event::Enter { .. } => (),
            Event::Leave { .. } => *repeat = None, // will drop the timer on its next event (Weak::upgrade=None)
            key @ Event::Key{ state, utf8, .. } => {
                if state == KeyState::Pressed {
                    if let Some(repeat) = repeat { // Update existing repeat cell (also triggered by the actual repetition => noop)
                        repeat.set(event);
                        // Note: This keeps the same timer on key repeat change. No delay! Nice!
                    } else { // New repeat timer (registers in the reactor on first poll)
                        //assert!(!is_repeat);
                        let repeat = Rc::new(Cell::new(event));
                        use futures::stream;
                        streams.get_mut().push(
                            stream::unfold(Instant::now()+Duration::from_millis(300), {
                                let repeat = Rc::downgrade(&repeat);
                                |last| {
                                    let next = last+Duration::from_millis(100);
                                    smol::Timer::at(next).map(move |_| { repeat.upgrade().map(|x| (Item::Key(x.clone().into_inner()), next) ) }) // Option<Key> (None stops the stream, autodrops from streams)
                                }
                            })
                        );
                        repeat = Some(Cell::new(event));
                    }
                } else {
                    if repeat.filter(|r| r.get()==event).is_some() { repeat = None }
                }
                events.push(Event::Keyboard(keyboard::Event::Input{
                    key_code: conversion::key(key),
                    state: if state == KeyState::Pressed { ButtonState::Pressed } else { ButtonState::Released },
                    modifiers,
                }));
                if let Some(ref txt) = utf8 {
                    for char in txt.chars() {
                        events.push(Event::Keyboard(keyboard::Event::CharacterReceived(char)));
                    }
                }
            }
            Event::Modifiers {
                modifiers: new_modifiers,
            } => {
                *modifiers = conversion::modifiers(new_modifiers);
                    if let Some(repeat) = repeat { repeat.update(|r| r.modifiers = modifiers )} // Optional logic
            }
        }
    }
}
