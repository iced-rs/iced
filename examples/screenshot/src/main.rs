use iced::keyboard;
use iced::widget::{button, column, container, image, row, text, text_input};
use iced::window;
use iced::window::screenshot::{self, Screenshot};
use iced::{
    Center, ContentFit, Element, Fill, FillPortion, Rectangle, Subscription,
    Task,
};

use ::image as img;
use ::image::ColorType;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("Screenshot - Iced", Example::update, Example::view)
        .subscription(Example::subscription)
        .run()
}

#[derive(Default)]
struct Example {
    screenshot: Option<(Screenshot, image::Handle)>,
    saved_png_path: Option<Result<String, PngError>>,
    png_saving: bool,
    crop_error: Option<screenshot::CropError>,
    x_input_value: Option<u32>,
    y_input_value: Option<u32>,
    width_input_value: Option<u32>,
    height_input_value: Option<u32>,
}

#[derive(Clone, Debug)]
enum Message {
    Crop,
    Screenshot,
    Screenshotted(Screenshot),
    Png,
    PngSaved(Result<String, PngError>),
    XInputChanged(Option<u32>),
    YInputChanged(Option<u32>),
    WidthInputChanged(Option<u32>),
    HeightInputChanged(Option<u32>),
}

impl Example {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Screenshot => {
                return window::get_latest()
                    .and_then(window::screenshot)
                    .map(Message::Screenshotted);
            }
            Message::Screenshotted(screenshot) => {
                self.screenshot = Some((
                    screenshot.clone(),
                    image::Handle::from_rgba(
                        screenshot.size.width,
                        screenshot.size.height,
                        screenshot.bytes,
                    ),
                ));
            }
            Message::Png => {
                if let Some((screenshot, _handle)) = &self.screenshot {
                    self.png_saving = true;

                    return Task::perform(
                        save_to_png(screenshot.clone()),
                        Message::PngSaved,
                    );
                }
            }
            Message::PngSaved(res) => {
                self.png_saving = false;
                self.saved_png_path = Some(res);
            }
            Message::XInputChanged(new_value) => {
                self.x_input_value = new_value;
            }
            Message::YInputChanged(new_value) => {
                self.y_input_value = new_value;
            }
            Message::WidthInputChanged(new_value) => {
                self.width_input_value = new_value;
            }
            Message::HeightInputChanged(new_value) => {
                self.height_input_value = new_value;
            }
            Message::Crop => {
                if let Some((screenshot, _handle)) = &self.screenshot {
                    let cropped = screenshot.crop(Rectangle::<u32> {
                        x: self.x_input_value.unwrap_or(0),
                        y: self.y_input_value.unwrap_or(0),
                        width: self.width_input_value.unwrap_or(0),
                        height: self.height_input_value.unwrap_or(0),
                    });

                    match cropped {
                        Ok(screenshot) => {
                            self.screenshot = Some((
                                screenshot.clone(),
                                image::Handle::from_rgba(
                                    screenshot.size.width,
                                    screenshot.size.height,
                                    screenshot.bytes,
                                ),
                            ));
                            self.crop_error = None;
                        }
                        Err(crop_error) => {
                            self.crop_error = Some(crop_error);
                        }
                    }
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let image: Element<Message> =
            if let Some((_screenshot, handle)) = &self.screenshot {
                image(handle)
                    .content_fit(ContentFit::Contain)
                    .width(Fill)
                    .height(Fill)
                    .into()
            } else {
                text("Press the button to take a screenshot!").into()
            };

        let image = container(image)
            .center_y(FillPortion(2))
            .padding(10)
            .style(container::rounded_box);

        let crop_origin_controls = row![
            text("X:").width(30),
            numeric_input("0", self.x_input_value).map(Message::XInputChanged),
            text("Y:").width(30),
            numeric_input("0", self.y_input_value).map(Message::YInputChanged)
        ]
        .spacing(10)
        .align_y(Center);

        let crop_dimension_controls = row![
            text("W:").width(30),
            numeric_input("0", self.width_input_value)
                .map(Message::WidthInputChanged),
            text("H:").width(30),
            numeric_input("0", self.height_input_value)
                .map(Message::HeightInputChanged)
        ]
        .spacing(10)
        .align_y(Center);

        let crop_controls =
            column![crop_origin_controls, crop_dimension_controls]
                .push_maybe(
                    self.crop_error
                        .as_ref()
                        .map(|error| text!("Crop error! \n{error}")),
                )
                .spacing(10)
                .align_x(Center);

        let controls = {
            let save_result =
                self.saved_png_path.as_ref().map(
                    |png_result| match png_result {
                        Ok(path) => format!("Png saved as: {path:?}!"),
                        Err(PngError(error)) => {
                            format!("Png could not be saved due to:\n{}", error)
                        }
                    },
                );

            column![
                column![
                    button(centered_text("Screenshot!"))
                        .padding([10, 20])
                        .width(Fill)
                        .on_press(Message::Screenshot),
                    if !self.png_saving {
                        button(centered_text("Save as png")).on_press_maybe(
                            self.screenshot.is_some().then(|| Message::Png),
                        )
                    } else {
                        button(centered_text("Saving..."))
                            .style(button::secondary)
                    }
                    .style(button::secondary)
                    .padding([10, 20])
                    .width(Fill)
                ]
                .spacing(10),
                column![
                    crop_controls,
                    button(centered_text("Crop"))
                        .on_press(Message::Crop)
                        .style(button::danger)
                        .padding([10, 20])
                        .width(Fill),
                ]
                .spacing(10)
                .align_x(Center),
            ]
            .push_maybe(save_result.map(text))
            .spacing(40)
        };

        let side_content = container(controls).center_y(Fill);

        let content = row![side_content, image]
            .spacing(10)
            .width(Fill)
            .height(Fill)
            .align_y(Center);

        container(content).padding(10).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;

        keyboard::on_key_press(|key, _modifiers| {
            if let keyboard::Key::Named(key::Named::F5) = key {
                Some(Message::Screenshot)
            } else {
                None
            }
        })
    }
}

async fn save_to_png(screenshot: Screenshot) -> Result<String, PngError> {
    let path = "screenshot.png".to_string();

    tokio::task::spawn_blocking(move || {
        img::save_buffer(
            &path,
            &screenshot.bytes,
            screenshot.size.width,
            screenshot.size.height,
            ColorType::Rgba8,
        )
        .map(|_| path)
        .map_err(|error| PngError(error.to_string()))
    })
    .await
    .expect("Blocking task to finish")
}

#[derive(Clone, Debug)]
struct PngError(String);

fn numeric_input(
    placeholder: &str,
    value: Option<u32>,
) -> Element<'_, Option<u32>> {
    text_input(
        placeholder,
        &value.as_ref().map(ToString::to_string).unwrap_or_default(),
    )
    .on_input(move |text| {
        if text.is_empty() {
            None
        } else if let Ok(new_value) = text.parse() {
            Some(new_value)
        } else {
            value
        }
    })
    .width(40)
    .into()
}

fn centered_text(content: &str) -> Element<'_, Message> {
    text(content).width(Fill).align_x(Center).into()
}
