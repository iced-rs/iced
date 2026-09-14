use iced::widget;

// derives `Default` because `iced::application()` requires either `App::new()` or `App::default()`
/// program state
#[derive(Default)]
struct App {
    /// should the popup be shown?
    show_modal: bool,
}

// derives `Clone` so `iced_core::element::Element` can impl From for widgets that send a message
/// what should `App::update()` do?...
#[derive(Clone)]
enum Message {
    /// ...it should hide the popup
    HideModal,
    /// ...it should show the popup
    ShowModal,
}

impl App {
    /// update `App` state, or do nothing (e.g. `iced::Task::none()`)
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::HideModal => {
                self.show_modal = false;
            }
            Message::ShowModal => {
                self.show_modal = true;
            }
        }

        // no task produced by incoming message
        iced::Task::none()
    }

    /// update `App`'s viewed or hidden widgets
    fn view(&self) -> iced::Element<'_, Message> {
        // `widget::button()` - press to show modal popup via `Message`
        //      `.on_press()` - the `Message` to show the modal popup
        let base_button = widget::button(widget::text!("show popup")).on_press(Message::ShowModal);

        // `widget::center()` - place in center and fill parent (window)
        let base_content = widget::center(base_button);

        if self.show_modal {
            // `widget::button()` - press to hide modal popup via `Message`
            //      `.on_press()` - the `Message` to hide the modal popup
            let popup_button = widget::button(widget::text!("hide")).on_press(Message::HideModal);

            // `widget_modal()` - show popup on-top of base using `widget::stack()`
            widget_modal(base_content, popup_button)
        } else {
            // show base but not popup if the button isn't clicked
            base_content.into()
        }
    }
}

/// custom widget to create a modal popup using `widget::stack!()` and `widget::opaque()`
fn widget_modal<'a, Message>(
    base_content: impl Into<iced::Element<'a, Message>>,
    popup_content: impl Into<iced::Element<'a, Message>>,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
{
    // create a "blurred" background via a semi transparent (alpha 0.8) black background
    let container_style = widget::container::Style {
        background: Some(
            iced::Color {
                a: 0.8,
                ..iced::Color::BLACK
            }
            .into(),
        ),
        ..Default::default()
    };

    // `widget::center()` - place in center and fill parent (window)
    //         `.style()` - set the style on `widget::center()` to the "blurred" background
    let blurred_container = widget::center(popup_content).style(move |_theme| container_style);

    // `widget::opaque()` - capture mouse button events so you cant click outside the popup
    let mouse_capture = widget::opaque(blurred_container);

    // `widget::stack!()` - show popup on-top of base
    widget::stack!(base_content.into(), mouse_capture).into()
}

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view).run()
}
