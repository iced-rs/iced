//! Virtual keyboard support for web (WASM) targets.
//!
//! On mobile browsers the virtual keyboard only appears when a native HTML
//! `<input>` element is focused. [`TextAgent`] manages a hidden `<input>`
//! and bridges its events into iced's [`input_method::Event`] flow.
//!
//! [`text_input`]: iced_widget::text_input
use crate::core::input_method;
use crate::core::keyboard::{self, key};
use crate::core::{self, window};
use crate::futures::futures::channel::mpsc;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

/// Manages a hidden HTML `<input>` element so that iced [`text_input`]
/// widgets trigger the native virtual keyboard on mobile browsers.
///
/// When a [`text_input`] is focused, [`TextAgent::enable`] is called.
/// Events from the input (composition, plain text, special keys) are
/// forwarded as [`input_method::Event`] / [`keyboard::Event`] variants.
///
/// ## Mobile focus strategy
///
/// Android and desktop browsers allow `focus()` from any JS context, so
/// [`enable`] calls it directly. iOS Safari requires `focus()` to be called
/// synchronously inside a user-gesture handler — a `touchstart` listener on
/// `document` handles this.
///
/// [`enable`]: TextAgent::enable
/// [`text_input`]: iced_widget::text_input
pub struct TextAgent {
    input: HtmlInputElement,
    /// `true` while a [`text_input`] is focused. The event loop suppresses
    /// [`window::Event::Unfocused`] while this is set.
    active: Rc<Cell<bool>>,
}

