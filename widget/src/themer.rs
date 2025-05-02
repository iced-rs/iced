use crate::container;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Background, Clipboard, Color, Element, Event, Layout, Length, Rectangle,
    Shell, Size, Vector, Widget,
};

use std::marker::PhantomData;

/// A widget that applies any `Theme` to its contents.
///
/// This widget can be useful to leverage multiple `Theme`
/// types in an application.
#[allow(missing_debug_implementations)]
pub struct Themer<'a, Message, Theme, NewTheme, F, Renderer = crate::Renderer>
where
    F: Fn(&Theme) -> NewTheme,
    Renderer: crate::core::Renderer,
{
    content: Element<'a, Message, NewTheme, Renderer>,
    to_theme: F,
    text_color: Option<fn(&NewTheme) -> Color>,
    background: Option<fn(&NewTheme) -> Background>,
    old_theme: PhantomData<Theme>,
}

impl<'a, Message, Theme, NewTheme, F, Renderer>
    Themer<'a, Message, Theme, NewTheme, F, Renderer>
where
    F: Fn(&Theme) -> NewTheme,
    Renderer: crate::core::Renderer,
{
    /// Creates an empty [`Themer`] that applies the given `Theme`
    /// to the provided `content`.
    pub fn new<T>(to_theme: F, content: T) -> Self
    where
        T: Into<Element<'a, Message, NewTheme, Renderer>>,
    {
        Self {
            content: content.into(),
            to_theme,
            text_color: None,
            background: None,
            old_theme: PhantomData,
        }
    }

    /// Sets the default text [`Color`] of the [`Themer`].
    pub fn text_color(mut self, f: fn(&NewTheme) -> Color) -> Self {
        self.text_color = Some(f);
        self
    }

    /// Sets the [`Background`] of the [`Themer`].
    pub fn background(mut self, f: fn(&NewTheme) -> Background) -> Self {
        self.background = Some(f);
        self
    }
}

impl<Message, Theme, NewTheme, F, Renderer> Widget<Message, Theme, Renderer>
    for Themer<'_, Message, Theme, NewTheme, F, Renderer>
where
    F: Fn(&Theme) -> NewTheme,
    Renderer: crate::core::Renderer,
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
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
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
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let theme = (self.to_theme)(theme);

        if let Some(background) = self.background {
            container::draw_background(
                renderer,
                &container::Style {
                    background: Some(background(&theme)),
                    ..container::Style::default()
                },
                layout.bounds(),
            );
        }

        let style = if let Some(text_color) = self.text_color {
            renderer::Style {
                text_color: text_color(&theme),
            }
        } else {
            *style
        };

        self.content
            .as_widget()
            .draw(tree, renderer, &theme, &style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        struct Overlay<'a, Message, Theme, NewTheme, Renderer> {
            to_theme: &'a dyn Fn(&Theme) -> NewTheme,
            content: overlay::Element<'a, Message, NewTheme, Renderer>,
        }

        impl<Message, Theme, NewTheme, Renderer>
            overlay::Overlay<Message, Theme, Renderer>
            for Overlay<'_, Message, Theme, NewTheme, Renderer>
        where
            Renderer: crate::core::Renderer,
        {
            fn layout(
                &mut self,
                renderer: &Renderer,
                bounds: Size,
            ) -> layout::Node {
                self.content.as_overlay_mut().layout(renderer, bounds)
            }

            fn draw(
                &self,
                renderer: &mut Renderer,
                theme: &Theme,
                style: &renderer::Style,
                layout: Layout<'_>,
                cursor: mouse::Cursor,
            ) {
                self.content.as_overlay().draw(
                    renderer,
                    &(self.to_theme)(theme),
                    style,
                    layout,
                    cursor,
                );
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
                self.content
                    .as_overlay_mut()
                    .update(event, layout, cursor, renderer, clipboard, shell);
            }

            fn operate(
                &mut self,
                layout: Layout<'_>,
                renderer: &Renderer,
                operation: &mut dyn Operation,
            ) {
                self.content
                    .as_overlay_mut()
                    .operate(layout, renderer, operation);
            }

            fn mouse_interaction(
                &self,
                layout: Layout<'_>,
                cursor: mouse::Cursor,
                renderer: &Renderer,
            ) -> mouse::Interaction {
                self.content
                    .as_overlay()
                    .mouse_interaction(layout, cursor, renderer)
            }

            fn overlay<'b>(
                &'b mut self,
                layout: Layout<'_>,
                renderer: &Renderer,
            ) -> Option<overlay::Element<'b, Message, Theme, Renderer>>
            {
                self.content
                    .as_overlay_mut()
                    .overlay(layout, renderer)
                    .map(|content| Overlay {
                        to_theme: &self.to_theme,
                        content,
                    })
                    .map(|overlay| overlay::Element::new(Box::new(overlay)))
            }
        }

        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
            .map(|content| Overlay {
                to_theme: &self.to_theme,
                content,
            })
            .map(|overlay| overlay::Element::new(Box::new(overlay)))
    }
}

impl<'a, Message, Theme, NewTheme, F, Renderer>
    From<Themer<'a, Message, Theme, NewTheme, F, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    NewTheme: 'a,
    F: Fn(&Theme) -> NewTheme + 'a,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(
        themer: Themer<'a, Message, Theme, NewTheme, F, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(themer)
    }
}
