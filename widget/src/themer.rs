use crate::container;
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::Operation;
use crate::core::{
    Background, Clipboard, Element, Layout, Length, Point, Rectangle, Shell,
    Size, Vector, Widget,
};
use crate::style::application;

/// A widget that applies any `Theme` to its contents.
///
/// This widget can be useful to leverage multiple `Theme`
/// types in an application.
#[allow(missing_debug_implementations)]
pub struct Themer<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: application::StyleSheet,
{
    content: Element<'a, Message, Theme, Renderer>,
    theme: Theme,
    style: Theme::Style,
    show_background: bool,
}

impl<'a, Message, Theme, Renderer> Themer<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: application::StyleSheet,
{
    /// Creates an empty [`Themer`] that applies the given `Theme`
    /// to the provided `content`.
    pub fn new<T>(theme: Theme, content: T) -> Self
    where
        T: Into<Element<'a, Message, Theme, Renderer>>,
    {
        Self {
            content: content.into(),
            theme,
            style: Theme::Style::default(),
            show_background: false,
        }
    }

    /// Sets whether to draw the background color of the `Theme`.
    pub fn background(mut self, background: bool) -> Self {
        self.show_background = background;
        self
    }
}

impl<'a, AnyTheme, Message, Theme, Renderer> Widget<Message, AnyTheme, Renderer>
    for Themer<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
    Theme: application::StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget().layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        self.content
            .as_widget()
            .operate(tree, layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &AnyTheme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let appearance = self.theme.appearance(&self.style);

        if self.show_background {
            container::draw_background(
                renderer,
                &container::Appearance {
                    background: Some(Background::Color(
                        appearance.background_color,
                    )),
                    ..container::Appearance::default()
                },
                layout.bounds(),
            );
        }

        self.content.as_widget().draw(
            tree,
            renderer,
            &self.theme,
            &renderer::Style {
                text_color: appearance.text_color,
            },
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, AnyTheme, Renderer>> {
        struct Overlay<'a, Message, Theme, Renderer> {
            theme: &'a Theme,
            content: overlay::Element<'a, Message, Theme, Renderer>,
        }

        impl<'a, AnyTheme, Message, Theme, Renderer>
            overlay::Overlay<Message, AnyTheme, Renderer>
            for Overlay<'a, Message, Theme, Renderer>
        where
            Renderer: crate::core::Renderer,
        {
            fn layout(
                &mut self,
                renderer: &Renderer,
                bounds: Size,
            ) -> layout::Node {
                self.content.layout(renderer, bounds)
            }

            fn draw(
                &self,
                renderer: &mut Renderer,
                _theme: &AnyTheme,
                style: &renderer::Style,
                layout: Layout<'_>,
                cursor: mouse::Cursor,
            ) {
                self.content
                    .draw(renderer, self.theme, style, layout, cursor);
            }

            fn on_event(
                &mut self,
                event: Event,
                layout: Layout<'_>,
                cursor: mouse::Cursor,
                renderer: &Renderer,
                clipboard: &mut dyn Clipboard,
                shell: &mut Shell<'_, Message>,
            ) -> event::Status {
                self.content
                    .on_event(event, layout, cursor, renderer, clipboard, shell)
            }

            fn operate(
                &mut self,
                layout: Layout<'_>,
                renderer: &Renderer,
                operation: &mut dyn Operation<Message>,
            ) {
                self.content.operate(layout, renderer, operation);
            }

            fn mouse_interaction(
                &self,
                layout: Layout<'_>,
                cursor: mouse::Cursor,
                viewport: &Rectangle,
                renderer: &Renderer,
            ) -> mouse::Interaction {
                self.content
                    .mouse_interaction(layout, cursor, viewport, renderer)
            }

            fn is_over(
                &self,
                layout: Layout<'_>,
                renderer: &Renderer,
                cursor_position: Point,
            ) -> bool {
                self.content.is_over(layout, renderer, cursor_position)
            }

            fn overlay<'b>(
                &'b mut self,
                layout: Layout<'_>,
                renderer: &Renderer,
            ) -> Option<overlay::Element<'b, Message, AnyTheme, Renderer>>
            {
                self.content
                    .overlay(layout, renderer)
                    .map(|content| Overlay {
                        theme: self.theme,
                        content,
                    })
                    .map(|overlay| overlay::Element::new(Box::new(overlay)))
            }
        }

        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, translation)
            .map(|content| Overlay {
                theme: &self.theme,
                content,
            })
            .map(|overlay| overlay::Element::new(Box::new(overlay)))
    }
}

impl<'a, AnyTheme, Message, Theme, Renderer>
    From<Themer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, AnyTheme, Renderer>
where
    Message: 'a,
    Theme: 'a + application::StyleSheet,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(
        themer: Themer<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, AnyTheme, Renderer> {
        Element::new(themer)
    }
}
