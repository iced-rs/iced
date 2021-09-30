use crate::image::atlas::{self, Atlas};
use iced_native::image;
use std::collections::{HashMap, HashSet};

use bitflags::bitflags;

#[derive(Debug)]
pub enum Memory {
    Host(::image_rs::ImageBuffer<::image_rs::Bgra<u8>, Vec<u8>>),
    Device(atlas::Entry),
    NotFound,
    Invalid,
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host(image) => image.dimensions(),
            Memory::Device(entry) => entry.size(),
            Memory::NotFound => (1, 1),
            Memory::Invalid => (1, 1),
        }
    }
}

#[derive(Debug)]
pub struct Cache {
    map: HashMap<u64, Memory>,
    hits: HashSet<u64>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hits: HashSet::new(),
        }
    }

    pub fn load(&mut self, handle: &image::Handle) -> &mut Memory {
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

                    Memory::Host(operation.perform(image.to_bgra8()))
                } else {
                    Memory::NotFound
                }
            }
            image::Data::Bytes(bytes) => {
                if let Ok(image) = image_rs::load_from_memory(&bytes) {
                    let operation =
                        Operation::from_exif(&mut std::io::Cursor::new(bytes))
                            .ok()
                            .unwrap_or_else(Operation::empty);

                    Memory::Host(operation.perform(image.to_bgra8()))
                } else {
                    Memory::Invalid
                }
            }
            image::Data::Pixels {
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

    pub fn upload(
        &mut self,
        handle: &image::Handle,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        atlas: &mut Atlas,
    ) -> Option<&atlas::Entry> {
        let memory = self.load(handle);

        if let Memory::Host(image) = memory {
            let (width, height) = image.dimensions();

            let entry = atlas.upload(width, height, &image, device, encoder)?;

            *memory = Memory::Device(entry);
        }

        if let Memory::Device(allocation) = memory {
            Some(allocation)
        } else {
            None
        }
    }

    pub fn trim(&mut self, atlas: &mut Atlas) {
        let hits = &self.hits;

        self.map.retain(|k, memory| {
            let retain = hits.contains(k);

            if !retain {
                if let Memory::Device(entry) = memory {
                    atlas.remove(entry);
                }
            }

            retain
        });

        self.hits.clear();
    }

    fn get(&mut self, handle: &image::Handle) -> Option<&mut Memory> {
        let _ = self.hits.insert(handle.id());

        self.map.get_mut(&handle.id())
    }

    fn insert(&mut self, handle: &image::Handle, memory: Memory) {
        let _ = self.map.insert(handle.id(), memory);
    }

    fn contains(&self, handle: &image::Handle) -> bool {
        self.map.contains_key(&handle.id())
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
        use std::convert::TryFrom;

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
