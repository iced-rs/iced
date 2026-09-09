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
