//! Data structures for handling screenshots

extern crate image as eimage;

#[derive(Debug, Clone)]
/// A single screencap. The payload in this structure is always a raw RGB image
pub struct Screenshot {
    payload: Vec<u8>,
    width: u32,
    height: u32,
}

impl Screenshot {
    /// Create a new screenshot object
    pub fn new(payload: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            payload,
            width,
            height,
        }
    }

    /// Convert a screenshot into an RgbImage object
    pub fn into_image(&self) -> eimage::RgbaImage {
        eimage::RgbaImage::from_raw(
            self.width,
            self.height,
            self.payload.clone(),
        )
        .unwrap()
    }

    /// Saves the screenshot to the input path
    pub fn save_image_to_path<S: AsRef<std::path::Path>>(&self, path: S) {
        self.into_image()
            .save(path.as_ref())
            .expect("Saving image to path failed");
    }
}
