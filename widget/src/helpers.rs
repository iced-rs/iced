//! Helper functions to create pure widgets.
use crate::button::{self, Button};
use crate::checkbox::{self, Checkbox};
use crate::container::{self, Container};
use crate::core;
use crate::core::widget::operation;
use crate::core::{Element, Length, Pixels};
use crate::overlay;
use crate::pick_list::{self, PickList};
use crate::progress_bar::{self, ProgressBar};
use crate::radio::{self, Radio};
use crate::rule::{self, Rule};
use crate::runtime::Command;
use crate::scrollable::{self, Scrollable};
use crate::slider::{self, Slider};
use crate::text::{self, Text};
use crate::text_input::{self, TextInput};
use crate::toggler::{self, Toggler};
use crate::tooltip::{self, Tooltip};
use crate::{Column, MouseArea, Row, Space, VerticalSlider};

#[cfg(feature = "wayland")]
use crate::dnd_listener::DndListener;
#[cfg(feature = "wayland")]
use crate::dnd_source::DndSource;

use std::borrow::Cow;
use std::ops::RangeInclusive;

/// Creates a [`Column`] with the given children.
///
/// [`Column`]: widget::Column
#[macro_export]
macro_rules! column {
    () => (
        $crate::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Column::with_children(vec![$($crate::core::Element::from($x)),+])
    );
}

/// Creates a [`Row`] with the given children.
///
/// [`Row`]: widget::Row
#[macro_export]
macro_rules! row {
    () => (
        $crate::Row::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Row::with_children(vec![$($crate::core::Element::from($x)),+])
    );
}

/// Creates a new [`Container`] with the provided content.
///
/// [`Container`]: widget::Container
pub fn container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Container<'a, Message, Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: container::StyleSheet,
{
    Container::new(content)
}

/// Creates a new [`Column`] with the given children.
///
/// [`Column`]: widget::Column
pub fn column<Message, Renderer>(
    children: Vec<Element<'_, Message, Renderer>>,
) -> Column<'_, Message, Renderer> {
    Column::with_children(children)
}

/// Creates a new [`Row`] with the given children.
///
/// [`Row`]: widget::Row
pub fn row<Message, Renderer>(
    children: Vec<Element<'_, Message, Renderer>>,
) -> Row<'_, Message, Renderer> {
    Row::with_children(children)
}

/// Creates a new [`Scrollable`] with the provided content.
///
/// [`Scrollable`]: widget::Scrollable
pub fn scrollable<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Scrollable<'a, Message, Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: scrollable::StyleSheet,
{
    Scrollable::new(content)
}

/// Creates a new [`Button`] with the provided content.
///
/// [`Button`]: widget::Button
pub fn button<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Button<'a, Message, Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: button::StyleSheet,
    <Renderer::Theme as button::StyleSheet>::Style: Default,
{
    Button::new(content)
}

/// Creates a new [`Tooltip`] with the provided content, tooltip text, and [`tooltip::Position`].
///
/// [`Tooltip`]: widget::Tooltip
/// [`tooltip::Position`]: widget::tooltip::Position
pub fn tooltip<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    tooltip: impl ToString,
    position: tooltip::Position,
) -> crate::Tooltip<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: container::StyleSheet + text::StyleSheet,
{
    Tooltip::new(content, tooltip.to_string(), position)
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: widget::Text
pub fn text<'a, Renderer>(text: impl ToString) -> Text<'a, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: text::StyleSheet,
{
    Text::new(text.to_string())
}

/// Creates a new [`Checkbox`].
///
/// [`Checkbox`]: widget::Checkbox
pub fn checkbox<'a, Message, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Checkbox<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: checkbox::StyleSheet + text::StyleSheet,
{
    Checkbox::new(label, is_checked, f)
}

/// Creates a new [`Radio`].
///
/// [`Radio`]: widget::Radio
pub fn radio<Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Radio<Message, Renderer>
where
    Message: Clone,
    Renderer: core::text::Renderer,
    Renderer::Theme: radio::StyleSheet,
    V: Copy + Eq,
{
    Radio::new(label, value, selected, on_click)
}

/// Creates a new [`Toggler`].
///
/// [`Toggler`]: widget::Toggler
pub fn toggler<'a, Message, Renderer>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> Toggler<'a, Message, Renderer>
where
    Renderer: core::text::Renderer,
    Renderer::Theme: toggler::StyleSheet,
{
    Toggler::new(label, is_checked, f)
}

/// Creates a new [`TextInput`].
///
/// [`TextInput`]: widget::TextInput
pub fn text_input<'a, Message, Renderer>(
    placeholder: &str,
    value: &str,
) -> TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: core::text::Renderer,
    Renderer::Theme: text_input::StyleSheet,
{
    TextInput::new(placeholder, value)
}

