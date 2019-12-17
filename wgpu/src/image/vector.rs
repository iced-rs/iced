use iced_native::svg;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

pub enum Svg {
    Loaded { tree: resvg::usvg::Tree },
    NotFound,
}

impl Svg {
    pub fn viewport_dimensions(&self) -> (u32, u32) {
        match self {
            Svg::Loaded { tree } => {
                let size = tree.svg_node().size;

                (size.width() as u32, size.height() as u32)
            }
            Svg::NotFound => (1, 1),
        }
    }
}

impl std::fmt::Debug for Svg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Svg")
    }
}

#[derive(Debug)]
pub struct Cache {
    svgs: HashMap<u64, Svg>,
    rasterized: Arc<Mutex<HashMap<(u64, u32, u32), Vec<u32>>>>,
    uploaded: HashMap<(u64, u32, u32), Rc<wgpu::BindGroup>>,
    svg_hits: HashSet<u64>,
    uploaded_hits: HashSet<(u64, u32, u32)>,
    rasterizing: Arc<AtomicBool>,
    raster_thread: Option<thread::JoinHandle<()>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            svgs: HashMap::new(),
            rasterized: Arc::new(Mutex::new(HashMap::new())),
            uploaded: HashMap::new(),
            svg_hits: HashSet::new(),
            uploaded_hits: HashSet::new(),
            rasterizing: Arc::new(AtomicBool::new(false)),
            raster_thread: None,
        }
    }

    pub fn load(&mut self, handle: &svg::Handle) -> &Svg {
        if self.svgs.contains_key(&handle.id()) {
            return self.svgs.get(&handle.id()).unwrap();
        }

        let opt = resvg::Options::default();

        let svg = match resvg::usvg::Tree::from_file(handle.path(), &opt.usvg) {
            Ok(tree) => Svg::Loaded { tree },
            Err(_) => Svg::NotFound,
        };

        let _ = self.svgs.insert(handle.id(), svg);
        self.svgs.get(&handle.id()).unwrap()
    }

    pub fn upload(
        &mut self,
        handle: &svg::Handle,
        [w, h]: [f32; 2],
        scale: f32,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Option<Rc<wgpu::BindGroup>> {
        let id = handle.id();

        let _ = self.svg_hits.insert(id);

        let (width, height) =
            ((scale * w).round() as u32, (scale * h).round() as u32);

        // Upload rasterized data
        for (key, raster_data) in self.rasterized.lock().unwrap().drain() {
            let bind_group = Rc::new(create_bind_group(
                &raster_data,
                key.1,
                key.2,
                device,
                encoder,
                texture_layout,
            ));

            let _ = self.uploaded.insert(key, bind_group);
        }

        if let Some(bind_group) = self.uploaded.get(&(id, width, height)) {
            let _ = self.uploaded_hits.insert((id, width, height));

            return Some(bind_group.clone());
        }

        let path = handle.path().to_path_buf();
        let rasterized = self.rasterized.clone();
        let rasterizing = self.rasterizing.clone();

        if !rasterizing.compare_and_swap(false, true, Ordering::Relaxed) {
            self.raster_thread = Some(thread::spawn(move || {
                rasterize(path, rasterized, id, width, height, rasterizing)
            }));
        }

        // If no perfect fit can be found, use the biggest available.
        if let Some((key, value)) = self
            .uploaded
            .iter()
            .filter(|((i, _, _), _)| *i == id)
            .max_by_key(|((_, w, _), _)| w)
        {
            let _ = self.uploaded_hits.insert(*key);
            return Some(value.clone());
        }

        // If no bind group with the right id can be found, block until one can
        // be created.
        if let Some(raster_thread) = self.raster_thread.take() {
            let _ = raster_thread.join();
            return self.upload(
                handle,
                [w, h],
                scale,
                device,
                encoder,
                texture_layout,
            );
        }

        None
    }

    pub fn trim(&mut self) {
        let svg_hits = &self.svg_hits;
        let uploaded_hits = &self.uploaded_hits;

        self.svgs.retain(|k, _| svg_hits.contains(k));
        self.uploaded.retain(|k, _| uploaded_hits.contains(k));
        self.svg_hits.clear();
        self.uploaded_hits.clear();
    }
}

fn rasterize(
    path: PathBuf,
    rasterized: Arc<Mutex<HashMap<(u64, u32, u32), Vec<u32>>>>,
    id: u64,
    width: u32,
    height: u32,
    rasterizing: Arc<AtomicBool>,
) {
    let opt = resvg::Options::default();

    if let Ok(tree) = resvg::usvg::Tree::from_file(path, &opt.usvg) {
        let screen_size = resvg::ScreenSize::new(width, height).unwrap();

        let mut canvas =
            resvg::raqote::DrawTarget::new(width as i32, height as i32);

        resvg::backend_raqote::render_to_canvas(
            &tree,
            &resvg::Options::default(),
            screen_size,
            &mut canvas,
        );

        let _ = rasterized
            .lock()
            .unwrap()
            .insert((id, width, height), canvas.into_vec());
        rasterizing.store(false, Ordering::Relaxed);
    }
}

fn create_bind_group(
    data: &Vec<u32>,
    width: u32,
    height: u32,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
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
        usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
    });

    let temp_buf = device
        .create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(data);

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

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(
                &texture.create_default_view(),
            ),
        }],
    })
}
