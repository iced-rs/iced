//! Data structures for handling screenshots in wgpu

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
#[derive(Debug, Clone, PartialEq)]
/// A single screencap. The payload in this structure is always a raw RGB image
pub struct Screenshot {
    payload: Arc<Vec<u8>>,
    width: u32,
    height: u32,
    encoding: ColorType,
}
#[derive(Debug, Clone, Copy, PartialEq)]
///Decribes pixel encoding. Equivalent to png::ColorType
pub enum ColorType {
    /// Screenshot has RBGA format
    Rgba,
    /// Screenshot has RBG format
    Rgb,
}

impl Into<png::ColorType> for ColorType {
    fn into(self) -> png::ColorType {
        match self {
            Self::Rgb => png::ColorType::Rgb,
            Self::Rgba => png::ColorType::Rgba,
        }
    }
}

impl From<png::ColorType> for ColorType {
    fn from(color: png::ColorType) -> ColorType {
        match color {
            png::ColorType::Rgb => ColorType::Rgb,
            png::ColorType::Rgba => ColorType::Rgba,
            _ => panic!("Unsupported"),
        }
    }
}

impl Screenshot {
    /// Create a new [`Screenshot`] object
    pub fn new(payload: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            payload: Arc::new(payload),
            width,
            height,
            encoding: ColorType::Rgba,
        }
    }

    /// Sets the encoding field for a [`Screenshot`] object
    pub fn encoding(mut self, color_type: ColorType) -> Self {
        self.encoding = color_type;
        self
    }

    /// Creates a [`Screenshot`] object from png
    pub fn from_png<S: AsRef<std::path::Path>>(
        path: S,
    ) -> Result<Self, Box<dyn Error>> {
        let decoder = png::Decoder::new(File::open(path)?);
        let mut reader = decoder.read_info()?;
        let mut payload = vec![0; reader.output_buffer_size()];
        let out = reader.next_frame(&mut payload)?;

        Ok(Self {
            payload: Arc::new(payload),
            width: out.width,
            height: out.height,
            encoding: reader.info().color_type.into(),
        })
    }

    /// Saves the [`Screenshot`] to the input path
    pub fn save_image_to_png<S: AsRef<std::path::Path>>(&self, path: S) {
        let mut png_encoder = png::Encoder::new(
            std::fs::File::create(path).unwrap(),
            self.width as u32,
            self.height as u32,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(self.encoding.into());

        let bytes_per_pixel = match self.encoding {
            ColorType::Rgba => std::mem::size_of::<u32>(),
            ColorType::Rgb => std::mem::size_of::<u8>() * 3,
        };

        let align = 256;
        let unpadded_bytes_per_row = self.width * bytes_per_pixel as u32;
        let padded_bytes_per_row = unpadded_bytes_per_row
            + (align - unpadded_bytes_per_row % align) % align;

        let mut png_writer = png_encoder
            .write_header()
            .unwrap()
            .into_stream_writer_with_size(padded_bytes_per_row as usize)
            .unwrap();

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

        if start <= self.payload.len() {
            png_writer
                .write_all(&self.payload.as_slice()[start..])
                .unwrap();
        }

        png_writer.finish().expect("Png writer finish failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn round_trip() {
        let payload = vec![0xfe; 4 * 512 * 512];
        let ss = Screenshot::new(payload, 512, 512);
        let temp_png =
            tempfile::NamedTempFile::new().expect("tempfile creation failed");
        ss.save_image_to_png(temp_png.path());
        let ss_from_file =
            Screenshot::from_png(temp_png.path()).expect("Decoder fail");
        assert_eq!(ss, ss_from_file);
    }

    #[test]
    fn round_trip_rgb() {
        let payload = vec![0xfe; 3 * 512 * 512];
        let ss = Screenshot::new(payload, 512, 512).encoding(ColorType::Rgb);
        let temp_png =
            tempfile::NamedTempFile::new().expect("tempfile creation failed");
        ss.save_image_to_png(temp_png.path());
        let ss_from_file =
            Screenshot::from_png(temp_png.path()).expect("Decoder fail");
        assert_eq!(ss, ss_from_file);
    }
}
