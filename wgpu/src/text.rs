pub use iced_native::text::Hit;

use iced_graphics::layer::Text;
use iced_native::{Font, Size};

#[allow(missing_debug_implementations)]
pub struct Pipeline {
    renderer: glyphon::TextRenderer,
    atlas: glyphon::TextAtlas,
    cache: glyphon::SwashCache<'static>,
}

// TODO: Share with `iced_graphics`
static FONT_SYSTEM: once_cell::sync::Lazy<glyphon::FontSystem> =
    once_cell::sync::Lazy::new(glyphon::FontSystem::new);

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        _default_font: Option<&[u8]>,
        _multithreading: bool,
    ) -> Self {
        Pipeline {
            renderer: glyphon::TextRenderer::new(device, queue),
            atlas: glyphon::TextAtlas::new(device, queue, format),
            cache: glyphon::SwashCache::new(&FONT_SYSTEM),
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sections: &[Text<'_>],
        scale_factor: f32,
        target_size: Size<u32>,
    ) {
        let buffers: Vec<_> = sections
            .iter()
            .map(|section| {
                let metrics = glyphon::Metrics::new(
                    (section.size * scale_factor) as i32,
                    (section.size * 1.2 * scale_factor) as i32,
                );

                let mut buffer = glyphon::Buffer::new(&FONT_SYSTEM, metrics);

                buffer.set_size(
                    (section.bounds.width * scale_factor).ceil() as i32,
                    (section.bounds.height * scale_factor).ceil() as i32,
                );

                buffer.set_text(
                    section.content,
                    glyphon::Attrs::new()
                        .color({
                            let [r, g, b, a] = section.color.into_rgba8();
                            glyphon::Color::rgba(r, g, b, a)
                        })
                        .family(match section.font {
                            Font::Default => glyphon::Family::SansSerif,
                            Font::External { name, .. } => {
                                glyphon::Family::Name(name)
                            }
                        }),
                );

                buffer.shape_until_scroll();

                buffer
            })
            .collect();

        let text_areas: Vec<_> = sections
            .iter()
            .zip(buffers.iter())
            .map(|(section, buffer)| glyphon::TextArea {
                buffer,
                left: (section.bounds.x * scale_factor) as i32,
                top: (section.bounds.y * scale_factor) as i32,
                bounds: glyphon::TextBounds {
                    left: (section.bounds.x * scale_factor) as i32,
                    top: (section.bounds.y * scale_factor) as i32,
                    right: ((section.bounds.x + section.bounds.width)
                        * scale_factor) as i32,
                    bottom: ((section.bounds.y + section.bounds.height)
                        * scale_factor) as i32,
                },
            })
            .collect();

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.atlas,
                glyphon::Resolution {
                    width: target_size.width,
                    height: target_size.height,
                },
                &text_areas,
                glyphon::Color::rgb(0, 0, 0),
                &mut self.cache,
            )
            .expect("Prepare text sections");
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut render_pass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        self.renderer
            .render(&self.atlas, &mut render_pass)
            .expect("Render text");
    }

    pub fn measure(
        &self,
        content: &str,
        size: f32,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        let attrs = match font {
            Font::Default => glyphon::Attrs::new(),
            Font::External { name, .. } => glyphon::Attrs {
                family: glyphon::Family::Name(name),
                ..glyphon::Attrs::new()
            },
        };

        let mut paragraph =
            glyphon::BufferLine::new(content, glyphon::AttrsList::new(attrs));

        // TODO: Cache layout
        let layout = paragraph.layout(
            &FONT_SYSTEM,
            size as i32,
            bounds.width as i32,
            glyphon::Wrap::Word,
        );

        (
            layout.iter().fold(0.0, |max, line| line.w.max(max)),
            size * 1.2 * layout.len() as f32,
        )
    }

    pub fn hit_test(
        &self,
        _content: &str,
        _size: f32,
        _font: iced_native::Font,
        _bounds: iced_native::Size,
        _point: iced_native::Point,
        _nearest_only: bool,
    ) -> Option<Hit> {
        None
    }

    pub fn trim_measurement_cache(&mut self) {}
}
