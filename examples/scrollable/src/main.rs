use iced::executor;
use iced::widget::scrollable::Properties;
use iced::widget::{
    button, column, container, horizontal_space, progress_bar, radio, row,
    scrollable, slider, text, vertical_space, Scrollable,
};
use iced::{
    Alignment, Application, Color, Command, Element, Length, Settings, Theme,
};

use once_cell::sync::Lazy;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub fn main() -> iced::Result {
    ScrollableDemo::run(Settings::default())
}

struct ScrollableDemo {
    scrollable_direction: Direction,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    current_scroll_offset: scrollable::RelativeOffset,
    alignment: scrollable::Alignment,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum Direction {
    Vertical,
    Horizontal,
    Multi,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchDirection(Direction),
    AlignmentChanged(scrollable::Alignment),
    ScrollbarWidthChanged(u16),
    ScrollbarMarginChanged(u16),
    ScrollerWidthChanged(u16),
    ScrollToBeginning,
    ScrollToEnd,
    Scrolled(scrollable::Viewport),
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
                current_scroll_offset: scrollable::RelativeOffset::START,
                alignment: scrollable::Alignment::Start,
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
                self.current_scroll_offset = scrollable::RelativeOffset::START;
                self.scrollable_direction = direction;

                scrollable::snap_to(
                    SCROLLABLE_ID.clone(),
                    self.current_scroll_offset,
                )
            }
            Message::AlignmentChanged(alignment) => {
                self.current_scroll_offset = scrollable::RelativeOffset::START;
                self.alignment = alignment;

                scrollable::snap_to(
                    SCROLLABLE_ID.clone(),
                    self.current_scroll_offset,
                )
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
                self.current_scroll_offset = scrollable::RelativeOffset::START;

                scrollable::snap_to(
                    SCROLLABLE_ID.clone(),
                    self.current_scroll_offset,
                )
            }
            Message::ScrollToEnd => {
                self.current_scroll_offset = scrollable::RelativeOffset::END;

                scrollable::snap_to(
                    SCROLLABLE_ID.clone(),
                    self.current_scroll_offset,
                )
            }
            Message::Scrolled(viewport) => {
                self.current_scroll_offset = viewport.relative_offset();

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
        .spacing(10);

        let scroll_orientation_controls = column![
            text("Scrollbar direction:"),
            radio(
                "Vertical",
                Direction::Vertical,
                Some(self.scrollable_direction),
                Message::SwitchDirection,
            ),
            radio(
                "Horizontal",
                Direction::Horizontal,
                Some(self.scrollable_direction),
                Message::SwitchDirection,
            ),
            radio(
                "Both!",
                Direction::Multi,
                Some(self.scrollable_direction),
                Message::SwitchDirection,
            ),
        ]
        .spacing(10);

        let scroll_alignment_controls = column![
            text("Scrollable alignment:"),
            radio(
                "Start",
                scrollable::Alignment::Start,
                Some(self.alignment),
                Message::AlignmentChanged,
            ),
            radio(
                "End",
                scrollable::Alignment::End,
                Some(self.alignment),
                Message::AlignmentChanged,
            )
        ]
        .spacing(10);

        let scroll_controls = row![
            scroll_slider_controls,
            scroll_orientation_controls,
            scroll_alignment_controls
        ]
        .spacing(20);

        let scroll_to_end_button = || {
            button("Scroll to end")
                .padding(10)
                .on_press(Message::ScrollToEnd)
        };

        let scroll_to_beginning_button = || {
            button("Scroll to beginning")
                .padding(10)
                .on_press(Message::ScrollToBeginning)
        };

        let scrollable_content: Element<Message> =
            Element::from(match self.scrollable_direction {
                Direction::Vertical => Scrollable::with_direction(
                    column![
                        scroll_to_end_button(),
                        text("Beginning!"),
                        vertical_space().height(1200),
                        text("Middle!"),
                        vertical_space().height(1200),
                        text("End!"),
                        scroll_to_beginning_button(),
                    ]
                    .align_items(Alignment::Center)
                    .padding([40, 0, 40, 0])
                    .spacing(40),
                    scrollable::Direction::Vertical(
                        Properties::new()
                            .width(self.scrollbar_width)
                            .margin(self.scrollbar_margin)
                            .scroller_width(self.scroller_width)
                            .alignment(self.alignment),
                    ),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                Direction::Horizontal => Scrollable::with_direction(
                    row![
                        scroll_to_end_button(),
                        text("Beginning!"),
                        horizontal_space().width(1200),
                        text("Middle!"),
                        horizontal_space().width(1200),
                        text("End!"),
                        scroll_to_beginning_button(),
                    ]
                    .height(450)
                    .align_items(Alignment::Center)
                    .padding([0, 40, 0, 40])
                    .spacing(40),
                    scrollable::Direction::Horizontal(
                        Properties::new()
                            .width(self.scrollbar_width)
                            .margin(self.scrollbar_margin)
                            .scroller_width(self.scroller_width)
                            .alignment(self.alignment),
                    ),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                Direction::Multi => Scrollable::with_direction(
                    //horizontal content
                    row![
                        column![
                            text("Let's do some scrolling!"),
                            vertical_space().height(2400)
                        ],
                        scroll_to_end_button(),
                        text("Horizontal - Beginning!"),
                        horizontal_space().width(1200),
                        //vertical content
                        column![
                            text("Horizontal - Middle!"),
                            scroll_to_end_button(),
                            text("Vertical - Beginning!"),
                            vertical_space().height(1200),
                            text("Vertical - Middle!"),
                            vertical_space().height(1200),
                            text("Vertical - End!"),
                            scroll_to_beginning_button(),
                            vertical_space().height(40),
                        ]
                        .spacing(40),
                        horizontal_space().width(1200),
                        text("Horizontal - End!"),
                        scroll_to_beginning_button(),
                    ]
                    .align_items(Alignment::Center)
                    .padding([0, 40, 0, 40])
                    .spacing(40),
                    {
                        let properties = Properties::new()
                            .width(self.scrollbar_width)
                            .margin(self.scrollbar_margin)
                            .scroller_width(self.scroller_width)
                            .alignment(self.alignment);

                        scrollable::Direction::Both {
                            horizontal: properties,
                            vertical: properties,
                        }
                    },
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
            });

        let progress_bars: Element<Message> = match self.scrollable_direction {
            Direction::Vertical => {
                progress_bar(0.0..=1.0, self.current_scroll_offset.y).into()
            }
            Direction::Horizontal => {
                progress_bar(0.0..=1.0, self.current_scroll_offset.x)
                    .style(progress_bar_custom_style)
                    .into()
            }
            Direction::Multi => column![
                progress_bar(0.0..=1.0, self.current_scroll_offset.y),
                progress_bar(0.0..=1.0, self.current_scroll_offset.x)
                    .style(progress_bar_custom_style)
            ]
            .spacing(10)
            .into(),
        };

        let content: Element<Message> =
            column![scroll_controls, scrollable_content, progress_bars]
                .align_items(Alignment::Center)
                .spacing(10)
                .into();

        container(content).padding(20).center_x().center_y().into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

fn progress_bar_custom_style(theme: &Theme) -> progress_bar::Appearance {
    progress_bar::Appearance {
        background: theme.extended_palette().background.strong.color.into(),
        bar: Color::from_rgb8(250, 85, 134).into(),
        buffer: Color::from_rgba8(85, 250, 134, 0.5).into(),
        border_radius: 0.0.into(),
    }
}
