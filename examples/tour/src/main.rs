use iced::alignment::{self, Alignment};
use iced::widget::{
    button, checkbox, column, container, horizontal_space, image, radio, row,
    scrollable, slider, text, text_input, toggler, vertical_space,
};
use iced::widget::{Button, Column, Container, Slider};
use iced::{Color, Element, Font, Length, Pixels};

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("Initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::program(Tour::title, Tour::update, Tour::view)
        .centered()
        .run()
}

#[derive(Default)]
pub struct Tour {
    steps: Steps,
    debug: bool,
}

impl Tour {
    fn title(&self) -> String {
        format!("{} - Iced", self.steps.title())
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::BackPressed => {
                self.steps.go_back();
            }
            Message::NextPressed => {
                self.steps.advance();
            }
            Message::StepMessage(step_msg) => {
                self.steps.update(step_msg, &mut self.debug);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let Tour { steps, .. } = self;

        let controls =
            row![]
                .push_maybe(steps.has_previous().then(|| {
                    padded_button("Back")
                        .on_press(Message::BackPressed)
                        .style(button::secondary)
                }))
                .push(horizontal_space())
                .push_maybe(steps.can_continue().then(|| {
                    padded_button("Next").on_press(Message::NextPressed)
                }));

        let content: Element<_> = column![
            steps.view(self.debug).map(Message::StepMessage),
            controls,
        ]
        .max_width(540)
        .spacing(20)
        .padding(20)
        .into();

        let scrollable = scrollable(
            container(if self.debug {
                content.explain(Color::BLACK)
            } else {
                content
            })
            .width(Length::Fill)
            .center_x(),
        );

        container(scrollable).height(Length::Fill).center_y().into()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    BackPressed,
    NextPressed,
    StepMessage(StepMessage),
}

struct Steps {
    steps: Vec<Step>,
    current: usize,
}

impl Steps {
    fn new() -> Steps {
        Steps {
            steps: vec![
                Step::Welcome,
                Step::Slider { value: 50 },
                Step::RowsAndColumns {
                    layout: Layout::Row,
                    spacing: 20,
                },
                Step::Text {
                    size: 30,
                    color: Color::BLACK,
                },
                Step::Radio { selection: None },
                Step::Toggler {
                    can_continue: false,
                },
                Step::Image {
                    width: 300,
                    filter_method: image::FilterMethod::Linear,
                },
                Step::Scrollable,
                Step::TextInput {
                    value: String::new(),
                    is_secure: false,
                    is_showing_icon: false,
                },
                Step::Debugger,
                Step::End,
            ],
            current: 0,
        }
    }

    fn update(&mut self, msg: StepMessage, debug: &mut bool) {
        self.steps[self.current].update(msg, debug);
    }

    fn view(&self, debug: bool) -> Element<StepMessage> {
        self.steps[self.current].view(debug)
    }

    fn advance(&mut self) {
        if self.can_continue() {
            self.current += 1;
        }
    }

    fn go_back(&mut self) {
        if self.has_previous() {
            self.current -= 1;
        }
    }

    fn has_previous(&self) -> bool {
        self.current > 0
    }

    fn can_continue(&self) -> bool {
        self.current + 1 < self.steps.len()
            && self.steps[self.current].can_continue()
    }

    fn title(&self) -> &str {
        self.steps[self.current].title()
    }
}

impl Default for Steps {
    fn default() -> Self {
        Steps::new()
    }
}

enum Step {
    Welcome,
    Slider {
        value: u8,
    },
    RowsAndColumns {
        layout: Layout,
        spacing: u16,
    },
    Text {
        size: u16,
        color: Color,
    },
    Radio {
        selection: Option<Language>,
    },
    Toggler {
        can_continue: bool,
    },
    Image {
        width: u16,
        filter_method: image::FilterMethod,
    },
    Scrollable,
    TextInput {
        value: String,
        is_secure: bool,
        is_showing_icon: bool,
    },
    Debugger,
    End,
}

#[derive(Debug, Clone)]
pub enum StepMessage {
    SliderChanged(u8),
    LayoutChanged(Layout),
    SpacingChanged(u16),
    TextSizeChanged(u16),
    TextColorChanged(Color),
    LanguageSelected(Language),
    ImageWidthChanged(u16),
    ImageUseNearestToggled(bool),
    InputChanged(String),
    ToggleSecureInput(bool),
    ToggleTextInputIcon(bool),
    DebugToggled(bool),
    TogglerChanged(bool),
}

impl<'a> Step {
    fn update(&mut self, msg: StepMessage, debug: &mut bool) {
        match msg {
            StepMessage::DebugToggled(value) => {
                if let Step::Debugger = self {
                    *debug = value;
                }
            }
            StepMessage::LanguageSelected(language) => {
                if let Step::Radio { selection } = self {
                    *selection = Some(language);
                }
            }
            StepMessage::SliderChanged(new_value) => {
                if let Step::Slider { value, .. } = self {
                    *value = new_value;
                }
            }
            StepMessage::TextSizeChanged(new_size) => {
                if let Step::Text { size, .. } = self {
                    *size = new_size;
                }
            }
            StepMessage::TextColorChanged(new_color) => {
                if let Step::Text { color, .. } = self {
                    *color = new_color;
                }
            }
            StepMessage::LayoutChanged(new_layout) => {
                if let Step::RowsAndColumns { layout, .. } = self {
                    *layout = new_layout;
                }
            }
            StepMessage::SpacingChanged(new_spacing) => {
                if let Step::RowsAndColumns { spacing, .. } = self {
                    *spacing = new_spacing;
                }
            }
            StepMessage::ImageWidthChanged(new_width) => {
                if let Step::Image { width, .. } = self {
                    *width = new_width;
                }
            }
            StepMessage::ImageUseNearestToggled(use_nearest) => {
                if let Step::Image { filter_method, .. } = self {
                    *filter_method = if use_nearest {
                        image::FilterMethod::Nearest
                    } else {
                        image::FilterMethod::Linear
                    };
                }
            }
            StepMessage::InputChanged(new_value) => {
                if let Step::TextInput { value, .. } = self {
                    *value = new_value;
                }
            }
            StepMessage::ToggleSecureInput(toggle) => {
                if let Step::TextInput { is_secure, .. } = self {
                    *is_secure = toggle;
                }
            }
            StepMessage::TogglerChanged(value) => {
                if let Step::Toggler { can_continue, .. } = self {
                    *can_continue = value;
                }
            }
            StepMessage::ToggleTextInputIcon(toggle) => {
                if let Step::TextInput {
                    is_showing_icon, ..
                } = self
                {
                    *is_showing_icon = toggle;
                }
            }
        };
    }

    fn title(&self) -> &str {
        match self {
            Step::Welcome => "Welcome",
            Step::Radio { .. } => "Radio button",
            Step::Toggler { .. } => "Toggler",
            Step::Slider { .. } => "Slider",
            Step::Text { .. } => "Text",
            Step::Image { .. } => "Image",
            Step::RowsAndColumns { .. } => "Rows and columns",
            Step::Scrollable => "Scrollable",
            Step::TextInput { .. } => "Text input",
            Step::Debugger => "Debugger",
            Step::End => "End",
        }
    }

    fn can_continue(&self) -> bool {
        match self {
            Step::Welcome => true,
            Step::Radio { selection } => *selection == Some(Language::Rust),
            Step::Toggler { can_continue } => *can_continue,
            Step::Slider { .. } => true,
            Step::Text { .. } => true,
            Step::Image { .. } => true,
            Step::RowsAndColumns { .. } => true,
            Step::Scrollable => true,
            Step::TextInput { value, .. } => !value.is_empty(),
            Step::Debugger => true,
            Step::End => false,
        }
    }

    fn view(&self, debug: bool) -> Element<StepMessage> {
        match self {
            Step::Welcome => Self::welcome(),
            Step::Radio { selection } => Self::radio(*selection),
            Step::Toggler { can_continue } => Self::toggler(*can_continue),
            Step::Slider { value } => Self::slider(*value),
            Step::Text { size, color } => Self::text(*size, *color),
            Step::Image {
                width,
                filter_method,
            } => Self::image(*width, *filter_method),
            Step::RowsAndColumns { layout, spacing } => {
                Self::rows_and_columns(*layout, *spacing)
            }
            Step::Scrollable => Self::scrollable(),
            Step::TextInput {
                value,
                is_secure,
                is_showing_icon,
            } => Self::text_input(value, *is_secure, *is_showing_icon),
            Step::Debugger => Self::debugger(debug),
            Step::End => Self::end(),
        }
        .into()
    }

    fn container(title: &str) -> Column<'_, StepMessage> {
        column![text(title).size(50)].spacing(20)
    }

