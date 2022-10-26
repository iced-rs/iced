use iced_bench::{glow::GlowBench, wgpu::WgpuBench, Bench};

fn rand_pixels(size: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(size as usize * size as usize * 4);
    for _y in 0..size {
        for _x in 0..size {
            let b = rand::random();
            let g = rand::random();
            let r = rand::random();
            bytes.extend_from_slice(&[b, g, r, 255]);
        }
    }
    bytes
}

#[test]
fn render_image_primitive_wgpu() {
    let size: u16 = 8;

    let mut bench = WgpuBench::new(size.into(), size.into());

    let pixels = rand_pixels(size.into());
    let handle = iced_native::image::Handle::from_pixels(
        size.into(),
        size.into(),
        pixels.clone(),
    );
    let bounds =
        iced::Rectangle::with_size(iced::Size::new(size.into(), size.into()));

    let state = bench.clear();
    bench
        .renderer()
        .draw_primitive(iced_graphics::Primitive::Image {
            handle: handle.clone(),
            bounds,
        });
    bench.present(state);

    let output_pixels =
        image_rs::DynamicImage::ImageRgba8(bench.read_pixels()).to_bgra8();

    assert_eq!(pixels, *output_pixels);
}

#[test]
fn render_image_primitive_glow() {
    let size: u16 = 16;

    let mut bench = GlowBench::new(size.into(), size.into());

    let pixels = rand_pixels(size.into());
    let handle = iced_native::image::Handle::from_pixels(
        size.into(),
        size.into(),
        pixels.clone(),
    );
    let bounds =
        iced::Rectangle::with_size(iced::Size::new(size.into(), size.into()));

    let state = bench.clear();
    bench
        .renderer()
        .draw_primitive(iced_graphics::Primitive::Image {
            handle: handle.clone(),
            bounds,
        });
    bench.present(state);

    let output_pixels =
        image_rs::DynamicImage::ImageRgba8(bench.read_pixels()).to_bgra8();

    assert_eq!(pixels, *output_pixels);
}
