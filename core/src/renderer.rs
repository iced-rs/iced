//! Write your own renderer.
#[cfg(debug_assertions)]
mod null;

use crate::image;
use crate::{
    Background, Border, Color, Font, Outline, Pixels, Rectangle, Shadow, Size, Transformation,
    Vector,
};

/// Whether anti-aliasing should be avoided by snapping primitive coordinates to the
/// pixel grid.
pub const CRISP: bool = cfg!(feature = "crisp");

/// A component that can be used by widgets to draw themselves on a screen.
pub trait Renderer {
    /// Starts recording a new layer.
    fn start_layer(&mut self, bounds: Rectangle);

    /// Ends recording a new layer.
    ///
    /// The new layer will clip its contents to the provided `bounds`.
    fn end_layer(&mut self);

    /// Draws the primitives recorded in the given closure in a new layer.
    ///
    /// The layer will clip its contents to the provided `bounds`.
    fn with_layer(&mut self, bounds: Rectangle, f: impl FnOnce(&mut Self)) {
        self.start_layer(bounds);
        f(self);
        self.end_layer();
    }

    /// Starts recording with a new [`Transformation`].
    fn start_transformation(&mut self, transformation: Transformation);

    /// Ends recording a new layer.
    ///
    /// The new layer will clip its contents to the provided `bounds`.
    fn end_transformation(&mut self);

    /// Applies a [`Transformation`] to the primitives recorded in the given closure.
    fn with_transformation(&mut self, transformation: Transformation, f: impl FnOnce(&mut Self)) {
        self.start_transformation(transformation);
        f(self);
        self.end_transformation();
    }

    /// Applies a translation to the primitives recorded in the given closure.
    fn with_translation(&mut self, translation: Vector, f: impl FnOnce(&mut Self)) {
        self.with_transformation(Transformation::translate(translation.x, translation.y), f);
    }

    /// Starts recording a new opacity group.
    ///
    /// All primitives drawn until [`end_opacity`](Self::end_opacity) is called
    /// will be rendered to an offscreen texture and then composited with the
    /// given opacity value.
    ///
    /// Opacity values should be in the range `0.0` (fully transparent) to `1.0` (fully opaque).
    fn start_opacity(&mut self, _bounds: Rectangle, _opacity: f32) {}

    /// Ends recording the current opacity group.
    ///
    /// The contents will be composited with the opacity specified in [`start_opacity`](Self::start_opacity).
    fn end_opacity(&mut self) {}

    /// Draws the primitives recorded in the given closure with the specified opacity.
    ///
    /// The primitives will be rendered to an offscreen texture and then composited
    /// with the given opacity value.
    fn with_opacity(&mut self, bounds: Rectangle, opacity: f32, f: impl FnOnce(&mut Self)) {
        self.start_opacity(bounds, opacity);
        f(self);
        self.end_opacity();
    }

    /// Starts recording a gradient fade effect.
    ///
    /// All primitives drawn until [`end_gradient_fade`](Self::end_gradient_fade) is called
    /// will be rendered to an offscreen texture and then composited with a gradient
    /// alpha mask for fade effects.
    ///
    /// # Arguments
    /// * `bounds` - The area where the gradient fade applies
    /// * `direction` - The direction of the fade (0=TopToBottom, 1=BottomToTop, 2=LeftToRight, 3=RightToLeft)
    /// * `fade_start` - Where the fade begins (0.0 to 1.0, relative to bounds)
    /// * `fade_end` - Where the fade ends (0.0 to 1.0, relative to bounds)
    fn start_gradient_fade(
        &mut self,
        _bounds: Rectangle,
        _direction: u8,
        _fade_start: f32,
        _fade_end: f32,
    ) {
    }

    /// Ends recording the current gradient fade effect.
    ///
    /// The contents will be composited with the gradient alpha mask specified in
    /// [`start_gradient_fade`](Self::start_gradient_fade).
    fn end_gradient_fade(&mut self) {}

    /// Draws the primitives recorded in the given closure with a gradient fade effect.
    ///
    /// The primitives will be rendered to an offscreen texture and then composited
    /// with a gradient alpha mask.
    fn with_gradient_fade(
        &mut self,
        bounds: Rectangle,
        direction: u8,
        fade_start: f32,
        fade_end: f32,
        f: impl FnOnce(&mut Self),
    ) {
        self.start_gradient_fade(bounds, direction, fade_start, fade_end);
        f(self);
        self.end_gradient_fade();
    }

    /// Draws a backdrop blur effect at the given bounds.
    ///
    /// This blurs all content that was drawn BEFORE this call within the specified bounds.
    /// Content drawn after this call will appear on top of the blurred area.
    ///
    /// # Arguments
    /// * `bounds` - The area where the blur effect applies
    /// * `radius` - The blur radius in logical pixels
    /// * `border_radius` - Border radius [top_left, top_right, bottom_right, bottom_left] in logical pixels
    fn draw_backdrop_blur(
        &mut self,
        _bounds: Rectangle,
        _radius: f32,
        _border_radius: [f32; 4],
        _fade_start: f32,
    ) {
    }

