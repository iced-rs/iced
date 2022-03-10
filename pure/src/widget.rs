pub mod image;
pub mod pane_grid;
pub mod progress_bar;
pub mod rule;
pub mod tree;

mod button;
mod checkbox;
mod column;
mod container;
mod element;
mod pick_list;
mod radio;
mod row;
mod scrollable;
mod slider;
mod space;
mod text;
mod text_input;
mod toggler;

pub use button::Button;
pub use checkbox::Checkbox;
pub use column::Column;
pub use container::Container;
pub use element::Element;
pub use image::Image;
pub use pane_grid::PaneGrid;
pub use pick_list::PickList;
pub use progress_bar::ProgressBar;
pub use radio::Radio;
pub use row::Row;
pub use rule::Rule;
pub use scrollable::Scrollable;
pub use slider::Slider;
pub use space::Space;
pub use text::Text;
pub use text_input::TextInput;
pub use toggler::Toggler;
pub use tree::Tree;

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::{Clipboard, Length, Point, Rectangle, Shell};

use std::borrow::Cow;
use std::ops::RangeInclusive;

pub trait Widget<Message, Renderer> {
    fn width(&self) -> Length;

    fn height(&self) -> Length;

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node;

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    );

    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn mouse_interaction(
        &self,
        _state: &Tree,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Idle
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        event::Status::Ignored
    }

    fn overlay<'a>(
        &'a self,
        _state: &'a mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Renderer>> {
        None
    }
}

pub fn container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Container<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    Container::new(content)
}

pub fn column<'a, Message, Renderer>() -> Column<'a, Message, Renderer> {
    Column::new()
}

pub fn row<'a, Message, Renderer>() -> Row<'a, Message, Renderer> {
    Row::new()
}

pub fn scrollable<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Scrollable<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    Scrollable::new(content)
}

pub fn button<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Button<'a, Message, Renderer> {
    Button::new(content)
}

pub fn text<Renderer>(text: impl Into<String>) -> Text<Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    Text::new(text)
}

pub fn checkbox<'a, Message, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Checkbox<'a, Message, Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    Checkbox::new(is_checked, label, f)
}

pub fn radio<'a, Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Radio<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::text::Renderer,
    V: Copy + Eq,
{
    Radio::new(value, label, selected, on_click)
}

pub fn toggler<'a, Message, Renderer>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Toggler<'a, Message, Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    Toggler::new(is_checked, label, f)
}

pub fn text_input<'a, Message, Renderer>(
    placeholder: &str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::text::Renderer,
{
    TextInput::new(placeholder, value, on_change)
}

pub fn slider<'a, Message, T>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> Slider<'a, T, Message>
where
    Message: Clone,
    T: Copy + From<u8> + std::cmp::PartialOrd,
{
    Slider::new(range, value, on_change)
}

pub fn pick_list<'a, Message, Renderer, T>(
    options: impl Into<Cow<'a, [T]>>,
    selected: Option<T>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, Message, Renderer>
where
    T: ToString + Eq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: iced_native::text::Renderer,
{
    PickList::new(options, selected, on_selected)
}

pub fn image<Handle>(handle: impl Into<Handle>) -> Image<Handle> {
    Image::new(handle.into())
}

pub fn horizontal_space(width: Length) -> Space {
    Space::with_width(width)
}

pub fn vertical_space(height: Length) -> Space {
    Space::with_height(height)
}

/// Creates a horizontal [`Rule`] with the given height.
pub fn horizontal_rule<'a>(height: u16) -> Rule<'a> {
    Rule::horizontal(height)
}

/// Creates a vertical [`Rule`] with the given width.
pub fn vertical_rule<'a>(width: u16) -> Rule<'a> {
    Rule::horizontal(width)
}

/// Creates a new [`ProgressBar`].
///
/// It expects:
///   * an inclusive range of possible values
///   * the current value of the [`ProgressBar`]
pub fn progress_bar<'a>(
    range: RangeInclusive<f32>,
    value: f32,
) -> ProgressBar<'a> {
    ProgressBar::new(range, value)
}
