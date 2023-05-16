use iced::id::Id;
use iced::widget::scrollable::{Properties, Scrollbar, Scroller};
use iced::widget::{
    button, column, container, horizontal_space, progress_bar, radio, row,
    scrollable, slider, text, vertical_space,
};
use iced::{executor, theme, Alignment, Color};
use iced::{Application, Command, Element, Length, Settings, Theme};
use once_cell::sync::Lazy;

static SCROLLABLE_ID: Lazy<Id> = Lazy::new(|| Id::new("scrollable"));

pub fn main() -> iced::Result {
    ScrollableDemo::run(Settings::default())
}

struct ScrollableDemo {
    scrollable_direction: Direction,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    current_scroll_offset: scrollable::RelativeOffset,
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
        .spacing(10)
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
            radio(
                "Both!",
                Direction::Multi,
                Some(self.scrollable_direction),
                Message::SwitchDirection,
            )
            .into(),
        ])
        .spacing(10)
        .width(Length::Fill);

        let scroll_controls =
            row![scroll_slider_controls, scroll_orientation_controls]
                .spacing(20)
                .width(Length::Fill);

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
                        vertical_space(1200),
                        text("Middle!"),
                        vertical_space(1200),
                        text("End!"),
                        scroll_to_beginning_button(),
                    ]
                    .width(Length::Fill)
                    .align_items(Alignment::Center)
                    .padding([40, 0, 40, 0])
                    .spacing(40),
                )
                .height(Length::Fill)
                .vertical_scroll(
                    Properties::new()
                        .width(self.scrollbar_width)
                        .margin(self.scrollbar_margin)
                        .scroller_width(self.scroller_width),
                )
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                Direction::Horizontal => scrollable(
                    row![
                        scroll_to_end_button(),
                        text("Beginning!"),
                        horizontal_space(1200),
                        text("Middle!"),
                        horizontal_space(1200),
                        text("End!"),
                        scroll_to_beginning_button(),
                    ]
                    .height(450)
                    .align_items(Alignment::Center)
                    .padding([0, 40, 0, 40])
                    .spacing(40),
                )
                .height(Length::Fill)
                .horizontal_scroll(
                    Properties::new()
                        .width(self.scrollbar_width)
                        .margin(self.scrollbar_margin)
                        .scroller_width(self.scroller_width),
                )
                .style(theme::Scrollable::custom(ScrollbarCustomStyle))
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
                Direction::Multi => scrollable(
                    //horizontal content
                    row![
                        column![
                            text("Let's do some scrolling!"),
                            vertical_space(2400)
                        ],
                        scroll_to_end_button(),
                        text("Horizontal - Beginning!"),
                        horizontal_space(1200),
                        //vertical content
                        column![
                            text("Horizontal - Middle!"),
                            scroll_to_end_button(),
                            text("Vertical - Beginning!"),
                            vertical_space(1200),
                            text("Vertical - Middle!"),
                            vertical_space(1200),
                            text("Vertical - End!"),
                            scroll_to_beginning_button(),
                            vertical_space(40),
                        ]
                        .spacing(40),
                        horizontal_space(1200),
                        text("Horizontal - End!"),
                        scroll_to_beginning_button(),
                    ]
                    .align_items(Alignment::Center)
                    .padding([0, 40, 0, 40])
                    .spacing(40),
                )
                .height(Length::Fill)
                .vertical_scroll(
                    Properties::new()
                        .width(self.scrollbar_width)
                        .margin(self.scrollbar_margin)
                        .scroller_width(self.scroller_width),
                )
                .horizontal_scroll(
                    Properties::new()
                        .width(self.scrollbar_width)
                        .margin(self.scrollbar_margin)
                        .scroller_width(self.scroller_width),
                )
                .style(theme::Scrollable::Custom(Box::new(
                    ScrollbarCustomStyle,
                )))
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
            });

        let progress_bars: Element<Message> = match self.scrollable_direction {
            Direction::Vertical => {
                progress_bar(0.0..=1.0, self.current_scroll_offset.y).into()
            }
            Direction::Horizontal => {
                progress_bar(0.0..=1.0, self.current_scroll_offset.x)
                    .style(theme::ProgressBar::Custom(Box::new(
                        ProgressBarCustomStyle,
                    )))
                    .into()
            }
            Direction::Multi => column![
                progress_bar(0.0..=1.0, self.current_scroll_offset.y),
                progress_bar(0.0..=1.0, self.current_scroll_offset.x).style(
                    theme::ProgressBar::Custom(Box::new(
                        ProgressBarCustomStyle,
                    ))
                )
            ]
            .spacing(10)
            .into(),
        };

        let content: Element<Message> =
            column![scroll_controls, scrollable_content, progress_bars]
                .width(Length::Fill)
                .height(Length::Fill)
                .align_items(Alignment::Center)
                .spacing(10)
                .into();

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

struct ScrollbarCustomStyle;

impl scrollable::StyleSheet for ScrollbarCustomStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> Scrollbar {
        style.active(&theme::Scrollable::Default)
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> Scrollbar {
        style.hovered(&theme::Scrollable::Default, is_mouse_over_scrollbar)
    }

    fn hovered_horizontal(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> Scrollbar {
        if is_mouse_over_scrollbar {
            Scrollbar {
                background: style
                    .active(&theme::Scrollable::default())
                    .background,
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Default::default(),
                scroller: Scroller {
                    color: Color::from_rgb8(250, 85, 134),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Default::default(),
                },
            }
        } else {
            self.active(style)
        }
    }
}

struct ProgressBarCustomStyle;

impl progress_bar::StyleSheet for ProgressBarCustomStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: style.extended_palette().background.strong.color.into(),
            bar: Color::from_rgb8(250, 85, 134).into(),
            border_radius: 0.0,
        }
    }
}
