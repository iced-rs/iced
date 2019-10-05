use iced::{
    button, slider, text::HorizontalAlignment, Align, Button, Checkbox, Color,
    Column, Element, Image, Length, Radio, Row, Slider, Text, UserInterface,
};

pub fn main() {
    let tour = Tour::new();

    tour.run();
}

pub struct Tour {
    steps: Steps,
    back_button: button::State,
    next_button: button::State,
    debug: bool,
}

impl Tour {
    pub fn new() -> Tour {
        Tour {
            steps: Steps::new(),
            back_button: button::State::new(),
            next_button: button::State::new(),
            debug: false,
        }
    }
}

impl UserInterface for Tour {
    type Message = Message;

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

    fn view(&mut self) -> Element<Message> {
        let Tour {
            steps,
            back_button,
            next_button,
            ..
        } = self;

        let mut controls = Row::new();

        if steps.has_previous() {
            controls = controls.push(
                Button::new(back_button, "Back")
                    .on_press(Message::BackPressed)
                    .class(button::Class::Secondary),
            );
        }

        controls = controls.push(Column::new());

        if steps.can_continue() {
            controls = controls.push(
                Button::new(next_button, "Next").on_press(Message::NextPressed),
            );
        }

        let element: Element<_> = Column::new()
            .max_width(Length::Units(500))
            .spacing(20)
            .padding(20)
            .push(steps.view(self.debug).map(Message::StepMessage))
            .push(controls)
            .into();

        if self.debug {
            element.explain(Color::BLACK)
        } else {
            element
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
                Step::Slider {
                    state: slider::State::new(),
                    value: 50,
                },
                Step::RowsAndColumns {
                    layout: Layout::Row,
                    spacing_slider: slider::State::new(),
                    spacing: 20,
                },
                Step::Text {
                    size_slider: slider::State::new(),
                    size: 30,
                    color_sliders: [slider::State::new(); 3],
                    color: Color::BLACK,
                },
                Step::Radio { selection: None },
                Step::Image {
                    width: 300,
                    slider: slider::State::new(),
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

    fn view(&mut self, debug: bool) -> Element<StepMessage> {
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
}

enum Step {
    Welcome,
    Slider {
        state: slider::State,
        value: u16,
    },
    RowsAndColumns {
        layout: Layout,
        spacing_slider: slider::State,
        spacing: u16,
    },
    Text {
        size_slider: slider::State,
        size: u16,
        color_sliders: [slider::State; 3],
        color: Color,
    },
    Radio {
        selection: Option<Language>,
    },
    Image {
        width: u16,
        slider: slider::State,
    },
    Debugger,
    End,
}

#[derive(Debug, Clone, Copy)]
pub enum StepMessage {
    SliderChanged(f32),
    LayoutChanged(Layout),
    SpacingChanged(f32),
    TextSizeChanged(f32),
    TextColorChanged(Color),
    LanguageSelected(Language),
    ImageWidthChanged(f32),
    DebugToggled(bool),
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
                    *value = new_value.round() as u16;
                }
            }
            StepMessage::TextSizeChanged(new_size) => {
                if let Step::Text { size, .. } = self {
                    *size = new_size.round() as u16;
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
                    *spacing = new_spacing.round() as u16;
                }
            }
            StepMessage::ImageWidthChanged(new_width) => {
                if let Step::Image { width, .. } = self {
                    *width = new_width.round() as u16;
                }
            }
        };
    }

    fn can_continue(&self) -> bool {
        match self {
            Step::Welcome => true,
            Step::Radio { selection } => *selection == Some(Language::Rust),
            Step::Slider { .. } => true,
            Step::Text { .. } => true,
            Step::Image { .. } => true,
            Step::RowsAndColumns { .. } => true,
            Step::Debugger => true,
            Step::End => false,
        }
    }

