//! Load and operate on images.
#[cfg(feature = "image")]
pub use ::image as image_rs;

use crate::core::{image, svg, Color, Radians, Rectangle};

/// A raster or vector image.
#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    /// A raster image.
    Raster {
        /// The handle of a raster image.
        handle: image::Handle,

        /// The filter method of a raster image.
        filter_method: image::FilterMethod,

        /// The bounds of the image.
        bounds: Rectangle,

        /// The rotation of the image.
        rotation: Radians,

        /// The opacity of the image.
        opacity: f32,
    },
    /// A vector image.
    Vector {
        /// The handle of a vector image.
        handle: svg::Handle,

        /// The [`Color`] filter
        color: Option<Color>,

        /// The bounds of the image.
        bounds: Rectangle,

        /// The rotation of the image.
        rotation: Radians,

        /// The opacity of the image.
        opacity: f32,
    },
}

impl Image {
    /// Returns the bounds of the [`Image`].
    pub fn bounds(&self) -> Rectangle {
        match self {
            Image::Raster {
                bounds, rotation, ..
            }
            | Image::Vector {
                bounds, rotation, ..
            } => bounds.rotate(*rotation),
        }
    }
}

#[cfg(feature = "image")]
/// Tries to load an image by its [`Handle`].
///
/// [`Handle`]: image::Handle
pub fn load(
    handle: &image::Handle,
) -> ::image::ImageResult<::image::ImageBuffer<::image::Rgba<u8>, image::Bytes>>
{
    use bitflags::bitflags;

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

        fn perform(
            self,
            mut image: ::image::DynamicImage,
        ) -> ::image::DynamicImage {
            use ::image::imageops;

            if self.contains(Self::FLIP_DIAGONALLY) {
                imageops::flip_vertical_in_place(&mut image);
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

    let (width, height, pixels) = match handle {
        image::Handle::Path(_, path) => {
            let image = ::image::open(path)?;

            let operation = std::fs::File::open(path)
                .ok()
                .map(std::io::BufReader::new)
                .and_then(|mut reader| Operation::from_exif(&mut reader).ok())
                .unwrap_or_else(Operation::empty);

            let rgba = operation.perform(image).into_rgba8();

            (
                rgba.width(),
                rgba.height(),
                image::Bytes::from(rgba.into_raw()),
            )
        }
        image::Handle::Bytes(_, bytes) => {
            let image = ::image::load_from_memory(bytes)?;
            let operation =
                Operation::from_exif(&mut std::io::Cursor::new(bytes))
                    .ok()
                    .unwrap_or_else(Operation::empty);

            let rgba = operation.perform(image).into_rgba8();

            (
                rgba.width(),
                rgba.height(),
                image::Bytes::from(rgba.into_raw()),
            )
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
        Err(::image::error::ImageError::Limits(
            ::image::error::LimitError::from_kind(
                ::image::error::LimitErrorKind::DimensionError,
            ),
        ))
    }
}
