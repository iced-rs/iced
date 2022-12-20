use std::marker::PhantomData;
use std::ops::RangeBounds;

#[derive(Debug)]
pub struct Buffer<T> {
    label: &'static str,
    size: u64,
    usage: wgpu::BufferUsages,
    raw: wgpu::Buffer,
    type_: PhantomData<T>,
}

impl<T: bytemuck::Pod> Buffer<T> {
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        amount: usize,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let size = next_copy_size::<T>(amount);

        let raw = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self {
            label,
            size,
            usage,
            raw,
            type_: PhantomData,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, new_count: usize) -> bool {
        let new_size = (std::mem::size_of::<T>() * new_count) as u64;

        if self.size < new_size {
            self.raw = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: new_size,
                usage: self.usage,
                mapped_at_creation: false,
            });

            self.size = new_size;

            true
        } else {
            false
        }
    }

    pub fn write(
        &self,
        queue: &wgpu::Queue,
        offset_count: usize,
        contents: &[T],
    ) {
        queue.write_buffer(
            &self.raw,
            (std::mem::size_of::<T>() * offset_count) as u64,
            bytemuck::cast_slice(contents),
        );
    }

    pub fn slice(
        &self,
        bounds: impl RangeBounds<wgpu::BufferAddress>,
    ) -> wgpu::BufferSlice<'_> {
        self.raw.slice(bounds)
    }
}

fn next_copy_size<T>(amount: usize) -> u64 {
    let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;

    (((std::mem::size_of::<T>() * amount).next_power_of_two() as u64
        + align_mask)
        & !align_mask)
        .max(wgpu::COPY_BUFFER_ALIGNMENT)
}

//TODO(shan)
/// A generic buffer struct useful for items which have no alignment requirements
/// (e.g. Vertex, Index buffers) & no dynamic offsets.
#[derive(Debug)]
pub struct Static<T> {
    //stored sequentially per mesh iteration; refers to the offset index in the GPU buffer
    offsets: Vec<wgpu::BufferAddress>,
    label: &'static str,
    usages: wgpu::BufferUsages,
    gpu: wgpu::Buffer,
    size: wgpu::BufferAddress,
    _data: PhantomData<T>,
}

impl<T: Pod + Zeroable> Static<T> {
    /// Initialize a new static buffer.
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        usages: wgpu::BufferUsages,
        count: usize,
    ) -> Self {
        let size = (mem::size_of::<T>() * count) as u64;

        Self {
            offsets: Vec::new(),
            label,
            usages,
            gpu: Self::gpu_buffer(device, label, size, usages),
            size,
            _data: PhantomData,
        }
    }

    fn gpu_buffer(
        device: &wgpu::Device,
        label: &'static str,
        size: wgpu::BufferAddress,
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    /// Returns whether or not the buffer needs to be recreated.
    pub fn resize(&mut self, device: &wgpu::Device, new_count: usize) -> bool {
        let size = (mem::size_of::<T>() * new_count) as u64;

        if self.size < size {
            self.offsets.clear();
            self.size = size;
            self.gpu = Self::gpu_buffer(device, self.label, size, self.usages);
            true
        } else {
            false
        }
    }

    /// Writes the current vertex data to the gpu buffer with a memcpy & stores its offset.
    ///
    /// Returns the size of the written bytes.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        offset: u64,
        content: &[T],
    ) -> u64 {
        let bytes = bytemuck::cast_slice(content);
        let bytes_size = bytes.len() as u64;

        if let Some(buffer_size) = wgpu::BufferSize::new(bytes_size) {
            let mut buffer = staging_belt.write_buffer(
                encoder,
                &self.gpu,
                offset,
                buffer_size,
                device,
            );

            buffer.copy_from_slice(bytes);

            self.offsets.push(offset);
        }

        bytes_size
    }

    fn offset_at(&self, index: usize) -> &wgpu::BufferAddress {
        self.offsets
            .get(index)
            .expect("Offset at index does not exist.")
    }

    /// Returns the slice calculated from the offset stored at the given index.
    /// e.g. to calculate the slice for the 2nd mesh in the layer, this would be the offset at index
    /// 1 that we stored earlier when writing.
    pub fn slice_from_index(&self, index: usize) -> wgpu::BufferSlice<'_> {
        self.gpu.slice(self.offset_at(index)..)
    }

    /// Clears any temporary data from the buffer.
    pub fn clear(&mut self) {
        self.offsets.clear()
    }

    /// Returns a reference to the GPU buffer.
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.gpu
    }
}

/// A dynamic uniform buffer is any type of buffer which does not have a static offset.
pub struct DynamicUniform<T: ShaderType> {
    offsets: Vec<wgpu::DynamicOffset>,
    cpu: encase::DynamicUniformBuffer<Vec<u8>>,
    gpu: wgpu::Buffer,
    label: &'static str,
    size: u64,
    _data: PhantomData<T>,
}

impl<T: ShaderType> Debug for DynamicUniform<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}, {:?}",
            self.offsets, self.gpu, self.label, self.size
        )
    }
}

impl<T: ShaderType + WriteInto> DynamicUniform<T> {
    pub fn new(device: &wgpu::Device, label: &'static str) -> Self {
        let initial_size = u64::from(T::min_size());

        Self {
            offsets: Vec::new(),
            cpu: encase::DynamicUniformBuffer::new(Vec::new()),
            gpu: DynamicUniform::<T>::create_gpu_buffer(
                device,
                label,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                initial_size,
            ),
            label,
            size: initial_size,
            _data: Default::default(),
        }
    }

    fn create_gpu_buffer(
        device: &wgpu::Device,
        label: &'static str,
        usage: wgpu::BufferUsages,
        size: u64,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    /// Write a new value to the CPU buffer with proper alignment. Stores the returned offset value
    /// in the buffer for future use.
    pub fn push(&mut self, value: &T) {
        //this write operation on the cpu buffer will adjust for uniform alignment requirements
        let offset = self
            .cpu
            .write(value)
            .expect("Error when writing to dynamic uniform buffer.")
            as wgpu::DynamicOffset;

        self.offsets.push(offset);
    }

    /// Resize buffer contents if necessary. This will re-create the GPU buffer if current size is
    /// less than the newly computed size from the CPU buffer.
    ///
    /// If the gpu buffer is resized, its bind group will need to be recreated!
    pub fn resize(&mut self, device: &wgpu::Device) -> bool {
        let new_size = self.cpu.as_ref().len() as u64;

        if self.size < new_size {
            self.gpu = DynamicUniform::<T>::create_gpu_buffer(
                device,
                self.label,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                new_size,
            );
            self.size = new_size;
            true
        } else {
            false
        }
    }

    /// Write the contents of this dynamic uniform buffer to the GPU via staging belt command.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let size = self.cpu.as_ref().len();

        if let Some(buffer_size) = wgpu::BufferSize::new(size as u64) {
            let mut buffer = staging_belt.write_buffer(
                encoder,
                &self.gpu,
                0,
                buffer_size,
                device,
            );

            buffer.copy_from_slice(self.cpu.as_ref());
        }
    }

    // Gets the aligned offset at the given index from the CPU buffer.
    pub fn offset_at_index(&self, index: usize) -> wgpu::DynamicOffset {
        let offset = self
            .offsets
            .get(index)
            .copied()
            .expect("Index not found in offsets.");

        offset
    }

    /// Returns a reference to the GPU buffer.
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.gpu
    }

    /// Reset the buffer.
    pub fn clear(&mut self) {
        self.offsets.clear();
        self.cpu.as_mut().clear();
        self.cpu.set_offset(0);
    }
}