/// Creates a new [`Slider`].
///
/// [`Slider`]: widget::Slider
pub fn slider<'a, T, Message, Renderer>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> Slider<'a, T, Message, Renderer>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Renderer: core::Renderer,
    Renderer::Theme: slider::StyleSheet,
{
    Slider::new(range, value, on_change)
}

/// Creates a new [`VerticalSlider`].
///
/// [`VerticalSlider`]: widget::VerticalSlider
pub fn vertical_slider<'a, T, Message, Renderer>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> VerticalSlider<'a, T, Message, Renderer>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Renderer: core::Renderer,
    Renderer::Theme: slider::StyleSheet,
{
    VerticalSlider::new(range, value, on_change)
}

/// Creates a new [`PickList`].
///
/// [`PickList`]: widget::PickList
pub fn pick_list<'a, Message, Renderer, T>(
    options: impl Into<Cow<'a, [T]>>,
    selected: Option<T>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, Message, Renderer>
where
    T: ToString + Eq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: core::text::Renderer,
    Renderer::Theme: pick_list::StyleSheet
        + scrollable::StyleSheet
        + overlay::menu::StyleSheet
        + container::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as pick_list::StyleSheet>::Style>,
{
    PickList::new(options, selected, on_selected)
}

/// Creates a new horizontal [`Space`] with the given [`Length`].
///
/// [`Space`]: widget::Space
pub fn horizontal_space(width: impl Into<Length>) -> Space {
    Space::with_width(width)
}

/// Creates a new vertical [`Space`] with the given [`Length`].
///
/// [`Space`]: widget::Space
pub fn vertical_space(height: impl Into<Length>) -> Space {
    Space::with_height(height)
}

/// Creates a horizontal [`Rule`] with the given height.
///
/// [`Rule`]: widget::Rule
pub fn horizontal_rule<Renderer>(height: impl Into<Pixels>) -> Rule<Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: rule::StyleSheet,
{
    Rule::horizontal(height)
}

/// Creates a vertical [`Rule`] with the given width.
///
/// [`Rule`]: widget::Rule
pub fn vertical_rule<Renderer>(width: impl Into<Pixels>) -> Rule<Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: rule::StyleSheet,
{
    Rule::vertical(width)
}

/// Creates a new [`ProgressBar`].
///
/// It expects:
///   * an inclusive range of possible values, and
///   * the current value of the [`ProgressBar`].
///
/// [`ProgressBar`]: widget::ProgressBar
pub fn progress_bar<Renderer>(
    range: RangeInclusive<f32>,
    value: f32,
) -> ProgressBar<Renderer>
where
    Renderer: core::Renderer,
    Renderer::Theme: progress_bar::StyleSheet,
{
    ProgressBar::new(range, value)
}

/// Creates a new [`Image`].
///
/// [`Image`]: widget::Image
#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
pub fn image<'a, Handle>(
    handle: impl Into<Handle>,
) -> crate::Image<'a, Handle> {
    crate::Image::new(handle.into())
}

/// Creates a new [`Svg`] widget from the given [`Handle`].
///
/// [`Svg`]: widget::Svg
/// [`Handle`]: widget::svg::Handle
#[cfg(feature = "svg")]
#[cfg_attr(docsrs, doc(cfg(feature = "svg")))]
pub fn svg<'a, Renderer>(
    handle: impl Into<core::svg::Handle>,
) -> crate::Svg<'a, Renderer>
where
    Renderer: core::svg::Renderer,
    Renderer::Theme: crate::svg::StyleSheet,
{
    crate::Svg::new(handle)
}

/// Creates a new [`Canvas`].
#[cfg(feature = "canvas")]
#[cfg_attr(docsrs, doc(cfg(feature = "canvas")))]
pub fn canvas<P, Message, Renderer>(
    program: P,
) -> crate::Canvas<P, Message, Renderer>
where
    Renderer: crate::graphics::geometry::Renderer,
    P: crate::canvas::Program<Message, Renderer>,
{
    crate::Canvas::new(program)
}

/// Focuses the previous focusable widget.
pub fn focus_previous<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(operation::focusable::focus_previous())
}

/// Focuses the next focusable widget.
pub fn focus_next<Message>() -> Command<Message>
where
    Message: 'static,
{
    Command::widget(operation::focusable::focus_next())
}

/// A container intercepting mouse events.
pub fn mouse_area<'a, Message, Renderer>(
    widget: impl Into<Element<'a, Message, Renderer>>,
) -> MouseArea<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    MouseArea::new(widget)
}

#[cfg(feature = "wayland")]
/// A container for a dnd source
pub fn dnd_source<'a, Message, Renderer>(
    widget: impl Into<Element<'a, Message, Renderer>>,
) -> DndSource<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    DndSource::new(widget)
}

#[cfg(feature = "wayland")]
/// A container for a dnd target
pub fn dnd_listener<'a, Message, Renderer>(
    widget: impl Into<Element<'a, Message, Renderer>>,
) -> DndListener<'a, Message, Renderer>
where
    Renderer: core::Renderer,
{
    DndListener::new(widget)
}
