use crate::gfx::Gfx;
use crate::gfx::image::Image;
use crate::tools::Vector;

#[derive(Clone, Debug)]
pub struct Texture2D {
    handle: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture2D {
    pub fn create(
        device: &wgpu::Device,
        label: wgpu::Label,
        sampler: wgpu::Sampler,
        format: wgpu::TextureFormat,
        size: Vector<u32, 2>,
    ) -> Self {
        let handle = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.x(),
                height: size.y(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = handle.create_view(&Default::default());

        Self {
            handle,
            view,
            sampler,
        }
    }

    pub fn create_from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: wgpu::Label,
        sampler: wgpu::Sampler,
        image: &Image,
        spare_size: Vector<u32, 2>,
    ) -> Self {
        let texture = Self::create(
            device,
            label,
            sampler,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            image.size() + spare_size,
        );
        texture.write(queue, Vector::zero(), image);
        texture
    }

    pub fn handle(&self) -> &wgpu::Texture {
        &self.handle
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn set_sampler(&mut self, sampler: wgpu::Sampler) {
        self.sampler = sampler;
    }

    pub fn destroy(&self) {
        self.handle.destroy();
    }

    pub fn size(&self) -> Vector<u32, 2> {
        let wgpu::Extent3d { width, height, .. } = self.handle.size();
        Vector([width, height])
    }

    pub fn write(&self, queue: &wgpu::Queue, offset: Vector<u32, 2>, image: &Image) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.handle,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: offset.x(),
                    y: offset.y(),
                    z: 0,
                },
            },
            image.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: Some(image.height()),
            },
            wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
        );
    }
}

pub struct DynamicTexture2D {
    device: wgpu::Device,
    queue: wgpu::Queue,
    label: Option<Box<str>>,
    sampler: wgpu::Sampler,
    inner: Option<Texture2D>,
    size: Vector<u32, 2>,
}

impl DynamicTexture2D {
    pub fn create(
        gfx: &Gfx,
        label: wgpu::Label,
        sampler: wgpu::Sampler,
        image: &Image,
    ) -> Self {
        let inner = (!image.is_empty()).then(|| {
            let size = image.size();
            Texture2D::create_from_image(
                gfx.device(),
                gfx.queue(),
                label,
                sampler.clone(),
                image,
                size.map(Self::capacity_for_size) - size,
            )
        });

        Self {
            device: gfx.device().clone(),
            queue: gfx.queue().clone(),
            label: label.map(Into::into),
            sampler,
            inner,
            size: image.size(),
        }
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn set_sampler(&mut self, sampler: wgpu::Sampler) {
        if let Some(inner) = &mut self.inner {
            inner.set_sampler(sampler.clone());
        }
        self.sampler = sampler;
    }

    pub fn inner(&self) -> Option<&Texture2D> {
        self.inner.as_ref()
    }

    pub fn size(&self) -> Vector<u32, 2> {
        self.size
    }

    pub fn capacity(&self) -> Vector<u32, 2> {
        self.inner.as_ref().map_or(Vector::zero(), Texture2D::size)
    }

    pub fn write(&self, offset: Vector<u32, 2>, image: &Image) {
        self.inner
            .as_ref()
            // Reasonable to panic if no buffer since wgpu panics for out-of-bounds writes anyway.
            .expect("cannot write to empty texture")
            .write(&self.queue, offset, image);
    }

    pub fn clear(&mut self) {
        if let Some(texture) = self.inner.take() {
            // If someone else is still using this texture, that's their problem.
            texture.handle().destroy();
        }
        self.size = Vector::zero();
    }

    pub fn resize(&mut self, new_size: Vector<u32, 2>) {
        if new_size.x() == 0 || new_size.y() == 0 {
            self.clear();
            return;
        }
        else if new_size == self.size {
            return;
        }

        let new_capacity = new_size.map(Self::capacity_for_size);

        if new_capacity != self.capacity() {
            let new_texture = Texture2D::create(
                &self.device,
                self.label.as_deref(),
                self.sampler.clone(),
                wgpu::TextureFormat::Rgba8UnormSrgb,
                new_capacity,
            );

            if let Some(old_texture) = self.inner.take() {
                // Copy the old texture contents to the new texture.
                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Texture Copy Command Encoder"),
                });
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        aspect: wgpu::TextureAspect::All,
                        texture: old_texture.handle(),
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    wgpu::TexelCopyTextureInfo {
                        aspect: wgpu::TextureAspect::All,
                        texture: new_texture.handle(),
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    wgpu::Extent3d {
                        width: std::cmp::min(old_texture.handle().width(), new_texture.handle().width()),
                        height: std::cmp::min(old_texture.handle().height(), new_texture.handle().height()),
                        depth_or_array_layers: 1,
                    },
                );
                self.queue.submit(std::iter::once(encoder.finish()));
                // Nothing else should be using the old texture, so we can just destroy it once
                // the GPU finishes copying from it.
                old_texture.destroy();
            }

            self.inner = Some(new_texture);
        }

        self.size = new_size;
    }

    pub fn rewrite(&mut self, new_image: &Image) {
        if new_image.is_empty() {
            self.clear();
            return;
        }

        let new_size = new_image.size();
        let new_capacity = new_size.map(Self::capacity_for_size);

        if self.inner.is_some() && new_capacity == self.capacity() {
            // Reuse the existing texture since the new size fits within capacity.
            self.inner.as_ref().unwrap().write(&self.queue, Vector::zero(), new_image);
        }
        else {
            // The new image doesn't fit in the existing capacity, so reallocate the texture
            if let Some(old_texture) = self.inner.take() {
                old_texture.destroy();
            }

            self.inner = Some(Texture2D::create_from_image(
                &self.device,
                &self.queue,
                self.label.as_deref(),
                self.sampler.clone(),
                new_image,
                new_capacity - new_size,
            ));
        }

        self.size = new_size;
    }

    fn capacity_for_size(size: u32) -> u32 {
        // Grow and shrink exponentially so the texture doesn't have to be reallocated as often if
        // the size doesn't change much. Also, avoid reallocating often for small sizes.
        size.next_power_of_two().max(256)
    }
}

impl Drop for DynamicTexture2D {
    fn drop(&mut self) {
        if let Some(texture) = self.inner.take() {
            // If someone else is still using this texture, that's their problem.
            texture.destroy();
        }
    }
}
