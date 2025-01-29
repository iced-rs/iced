//! Trigger elements with key combinations.
use crate::core;
use crate::core::keyboard;
use crate::core::keyboard::Hotkey;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    Clipboard, Element, Event, Layout, Length, Rectangle, Shell, Size, Vector,
    Widget,
};

/// A widget that triggers a "click" on its target when a certain hotkey combination
/// is pressed.
#[allow(missing_debug_implementations)]
pub struct Keybind<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> {
    hotkey: Hotkey,
    target: Element<'a, Message, Theme, Renderer>,
    repeat: bool,
}

impl<'a, Message, Theme, Renderer> Keybind<'a, Message, Theme, Renderer> {
    /// Creates a new [`Keybind`] with the given [`Hotkey`] and target.
    pub fn new(
        hotkey: impl Into<Hotkey>,
        target: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            hotkey: hotkey.into(),
            target: target.into(),
            repeat: false,
        }
    }

    /// Sets whether the [`Keybind`] should trigger on repeated key press events.
    ///
    /// By default, this is disabled. This means that a user holding down the [`Hotkey`]
    /// will only trigger the [`Keybind`] once until any key is released.
    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Keybind<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let key_pressed = match &event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                modified_key,
                modifiers,
                location,
                repeat,
                ..
            }) => {
                modified_key.as_ref() == self.hotkey.key.as_ref()
                    && *modifiers == self.hotkey.modifiers
                    && self.hotkey.location.is_none_or(|hotkey_location| {
                        *location == hotkey_location
                    })
                    && (!repeat || self.repeat)
            }
            _ => false,
        };

        self.target.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );

        if shell.is_event_captured() || !key_pressed {
            return;
        }

        let click_events = [
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ];

        let mut local_messages = Vec::new();

        for event in click_events {
            let mut local_shell = Shell::new(&mut local_messages);

            self.target.as_widget_mut().update(
                tree,
                event,
                layout,
                mouse::Cursor::Available(layout.bounds().center()),
                renderer,
                clipboard,
                &mut local_shell,
                viewport,
            );

            shell.merge(local_shell, std::convert::identity);
        }
    }

    fn tag(&self) -> widget::tree::Tag {
        self.target.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.target.as_widget().state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.target.as_widget().children()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        self.target.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.target.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.target.as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.target.as_widget().layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.target
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.target
            .as_widget()
            .operate(tree, layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.target
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.target
            .as_widget_mut()
            .overlay(tree, layout, renderer, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<Keybind<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(keybind: Keybind<'a, Message, Theme, Renderer>) -> Self {
        Element::new(keybind)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::button;
    use crate::core::event;

    use iced_test::simulator;

    #[test]
    fn it_triggers_a_click() {
        let keybind: Keybind<'_, _, crate::Theme> =
            Keybind::new('s', button("Test!").on_press(42));

        let mut ui = simulator(keybind);
        let status = ui.tap_hotkey('s');

        assert_eq!(status, event::Status::Captured);
        assert_eq!(ui.into_messages().collect::<Vec<_>>(), vec![42]);
    }
}
