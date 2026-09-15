//! Integration tests for the [`UserInterface`].
use iced_runtime::core;
use iced_runtime::core::layout;
use iced_runtime::core::overlay;
use iced_runtime::core::renderer;
use iced_runtime::core::shell;
use iced_runtime::core::widget;
use iced_runtime::core::window;
use iced_runtime::core::{Element, Event, Length, Rectangle, Shell, Size};
use iced_runtime::user_interface::{Cache, UserInterface};

type Renderer = ();

struct Root {
    overlay_visible: bool,
}

impl widget::Widget<(), core::Theme, Renderer> for Root {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::ZERO)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        _renderer: &mut Renderer,
        _theme: &core::Theme,
        _style: &renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn update(
        &mut self,
        _tree: &mut widget::Tree,
        event: &Event,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, ()>,
        _viewport: &Rectangle,
    ) {
        if matches!(event, Event::Mouse(core::mouse::Event::ButtonReleased(_))) {
            shell.publish(());
        }
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut widget::Tree,
        _layout: layout::Layout<'a>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: core::Vector,
    ) -> Vec<overlay::Element<'a, (), core::Theme, Renderer>> {
        if self.overlay_visible {
            vec![overlay::Element::new(Box::new(HidingOverlay {
                overlay_visible: &mut self.overlay_visible,
            }))]
        } else {
            Vec::new()
        }
    }
}

struct HidingOverlay<'a> {
    overlay_visible: &'a mut bool,
}

impl<'a> overlay::Overlay<(), core::Theme, Renderer> for HidingOverlay<'a> {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(1.0, 1.0))
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &core::Theme,
        _style: &renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
    ) {
    }

    fn update(
        &mut self,
        event: &Event,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, ()>,
    ) {
        if matches!(event, Event::Mouse(core::mouse::Event::ButtonPressed(_))) {
            *self.overlay_visible = false;
            shell.invalidate_layout();
        }
    }
}

#[test]
fn events_after_an_overlay_disappears_reach_the_base_widget() {
    let mut renderer = ();

    let mut user_interface = UserInterface::build(
        Element::new(Root {
            overlay_visible: true,
        }),
        window::Settings::default().size,
        Cache::default(),
        &mut renderer,
    );

    let mut messages = shell::Bus::new();

    let events = [
        Event::Mouse(core::mouse::Event::ButtonPressed(core::mouse::Button::Left)),
        Event::Mouse(core::mouse::Event::ButtonReleased(
            core::mouse::Button::Left,
        )),
    ];

    let (_state, statuses) = user_interface.update(
        &window::Headless,
        &shell::Waker::noop(),
        &events,
        core::mouse::Cursor::Unavailable,
        &mut renderer,
        &mut messages,
    );

    assert_eq!(statuses.len(), 2);
    assert_eq!(messages.len(), 1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayMessage {
    Top,
    Bottom,
    Nested,
}

struct MultiOverlayRoot;

impl widget::Widget<OverlayMessage, core::Theme, Renderer> for MultiOverlayRoot {
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
        _theme: &core::Theme,
        _style: &renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn update(
        &mut self,
        _tree: &mut widget::Tree,
        _event: &Event,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
        _renderer: &Renderer,
        _shell: &mut Shell<'_, OverlayMessage>,
        _viewport: &Rectangle,
    ) {
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut widget::Tree,
        _layout: layout::Layout<'a>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: core::Vector,
    ) -> Vec<overlay::Element<'a, OverlayMessage, core::Theme, Renderer>> {
        vec![
            overlay::Element::new(Box::new(TopOverlay)),
            overlay::Element::new(Box::new(BottomOverlay)),
        ]
    }
}

/// The topmost overlay, covering `(60..80, 0..20)`.
struct TopOverlay;

impl overlay::Overlay<OverlayMessage, core::Theme, Renderer> for TopOverlay {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(20.0, 20.0)).translate(core::Vector::new(60.0, 0.0))
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &core::Theme,
        _style: &renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
    ) {
    }

    fn update(
        &mut self,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: core::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, OverlayMessage>,
    ) {
        if matches!(event, Event::Mouse(core::mouse::Event::ButtonPressed(_)))
            && cursor.position_over(layout.bounds()).is_some()
        {
            shell.publish(OverlayMessage::Top);
            shell.capture_event();
        }
    }

    fn index(&self) -> f32 {
        1.0
    }
}

/// The bottom overlay, covering `(0..40, 0..40)`, with a nested overlay
/// covering `(0..20, 0..20)`.
struct BottomOverlay;

