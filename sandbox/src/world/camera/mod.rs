use innovus::gfx::buffer::{ArrayBuffer, ArrayBufferDescriptor};
use innovus::gfx::Gfx;
use innovus::gfx::pipeline::BindGroup;
use innovus::tools::*;

pub struct Camera {
    view: Transform3D<f32>,
    projection: Transform3D<f32>,
    view_projection: Transform3D<f32>,
    position: Vector<f32, 2>,
    target: Vector<f32, 2>,
    size: Vector<f32, 2>,
    zoom: Vector<f32, 2>,
    speed: f32,
    queue: wgpu::Queue,
    buffer: ArrayBuffer<Transform3D<f32>>,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(gfx: &Gfx, layout: &wgpu::BindGroupLayout, position: Vector<f32, 2>, size: Vector<f32, 2>, zoom: Vector<f32, 2>, speed: f32) -> Self {
        let buffer = ArrayBuffer::create(gfx.device(), ArrayBufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            spare_len: 1,
            ..Default::default()
        });

        let bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            view: Transform3D::identity(),
            projection: Transform3D::identity(),
            view_projection: Transform3D::identity(),
            position,
            target: position,
            size,
            zoom,
            speed,
            queue: gfx.queue().clone(),
            buffer,
            bind_group,
        }
    }

    pub fn view(&self) -> &Transform3D<f32> {
        &self.view
    }

    pub fn projection(&self) -> &Transform3D<f32> {
        &self.projection
    }

    pub fn view_projection(&self) -> &Transform3D<f32> {
        &self.view_projection
    }

    pub fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector<f32, 2>) {
        self.position = position;
    }

    pub fn snap_to_target(&mut self) {
        self.position = self.target;
    }

    pub fn target(&self) -> Vector<f32, 2> {
        self.target
    }

    pub fn set_target(&mut self, target: Vector<f32, 2>) {
        self.target = target;
    }

    pub fn size(&self) -> Vector<f32, 2> {
        self.size
    }

    pub fn set_size(&mut self, size: Vector<f32, 2>) {
        self.size = size;
    }

    pub fn zoom(&self) -> Vector<f32, 2> {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: Vector<f32, 2>) {
        self.zoom = zoom;
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn get_world_pos(&self, screen_pos: Vector<f32, 2>) -> Vector<f32, 2> {
        (screen_pos - self.size.mul(0.5)) * Vector([1.0, -1.0]) / self.zoom + self.position
    }

    pub fn update(&mut self, dt: f32) {
        self.view.set_identity();
        self.view.look_at(
            Vector([self.position.x(), self.position.y(), 1.0]),
            Vector([self.position.x(), self.position.y(), 0.0]),
            Vector([0.0, 1.0, 0.0]),
        );

        self.projection.set_identity();
        let pixel_size = self.zoom.map(|x| 1.0 / x);
        self.projection.orthographic(
            self.size.x() * -pixel_size.x() / 2.0,
            self.size.x() * pixel_size.x() / 2.0,
            self.size.y() * -pixel_size.y() / 2.0,
            self.size.y() * pixel_size.y() / 2.0,
            100.0,
            -100.0,
        );

        self.view_projection = self.projection * self.view;

        self.position = self.position.lerp(self.target, (self.speed * dt).min(1.0));

        self.buffer.write(&self.queue, 0, &[self.view_projection]);
    }
}

impl BindGroup for Camera {
    const ENTRIES: &'static [wgpu::BindGroupLayoutEntry] = &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ];

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
