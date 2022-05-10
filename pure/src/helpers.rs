//! Helper functions to create pure widgets.
use crate::widget;
use crate::Element;

use iced_native::Length;
use std::borrow::Cow;
use std::ops::RangeInclusive;

/// Creates a new [`Container`] with the provided content.
///
/// [`Container`]: widget::Container
pub fn container<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> widget::Container<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    widget::Container::new(content)
}

/// Creates a new [`Column`].
///
/// [`Column`]: widget::Column
pub fn column<'a, Message, Renderer>() -> widget::Column<'a, Message, Renderer>
{
    widget::Column::new()
}

/// Creates a new [`Row`].
///
/// [`Row`]: widget::Row
pub fn row<'a, Message, Renderer>() -> widget::Row<'a, Message, Renderer> {
    widget::Row::new()
}

/// Creates a new [`Scrollable`] with the provided content.
///
/// [`Scrollable`]: widget::Scrollable
pub fn scrollable<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> widget::Scrollable<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    widget::Scrollable::new(content)
}

/// Creates a new [`Button`] with the provided content.
///
/// [`Button`]: widget::Button
pub fn button<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> widget::Button<'a, Message, Renderer> {
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
    Renderer: iced_native::text::Renderer,
{
    widget::Tooltip::new(content, tooltip, position)
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: widget::Text
pub fn text<Renderer>(text: impl Into<String>) -> widget::Text<Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    widget::Text::new(text)
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
    Renderer: iced_native::text::Renderer,
{
    widget::Checkbox::new(is_checked, label, f)
}

/// Creates a new [`Radio`].
///
/// [`Radio`]: widget::Radio
pub fn radio<'a, Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> widget::Radio<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::text::Renderer,
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
    Renderer: iced_native::text::Renderer,
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
    Renderer: iced_native::text::Renderer,
{
    widget::TextInput::new(placeholder, value, on_change)
}

/// Creates a new [`Slider`].
///
/// [`Slider`]: widget::Slider
pub fn slider<'a, Message, T>(
    range: std::ops::RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> Message + 'a,
) -> widget::Slider<'a, T, Message>
where
    Message: Clone,
    T: Copy + From<u8> + std::cmp::PartialOrd,
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
    Renderer: iced_native::text::Renderer,
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
pub fn horizontal_rule<'a>(height: u16) -> widget::Rule<'a> {
    widget::Rule::horizontal(height)
}

/// Creates a vertical [`Rule`] with the given width.
///
/// [`Rule`]: widget::Rule
pub fn vertical_rule<'a>(width: u16) -> widget::Rule<'a> {
    widget::Rule::vertical(width)
}

/// Creates a new [`ProgressBar`].
///
/// It expects:
///   * an inclusive range of possible values, and
///   * the current value of the [`ProgressBar`].
///
/// [`ProgressBar`]: widget::ProgressBar
pub fn progress_bar<'a>(
    range: RangeInclusive<f32>,
    value: f32,
) -> widget::ProgressBar<'a> {
    widget::ProgressBar::new(range, value)
}