    /// Begins recording content that should be rendered AFTER backdrop blur effects.
    ///
    /// Content drawn between `start_post_blur_layer` and `end_post_blur_layer` will be
    /// rendered after all blur effects are applied, ensuring it appears on top of blurred areas.
    fn start_post_blur_layer(&mut self, _bounds: Rectangle) {}

    /// Ends recording post-blur content.
    fn end_post_blur_layer(&mut self) {}

    /// Helper to draw content that appears on top of backdrop blur.
    ///
    /// This is a convenience wrapper around `start_post_blur_layer` and `end_post_blur_layer`.
    fn with_post_blur_layer(&mut self, bounds: Rectangle, f: impl FnOnce(&mut Self)) {
        self.start_post_blur_layer(bounds);
        f(self);
        self.end_post_blur_layer();
    }

    /// Fills a [`Quad`] with the provided [`Background`].
    fn fill_quad(&mut self, quad: Quad, background: impl Into<Background>);

    /// Draws a focus outline ring around the given bounds.
    ///
    /// The outline is rendered as a border-only quad that extends outside the
    /// widget bounds. The [`Outline::gap`] controls the space between the
    /// widget edge and the inner edge of the outline ring.
    ///
    /// Use this for keyboard-focus indicators on text inputs, editors, or any
    /// focusable widget.
    fn draw_outline(&mut self, bounds: Rectangle, outline: Outline) {
        let offset = outline.gap + outline.border.width;
        self.fill_quad(
            Quad {
                bounds: Rectangle {
                    x: bounds.x - offset,
                    y: bounds.y - offset,
                    width: bounds.width + offset * 2.0,
                    height: bounds.height + offset * 2.0,
                },
                border: outline.border,
                ..Quad::default()
            },
            Color::TRANSPARENT,
        );
    }

    /// Creates an [`image::Allocation`] for the given [`image::Handle`] and calls the given callback with it.
    fn allocate_image(
        &mut self,
        handle: &image::Handle,
        callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
    );

    /// Provides hints to the [`Renderer`] about the rendering target.
    ///
    /// This may be used internally by the [`Renderer`] to perform optimizations
    /// and/or improve rendering quality.
    ///
    /// For instance, providing a `scale_factor` may be used by some renderers to
    /// perform metrics hinting internally in physical coordinates while keeping
    /// layout coordinates logical and, therefore, maintain linearity.
    fn hint(&mut self, scale_factor: f32);

    /// Returns the last scale factor provided as a [`hint`](Self::hint).
    fn scale_factor(&self) -> Option<f32>;

    /// Resets the [`Renderer`] to start drawing in the `new_bounds` from scratch.
    fn reset(&mut self, new_bounds: Rectangle);

    /// Polls any concurrent computations that may be pending in the [`Renderer`].
    ///
    /// By default, it does nothing.
    fn tick(&mut self) {}
}

/// A polygon with four sides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    /// The bounds of the [`Quad`].
    pub bounds: Rectangle,

    /// The [`Border`] of the [`Quad`]. The border is drawn on the inside of the [`Quad`].
    pub border: Border,

    /// The [`Shadow`] of the [`Quad`].
    pub shadow: Shadow,

    /// Whether the [`Quad`] should be snapped to the pixel grid.
    pub snap: bool,

    /// Whether only the border should be rendered (clip the interior).
    /// When true, the background fills only the border region, not the interior.
    /// This is useful for gradient borders where you want the gradient to appear
    /// only in the border area.
    pub border_only: bool,
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            bounds: Rectangle::with_size(Size::ZERO),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: CRISP,
            border_only: false,
        }
    }
}

/// The styling attributes of a [`Renderer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The text color
    pub text_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            text_color: Color::BLACK,
        }
    }
}

/// A headless renderer is a renderer that can render offscreen without
/// a window nor a compositor.
pub trait Headless {
    /// Creates a new [`Headless`] renderer;
    fn new(
        default_font: Font,
        default_text_size: Pixels,
        backend: Option<&str>,
    ) -> impl Future<Output = Option<Self>>
    where
        Self: Sized;

    /// Returns the unique name of the renderer.
    ///
    /// This name may be used by testing libraries to uniquely identify
    /// snapshots.
    fn name(&self) -> String;

    /// Draws offscreen into a screenshot, returning a collection of
    /// bytes representing the rendered pixels in RGBA order.
    fn screenshot(
        &mut self,
        size: Size<u32>,
        scale_factor: f32,
        background_color: Color,
    ) -> Vec<u8>;
}
