use std::marker::PhantomData;
use std::ops::RangeBounds;
use bytemuck::{Pod, Zeroable};
use crate::gfx::Gfx;

pub struct ArrayBufferDescriptor<'a, T> {
    label: wgpu::Label<'a>,
    usage: wgpu::BufferUsages,
    elements: &'a [T],
    spare_len: wgpu::BufferAddress,
    mapped_at_creation: bool,
}

impl<'a, T> Default for ArrayBufferDescriptor<'a, T> {
    fn default() -> Self {
        Self {
            label: Default::default(),
            usage: wgpu::BufferUsages::empty(),
            elements: &[],
            spare_len: 0,
            mapped_at_creation: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArrayBuffer<T: Zeroable + Pod> {
    handle: wgpu::Buffer,
    len: wgpu::BufferAddress,
    _element_type: PhantomData<T>,
}

impl<T: Zeroable + Pod> ArrayBuffer<T> {
    pub fn create(device: &wgpu::Device, desc: ArrayBufferDescriptor<T>) -> Self {
        // This logic is loosely based on `wgpu::util::DeviceExt::create_buffer_init`.
        if desc.elements.is_empty() {
            Self {
                handle: device.create_buffer(&wgpu::BufferDescriptor {
                    label: desc.label,
                    size: desc.spare_len * size_of::<T>() as wgpu::BufferAddress,
                    usage: desc.usage,
                    mapped_at_creation: desc.mapped_at_creation,
                }),
                len: desc.spare_len,
                _element_type: PhantomData,
            }
        }
        else {
            let len = desc.elements.len() as wgpu::BufferAddress + desc.spare_len;
            let unpadded_size = len * size_of::<T>() as wgpu::BufferAddress;
            // From `wgpu::util::DeviceExt::create_buffer_init`:
            //     Valid vulkan usage is
            //     1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
            //     2. buffer size must be greater than 0.
            //     Therefore we round the value up to the nearest multiple, and ensure it's
            //     at least COPY_BUFFER_ALIGNMENT.
            let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
            let padded_size = ((unpadded_size + align_mask) & !align_mask)
                .max(wgpu::COPY_BUFFER_ALIGNMENT);

            let handle = device.create_buffer(&wgpu::BufferDescriptor {
                label: desc.label,
                size: padded_size,
                usage: desc.usage,
                mapped_at_creation: true,
            });

            handle
                .get_mapped_range_mut(..)
                .slice(..(desc.elements.len() * size_of::<T>()))
                .copy_from_slice(bytemuck::cast_slice(desc.elements));

            if !desc.mapped_at_creation {
                handle.unmap();
            }

            Self {
                handle,
                len,
                _element_type: PhantomData,
            }
        }
    }

    pub fn handle(&self) -> &wgpu::Buffer {
        &self.handle
    }

    pub fn destroy(&self) {
        self.handle.destroy();
    }

    pub fn usage(&self) -> wgpu::BufferUsages {
        self.handle.usage()
    }

    pub fn len(&self) -> wgpu::BufferAddress {
        self.len
    }

    pub fn slice<B>(&self, bounds: B) -> wgpu::BufferSlice<'_>
    where
        B: RangeBounds<wgpu::BufferAddress>,
    {
        let index_to_offset = |&index| index * size_of::<T>() as wgpu::BufferAddress;
        let start_bound = bounds.start_bound().map(index_to_offset);
        let end_bound = bounds.end_bound().map(index_to_offset);
        self.handle.slice((start_bound, end_bound))
    }

    pub fn write(&self, queue: &wgpu::Queue, start_index: wgpu::BufferAddress, elements: &[T]) {
        queue.write_buffer(
            &self.handle,
            start_index * size_of::<T>() as wgpu::BufferAddress,
            bytemuck::cast_slice(elements),
        );
    }
}

// This might be stupid, idk. Not sure how else to do it without changing approach to meshes.
#[derive(Debug)]
pub struct DynamicArrayBuffer<T: Zeroable + Pod> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    label: Option<Box<str>>,
    usage: wgpu::BufferUsages,
    inner: Option<ArrayBuffer<T>>,
    len: wgpu::BufferAddress,
}

impl<T: Zeroable + Pod> DynamicArrayBuffer<T> {
    pub fn create(
        gfx: &Gfx,
        label: wgpu::Label,
        usage: wgpu::BufferUsages,
        elements: &[T],
    ) -> Self {
        let usage = usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        let len = elements.len() as wgpu::BufferAddress;
        let inner = (!elements.is_empty())
            .then(|| ArrayBuffer::create(gfx.device(), ArrayBufferDescriptor {
                label,
                usage,
                elements,
                spare_len: Self::capacity_for_len(len) - len,
                ..Default::default()
            }));

        Self {
            device: gfx.device().clone(),
            queue: gfx.queue().clone(),
            label: label.map(Into::into),
            usage,
            inner,
            len,
        }
    }

    pub fn inner(&self) -> Option<&ArrayBuffer<T>> {
        self.inner.as_ref()
    }

    pub fn usage(&self) -> wgpu::BufferUsages {
        self.usage
    }

    pub fn len(&self) -> wgpu::BufferAddress {
        self.len
    }

    pub fn capacity(&self) -> wgpu::BufferAddress {
        self.inner.as_ref().map_or(0, ArrayBuffer::len)
    }

    pub fn slice<B>(&self, bounds: B) -> wgpu::BufferSlice<'_>
    where
        B: RangeBounds<wgpu::BufferAddress>,
    {
        self.inner
            .as_ref()
            // Reasonable to panic if no buffer since wgpu panics for slicing with size < 1 anyway.
            .expect("cannot slice empty buffer")
            .slice(bounds)
    }

    pub fn write(&self, start_index: wgpu::BufferAddress, elements: &[T]) {
        if elements.is_empty() {
            return;
        }
        self.inner
            .as_ref()
            // Reasonable to panic if no buffer since wgpu panics for out-of-bounds writes anyway.
            .expect("cannot write to empty buffer")
            .write(&self.queue, start_index, elements);
    }

    pub fn clear(&mut self) {
        if let Some(buffer) = self.inner.take() {
            // If someone else is still using this buffer, that's their problem.
            buffer.destroy();
        }
        self.len = 0;
    }

    pub fn resize(&mut self, new_len: wgpu::BufferAddress) {
        if new_len == 0 {
            self.clear();
            return;
        }
        else if new_len == self.len {
            return;
        }

        let new_capacity = Self::capacity_for_len(new_len);

        if new_capacity != self.capacity() {
            let new_buffer = ArrayBuffer::create(&self.device, ArrayBufferDescriptor {
                label: self.label.as_deref(),
                usage: self.usage,
                spare_len: new_capacity,
                ..Default::default()
            });

            if let Some(old_buffer) = self.inner.take() {
                // Copy the old buffer contents to the new buffer.
                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Buffer Copy Command Encoder"),
                });
                encoder.copy_buffer_to_buffer(
                    old_buffer.handle(),
                    0,
                    new_buffer.handle(),
                    0,
                    std::cmp::min(old_buffer.handle().size(), new_buffer.handle().size()),
                );
                self.queue.submit(std::iter::once(encoder.finish()));
                // Nothing else should be using the old buffer, so we can just destroy it once
                // the GPU finishes copying from it.
                old_buffer.destroy();
            }

            self.inner = Some(new_buffer);
        }

        self.len = new_len;
    }