    fn welcome() -> Column<'a, StepMessage> {
        Self::container("Welcome!")
            .push(
                "This is a simple tour meant to showcase a bunch of widgets \
                 that can be easily implemented on top of Iced.",
            )
            .push(
                "Iced is a cross-platform GUI library for Rust focused on \
                 simplicity and type-safety. It is heavily inspired by Elm.",
            )
            .push(
                "It was originally born as part of Coffee, an opinionated \
                 2D game engine for Rust.",
            )
            .push(
                "On native platforms, Iced provides by default a renderer \
                 built on top of wgpu, a graphics library supporting Vulkan, \
                 Metal, DX11, and DX12.",
            )
            .push(
                "Additionally, this tour can also run on WebAssembly thanks \
                 to dodrio, an experimental VDOM library for Rust.",
            )
            .push(
                "You will need to interact with the UI in order to reach the \
                 end!",
            )
    }

    fn slider(value: u8) -> Column<'a, StepMessage> {
        Self::container("Slider")
            .push(
                "A slider allows you to smoothly select a value from a range \
                 of values.",
            )
            .push(
                "The following slider lets you choose an integer from \
                 0 to 100:",
            )
            .push(slider(0..=100, value, StepMessage::SliderChanged))
            .push(
                text(value.to_string())
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
    }

    fn rows_and_columns(
        layout: Layout,
        spacing: u16,
    ) -> Column<'a, StepMessage> {
        let row_radio =
            radio("Row", Layout::Row, Some(layout), StepMessage::LayoutChanged);

        let column_radio = radio(
            "Column",
            Layout::Column,
            Some(layout),
            StepMessage::LayoutChanged,
        );

        let layout_section: Element<_> = match layout {
            Layout::Row => {
                row![row_radio, column_radio].spacing(spacing).into()
            }
            Layout::Column => {
                column![row_radio, column_radio].spacing(spacing).into()
            }
        };

        let spacing_section = column![
            slider(0..=80, spacing, StepMessage::SpacingChanged),
            text(format!("{spacing} px"))
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center),
        ]
        .spacing(10);

        Self::container("Rows and columns")
            .spacing(spacing)
            .push(
                "Iced uses a layout model based on flexbox to position UI \
                 elements.",
            )
            .push(
                "Rows and columns can be used to distribute content \
                 horizontally or vertically, respectively.",
            )
            .push(layout_section)
            .push("You can also easily change the spacing between elements:")
            .push(spacing_section)
    }

    fn text(size: u16, color: Color) -> Column<'a, StepMessage> {
        let size_section = column![
            "You can change its size:",
            text(format!("This text is {size} pixels")).size(size),
            slider(10..=70, size, StepMessage::TextSizeChanged),
        ]
        .padding(20)
        .spacing(20);

        let color_sliders = row![
            color_slider(color.r, move |r| Color { r, ..color }),
            color_slider(color.g, move |g| Color { g, ..color }),
            color_slider(color.b, move |b| Color { b, ..color }),
        ]
        .spacing(10);

        let color_section = column![
            "And its color:",
            text(format!("{color:?}")).color(color),
            color_sliders,
        ]
        .padding(20)
        .spacing(20);

        Self::container("Text")
            .push(
                "Text is probably the most essential widget for your UI. \
                 It will try to adapt to the dimensions of its container.",
            )
            .push(size_section)
            .push(color_section)
    }

    fn radio(selection: Option<Language>) -> Column<'a, StepMessage> {
        let question = column![
            text("Iced is written in...").size(24),
            column(
                Language::all()
                    .iter()
                    .copied()
                    .map(|language| {
                        radio(
                            language,
                            language,
                            selection,
                            StepMessage::LanguageSelected,
                        )
                    })
                    .map(Element::from)
            )
            .spacing(10)
        ]
        .padding(20)
        .spacing(10);

        Self::container("Radio button")
            .push(
                "A radio button is normally used to represent a choice... \
                 Surprise test!",
            )
            .push(question)
            .push(
                "Iced works very well with iterators! The list above is \
                 basically created by folding a column over the different \
                 choices, creating a radio button for each one of them!",
            )
    }

    fn toggler(can_continue: bool) -> Column<'a, StepMessage> {
        Self::container("Toggler")
            .push("A toggler is mostly used to enable or disable something.")
            .push(
                Container::new(toggler(
                    "Toggle me to continue...".to_owned(),
                    can_continue,
                    StepMessage::TogglerChanged,
                ))
                .padding([0, 40]),
            )
    }

    fn image(
        width: u16,
        filter_method: image::FilterMethod,
    ) -> Column<'a, StepMessage> {
        Self::container("Image")
            .push("An image that tries to keep its aspect ratio.")
            .push(ferris(width, filter_method))
            .push(slider(100..=500, width, StepMessage::ImageWidthChanged))
            .push(
                text(format!("Width: {width} px"))
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .push(
                checkbox(
                    "Use nearest interpolation",
                    filter_method == image::FilterMethod::Nearest,
                )
                .on_toggle(StepMessage::ImageUseNearestToggled),
            )
            .align_items(Alignment::Center)
    }

    fn scrollable() -> Column<'a, StepMessage> {
        Self::container("Scrollable")
            .push(
                "Iced supports scrollable content. Try it out! Find the \
                 button further below.",
            )
            .push(
                text("Tip: You can use the scrollbar to scroll down faster!")
                    .size(16),
            )
            .push(vertical_space().height(4096))
            .push(
                text("You are halfway there!")
                    .width(Length::Fill)
                    .size(30)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .push(vertical_space().height(4096))
            .push(ferris(300, image::FilterMethod::Linear))
            .push(
                text("You made it!")
                    .width(Length::Fill)
                    .size(50)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
    }

    fn text_input(
        value: &str,
        is_secure: bool,
        is_showing_icon: bool,
    ) -> Column<'_, StepMessage> {
        let mut text_input = text_input("Type something to continue...", value)
            .on_input(StepMessage::InputChanged)
            .padding(10)
            .size(30);

        if is_showing_icon {
            text_input = text_input.icon(text_input::Icon {
                font: Font::default(),
                code_point: '🚀',
                size: Some(Pixels(28.0)),
                spacing: 10.0,
                side: text_input::Side::Right,
            });
        }

        Self::container("Text input")
            .push("Use a text input to ask for different kinds of information.")
            .push(text_input.secure(is_secure))
            .push(
                checkbox("Enable password mode", is_secure)
                    .on_toggle(StepMessage::ToggleSecureInput),
            )
            .push(
                checkbox("Show icon", is_showing_icon)
                    .on_toggle(StepMessage::ToggleTextInputIcon),
            )
            .push(
                "A text input produces a message every time it changes. It is \
                 very easy to keep track of its contents:",
            )
            .push(
                text(if value.is_empty() {
                    "You have not typed anything yet..."
                } else {
                    value
                })
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center),
            )
    }

    fn debugger(debug: bool) -> Column<'a, StepMessage> {
        Self::container("Debugger")
            .push(
                "You can ask Iced to visually explain the layouting of the \
                 different elements comprising your UI!",
            )
            .push(
                "Give it a shot! Check the following checkbox to be able to \
                 see element boundaries.",
            )
            .push(
                checkbox("Explain layout", debug)
                    .on_toggle(StepMessage::DebugToggled),
            )
            .push("Feel free to go back and take a look.")
    }

    fn end() -> Column<'a, StepMessage> {
        Self::container("You reached the end!")
            .push("This tour will be updated as more features are added.")
            .push("Make sure to keep an eye on it!")
    }
}

