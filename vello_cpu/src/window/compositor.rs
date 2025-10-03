use crate::core::{Color, Rectangle, Size};
use crate::graphics::compositor::{self, Information};
use crate::graphics::damage;
use crate::graphics::error::{self, Error};
use crate::graphics::{self, Viewport};
use crate::{Layer, Renderer, Settings};

use std::collections::VecDeque;
use std::num::NonZeroU32;

pub struct Compositor {
    context: softbuffer::Context<Box<dyn compositor::Window>>,
    settings: Settings,
}

pub struct Surface {
    window: softbuffer::Surface<
        Box<dyn compositor::Window>,
        Box<dyn compositor::Window>,
    >,
    layer_stack: VecDeque<Vec<Layer>>,
    background_color: Color,
    max_age: u8,
}

impl crate::graphics::Compositor for Compositor {
    type Renderer = Renderer;
    type Surface = Surface;

    async fn with_backend<W: compositor::Window>(
        settings: graphics::Settings,
        compatible_window: W,
        backend: Option<&str>,
    ) -> Result<Self, Error> {
        match backend {
            None | Some("vello-cpu") | Some("vello_cpu") => {
                Ok(new(settings.into(), compatible_window))
            }
            Some(backend) => Err(Error::GraphicsAdapterNotFound {
                backend: "vello-cpu",
                reason: error::Reason::DidNotMatch {
                    preferred_backend: backend.to_owned(),
                },
            }),
        }
    }

    fn create_renderer(&self) -> Self::Renderer {
        Renderer::new(
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
            layer_stack: VecDeque::new(),
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

        surface.layer_stack.clear();
    }

    fn information(&self) -> Information {
        Information {
            adapter: String::from("CPU"),
            backend: String::from("vello-cpu"),
        }
    }

    fn present(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &Viewport,
        background_color: Color,
        on_pre_present: impl FnOnce(),
    ) -> Result<(), compositor::SurfaceError> {
        present(
            renderer,
            surface,
            viewport,
            background_color,
            on_pre_present,
        )
    }

    fn screenshot(
        &mut self,
        renderer: &mut Self::Renderer,
        viewport: &Viewport,
        background_color: Color,
    ) -> Vec<u8> {
        screenshot(renderer, viewport, background_color)
    }
}

pub fn new<W: compositor::Window>(
    settings: Settings,
    compatible_window: W,
) -> Compositor {
    #[allow(unsafe_code)]
    let context = softbuffer::Context::new(Box::new(compatible_window) as _)
        .expect("Create softbuffer context");

    Compositor { context, settings }
}

pub fn present(
    renderer: &mut Renderer,
    surface: &mut Surface,
    viewport: &Viewport,
    background_color: Color,
    on_pre_present: impl FnOnce(),
) -> Result<(), compositor::SurfaceError> {
    let physical_size = viewport.physical_size();

    let mut buffer = surface
        .window
        .buffer_mut()
        .map_err(|_| compositor::SurfaceError::Lost)?;

    let last_layers = {
        let age = buffer.age();

        surface.max_age = surface.max_age.max(age);
        surface.layer_stack.truncate(surface.max_age as usize);

        if age > 0 {
            surface.layer_stack.get(age as usize - 1)
        } else {
            None
        }
    };

    let damage = last_layers
        .and_then(|last_layers| {
            (surface.background_color == background_color).then(|| {
                damage::diff(
                    last_layers,
                    renderer.layers(),
                    |layer| vec![layer.bounds],
                    Layer::damage,
                )
            })
        })
        .unwrap_or_else(|| vec![Rectangle::with_size(viewport.logical_size())]);

    if damage.is_empty() {
        return Ok(());
    }

    surface.layer_stack.push_front(renderer.layers().to_vec());
    surface.background_color = background_color;

    let damage =
        damage::group(damage, Rectangle::with_size(viewport.logical_size()));

    let mut render_context = vello_cpu::RenderContext::new(
        physical_size.width as u16,
        physical_size.height as u16,
    );

    let data: Vec<vello_cpu::color::PremulRgba8> = buffer
        .iter()
        .map(|x| vello_cpu::color::PremulRgba8::from_u32(*x))
        .collect();

    let mut pixmap = vello_cpu::Pixmap::from_parts(
        data,
        physical_size.width as u16,
        physical_size.height as u16,
    );

    renderer.draw(
        &mut pixmap,
        &mut render_context,
        viewport,
        &damage,
        background_color,
    );

    // let p: Vec<u32> = pixmap.data().iter().map(|x| x.to_u32()).collect();
    buffer.copy_from_slice(bytemuck::cast_slice(pixmap.data()));

    on_pre_present();
    buffer.present().map_err(|_| compositor::SurfaceError::Lost)
}

pub fn screenshot(
    renderer: &mut Renderer,
    viewport: &Viewport,
    background_color: Color,
) -> Vec<u8> {
    let physical_size = viewport.physical_size();

    let mut render_context = vello_cpu::RenderContext::new(
        physical_size.width as u16,
        physical_size.height as u16,
    );

    let mut pixmap = vello_cpu::Pixmap::new(
        physical_size.width as u16,
        physical_size.height as u16,
    );

    renderer.draw(
        &mut pixmap,
        &mut render_context,
        viewport,
        &[Rectangle::with_size(Size::new(
            physical_size.width as f32,
            physical_size.height as f32,
        ))],
        background_color,
    );

    pixmap.data_as_u8_slice().to_vec()
}
