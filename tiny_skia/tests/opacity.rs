//! Verifies that the `opacity` group is composited as a single flattened layer
//! rather than fading each primitive independently.
use iced_tiny_skia::Layer;
use iced_tiny_skia::Renderer;
use iced_tiny_skia::core::renderer::{Quad, Renderer as _, Scale, Settings};
use iced_tiny_skia::core::{Background, Border, Color, Rectangle, Shadow, Size};
use iced_tiny_skia::graphics::Viewport;

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

/// Renders `f` on a black background over the given damage regions and returns
/// the resulting pixmap.
fn render_with_damage(damage: &[Rectangle], f: impl FnOnce(&mut Renderer)) -> tiny_skia::Pixmap {
    let mut renderer = Renderer::new(Settings::default());
    let viewport = Viewport::with_physical_size(Size::new(100, 100), Scale::default());

    renderer.reset(Rectangle::with_size(Size::new(100.0, 100.0)));
    f(&mut renderer);

    let mut pixmap = tiny_skia::Pixmap::new(100, 100).expect("Create pixmap");
    let mut mask = tiny_skia::Mask::new(100, 100).expect("Create mask");

    {
        let mut view = pixmap.as_mut();
        renderer.draw(&mut view, &mut mask, &viewport, damage, Color::BLACK);
    }

    pixmap
}

/// Renders `f` on a black background (full-window damage) and returns the pixmap.
fn render(f: impl FnOnce(&mut Renderer)) -> tiny_skia::Pixmap {
    render_with_damage(&[Rectangle::with_size(Size::new(100.0, 100.0))], f)
}

/// Returns the straight (demultiplied) RGB of a pixel, in iced color space.
///
/// `iced_tiny_skia` stores colors as BGRA (see `engine::into_color`), so the
/// pixmap's red/blue channels are swapped relative to iced's `Color`.
fn rgb(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> (u8, u8, u8) {
    let c = pixmap.pixel(x, y).expect("Pixel in bounds").demultiply();
    (c.blue(), c.green(), c.red())
}

#[test]
fn overlapping_group_does_not_bleed_through() {
    // A red quad (back) and a green quad (front) overlapping in the center, all
    // inside a single 50% opacity group.
    let pixmap = render(|renderer| {
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
    });

    // In the overlap, only the front (green) quad must show at ~50% over black.
    // If opacity were applied per-primitive, the back red quad would bleed
    // through here (red ~63).
    let (r, g, b) = rgb(&pixmap, 50, 50);
    assert!(
        r < 12,
        "red bled through the overlap: got r={r} (expected ~0)"
    );
    assert!(
        (110..=145).contains(&g),
        "front green should be ~half: g={g}"
    );
    assert!(b < 12, "unexpected blue in overlap: b={b}");

    // Back-only region: red at ~50% over black.
    let (r, g, _b) = rgb(&pixmap, 18, 50);
    assert!((110..=145).contains(&r), "back red should be ~half: r={r}");
    assert!(g < 12, "unexpected green in back-only region: g={g}");

    // Front-only region: green at ~50% over black.
    let (r, g, _b) = rgb(&pixmap, 82, 50);
    assert!(r < 12, "unexpected red in front-only region: r={r}");
    assert!(
        (110..=145).contains(&g),
        "front green should be ~half: g={g}"
    );

    // Outside the group: untouched black background.
    assert_eq!(rgb(&pixmap, 2, 2), (0, 0, 0));
}

#[test]
fn full_opacity_is_opaque() {
    // A single opaque quad at opacity 1.0 must remain fully opaque.
    let pixmap = render(|renderer| {
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
    });

    assert_eq!(rgb(&pixmap, 40, 40), (255, 0, 0));
}

#[test]
fn split_damage_regions_do_not_double_composite() {
    // A single 50% red quad spanning two disjoint damage regions. Each region
    // must composite only its own slice of the group, so every pixel is blended
    // exactly once (~50% red over black), never twice.
    let pixmap = render_with_damage(
        &[
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 100.0,
            },
            Rectangle {
                x: 50.0,
                y: 0.0,
                width: 50.0,
                height: 100.0,
            },
        ],
        |renderer| {
            renderer.with_opacity(
                Rectangle {
                    x: 10.0,
                    y: 10.0,
                    width: 80.0,
                    height: 60.0,
                },
                0.5,
                |renderer| {
                    renderer.fill_quad(quad(10.0, 10.0, 80.0, 60.0), Background::Color(RED));
                },
            );
        },
    );

    // Sample either side of the region boundary (x=50); both are single-composited.
    for x in [30, 70] {
        let (r, g, b) = rgb(&pixmap, x, 40);
        assert!(
            (110..=145).contains(&r),
            "x={x}: expected single ~50% composite, got r={r} (double would be ~190)"
        );
        assert!(g < 12 && b < 12, "x={x}: unexpected color g={g} b={b}");
    }
}

#[test]
fn opacity_change_is_damaged() {
    // A changing opacity leaves the primitives identical, so it must be caught by
    // the layer's opacity field; otherwise the software compositor would reuse
    // the stale (previous-opacity) pixels and the UI would not update.
    let previous = Layer {
        bounds: Rectangle {
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 50.0,
        },
        opacity: 1.0,
        ..Layer::default()
    };

    let current = Layer {
        opacity: 0.5,
        ..previous.clone()
    };

    assert!(
        !Layer::damage(&previous, &current).is_empty(),
        "an opacity change must produce damage"
    );

    // Identical layers (same opacity, same content) produce no damage.
    assert!(
        Layer::damage(&previous, &previous.clone()).is_empty(),
        "identical layers should not be damaged"
    );
}

#[test]
fn opacity_change_is_damaged_end_to_end() {
    // Mimics the software compositor: diff the layers of two real draws that
    // differ only in opacity, through iced's own `damage::diff`. This is what
    // decides whether the window repaints when the opacity slider moves.
    use iced_tiny_skia::graphics::damage;

    let layers_at = |opacity: f32| -> Vec<Layer> {
        let mut renderer = Renderer::new(Settings::default());
        renderer.reset(Rectangle::with_size(Size::new(100.0, 100.0)));
        renderer.with_opacity(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 60.0,
            },
            opacity,
            |renderer| {
                renderer.fill_quad(quad(10.0, 10.0, 60.0, 60.0), Background::Color(RED));
            },
        );
        renderer.layers().to_vec()
    };

    let previous = layers_at(0.8);
    let current = layers_at(0.4);

    let damage = damage::diff(
        &previous,
        &current,
        |layer| vec![layer.bounds],
        Layer::damage,
    );

    assert!(
        !damage.is_empty(),
        "changing group opacity (0.8 -> 0.4) must be detected as damage"
    );
}

#[test]
fn nested_opacity_multiplies() {
    // 50% inside 50% -> the inner content ends up at ~25%.
    let pixmap = render(|renderer| {
        renderer.with_opacity(
            Rectangle {
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 60.0,
            },
            0.5,
            |renderer| {
                renderer.with_opacity(
                    Rectangle {
                        x: 10.0,
                        y: 10.0,
                        width: 60.0,
                        height: 60.0,
                    },
                    0.5,
                    |renderer| {
                        renderer.fill_quad(quad(10.0, 10.0, 60.0, 60.0), Background::Color(RED));
                    },
                );
            },
        );
    });

    // 255 * 0.25 ~= 64 over black.
    let (r, _g, _b) = rgb(&pixmap, 40, 40);
    assert!(
        (52..=76).contains(&r),
        "nested opacity should be ~25%: r={r}"
    );
}
