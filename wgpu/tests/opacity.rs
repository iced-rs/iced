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

fn red_quad_at(renderer_opacity: &dyn Fn(&mut Renderer)) -> Option<(u8, u8, u8)> {
    render(renderer_opacity).map(|bytes| rgb(&bytes, 40, 40))
}

#[test]
fn nested_opacity_multiplies() {
    // Exercises the texture pool at two nesting depths (outer uses pool[0], inner
    // pool[1], inner composites into pool[0], then pool[0] into the frame).
    //
    // Nesting must multiply: 50% inside 50% has to match a single 25% group.
    // Comparing renders avoids depending on the exact gamma/sRGB value.
    let single = |alpha: f32| {
        move |renderer: &mut Renderer| {
            renderer.with_opacity(
                Rectangle {
                    x: 10.0,
                    y: 10.0,
                    width: 60.0,
                    height: 60.0,
                },
                alpha,
                |renderer| {
                    renderer.fill_quad(quad(10.0, 10.0, 60.0, 60.0), Background::Color(RED));
                },
            );
        }
    };

    let nested = |renderer: &mut Renderer| {
        renderer.with_opacity(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 60.0,
            },
            0.5,
            |renderer| {
                (single(0.5))(renderer);
            },
        );
    };

    let Some((nested_r, _, _)) = red_quad_at(&nested) else {
        eprintln!("skipping: no wgpu adapter available");
        return;
    };
    let (quarter_r, _, _) = red_quad_at(&single(0.25)).unwrap();
    let (half_r, _, _) = red_quad_at(&single(0.5)).unwrap();

    // Nested 0.5*0.5 should match a single 0.25 group, and be clearly darker
    // than a single 0.5 group (i.e. the inner opacity is not lost).
    assert!(
        nested_r.abs_diff(quarter_r) <= 6,
        "nested 0.5*0.5 (r={nested_r}) should match single 0.25 (r={quarter_r})"
    );
    assert!(
        nested_r + 10 < half_r,
        "nested 0.5*0.5 (r={nested_r}) should be darker than single 0.5 (r={half_r})"
    );
}

#[test]
fn non_overlapping_siblings_batch_correctly() {
    // Two independent 50% groups side by side are batched into one isolated
    // target and rendered in a single shared pass. The result must be unchanged:
    // each quad at ~50% over black, with an untouched gap between them.
    let Some(bytes) = render(|renderer| {
        renderer.with_opacity(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 30.0,
                height: 80.0,
            },
            0.5,
            |renderer| {
                renderer.fill_quad(quad(10.0, 10.0, 30.0, 80.0), Background::Color(RED));
            },
        );
        renderer.with_opacity(
            Rectangle {
                x: 60.0,
                y: 10.0,
                width: 30.0,
                height: 80.0,
            },
            0.5,
            |renderer| {
                renderer.fill_quad(quad(60.0, 10.0, 30.0, 80.0), Background::Color(GREEN));
            },
        );
    }) else {
        eprintln!("skipping: no wgpu adapter available");
        return;
    };

    let (r, g, _b) = rgb(&bytes, 25, 50);
    assert!(r > 90, "left group should show ~50% red: r={r}");
    assert!(g < 24, "left group should be pure red: g={g}");

    let (r, g, _b) = rgb(&bytes, 75, 50);
    assert!(g > 90, "right group should show ~50% green: g={g}");
    assert!(r < 24, "right group should be pure green: r={r}");

    let (r, g, b) = rgb(&bytes, 50, 50);
    assert!(
        r < 8 && g < 8 && b < 8,
        "gap between groups must stay black"
    );
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
