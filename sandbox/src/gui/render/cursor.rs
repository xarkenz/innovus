use innovus::gfx::MeshRenderer;
use innovus::tools::Vector;
use crate::gui::render::GuiVertex;
use crate::tools::asset::AssetPool;
use crate::world::item::ItemType;

pub struct CursorRenderer {
    offset: Vector<f32, 2>,
    item_type: &'static ItemType,
    geometry: MeshRenderer<GuiVertex>,
}

impl CursorRenderer {
    pub fn new(offset: Vector<f32, 2>, item_type: &'static ItemType) -> Self {
        Self {
            offset,
            item_type,
            geometry: MeshRenderer::create().unwrap(),
        }
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
        let cursor_atlas_region = assets.get_gui_image("gui/cursor").unwrap();
        let to_f32 = |x: u32| x as f32;

        self.geometry.clear();
        self.geometry.add(
            &[
                GuiVertex::new(
                    Vector([-1.0, 1.0]),
                    self.offset + Vector([0.0, -16.0]),
                    None,
                    Some(cursor_atlas_region.min_x_max_y().map(to_f32)),
                ),
                GuiVertex::new(
                    Vector([-1.0, 1.0]),
                    self.offset + Vector([0.0, 0.0]),
                    None,
                    Some(cursor_atlas_region.min().map(to_f32)),
                ),
                GuiVertex::new(
                    Vector([-1.0, 1.0]),
                    self.offset + Vector([16.0, 0.0]),
                    None,
                    Some(cursor_atlas_region.max_x_min_y().map(to_f32)),
                ),
                GuiVertex::new(
                    Vector([-1.0, 1.0]),
                    self.offset + Vector([16.0, -16.0]),
                    None,
                    Some(cursor_atlas_region.max().map(to_f32)),
                ),
            ],
            &[
                [0, 1, 2],
                [2, 3, 0],
            ],
        );

        assets.gui_texture().bind();
        self.geometry.render();

        if let Some(item_atlas_region) = assets.get_item_image(self.item_type) {
            self.geometry.clear();
            self.geometry.add(
                &[
                    GuiVertex::new(
                        Vector([-1.0, 1.0]),
                        self.offset + Vector([4.0, -16.0]),
                        None,
                        Some(item_atlas_region.min_x_max_y().map(to_f32)),
                    ),
                    GuiVertex::new(
                        Vector([-1.0, 1.0]),
                        self.offset + Vector([4.0, -4.0]),
                        None,
                        Some(item_atlas_region.min().map(to_f32)),
                    ),
                    GuiVertex::new(
                        Vector([-1.0, 1.0]),
                        self.offset + Vector([16.0, -4.0]),
                        None,
                        Some(item_atlas_region.max_x_min_y().map(to_f32)),
                    ),
                    GuiVertex::new(
                        Vector([-1.0, 1.0]),
                        self.offset + Vector([16.0, -16.0]),
                        None,
                        Some(item_atlas_region.max().map(to_f32)),
                    ),
                ],
                &[
                    [0, 1, 2],
                    [2, 3, 0],
                ],
            );

            assets.item_texture().bind();
            self.geometry.render();
        }
    }
}
