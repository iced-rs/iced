//! Images display raster graphics in different formats (PNG, JPG, etc.).
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::image;
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     image("ferris.png").into()
//! }
//! ```
//! <img src="https://github.com/iced-rs/iced/blob/9712b319bb7a32848001b96bd84977430f14b623/examples/resources/ferris.png?raw=true" width="300">
pub mod viewer;
pub use viewer::Viewer;

use crate::core;
use crate::core::border;
use crate::core::image;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    ContentFit, Element, Layout, Length, Point, Rectangle, Rotation, Shadow,
    Size, Vector, Widget,
};

pub use image::{FilterMethod, Handle};

/// Creates a new [`Viewer`] with the given image `Handle`.
pub fn viewer<Handle>(handle: Handle) -> Viewer<Handle> {
    Viewer::new(handle)
}

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::image;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     image("ferris.png").into()
/// }
/// ```
/// <img src="https://github.com/iced-rs/iced/blob/9712b319bb7a32848001b96bd84977430f14b623/examples/resources/ferris.png?raw=true" width="300">
#[derive(Debug)]
pub struct Image<'a, Handle = image::Handle, Theme = crate::Theme>
where
    Theme: Catalog,
{
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    filter_method: FilterMethod,
    rotation: Rotation,
    opacity: f32,
    scale: f32,
    float: bool,
    class: Theme::Class<'a>,
}

impl<'a, Handle, Theme> Image<'a, Handle, Theme>
where
    Theme: Catalog,
{
    /// Creates a new [`Image`] with the given path.
    pub fn new(handle: impl Into<Handle>) -> Self {
        Image {
            handle: handle.into(),
            width: Length::Shrink,
            height: Length::Shrink,
            content_fit: ContentFit::default(),
            filter_method: FilterMethod::default(),
            rotation: Rotation::default(),
            opacity: 1.0,
            scale: 1.0,
            float: false,
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`Image`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Image`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`ContentFit`] of the [`Image`].
    ///
    /// Defaults to [`ContentFit::Contain`]
    pub fn content_fit(mut self, content_fit: ContentFit) -> Self {
        self.content_fit = content_fit;
        self
    }

    /// Sets the [`FilterMethod`] of the [`Image`].
    pub fn filter_method(mut self, filter_method: FilterMethod) -> Self {
        self.filter_method = filter_method;
        self
    }

    /// Applies the given [`Rotation`] to the [`Image`].
    pub fn rotation(mut self, rotation: impl Into<Rotation>) -> Self {
        self.rotation = rotation.into();
        self
    }

    /// Sets the opacity of the [`Image`].
    ///
    /// It should be in the [0.0, 1.0] range—`0.0` meaning completely transparent,
    /// and `1.0` meaning completely opaque.
    pub fn opacity(mut self, opacity: impl Into<f32>) -> Self {
        self.opacity = opacity.into();
        self
    }

    /// Sets the scale of the [`Image`].
    ///
    /// The region of the [`Image`] drawn will be scaled from the center by the given scale factor.
    /// This can be useful to create certain effects and animations, like smooth zoom in / out.
    pub fn scale(mut self, scale: impl Into<f32>) -> Self {
        self.scale = scale.into();
        self
    }

    /// Sets whether an [`Image`] should float above other content when scaled up.
    ///
    /// By default, an [`Image`] has this flag set to `false`; meaning it
    /// will be clipped or "framed" inside its bounds when scaled.
    ///
    /// Enabling this flag is useful to create cool hover effects!
    pub fn float(mut self, float: bool) -> Self {
        self.float = float;
        self
    }

    /// Sets the style of the [`Image`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Image`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// Computes the layout of an [`Image`].
pub fn layout<Renderer, Handle>(
    renderer: &Renderer,
    limits: &layout::Limits,
    handle: &Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    rotation: Rotation,
) -> layout::Node
where
    Renderer: image::Renderer<Handle = Handle>,
{
    // The raw w/h of the underlying image
    let image_size = renderer.measure_image(handle);
    let image_size =
        Size::new(image_size.width as f32, image_size.height as f32);

    // The rotated size of the image
    let rotated_size = rotation.apply(image_size);

    // The size to be available to the widget prior to `Shrink`ing
    let raw_size = limits.resolve(width, height, rotated_size);

    // The uncropped size of the image when fit to the bounds above
    let full_size = content_fit.fit(rotated_size, raw_size);

    // Shrink the widget to fit the resized image, if requested
    let final_size = Size {
        width: match width {
            Length::Shrink => f32::min(raw_size.width, full_size.width),
            _ => raw_size.width,
        },
        height: match height {
            Length::Shrink => f32::min(raw_size.height, full_size.height),
            _ => raw_size.height,
        },
    };

    layout::Node::new(final_size)
}

fn drawing_bounds<Renderer, Handle>(
    renderer: &Renderer,
    bounds: Rectangle,
    handle: &Handle,
    content_fit: ContentFit,
    rotation: Rotation,
    scale: f32,
) -> Rectangle
where
    Renderer: image::Renderer<Handle = Handle>,
{
    let Size { width, height } = renderer.measure_image(handle);
    let image_size = Size::new(width as f32, height as f32);
    let rotated_size = rotation.apply(image_size);
    let adjusted_fit = content_fit.fit(rotated_size, bounds.size());

    let fit_scale = Vector::new(
        adjusted_fit.width / rotated_size.width,
        adjusted_fit.height / rotated_size.height,
    );

    let final_size = image_size * fit_scale * scale;

    let position = match content_fit {
        ContentFit::None => Point::new(
            bounds.x + (rotated_size.width - adjusted_fit.width) / 2.0,
            bounds.y + (rotated_size.height - adjusted_fit.height) / 2.0,
        ),
        _ => Point::new(
            bounds.center_x() - final_size.width / 2.0,
            bounds.center_y() - final_size.height / 2.0,
        ),
    };

    Rectangle::new(position, final_size)
}

fn must_clip(bounds: Rectangle, drawing_bounds: Rectangle) -> bool {
    drawing_bounds.width > bounds.width || drawing_bounds.height > bounds.height
}

/// Draws an [`Image`]
pub fn draw<Renderer, Handle>(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    viewport: &Rectangle,
    handle: &Handle,
    content_fit: ContentFit,
    filter_method: FilterMethod,
    rotation: Rotation,
    opacity: f32,
    scale: f32,
    float: bool,
    style: Style,
) where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone,
{
    let bounds = layout.bounds();
    let drawing_bounds =
        drawing_bounds(renderer, bounds, handle, content_fit, rotation, scale);

    if must_clip(bounds, drawing_bounds) {
        if scale > 1.0 && float {
            return;
        }

        if let Some(bounds) = bounds.intersection(viewport) {
            renderer.with_layer(bounds, |renderer| {
                render(
                    renderer,
                    handle,
                    filter_method,
                    rotation,
                    opacity,
                    drawing_bounds,
                );
            });
        }
    } else {
        render(
            renderer,
            handle,
            filter_method,
            rotation,
            opacity,
            drawing_bounds,
        );
    }

    if style.shadow.color.a > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds: bounds.shrink(1.0),
                shadow: style.shadow,
                border: border::rounded(style.shadow_border_radius),
            },
            style.shadow.color,
        );
    }
}

