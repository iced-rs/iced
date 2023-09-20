use iced::alignment;
use iced::keyboard::KeyCode;
use iced::theme::{Button, Container};
use iced::widget::{button, column, container, image, row, text, text_input};
use iced::window::screenshot::{self, Screenshot};
use iced::{
    event, executor, keyboard, Alignment, Application, Command, ContentFit,
    Element, Event, Length, Rectangle, Renderer, Subscription, Theme,
};

use ::image as img;
use ::image::ColorType;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    Example::run(iced::Settings::default())
}

struct Example {
    screenshot: Option<Screenshot>,
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
    ScreenshotData(Screenshot),
    Png,
    PngSaved(Result<String, PngError>),
    XInputChanged(Option<u32>),
    YInputChanged(Option<u32>),
    WidthInputChanged(Option<u32>),
    HeightInputChanged(Option<u32>),
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Example {
                screenshot: None,
                saved_png_path: None,
                png_saving: false,
                crop_error: None,
                x_input_value: None,
                y_input_value: None,
                width_input_value: None,
                height_input_value: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Screenshot".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Screenshot => {
                return iced::window::screenshot(Message::ScreenshotData);
            }
            Message::ScreenshotData(screenshot) => {
                self.screenshot = Some(screenshot);
            }
            Message::Png => {
                if let Some(screenshot) = &self.screenshot {
                    self.png_saving = true;

                    return Command::perform(
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
                if let Some(screenshot) = &self.screenshot {
                    let cropped = screenshot.crop(Rectangle::<u32> {
                        x: self.x_input_value.unwrap_or(0),
                        y: self.y_input_value.unwrap_or(0),
                        width: self.width_input_value.unwrap_or(0),
                        height: self.height_input_value.unwrap_or(0),
                    });

                    match cropped {
                        Ok(screenshot) => {
                            self.screenshot = Some(screenshot);
                            self.crop_error = None;
                        }
                        Err(crop_error) => {
                            self.crop_error = Some(crop_error);
                        }
                    }
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let image: Element<Message> = if let Some(screenshot) = &self.screenshot
        {
            image(image::Handle::from_pixels(
                screenshot.size.width,
                screenshot.size.height,
                screenshot.clone(),
            ))
            .content_fit(ContentFit::Contain)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            text("Press the button to take a screenshot!").into()
        };

        let image = container(image)
            .padding(10)
            .style(Container::Box)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .center_x()
            .center_y();

        let crop_origin_controls = row![
            text("X:")
                .vertical_alignment(alignment::Vertical::Center)
                .width(30),
            numeric_input("0", self.x_input_value).map(Message::XInputChanged),
            text("Y:")
                .vertical_alignment(alignment::Vertical::Center)
                .width(30),
            numeric_input("0", self.y_input_value).map(Message::YInputChanged)
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let crop_dimension_controls = row![
            text("W:")
                .vertical_alignment(alignment::Vertical::Center)
                .width(30),
            numeric_input("0", self.width_input_value)
                .map(Message::WidthInputChanged),
            text("H:")
                .vertical_alignment(alignment::Vertical::Center)
                .width(30),
            numeric_input("0", self.height_input_value)
                .map(Message::HeightInputChanged)
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let mut crop_controls =
            column![crop_origin_controls, crop_dimension_controls]
                .spacing(10)
                .align_items(Alignment::Center);

        if let Some(crop_error) = &self.crop_error {
            crop_controls =
                crop_controls.push(text(format!("Crop error! \n{crop_error}")));
        }

        let mut controls = column![
            column![
                button(centered_text("Screenshot!"))
                    .padding([10, 20, 10, 20])
                    .width(Length::Fill)
                    .on_press(Message::Screenshot),
                if self.png_saving {
                    button(centered_text("Saving...")).style(Button::Secondary)
                } else {
                    button(centered_text("Save as png")).on_press_maybe(
                        self.screenshot.is_some().then(|| Message::Png),
                    )
                }
                .style(Button::Secondary)
                .padding([10, 20, 10, 20])
                .width(Length::Fill)
            ]
            .spacing(10),
            column![
                crop_controls,
                button(centered_text("Crop"))
                    .on_press(Message::Crop)
                    .style(Button::Destructive)
                    .padding([10, 20, 10, 20])
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        ]
        .spacing(40);

        if let Some(png_result) = &self.saved_png_path {
            let msg = match png_result {
                Ok(path) => format!("Png saved as: {path:?}!"),
                Err(msg) => {
                    format!("Png could not be saved due to:\n{msg:?}")
                }
            };

            controls = controls.push(text(msg));
        }

        let side_content = container(controls)
            .align_x(alignment::Horizontal::Center)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .center_y()
            .center_x();

        let content = row![side_content, image]
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::F5,
                ..
            }) = event
            {
                Some(Message::Screenshot)
            } else {
                None
            }
        })
    }
}

async fn save_to_png(screenshot: Screenshot) -> Result<String, PngError> {
    let path = "screenshot.png".to_string();
    img::save_buffer(
        &path,
        &screenshot.bytes,
        screenshot.size.width,
        screenshot.size.height,
        ColorType::Rgba8,
    )
    .map(|_| path)
    .map_err(|err| PngError(format!("{err:?}")))
}

#[derive(Clone, Debug)]
struct PngError(String);

fn numeric_input(
    placeholder: &str,
    value: Option<u32>,
) -> Element<'_, Option<u32>> {
    text_input(
        placeholder,
        &value.as_ref().map_or_else(String::new, ToString::to_string),
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
    text(content)
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Center)
        .into()
}
