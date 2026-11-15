//! Behavioral tests for the `toast` widget.
use iced_test::simulator;
use iced_widget::core::{Element, Theme};
use iced_widget::{Renderer, toast};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Close,
}

#[test]
fn toast_shows_its_content_and_reports_close() {
    let element: Element<'_, Message, Theme, Renderer> =
        toast::Toast::new("Saved", "Your changes were saved.")
            .status(toast::Status::Success)
            .on_close(Message::Close)
            .view();

    let mut ui = simulator(element);

    assert!(ui.find("Saved").is_ok(), "toast title should be visible");
    assert!(
        ui.find("Your changes were saved.").is_ok(),
        "toast body should be visible"
    );

    let _ = ui.click("X");

    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Close],
        "close button should report its message"
    );
}
