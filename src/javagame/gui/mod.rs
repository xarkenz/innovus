use std::mem::offset_of;
use innovus::gfx::{Geometry, Vertex, VertexAttribute, VertexAttributeType};
use innovus::tools::Vector;
use crate::gui::text::StringRenderer;
use crate::tools::asset::AssetPool;

pub mod text;

#[repr(C)]
#[derive(Clone, Debug)]
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
            uv: uv.unwrap_or(Vector::filled(f32::NAN)),
        }
    }
}

impl Vertex for GuiVertex {
    const ATTRIBUTES: &'static [VertexAttribute] = &[
        VertexAttribute::new(VertexAttributeType::F32, 2, offset_of!(Self, anchor)),
        VertexAttribute::new(VertexAttributeType::F32, 2, offset_of!(Self, offset)),
        VertexAttribute::new(VertexAttributeType::F32, 4, offset_of!(Self, color)),
        VertexAttribute::new(VertexAttributeType::F32, 2, offset_of!(Self, uv)),
    ];
}

pub struct GuiManager {
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    gui_scale: f32,
    offset_scale: Vector<f32, 2>,
    hotbar: Geometry<GuiVertex>,
    pub fps_display: StringRenderer,
}

impl GuiManager {
    pub fn new(viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>, gui_scale: f32) -> Self {
        Self {
            viewport_size,
            content_scale,
            gui_scale,
            offset_scale: Self::compute_offset_scale(viewport_size, content_scale * gui_scale),
            hotbar: Geometry::new_render().unwrap(),
            fps_display: StringRenderer::new(
                Vector([-1.0, 1.0]),
                Vector([0.0, -4.0]),
                Vector([0.0, 1.0]),
                Vector([1.0, 1.0, 1.0, 1.0]),
                Vector([0.0, 0.0, 0.0, 0.5]),
                "Gathering data...".into(),
            ),
        }
    }

    pub fn viewport_size(&self) -> Vector<f32, 2> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector<f32, 2>) {
        self.viewport_size = viewport_size;
        self.offset_scale = Self::compute_offset_scale(viewport_size, self.content_scale * self.gui_scale);
    }

    pub fn content_scale(&self) -> Vector<f32, 2> {
        self.content_scale
    }

    pub fn set_content_scale(&mut self, content_scale: Vector<f32, 2>) {
        self.content_scale = content_scale;
        self.offset_scale = Self::compute_offset_scale(self.viewport_size, content_scale * self.gui_scale);
    }

    pub fn gui_scale(&self) -> f32 {
        self.gui_scale
    }

    pub fn set_gui_scale(&mut self, gui_scale: f32) {
        self.gui_scale = gui_scale;
        self.offset_scale = Self::compute_offset_scale(self.viewport_size, self.content_scale * gui_scale);
    }

    fn compute_offset_scale(viewport_size: Vector<f32, 2>, scale: Vector<f32, 2>) -> Vector<f32, 2> {
        scale / viewport_size
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        assets.gui_texture().bind();
        assets.gui_shaders().set_uniform("offset_scale", self.offset_scale);
        assets.gui_shaders().set_uniform("tex_atlas", assets.gui_texture());

        if self.hotbar.is_empty() {
            let atlas_region = assets.get_gui_image("gui/hotbar").unwrap();
            let anchor = Vector([0.0, -1.0]);
            let to_f32 = |x: u32| x as f32;
            self.hotbar.add(
                &[
                    GuiVertex::new(anchor, Vector([-106.0, 0.0]), None, Some(atlas_region.min_x_max_y().map(to_f32))),
                    GuiVertex::new(anchor, Vector([-106.0, 32.0]), None, Some(atlas_region.min().map(to_f32))),
                    GuiVertex::new(anchor, Vector([106.0, 32.0]), None, Some(atlas_region.max_x_min_y().map(to_f32))),
                    GuiVertex::new(anchor, Vector([106.0, 0.0]), None, Some(atlas_region.max().map(to_f32))),
                ],
                &[
                    [0, 1, 2],
                    [2, 3, 0],
                ],
            );
        }
        self.hotbar.render();

        self.fps_display.render(assets);
    }
}
