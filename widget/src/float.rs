//! Make elements float!
use crate::core;
use crate::core::border;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::tree;
use crate::core::{
    Element, Event, Layout, Length, Rectangle, Shadow, Shell, Size, Transformation, Vector, Widget,
};

/// A widget that can make its contents float over other widgets.
pub struct Float<'a, W, Theme = crate::Theme>
where
    Theme: Catalog,
{
    content: W,
    scale: f32,
    translate: Option<Box<dyn Fn(Rectangle, Rectangle) -> Vector + 'a>>,
    class: Theme::Class<'a>,
}

impl<'a, W, Theme> Float<'a, W, Theme>
where
    Theme: Catalog,
{
    /// Creates a new [`Float`] widget with the given content.
    pub fn new(content: W) -> Self {
        Self {
            content,
            scale: 1.0,
            translate: None,
            class: Theme::default(),
        }
    }

    /// Sets the scale to be applied to the contents of the [`Float`].
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the translation logic to be applied to the contents of the [`Float`].
    ///
    /// The logic takes the original (non-scaled) bounds of the contents and the
    /// viewport bounds. These bounds can be useful to ensure the floating elements
    /// always stay on screen.
    pub fn translate(mut self, translate: impl Fn(Rectangle, Rectangle) -> Vector + 'a) -> Self {
        self.translate = Some(Box::new(translate));
        self
    }

    /// Sets the style of the [`Float`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Float`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    fn is_floating(&self, bounds: Rectangle, viewport: Rectangle) -> bool {
        self.scale > 1.0
            || self
                .translate
                .as_ref()
                .is_some_and(|translate| translate(bounds, viewport) != Vector::ZERO)
    }
}

impl<W, Theme> widget::Node for Float<'_, W, Theme> where Theme: Catalog {}

impl<W, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Float<'_, W, Theme>
where
    Theme: Catalog,
    Renderer: core::Renderer,
    W: Widget<Message, Theme, Renderer>,
{
    fn tag(&self) -> tree::Tag {
        self.content.tag()
    }

    fn state(&self) -> tree::State {
        self.content.state()
    }

    fn diff(&mut self, tree: &mut widget::Tree) {
        self.content.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.size()
    }

    fn layout(&mut self, tree: &mut widget::Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.content.layout(tree, renderer, limits);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.is_floating(layout.bounds(), *viewport) {
            return;
        }

        self.content
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if self.is_floating(layout.bounds(), *viewport) {
            return;
        }

        {
            let style = theme.style(&self.class);

            if style.shadow.color.a > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds().shrink(1.0),
                        shadow: style.shadow,
                        border: border::rounded(style.shadow_border_radius),
                        snap: false,
                    },
                    style.shadow.color,
                );
            }
        }

        self.content
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.is_floating(layout.bounds(), *viewport) {
            return mouse::Interaction::None;
        }

        self.content
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .operate(tree, layout, viewport, renderer, operation);
    }

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut widget::Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        offset: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        let bounds = layout.bounds();

        let translation = self
            .translate
            .as_ref()
            .map(|translate| translate(bounds + offset, *viewport + offset))
            .unwrap_or(Vector::ZERO);

        if self.scale > 1.0 || translation != Vector::ZERO {
            let translation = translation + offset;

            let transformation = Transformation::translate(
                bounds.x + bounds.width / 2.0 + translation.x,
                bounds.y + bounds.height / 2.0 + translation.y,
            ) * Transformation::scale(self.scale)
                * Transformation::translate(
                    -bounds.x - bounds.width / 2.0,
                    -bounds.y - bounds.height / 2.0,
                );

            vec![overlay::Element::new(Box::new(Overlay {
                float: self,
                state,
                layout,
                viewport: *viewport + offset,
                window,
                transformation,
            }))]
        } else {
            self.content
                .overlay(state, layout, renderer, viewport, offset, window)
        }
    }
}

impl<'a, W, Message, Theme, Renderer> From<Float<'a, W, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a,
    W: Widget<Message, Theme, Renderer> + 'a,
{
    fn from(float: Float<'a, W, Theme>) -> Self {
        float.boxed()
    }
}

struct Overlay<'a, 'b, W, Theme>
where
    Theme: Catalog,
{
    float: &'a mut Float<'b, W, Theme>,
    state: &'a mut widget::Tree,
    layout: Layout,
    viewport: Rectangle,
    window: Size,
    transformation: Transformation,
}

impl<W, Message, Theme, Renderer> core::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, W, Theme>
where
    Theme: Catalog,
    Renderer: core::Renderer,
    W: Widget<Message, Theme, Renderer>,
{
    fn update(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let inverse = self.transformation.inverse();

        self.float.content.update(
            self.state,
            event,
            self.layout,
            cursor * inverse,
            renderer,
            shell,
            &(self.viewport * inverse),
        );
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        cursor: mouse::Cursor,
    ) {
        let bounds = self.layout.bounds();
        let inverse = self.transformation.inverse();

        renderer.with_layer(self.viewport, |renderer| {
            renderer.with_transformation(self.transformation, |renderer| {
                {
                    let style = theme.style(&self.float.class);

                    if style.shadow.color.a > 0.0 {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: bounds.shrink(1.0),
                                shadow: style.shadow,
                                border: border::rounded(style.shadow_border_radius),
                                snap: false,
                            },
                            style.shadow.color,
                        );
                    }
                }

                self.float.content.draw(
                    self.state,
                    renderer,
                    theme,
                    style,
                    self.layout,
                    cursor * inverse,
                    &(self.viewport * inverse),
                );
            });
        });
    }

    fn mouse_interaction(&self, cursor: mouse::Cursor, renderer: &Renderer) -> mouse::Interaction {
        let bounds = self.layout.bounds() * self.transformation;

        if !cursor.is_over(bounds) {
            return mouse::Interaction::None;
        }

        let inverse = self.transformation.inverse();

        self.float.content.mouse_interaction(
            self.state,
            self.layout,
            cursor * inverse,
            &(self.viewport * inverse),
            renderer,
        )
    }

    fn index(&self) -> f32 {
        self.float.scale * 0.5
    }

    fn overlay<'a>(
        &'a mut self,
        renderer: &Renderer,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        self.float.content.overlay(
            self.state,
            self.layout,
            renderer,
            &(self.viewport * self.transformation.inverse()),
            self.transformation.translation(),
            self.window,
        )
    }
}

/// The theme catalog of a [`Float`].
///
/// All themes that can be used with [`Float`]
/// must implement this trait.
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Float`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The style of a [`Float`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    /// The [`Shadow`] of the [`Float`].
    pub shadow: Shadow,
    /// The border radius of the shadow.
    pub shadow_border_radius: border::Radius,
}
