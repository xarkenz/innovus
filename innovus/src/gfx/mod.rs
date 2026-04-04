use std::error::Error;
use pollster::FutureExt;
use crate::gfx::color::Color;
use crate::tools::Vector;

pub mod buffer;
pub mod color;
pub mod texture;
pub mod image;
pub mod mesh;
pub mod pipeline;

pub struct Gfx<'window> {
    surface: wgpu::Surface<'window>,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    device: wgpu::Device,
    queue: wgpu::Queue,
    clear_color: Color,
}

impl<'window> Gfx<'window> {
    pub fn create(
        target: impl Into<wgpu::SurfaceTarget<'window>>,
        surface_size: Vector<u32, 2>,
        clear_color: Color,
    ) -> Result<Self, CreateGfxError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        let surface = instance.create_surface(target)
            .map_err(CreateGfxError::CreateSurface)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .map_err(CreateGfxError::RequestAdapter)?;

        let (device, queue) = adapter
            .request_device(&Default::default())
            .block_on()
            .map_err(CreateGfxError::RequestDevice)?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_size.x(),
            height: surface_size.y(),
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        let is_surface_configured = config.width > 0 && config.height > 0 && {
            surface.configure(&device, &config);
            true
        };

        Ok(Self {
            surface,
            config,
            is_surface_configured,
            device,
            queue,
            clear_color,
        })
    }

    pub fn surface(&self) -> &wgpu::Surface<'window> {
        &self.surface
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn resize(&mut self, surface_size: Vector<u32, 2>) {
        if surface_size.x() > 0 && surface_size.y() > 0 {
            self.config.width = surface_size.x();
            self.config.height = surface_size.y();
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render<F, T>(&self, renderer: F) -> Option<T>
    where
        F: FnOnce(&mut wgpu::RenderPass) -> T,
    {
        if !self.is_surface_configured {
            return None;
        }

        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            // TODO: how to handle this case properly?
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            _ => return None
        };
        let texture_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Command Encoder"),
        });

        let result = renderer(
            &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            }),
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Some(result)
    }
}

#[derive(Debug)]
pub enum CreateGfxError {
    CreateSurface(wgpu::CreateSurfaceError),
    RequestAdapter(wgpu::RequestAdapterError),
    RequestDevice(wgpu::RequestDeviceError),
}

impl std::fmt::Display for CreateGfxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateSurface(error) => error.fmt(f),
            Self::RequestAdapter(error) => error.fmt(f),
            Self::RequestDevice(error) => error.fmt(f),
        }
    }
}

impl Error for CreateGfxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CreateSurface(error) => Some(error),
            Self::RequestAdapter(error) => Some(error),
            Self::RequestDevice(error) => Some(error),
        }
    }
}
