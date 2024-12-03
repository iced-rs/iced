//! Test your `iced` applications in headless mode.
#![allow(missing_docs, missing_debug_implementations)]
pub mod selector;

pub use selector::Selector;

use iced_renderer as renderer;
use iced_runtime as runtime;
use iced_runtime::core;
use iced_tiny_skia as tiny_skia;

use crate::core::clipboard;
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::widget;
use crate::core::{Element, Event, Font, Pixels, Rectangle, Size, SmolStr};
use crate::renderer::Renderer;
use crate::runtime::user_interface;
use crate::runtime::UserInterface;

pub fn interface<'a, Message, Theme>(
    element: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Interface<'a, Message, Theme, Renderer> {
    let mut renderer = Renderer::Secondary(tiny_skia::Renderer::new(
        Font::default(),
        Pixels(16.0),
    ));

    let raw = UserInterface::build(
        element,
        Size::new(1024.0, 1024.0),
        user_interface::Cache::default(),
        &mut renderer,
    );

    Interface {
        raw,
        renderer,
        messages: Vec::new(),
    }
}

pub struct Interface<'a, Message, Theme, Renderer> {
    raw: UserInterface<'a, Message, Theme, Renderer>,
    renderer: Renderer,
    messages: Vec<Message>,
}

pub struct Target {
    bounds: Rectangle,
}

impl<Message, Theme, Renderer> Interface<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    pub fn find(
        &mut self,
        selector: impl Into<Selector>,
    ) -> Result<Target, Error> {
        let selector = selector.into();

        match &selector {
            Selector::Id(id) => {
                struct FindById<'a> {
                    id: &'a widget::Id,
                    target: Option<Target>,
                }

                impl widget::Operation for FindById<'_> {
                    fn container(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        operate_on_children: &mut dyn FnMut(
                            &mut dyn widget::Operation<()>,
                        ),
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                            return;
                        }

                        operate_on_children(self);
                    }

                    fn scrollable(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _content_bounds: Rectangle,
                        _translation: core::Vector,
                        _state: &mut dyn widget::operation::Scrollable,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn text_input(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _state: &mut dyn widget::operation::TextInput,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn text(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _text: &str,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }

                    fn custom(
                        &mut self,
                        id: Option<&widget::Id>,
                        bounds: Rectangle,
                        _state: &mut dyn std::any::Any,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if Some(self.id) == id {
                            self.target = Some(Target { bounds });
                        }
                    }
                }

                let mut find = FindById { id, target: None };

                self.raw.operate(&self.renderer, &mut find);

                find.target.ok_or(Error::NotFound(selector))
            }
            Selector::Text(text) => {
                struct FindByText<'a> {
                    text: &'a str,
                    target: Option<Target>,
                }

                impl widget::Operation for FindByText<'_> {
                    fn container(
                        &mut self,
                        _id: Option<&widget::Id>,
                        _bounds: Rectangle,
                        operate_on_children: &mut dyn FnMut(
                            &mut dyn widget::Operation<()>,
                        ),
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        operate_on_children(self);
                    }

                    fn text(
                        &mut self,
                        _id: Option<&widget::Id>,
                        bounds: Rectangle,
                        text: &str,
                    ) {
                        if self.target.is_some() {
                            return;
                        }

                        if self.text == text {
                            self.target = Some(Target { bounds });
                        }
                    }
                }

                let mut find = FindByText { text, target: None };

                self.raw.operate(&self.renderer, &mut find);

                find.target.ok_or(Error::NotFound(selector))
            }
        }
    }

    pub fn click(
        &mut self,
        selector: impl Into<Selector>,
    ) -> Result<Target, Error> {
        let target = self.find(selector)?;

        let _ = self.raw.update(
            &[
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            ],
            mouse::Cursor::Available(target.bounds.center()),
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );

        Ok(target)
    }

    pub fn typewrite(&mut self, text: impl AsRef<str>) {
        let events: Vec<_> = text
            .as_ref()
            .chars()
            .map(|c| SmolStr::new_inline(&c.to_string()))
            .flat_map(|c| {
                key_press_and_release(
                    keyboard::Key::Character(c.clone()),
                    Some(c),
                )
            })
            .collect();

        let _ = self.raw.update(
            &events,
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    pub fn press_key(&mut self, key: impl Into<keyboard::Key>) {
        let _ = self.raw.update(
            &key_press_and_release(key, None),
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
    }

    pub fn into_messages(self) -> impl IntoIterator<Item = Message> {
        self.messages
    }
}

fn key_press_and_release(
    key: impl Into<keyboard::Key>,
    text: Option<SmolStr>,
) -> [Event; 2] {
    let key = key.into();

    [
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key.clone(),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            text,
        }),
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
        }),
    ]
}

#[derive(Debug, Clone)]
pub enum Error {
    NotFound(Selector),
}
