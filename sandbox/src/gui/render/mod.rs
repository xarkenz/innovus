use innovus::gfx::buffer::{ArrayBuffer, ArrayBufferDescriptor};
use innovus::gfx::color::AlphaColor;
use innovus::gfx::Gfx;
use innovus::gfx::mesh::{Mesh, MeshRenderer};
use innovus::gfx::pipeline::BindGroup;
use innovus::tools::{Rectangle, Vector};

pub mod cursor;
pub mod item;
pub mod text;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GuiVertex {
    pub anchor: Vector<f32, 2>,
    pub offset: Vector<f32, 2>,
    pub color: Vector<f32, 4>,
    pub uv: Vector<f32, 2>,
}

impl GuiVertex {
    pub fn new(
        anchor: Vector<f32, 2>,
        offset: Vector<f32, 2>,
        color: Option<Vector<f32, 4>>,
        uv: Option<Vector<f32, 2>>,
    ) -> Self {
        Self {
            anchor,
            offset,
            color: color.unwrap_or(Vector::one()),
            uv: uv.unwrap_or(Vector::splat(f32::NAN)),
        }
    }
}

innovus::impl_vertex! {
    for GuiVertex,
    anchor: Float32x2 @ 0,
    offset: Float32x2 @ 1,
    color: Float32x4 @ 2,
    uv: Float32x2 @ 3,
}

pub type GuiLayerMesh = MeshRenderer<GuiVertex>;

pub struct GuiParams {
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    gui_scale: f32,
    offset_scale: Vector<f32, 2>,
    queue: wgpu::Queue,
    buffer: ArrayBuffer<Vector<f32, 2>>,
    bind_group: wgpu::BindGroup,
}

impl GuiParams {
    pub fn new(gfx: &Gfx, layout: &wgpu::BindGroupLayout, viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>, gui_scale: f32) -> Self {
        let offset_scale = Self::compute_offset_scale(viewport_size, content_scale, gui_scale);

        let buffer = ArrayBuffer::create(gfx.device(), ArrayBufferDescriptor {
            label: Some("GuiParams Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            elements: &[offset_scale],
            ..Default::default()
        });

        let bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GuiParams Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            viewport_size,
            content_scale,
            gui_scale,
            offset_scale,
            queue: gfx.queue().clone(),
            buffer,
            bind_group,
        }
    }

    pub fn viewport_size(&self) -> Vector<f32, 2> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector<f32, 2>) {
        self.viewport_size = viewport_size;
        self.update_offset_scale();
    }

    pub fn content_scale(&self) -> Vector<f32, 2> {
        self.content_scale
    }

    pub fn set_content_scale(&mut self, content_scale: Vector<f32, 2>) {
        self.content_scale = content_scale;
        self.update_offset_scale();
    }

    pub fn gui_scale(&self) -> f32 {
        self.gui_scale
    }

    pub fn set_gui_scale(&mut self, gui_scale: f32) {
        self.gui_scale = gui_scale;
        self.update_offset_scale();
    }

    fn update_offset_scale(&mut self) {
        self.offset_scale = Self::compute_offset_scale(self.viewport_size, self.content_scale, self.gui_scale);

        self.buffer.write(&self.queue, 0, &[self.offset_scale]);
    }

    fn compute_offset_scale(viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>, gui_scale: f32) -> Vector<f32, 2> {
        content_scale.mul(gui_scale) / viewport_size
    }

    pub fn anchor_adjustment(&self, from_anchor: Vector<f32, 2>, to_anchor: Vector<f32, 2>) -> Vector<f32, 2> {
        (from_anchor - to_anchor).mul(2.0) / self.offset_scale
    }
}

impl BindGroup for GuiParams {
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

#[derive(Debug)]
pub struct GuiImage {
    bounds: Rectangle<f32>,
    color: AlphaColor,
    atlas_region: Rectangle<u32>,
}

impl GuiImage {
    pub fn new(bounds: Rectangle<f32>, color: AlphaColor, atlas_region: Rectangle<u32>) -> Self {
        Self {
            bounds,
            color,
            atlas_region,
        }
    }

    pub fn bounds(&self) -> Rectangle<f32> {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: Rectangle<f32>) {
        self.bounds = bounds;
    }

    pub fn color(&self) -> AlphaColor {
        self.color
    }

    pub fn set_color(&mut self, color: AlphaColor) {
        self.color = color;
    }

    pub fn atlas_region(&self) -> Rectangle<u32> {
        self.atlas_region
    }

    pub fn set_atlas_region(&mut self, region: Rectangle<u32>) {
        self.atlas_region = region;
    }

    pub fn append_to_mesh(&self, mesh: &mut Mesh<GuiVertex>, anchor: Vector<f32, 2>, offset: Vector<f32, 2>) {
        let rgba_color = self.color.rgba();
        let to_f32 = |x: u32| x as f32;
        mesh.add(
            &[
                GuiVertex::new(
                    anchor,
                    offset + self.bounds.min,
                    Some(rgba_color),
                    Some(self.atlas_region.min.map(to_f32)),
                ),
                GuiVertex::new(
                    anchor,
                    offset + self.bounds.min_x_max_y(),
                    Some(rgba_color),
                    Some(self.atlas_region.min_x_max_y().map(to_f32)),
                ),
                GuiVertex::new(
                    anchor,
                    offset + self.bounds.max,
                    Some(rgba_color),
                    Some(self.atlas_region.max.map(to_f32)),
                ),
                GuiVertex::new(
                    anchor,
                    offset + self.bounds.max_x_min_y(),
                    Some(rgba_color),
                    Some(self.atlas_region.max_x_min_y().map(to_f32)),
                ),
            ],
            &[
                [0, 1, 2],
                [2, 3, 0],
            ],
        );
    }
}
