//! Load and operate on images.
#[cfg(feature = "image")]
use crate::core::Bytes;

use crate::core::Rectangle;
use crate::core::image;
use crate::core::svg;

/// A raster or vector image.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    /// A raster image.
    Raster {
        image: image::Image,
        bounds: Rectangle,
        clip_bounds: Rectangle,
    },

    /// A vector image.
    Vector {
        svg: svg::Svg,
        bounds: Rectangle,
        clip_bounds: Rectangle,
    },
}

impl Image {
    /// Returns the bounds of the [`Image`].
    pub fn bounds(&self) -> Rectangle {
        match self {
            Image::Raster { image, bounds, .. } => {
                bounds.rotate(image.rotation)
            }
            Image::Vector { svg, bounds, .. } => bounds.rotate(svg.rotation),
        }
    }
}

/// An image buffer.
#[cfg(feature = "image")]
pub type Buffer = ::image::ImageBuffer<::image::Rgba<u8>, Bytes>;

#[cfg(feature = "image")]
/// Tries to load an image by its [`Handle`].
///
/// [`Handle`]: image::Handle
pub fn load(handle: &image::Handle) -> Result<Buffer, image::Error> {
    use bitflags::bitflags;

    bitflags! {
        struct Operation: u8 {
            const FLIP_HORIZONTALLY = 0b1;
            const ROTATE_180 = 0b10;
            const FLIP_VERTICALLY= 0b100;
            const ROTATE_90 = 0b1000;
            const ROTATE_270 = 0b10000;
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
                .map(|value| match value {
                    1 => Operation::empty(),
                    2 => Operation::FLIP_HORIZONTALLY,
                    3 => Operation::ROTATE_180,
                    4 => Operation::FLIP_VERTICALLY,
                    5 => Operation::ROTATE_90 | Operation::FLIP_HORIZONTALLY,
                    6 => Operation::ROTATE_90,
                    7 => Operation::ROTATE_90 | Operation::FLIP_VERTICALLY,
                    8 => Operation::ROTATE_270,
                    _ => Operation::empty(),
                })
                .unwrap_or_else(Self::empty))
        }

        fn perform(
            self,
            mut image: ::image::DynamicImage,
        ) -> ::image::DynamicImage {
            use ::image::imageops;

            if self.contains(Operation::ROTATE_90) {
                image = imageops::rotate90(&image).into();
            }

            if self.contains(Self::ROTATE_180) {
                imageops::rotate180_in_place(&mut image);
            }

            if self.contains(Operation::ROTATE_270) {
                image = imageops::rotate270(&image).into();
            }

            if self.contains(Self::FLIP_VERTICALLY) {
                imageops::flip_vertical_in_place(&mut image);
            }

            if self.contains(Self::FLIP_HORIZONTALLY) {
                imageops::flip_horizontal_in_place(&mut image);
            }

            image
        }
    }

    let (width, height, pixels) = match handle {
        image::Handle::Path(_, path) => {
            let image = ::image::open(path).map_err(to_error)?;

            let operation = std::fs::File::open(path)
                .ok()
                .map(std::io::BufReader::new)
                .and_then(|mut reader| Operation::from_exif(&mut reader).ok())
                .unwrap_or_else(Operation::empty);

            let rgba = operation.perform(image).into_rgba8();

            (rgba.width(), rgba.height(), Bytes::from(rgba.into_raw()))
        }
        image::Handle::Bytes(_, bytes) => {
            let image = ::image::load_from_memory(bytes).map_err(to_error)?;

            let operation =
                Operation::from_exif(&mut std::io::Cursor::new(bytes))
                    .ok()
                    .unwrap_or_else(Operation::empty);

            let rgba = operation.perform(image).into_rgba8();

            (rgba.width(), rgba.height(), Bytes::from(rgba.into_raw()))
        }
        image::Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } => (*width, *height, pixels.clone()),
    };

    if let Some(image) = ::image::ImageBuffer::from_raw(width, height, pixels) {
        Ok(image)
    } else {
        Err(to_error(::image::error::ImageError::Limits(
            ::image::error::LimitError::from_kind(
                ::image::error::LimitErrorKind::DimensionError,
            ),
        )))
    }
}

#[cfg(feature = "image")]
fn to_error(error: ::image::ImageError) -> image::Error {
    use std::sync::Arc;

    match error {
        ::image::ImageError::IoError(error) => {
            image::Error::Inaccessible(Arc::new(error))
        }
        error => image::Error::Invalid(Arc::new(error)),
    }
}
