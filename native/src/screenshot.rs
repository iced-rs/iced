//! Data structures for handling screenshots

use std::sync::Arc;

use std::io::Write;
#[derive(Debug, Clone)]
/// A single screencap. The payload in this structure is always a raw RGB image
pub struct Screenshot {
    payload: Arc<Vec<u8>>,
    width: u32,
    height: u32,
}

impl Screenshot {
    /// Create a new screenshot object
    pub fn new(payload: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            payload: Arc::new(payload),
            width,
            height,
        }
    }

    /// Saves the screenshot to the input path
    pub fn save_image_to_png<S: AsRef<std::path::Path>>(&self, path: S) {
        let mut png_encoder = png::Encoder::new(
            std::fs::File::create(path).unwrap(),
            self.width as u32,
            self.height as u32,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::RGBA);

        let bytes_per_pixel = std::mem::size_of::<u32>();
        let align = 256;
        let unpadded_bytes_per_row = self.width * bytes_per_pixel as u32;
        let padded_bytes_per_row = unpadded_bytes_per_row
            + (align - unpadded_bytes_per_row % align) % align;

        let mut png_writer = png_encoder
            .write_header()
            .unwrap()
            .into_stream_writer_with_size(padded_bytes_per_row as usize);

        let mut start = 0;

        loop {
            let end = start + unpadded_bytes_per_row as usize;
            if end >= self.payload.len() {
                break;
            }
            png_writer
                .write_all(&self.payload.as_slice()[start..end])
                .unwrap();
            start += padded_bytes_per_row as usize;
        }
        png_writer.finish().expect("Png writer finish failed");
    }
}
