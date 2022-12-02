//! Helper functions to create pure widgets.
use crate::overlay;
use crate::widget;
use crate::{Element, Length};

use std::borrow::Cow;
use std::ops::RangeInclusive;

/// Creates a [`Column`] with the given children.
///
/// [`Column`]: widget::Column
#[macro_export]
macro_rules! column {
    () => (
        $crate::widget::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Column::with_children(vec![$($crate::Element::from($x)),+])
    );
}

/// Creates a [`Row`] with the given children.
///
/// [`Row`]: widget::Row
#[macro_export]
macro_rules! row {
    () => (
        $crate::widget::Row::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Row::with_children(vec![$($crate::Element::from($x)),+])
    );
}

/// Creates a new [`Container`] with the provided content.
///
/// [`Container`]: widget::Container
pub fn container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> widget::Container<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: widget::container::StyleSheet,
{
    widget::Container::new(content)
}

/// Creates a new [`Column`] with the given children.
///
/// [`Column`]: widget::Column
pub fn column<Message, Renderer>(
    children: Vec<Element<'_, Message, Renderer>>,
) -> widget::Column<'_, Message, Renderer> {
    widget::Column::with_children(children)
}

/// Creates a new [`Row`] with the given children.
///
/// [`Row`]: widget::Row
pub fn row<Message, Renderer>(
    children: Vec<Element<'_, Message, Renderer>>,
) -> widget::Row<'_, Message, Renderer> {
    widget::Row::with_children(children)
}

/// Creates a new [`Scrollable`] with the provided content.
///
/// [`Scrollable`]: widget::Scrollable
pub fn scrollable<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> widget::Scrollable<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: widget::scrollable::StyleSheet,
{
    widget::Scrollable::new(content)
}

/// Creates a new [`Button`] with the provided content.
///
/// [`Button`]: widget::Button
pub fn button<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> widget::Button<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: widget::button::StyleSheet,
    <Renderer::Theme as widget::button::StyleSheet>::Style: Default,
{
    widget::Button::new(content)
}

/// Creates a new [`Tooltip`] with the provided content, tooltip text, and [`tooltip::Position`].
///
/// [`Tooltip`]: widget::Tooltip
/// [`tooltip::Position`]: widget::tooltip::Position
pub fn tooltip<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    tooltip: impl ToString,
    position: widget::tooltip::Position,
) -> widget::Tooltip<'a, Message, Renderer>
where
    Renderer: crate::text::Renderer,
    Renderer::Theme: widget::container::StyleSheet + widget::text::StyleSheet,
{
    widget::Tooltip::new(content, tooltip.to_string(), position)
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: widget::Text
pub fn text<'a, Renderer>(text: impl ToString) -> widget::Text<'a, Renderer>
where
    Renderer: crate::text::Renderer,
    Renderer::Theme: widget::text::StyleSheet,
{
    widget::Text::new(text.to_string())
}

/// Creates a new [`Checkbox`].
///
/// [`Checkbox`]: widget::Checkbox
pub fn checkbox<'a, Message, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> widget::Checkbox<'a, Message, Renderer>
where
    Renderer: crate::text::Renderer,
    Renderer::Theme: widget::checkbox::StyleSheet + widget::text::StyleSheet,
{
    widget::Checkbox::new(is_checked, label, f)
}

/// Creates a new [`Radio`].
///
/// [`Radio`]: widget::Radio
pub fn radio<Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> widget::Radio<Message, Renderer>
where
    Message: Clone,
    Renderer: crate::text::Renderer,
    Renderer::Theme: widget::radio::StyleSheet,
    V: Copy + Eq,
{
    widget::Radio::new(value, label, selected, on_click)
}

/// Creates a new [`Toggler`].
///
/// [`Toggler`]: widget::Toggler
pub fn toggler<'a, Message, Renderer>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> widget::Toggler<'a, Message, Renderer>
where
    Renderer: crate::text::Renderer,
    Renderer::Theme: widget::toggler::StyleSheet,
{
    widget::Toggler::new(is_checked, label, f)
}

/// Creates a new [`TextInput`].
///
/// [`TextInput`]: widget::TextInput
pub fn text_input<'a, Message, Renderer>(
    placeholder: &str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> widget::TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: crate::text::Renderer,
    Renderer::Theme: widget::text_input::StyleSheet,
{
    widget::TextInput::new(placeholder, value, on_change)
}

/// Creates a new [`Slider`].
///
/// [`Slider`]: widget::Slider
pub fn slider<'a, T, Message, Renderer>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> widget::Slider<'a, T, Message, Renderer>
where
    T: Copy + From<u8> + std::cmp::PartialOrd,
    Message: Clone,
    Renderer: crate::Renderer,
    Renderer::Theme: widget::slider::StyleSheet,
{
    widget::Slider::new(range, value, on_change)
}

/// Creates a new [`PickList`].
///
/// [`PickList`]: widget::PickList
pub fn pick_list<'a, Message, Renderer, T>(
    options: impl Into<Cow<'a, [T]>>,
    selected: Option<T>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> widget::PickList<'a, T, Message, Renderer>
where
    T: ToString + Eq + 'static,
    [T]: ToOwned<Owned = Vec<T>>,
    Renderer: crate::text::Renderer,
    Renderer::Theme: widget::pick_list::StyleSheet
        + widget::scrollable::StyleSheet
        + overlay::menu::StyleSheet
        + widget::container::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as widget::pick_list::StyleSheet>::Style>,
{
    widget::PickList::new(options, selected, on_selected)
}

/// Creates a new [`Image`].
///
/// [`Image`]: widget::Image
pub fn image<Handle>(handle: impl Into<Handle>) -> widget::Image<Handle> {
    widget::Image::new(handle.into())
}

/// Creates a new horizontal [`Space`] with the given [`Length`].
///
/// [`Space`]: widget::Space
pub fn horizontal_space(width: Length) -> widget::Space {
    widget::Space::with_width(width)
}

/// Creates a new vertical [`Space`] with the given [`Length`].
///
/// [`Space`]: widget::Space
pub fn vertical_space(height: Length) -> widget::Space {
    widget::Space::with_height(height)
}

/// Creates a horizontal [`Rule`] with the given height.
///
/// [`Rule`]: widget::Rule
pub fn horizontal_rule<Renderer>(height: u16) -> widget::Rule<Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: widget::rule::StyleSheet,
{
    widget::Rule::horizontal(height)
}

/// Creates a vertical [`Rule`] with the given width.
///
/// [`Rule`]: widget::Rule
pub fn vertical_rule<Renderer>(width: u16) -> widget::Rule<Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: widget::rule::StyleSheet,
{
    widget::Rule::vertical(width)
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
) -> widget::ProgressBar<Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: widget::progress_bar::StyleSheet,
{
    widget::ProgressBar::new(range, value)
}

/// Creates a new [`Svg`] widget from the given [`Handle`].
///
/// [`Svg`]: widget::Svg
/// [`Handle`]: widget::svg::Handle
pub fn svg(handle: impl Into<widget::svg::Handle>) -> widget::Svg {
    widget::Svg::new(handle)
}
