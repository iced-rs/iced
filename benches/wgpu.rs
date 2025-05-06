#![allow(missing_docs)]
use criterion::{Bencher, Criterion, criterion_group, criterion_main};

use iced::alignment;
use iced::mouse;
use iced::widget::{canvas, scrollable, stack, text};
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Theme,
};
use iced_wgpu::Renderer;
use iced_wgpu::wgpu;

criterion_main!(benches);
criterion_group!(benches, wgpu_benchmark);

#[allow(unused_results)]
pub fn wgpu_benchmark(c: &mut Criterion) {
    use iced_futures::futures::executor;
    use iced_wgpu::wgpu;

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
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
            memory_hints: wgpu::MemoryHints::MemoryUsage,
        },
        None,
    ))
    .expect("request device");

    c.bench_function("wgpu — canvas (light)", |b| {
        benchmark(b, &adapter, &device, &queue, |_| scene(10));
    });
    c.bench_function("wgpu — canvas (heavy)", |b| {
        benchmark(b, &adapter, &device, &queue, |_| scene(1_000));
    });

    c.bench_function("wgpu - layered text (light)", |b| {
        benchmark(b, &adapter, &device, &queue, |_| layered_text(10));
    });
    c.bench_function("wgpu - layered text (heavy)", |b| {
        benchmark(b, &adapter, &device, &queue, |_| layered_text(1_000));
    });

    c.bench_function("wgpu - dynamic text (light)", |b| {
        benchmark(b, &adapter, &device, &queue, |i| dynamic_text(1_000, i));
    });
    c.bench_function("wgpu - dynamic text (heavy)", |b| {
        benchmark(b, &adapter, &device, &queue, |i| dynamic_text(100_000, i));
    });
}

fn benchmark<'a>(
    bencher: &mut Bencher<'_>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view: impl Fn(usize) -> Element<'a, (), Theme, Renderer>,
) {
    use iced_wgpu::graphics;
    use iced_wgpu::graphics::Antialiasing;
    use iced_wgpu::wgpu;
    use iced_winit::core;
    use iced_winit::runtime;

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let engine = iced_wgpu::Engine::new(
        adapter,
        device.clone(),
        queue.clone(),
        format,
        Some(Antialiasing::MSAAx4),
    );

    let mut renderer = Renderer::new(engine, Font::DEFAULT, Pixels::from(16));

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

    let mut i = 0;
    let mut cache = Some(runtime::user_interface::Cache::default());

    bencher.iter(|| {
        let mut user_interface = runtime::UserInterface::build(
            view(i),
            viewport.logical_size(),
            cache.take().unwrap(),
            &mut renderer,
        );

        user_interface.draw(
            &mut renderer,
            &Theme::Dark,
            &core::renderer::Style {
                text_color: Color::WHITE,
            },
            mouse::Cursor::Unavailable,
        );

        cache = Some(user_interface.into_cache());

        let submission = renderer.present(
            Some(Color::BLACK),
            format,
            &texture_view,
            &viewport,
        );

        let _ = device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission));

        i += 1;
    });
}

fn scene<'a, Message: 'a>(n: usize) -> Element<'a, Message, Theme, Renderer> {
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
                        align_x: text::Alignment::Left,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Basic,
                        max_width: f32::INFINITY,
                    });
                }
            })]
        }
    }

    canvas(Scene { n })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn layered_text<'a, Message: 'a>(
    n: usize,
) -> Element<'a, Message, Theme, Renderer> {
    stack((0..n).map(|i| text(format!("I am paragraph {i}!")).into()))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn dynamic_text<'a, Message: 'a>(
    n: usize,
    i: usize,
) -> Element<'a, Message, Theme, Renderer> {
    const LOREM_IPSUM: &str = include_str!("ipsum.txt");

    scrollable(
        text(format!(
            "{}... Iteration {i}",
            std::iter::repeat(LOREM_IPSUM.chars())
                .flatten()
                .take(n)
                .collect::<String>(),
        ))
        .size(10),
    )
    .into()
}
