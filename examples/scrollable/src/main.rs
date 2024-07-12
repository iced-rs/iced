use iced::widget::{
    button, column, container, horizontal_space, progress_bar, radio, row,
    scrollable, slider, text, vertical_space,
};
use iced::{Border, Center, Color, Element, Fill, Task, Theme};

use once_cell::sync::Lazy;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub fn main() -> iced::Result {
    iced::application(
        "Scrollable - Iced",
        ScrollableDemo::update,
        ScrollableDemo::view,
    )
    .theme(ScrollableDemo::theme)
    .run()
}

struct ScrollableDemo {
    scrollable_direction: Direction,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    current_scroll_offset: scrollable::RelativeOffset,
    anchor: scrollable::Anchor,
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
    AlignmentChanged(scrollable::Anchor),
    ScrollbarWidthChanged(u16),
    ScrollbarMarginChanged(u16),
    ScrollerWidthChanged(u16),
    ScrollToBeginning,
    ScrollToEnd,
    Scrolled(scrollable::Viewport),
}

impl ScrollableDemo {
    fn new() -> Self {
        ScrollableDemo {
            scrollable_direction: Direction::Vertical,
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            current_scroll_offset: scrollable::RelativeOffset::START,
            anchor: scrollable::Anchor::Start,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
                self.anchor = alignment;

                scrollable::snap_to(
                    SCROLLABLE_ID.clone(),
                    self.current_scroll_offset,
                )
            }
            Message::ScrollbarWidthChanged(width) => {
                self.scrollbar_width = width;

                Task::none()
            }
            Message::ScrollbarMarginChanged(margin) => {
                self.scrollbar_margin = margin;

                Task::none()
            }
            Message::ScrollerWidthChanged(width) => {
                self.scroller_width = width;

                Task::none()
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

                Task::none()
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
                scrollable::Anchor::Start,
                Some(self.anchor),
                Message::AlignmentChanged,
            ),
            radio(
                "End",
                scrollable::Anchor::End,
                Some(self.anchor),
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
                Direction::Vertical => scrollable(
                    column![
                        scroll_to_end_button(),
                        text("Beginning!"),
                        vertical_space().height(1200),
                        text("Middle!"),
                        vertical_space().height(1200),
                        text("End!"),
                        scroll_to_beginning_button(),
                    ]
                    .align_x(Center)
                    .padding([40, 0])
                    .spacing(40),
                )
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new()
                        .width(self.scrollbar_width)
                        .margin(self.scrollbar_margin)
                        .scroller_width(self.scroller_width)
                        .anchor(self.anchor),
                ))
                .width(Fill)
                .height(Fill)
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                Direction::Horizontal => scrollable(
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
                    .align_y(Center)
                    .padding([0, 40])
                    .spacing(40),
                )
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::new()
                        .width(self.scrollbar_width)
                        .margin(self.scrollbar_margin)
                        .scroller_width(self.scroller_width)
                        .anchor(self.anchor),
                ))
                .width(Fill)
                .height(Fill)
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                Direction::Multi => scrollable(
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
                    .align_y(Center)
                    .padding([0, 40])
                    .spacing(40),
                )
                .direction({
                    let scrollbar = scrollable::Scrollbar::new()
                        .width(self.scrollbar_width)
                        .margin(self.scrollbar_margin)
                        .scroller_width(self.scroller_width)
                        .anchor(self.anchor);

                    scrollable::Direction::Both {
                        horizontal: scrollbar,
                        vertical: scrollbar,
                    }
                })
                .width(Fill)
                .height(Fill)
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
                .align_x(Center)
                .spacing(10)
                .into();

        container(content).padding(20).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl Default for ScrollableDemo {
    fn default() -> Self {
        Self::new()
    }
}

fn progress_bar_custom_style(theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: theme.extended_palette().background.strong.color.into(),
        bar: Color::from_rgb8(250, 85, 134).into(),
        border: Border::default(),
    }
}
