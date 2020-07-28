use iced::{
    button, tooltip::TooltipPosition, Button, Column, Container, Element,
    Length, Row, Sandbox, Settings, Text, Tooltip,
};

pub fn main() {
    Example::run(Settings::default()).unwrap()
}

#[derive(Default)]
struct Example {
    tooltip_top_button_state: button::State,
    tooltip_bottom_button_state: button::State,
    tooltip_right_button_state: button::State,
    tooltip_left_button_state: button::State,
    tooltip_cursor_button_state: button::State,
}

#[derive(Debug, Clone, Copy)]
struct Message;

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Tooltip - Iced")
    }

    fn update(&mut self, _message: Message) {}

    fn view(&mut self) -> Element<Message> {
        let tooltip_top = tooltip_builder(
            "Tooltip at top",
            &mut self.tooltip_top_button_state,
            TooltipPosition::Top,
        );
        let tooltip_bottom = tooltip_builder(
            "Tooltip at bottom",
            &mut self.tooltip_bottom_button_state,
            TooltipPosition::Bottom,
        );
        let tooltip_right = tooltip_builder(
            "Tooltip at right",
            &mut self.tooltip_right_button_state,
            TooltipPosition::Right,
        );
        let tooltip_left = tooltip_builder(
            "Tooltip at left",
            &mut self.tooltip_left_button_state,
            TooltipPosition::Left,
        );

        let fixed_tooltips = Row::with_children(vec![
            tooltip_top.into(),
            tooltip_bottom.into(),
            tooltip_left.into(),
            tooltip_right.into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(iced::Align::Center)
        .spacing(120);

        let cursor_tooltip_area = Tooltip::new(
            Button::new(
                &mut self.tooltip_cursor_button_state,
                Container::new(Text::new("Tooltip follows cursor").size(40))
                    .center_y()
                    .center_x()
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(Message)
            .width(Length::Fill)
            .height(Length::Fill),
            tooltip(),
            TooltipPosition::FollowCursor,
        );

        let content = Column::with_children(vec![
            Container::new(fixed_tooltips)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            cursor_tooltip_area.into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn tooltip_builder<'a>(
    label: &str,
    button_state: &'a mut button::State,
    position: TooltipPosition,
) -> Container<'a, Message> {
    Container::new(Tooltip::new(
        Button::new(button_state, Text::new(label).size(40)).on_press(Message),
        tooltip(),
        position,
    ))
    .center_x()
    .center_y()
    .width(Length::Fill)
    .height(Length::Fill)
}

fn tooltip() -> Text {
    Text::new("Tooltip").size(20)
}
