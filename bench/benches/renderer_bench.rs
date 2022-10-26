use criterion::{black_box, criterion_group, criterion_main, Criterion};

use iced_native::{widget, Renderer};

use iced_bench::{glow::GlowBench, render_widget, wgpu::WgpuBench, Bench};

static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

fn image(size: u32) -> iced_native::image::Handle {
    let mut bytes = Vec::with_capacity(size as usize * size as usize * 4);
    for _y in 0..size {
        for _x in 0..size {
            let r = rand::random();
            let g = rand::random();
            let b = rand::random();
            bytes.extend_from_slice(&[r, g, b, 255]);
        }
    }
    iced_native::image::Handle::from_pixels(size, size, bytes)
}

fn text_primitive<
    R: iced_native::Renderer<Theme = iced::Theme> + iced_native::text::Renderer,
>(
    renderer: &R,
) -> iced_graphics::Primitive {
    iced_graphics::Primitive::Text {
        content: LOREM_IPSUM.to_string(),
        bounds: iced::Rectangle::with_size(iced::Size::new(1024.0, 1024.0)),
        color: iced_native::Color::BLACK,
        size: f32::from(renderer.default_size()),
        font: Default::default(),
        horizontal_alignment: iced_native::alignment::Horizontal::Left,
        vertical_alignment: iced_native::alignment::Vertical::Top,
    }
}

fn text_widget<
    R: iced_native::Renderer<Theme = iced::Theme> + iced_native::text::Renderer,
>() -> widget::Text<'static, R> {
    widget::helpers::text(LOREM_IPSUM)
}

fn iter_render<
    B: Bench,
    F: FnMut(&mut iced_graphics::Renderer<B::Backend, iced::Theme>),
>(
    b: &mut criterion::Bencher,
    bench: &mut B,
    mut draw_cb: F,
) {
    b.iter(|| {
        bench.renderer().clear();
        let state = bench.clear();
        draw_cb(bench.renderer());
        bench.present(state);
    })
}

fn bench_function<
    B: Bench,
    F: FnMut(&mut iced_graphics::Renderer<B::Backend, iced::Theme>),
>(
    c: &mut Criterion,
    bench: &mut B,
    id: &str,
    mut draw_cb: F,
) {
    c.bench_function(&format!("{} {}", B::BACKEND_NAME, id), |b| {
        iter_render(b, bench, |backend| draw_cb(backend));
    });

    // Write output to file, so there's a way to see that generated
    // image is correct.
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("bench-renderers/{}", B::BACKEND_NAME));
    std::fs::create_dir_all(&dir).unwrap();
    bench
        .read_pixels()
        .save(&dir.join(&format!("{}.png", id)))
        .unwrap();
}

fn generic_benchmark<B: Bench>(c: &mut Criterion, bench: &mut B)
where
    B::Backend: iced_graphics::backend::Text,
{
    bench_function(c, bench, "draw no primitive", |_renderer| {});
    bench_function(c, bench, "draw quad primitive", |renderer| {
        renderer.draw_primitive(black_box(iced_graphics::Primitive::Quad {
            bounds: iced::Rectangle::with_size(iced::Size::new(256.0, 256.0)),
            background: iced_native::Background::Color(
                iced_native::Color::BLACK,
            ),
            border_radius: 0.,
            border_width: 0.,
            border_color: Default::default(),
        }));
    });
    bench_function(c, bench, "draw text primitive", |renderer| {
        renderer.draw_primitive(black_box(text_primitive(renderer)));
    });
    let widget = text_widget();
    bench_function(c, bench, "render text", |renderer| {
        render_widget(&widget, renderer);
    });
    let handle = image(1024);
    let bounds = iced::Rectangle::with_size(iced::Size::new(1024.0, 1024.0));
    bench_function(c, bench, "draw image primitive", |renderer| {
        renderer.draw_primitive(iced_graphics::Primitive::Image {
            handle: handle.clone(),
            bounds,
        });
    });
}

fn glow_benchmark(c: &mut Criterion) {
    let mut bench = GlowBench::new(1024, 1024);

    generic_benchmark(c, &mut bench);
}

fn wgpu_benchmark(c: &mut Criterion) {
    let mut bench = WgpuBench::new(1024, 1024);

    generic_benchmark(c, &mut bench);

    let widget = widget::helpers::image(image(1024));
    bench_function(c, &mut bench, "render image", |renderer| {
        render_widget(&widget, renderer);
    });
}

criterion_group!(benches, glow_benchmark, wgpu_benchmark);
criterion_main!(benches);
