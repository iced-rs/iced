//! Show toggle controls using checkboxes.
use crate::core::alignment;
use crate::core::event::{self, Event};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Clipboard, Element, Layout, Length, Pixels, Rectangle, Shell, Size, Widget,
};

pub use crate::style::checkbox::{Appearance, StyleSheet};

/// A box that can be checked.
///
/// # Example
///
/// ```no_run
/// # type Checkbox<'a, Message> =
/// #     iced_widget::Checkbox<'a, Message, iced_widget::style::Theme, iced_widget::renderer::Renderer>;
/// #
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// let is_checked = true;
///
/// Checkbox::new("Toggle me!", is_checked).on_toggle(Message::CheckboxToggled);
/// ```
///
/// ![Checkbox drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/checkbox.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Checkbox<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Theme: StyleSheet + crate::text::StyleSheet,
    Renderer: text::Renderer,
{
    is_checked: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    label: String,
    width: Length,
    size: f32,
    spacing: f32,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    icon: Icon<Renderer::Font>,
    style: <Theme as StyleSheet>::Style,
}

impl<'a, Message, Theme, Renderer> Checkbox<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
    Theme: StyleSheet + crate::text::StyleSheet,
{
    /// The default size of a [`Checkbox`].
    const DEFAULT_SIZE: f32 = 20.0;

    /// The default spacing of a [`Checkbox`].
    const DEFAULT_SPACING: f32 = 10.0;

    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * the label of the [`Checkbox`]
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    pub fn new(label: impl Into<String>, is_checked: bool) -> Self {
        Checkbox {
            is_checked,
            on_toggle: None,
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            spacing: Self::DEFAULT_SPACING,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::Basic,
            font: None,
            icon: Icon {
                font: Renderer::ICON_FONT,
                code_point: Renderer::CHECKMARK_ICON,
                size: None,
                line_height: text::LineHeight::default(),
                shaping: text::Shaping::Basic,
            },
            style: Default::default(),
        }
    }

    /// Sets the function that will be called when the [`Checkbox`] is toggled.
    /// It will receive the new state of the [`Checkbox`] and must produce a
    /// `Message`.
    ///
    /// Unless `on_toggle` is called, the [`Checkbox`] will be disabled.
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(bool) -> Message,
    {
        self.on_toggle = Some(Box::new(f));
        self
    }

    /// Sets the function that will be called when the [`Checkbox`] is toggled,
    /// if `Some`.
    ///
    /// If `None`, the checkbox will be disabled.
    pub fn on_toggle_maybe<F>(mut self, f: Option<F>) -> Self
    where
        F: Fn(bool) -> Message + 'a,
    {
        self.on_toggle = f.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the size of the [`Checkbox`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    /// Sets the width of the [`Checkbox`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the spacing between the [`Checkbox`] and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the text size of the [`Checkbox`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the [`Checkbox`].
    pub fn text_line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Checkbox`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the [`Renderer::Font`] of the text of the [`Checkbox`].
    ///
    /// [`Renderer::Font`]: crate::core::text::Renderer
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Icon`] of the [`Checkbox`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.icon = icon;
        self
    }

    /// Sets the style of the [`Checkbox`].
    pub fn style(
        mut self,
        style: impl Into<<Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Checkbox<'a, Message, Theme, Renderer>
where
    Theme: StyleSheet + crate::text::StyleSheet,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<widget::text::State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(widget::text::State::<Renderer::Paragraph>::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::next_to_each_other(
            &limits.width(self.width),
            self.spacing,
            |_| layout::Node::new(Size::new(self.size, self.size)),
            |limits| {
                let state = tree
                    .state
                    .downcast_mut::<widget::text::State<Renderer::Paragraph>>();

                widget::text::layout(
                    state,
                    renderer,
                    limits,
                    self.width,
                    Length::Shrink,
                    &self.label,
                    self.text_line_height,
                    self.text_size,
                    self.font,
                    alignment::Horizontal::Left,
                    alignment::Vertical::Top,
                    self.text_shaping,
                )
            },
        )
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let mouse_over = cursor.is_over(layout.bounds());

                if mouse_over {
                    if let Some(on_toggle) = &self.on_toggle {
                        shell.publish((on_toggle)(!self.is_checked));
                        return event::Status::Captured;
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) && self.on_toggle.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
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
        let is_mouse_over = cursor.is_over(layout.bounds());
        let is_disabled = self.on_toggle.is_none();

        let mut children = layout.children();

        let custom_style = if is_disabled {
            theme.disabled(&self.style, self.is_checked)
        } else if is_mouse_over {
            theme.hovered(&self.style, self.is_checked)
        } else {
            theme.active(&self.style, self.is_checked)
        };

        {
            let layout = children.next().unwrap();
            let bounds = layout.bounds();

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: custom_style.border,
                    ..renderer::Quad::default()
                },
                custom_style.background,
            );

            let Icon {
                font,
                code_point,
                size,
                line_height,
                shaping,
            } = &self.icon;
            let size = size.unwrap_or(Pixels(bounds.height * 0.7));

            if self.is_checked {
                renderer.fill_text(
                    text::Text {
                        content: &code_point.to_string(),
                        font: *font,
                        size,
                        line_height: *line_height,
                        bounds: bounds.size(),
                        horizontal_alignment: alignment::Horizontal::Center,
                        vertical_alignment: alignment::Vertical::Center,
                        shaping: *shaping,
                    },
                    bounds.center(),
                    custom_style.icon_color,
                    *viewport,
                );
            }
        }

        {
            let label_layout = children.next().unwrap();

            crate::text::draw(
                renderer,
                style,
                label_layout,
                tree.state.downcast_ref(),
                crate::text::Appearance {
                    color: custom_style.text_color,
                },
                viewport,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Checkbox<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + StyleSheet + crate::text::StyleSheet,
    Renderer: 'a + text::Renderer,
{
    fn from(
        checkbox: Checkbox<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(checkbox)
    }
}

/// The icon in a [`Checkbox`].
#[derive(Debug, Clone, PartialEq)]
pub struct Icon<Font> {
    /// Font that will be used to display the `code_point`,
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// Font size of the content.
    pub size: Option<Pixels>,
    /// The line height of the icon.
    pub line_height: text::LineHeight,
    /// The shaping strategy of the icon.
    pub shaping: text::Shaping,
}
