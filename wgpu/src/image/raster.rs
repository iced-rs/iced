use iced_native::image;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

type ImageBuffer = ::image::ImageBuffer<::image::Bgra<u8>, Vec<u8>>;

#[derive(Debug)]
pub enum Memory {
    Host(Rc<ImageBuffer>),
    Device {
        image: Rc<ImageBuffer>,
        bind_group: Rc<wgpu::BindGroup>,
        width: u32,
        height: u32,
    },
    NotFound,
    Invalid,
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host(image) => image.dimensions(),
            Memory::Device { width, height, .. } => (*width, *height),
            Memory::NotFound => (1, 1),
            Memory::Invalid => (1, 1),
        }
    }
}

#[derive(Debug)]
pub struct Cache {
    map: HashMap<u64, Memory>,
    load_hits: HashSet<u64>,
    upload_hits: HashSet<u64>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            load_hits: HashSet::new(),
            upload_hits: HashSet::new(),
        }
    }

    pub fn load(&mut self, handle: &image::Handle) -> &mut Memory {
        if self.contains(handle) {
            return self.get(handle).unwrap();
        }

        let memory = match handle.data() {
            image::Data::Path(path) => {
                if let Ok(image) = ::image::open(path) {
                    Memory::Host(Rc::new(image.to_bgra()))
                } else {
                    Memory::NotFound
                }
            }
            image::Data::Bytes(bytes) => {
                if let Ok(image) = ::image::load_from_memory(&bytes) {
                    Memory::Host(Rc::new(image.to_bgra()))
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
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Option<Rc<wgpu::BindGroup>> {
        let memory = {
            let _ = self.upload_hits.insert(handle.id());

            self.load(handle)
        };

        match memory {
            Memory::Host(image) => {
                let (width, height) = image.dimensions();

                let extent = wgpu::Extent3d {
                    width,
                    height,
                    depth: 1,
                };

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: extent,
                    array_layer_count: 1,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUsage::COPY_DST
                        | wgpu::TextureUsage::SAMPLED,
                });

                let temp_buf = {
                    let flat_samples = image.as_flat_samples();
                    let slice = flat_samples.as_slice();

                    device
                        .create_buffer_mapped(
                            slice.len(),
                            wgpu::BufferUsage::COPY_SRC,
                        )
                        .fill_from_slice(slice)
                };

                encoder.copy_buffer_to_texture(
                    wgpu::BufferCopyView {
                        buffer: &temp_buf,
                        offset: 0,
                        row_pitch: 4 * width as u32,
                        image_height: height as u32,
                    },
                    wgpu::TextureCopyView {
                        texture: &texture,
                        array_layer: 0,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    extent,
                );

                let bind_group =
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: texture_layout,
                        bindings: &[wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &texture.create_default_view(),
                            ),
                        }],
                    });

                let bind_group = Rc::new(bind_group);

                *memory = Memory::Device {
                    image: image.clone(),
                    bind_group: bind_group.clone(),
                    width,
                    height,
                };

                Some(bind_group)
            }
            Memory::Device { bind_group, .. } => Some(bind_group.clone()),
            Memory::NotFound => None,
            Memory::Invalid => None,
        }
    }

    pub fn trim(&mut self) {
        let load_hits = &self.load_hits;
        let upload_hits = &self.upload_hits;

        self.map.retain(|k, memory| {
            if upload_hits.contains(k) {
                return true;
            }

            let retain = load_hits.contains(k);

            if let Memory::Device { image, .. } = memory {
                if retain {
                    *memory = Memory::Host(image.clone());
                }
            }

            retain
        });

        self.load_hits.clear();
        self.upload_hits.clear();
    }

    fn get(&mut self, handle: &image::Handle) -> Option<&mut Memory> {
        let _ = self.load_hits.insert(handle.id());

        self.map.get_mut(&handle.id())
    }

    fn insert(&mut self, handle: &image::Handle, memory: Memory) {
        let _ = self.map.insert(handle.id(), memory);
    }

    fn contains(&self, handle: &image::Handle) -> bool {
        self.map.contains_key(&handle.id())
    }
}
