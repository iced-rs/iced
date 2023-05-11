//! Load and operate on images.
use crate::core::image::{Data, Handle};

use bitflags::bitflags;

pub use ::image as image_rs;

/// Tries to load an image by its [`Handle`].
pub fn load(handle: &Handle) -> image_rs::ImageResult<image_rs::DynamicImage> {
    match handle.data() {
        Data::Path(path) => {
            let image = ::image::open(path)?;

            let operation = std::fs::File::open(path)
                .ok()
                .map(std::io::BufReader::new)
                .and_then(|mut reader| Operation::from_exif(&mut reader).ok())
                .unwrap_or_else(Operation::empty);

            Ok(operation.perform(image))
        }
        Data::Bytes(bytes) => {
            let image = ::image::load_from_memory(bytes)?;
            let operation =
                Operation::from_exif(&mut std::io::Cursor::new(bytes))
                    .ok()
                    .unwrap_or_else(Operation::empty);

            Ok(operation.perform(image))
        }
        Data::Rgba {
            width,
            height,
            pixels,
        } => {
            if let Some(image) = image_rs::ImageBuffer::from_vec(
                *width,
                *height,
                pixels.to_vec(),
            ) {
                Ok(image_rs::DynamicImage::ImageRgba8(image))
            } else {
                Err(image_rs::error::ImageError::Limits(
                    image_rs::error::LimitError::from_kind(
                        image_rs::error::LimitErrorKind::DimensionError,
                    ),
                ))
            }
        }
    }
}

bitflags! {
    struct Operation: u8 {
        const FLIP_HORIZONTALLY = 0b001;
        const ROTATE_180 = 0b010;
        const FLIP_DIAGONALLY = 0b100;
    }
}

impl Operation {
    // Meaning of the returned value is described e.g. at:
    // https://magnushoff.com/articles/jpeg-orientation/
    fn from_exif<R>(reader: &mut R) -> Result<Self, exif::Error>
    where
        R: std::io::BufRead + std::io::Seek,
    {
        let exif = exif::Reader::new().read_from_container(reader)?;

        Ok(exif
            .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
            .and_then(|field| field.value.get_uint(0))
            .and_then(|value| u8::try_from(value).ok())
            .and_then(|value| Self::from_bits(value.saturating_sub(1)))
            .unwrap_or_else(Self::empty))
    }

    fn perform(self, mut image: image::DynamicImage) -> image::DynamicImage {
        use image::imageops;

        if self.contains(Self::FLIP_DIAGONALLY) {
            imageops::flip_vertical_in_place(&mut image)
        }

        if self.contains(Self::ROTATE_180) {
            imageops::rotate180_in_place(&mut image);
        }

        if self.contains(Self::FLIP_HORIZONTALLY) {
            imageops::flip_horizontal_in_place(&mut image);
        }

        image
    }
}
