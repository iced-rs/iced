use bytemuck::{Pod, Zeroable};
use std::marker::PhantomData;
use std::mem;

//128 triangles/indices
const DEFAULT_STATIC_BUFFER_COUNT: wgpu::BufferAddress = 1_000;

/// A generic buffer struct useful for items which have no alignment requirements
/// (e.g. Vertex, Index buffers) & no dynamic offsets.
#[derive(Debug)]
pub struct Buffer<T> {
    //stored sequentially per mesh iteration; refers to the offset index in the GPU buffer
    offsets: Vec<wgpu::BufferAddress>,
    label: &'static str,
    usages: wgpu::BufferUsages,
    gpu: wgpu::Buffer,
    size: wgpu::BufferAddress,
    _data: PhantomData<T>,
}

impl<T: Pod + Zeroable> Buffer<T> {
    /// Initialize a new static buffer.
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        usages: wgpu::BufferUsages,
    ) -> Self {
        let size = (mem::size_of::<T>() as u64) * DEFAULT_STATIC_BUFFER_COUNT;

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

    /// Returns whether or not the buffer needs to be recreated. This can happen whenever mesh data
    /// changes & a redraw is requested.
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
        queue: &wgpu::Queue,
        offset: u64,
        content: &[T],
    ) -> u64 {
        let bytes = bytemuck::cast_slice(content);
        let bytes_size = bytes.len() as u64;

        queue.write_buffer(&self.gpu, offset, bytes);
        self.offsets.push(offset);

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
}
