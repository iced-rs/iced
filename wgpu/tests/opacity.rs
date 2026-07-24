//! Verifies that `iced_wgpu` composites an `opacity` group as a single
//! flattened layer (offscreen texture) rather than fading each primitive
//! independently.
//!
//! Requires a working `wgpu` adapter; the test skips itself if none is
//! available (e.g. on a headless CI without a GPU).
use iced_wgpu::Renderer;
use iced_wgpu::core::renderer::{Headless, Quad, Renderer as _, Scale, Settings};
use iced_wgpu::core::{Background, Border, Color, Rectangle, Shadow, Size};
use iced_wgpu::graphics::Viewport;

const RED: Color = Color::from_rgb(1.0, 0.0, 0.0);
const GREEN: Color = Color::from_rgb(0.0, 1.0, 0.0);

fn quad(x: f32, y: f32, width: f32, height: f32) -> Quad {
    Quad {
        bounds: Rectangle {
            x,
            y,
            width,
            height,
        },
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Renders `f` on a black background and returns the RGBA screenshot bytes,
/// or `None` if no GPU adapter is available.
fn render(f: impl FnOnce(&mut Renderer)) -> Option<Vec<u8>> {
    let mut renderer =
        futures::executor::block_on(<Renderer as Headless>::new(Settings::default(), None))?;

    let viewport = Viewport::with_physical_size(Size::new(100, 100), Scale::default());

    renderer.reset(Rectangle::with_size(Size::new(100.0, 100.0)));
    f(&mut renderer);

    Some(Headless::screenshot(
        &mut renderer,
        viewport.physical_size(),
        1.0,
        Color::BLACK,
    ))
}

fn rgb(bytes: &[u8], x: u32, y: u32) -> (u8, u8, u8) {
    let i = ((y * 100 + x) * 4) as usize;
    (bytes[i], bytes[i + 1], bytes[i + 2])
}

#[test]
fn overlapping_group_does_not_bleed_through() {
    let Some(bytes) = render(|renderer| {
        renderer.with_opacity(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 80.0,
                height: 80.0,
            },
            0.5,
            |renderer| {
                renderer.fill_quad(quad(10.0, 10.0, 60.0, 60.0), Background::Color(RED));
                renderer.fill_quad(quad(30.0, 30.0, 60.0, 60.0), Background::Color(GREEN));
            },
        );
    }) else {
        eprintln!("skipping: no wgpu adapter available");
        return;
    };

    // In the overlap, only the front (green) quad may show. If opacity were
    // applied per-primitive, the back red quad would bleed through here.
    // (Exact channel magnitudes depend on gamma, so bounds are loose; the key
    // assertion is the near-absence of red.)
    let (r, g, b) = rgb(&bytes, 50, 50);
    assert!(r < 24, "red bled through the overlap: r={r}");
    assert!(g > 90, "front green missing in overlap: g={g}");
    assert!(b < 24, "unexpected blue in overlap: b={b}");
    assert!(
        r + 24 < g,
        "red should be far below green in overlap: r={r} g={g}"
    );

    // Back-only region shows red; front-only region shows green.
    let (r, g, _b) = rgb(&bytes, 18, 50);
    assert!(r > 90, "back red missing: r={r}");
    assert!(g < 24, "unexpected green in back-only region: g={g}");

    let (r, g, _b) = rgb(&bytes, 82, 50);
    assert!(r < 24, "unexpected red in front-only region: r={r}");
    assert!(g > 90, "front green missing: g={g}");

    // Outside the group: untouched black background.
    let (r, g, b) = rgb(&bytes, 2, 2);
    assert!(r < 8 && g < 8 && b < 8, "background not black: {r},{g},{b}");
}

#[test]
fn full_opacity_is_opaque() {
    let Some(bytes) = render(|renderer| {
        renderer.with_opacity(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 60.0,
            },
            1.0,
            |renderer| {
                renderer.fill_quad(quad(10.0, 10.0, 60.0, 60.0), Background::Color(RED));
            },
        );
    }) else {
        eprintln!("skipping: no wgpu adapter available");
        return;
    };

    let (r, g, b) = rgb(&bytes, 40, 40);
    assert!(r > 240, "opaque red should be full: r={r}");
    assert!(g < 12 && b < 12, "opaque red should be pure: g={g} b={b}");
}
