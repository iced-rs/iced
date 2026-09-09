//! Benchmarks for the overlay machinery.
//!
//! The stock benchmarks (`cpu`, `wgpu`) don't contain a single overlay, so
//! they don't exercise this code path at all. Here we drive a
//! [`UserInterface`] with deeply nested widget trees where every nesting
//! level contributes its own overlay (think scrollables with tooltips,
//! menus, pick lists, ...), and measure the steady-state per-frame cost of
//! `update` + `draw` — the two phases that query, sort, lay out, and draw
//! overlays.
//!
//! The software `tiny-skia` renderer is used on purpose: it's a real,
//! deferred (layers + quad recording) renderer with no GPU involved, so the
//! measured cost is the true per-frame overlay cost, without driver/GPU
//! noise.
#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use iced_core::overlay;
use iced_core::widget::{Tree, Widget};
use iced_core::{
    Color, Element, Event, Layout, Length, Point, Rectangle, Renderer as _, Shell, Size, Theme,
    Vector, layout, mouse, renderer, shell, window,
};
use iced_runtime::user_interface::{self, Cache, UserInterface};

type Renderer = iced_tiny_skia::Renderer;
type Message = ();

const BOUNDS: Size = Size::new(1024.0, 1024.0);
const OVERLAY_SIZE: f32 = 64.0;
const CURSOR: Point = Point::new(10.0, 10.0);

/// A small leaf widget that produces a single overlay.
struct OverlayPoint {
    index: f32,
}

impl OverlayPoint {
    const fn new(index: f32) -> Self {
        Self { index }
    }
}

/// The overlay produced by [`OverlayPoint`].
struct PointOverlay {
    index: f32,
}

impl overlay::Overlay<Message, Theme, Renderer> for PointOverlay {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::new(OVERLAY_SIZE, OVERLAY_SIZE))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..Default::default()
            },
            Color::from_rgb8(64, 96, 160),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(position) = cursor.position() else {
            return mouse::Interaction::None;
        };

        if layout.bounds().contains(position) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn index(&self) -> f32 {
        self.index
    }
}

impl Widget<Message, Theme, Renderer> for OverlayPoint {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(OVERLAY_SIZE), Length::Fixed(OVERLAY_SIZE))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(OVERLAY_SIZE, OVERLAY_SIZE))
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut Tree,
        _layout: Layout<'a>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        vec![overlay::Element::new(Box::new(PointOverlay {
            index: self.index,
        }))]
    }
}

/// A fill-sized wrapper that produces an overlay of its own and delegates
/// everything else to its child.
///
/// This mimics real widgets like a `Scrollable` with a tooltip or an open
/// menu: each nesting level contributes one overlay on top of the overlays
/// of the content it wraps.
struct OverlayFrame {
    index: f32,
    child: Element<'static, Message, Theme, Renderer>,
}

impl Widget<Message, Theme, Renderer> for OverlayFrame {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.child));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .child
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(child.size(), vec![child])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(child_layout) = layout.children().next() {
            self.child.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Some(child_layout) = layout.children().next() {
            self.child.as_widget_mut().update(
                &mut tree.children[0],
                event,
                child_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        let mut overlays = vec![overlay::Element::new(Box::new(PointOverlay {
            index: self.index,
        }))];

        overlays.extend(self.child.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        ));

        overlays
    }
}

/// A minimal horizontal row with overlapping children, used for the wide
/// scenario (shallow tree, many overlays).
struct Row {
    children: Vec<Element<'static, Message, Theme, Renderer>>,
}

impl Widget<Message, Theme, Renderer> for Row {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut self.children);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut height: f32 = 0.0;
        let mut children = Vec::with_capacity(self.children.len());

        for (child, child_tree) in self.children.iter_mut().zip(&mut tree.children) {
            let node = child.as_widget_mut().layout(child_tree, renderer, limits);
            height = height.max(node.size().height);
            children.push(node);
        }

        layout::Node::with_children(Size::new(OVERLAY_SIZE, height), children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, child_tree), child_layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.as_widget().draw(
                child_tree,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, child_tree), child_layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                child_tree,
                event,
                child_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// Builds a view `levels` deep where every level contributes its own
/// overlay (one overlay per level, plus the leaf's).
///
/// Indices are assigned in descending document order, so the runtime has to
/// do real sorting work to restore the correct z-order.
fn deep_view(levels: usize) -> Element<'static, Message, Theme, Renderer> {
    let mut element = Element::new(OverlayPoint::new(levels as f32));

    for level in (0..levels).rev() {
        element = Element::new(OverlayFrame {
            index: level as f32,
            child: element,
        });
    }

    element
}

/// Builds a shallow view with `count` overlays in a flat row.
fn wide_view(count: usize) -> Element<'static, Message, Theme, Renderer> {
    let children = (0..count)
        .map(|index| Element::new(OverlayPoint::new(index as f32)))
        .collect();

    Element::new(Row { children })
}

fn bench(c: &mut Criterion, name: &str, view: Element<'static, Message, Theme, Renderer>) {
    let mut renderer = Renderer::new(renderer::Settings::default());

    let mut ui = UserInterface::build(view, BOUNDS, Cache::new(), &mut renderer);

    let events = [Event::Mouse(mouse::Event::CursorMoved { position: CURSOR })];
    let cursor = mouse::Cursor::Available(CURSOR);
    let theme = Theme::Dark;
    let style = renderer::Style::default();
    let window = window::Headless;
    let waker = shell::Waker::noop();
    let mut messages = shell::Bus::<Message>::new();

    // Sanity check: the view must be stable (nothing invalidates itself), so
    // the measured cost is the steady-state per-frame cost.
    let (state, statuses) = ui.update(
        &window,
        &waker,
        &events,
        cursor,
        &mut renderer,
        &mut messages,
    );

    assert_eq!(statuses.len(), events.len());
    assert!(
        matches!(
            state,
            user_interface::State::Updated {
                has_layout_changed: false,
                ..
            }
        ),
        "the bench view must not invalidate itself"
    );

    let _ = c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ui.update(
                &window,
                &waker,
                &events,
                cursor,
                &mut renderer,
                &mut messages,
            );
            ui.draw(&mut renderer, &theme, &style, cursor);
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench(c, "overlay — deep 32", deep_view(32));
    bench(c, "overlay — deep 128", deep_view(128));
    bench(c, "overlay — deep 512", deep_view(512));
    bench(c, "overlay — wide 256", wide_view(256));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
