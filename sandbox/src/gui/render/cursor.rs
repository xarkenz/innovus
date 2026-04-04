use innovus::gfx::color::Color;
use innovus::gfx::Gfx;
use innovus::gfx::pipeline::BindGroup;
use innovus::tools::{Rectangle, Vector};
use crate::gui::render::{GuiImage, GuiLayerMesh};
use crate::tools::asset::AssetPool;
use crate::world::item::ItemType;

pub struct GuiCursor {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    item_type: &'static ItemType,
    layer: GuiLayerMesh,
}

impl GuiCursor {
    pub fn create(
        gfx: &Gfx,
        anchor: Vector<f32, 2>,
        offset: Vector<f32, 2>,
        item_type: &'static ItemType,
    ) -> Self {
        Self {
            anchor,
            offset,
            item_type,
            layer: GuiLayerMesh::create(gfx),
        }
    }

    pub fn anchor(&self) -> Vector<f32, 2> {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector<f32, 2>) {
        self.anchor = anchor
    }

    pub fn offset(&self) -> Vector<f32, 2> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vector<f32, 2>) {
        self.offset = offset;
    }

    pub fn item_type(&self) -> &'static ItemType {
        self.item_type
    }

    pub fn set_item_type(&mut self, item_type: &'static ItemType) {
        self.item_type = item_type;
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, assets: &mut AssetPool) {
        if let Some(item_atlas_region) = assets.get_item_image(self.item_type) {
            let item_image = GuiImage::new(
                Rectangle::from_span(Vector([4.0, 4.0]), Vector([16.0, 16.0])),
                Color::White.into(),
                item_atlas_region,
            );
            self.layer.clear();
            item_image.append_to_mesh(self.layer.mesh_mut(), self.anchor, self.offset);
            self.layer.upload_buffers();

            render_pass.set_bind_group(0, assets.item_texture().bind_group(), &[]);
            self.layer.render(render_pass);
        }

        let cursor_image = GuiImage::new(
            Rectangle::from_span(Vector::zero(), Vector([16.0, 16.0])),
            Color::White.into(),
            assets.get_gui_image("gui/cursor").unwrap(),
        );
        self.layer.clear();
        cursor_image.append_to_mesh(self.layer.mesh_mut(), self.anchor, self.offset);
        self.layer.upload_buffers();

        render_pass.set_bind_group(0, assets.gui_texture().bind_group(), &[]);
        self.layer.render(render_pass);
    }
}
