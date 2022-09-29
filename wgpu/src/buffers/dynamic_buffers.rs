//! Utilities for uniform buffer operations.
use encase::private::WriteInto;
use encase::ShaderType;
use std::marker::PhantomData;

// Currently supported dynamic buffers.
enum DynamicBufferType {
    Uniform(encase::DynamicUniformBuffer<Vec<u8>>),
    Storage(encase::DynamicStorageBuffer<Vec<u8>>),
}

impl DynamicBufferType {
    /// Writes the current value to its CPU buffer with proper alignment.
    pub(super) fn write<T: ShaderType + WriteInto>(
        &mut self,
        value: &T,
    ) -> wgpu::DynamicOffset {
        match self {
            DynamicBufferType::Uniform(buf) => buf
                .write(value)
                .expect("Error when writing to dynamic uniform buffer.")
                as u32,
            DynamicBufferType::Storage(buf) => buf
                .write(value)
                .expect("Error when writing to dynamic storage buffer.")
                as u32,
        }
    }

    /// Returns bytearray of aligned CPU buffer.
    pub(super) fn get_ref(&self) -> &Vec<u8> {
        match self {
            DynamicBufferType::Uniform(buf) => buf.as_ref(),
            DynamicBufferType::Storage(buf) => buf.as_ref(),
        }
    }

    /// Resets the CPU buffer.
    pub(super) fn clear(&mut self) {
        match self {
            DynamicBufferType::Uniform(buf) => {
                buf.as_mut().clear();
                buf.set_offset(0);
            }
            DynamicBufferType::Storage(buf) => {
                buf.as_mut().clear();
                buf.set_offset(0);
            }
        }
    }
}

//TODO think about making cpu & gpu buffers optional
pub(crate) struct DynamicBuffer<T: ShaderType> {
    offsets: Vec<wgpu::DynamicOffset>,
    cpu: DynamicBufferType,
    gpu: wgpu::Buffer,
    label: &'static str,
    size: u64,
    _data: PhantomData<T>,
}

impl<T: ShaderType + WriteInto> DynamicBuffer<T> {
    /// Creates a new dynamic uniform buffer.
    pub fn uniform(device: &wgpu::Device, label: &'static str) -> Self {
        DynamicBuffer::new(
            device,
            DynamicBufferType::Uniform(encase::DynamicUniformBuffer::new(
                Vec::new(),
            )),
            label,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        )
    }

    /// Creates a new dynamic storage buffer.
    pub fn storage(device: &wgpu::Device, label: &'static str) -> Self {
        DynamicBuffer::new(
            device,
            DynamicBufferType::Storage(encase::DynamicStorageBuffer::new(
                Vec::new(),
            )),
            label,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        )
    }

    fn new(
        device: &wgpu::Device,
        dynamic_buffer_type: DynamicBufferType,
        label: &'static str,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let initial_size = u64::from(T::min_size());

        Self {
            offsets: Vec::new(),
            cpu: dynamic_buffer_type,
            gpu: DynamicBuffer::<T>::create_gpu_buffer(
                device,
                label,
                usage,
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
        //this write operation on the buffer will adjust for uniform alignment requirements
        let offset = self.cpu.write(value);
        self.offsets.push(offset as u32);
    }

    /// Resize buffer contents if necessary. This will re-create the GPU buffer if current size is
    /// less than the newly computed size from the CPU buffer.
    pub fn resize(&mut self, device: &wgpu::Device) -> bool {
        let new_size = self.cpu.get_ref().len() as u64;

        if self.size < new_size {
            let usages = match self.cpu {
                DynamicBufferType::Uniform(_) => {
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
                }
                DynamicBufferType::Storage(_) => {
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
                }
            };

            //Re-create the GPU buffer since it needs to be resized.
            self.gpu = DynamicBuffer::<T>::create_gpu_buffer(
                device, self.label, usages, new_size,
            );
            self.size = new_size;
            true
        } else {
            false
        }
    }

    /// Write the contents of this dynamic buffer to the GPU via staging belt command.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let size = self.cpu.get_ref().len();

        if let Some(buffer_size) = wgpu::BufferSize::new(size as u64) {
            let mut buffer = staging_belt.write_buffer(
                encoder,
                &self.gpu,
                0,
                buffer_size,
                device,
            );

            buffer.copy_from_slice(self.cpu.get_ref());
        }
    }

    // Gets the aligned offset at the given index from the CPU buffer.
    pub fn offset_at_index(&self, index: usize) -> wgpu::DynamicOffset {
        let offset = self
            .offsets
            .get(index)
            .expect(&format!("Index {} not found in offsets.", index))
            .clone();

        offset
    }

    /// Returns a reference to the GPU buffer.
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.gpu
    }

    /// Reset the buffer.
    pub fn clear(&mut self) {
        self.offsets.clear();
        self.cpu.clear();
    }
}
