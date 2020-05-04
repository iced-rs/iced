use std::{rc::Rc, cell::Cell, time::{Instant, Duration}};
use futures::{future::FutureExt, stream::StreamExt};
pub use smithay_client_toolkit::reexports::client::protocol::wl_keyboard::{self, KeyState};
use {super::{Streams,Item}, crate::{Event, input::{ButtonState, keyboard::{self, ModifiersState}}}, super::conversion};

/// Track modifiers and key repetition
#[derive(Default)] pub(crate) struct Keyboard {
    modifiers : ModifiersState,
    repeat : Option<Rc<Cell<u32>>>,
}

impl Keyboard {
    pub(crate) fn map<M>(&mut self, streams: &mut Streams<M>, event: wl_keyboard::Event) -> Option<Event> {
        let Self{modifiers, repeat} = self;
        use wl_keyboard::Event::*;
        match event {
            Keymap { .. } => None, // todo
            RepeatInfo { .. } => None, // todo
            Enter { .. } => None,
            Leave { .. } => { *repeat = None; None } // will drop the timer on its next event (Weak::upgrade=None)
            Key{ key, state, .. } => {
                match state {
                    KeyState::Pressed => {
                        if let Some(repeat) = repeat { // Update existing repeat cell
                            repeat.set(key);
                            // Note: This keeps the same timer on key repeat change. No delay! Nice!
                        } else { // New repeat timer (registers in the reactor on first poll)
                            //assert!(!is_repeat);
                            let repeat = Rc::new(Cell::new(key));
                            use futures::stream;
                            streams.push(
                                stream::unfold(Instant::now()+Duration::from_millis(300), {
                                    let repeat = Rc::downgrade(&repeat);
                                    move |last| {
                                        let next = last+Duration::from_millis(100);
                                        smol::Timer::at(next).map({
                                            let repeat = repeat.clone();
                                            move |_| { repeat.upgrade().map(|x| (Item::KeyRepeat(x.get()), next) ) } // Option<Key> (None stops the stream, autodrops from streams)
                                        })
                                    }
                                }).boxed_local()
                            );
                            self.repeat = Some(repeat);
                        }
                    }
                    KeyState::Released => {
                        if repeat.as_ref().filter(|r| r.get()==key ).is_some() { *repeat = None }
                    }
                    _ => unreachable!(),
                }
                Some(self.key(key, state == KeyState::Pressed))
            }
            Modifiers {mods_depressed, mods_latched, mods_locked, group: locked_group, ..} => {
                *modifiers = conversion::modifiers(mods_depressed, mods_latched, mods_locked, locked_group);
                //if let Some(repeat) = repeat { repeat.update(|r| r.modifiers = modifiers )} // Optional logic
                None
            }
            _ => panic!("Keyboard"),
        }
    }
    pub fn key(&self, key: u32, state: bool) -> Event {
        Event::Keyboard(keyboard::Event::Input{
            key_code: conversion::key(key),
            state: if state { ButtonState::Pressed } else { ButtonState::Released },
            modifiers: self.modifiers,
        })
        /*if let Some(ref txt) = utf8 {
            for char in txt.chars() {
                events.push(Event::Keyboard(keyboard::Event::CharacterReceived(char)));
            }
        }*/
    }
}