impl overlay::Overlay<OverlayMessage, core::Theme, Renderer> for BottomOverlay {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(40.0, 40.0))
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &core::Theme,
        _style: &renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
    ) {
    }

    fn update(
        &mut self,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: core::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, OverlayMessage>,
    ) {
        if matches!(event, Event::Mouse(core::mouse::Event::ButtonPressed(_)))
            && cursor.position_over(layout.bounds()).is_some()
        {
            shell.publish(OverlayMessage::Bottom);
            shell.capture_event();
        }
    }

    fn index(&self) -> f32 {
        0.5
    }

    fn overlay<'b>(
        &'b mut self,
        _layout: layout::Layout<'b>,
        _renderer: &Renderer,
    ) -> Vec<overlay::Element<'b, OverlayMessage, core::Theme, Renderer>> {
        vec![overlay::Element::new(Box::new(NestedOverlay))]
    }
}

/// The nested overlay of [`BottomOverlay`], covering `(0..20, 0..20)`.
struct NestedOverlay;

impl overlay::Overlay<OverlayMessage, core::Theme, Renderer> for NestedOverlay {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(20.0, 20.0))
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &core::Theme,
        _style: &renderer::Style,
        _layout: layout::Layout<'_>,
        _cursor: core::mouse::Cursor,
    ) {
    }

    fn update(
        &mut self,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: core::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, OverlayMessage>,
    ) {
        if matches!(event, Event::Mouse(core::mouse::Event::ButtonPressed(_)))
            && cursor.position_over(layout.bounds()).is_some()
        {
            shell.publish(OverlayMessage::Nested);
            shell.capture_event();
        }
    }
}

#[test]
fn multiple_overlays_are_handled_by_index_order_and_nested_overlays_capture_events() {
    let mut renderer = ();

    let mut user_interface = UserInterface::build(
        Element::new(MultiOverlayRoot),
        Size::new(100.0, 100.0),
        Cache::default(),
        &mut renderer,
    );

    let mut messages = shell::Bus::new();
    let events = [Event::Mouse(core::mouse::Event::ButtonPressed(
        core::mouse::Button::Left,
    ))];

    // The nested overlay (index 0.9) captures events over `(0..20, 0..20)`
    let (_state, statuses) = user_interface.update(
        &window::Headless,
        &shell::Waker::noop(),
        &events,
        core::mouse::Cursor::Available(core::Point::new(10.0, 10.0)),
        &mut renderer,
        &mut messages,
    );
    assert_eq!(statuses, [core::event::Status::Captured]);
    assert_eq!(messages.len(), 1);

    // The bottom overlay (index 0.5) captures events outside its nested overlay
    let (_state, statuses) = user_interface.update(
        &window::Headless,
        &shell::Waker::noop(),
        &events,
        core::mouse::Cursor::Available(core::Point::new(30.0, 30.0)),
        &mut renderer,
        &mut messages,
    );
    assert_eq!(statuses, [core::event::Status::Captured]);
    assert_eq!(messages.len(), 2);

    // The top overlay (index 1.0) captures events over `(60..80, 0..20)`,
    // skipping the bottom overlay
    let (_state, statuses) = user_interface.update(
        &window::Headless,
        &shell::Waker::noop(),
        &events,
        core::mouse::Cursor::Available(core::Point::new(70.0, 10.0)),
        &mut renderer,
        &mut messages,
    );
    assert_eq!(statuses, [core::event::Status::Captured]);
    assert_eq!(messages.len(), 3);

    // Events not covered by any overlay reach the base widget
    let (_state, statuses) = user_interface.update(
        &window::Headless,
        &shell::Waker::noop(),
        &events,
        core::mouse::Cursor::Available(core::Point::new(90.0, 90.0)),
        &mut renderer,
        &mut messages,
    );
    assert_eq!(statuses, [core::event::Status::Ignored]);
    assert_eq!(messages.len(), 3);

    // The overlays can be drawn
    user_interface.draw(
        &mut renderer,
        &core::Theme::Light,
        &renderer::Style::default(),
        core::mouse::Cursor::Available(core::Point::new(10.0, 10.0)),
    );

    let published = messages
        .drain()
        .map(|(message, _)| message)
        .collect::<Vec<_>>();

    assert_eq!(
        published,
        [
            OverlayMessage::Nested,
            OverlayMessage::Bottom,
            OverlayMessage::Top,
        ]
    );
}