    pub fn rewrite(&mut self, new_elements: &[T]) {
        if new_elements.is_empty() {
            self.clear();
            return;
        }

        let new_len = new_elements.len() as wgpu::BufferAddress;
        let new_capacity = Self::capacity_for_len(new_len);

        if self.inner.is_some() && new_capacity == self.capacity() {
            // Reuse the existing buffer since the new length fits within capacity.
            self.inner.as_ref().unwrap().write(&self.queue, 0, new_elements);
        }
        else {
            // The new data doesn't fit in the existing capacity, so reallocate the buffer
            if let Some(old_buffer) = self.inner.take() {
                old_buffer.destroy();
            }

            self.inner = Some(ArrayBuffer::create(&self.device, ArrayBufferDescriptor {
                label: self.label.as_deref(),
                usage: self.usage,
                elements: new_elements,
                spare_len: new_capacity - new_len,
                ..Default::default()
            }));
        }

        self.len = new_len;
    }

    fn capacity_for_len(len: wgpu::BufferAddress) -> wgpu::BufferAddress {
        // Grow and shrink exponentially so the buffer doesn't have to be reallocated as often if
        // the length doesn't change much. Also, avoid reallocating often for small lengths.
        len.next_power_of_two().max(16)
    }
}

impl<T: Zeroable + Pod> Drop for DynamicArrayBuffer<T> {
    fn drop(&mut self) {
        if let Some(buffer) = self.inner.take() {
            // If someone else is still using this buffer, that's their problem.
            buffer.destroy();
        }
    }
}
