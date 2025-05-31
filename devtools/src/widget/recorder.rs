use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree;
use crate::core::{
    self, Clipboard, Element, Event, Layout, Length, Point, Rectangle, Shell,
    Size, Widget,
};

#[allow(missing_debug_implementations)]
pub struct Recorder<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_event: Option<Box<dyn Fn(Event) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer> Recorder<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            on_event: None,
        }
    }

    pub fn on_event(
        mut self,
        on_event: impl Fn(Event) -> Message + 'a,
    ) -> Self {
        self.on_event = Some(Box::new(on_event));
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Recorder<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn update(
        &mut self,
        state: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if shell.is_event_captured() {
            return;
        }

        self.content.as_widget_mut().update(
            state, event, layout, cursor, renderer, clipboard, shell, viewport,
        );

        if let Some(on_event) = &self.on_event {
            match event {
                Event::Mouse(event) => {
                    if !cursor.is_over(layout.bounds()) {
                        return;
                    }

                    match event {
                        mouse::Event::ButtonPressed(_)
                        | mouse::Event::ButtonReleased(_)
                        | mouse::Event::WheelScrolled { .. } => {
                            shell.publish(on_event(Event::Mouse(*event)));
                        }
                        mouse::Event::CursorMoved { position } => {
                            shell.publish(on_event(Event::Mouse(
                                mouse::Event::CursorMoved {
                                    position: *position
                                        - (layout.bounds().position()
                                            - Point::ORIGIN),
                                },
                            )));
                        }
                        _ => {}
                    }
                }
                Event::Keyboard(event) => {
                    shell.publish(on_event(Event::Keyboard(event.clone())));
                }
                _ => {}
            }
        }
    }

    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget().layout(tree, renderer, limits)
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
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        state: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }

    fn operate(
        &self,
        state: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget()
            .operate(state, layout, renderer, operation);
    }
}

impl<'a, Message, Theme, Renderer> From<Recorder<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(recorder: Recorder<'a, Message, Theme, Renderer>) -> Self {
        Element::new(recorder)
    }
}
