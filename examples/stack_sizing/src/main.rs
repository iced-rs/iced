//! Example demonstrating Stack SizingMode options
//!
//! This example shows the difference between:
//! - SizingMode::BaseLayer (default): Stack sizes to the first element
//! - SizingMode::LargestChild: Stack sizes to accommodate all children

use iced::widget::{button, column, container, row, text, Opacity, Stack, SizingMode};
use iced::{Center, Color, Element, Fill, Length};

pub fn main() -> iced::Result {
    iced::run(App::update, App::view)
}

#[derive(Default)]
struct App {
    current_option: usize,
    use_largest_child: bool,
}

#[derive(Debug, Clone)]
enum Message {
    NextOption,
    PrevOption,
    ToggleSizingMode,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::NextOption => {
                self.current_option = (self.current_option + 1) % 3;
            }
            Message::PrevOption => {
                self.current_option = if self.current_option == 0 {
                    2
                } else {
                    self.current_option - 1
                };
            }
            Message::ToggleSizingMode => {
                self.use_largest_child = !self.use_largest_child;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Three options with different text lengths
        let options = [
            ("Short", "Brief description"),
            ("Medium Option", "This is a medium length description that takes up more space"),
            ("Very Long Option Title", "This is a much longer description that demonstrates how the stack sizing mode affects the layout. When using LargestChild mode, the stack will size to fit this content even when showing shorter options."),
        ];

        // Build the stack with opacity-based visibility
        let sizing_mode = if self.use_largest_child {
            SizingMode::LargestChild
        } else {
            SizingMode::BaseLayer
        };

        let mut stk = Stack::new()
            .width(Length::Fixed(400.0))
            .sizing_mode(sizing_mode);

        for (i, (title, desc)) in options.iter().enumerate() {
            let opacity = if i == self.current_option { 1.0 } else { 0.0 };

            let content: Element<'_, Message> = container(
                column![
                    text(*title).size(24),
                    text(*desc).size(14),
                    row![
                        button("Action 1").on_press(Message::NextOption),
                        button("Action 2").on_press(Message::PrevOption),
                    ]
                    .spacing(10)
                ]
                .spacing(10),
            )
            .padding(20)
            .style(|_theme| container::Style {
                background: Some(Color::from_rgb(0.2, 0.2, 0.3).into()),
                border: iced::Border::default().rounded(8),
                ..Default::default()
            })
            .into();

            // Wrap with opacity widget
            let with_opacity: Element<'_, Message> = Opacity::new(opacity, content).into();
            stk = stk.push(with_opacity);
        }

        // Controls
        let controls = row![
            button("← Prev").on_press(Message::PrevOption),
            button("Next →").on_press(Message::NextOption),
            button(if self.use_largest_child {
                "Mode: LargestChild"
            } else {
                "Mode: BaseLayer"
            })
            .on_press(Message::ToggleSizingMode),
        ]
        .spacing(10);

        let mode_label = text(format!(
            "Current: Option {} | SizingMode::{:?}",
            self.current_option + 1,
            if self.use_largest_child {
                "LargestChild"
            } else {
                "BaseLayer"
            }
        ))
        .size(16);

        let explanation = text(if self.use_largest_child {
            "LargestChild: Stack height stays constant (sized to largest option)"
        } else {
            "BaseLayer: Stack height changes based on first element (option 1)"
        })
        .size(12);

        // Main layout
        container(
            column![mode_label, explanation, Element::from(stk), controls,]
                .spacing(20)
                .align_x(Center),
        )
        .width(Fill)
        .height(Fill)
        .center(Fill)
        .into()
    }
}
