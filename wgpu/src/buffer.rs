use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::ops::RangeBounds;

pub const MAX_WRITE_SIZE: usize = 100 * 1024;

const MAX_WRITE_SIZE_U64: NonZeroU64 = NonZeroU64::new(MAX_WRITE_SIZE as u64)
    .expect("MAX_WRITE_SIZE must be non-zero");

#[derive(Debug)]
pub struct Buffer<T> {
    label: &'static str,
    size: u64,
    usage: wgpu::BufferUsages,
    pub(crate) raw: wgpu::Buffer,
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
        let new_size = next_copy_size::<T>(new_count);

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

    /// Returns the size of the written bytes.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        offset: usize,
        contents: &[T],
    ) -> usize {
        let bytes: &[u8] = bytemuck::cast_slice(contents);
        let mut bytes_written = 0;

        // Split write into multiple chunks if necessary
        while bytes_written + MAX_WRITE_SIZE < bytes.len() {
            belt.write_buffer(
                encoder,
                &self.raw,
                (offset + bytes_written) as u64,
                MAX_WRITE_SIZE_U64,
                device,
            )
            .copy_from_slice(
                &bytes[bytes_written..bytes_written + MAX_WRITE_SIZE],
            );

            bytes_written += MAX_WRITE_SIZE;
        }

        // There will always be some bytes left, since the previous
        // loop guarantees `bytes_written < bytes.len()`
        let bytes_left = ((bytes.len() - bytes_written) as u64)
            .try_into()
            .expect("non-empty write");

        // Write them
        belt.write_buffer(
            encoder,
            &self.raw,
            (offset + bytes_written) as u64,
            bytes_left,
            device,
        )
        .copy_from_slice(&bytes[bytes_written..]);

        bytes.len()
    }

    pub fn slice(
        &self,
        bounds: impl RangeBounds<wgpu::BufferAddress>,
    ) -> wgpu::BufferSlice<'_> {
        self.raw.slice(bounds)
    }

    pub fn range(&self, start: usize, end: usize) -> wgpu::BufferSlice<'_> {
        self.slice(
            start as u64 * std::mem::size_of::<T>() as u64
                ..end as u64 * std::mem::size_of::<T>() as u64,
        )
    }
}

fn next_copy_size<T>(amount: usize) -> u64 {
    let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;

    (((std::mem::size_of::<T>() * amount).next_power_of_two() as u64
        + align_mask)
        & !align_mask)
        .max(wgpu::COPY_BUFFER_ALIGNMENT)
}
