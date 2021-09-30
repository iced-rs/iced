use crate::image::atlas::{self, Atlas};
use iced_native::image;
use std::collections::{HashMap, HashSet};

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
                if let Ok(image) = ::image_rs::open(path) {
                    let orientation = std::fs::File::open(path)
                        .ok()
                        .map(std::io::BufReader::new)
                        .and_then(|mut reader| {
                            Orientation::from_exif(&mut reader).ok()
                        })
                        .unwrap_or(Orientation::Default);

                    Memory::Host(orientation.apply(image.to_bgra8()))
                } else {
                    Memory::NotFound
                }
            }
            image::Data::Bytes(bytes) => {
                if let Ok(image) = ::image_rs::load_from_memory(&bytes) {
                    let orientation = Orientation::from_exif(
                        &mut std::io::Cursor::new(bytes),
                    )
                    .unwrap_or(Orientation::Default);

                    Memory::Host(orientation.apply(image.to_bgra8()))
                } else {
                    Memory::Invalid
                }
            }
            image::Data::Pixels {
                width,
                height,
                pixels,
            } => {
                if let Some(image) = ::image_rs::ImageBuffer::from_vec(
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

#[derive(Debug, Clone, Copy)]
enum Orientation {
    Default,
    FlippedHorizontally,
    FlippedVertically,
    Rotated90,
    Rotated180,
    Rotated270,
    Rotated90AndFlippedHorizontally,
    Rotated90AndFlippedVertically,
}

impl Orientation {
    // Meaning of the returned value is described e.g. at:
    // https://magnushoff.com/articles/jpeg-orientation/
    fn from_exif<R>(reader: &mut R) -> Result<Self, exif::Error>
    where
        R: std::io::BufRead + std::io::Seek,
    {
        let exif = ::exif::Reader::new().read_from_container(reader)?;

        Ok(exif
            .get_field(::exif::Tag::Orientation, ::exif::In::PRIMARY)
            .and_then(|field| field.value.get_uint(0))
            .map(|value| match value {
                2 => Orientation::FlippedHorizontally,
                3 => Orientation::Rotated180,
                4 => Orientation::FlippedVertically,
                5 => Orientation::Rotated90AndFlippedHorizontally,
                6 => Orientation::Rotated90,
                7 => Orientation::Rotated90AndFlippedVertically,
                8 => Orientation::Rotated270,
                _ => Orientation::Default,
            })
            .unwrap_or(Orientation::Default))
    }

    fn apply(
        self,
        mut img: ::image_rs::ImageBuffer<::image_rs::Bgra<u8>, Vec<u8>>,
    ) -> ::image_rs::ImageBuffer<::image_rs::Bgra<u8>, Vec<u8>> {
        use ::image_rs::imageops::*;

        match self {
            Self::FlippedHorizontally => flip_horizontal_in_place(&mut img),
            Self::Rotated180 => rotate180_in_place(&mut img),
            Self::FlippedVertically => flip_vertical_in_place(&mut img),
            Self::Rotated90AndFlippedHorizontally => {
                img = rotate90(&img);
                flip_horizontal_in_place(&mut img);
            }
            Self::Rotated90 => img = rotate90(&img),
            Self::Rotated90AndFlippedVertically => {
                img = rotate90(&img);
                flip_vertical_in_place(&mut img);
            }
            Self::Rotated270 => img = rotate270(&img),
            Self::Default => {}
        };

        img
    }
}
