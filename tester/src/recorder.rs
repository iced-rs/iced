use std::cell::Cell;

use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::theme;
use crate::core::widget;
use crate::core::widget::operation;
use crate::core::widget::tree;
use crate::core::{
    self, Color, Element, Event, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
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
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_record: None,
            has_overlay: false,
        }
    }

    pub fn on_record(mut self, on_record: impl Fn(Interaction) -> Message + 'a) -> Self {
        self.on_record = Some(Box::new(on_record));
        self
    }
}

struct State {
    last_hovered: Cell<Option<Rectangle>>,
    last_hovered_overlay: Cell<Option<Rectangle>>,
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
            last_hovered: Cell::new(None),
            last_hovered_overlay: Cell::new(None),
        })
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
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
                &state.last_hovered,
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
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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

        let Some(last_hovered) = state.last_hovered.get() else {
            return;
        };

        renderer.with_layer(*viewport, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: last_hovered,
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
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        let raw_overlays = self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            &layout.bounds(),
            translation,
        );

        self.has_overlay = !raw_overlays.is_empty();

        if !self.has_overlay {
            return Vec::new();
        }

        // Each overlay gets its own wrapper so that the runtime can still
        // z-order them by index. The `Cell` below lets all of the wrappers
        // share the recorder's "last hovered" highlight.
        let state = tree.state.downcast_mut::<State>();

        let mut overlays = Vec::with_capacity(raw_overlays.len());

        for raw in raw_overlays {
            overlays.push(overlay::Element::new(Box::new(Overlay {
                raw,
                bounds: layout.bounds(),
                last_hovered: &state.last_hovered_overlay,
                on_record: self.on_record.as_deref(),
            })));
        }

        overlays
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
    last_hovered: &'a Cell<Option<Rectangle>>,
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

    fn index(&self) -> f32 {
        self.raw.as_overlay().index()
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

        let Some(last_hovered) = self.last_hovered.get() else {
            return;
        };

        renderer.with_layer(self.bounds, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: last_hovered,
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
            .update(event, layout, cursor, renderer, shell);
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
}

fn record<Message>(
    event: &Event,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    bounds: Rectangle,
    last_hovered: &Cell<Option<Rectangle>>,
    on_record: impl Fn(Interaction) -> Message,
    operate: impl FnMut(&mut dyn widget::Operation),
) {
    if let Event::Mouse(_) = event
        && !cursor.is_over(bounds)
    {
        return;
    }

    let interaction = if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
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
        last_hovered.set(visible_bounds);
    } else {
        last_hovered.set(None);
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

    let (content, visible_bounds) = targets.into_iter().rev().find_map(|target| {
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
        .seed()
        .map(|seed| seed.primary)
        .unwrap_or(Color::from_rgb(0.0, 0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::image;
    use crate::core::{Background, Transformation};

    /// An overlay with a fixed index.
    struct ByIndex(f32);

    impl<Message, Theme, Renderer> core::Overlay<Message, Theme, Renderer> for ByIndex
    where
        Renderer: core::Renderer,
    {
        fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
            layout::Node::new(Size::ZERO)
        }

        fn draw(
            &self,
            _renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            _layout: Layout<'_>,
            _cursor: mouse::Cursor,
        ) {
        }

        fn index(&self) -> f32 {
            self.0
        }
    }

    /// A widget that produces two overlays.
    struct TwoOverlays;

    impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for TwoOverlays
    where
        Renderer: core::Renderer,
    {
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fill, Length::Fill)
        }

        fn layout(
            &mut self,
            _tree: &mut widget::Tree,
            _renderer: &Renderer,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(100.0, 100.0))
        }

        fn draw(
            &self,
            _tree: &widget::Tree,
            _renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            _layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
        }

        fn overlay<'a>(
            &'a mut self,
            _tree: &'a mut widget::Tree,
            _layout: Layout<'a>,
            _renderer: &Renderer,
            _viewport: &Rectangle,
            _translation: Vector,
        ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
            vec![
                overlay::Element::new(Box::new(ByIndex(1.0))),
                overlay::Element::new(Box::new(ByIndex(2.0))),
            ]
        }
    }

    /// A minimal renderer that does nothing.
    struct Dummy;

    impl core::Renderer for Dummy {
        fn start_layer(&mut self, _bounds: Rectangle) {}

        fn end_layer(&mut self) {}

        fn start_transformation(&mut self, _transformation: Transformation) {}

        fn end_transformation(&mut self) {}

        fn fill_quad(&mut self, _quad: renderer::Quad, _background: impl Into<Background>) {}

        fn allocate_image(
            &self,
            _handle: &image::Handle,
            _callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
        ) {
        }

        fn hint(&mut self, _scale: renderer::Scale) {}

        fn scale(&self) -> Option<renderer::Scale> {
            None
        }

        fn reset(&mut self, _new_bounds: Rectangle) {}

        fn settings(&self) -> renderer::Settings {
            renderer::Settings::default()
        }
    }

    #[test]
    fn recorder_wraps_all_overlays() {
        let mut recorder: Recorder<'_, (), iced_widget::Theme, Dummy> =
            Recorder::new(Element::new(TwoOverlays));

        let mut tree = widget::Tree::new(&recorder as &dyn Widget<(), iced_widget::Theme, Dummy>);
        recorder.diff(&mut tree);

        let node = layout::Node::new(Size::new(100.0, 100.0));
        let layout = Layout::with_offset(Vector::ZERO, &node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0));

        let overlays = recorder.overlay(&mut tree, layout, &Dummy, &viewport, Vector::ZERO);

        // Both overlays are wrapped, each keeping its own index.
        assert_eq!(overlays.len(), 2);
        assert_eq!(overlays[0].as_overlay().index(), 1.0);
        assert_eq!(overlays[1].as_overlay().index(), 2.0);
    }
}
