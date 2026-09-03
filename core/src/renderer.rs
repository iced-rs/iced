//! Write your own renderer.
#[cfg(debug_assertions)]
mod null;

use crate::image;
use crate::{
    Background, Border, Color, Font, Pixels, Rectangle, Shadow, Size, Transformation, Vector,
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

    /// Starts recording a new group of layers inside the given `bounds`.
    ///
    /// All primitives drawn until [`end_group`](Self::end_group) is called are
    /// rendered to an isolated, offscreen buffer and then composited back as a
    /// whole with the given [`GroupEffect`]. Because the group is flattened
    /// before the effect is applied, overlapping primitives are affected together
    /// instead of independently, and nested groups compose.
    fn start_group(&mut self, _bounds: Rectangle, _effect: GroupEffect) {}

    /// Ends recording the current group.
    ///
    /// The contents will be composited with the [`GroupEffect`] specified in
    /// [`start_group`](Self::start_group).
    fn end_group(&mut self) {}

    /// Draws the primitives recorded in the given closure as a single group,
    /// compositing them back with the given [`GroupEffect`].
    fn with_group(&mut self, bounds: Rectangle, effect: GroupEffect, f: impl FnOnce(&mut Self)) {
        self.start_group(bounds, effect);
        f(self);
        self.end_group();
    }

    /// Draws the primitives recorded in the given closure as a single group with
    /// the specified opacity.
    ///
    /// This is a convenience for [`with_group`](Self::with_group) with a
    /// [`GroupEffect::Opacity`]. The primitives are rendered to an isolated,
    /// offscreen buffer and then composited as a whole with the given opacity.
    fn with_opacity(&mut self, bounds: Rectangle, opacity: f32, f: impl FnOnce(&mut Self)) {
        self.with_group(bounds, GroupEffect::Opacity(opacity), f);
    }

    /// Fills a [`Quad`] with the provided [`Background`].
    fn fill_quad(&mut self, quad: Quad, background: impl Into<Background>);

    /// Creates an [`image::Allocation`] for the given [`image::Handle`] and calls the given callback with it.
    fn allocate_image(
        &self,
        handle: &image::Handle,
        callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
    );

    /// Provides hints to the [`Renderer`] about the rendering target.
    ///
    /// This may be used internally by the [`Renderer`] to perform optimizations
    /// and/or improve rendering quality.
    ///
    /// For instance, providing a [`Scale`] may be used by some renderers to
    /// perform metrics hinting internally in physical coordinates while keeping
    /// layout coordinates logical and, therefore, maintain linearity.
    fn hint(&mut self, scale: Scale);

    /// Returns the last [`Scale`] provided as a [`hint`](Self::hint).
    fn scale(&self) -> Option<Scale>;

    /// Returns the last hint factor provided as a [`hint`](Self::hint),
    /// only if [`Settings::metrics_hinting`] is enabled.
    fn hint_factor(&self) -> Option<f32> {
        if !self.settings().metrics_hinting {
            return None;
        }

        self.scale().map(Scale::total)
    }

    /// Resets the [`Renderer`] to start drawing in the `new_bounds` from scratch.
    fn reset(&mut self, new_bounds: Rectangle);

    /// Returns the [`Settings`] of this [`Renderer`].
    fn settings(&self) -> Settings;

    /// Polls any concurrent computations that may be pending in the [`Renderer`].
    ///
    /// By default, it does nothing.
    fn tick(&mut self) {}
}

/// The effect applied when compositing a group of layers back onto its target.
///
/// A group (see [`Renderer::start_group`]) isolates its layers into an offscreen
/// buffer; the effect describes how that buffer is blended back. This makes
/// effects composable and lets new ones — like blur or color filters — reuse the
/// same isolation machinery.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupEffect {
    /// Blends the group with the given opacity, from `0.0` (fully transparent)
    /// to `1.0` (fully opaque).
    Opacity(f32),
}

impl GroupEffect {
    /// Returns whether the effect has no visible impact, so the group can be
    /// drawn inline without an isolated target.
    pub fn is_noop(self) -> bool {
        match self {
            GroupEffect::Opacity(opacity) => opacity >= 1.0,
        }
    }

    /// Returns whether adjacent, non-overlapping groups with an equal effect can
    /// be merged and composited together.
    ///
    /// Effects whose result depends only on each pixel independently (like
    /// opacity) can batch; effects that read neighbouring pixels (like blur)
    /// cannot.
    pub fn is_batchable(self) -> bool {
        match self {
            GroupEffect::Opacity(_) => true,
        }
    }
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
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            bounds: Rectangle::with_size(Size::ZERO),
            border: Border::default(),
            shadow: Shadow::default(),
            snap: CRISP,
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
    fn new(settings: Settings, backend: Option<&str>) -> impl Future<Output = Option<Self>>
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

/// The settings of a [`Renderer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The default [`Font`] to use.
    pub default_font: Font,

    /// The default size of text.
    ///
    /// By default, it will be set to `16.0`.
    pub default_text_size: Pixels,

    /// Whether the [`Renderer`] should perform metrics hinting.
    ///
    /// By default, it is enabled.
    pub metrics_hinting: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_font: Font::DEFAULT,
            default_text_size: Pixels(16.0),
            metrics_hinting: true,
        }
    }
}

/// The scale factor of a [`Renderer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    /// The global scale factor of the window.
    ///
    /// This is normally controlled by the OS, and applied globally to all apps.
    pub window: f32,

    /// The local scale factor of the application.
    pub application: f32,
}

impl Scale {
    /// Returns the total scale factor applied by the [`Renderer`].
    ///
    /// This is the product of the window and application scale factors.
    pub fn total(self) -> f32 {
        self.window * self.application
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self {
            window: 1.0,
            application: 1.0,
        }
    }
}
