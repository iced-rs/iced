use crate::core::{Color, Rectangle, Size};
use crate::graphics::compositor::{self, Information};
use crate::graphics::damage;
use crate::graphics::{Error, Viewport};
use crate::{Backend, Primitive, Renderer, Settings};

use std::collections::VecDeque;
use std::marker::PhantomData;
use std::num::NonZeroU32;

pub struct Compositor<Theme> {
    context: softbuffer::Context<Box<dyn compositor::Window>>,
    settings: Settings,
    _theme: PhantomData<Theme>,
}

pub struct Surface {
    window: softbuffer::Surface<
        Box<dyn compositor::Window>,
        Box<dyn compositor::Window>,
    >,
    clip_mask: tiny_skia::Mask,
    primitive_stack: VecDeque<Vec<Primitive>>,
    background_color: Color,
    max_age: u8,
}

impl<Theme> crate::graphics::Compositor for Compositor<Theme> {
    type Settings = Settings;
    type Renderer = Renderer<Theme>;
    type Surface = Surface;

    fn new<W: compositor::Window>(
        settings: Self::Settings,
        compatible_window: W,
    ) -> Result<Self, Error> {
        Ok(new(settings, compatible_window))
    }

    fn create_renderer(&self) -> Self::Renderer {
        Renderer::new(
            Backend::new(),
            self.settings.default_font,
            self.settings.default_text_size,
        )
    }

    fn create_surface<W: compositor::Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface {
        let window = softbuffer::Surface::new(
            &self.context,
            Box::new(window.clone()) as _,
        )
        .expect("Create softbuffer surface for window");

        let mut surface = Surface {
            window,
            clip_mask: tiny_skia::Mask::new(width, height)
                .expect("Create clip mask"),
            primitive_stack: VecDeque::new(),
            background_color: Color::BLACK,
            max_age: 0,
        };

        self.configure_surface(&mut surface, width, height);

        surface
    }

    fn configure_surface(
        &mut self,
        surface: &mut Self::Surface,
        width: u32,
        height: u32,
    ) {
        surface
            .window
            .resize(
                NonZeroU32::new(width).expect("Non-zero width"),
                NonZeroU32::new(height).expect("Non-zero height"),
            )
            .expect("Resize surface");

        surface.clip_mask =
            tiny_skia::Mask::new(width, height).expect("Create clip mask");
        surface.primitive_stack.clear();
    }

    fn fetch_information(&self) -> Information {
        Information {
            adapter: String::from("CPU"),
            backend: String::from("tiny-skia"),
        }
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), compositor::SurfaceError> {
        renderer.with_primitives(|backend, primitives| {
            present(
                backend,
                surface,
                primitives,
                viewport,
                background_color,
                overlay,
            )
        })
    }

    fn screenshot<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Vec<u8> {
        renderer.with_primitives(|backend, primitives| {
            screenshot(
                surface,
                backend,
                primitives,
                viewport,
                background_color,
                overlay,
            )
        })
    }
}

pub fn new<W: compositor::Window, Theme>(
    settings: Settings,
    compatible_window: W,
) -> Compositor<Theme> {
    #[allow(unsafe_code)]
    let context = softbuffer::Context::new(Box::new(compatible_window) as _)
        .expect("Create softbuffer context");

    Compositor {
        context,
        settings,
        _theme: PhantomData,
    }
}

pub fn present<T: AsRef<str>>(
    backend: &mut Backend,
    surface: &mut Surface,
    primitives: &[Primitive],
    viewport: &Viewport,
    background_color: Color,
    overlay: &[T],
) -> Result<(), compositor::SurfaceError> {
    let physical_size = viewport.physical_size();
    let scale_factor = viewport.scale_factor() as f32;

    let mut buffer = surface
        .window
        .buffer_mut()
        .map_err(|_| compositor::SurfaceError::Lost)?;

    let last_primitives = {
        let age = buffer.age();

        surface.max_age = surface.max_age.max(age);
        surface.primitive_stack.truncate(surface.max_age as usize);

        if age > 0 {
            surface.primitive_stack.get(age as usize - 1)
        } else {
            None
        }
    };

    let damage = last_primitives
        .and_then(|last_primitives| {
            (surface.background_color == background_color)
                .then(|| damage::list(last_primitives, primitives))
        })
        .unwrap_or_else(|| vec![Rectangle::with_size(viewport.logical_size())]);

    if damage.is_empty() {
        return Ok(());
    }

    surface.primitive_stack.push_front(primitives.to_vec());
    surface.background_color = background_color;

    let damage = damage::group(damage, scale_factor, physical_size);

    let mut pixels = tiny_skia::PixmapMut::from_bytes(
        bytemuck::cast_slice_mut(&mut buffer),
        physical_size.width,
        physical_size.height,
    )
    .expect("Create pixel map");

    backend.draw(
        &mut pixels,
        &mut surface.clip_mask,
        primitives,
        viewport,
        &damage,
        background_color,
        overlay,
    );

    buffer.present().map_err(|_| compositor::SurfaceError::Lost)
}

pub fn screenshot<T: AsRef<str>>(
    surface: &mut Surface,
    backend: &mut Backend,
    primitives: &[Primitive],
    viewport: &Viewport,
    background_color: Color,
    overlay: &[T],
) -> Vec<u8> {
    let size = viewport.physical_size();

    let mut offscreen_buffer: Vec<u32> =
        vec![0; size.width as usize * size.height as usize];

    backend.draw(
        &mut tiny_skia::PixmapMut::from_bytes(
            bytemuck::cast_slice_mut(&mut offscreen_buffer),
            size.width,
            size.height,
        )
        .expect("Create offscreen pixel map"),
        &mut surface.clip_mask,
        primitives,
        viewport,
        &[Rectangle::with_size(Size::new(
            size.width as f32,
            size.height as f32,
        ))],
        background_color,
        overlay,
    );

    offscreen_buffer.iter().fold(
        Vec::with_capacity(offscreen_buffer.len() * 4),
        |mut acc, pixel| {
            const A_MASK: u32 = 0xFF_00_00_00;
            const R_MASK: u32 = 0x00_FF_00_00;
            const G_MASK: u32 = 0x00_00_FF_00;
            const B_MASK: u32 = 0x00_00_00_FF;

            let a = ((A_MASK & pixel) >> 24) as u8;
            let r = ((R_MASK & pixel) >> 16) as u8;
            let g = ((G_MASK & pixel) >> 8) as u8;
            let b = (B_MASK & pixel) as u8;

            acc.extend([r, g, b, a]);
            acc
        },
    )
}
