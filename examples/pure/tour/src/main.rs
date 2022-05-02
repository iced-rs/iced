use iced::alignment;
use iced::pure::widget::{Button, Column, Container, Slider};
use iced::pure::{
    checkbox, column, container, horizontal_space, image, radio, row,
    scrollable, slider, text, text_input, toggler, vertical_space,
};
use iced::pure::{Element, Sandbox};
use iced::{Color, Length, Settings};

pub fn main() -> iced::Result {
    env_logger::init();

    Tour::run(Settings::default())
}

pub struct Tour {
    steps: Steps,
    debug: bool,
}

impl Sandbox for Tour {
    type Message = Message;

    fn new() -> Tour {
        Tour {
            steps: Steps::new(),
            debug: false,
        }
    }

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

        let mut controls = row();

        if steps.has_previous() {
            controls = controls.push(
                button("Back")
                    .on_press(Message::BackPressed)
                    .style(style::Button::Secondary),
            );
        }

        controls = controls.push(horizontal_space(Length::Fill));

        if steps.can_continue() {
            controls = controls.push(
                button("Next")
                    .on_press(Message::NextPressed)
                    .style(style::Button::Primary),
            );
        }

        let content: Element<_> = column()
            .max_width(540)
            .spacing(20)
            .padding(20)
            .push(steps.view(self.debug).map(Message::StepMessage))
            .push(controls)
            .into();

        let content = if self.debug {
            // TODO
            //content.explain(Color::BLACK)
            content
        } else {
            content
        };

        let scrollable =
            scrollable(container(content).width(Length::Fill).center_x());

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
                Step::Image { width: 300 },
                Step::Scrollable,
                Step::TextInput {
                    value: String::new(),
                    is_secure: false,
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

enum Step {
    Welcome,
    Slider { value: u8 },
    RowsAndColumns { layout: Layout, spacing: u16 },
    Text { size: u16, color: Color },
    Radio { selection: Option<Language> },
    Toggler { can_continue: bool },
    Image { width: u16 },
    Scrollable,
    TextInput { value: String, is_secure: bool },
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
    InputChanged(String),
    ToggleSecureInput(bool),
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
            Step::Image { width } => Self::image(*width),
            Step::RowsAndColumns { layout, spacing } => {
                Self::rows_and_columns(*layout, *spacing)
            }
            Step::Scrollable => Self::scrollable(),
            Step::TextInput { value, is_secure } => {
                Self::text_input(value, *is_secure)
            }
            Step::Debugger => Self::debugger(debug),
            Step::End => Self::end(),
        }
        .into()
    }

    fn container(title: &str) -> Column<'a, StepMessage> {
        column().spacing(20).push(text(title).size(50))
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
            Layout::Row => row()
                .spacing(spacing)
                .push(row_radio)
                .push(column_radio)
                .into(),
            Layout::Column => column()
                .spacing(spacing)
                .push(row_radio)
                .push(column_radio)
                .into(),
        };

        let spacing_section = column()
            .spacing(10)
            .push(slider(0..=80, spacing, StepMessage::SpacingChanged))
            .push(
                text(format!("{} px", spacing))
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            );

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
        let size_section = column()
            .padding(20)
            .spacing(20)
            .push("You can change its size:")
            .push(text(format!("This text is {} pixels", size)).size(size))
            .push(Slider::new(10..=70, size, StepMessage::TextSizeChanged));

        let color_sliders = row()
            .spacing(10)
            .push(color_slider(color.r, move |r| Color { r, ..color }))
            .push(color_slider(color.g, move |g| Color { g, ..color }))
            .push(color_slider(color.b, move |b| Color { b, ..color }));

        let color_section = column()
            .padding(20)
            .spacing(20)
            .push("And its color:")
            .push(text(format!("{:?}", color)).color(color))
            .push(color_sliders);

        Self::container("Text")
            .push(
                "Text is probably the most essential widget for your UI. \
                 It will try to adapt to the dimensions of its container.",
            )
            .push(size_section)
            .push(color_section)
    }

    fn radio(selection: Option<Language>) -> Column<'a, StepMessage> {
        let question = column()
            .padding(20)
            .spacing(10)
            .push(text("Iced is written in...").size(24))
            .push(Language::all().iter().cloned().fold(
                column().padding(10).spacing(20),
                |choices, language| {
                    choices.push(radio(
                        language,
                        language,
                        selection,
                        StepMessage::LanguageSelected,
                    ))
                },
            ));

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

    fn image(width: u16) -> Column<'a, StepMessage> {
        Self::container("Image")
            .push("An image that tries to keep its aspect ratio.")
            .push(ferris(width))
            .push(slider(100..=500, width, StepMessage::ImageWidthChanged))
            .push(
                text(format!("Width: {} px", width.to_string()))
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
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
            .push(vertical_space(Length::Units(4096)))
            .push(
                text("You are halfway there!")
                    .width(Length::Fill)
                    .size(30)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .push(vertical_space(Length::Units(4096)))
            .push(ferris(300))
            .push(
                text("You made it!")
                    .width(Length::Fill)
                    .size(50)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
    }

    fn text_input(value: &str, is_secure: bool) -> Column<'a, StepMessage> {
        let text_input = text_input(
            "Type something to continue...",
            value,
            StepMessage::InputChanged,
        )
        .padding(10)
        .size(30);

        Self::container("Text input")
            .push("Use a text input to ask for different kinds of information.")
            .push(if is_secure {
                text_input.password()
            } else {
                text_input
            })
            .push(checkbox(
                "Enable password mode",
                is_secure,
                StepMessage::ToggleSecureInput,
            ))
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
            .push(if cfg!(target_arch = "wasm32") {
                Element::new(
                    text("Not available on web yet!")
                        .color([0.7, 0.7, 0.7])
                        .horizontal_alignment(alignment::Horizontal::Center),
                )
            } else {
                checkbox("Explain layout", debug, StepMessage::DebugToggled)
                    .into()
            })
            .push("Feel free to go back and take a look.")
    }

    fn end() -> Column<'a, StepMessage> {
        Self::container("You reached the end!")
            .push("This tour will be updated as more features are added.")
            .push("Make sure to keep an eye on it!")
    }
}

fn ferris<'a>(width: u16) -> Container<'a, StepMessage> {
    container(
        // This should go away once we unify resource loading on native
        // platforms
        if cfg!(target_arch = "wasm32") {
            image("tour/images/ferris.png")
        } else {
            image(format!(
                "{}/../../tour/images/ferris.png",
                env!("CARGO_MANIFEST_DIR")
            ))
        }
        .width(Length::Units(width)),
    )
    .width(Length::Fill)
    .center_x()
}

fn button<'a, Message: Clone>(label: &str) -> Button<'a, Message> {
    iced::pure::button(
        text(label).horizontal_alignment(alignment::Horizontal::Center),
    )
    .padding(12)
    .width(Length::Units(100))
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

mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Primary,
        Secondary,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
                    Button::Secondary => Color::from_rgb(0.5, 0.5, 0.5),
                })),
                border_radius: 12.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            }
        }
    }
}
