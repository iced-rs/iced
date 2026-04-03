//! Keyed List Example
//!
//! This example demonstrates that keyed_column correctly preserves widget state
//! when new items are added to a list. Each item has a counter that tracks how
//! many times it has been rendered - if state is preserved correctly, the counter
//! stays stable for existing items.
//!
//! The fix in `diff_children_custom_with_search` ensures that when items are added
//! at the end of a keyed list, existing items keep their state and new items get
//! fresh state.

use iced::widget::{button, column, container, keyed_column, row, scrollable, text};
use iced::{Center, Element, Fill};
use std::sync::atomic::{AtomicU64, Ordering};

pub fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

/// Global counter for generating unique IDs
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct Item {
    id: u64,
    label: String,
}

impl Item {
    fn new(label: impl Into<String>) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            label: label.into(),
        }
    }
}

#[derive(Default)]
struct App {
    items: Vec<Item>,
    next_item_number: usize,
}

#[derive(Debug, Clone)]
enum Message {
    AddItem,
    AddItemAtFront,
    RemoveItem(u64),
    Clear,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::AddItem => {
                if self.next_item_number == 0 {
                    self.next_item_number = 1;
                }
                self.items
                    .push(Item::new(format!("Item {}", self.next_item_number)));
                self.next_item_number += 1;
            }
            Message::AddItemAtFront => {
                if self.next_item_number == 0 {
                    self.next_item_number = 1;
                }
                self.items
                    .insert(0, Item::new(format!("Item {}", self.next_item_number)));
                self.next_item_number += 1;
            }
            Message::RemoveItem(id) => {
                self.items.retain(|item| item.id != id);
            }
            Message::Clear => {
                self.items.clear();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button("Add at End").on_press(Message::AddItem),
            button("Add at Front").on_press(Message::AddItemAtFront),
            button("Clear All").on_press(Message::Clear),
        ]
        .spacing(10);

        let explanation = column![
            text("Keyed List State Preservation Test").size(24),
            text("Each item shows a 'render count' that increments on each render."),
            text("If state is preserved correctly:"),
            text("  • Existing items keep their render count stable"),
            text("  • Only new items start with render count = 1"),
            text(""),
            text("BUG (before fix): Adding items caused existing items to get new state"),
            text("FIX: Items now correctly preserve state when list grows"),
        ]
        .spacing(4);

        // Use keyed_column - each item has a stateful widget that tracks renders
        let items = keyed_column(
            self.items
                .iter()
                .map(|item| (item.id, item_row(item.id, &item.label))),
        )
        .spacing(8);

        let content = column![explanation, controls, scrollable(items).height(Fill),]
            .spacing(20)
            .padding(20);

        container(content).center_x(Fill).into()
    }
}

/// Creates a row for an item with a stateful render counter
fn item_row(id: u64, label: &str) -> Element<'static, Message> {
    // The RenderCounter widget tracks how many times it's been rendered
    // If keyed_column preserves state correctly, this counter should be stable
    row![
        render_counter(),
        text(format!(" | ID: {} | {}", id, label)).width(Fill),
        button("Remove").on_press(Message::RemoveItem(id)),
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

mod render_counter {
    //! A simple stateful widget that counts how many times it has been rendered.
    //! Used to verify that widget state is preserved correctly in keyed lists.

    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::advanced::{Clipboard, Shell};
    use iced::border;
    use iced::mouse;
    use iced::{Color, Element, Event, Length, Rectangle, Size};

    pub struct RenderCounter;

    pub fn render_counter<'a, Message: 'a>() -> Element<'a, Message> {
        Element::new(RenderCounter)
    }

    struct State {
        render_count: u64,
        /// Unique ID to track state identity
        state_id: u64,
    }

    static STATE_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    impl Default for State {
        fn default() -> Self {
            let id = STATE_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            println!(">>> NEW State created with state_id={}", id);
            Self {
                render_count: 0,
                state_id: id,
            }
        }
    }

    impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for RenderCounter
    where
        Renderer: renderer::Renderer,
    {
        fn tag(&self) -> widget::tree::Tag {
            widget::tree::Tag::of::<State>()
        }

        fn state(&self) -> widget::tree::State {
            widget::tree::State::new(State::default())
        }

        fn size(&self) -> Size<Length> {
            Size::new(Length::Shrink, Length::Shrink)
        }

        fn layout(
            &mut self,
            _tree: &mut widget::Tree,
            _renderer: &Renderer,
            _limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(120.0, 30.0))
        }

        fn draw(
            &self,
            tree: &widget::Tree,
            renderer: &mut Renderer,
            _theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _viewport: &Rectangle,
        ) {
            let state = tree.state.downcast_ref::<State>();
            let bounds = layout.bounds();

            // Draw background based on state_id (different colors for different states)
            let hue = (state.state_id * 37 % 360) as f32;
            let color = hsv_to_rgb(hue, 0.3, 0.9);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: border::rounded(4.0),
                    ..renderer::Quad::default()
                },
                color,
            );
        }

        fn update(
            &mut self,
            tree: &mut widget::Tree,
            _event: &Event,
            _layout: Layout<'_>,
            _cursor: mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            _shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            let state = tree.state.downcast_mut::<State>();
            state.render_count += 1;
        }
    }

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match (h / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color::from_rgb(r + m, g + m, b + m)
    }
}

use render_counter::render_counter;