fn render<Renderer, Handle>(
    renderer: &mut Renderer,
    handle: &Handle,
    filter_method: FilterMethod,
    rotation: Rotation,
    opacity: f32,
    drawing_bounds: Rectangle,
) where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone,
{
    renderer.draw_image(
        image::Image {
            handle: handle.clone(),
            filter_method,
            rotation: rotation.radians(),
            opacity,
            snap: true,
        },
        drawing_bounds,
    );
}

impl<Message, Theme, Renderer, Handle> Widget<Message, Theme, Renderer>
    for Image<'_, Handle, Theme>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            &self.handle,
            self.width,
            self.height,
            self.content_fit,
            self.rotation,
        )
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        draw(
            renderer,
            layout,
            viewport,
            &self.handle,
            self.content_fit,
            self.filter_method,
            self.rotation,
            self.opacity,
            self.scale,
            self.float,
            theme.style(&self.class),
        );
    }

    fn overlay<'a>(
        &'a mut self,
        _state: &'a mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        if !self.float || self.scale <= 1.0 {
            return None;
        }

        let bounds = layout.bounds() + translation;
        let drawing_bounds = drawing_bounds(
            renderer,
            bounds,
            &self.handle,
            self.content_fit,
            self.rotation,
            self.scale,
        );

        if must_clip(bounds, drawing_bounds) {
            Some(overlay::Element::new(Box::new(Overlay {
                image: self,
                clip_bounds: bounds,
                drawing_bounds,
            })))
        } else {
            None
        }
    }
}

impl<'a, Message, Theme, Renderer, Handle> From<Image<'a, Handle, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone + 'a,
    Theme: Catalog + 'a,
{
    fn from(
        image: Image<'a, Handle, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(image)
    }
}

/// The theme catalog of an [`Image`].
///
/// All themes that can be used with [`Image`]
/// must implement this trait.
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Button`].
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

/// The style of an [`Image`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    /// The [`Shadow`] of the [`Image`].
    pub shadow: Shadow,
    /// The border radius of the shadow.
    pub shadow_border_radius: border::Radius,
}

struct Overlay<'a, 'b, Handle, Theme>
where
    Theme: Catalog,
{
    image: &'a Image<'b, Handle, Theme>,
    clip_bounds: Rectangle,
    drawing_bounds: Rectangle,
}

impl<Message, Theme, Renderer, Handle> core::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Handle, Theme>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone,
    Theme: Catalog,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(self.clip_bounds.size())
            .move_to(self.clip_bounds.position())
    }

    fn is_over(
        &self,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _cursor_position: Point,
    ) -> bool {
        false
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let clip_bounds = Rectangle {
            x: self.clip_bounds.x
                - (self.clip_bounds.width * self.image.scale
                    - self.clip_bounds.width)
                    / 2.0,
            y: self.clip_bounds.y
                - (self.clip_bounds.height * self.image.scale
                    - self.clip_bounds.height)
                    / 2.0,
            width: self.clip_bounds.width * self.image.scale,
            height: self.clip_bounds.height * self.image.scale,
        };

        let style = theme.style(&self.image.class);

        if style.shadow.color.a > 0.0 {
            renderer.with_layer(
                clip_bounds.expand(style.shadow.blur_radius),
                |renderer| {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: clip_bounds.shrink(1.0),
                            shadow: style.shadow,
                            border: border::rounded(style.shadow_border_radius),
                        },
                        style.shadow.color,
                    );
                },
            );
        }

        renderer.with_layer(clip_bounds, |renderer| {
            render(
                renderer,
                &self.image.handle,
                self.image.filter_method,
                self.image.rotation,
                self.image.opacity,
                self.drawing_bounds,
            );
        });
    }

    fn index(&self) -> f32 {
        self.image.scale * 0.5
    }
}
