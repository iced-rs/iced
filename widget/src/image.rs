//! Display images in your user interface.
pub mod viewer;
use iced_runtime::core::widget::Id;
pub use viewer::Viewer;

use crate::core::image;
use crate::core::layout;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    ContentFit, Element, Layout, Length, Point, Rectangle, Size, Vector, Widget,
};

use std::borrow::Cow;
use std::hash::Hash;

pub use image::Handle;

/// Creates a new [`Viewer`] with the given image `Handle`.
pub fn viewer<Handle>(handle: Handle) -> Viewer<Handle> {
    Viewer::new(handle)
}

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
///
/// ```no_run
/// # use iced_widget::image::{self, Image};
/// #
/// let image = Image::<image::Handle>::new("resources/ferris.png");
/// ```
///
/// <img src="https://github.com/iced-rs/iced/blob/9712b319bb7a32848001b96bd84977430f14b623/examples/resources/ferris.png?raw=true" width="300">
#[derive(Debug)]
pub struct Image<'a, Handle> {
    id: Id,
    #[cfg(feature = "a11y")]
    name: Option<Cow<'a, str>>,
    #[cfg(feature = "a11y")]
    description: Option<iced_accessibility::Description<'a>>,
    #[cfg(feature = "a11y")]
    label: Option<Vec<iced_accessibility::accesskit::NodeId>>,
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    phantom_data: std::marker::PhantomData<&'a ()>,
}

impl<'a, Handle> Image<'a, Handle> {
    /// Creates a new [`Image`] with the given path.
    pub fn new<T: Into<Handle>>(handle: T) -> Self {
        Image {
            id: Id::unique(),
            #[cfg(feature = "a11y")]
            name: None,
            #[cfg(feature = "a11y")]
            description: None,
            #[cfg(feature = "a11y")]
            label: None,
            handle: handle.into(),
            width: Length::Shrink,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
            phantom_data: std::marker::PhantomData,
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
    pub fn content_fit(self, content_fit: ContentFit) -> Self {
        Self {
            content_fit,
            ..self
        }
    }

    #[cfg(feature = "a11y")]
    /// Sets the name of the [`Button`].
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description_widget<T: iced_accessibility::Describes>(
        mut self,
        description: &T,
    ) -> Self {
        self.description = Some(iced_accessibility::Description::Id(
            description.description(),
        ));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the description of the [`Button`].
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description =
            Some(iced_accessibility::Description::Text(description.into()));
        self
    }

    #[cfg(feature = "a11y")]
    /// Sets the label of the [`Button`].
    pub fn label(mut self, label: &dyn iced_accessibility::Labels) -> Self {
        self.label =
            Some(label.label().into_iter().map(|l| l.into()).collect());
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
) -> layout::Node
where
    Renderer: image::Renderer<Handle = Handle>,
{
    // The raw w/h of the underlying image
    let image_size = {
        let Size { width, height } = renderer.dimensions(handle);

        Size::new(width as f32, height as f32)
    };

    // The size to be available to the widget prior to `Shrink`ing
    let raw_size = limits.width(width).height(height).resolve(image_size);

    // The uncropped size of the image when fit to the bounds above
    let full_size = content_fit.fit(image_size, raw_size);

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

/// Draws an [`Image`]
pub fn draw<Renderer, Handle>(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    handle: &Handle,
    content_fit: ContentFit,
) where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone + Hash,
{
    let Size { width, height } = renderer.dimensions(handle);
    let image_size = Size::new(width as f32, height as f32);

    let bounds = layout.bounds();
    let adjusted_fit = content_fit.fit(image_size, bounds.size());

    let render = |renderer: &mut Renderer| {
        let offset = Vector::new(
            (bounds.width - adjusted_fit.width).max(0.0) / 2.0,
            (bounds.height - adjusted_fit.height).max(0.0) / 2.0,
        );

        let drawing_bounds = Rectangle {
            width: adjusted_fit.width,
            height: adjusted_fit.height,
            ..bounds
        };

        renderer.draw(handle.clone(), drawing_bounds + offset)
    };

    if adjusted_fit.width > bounds.width || adjusted_fit.height > bounds.height
    {
        renderer.with_layer(bounds, render);
    } else {
        render(renderer)
    }
}

impl<'a, Message, Renderer, Handle> Widget<Message, Renderer>
    for Image<'a, Handle>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone + Hash,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
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
        )
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(renderer, layout, &self.handle, self.content_fit)
    }

    #[cfg(feature = "a11y")]
    fn a11y_nodes(
        &self,
        layout: Layout<'_>,
        _state: &Tree,
        _cursor_position: Point,
    ) -> iced_accessibility::A11yTree {
        use iced_accessibility::{
            accesskit::{NodeBuilder, NodeId, Rect, Role},
            A11yTree,
        };

        let bounds = layout.bounds();
        let Rectangle {
            x,
            y,
            width,
            height,
        } = bounds;
        let bounds = Rect::new(
            x as f64,
            y as f64,
            (x + width) as f64,
            (y + height) as f64,
        );
        let mut node = NodeBuilder::new(Role::Image);
        node.set_bounds(bounds);
        if let Some(name) = self.name.as_ref() {
            node.set_name(name.clone());
        }
        match self.description.as_ref() {
            Some(iced_accessibility::Description::Id(id)) => {
                node.set_described_by(
                    id.iter()
                        .cloned()
                        .map(|id| NodeId::from(id))
                        .collect::<Vec<_>>(),
                );
            }
            Some(iced_accessibility::Description::Text(text)) => {
                node.set_description(text.clone());
            }
            None => {}
        }

        if let Some(label) = self.label.as_ref() {
            node.set_labelled_by(label.clone());
        }

        A11yTree::leaf(node, self.id.clone())
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }
}

impl<'a, Message, Renderer, Handle> From<Image<'a, Handle>>
    for Element<'a, Message, Renderer>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone + Hash + 'a,
{
    fn from(image: Image<'a, Handle>) -> Element<'a, Message, Renderer> {
        Element::new(image)
    }
}
