use innovus::gfx::MeshRenderer;
use innovus::tools::Vector;
use crate::gui::render::GuiVertex;
use crate::tools::asset::AssetPool;
use crate::world::item::ItemType;

pub struct GuiCursor {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    item_type: &'static ItemType,
    mesh: MeshRenderer<GuiVertex>,
}

impl GuiCursor {
    pub fn new(anchor: Vector<f32, 2>, offset: Vector<f32, 2>, item_type: &'static ItemType) -> Self {
        Self {
            anchor,
            offset,
            item_type,
            mesh: MeshRenderer::create(),
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

    pub fn render(&mut self, assets: &mut AssetPool) {
        assets.gui_shaders().set_uniform("anchor", &self.anchor);
        let cursor_atlas_region = assets.get_gui_image("gui/cursor").unwrap();
        let to_f32 = |x: u32| x as f32;

        if let Some(item_atlas_region) = assets.get_item_image(self.item_type) {
            self.mesh.clear();
            self.mesh.add(
                &[
                    GuiVertex::new(
                        self.offset + Vector([4.0, 4.0]),
                        None,
                        Some(item_atlas_region.min.map(to_f32)),
                    ),
                    GuiVertex::new(
                        self.offset + Vector([4.0, 20.0]),
                        None,
                        Some(item_atlas_region.min_x_max_y().map(to_f32)),
                    ),
                    GuiVertex::new(
                        self.offset + Vector([20.0, 20.0]),
                        None,
                        Some(item_atlas_region.max.map(to_f32)),
                    ),
                    GuiVertex::new(
                        self.offset + Vector([20.0, 4.0]),
                        None,
                        Some(item_atlas_region.max_x_min_y().map(to_f32)),
                    ),
                ],
                &[
                    [0, 1, 2],
                    [2, 3, 0],
                ],
            );

            assets.item_texture().bind();
            self.mesh.render();
        }

        self.mesh.clear();
        self.mesh.add(
            &[
                GuiVertex::new(
                    self.offset + Vector([0.0, 0.0]),
                    None,
                    Some(cursor_atlas_region.min.map(to_f32)),
                ),
                GuiVertex::new(
                    self.offset + Vector([0.0, 16.0]),
                    None,
                    Some(cursor_atlas_region.min_x_max_y().map(to_f32)),
                ),
                GuiVertex::new(
                    self.offset + Vector([16.0, 16.0]),
                    None,
                    Some(cursor_atlas_region.max.map(to_f32)),
                ),
                GuiVertex::new(
                    self.offset + Vector([16.0, 0.0]),
                    None,
                    Some(cursor_atlas_region.max_x_min_y().map(to_f32)),
                ),
            ],
            &[
                [0, 1, 2],
                [2, 3, 0],
            ],
        );

        assets.gui_texture().bind();
        self.mesh.render();
    }
}