fn ferris<'a>(
    width: u16,
    filter_method: image::FilterMethod,
) -> Container<'a, StepMessage> {
    container(
        // This should go away once we unify resource loading on native
        // platforms
        if cfg!(target_arch = "wasm32") {
            image("tour/images/ferris.png")
        } else {
            image(format!("{}/images/ferris.png", env!("CARGO_MANIFEST_DIR")))
        }
        .filter_method(filter_method)
        .width(width),
    )
    .width(Length::Fill)
    .center_x()
}

fn padded_button<Message: Clone>(label: &str) -> Button<'_, Message> {
    button(text(label)).padding([12, 24])
}

fn color_slider<'a>(
    component: f32,
    update: impl Fn(f32) -> Color + 'a,
) -> Slider<'a, f64, StepMessage> {
    slider(0.0..=1.0, f64::from(component), move |c| {
        StepMessage::TextColorChanged(update(c as f32))
    })
    .step(0.01)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Elm,
    Ruby,
    Haskell,
    C,
    Other,
}

impl Language {
    fn all() -> [Language; 6] {
        [
            Language::C,
            Language::Elm,
            Language::Ruby,
            Language::Haskell,
            Language::Rust,
            Language::Other,
        ]
    }
}

impl From<Language> for String {
    fn from(language: Language) -> String {
        String::from(match language {
            Language::Rust => "Rust",
            Language::Elm => "Elm",
            Language::Ruby => "Ruby",
            Language::Haskell => "Haskell",
            Language::C => "C",
            Language::Other => "Other",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Row,
    Column,
}
