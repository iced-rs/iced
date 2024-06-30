use iced::widget::{Column, row, button, keyed_column, text, column, container};


pub fn main() -> iced::Result {
    iced::run("A keyed_column example", Example::update, Example::view)
}

#[derive(Default)]
struct Example;

#[derive(Debug, Clone, Copy)]
enum Message {
    Nothing,
}

impl Example {
    fn update(&mut self, message: Message) {
        //
    }

    fn view(&self) -> Column<Message> {
        let items = vec![
            (
                1, // key
                container( // widget
                    row![
                        text("Item 1"),
                        text("Description of item 1"),
                        button("My button").on_press(Message::Nothing)
                    ].spacing(30).padding([10, 0, 0, 0])
                ).into()
            ),
            (
                2, // key
                container( // widget
                    row![
                        text("Item 2"),
                        text("Description of item 2"),
                        button("My button").on_press(Message::Nothing)
                    ].spacing(30).padding([10, 0, 0, 0])
                ).into()
            ),
            (
                3, // key
                container( // widget
                    row![
                        text("Item 3"),
                        text("Description of item 3"),
                        button("My button").on_press(Message::Nothing)
                    ].spacing(30).padding([10, 0, 0, 0])
                ).into()
            )
        ];

        column![
            text("My items with keys").size(30),
            keyed_column(items)
        ].padding(20)
    }
}
