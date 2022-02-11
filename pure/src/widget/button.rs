use crate::widget::{Element, Tree, Widget};

use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::renderer;
use iced_native::touch;
use iced_native::{
    Background, Clipboard, Color, Hasher, Layout, Length, Padding, Point,
    Rectangle, Shell, Vector,
};
use iced_style::button::StyleSheet;

use std::any::Any;

pub struct Button<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    style_sheet: Box<dyn StyleSheet + 'a>,
    width: Length,
    height: Length,
    padding: Padding,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Button {
            content: content.into(),
            on_press: None,
            style_sheet: Default::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5),
        }
    }

    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Message: 'static + Clone,
    Renderer: 'static + iced_native::Renderer,
{
    fn tag(&self) -> std::any::TypeId {
        std::any::TypeId::of::<State>()
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(State { is_pressed: false })
    }

    fn children(&self) -> &[Element<Message, Renderer>] {
        std::slice::from_ref(&self.content)
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        self.tag().hash(state);
        self.width.hash(state);
        self.content.as_widget().hash_layout(state);
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height)
            .pad(self.padding);

        let mut content = self.content.as_widget().layout(renderer, &limits);
        content.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        let size = limits.resolve(content.size()).pad(self.padding);

        layout::Node::with_children(size, vec![content])
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let state = if let Some(state) = tree.state.downcast_mut::<State>() {
            state
        } else {
            return event::Status::Ignored;
        };

        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        state.is_pressed = true;

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = self.on_press.clone() {
                    let bounds = layout.bounds();

                    if state.is_pressed {
                        state.is_pressed = false;

                        if bounds.contains(cursor_position) {
                            shell.publish(on_press);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                state.is_pressed = false;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let state = if let Some(state) = tree.state.downcast_ref::<State>() {
            state
        } else {
            return;
        };

        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let is_mouse_over = bounds.contains(cursor_position);
        let is_disabled = self.on_press.is_none();

        let styling = if is_disabled {
            self.style_sheet.disabled()
        } else if is_mouse_over {
            if state.is_pressed {
                self.style_sheet.pressed()
            } else {
                self.style_sheet.hovered()
            }
        } else {
            self.style_sheet.active()
        };

        if styling.background.is_some() || styling.border_width > 0.0 {
            if styling.shadow_offset != Vector::default() {
                // TODO: Implement proper shadow support
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + styling.shadow_offset.x,
                            y: bounds.y + styling.shadow_offset.y,
                            ..bounds
                        },
                        border_radius: styling.border_radius,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    Background::Color([0.0, 0.0, 0.0, 0.5].into()),
                );
            }

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: styling.border_radius,
                    border_width: styling.border_width,
                    border_color: styling.border_color,
                },
                styling
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            &renderer::Style {
                text_color: styling.text_color,
            },
            content_layout,
            cursor_position,
            &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = layout.bounds().contains(cursor_position);
        let is_disabled = self.on_press.is_none();

        if is_mouse_over && !is_disabled {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    is_pressed: bool,
}

impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for Button<'a, Message, Renderer>
where
    Message: Clone + 'static,
    Renderer: iced_native::Renderer + 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
