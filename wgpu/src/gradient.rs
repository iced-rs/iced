use crate::Buffer;
use crate::core::gradient::ColorStop;
use crate::graphics::color;

/// A row must be a multiple of [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`].
const WIDTH: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

type Texel = [f32; 4];

/// Represents the index into a [`Storage`] and the length of the gradient.
pub type Stops = [u32; 2];

pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("iced_wgpu.gradient.bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
    })
}

/// Represents a texture holding a set of gradients.
#[derive(Debug)]
pub struct Storage {
    texels: Vec<Texel>,
    staging: Buffer<Texel>,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
    rows: u32,
    max_rows: u32,
}

impl Storage {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let rows = 1;
        let (texture, bind_group) = create_texture(device, layout, rows);

        Self {
            texels: Vec::new(),
            staging: Buffer::new(
                device,
                "iced_wgpu.gradient.staging_buffer",
                WIDTH as usize,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            ),
            texture,
            bind_group,
            layout: layout.clone(),
            rows,
            max_rows: device.limits().max_texture_dimension_2d,
        }
    }

    pub fn clear(&mut self) {
        self.texels.clear();
    }

    /// Adds the color stops of a gradient and returns its location.
    ///
    /// The stops must be sorted by offset, which is guaranteed by the
    /// gradient builders.
    pub fn push(&mut self, stops: &[Option<ColorStop>]) -> Stops {
        let base = self.texels.len();
        let stops = || stops.iter().flatten();

        for stop in stops() {
            let [r, g, b, a] = color::pack(stop.color).components();
            // premultiply the colours
            self.texels.push([r * a, g * a, b * a, a]);
        }

        let count = self.texels.len() - base;

        for (i, stop) in stops().enumerate() {
            if i % 4 == 0 {
                self.texels.push([0.0; 4]);
            }
            self.texels.last_mut().expect("offsets texel")[i % 4] = stop.offset;
        }

        if self.texels.len() > (self.max_rows * WIDTH) as usize {
            log::warn!(
                "Gradient storage is full ({} texels); some gradients will be transparent",
                self.max_rows * WIDTH
            );

            self.texels.truncate(base);

            return [0, 0];
        }

        [base as u32, count as u32]
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
    ) {
        if self.texels.is_empty() {
            return;
        }

        let rows = (self.texels.len() as u32).div_ceil(WIDTH);

        self.texels.resize((rows * WIDTH) as usize, [0.0; 4]);

        if rows > self.rows {
            self.rows = rows.next_power_of_two().min(self.max_rows);

            let (texture, bind_group) = create_texture(device, &self.layout, self.rows);

            self.texture = texture;
            self.bind_group = bind_group;
        }

        let _ = self.staging.resize(device, self.texels.len());
        let _ = self.staging.write(encoder, belt, 0, &self.texels);

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &self.staging.raw,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(WIDTH * std::mem::size_of::<Texel>() as u32),
                    rows_per_image: Some(rows),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: WIDTH,
                height: rows,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

fn create_texture(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    rows: u32,
) -> (wgpu::Texture, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("iced_wgpu.gradient.texture"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: rows,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("iced_wgpu.gradient.bind_group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&view),
        }],
    });

    (texture, bind_group)
}
