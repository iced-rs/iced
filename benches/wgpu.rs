#![allow(missing_docs)]
use criterion::{criterion_group, criterion_main, Bencher, Criterion};

use iced::alignment;
use iced::mouse;
use iced::widget::{canvas, text};
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Theme,
};
use iced_wgpu::Renderer;

criterion_main!(benches);
criterion_group!(benches, wgpu_benchmark);

pub fn wgpu_benchmark(c: &mut Criterion) {
    let _ = c
        .bench_function("wgpu — canvas (light)", |b| benchmark(b, scene(10)));

    let _ = c.bench_function("wgpu — canvas (heavy)", |b| {
        benchmark(b, scene(1_000))
    });
}

fn benchmark<'a>(
    bencher: &mut Bencher<'_>,
    widget: Element<'a, (), Theme, Renderer>,
) {
    use iced_futures::futures::executor;
    use iced_wgpu::graphics;
    use iced_wgpu::graphics::Antialiasing;
    use iced_wgpu::wgpu;
    use iced_winit::core;
    use iced_winit::runtime;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = executor::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        },
    ))
    .expect("request adapter");

    let (device, queue) = executor::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    ))
    .expect("request device");

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let backend = iced_wgpu::Backend::new(
        &adapter,
        &device,
        &queue,
        iced_wgpu::Settings {
            present_mode: wgpu::PresentMode::Immediate,
            internal_backend: wgpu::Backends::all(),
            default_font: Font::DEFAULT,
            default_text_size: Pixels::from(16),
            antialiasing: Some(Antialiasing::MSAAx4),
        },
        format,
    );

    let mut renderer = Renderer::new(backend, Font::DEFAULT, Pixels::from(16));

    let viewport =
        graphics::Viewport::with_physical_size(Size::new(3840, 2160), 2.0);

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 3840,
            height: 2160,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let texture_view =
        texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut user_interface = runtime::UserInterface::build(
        widget,
        viewport.logical_size(),
        runtime::user_interface::Cache::default(),
        &mut renderer,
    );

    bencher.iter(|| {
        let _ = user_interface.draw(
            &mut renderer,
            &Theme::Dark,
            &core::renderer::Style {
                text_color: Color::WHITE,
            },
            mouse::Cursor::Unavailable,
        );

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None,
            });

        renderer.with_primitives(|backend, primitives| {
            backend.present::<&str>(
                &device,
                &queue,
                &mut encoder,
                Some(Color::BLACK),
                format,
                &texture_view,
                primitives,
                &viewport,
                &[],
            );

            let submission = queue.submit(Some(encoder.finish()));
            backend.recall();

            let _ =
                device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission));
        });
    });
}

fn scene<'a, Message: 'a, Theme: 'a>(
    n: usize,
) -> Element<'a, Message, Theme, Renderer> {
    canvas(Scene { n })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

struct Scene {
    n: usize,
}

impl<Message, Theme> canvas::Program<Message, Theme, Renderer> for Scene {
    type State = canvas::Cache<Renderer>;

    fn draw(
        &self,
        cache: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        vec![cache.draw(renderer, bounds.size(), |frame| {
            for i in 0..self.n {
                frame.fill_rectangle(
                    Point::new(0.0, i as f32),
                    Size::new(10.0, 10.0),
                    Color::WHITE,
                );
            }

            for i in 0..self.n {
                frame.fill_text(canvas::Text {
                    content: i.to_string(),
                    position: Point::new(0.0, i as f32),
                    color: Color::BLACK,
                    size: Pixels::from(16),
                    line_height: text::LineHeight::default(),
                    font: Font::DEFAULT,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping: text::Shaping::Basic,
                });
            }
        })]
    }
}
