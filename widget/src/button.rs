//! Allow your users to perform actions by pressing a button.
use crate::core::event::{self, Event};
use crate::core::keyboard::{self, key};
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::theme::palette;
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Background, Border, Clipboard, Color, Element, Layout, Length, Padding,
    Rectangle, Shadow, Shell, Size, Theme, Vector, Widget,
};

/// A generic widget that produces a message when pressed.
///
/// ```no_run
/// # type Button<'a, Message> = iced_widget::Button<'a, Message>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// let button = Button::new("Press me!").on_press(Message::ButtonPressed);
/// ```
///
/// If a [`Button::on_press`] handler is not set, the resulting [`Button`] will
/// be disabled:
///
/// ```
/// # type Button<'a, Message> = iced_widget::Button<'a, Message>;
/// #
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn disabled_button<'a>() -> Button<'a, Message> {
///     Button::new("I'm disabled!")
/// }
///
/// fn enabled_button<'a>() -> Button<'a, Message> {
///     disabled_button().on_press(Message::ButtonPressed)
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Button<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
{
    id: Option<widget::Id>,
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    style: Style<'a, Theme>,
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self
    where
        Theme: DefaultStyle + 'a,
    {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Button {
            id: Some(widget::Id::unique()),
            content,
            on_press: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: DEFAULT_PADDING,
            clip: false,
            style: Box::new(Theme::default_style),
        }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Button`] will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press;
        self
    }

    /// Sets the style variant of this [`Button`].
    pub fn style(
        mut self,
        style: impl Fn(&Theme, Status) -> Appearance + 'a,
    ) -> Self {
        self.style = Box::new(style);
        self
    }

    /// Sets whether the contents of the [`Button`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Focus {
    is_window_focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_focused: bool,
    is_pressed: bool,
    is_active: bool,
}

impl State {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn pressed(&mut self) {
        self.is_focused = true;
        self.is_pressed = true;
        self.is_active = true;
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn reset(&mut self) {
        self.is_pressed = false;
        self.is_active = false;
        self.is_focused = false;
    }
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        State::is_focused(&self)
    }

    fn focus(&mut self) {
        State::reset(self);
        State::focus(self)
    }

    fn unfocus(&mut self) {
        Self::reset(self)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + crate::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(
            limits,
            self.width,
            self.height,
            self.padding,
            |limits| {
                self.content.as_widget().layout(
                    &mut tree.children[0],
                    renderer,
                    limits,
                )
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<State>();
        operation.focusable(state, self.id.as_ref().map(|id| id));
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    let state = tree.state.downcast_mut::<State>();
                    if cursor.is_over(bounds) {
                        state.pressed();

                        return event::Status::Captured;
                    } else {
                        state.reset();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = self.on_press.clone() {
                    let state = tree.state.downcast_mut::<State>();

                    if state.is_pressed {
                        state.is_pressed = false;

                        let bounds = layout.bounds();

                        if cursor.is_over(bounds) {
                            shell.publish(on_press);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                let state = tree.state.downcast_mut::<State>();

                state.is_pressed = false;
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                match key.as_ref() {
                    keyboard::Key::Named(key::Named::Space)
                    | keyboard::Key::Named(key::Named::Enter) => {
                        let state = tree.state.downcast_mut::<State>();
                        if state.is_focused() {
                            if let Some(on_press) = self.on_press.clone() {
                                shell.publish(on_press);
                                state.pressed();
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let is_mouse_over = cursor.is_over(bounds);

        let state = tree.state.downcast_ref::<State>();
        let status = if self.on_press.is_none() {
            Status::Disabled
        } else if is_mouse_over {
            let state = tree.state.downcast_ref::<State>();

            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            if state.is_focused() {
                Status::Focused(state.is_active)
            } else {
                Status::Active
            }
        };

        let styling = (self.style)(theme, status);

        if styling.background.is_some()
            || styling.border.width > 0.0
            || styling.shadow.color.a > 0.0
        {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: styling.border,
                    shadow: styling.shadow,
                },
                styling
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: styling.text_color,
            },
            content_layout,
            cursor,
            &viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over && self.on_press.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: crate::core::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}

/// The default [`Padding`] of a [`Button`].
pub(crate) const DEFAULT_PADDING: Padding = Padding {
    top: 5.0,
    bottom: 5.0,
    right: 10.0,
    left: 10.0,
};

/// The possible status of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Button`] can be pressed.
    Active,
    /// The [`Button`] can be pressed and it is being hovered.
    Hovered,
    /// The [`Button`] is being pressed.
    Pressed,
    /// The [`Button`] cannot be pressed.
    Disabled,
    /// The [`Button`] can be pressed and it is being focused.
    Focused(bool),
}

/// The appearance of a button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    /// The [`Background`] of the button.
    pub background: Option<Background>,
    /// The text [`Color`] of the button.
    pub text_color: Color,
    /// The [`Border`] of the button.
    pub border: Border,
    /// The [`Shadow`] of the button.
    pub shadow: Shadow,
}

