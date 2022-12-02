//! Raster image loading and caching.
use crate::image::Storage;
use crate::Size;

use iced_native::image;

use bitflags::bitflags;
use std::collections::{HashMap, HashSet};

/// Entry in cache corresponding to an image handle
#[derive(Debug)]
pub enum Memory<T: Storage> {
    /// Image data on host
    Host(::image_rs::ImageBuffer<::image_rs::Rgba<u8>, Vec<u8>>),
    /// Storage entry
    Device(T::Entry),
    /// Image not found
    NotFound,
    /// Invalid image data
    Invalid,
}

impl<T: Storage> Memory<T> {
    /// Width and height of image
    pub fn dimensions(&self) -> Size<u32> {
        use crate::image::storage::Entry;

        match self {
            Memory::Host(image) => {
                let (width, height) = image.dimensions();

                Size::new(width, height)
            }
            Memory::Device(entry) => entry.size(),
            Memory::NotFound => Size::new(1, 1),
            Memory::Invalid => Size::new(1, 1),
        }
    }
}

/// Caches image raster data
#[derive(Debug)]
pub struct Cache<T: Storage> {
    map: HashMap<u64, Memory<T>>,
    hits: HashSet<u64>,
}

impl<T: Storage> Cache<T> {
    /// Load image
    pub fn load(&mut self, handle: &image::Handle) -> &mut Memory<T> {
        if self.contains(handle) {
            return self.get(handle).unwrap();
        }

        let memory = match handle.data() {
            image::Data::Path(path) => {
                if let Ok(image) = image_rs::open(path) {
                    let operation = std::fs::File::open(path)
                        .ok()
                        .map(std::io::BufReader::new)
                        .and_then(|mut reader| {
                            Operation::from_exif(&mut reader).ok()
                        })
                        .unwrap_or_else(Operation::empty);

                    Memory::Host(operation.perform(image.to_rgba8()))
                } else {
                    Memory::NotFound
                }
            }
            image::Data::Bytes(bytes) => {
                if let Ok(image) = image_rs::load_from_memory(bytes) {
                    let operation =
                        Operation::from_exif(&mut std::io::Cursor::new(bytes))
                            .ok()
                            .unwrap_or_else(Operation::empty);

                    Memory::Host(operation.perform(image.to_rgba8()))
                } else {
                    Memory::Invalid
                }
            }
            image::Data::Rgba {
                width,
                height,
                pixels,
            } => {
                if let Some(image) = image_rs::ImageBuffer::from_vec(
                    *width,
                    *height,
                    pixels.to_vec(),
                ) {
                    Memory::Host(image)
                } else {
                    Memory::Invalid
                }
            }
        };

        self.insert(handle, memory);
        self.get(handle).unwrap()
    }

    /// Load image and upload raster data
    pub fn upload(
        &mut self,
        handle: &image::Handle,
        state: &mut T::State<'_>,
        storage: &mut T,
    ) -> Option<&T::Entry> {
        let memory = self.load(handle);

        if let Memory::Host(image) = memory {
            let (width, height) = image.dimensions();

            let entry = storage.upload(width, height, image, state)?;

            *memory = Memory::Device(entry);
        }

        if let Memory::Device(allocation) = memory {
            Some(allocation)
        } else {
            None
        }
    }

    /// Trim cache misses from cache
    pub fn trim(&mut self, storage: &mut T, state: &mut T::State<'_>) {
        let hits = &self.hits;

        self.map.retain(|k, memory| {
            let retain = hits.contains(k);

            if !retain {
                if let Memory::Device(entry) = memory {
                    storage.remove(entry, state);
                }
            }

            retain
        });

        self.hits.clear();
    }

    fn get(&mut self, handle: &image::Handle) -> Option<&mut Memory<T>> {
        let _ = self.hits.insert(handle.id());

        self.map.get_mut(&handle.id())
    }

    fn insert(&mut self, handle: &image::Handle, memory: Memory<T>) {
        let _ = self.map.insert(handle.id(), memory);
    }

    fn contains(&self, handle: &image::Handle) -> bool {
        self.map.contains_key(&handle.id())
    }
}

impl<T: Storage> Default for Cache<T> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            hits: HashSet::new(),
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

    fn perform<P>(
        self,
        image: image_rs::ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> image_rs::ImageBuffer<P, Vec<P::Subpixel>>
    where
        P: image_rs::Pixel + 'static,
    {
        use image_rs::imageops;

        let mut image = if self.contains(Self::FLIP_DIAGONALLY) {
            flip_diagonally(image)
        } else {
            image
        };

        if self.contains(Self::ROTATE_180) {
            imageops::rotate180_in_place(&mut image);
        }

        if self.contains(Self::FLIP_HORIZONTALLY) {
            imageops::flip_horizontal_in_place(&mut image);
        }

        image
    }
}

fn flip_diagonally<I>(
    image: I,
) -> image_rs::ImageBuffer<I::Pixel, Vec<<I::Pixel as image_rs::Pixel>::Subpixel>>
where
    I: image_rs::GenericImage,
    I::Pixel: 'static,
{
    let (width, height) = image.dimensions();
    let mut out = image_rs::ImageBuffer::new(height, width);

    for x in 0..width {
        for y in 0..height {
            let p = image.get_pixel(x, y);

            out.put_pixel(y, x, p);
        }
    }

    out
}
