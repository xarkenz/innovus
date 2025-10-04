use innovus::gfx::MeshRenderer;
use innovus::tools::{Rectangle, Vector};
use crate::gui::render::{GuiImage, GuiVertex};
use crate::tools::asset::AssetPool;

pub struct Hotbar {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    main_image: GuiImage,
    gui_mesh: MeshRenderer<GuiVertex>,
    item_mesh: MeshRenderer<GuiVertex>,
}

impl Hotbar {
    pub fn new(assets: &mut AssetPool) -> Result<Self, String> {
        Ok(Self {
            anchor: Vector([0.0, -1.0]),
            offset: Vector([-106.0, 0.0]),
            main_image: GuiImage::new(
                Rectangle::from_size(Vector::zero(), Vector([212.0, 32.0])),
                Vector::one(),
                assets.get_gui_image("gui/hotbar")?,
            ),
            gui_mesh: MeshRenderer::create()?,
            item_mesh: MeshRenderer::create()?,
        })
    }

    pub fn anchor(&self) -> Vector<f32, 2> {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector<f32, 2>) {
        self.anchor = anchor;
        self.invalidate();
    }

    pub fn offset(&self) -> Vector<f32, 2> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vector<f32, 2>) {
        self.offset = offset;
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.gui_mesh.clear();
        self.item_mesh.clear();
    }

    pub fn reload_assets(&mut self, assets: &mut AssetPool) -> Result<(), String> {
        self.main_image.set_atlas_region(assets.get_gui_image("gui/hotbar")?);
        self.invalidate();
        Ok(())
    }

    pub fn render(&mut self, assets: &AssetPool) {
        if self.gui_mesh.is_empty() {
            self.gui_mesh.add_mesh(&self.main_image.generate_mesh(self.anchor, self.offset));
        }
        assets.gui_texture().bind();
        self.gui_mesh.render();
    }
}
