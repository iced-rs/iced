use iced::widget::{
    button, column, container, horizontal_space, progress_bar, radio, row,
    scrollable, slider, text, vertical_space,
};
use iced::{executor, Alignment};
use iced::{Application, Command, Element, Length, Settings, Theme};
use lazy_static::lazy_static;

lazy_static! {
    static ref SCROLLABLE_ID: scrollable::Id = scrollable::Id::unique();
}

pub fn main() -> iced::Result {
    ScrollableDemo::run(Settings::default())
}

struct ScrollableDemo {
    scrollable_direction: Direction,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    current_scroll_offset: f32,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchDirection(Direction),
    ScrollbarWidthChanged(u16),
    ScrollbarMarginChanged(u16),
    ScrollerWidthChanged(u16),
    ScrollToBeginning,
    ScrollToEnd,
    Scrolled(f32),
}

impl Application for ScrollableDemo {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            ScrollableDemo {
                scrollable_direction: Direction::Vertical,
                scrollbar_width: 10,
                scrollbar_margin: 0,
                scroller_width: 10,
                current_scroll_offset: 0.0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Scrollable - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SwitchDirection(direction) => {
                self.scrollable_direction = direction;

                Command::none()
            }
            Message::ScrollbarWidthChanged(width) => {
                self.scrollbar_width = width;

                Command::none()
            }
            Message::ScrollbarMarginChanged(margin) => {
                self.scrollbar_margin = margin;

                Command::none()
            }
            Message::ScrollerWidthChanged(width) => {
                self.scroller_width = width;

                Command::none()
            }
            Message::ScrollToBeginning => {
                self.current_scroll_offset = 0.0;
                scrollable::snap_to(SCROLLABLE_ID.clone(), 0.0)
            }
            Message::ScrollToEnd => {
                self.current_scroll_offset = 1.0;
                scrollable::snap_to(SCROLLABLE_ID.clone(), 1.0)
            }
            Message::Scrolled(new_offset) => {
                self.current_scroll_offset = new_offset;

                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let scrollbar_width_slider = slider(
            0..=15,
            self.scrollbar_width,
            Message::ScrollbarWidthChanged,
        );
        let scrollbar_margin_slider = slider(
            0..=15,
            self.scrollbar_margin,
            Message::ScrollbarMarginChanged,
        );
        let scroller_width_slider =
            slider(0..=15, self.scroller_width, Message::ScrollerWidthChanged);

        let scroll_slider_controls = column![
            text("Scrollbar width:"),
            scrollbar_width_slider,
            text("Scrollbar margin:"),
            scrollbar_margin_slider,
            text("Scroller width:"),
            scroller_width_slider,
        ]
        .width(Length::Fill);

        let scroll_orientation_controls = column(vec![
            text("Scrollbar direction:").into(),
            radio(
                "Vertical",
                Direction::Vertical,
                Some(self.scrollable_direction),
                Message::SwitchDirection,
            )
            .into(),
            radio(
                "Horizontal",
                Direction::Horizontal,
                Some(self.scrollable_direction),
                Message::SwitchDirection,
            )
            .into(),
        ])
        .width(Length::Fill);

        let scroll_controls =
            row![scroll_slider_controls, scroll_orientation_controls]
                .spacing(20)
                .width(Length::Fill);

        let scroll_to_end_button = button("Scroll to end")
            .padding(10)
            .width(Length::Units(120))
            .on_press(Message::ScrollToEnd);

        let scroll_to_beginning_button = button("Scroll to beginning")
            .padding(10)
            .width(Length::Units(120))
            .on_press(Message::ScrollToBeginning);

        let progress_bar = progress_bar(0.0..=1.0, self.current_scroll_offset);

        let content: Element<Message> = match self.scrollable_direction {
            Direction::Vertical => column![
                scroll_controls,
                scrollable::vertical(
                    column![
                        scroll_to_end_button,
                        vertical_space(Length::Units(100)),
                        text(
                            "Some content that should wrap within the \
                        scrollable. Let's output a lot of short words, so \
                        that we'll make sure to see how wrapping works \
                        with these scrollbars."
                        ),
                        vertical_space(Length::Units(1200)),
                        text("Middle!"),
                        vertical_space(Length::Units(1200)),
                        text("The end!"),
                        scroll_to_beginning_button
                    ]
                    .align_items(Alignment::Center)
                    .spacing(20)
                )
                .height(Length::Fill)
                .scrollbar_width(self.scrollbar_width)
                .scrollbar_margin(self.scrollbar_margin)
                .scroller_width(self.scroller_width)
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                progress_bar,
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(10)
            .into(),
            Direction::Horizontal => column![
                scroll_controls,
                scrollable::horizontal(
                    row![
                        scroll_to_end_button,
                        horizontal_space(Length::Units(100)),
                        text(
                            "Some content that should wrap within the \
                        scrollable. Let's output a lot of short words, so \
                        that we'll make sure to see how wrapping works \
                        with these scrollbars."
                        ),
                        horizontal_space(Length::Units(1200)),
                        text("Middle!"),
                        horizontal_space(Length::Units(1200)),
                        text("The end!"),
                        scroll_to_beginning_button
                    ]
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                    .spacing(20)
                )
                .width(Length::Fill)
                .scrollbar_height(self.scrollbar_width)
                .scrollbar_margin(self.scrollbar_margin)
                .scroller_height(self.scroller_width)
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                progress_bar,
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(10)
            .into(),
        };

        Element::from(
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(40)
                .center_x()
                .center_y(),
        )
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
