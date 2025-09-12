//! Draw custom primitives.
use crate::core::{self, Rectangle};
use crate::graphics::Viewport;
use crate::graphics::futures::{MaybeSend, MaybeSync};

use rustc_hash::FxHashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;

/// A batch of primitives.
pub type Batch = Vec<Instance>;

/// A set of methods which allows a [`Primitive`] to be rendered.
pub trait Primitive: Debug + MaybeSend + MaybeSync + 'static {
    /// The shared renderer of this [`Primitive`].
    ///
    /// Normally, this will contain a bunch of [`wgpu`] state; like
    /// a rendering pipeline, buffers, and textures.
    ///
    /// All instances of this [`Primitive`] type will share the same
    /// [`Renderer`].
    type Renderer: MaybeSend + MaybeSync;

    /// Initializes the [`Renderer`](Self::Renderer) of the [`Primitive`].
    ///
    /// This will only be called once, when the first [`Primitive`] of this kind
    /// is encountered.
    fn initialize(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self::Renderer;

    /// Processes the [`Primitive`], allowing for GPU buffer allocation.
    fn prepare(
        &self,
        renderer: &mut Self::Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    );

    /// Draws the [`Primitive`] in the given [`wgpu::RenderPass`].
    ///
    /// When possible, this should be implemented over [`render`](Self::render)
    /// since reusing the existing render pass should be considerably more
    /// efficient than issuing a new one.
    ///
    /// The viewport and scissor rect of the render pass provided is set
    /// to the bounds and clip bounds of the [`Primitive`], respectively.
    ///
    /// If you have complex composition needs, then you can leverage
    /// [`render`](Self::render) by returning `false` here.
    ///
    /// By default, it does nothing and returns `false`.
    fn draw(
        &self,
        _renderer: &Self::Renderer,
        _render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        false
    }

    /// Renders the [`Primitive`], using the given [`wgpu::CommandEncoder`].
    ///
    /// This will only be called if [`draw`](Self::draw) returns `false`.
    ///
    /// By default, it does nothing.
    fn render(
        &self,
        _renderer: &Self::Renderer,
        _encoder: &mut wgpu::CommandEncoder,
        _target: &wgpu::TextureView,
        _clip_bounds: &Rectangle<u32>,
    ) {
    }
}

pub(crate) trait Stored:
    Debug + MaybeSend + MaybeSync + 'static
{
    fn prepare(
        &self,
        storage: &mut Storage,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        bounds: &Rectangle,
        viewport: &Viewport,
    );

    fn draw(
        &self,
        storage: &Storage,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool;

    fn render(
        &self,
        storage: &Storage,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    );
}

#[derive(Debug)]
struct BlackBox<P: Primitive> {
    primitive: P,
}

impl<P: Primitive> Stored for BlackBox<P> {
    fn prepare(
        &self,
        storage: &mut Storage,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        if !storage.has::<P>() {
            storage.store::<P, _>(
                self.primitive.initialize(device, queue, format),
            );
        }

        let renderer = storage
            .get_mut::<P>()
            .expect("renderer should be initialized")
            .downcast_mut::<P::Renderer>()
            .expect("renderer should have the proper type");

        self.primitive
            .prepare(renderer, device, queue, bounds, viewport);
    }

    fn draw(
        &self,
        storage: &Storage,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        let renderer = storage
            .get::<P>()
            .expect("renderer should be initialized")
            .downcast_ref::<P::Renderer>()
            .expect("renderer should have the proper type");

        self.primitive.draw(renderer, render_pass)
    }

    fn render(
        &self,
        storage: &Storage,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let renderer = storage
            .get::<P>()
            .expect("renderer should be initialized")
            .downcast_ref::<P::Renderer>()
            .expect("renderer should have the proper type");

        self.primitive
            .render(renderer, encoder, target, clip_bounds);
    }
}

#[derive(Debug)]
/// An instance of a specific [`Primitive`].
pub struct Instance {
    /// The bounds of the [`Instance`].
    pub(crate) bounds: Rectangle,

    /// The [`Primitive`] to render.
    pub(crate) primitive: Box<dyn Stored>,
}

impl Instance {
    /// Creates a new [`Instance`] with the given [`Primitive`].
    pub fn new(bounds: Rectangle, primitive: impl Primitive) -> Self {
        Instance {
            bounds,
            primitive: Box::new(BlackBox { primitive }),
        }
    }
}

/// A renderer than can draw custom primitives.
pub trait Renderer: core::Renderer {
    /// Draws a custom primitive.
    fn draw_primitive(&mut self, bounds: Rectangle, primitive: impl Primitive);
}

/// Stores custom, user-provided types.
#[derive(Default)]
pub struct Storage {
    pipelines: FxHashMap<TypeId, Box<dyn AnyConcurrent>>,
}

impl Storage {
    /// Returns `true` if `Storage` contains a type `T`.
    pub fn has<T: 'static>(&self) -> bool {
        self.pipelines.contains_key(&TypeId::of::<T>())
    }

    /// Inserts the data `T` in to [`Storage`].
    pub fn store<T: 'static, D: Any + MaybeSend + MaybeSync>(
        &mut self,
        data: D,
    ) {
        let _ = self.pipelines.insert(TypeId::of::<T>(), Box::new(data));
    }

    /// Returns a reference to the data with type `T` if it exists in [`Storage`].
    pub fn get<T: 'static>(&self) -> Option<&dyn Any> {
        self.pipelines
            .get(&TypeId::of::<T>())
            .map(|pipeline| pipeline.as_ref() as &dyn Any)
    }

    /// Returns a mutable reference to the data with type `T` if it exists in [`Storage`].
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut dyn Any> {
        self.pipelines
            .get_mut(&TypeId::of::<T>())
            .map(|pipeline| pipeline.as_mut() as &mut dyn Any)
    }
}

trait AnyConcurrent: Any + MaybeSend + MaybeSync {}

impl<T> AnyConcurrent for T where T: Any + MaybeSend + MaybeSync {}
