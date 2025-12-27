use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, stack, text};
use iced::{Border, Element, Length, Settings};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .settings(Settings {
            antialiasing: true,
            ..Default::default()
        })
        .window_size((200, 100))
        .centered()
        .resizable(false)
        .run()
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
}

#[derive(Default)]
struct App {
    value: i64,
}

impl App {
    fn new() -> (Self, iced::Task<Message>) {
        (App { value: 0 }, iced::Task::none())
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Note: return type changed to Element
        // Define consistent button dimensions
        const BUTTON_WIDTH: f32 = 120.0;
        const BUTTON_HEIGHT: f32 = 40.0;

        let button_content: Element<'_, Message> = if self.value > 0 {
            stack![
                // Base button content (fills the entire space)
                container(text("Increment").size(16))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),

                // Badge overlay (positioned top-right)
                container(
                    container(text(self.value).size(12))
                        .padding(4)
                        .style(|theme: &iced::Theme| container::Style {
                            background: Some(theme.palette().danger.into()),
                            text_color: Some(theme.palette().background),
                            border: Border::default().rounded(10),
                            ..Default::default()
                        })
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([2, 2])
                .align_x(Horizontal::Right)
                .align_y(Vertical::Top)
            ]
            .width(Length::Fixed(BUTTON_WIDTH))
            .height(Length::Fixed(BUTTON_HEIGHT))
            .into()
        } else {
            // No badge when count is 0 - same dimensions as badged version
            container(
                container(text("Increment").size(16))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
            )
            .width(Length::Fixed(BUTTON_WIDTH))
            .height(Length::Fixed(BUTTON_HEIGHT))
            .into()
        };

        let badged_button = button(button_content).on_press(Message::Increment);

        container(column![badged_button].padding(20))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .into()
    }
}