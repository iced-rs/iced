use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::KeyCode;
use iced::theme::{Button, Container};
use iced::widget::runtime::{CropError, Screenshot};
use iced::widget::{
    button, column as col, container, image as iced_image, row, text,
    text_input,
};
use iced::{
    event, executor, keyboard, subscription, Alignment, Application, Command,
    ContentFit, Element, Event, Length, Rectangle, Renderer, Subscription,
    Theme,
};
use image as img;
use image::ColorType;

fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    Example::run(iced::Settings::default())
}

struct Example {
    screenshot: Option<Screenshot>,
    saved_png_path: Option<Result<String, PngError>>,
    png_saving: bool,
    crop_error: Option<CropError>,
    x_input_value: u32,
    y_input_value: u32,
    width_input_value: u32,
    height_input_value: u32,
}

#[derive(Clone, Debug)]
enum Message {
    Crop,
    Screenshot,
    ScreenshotData(Screenshot),
    Png,
    PngSaved(Result<String, PngError>),
    XInputChanged(String),
    YInputChanged(String),
    WidthInputChanged(String),
    HeightInputChanged(String),
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
                x_input_value: 0,
                y_input_value: 0,
                width_input_value: 0,
                height_input_value: 0,
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
                    return Command::perform(
                        save_to_png(screenshot.clone()),
                        Message::PngSaved,
                    );
                }
                self.png_saving = true;
            }
            Message::PngSaved(res) => {
                self.png_saving = false;
                self.saved_png_path = Some(res);
            }
            Message::XInputChanged(new) => {
                if let Ok(value) = new.parse::<u32>() {
                    self.x_input_value = value;
                }
            }
            Message::YInputChanged(new) => {
                if let Ok(value) = new.parse::<u32>() {
                    self.y_input_value = value;
                }
            }
            Message::WidthInputChanged(new) => {
                if let Ok(value) = new.parse::<u32>() {
                    self.width_input_value = value;
                }
            }
            Message::HeightInputChanged(new) => {
                if let Ok(value) = new.parse::<u32>() {
                    self.height_input_value = value;
                }
            }
            Message::Crop => {
                if let Some(screenshot) = &self.screenshot {
                    let cropped = screenshot.crop(Rectangle::<u32> {
                        x: self.x_input_value,
                        y: self.y_input_value,
                        width: self.width_input_value,
                        height: self.height_input_value,
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
            iced_image(iced_image::Handle::from_pixels(
                screenshot.size.width,
                screenshot.size.height,
                screenshot.bytes.clone(),
            ))
            .content_fit(ContentFit::ScaleDown)
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
            text("X:").vertical_alignment(Vertical::Center).width(14),
            text_input("0", &format!("{}", self.x_input_value),)
                .on_input(Message::XInputChanged)
                .width(40),
            text("Y:").vertical_alignment(Vertical::Center).width(14),
            text_input("0", &format!("{}", self.y_input_value),)
                .on_input(Message::YInputChanged)
                .width(40),
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let crop_dimension_controls = row![
            text("W:").vertical_alignment(Vertical::Center).width(14),
            text_input("0", &format!("{}", self.width_input_value),)
                .on_input(Message::WidthInputChanged)
                .width(40),
            text("H:").vertical_alignment(Vertical::Center).width(14),
            text_input("0", &format!("{}", self.height_input_value),)
                .on_input(Message::HeightInputChanged)
                .width(40),
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let mut crop_controls =
            col![crop_origin_controls, crop_dimension_controls]
                .spacing(10)
                .align_items(Alignment::Center);

        if let Some(crop_error) = &self.crop_error {
            crop_controls = crop_controls
                .push(text(format!("Crop error! \n{}", crop_error)));
        }

        let png_button = if !self.png_saving {
            button("Save to png.")
                .style(Button::Secondary)
                .padding([10, 20, 10, 20])
                .on_press(Message::Png)
        } else {
            button("Saving..")
                .style(Button::Secondary)
                .padding([10, 20, 10, 20])
        };

        let mut controls = col![
            button("Screenshot!")
                .padding([10, 20, 10, 20])
                .on_press(Message::Screenshot),
            button("Crop")
                .style(Button::Destructive)
                .padding([10, 20, 10, 20])
                .on_press(Message::Crop),
            crop_controls,
            png_button,
        ]
        .spacing(40)
        .align_items(Alignment::Center);

        if let Some(png_result) = &self.saved_png_path {
            let msg = match png_result {
                Ok(path) => format!("Png saved as: {:?}!", path),
                Err(msg) => {
                    format!("Png could not be saved due to:\n{:?}", msg)
                }
            };

            controls = controls.push(text(msg));
        }

        let side_content = container(controls)
            .align_x(Horizontal::Center)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .center_y()
            .center_x();

        let content = row![side_content, image]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center);

        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events_with(|event, status| {
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
    .map_err(|err| PngError(format!("{:?}", err)))
}

#[derive(Clone, Debug)]
struct PngError(String);
