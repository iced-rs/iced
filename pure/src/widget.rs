pub mod button;
pub mod checkbox;
pub mod container;
pub mod image;
pub mod pane_grid;
pub mod pick_list;
pub mod progress_bar;
pub mod radio;
pub mod rule;
pub mod scrollable;
pub mod slider;
pub mod svg;
pub mod text_input;
pub mod toggler;
pub mod tooltip;
pub mod tree;

mod column;
mod row;
mod space;
mod text;

pub use button::Button;
pub use checkbox::Checkbox;
pub use column::Column;
pub use container::Container;
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
pub use svg::Svg;
pub use text::Text;
pub use text_input::TextInput;
pub use toggler::Toggler;
pub use tooltip::{Position, Tooltip};
pub use tree::Tree;

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::{Clipboard, Length, Point, Rectangle, Shell};

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