impl Appearance {
    /// Updates the [`Appearance`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            background: None,
            text_color: Color::BLACK,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

/// The style of a [`Button`].
pub type Style<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Appearance + 'a>;

/// The default style of a [`Button`].
pub trait DefaultStyle {
    /// Returns the default style of a [`Button`].
    fn default_style(&self, status: Status) -> Appearance;
}

impl DefaultStyle for Theme {
    fn default_style(&self, status: Status) -> Appearance {
        primary(self, status)
    }
}

impl DefaultStyle for Appearance {
    fn default_style(&self, _status: Status) -> Appearance {
        *self
    }
}

impl DefaultStyle for Color {
    fn default_style(&self, _status: Status) -> Appearance {
        Appearance::default().with_background(*self)
    }
}

/// A primary button; denoting a main action.
pub fn primary(theme: &Theme, status: Status) -> Appearance {
    let palette = theme.extended_palette();
    let base = styled(palette.primary.strong);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Focused(active) => focused(base, active, palette.is_dark),
        Status::Hovered => Appearance {
            background: Some(Background::Color(palette.primary.base.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A secondary button; denoting a complementary action.
pub fn secondary(theme: &Theme, status: Status) -> Appearance {
    let palette = theme.extended_palette();
    let base = styled(palette.secondary.base);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Focused(active) => focused(base, active, palette.is_dark),
        Status::Hovered => Appearance {
            background: Some(Background::Color(palette.secondary.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A success button; denoting a good outcome.
pub fn success(theme: &Theme, status: Status) -> Appearance {
    let palette = theme.extended_palette();
    let base = styled(palette.success.base);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Focused(active) => focused(base, active, palette.is_dark),
        Status::Hovered => Appearance {
            background: Some(Background::Color(palette.success.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A danger button; denoting a destructive action.
pub fn danger(theme: &Theme, status: Status) -> Appearance {
    let palette = theme.extended_palette();
    let base = styled(palette.danger.base);

    match status {
        Status::Active | Status::Pressed => base,
        Status::Focused(active) => focused(base, active, palette.is_dark),
        Status::Hovered => Appearance {
            background: Some(Background::Color(palette.danger.strong.color)),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

/// A text button; useful for links.
pub fn text(theme: &Theme, status: Status) -> Appearance {
    let palette = theme.extended_palette();

    let base = Appearance {
        text_color: palette.background.base.text,
        ..Appearance::default()
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Focused(active) => focused(base, active, palette.is_dark),
        Status::Hovered => Appearance {
            text_color: palette.background.base.text.scale_alpha(0.8),
            ..base
        },
        Status::Disabled => disabled(base),
    }
}

fn styled(pair: palette::Pair) -> Appearance {
    Appearance {
        background: Some(Background::Color(pair.color)),
        text_color: pair.text,
        border: Border::rounded(2),
        ..Appearance::default()
    }
}

fn disabled(appearance: Appearance) -> Appearance {
    Appearance {
        background: appearance
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: appearance.text_color.scale_alpha(0.5),
        ..appearance
    }
}

fn focused(
    appearance: Appearance,
    active: bool,
    is_dark_theme: bool,
) -> Appearance {
    if active {
        return appearance;
    }

    Appearance {
        border: appearance
            .border
            .with_color(if is_dark_theme {
                Color::WHITE
            } else {
                Color::BLACK
            })
            .with_width(2),
        ..appearance
    }
}
