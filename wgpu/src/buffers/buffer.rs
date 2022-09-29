//! Utilities for static buffer operations.

/// A generic buffer struct useful for items which have no alignment requirements
/// (e.g. Vertex, Index buffers) and are set once and never changed until destroyed.
///
/// This buffer is mapped to the GPU on creation, so must be initialized with the correct capacity.
#[derive(Debug)]
pub(crate) struct StaticBuffer {
    //stored sequentially per mesh iteration
    offsets: Vec<wgpu::BufferAddress>,
    gpu: wgpu::Buffer,
    //the static size of the buffer
    size: wgpu::BufferAddress,
}

impl StaticBuffer {
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        size: u64,
        usage: wgpu::BufferUsages,
        total_offsets: usize,
    ) -> Self {
        Self {
            offsets: Vec::with_capacity(total_offsets),
            gpu: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: true,
            }),
            size,
        }
    }

    /// Resolves pending write operations & unmaps buffer from host memory.
    pub fn flush(&self) {
        (&self.gpu).unmap();
    }

    /// Returns whether or not the buffer needs to be recreated. This can happen whenever the mesh 
    /// data is re-submitted.
    pub fn needs_recreate(&self, new_size: usize) -> bool {
        self.size != new_size as u64
    }

    /// Writes the current vertex data to the gpu buffer with a memcpy & stores its offset.
    pub fn write(&mut self, offset: u64, content: &[u8]) {
        //offset has to be divisible by 8 for alignment reasons
        let actual_offset = if offset % 8 != 0 {
            offset + 4
        } else {
            offset
        };

        let mut buffer = self
            .gpu
            .slice(actual_offset..(actual_offset + content.len() as u64))
            .get_mapped_range_mut();
        buffer.copy_from_slice(content);
        self.offsets.push(actual_offset);
    }

    fn offset_at(&self, index: usize) -> &wgpu::BufferAddress {
        self.offsets
            .get(index)
            .expect(&format!("Offset index {} is not in range.", index))
    }

    /// Returns the slice calculated from the offset stored at the given index.
    /// e.g. to calculate the slice for the 2nd mesh in the layer, this would be the offset at index 
    /// 1 that we stored earlier when writing.
    pub fn slice_from_index<T>(
        &self,
        index: usize,
    ) -> wgpu::BufferSlice<'_> {
        self.gpu.slice(self.offset_at(index)..)
    }
}

/// Returns true if the current buffer doesn't exist & needs to be created, or if it's too small 
/// for the new content.
pub(crate) fn needs_recreate(
    buffer: &Option<StaticBuffer>,
    new_size: usize,
) -> bool {
    match buffer {
        None => true,
        Some(buf) => buf.needs_recreate(new_size),
    }
}
