use std::mem::offset_of;
use innovus::gfx::{Mesh, MeshRenderer, Vertex, VertexAttribute, VertexAttributeType};
use innovus::gfx::color::AlphaColor;
use innovus::tools::{Rectangle, Vector};

pub mod cursor;
pub mod item;
pub mod text;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct GuiVertex {
    pub offset: Vector<f32, 2>,
    pub color: Vector<f32, 4>,
    pub uv: Vector<f32, 2>,
}

impl GuiVertex {
    pub fn new(
        offset: Vector<f32, 2>,
        color: Option<Vector<f32, 4>>,
        uv: Option<Vector<f32, 2>>,
    ) -> Self {
        Self {
            offset,
            color: color.unwrap_or(Vector::one()),
            uv: uv.unwrap_or(Vector::splat(f32::NAN)),
        }
    }
}

impl Vertex for GuiVertex {
    const ATTRIBUTES: &'static [VertexAttribute] = &[
        VertexAttribute::new(VertexAttributeType::F32, 2, offset_of!(Self, offset)),
        VertexAttribute::new(VertexAttributeType::F32, 4, offset_of!(Self, color)),
        VertexAttribute::new(VertexAttributeType::F32, 2, offset_of!(Self, uv)),
    ];
}

pub type GuiLayerMesh = MeshRenderer<GuiVertex>;

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

    pub fn append_to_mesh(&self, mesh: &mut Mesh<GuiVertex>, offset: Vector<f32, 2>) {
        let rgba_color = self.color.rgba();
        let to_f32 = |x: u32| x as f32;
        mesh.add(
            &[
                GuiVertex::new(
                    offset + self.bounds.min,
                    Some(rgba_color),
                    Some(self.atlas_region.min.map(to_f32)),
                ),
                GuiVertex::new(
                    offset + self.bounds.min_x_max_y(),
                    Some(rgba_color),
                    Some(self.atlas_region.min_x_max_y().map(to_f32)),
                ),
                GuiVertex::new(
                    offset + self.bounds.max,
                    Some(rgba_color),
                    Some(self.atlas_region.max.map(to_f32)),
                ),
                GuiVertex::new(
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
