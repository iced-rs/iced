use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree;
use crate::core::{
    self, Clipboard, Element, Event, Layout, Length, Point, Rectangle, Shell,
    Size, Vector, Widget,
};
use crate::test::instruction::Interaction;

pub fn recorder<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Recorder<'a, Message, Theme, Renderer> {
    Recorder::new(content)
}

#[allow(missing_debug_implementations)]
pub struct Recorder<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_record: Option<Box<dyn Fn(Interaction) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer> Recorder<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            on_record: None,
        }
    }

    pub fn on_record(
        mut self,
        on_record: impl Fn(Interaction) -> Message + 'a,
    ) -> Self {
        self.on_record = Some(Box::new(on_record));
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

        if let Some(on_record) = &self.on_record {
            record(event, cursor, shell, layout.bounds(), on_record);
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

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(state, layout, renderer, &layout.bounds(), translation)
            .map(|raw| {
                overlay::Element::new(Box::new(Overlay {
                    raw,
                    bounds: layout.bounds(),
                    on_record: self.on_record.as_deref(),
                }))
            })
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

struct Overlay<'a, Message, Theme, Renderer> {
    raw: overlay::Element<'a, Message, Theme, Renderer>,
    bounds: Rectangle,
    on_record: Option<&'a dyn Fn(Interaction) -> Message>,
}

impl<'a, Message, Theme, Renderer> core::Overlay<Message, Theme, Renderer>
    for Overlay<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer + 'a,
{
    fn layout(&mut self, renderer: &Renderer, _bounds: Size) -> layout::Node {
        self.raw
            .as_overlay_mut()
            .layout(renderer, self.bounds.size())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.raw
            .as_overlay()
            .draw(renderer, theme, style, layout, cursor);
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.raw
            .as_overlay_mut()
            .operate(layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if shell.is_event_captured() {
            return;
        }

        self.raw
            .as_overlay_mut()
            .update(event, layout, cursor, renderer, clipboard, shell);

        if let Some(on_event) = &self.on_record {
            record(event, cursor, shell, self.bounds, on_event);
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.raw
            .as_overlay()
            .mouse_interaction(layout, cursor, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        layout: Layout<'b>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.raw
            .as_overlay_mut()
            .overlay(layout, renderer)
            .map(|raw| {
                overlay::Element::new(Box::new(Overlay {
                    raw,
                    bounds: self.bounds,
                    on_record: self.on_record,
                }))
            })
    }

    fn index(&self) -> f32 {
        self.raw.as_overlay().index()
    }
}

fn record<Message>(
    event: &Event,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    bounds: Rectangle,
    on_record: impl Fn(Interaction) -> Message,
) {
    if let Event::Mouse(_) = event
        && !cursor.is_over(bounds)
    {
        return;
    }

    let interaction =
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            Interaction::from_event(&Event::Mouse(mouse::Event::CursorMoved {
                position: *position - (bounds.position() - Point::ORIGIN),
            }))
        } else {
            Interaction::from_event(event)
        };

    if let Some(interaction) = interaction {
        shell.publish(on_record(interaction));
    }
}