    fn view(&mut self, debug: bool) -> Element<StepMessage> {
        match self {
            Step::Welcome => Self::welcome().into(),
            Step::Radio { selection } => Self::radio(*selection).into(),
            Step::Slider { state, value } => Self::slider(state, *value).into(),
            Step::Text {
                size_slider,
                size,
                color_sliders,
                color,
            } => Self::text(size_slider, *size, color_sliders, *color).into(),
            Step::Image { width, slider } => Self::image(*width, slider).into(),
            Step::RowsAndColumns {
                layout,
                spacing_slider,
                spacing,
            } => {
                Self::rows_and_columns(*layout, spacing_slider, *spacing).into()
            }
            Step::Debugger => Self::debugger(debug).into(),
            Step::End => Self::end().into(),
        }
    }

    fn container(title: &str) -> Column<'a, StepMessage> {
        Column::new()
            .spacing(20)
            .align_items(Align::Stretch)
            .push(Text::new(title).size(50))
    }

    fn welcome() -> Column<'a, StepMessage> {
        Self::container("Welcome!")
            .push(Text::new(
                "This a simple tour meant to showcase a bunch of widgets that \
                 can be easily implemented on top of Iced.",
            ))
            .push(Text::new(
                "Iced is a renderer-agnostic GUI library for Rust focused on \
                 simplicity and type-safety. It is heavily inspired by Elm.",
            ))
            .push(Text::new(
                "It was originally born as part of Coffee, an opinionated \
                 2D game engine for Rust.",
            ))
            .push(Text::new(
                "Iced does not provide a built-in renderer. This example runs \
                 on WebAssembly using dodrio, an experimental VDOM library \
                 for Rust.",
            ))
            .push(Text::new(
                "You will need to interact with the UI in order to reach the \
                 end!",
            ))
    }

    fn slider(
        state: &'a mut slider::State,
        value: u16,
    ) -> Column<'a, StepMessage> {
        Self::container("Slider")
            .push(Text::new(
                "A slider allows you to smoothly select a value from a range \
                 of values.",
            ))
            .push(Text::new(
                "The following slider lets you choose an integer from \
                 0 to 100:",
            ))
            .push(Slider::new(
                state,
                0.0..=100.0,
                value as f32,
                StepMessage::SliderChanged,
            ))
            .push(
                Text::new(&value.to_string())
                    .horizontal_alignment(HorizontalAlignment::Center),
            )
    }

    fn rows_and_columns(
        layout: Layout,
        spacing_slider: &'a mut slider::State,
        spacing: u16,
    ) -> Column<'a, StepMessage> {
        let row_radio = Radio::new(
            Layout::Row,
            "Row",
            Some(layout),
            StepMessage::LayoutChanged,
        );

        let column_radio = Radio::new(
            Layout::Column,
            "Column",
            Some(layout),
            StepMessage::LayoutChanged,
        );

        let layout_section: Element<_> = match layout {
            Layout::Row => Row::new()
                .spacing(spacing)
                .push(row_radio)
                .push(column_radio)
                .into(),
            Layout::Column => Column::new()
                .spacing(spacing)
                .push(row_radio)
                .push(column_radio)
                .into(),
        };

        let spacing_section = Column::new()
            .spacing(10)
            .push(Slider::new(
                spacing_slider,
                0.0..=80.0,
                spacing as f32,
                StepMessage::SpacingChanged,
            ))
            .push(
                Text::new(&format!("{} px", spacing))
                    .horizontal_alignment(HorizontalAlignment::Center),
            );

        Self::container("Rows and columns")
            .spacing(spacing)
            .push(Text::new(
                "Iced uses a layout model based on flexbox to position UI \
                 elements.",
            ))
            .push(Text::new(
                "Rows and columns can be used to distribute content \
                 horizontally or vertically, respectively.",
            ))
            .push(layout_section)
            .push(Text::new(
                "You can also easily change the spacing between elements:",
            ))
            .push(spacing_section)
    }

    fn text(
        size_slider: &'a mut slider::State,
        size: u16,
        color_sliders: &'a mut [slider::State; 3],
        color: Color,
    ) -> Column<'a, StepMessage> {
        let size_section = Column::new()
            .padding(20)
            .spacing(20)
            .push(Text::new("You can change its size:"))
            .push(
                Text::new(&format!("This text is {} pixels", size)).size(size),
            )
            .push(Slider::new(
                size_slider,
                10.0..=70.0,
                size as f32,
                StepMessage::TextSizeChanged,
            ));

        let [red, green, blue] = color_sliders;
        let color_section = Column::new()
            .padding(20)
            .spacing(20)
            .push(Text::new("And its color:"))
            .push(Text::new(&format!("{:?}", color)).color(color))
            .push(
                Row::new()
                    .spacing(10)
                    .push(Slider::new(red, 0.0..=1.0, color.r, move |r| {
                        StepMessage::TextColorChanged(Color { r, ..color })
                    }))
                    .push(Slider::new(green, 0.0..=1.0, color.g, move |g| {
                        StepMessage::TextColorChanged(Color { g, ..color })
                    }))
                    .push(Slider::new(blue, 0.0..=1.0, color.b, move |b| {
                        StepMessage::TextColorChanged(Color { b, ..color })
                    })),
            );

        Self::container("Text")
            .push(Text::new(
                "Text is probably the most essential widget for your UI. \
                 It will try to adapt to the dimensions of its container.",
            ))
            .push(size_section)
            .push(color_section)
    }

    fn radio(selection: Option<Language>) -> Column<'a, StepMessage> {
        let question = Column::new()
            .padding(20)
            .spacing(10)
            .push(Text::new("Iced is written in...").size(24))
            .push(Language::all().iter().cloned().fold(
                Column::new().padding(10).spacing(20),
                |choices, language| {
                    choices.push(Radio::new(
                        language,
                        language.into(),
                        selection,
                        StepMessage::LanguageSelected,
                    ))
                },
            ));

        Self::container("Radio button")
            .push(Text::new(
                "A radio button is normally used to represent a choice... \
                 Surprise test!",
            ))
            .push(question)
            .push(Text::new(
                "Iced works very well with iterators! The list above is \
                 basically created by folding a column over the different \
                 choices, creating a radio button for each one of them!",
            ))
    }

    fn image(
        width: u16,
        slider: &'a mut slider::State,
    ) -> Column<'a, StepMessage> {
        Self::container("Image")
            .push(Text::new("An image that tries to keep its aspect ratio."))
            .push(
                Image::new("resources/ferris.png")
                    .width(Length::Units(width))
                    .align_self(Align::Center),
            )
            .push(Slider::new(
                slider,
                100.0..=500.0,
                width as f32,
                StepMessage::ImageWidthChanged,
            ))
            .push(
                Text::new(&format!("Width: {} px", width.to_string()))
                    .horizontal_alignment(HorizontalAlignment::Center),
            )
    }

    fn debugger(debug: bool) -> Column<'a, StepMessage> {
        Self::container("Debugger")
            .push(Text::new(
                "You can ask Iced to visually explain the layouting of the \
                 different elements comprising your UI!",
            ))
            .push(Text::new(
                "Give it a shot! Check the following checkbox to be able to \
                 see element boundaries.",
            ))
            .push(Checkbox::new(
                debug,
                "Explain layout",
                StepMessage::DebugToggled,
            ))
            .push(Text::new("Feel free to go back and take a look."))
    }

    fn end() -> Column<'a, StepMessage> {
        Self::container("You reached the end!")
            .push(Text::new(
                "This tour will be updated as more features are added.",
            ))
            .push(Text::new("Make sure to keep an eye on it!"))
    }
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

impl From<Language> for &str {
    fn from(language: Language) -> &'static str {
        match language {
            Language::Rust => "Rust",
            Language::Elm => "Elm",
            Language::Ruby => "Ruby",
            Language::Haskell => "Haskell",
            Language::C => "C",
            Language::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Row,
    Column,
}
