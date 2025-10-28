use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::theme;
use crate::core::widget;
use crate::core::widget::operation;
use crate::core::widget::tree;
use crate::core::{
    self, Clipboard, Color, Element, Event, Layout, Length, Point, Rectangle,
    Shell, Size, Vector, Widget,
};
use crate::test::Selector;
use crate::test::instruction::{Interaction, Mouse, Target};
use crate::test::selector;

pub fn recorder<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Recorder<'a, Message, Theme, Renderer> {
    Recorder::new(content)
}

pub struct Recorder<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_record: Option<Box<dyn Fn(Interaction) -> Message + 'a>>,
    has_overlay: bool,
}

impl<'a, Message, Theme, Renderer> Recorder<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            on_record: None,
            has_overlay: false,
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

struct State {
    last_hovered: Option<Rectangle>,
    last_hovered_overlay: Option<Rectangle>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Recorder<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
    Theme: theme::Base,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            last_hovered: None,
            last_hovered_overlay: None,
        })
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
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

        if !self.has_overlay
            && let Some(on_record) = &self.on_record
        {
            let state = tree.state.downcast_mut::<State>();

            record(
                event,
                cursor,
                shell,
                layout.bounds(),
                &mut state.last_hovered,
                on_record,
                |operation| {
                    self.content.as_widget_mut().operate(
                        &mut tree.children[0],
                        layout,
                        renderer,
                        operation,
                    );
                },
            );
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            limits,
        )
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        let state = tree.state.downcast_ref::<State>();

        let Some(last_hovered) = &state.last_hovered else {
            return;
        };

        renderer.with_layer(*viewport, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: *last_hovered,
                    ..renderer::Quad::default()
                },
                highlight(theme).scale_alpha(0.7),
            );
        });
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.has_overlay = false;

        self.content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout,
                renderer,
                &layout.bounds(),
                translation,
            )
            .map(|raw| {
                self.has_overlay = true;

                let state = tree.state.downcast_mut::<State>();

                overlay::Element::new(Box::new(Overlay {
                    raw,
                    bounds: layout.bounds(),
                    last_hovered: &mut state.last_hovered_overlay,
                    on_record: self.on_record.as_deref(),
                }))
            })
    }
}

impl<'a, Message, Theme, Renderer> From<Recorder<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: theme::Base + 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(recorder: Recorder<'a, Message, Theme, Renderer>) -> Self {
        Element::new(recorder)
    }
}

struct Overlay<'a, Message, Theme, Renderer> {
    raw: overlay::Element<'a, Message, Theme, Renderer>,
    bounds: Rectangle,
    last_hovered: &'a mut Option<Rectangle>,
    on_record: Option<&'a dyn Fn(Interaction) -> Message>,
}

impl<'a, Message, Theme, Renderer> core::Overlay<Message, Theme, Renderer>
    for Overlay<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer + 'a,
    Theme: theme::Base + 'a,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.raw.as_overlay_mut().layout(renderer, bounds)
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

        let Some(last_hovered) = &self.last_hovered else {
            return;
        };

        renderer.with_layer(self.bounds, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: *last_hovered,
                    ..renderer::Quad::default()
                },
                highlight(theme).scale_alpha(0.7),
            );
        });
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

        if let Some(on_event) = &self.on_record {
            record(
                event,
                cursor,
                shell,
                self.bounds,
                self.last_hovered,
                on_event,
                |operation| {
                    self.raw
                        .as_overlay_mut()
                        .operate(layout, renderer, operation);
                },
            );
        }

        self.raw
            .as_overlay_mut()
            .update(event, layout, cursor, renderer, clipboard, shell);
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
                    last_hovered: self.last_hovered,
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
    last_hovered: &mut Option<Rectangle>,
    on_record: impl Fn(Interaction) -> Message,
    operate: impl FnMut(&mut dyn widget::Operation),
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

    let Some(mut interaction) = interaction else {
        return;
    };

    let Interaction::Mouse(
        Mouse::Move(target)
        | Mouse::Press {
            target: Some(target),
            ..
        }
        | Mouse::Release {
            target: Some(target),
            ..
        }
        | Mouse::Click {
            target: Some(target),
            ..
        },
    ) = &mut interaction
    else {
        shell.publish(on_record(interaction));
        return;
    };

    let Target::Point(position) = *target else {
        shell.publish(on_record(interaction));
        return;
    };

    if let Some((content, visible_bounds)) =
        find_text(position + (bounds.position() - Point::ORIGIN), operate)
    {
        *target = Target::Text(content);
        *last_hovered = visible_bounds;
    } else {
        *last_hovered = None;
    }

    shell.publish(on_record(interaction));
}

fn find_text(
    position: Point,
    mut operate: impl FnMut(&mut dyn widget::Operation),
) -> Option<(String, Option<Rectangle>)> {
    use widget::Operation;

    let mut by_position = position.find_all();
    operate(&mut operation::black_box(&mut by_position));

    let operation::Outcome::Some(targets) = by_position.finish() else {
        return None;
    };

    let (content, visible_bounds) =
        targets.into_iter().rev().find_map(|target| {
            if let selector::Target::Text {
                content,
                visible_bounds,
                ..
            }
            | selector::Target::TextInput {
                content,
                visible_bounds,
                ..
            } = target
            {
                Some((content, visible_bounds))
            } else {
                None
            }
        })?;

    let mut by_text = content.clone().find_all();
    operate(&mut operation::black_box(&mut by_text));

    let operation::Outcome::Some(texts) = by_text.finish() else {
        return None;
    };

    if texts.len() > 1 {
        return None;
    }

    Some((content, visible_bounds))
}

fn highlight(theme: &impl theme::Base) -> Color {
    theme
        .palette()
        .map(|palette| palette.primary)
        .unwrap_or(Color::from_rgb(0.0, 0.0, 1.0))
}
