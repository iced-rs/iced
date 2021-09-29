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
                        .and_then(|mut reader| exif_orientation(&mut reader));
                    Memory::Host(fix_orientation(image.to_bgra8(), orientation))
                } else {
                    Memory::NotFound
                }
            }
            image::Data::Bytes(bytes) => {
                if let Ok(image) = ::image_rs::load_from_memory(&bytes) {
                    let orientation =
                        exif_orientation(&mut std::io::Cursor::new(bytes));
                    Memory::Host(fix_orientation(image.to_bgra8(), orientation))
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

fn fix_orientation(
    mut img: ::image_rs::ImageBuffer<::image_rs::Bgra<u8>, Vec<u8>>,
    orientation: Option<u32>,
) -> ::image_rs::ImageBuffer<::image_rs::Bgra<u8>, Vec<u8>> {
    use ::image_rs::imageops::*;
    match orientation.unwrap_or(1) {
        2 => flip_horizontal_in_place(&mut img),
        3 => rotate180_in_place(&mut img),
        4 => flip_vertical_in_place(&mut img),
        5 => {
            img = rotate90(&img);
            flip_horizontal_in_place(&mut img);
        }
        6 => img = rotate90(&img),
        7 => {
            img = rotate90(&img);
            flip_vertical_in_place(&mut img);
        }
        8 => img = rotate270(&img),
        _ => {}
    };
    img
}

// Meaning of the returned value is described e.g. at:
// https://magnushoff.com/articles/jpeg-orientation/
fn exif_orientation<R>(reader: &mut R) -> Option<u32>
where
    R: std::io::BufRead + std::io::Seek,
{
    let exif = ::exif::Reader::new().read_from_container(reader).ok()?;
    exif.get_field(::exif::Tag::Orientation, ::exif::In::PRIMARY)?
        .value
        .get_uint(0)
}
