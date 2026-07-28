//! An example of masked text inputs for a credit card form.
//!
//! The displayed value contains formatting characters (spaces, `/`),
//! while the stored value contains only the raw digits.
//!
//! Cursor positions are automatically adjusted so that typing at the end
//! of a field keeps the cursor correctly aligned after formatting.
//!
//! Run with:
//! ```sh
//! cargo run --package masked_input
//! ```

use iced::widget::{center, column, row, text, text_input};
use iced::{Element};

pub fn main() -> iced::Result {
    iced::application(
        CreditCardForm::new,
        CreditCardForm::update,
        CreditCardForm::view,
    )
    .title("Credit Card Masked Input")
    .run()
}

struct CreditCardForm {
    card_number: String,
    expiry: String,
    cvv: String,
}

#[derive(Debug, Clone)]
enum Message {
    CardNumberChanged(String),
    ExpiryChanged(String),
    CvvChanged(String),
}

impl CreditCardForm {
    fn new() -> Self {
        Self {
            card_number: String::new(),
            expiry: String::new(),
            cvv: String::new(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CardNumberChanged(value) => {
                let raw: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

                if raw.len() <= 16 {
                    self.card_number = raw;
                }
            }
            Message::ExpiryChanged(value) => {
                let raw: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

                if raw.len() <= 4 {
                    self.expiry = raw;
                }
            }
            Message::CvvChanged(value) => {
                let raw: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

                if raw.len() <= 4 {
                    self.cvv = raw;
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let card_display = format_card(&self.card_number);
        let expiry_display = format_expiry(&self.expiry);

        // Compute the cursor position in the formatted display text.
        // The raw cursor is at the end (cursor = raw.len()) because the
        // most common editing operations (typing, backspace at end) leave
        // the cursor at the end of the content.
        let card_cursor = raw_cursor_to_display(&self.card_number, self.card_number.len(), 4, None);
        let expiry_cursor = raw_cursor_to_display(&self.expiry, self.expiry.len(), 4, Some(2));

        let content = column![
            text("Credit Card").size(24),
            // Card number field — formatted with spaces every 4 digits
            column![
                text("Card Number").size(14),
                text_input("1234 5678 9012 3456", &card_display)
                    .on_input(Message::CardNumberChanged)
                    .cursor_at(card_cursor)
                    .width(300),
                text(format!("Stored: {}", self.card_number)).size(12),
            ]
            .spacing(4),
            // Expiry and CVV side by side
            row![
                column![
                    text("Expiry").size(14),
                    text_input("MM/YY", &expiry_display)
                        .on_input(Message::ExpiryChanged)
                        .cursor_at(expiry_cursor)
                        .width(100),
                    text(format!("Stored: {}", self.expiry)).size(12),
                ]
                .spacing(4),
                column![
                    text("CVV").size(14),
                    text_input("123", &self.cvv)
                        .on_input(Message::CvvChanged)
                        .secure(true)
                        .width(100),
                ]
                .spacing(4),
            ]
            .spacing(20),
        ]
        .spacing(20)
        .padding(20)
        .max_width(400);

        center(content).into()
    }
}

/// Formats card digits with a space every 4 digits.
///
/// "4111111111111111" becomes "4111 1111 1111 1111"
fn format_card(digits: &str) -> String {
    let mut formatted = String::with_capacity(19);

    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && i % 4 == 0 {
            formatted.push(' ');
        }

        formatted.push(ch);
    }

    formatted
}

/// Formats expiry digits as MM/YY.
///
/// "1228" becomes "12/28"
fn format_expiry(digits: &str) -> String {
    let mut formatted = String::with_capacity(5);

    for (i, ch) in digits.chars().enumerate() {
        if i == 2 {
            formatted.push('/');
        }

        formatted.push(ch);
    }

    formatted
}

/// Converts a cursor position in raw digit text to the corresponding position
/// in the formatted display text.
///
/// `group_size` is the number of digits between each periodic separator
/// (e.g., 4 for credit card groups, producing a space every 4 digits).
/// `separator_at` is an optional one-off separator position
/// (e.g., `Some(2)` for MM/YY expiry formatting, producing a `/` after 2 digits).
///
/// The function counts how many formatting characters appear *before* the
/// cursor position in the raw string and adds that many to the cursor position.
///
/// For card numbers (space at positions 4, 8, 12...):
/// This is `raw_cursor + floor((raw_cursor - 1) / 4)` for `raw_cursor > 0`.
///
/// For expiry (slash at position 2):
/// This adds 1 once the cursor passes position 2 and there are >= 3 chars.
fn raw_cursor_to_display(
    raw: &str,
    raw_cursor: usize,
    group_size: usize,
    separator_at: Option<usize>,
) -> usize {
    let mut display_cursor = raw_cursor;

    // Add periodic separators (space every `group_size` digits).
    // A separator is inserted before a digit at index `group_size`, `2*group_size`, etc.
    // The separator is only before the cursor if its position < raw_cursor.
    if raw_cursor > 0 {
        display_cursor += (raw_cursor - 1) / group_size;
    }

    // Add one-off separator (slash for expiry at position 2).
    // It only appears in the display when there is content at the separator
    // position or beyond (i.e., raw.len() > separator_at).
    // And it shifts cursor positions that come after it (raw_cursor > separator_at).
    if let Some(sep) = separator_at {
        if raw_cursor > sep && raw.len() > sep {
            display_cursor += 1;
        }
    }

    display_cursor
}