impl TextAgent {
    /// Creates a new [`TextAgent`], appending a hidden `<input>` to
    /// `document.body`. Returns a [`JsValue`] error if the DOM is unavailable.
    pub fn new(
        window_id: window::Id,
        sender: mpsc::UnboundedSender<(window::Id, core::Event)>,
        request_redraw: impl Fn() + 'static,
    ) -> Result<Self, JsValue> {
        let web_window =
            web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let document = web_window
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let body = document
            .body()
            .ok_or_else(|| JsValue::from_str("no body"))?;

        let input: HtmlInputElement = document
            .create_element("input")
            .map_err(|_| JsValue::from_str("create_element failed"))?
            .dyn_into()
            .map_err(|_| JsValue::from_str("not an HtmlInputElement"))?;

        let _ = input.set_attribute("type", "text");
        let _ = input.set_attribute("autocapitalize", "off");
        let _ = input.set_attribute("autocomplete", "off");
        let _ = input.set_attribute("autocorrect", "off");
        let _ = input.set_attribute("spellcheck", "false");
        // tabindex=-1: reachable via focus() but skipped by Tab navigation.
        let _ = input.set_attribute("tabindex", "-1");

        // Must NOT be display:none or visibility:hidden — those prevent the
        // virtual keyboard on iOS/Android. Non-zero size and font-size:16px
        // prevent iOS from ignoring focus() and auto-zooming.
        let style = input.style();
        let _ = style.set_property("position", "fixed");
        let _ = style.set_property("top", "-100vh");
        let _ = style.set_property("left", "-100vw");
        let _ = style.set_property("width", "32px");
        let _ = style.set_property("height", "32px");
        let _ = style.set_property("font-size", "16px");
        let _ = style.set_property("opacity", "0");
        let _ = style.set_property("color", "transparent");
        let _ = style.set_property("background", "transparent");
        let _ = style.set_property("border", "none");
        let _ = style.set_property("outline", "none");
        let _ = style.set_property("padding", "0");
        let _ = style.set_property("caret-color", "transparent");
        let _ = style.set_property("pointer-events", "none");

        let _ = body
            .append_child(&input)
            .map_err(|_| JsValue::from_str("append_child failed"))?;

        let composing: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let active: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let redraw: Rc<dyn Fn()> = Rc::from(request_redraw);

        // iOS Safari requires focus() inside a synchronous user-gesture handler.
        {
            let input_ref = input.clone();
            let closure = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
                move |_: web_sys::Event| {
                    let _ = input_ref.focus();
                },
            ));
            document
                .add_event_listener_with_callback(
                    "touchstart",
                    closure.as_ref().unchecked_ref(),
                )
                .expect("touchstart listener");
            closure.forget();
        }

        // compositionstart → InputMethod::Opened
        {
            let sender = sender.clone();
            let composing = composing.clone();
            let redraw = redraw.clone();
            let closure =
                Closure::<dyn FnMut(web_sys::CompositionEvent)>::wrap(Box::new(
                    move |_: web_sys::CompositionEvent| {
                        *composing.borrow_mut() = true;
                        let _ = sender.unbounded_send((
                            window_id,
                            core::Event::InputMethod(input_method::Event::Opened),
                        ));
                        redraw();
                    },
                ));
            input
                .add_event_listener_with_callback(
                    "compositionstart",
                    closure.as_ref().unchecked_ref(),
                )
                .expect("compositionstart listener");
            closure.forget();
        }

        // compositionupdate → InputMethod::Preedit
        {
            let sender = sender.clone();
            let redraw = redraw.clone();
            let closure =
                Closure::<dyn FnMut(web_sys::CompositionEvent)>::wrap(Box::new(
                    move |e: web_sys::CompositionEvent| {
                        let data = e.data().unwrap_or_default();
                        let end = data.len();
                        let _ = sender.unbounded_send((
                            window_id,
                            core::Event::InputMethod(input_method::Event::Preedit(
                                data,
                                Some(end..end),
                            )),
                        ));
                        redraw();
                    },
                ));
            input
                .add_event_listener_with_callback(
                    "compositionupdate",
                    closure.as_ref().unchecked_ref(),
                )
                .expect("compositionupdate listener");
            closure.forget();
        }

        // compositionend → Preedit("") + Commit + Closed
        {
            let sender = sender.clone();
            let input_ref = input.clone();
            let composing = composing.clone();
            let redraw = redraw.clone();
            let closure =
                Closure::<dyn FnMut(web_sys::CompositionEvent)>::wrap(Box::new(
                    move |e: web_sys::CompositionEvent| {
                        *composing.borrow_mut() = false;
                        let text = e.data().unwrap_or_default();
                        // Reset value before the trailing `input` event fires.
                        input_ref.set_value("");
                        let _ = sender.unbounded_send((
                            window_id,
                            core::Event::InputMethod(input_method::Event::Preedit(
                                String::new(),
                                None,
                            )),
                        ));
                        let _ = sender.unbounded_send((
                            window_id,
                            core::Event::InputMethod(input_method::Event::Commit(text)),
                        ));
                        let _ = sender.unbounded_send((
                            window_id,
                            core::Event::InputMethod(input_method::Event::Closed),
                        ));
                        redraw();
                    },
                ));
            input
                .add_event_listener_with_callback(
                    "compositionend",
                    closure.as_ref().unchecked_ref(),
                )
                .expect("compositionend listener");
            closure.forget();
        }

        // input → InputMethod::Commit (plain typing / paste; skipped during IME)
        {
            let sender = sender.clone();
            let input_ref = input.clone();
            let redraw = redraw.clone();
            let closure = Closure::<dyn FnMut(web_sys::InputEvent)>::wrap(Box::new(
                move |_: web_sys::InputEvent| {
                    if *composing.borrow() {
                        return;
                    }
                    let text = input_ref.value();
                    if text.is_empty() {
                        return;
                    }
                    input_ref.set_value("");
                    let _ = sender.unbounded_send((
                        window_id,
                        core::Event::InputMethod(input_method::Event::Commit(text)),
                    ));
                    redraw();
                },
            ));
            input
                .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                .expect("input listener");
            closure.forget();
        }

        // keydown → KeyPressed for named keys (`input` doesn't fire for these)
        {
            let closure =
                Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new(
                    move |e: web_sys::KeyboardEvent| {
                        let named = match e.key().as_str() {
                            "Backspace" => key::Named::Backspace,
                            "Delete" => key::Named::Delete,
                            "Enter" => key::Named::Enter,
                            "Escape" => key::Named::Escape,
                            "ArrowLeft" => key::Named::ArrowLeft,
                            "ArrowRight" => key::Named::ArrowRight,
                            "ArrowUp" => key::Named::ArrowUp,
                            "ArrowDown" => key::Named::ArrowDown,
                            "Home" => key::Named::Home,
                            "End" => key::Named::End,
                            _ => return,
                        };
                        let logical_key = keyboard::Key::Named(named);
                        let _ = sender.unbounded_send((
                            window_id,
                            core::Event::Keyboard(keyboard::Event::KeyPressed {
                                key: logical_key.clone(),
                                modified_key: logical_key,
                                physical_key: key::Physical::Unidentified(
                                    key::NativeCode::Unidentified,
                                ),
                                location: keyboard::Location::Standard,
                                modifiers: keyboard::Modifiers::default(),
                                text: None,
                                repeat: false,
                            }),
                        ));
                        redraw();
                    },
                ));
            input
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .expect("keydown listener");
            closure.forget();
        }

        Ok(Self { input, active })
    }

    /// Marks the agent as active so the virtual keyboard stays visible.
    ///
    /// `focus()` is intentionally **not** called here — the `touchstart`
    /// listener already called it synchronously within the user gesture,
    /// which is the only path iOS Safari accepts.
    pub fn enable(&self) {
        self.active.set(true);
    }

    /// Blurs the hidden input, hiding the OS virtual keyboard.
    pub fn disable(&self) {
        self.active.set(false);
        let _ = self.input.blur();
    }

    /// Returns `true` while a [`text_input`] holds focus.
    ///
    /// [`text_input`]: iced_widget::text_input
    pub fn is_active(&self) -> bool {
        self.active.get()
    }
}

impl Drop for TextAgent {
    fn drop(&mut self) {
        if let Some(parent) = self.input.parent_node() {
            let _ = parent.remove_child(&self.input);
        }
    }
}
