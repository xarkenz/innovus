use innovus::gfx::MeshRenderer;
use innovus::tools::{Rectangle, Vector};
use crate::gui::render::{GuiImage, GuiVertex};
use crate::gui::render::text::TextLine;
use crate::tools::asset::AssetPool;

pub struct Hotbar {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    background_image: GuiImage,
    held_item_text: TextLine,
    background_layer: MeshRenderer<GuiVertex>,
    item_layer: MeshRenderer<GuiVertex>,
    foreground_layer: MeshRenderer<GuiVertex>,
}

impl Hotbar {
    pub fn new(assets: &mut AssetPool) -> Result<Self, String> {
        Ok(Self {
            anchor: Vector([0.0, -1.0]),
            offset: Vector([-106.0, 0.0]),
            background_image: GuiImage::new(
                Rectangle::from_size(Vector::zero(), Vector([212.0, 32.0])),
                Vector::one(),
                assets.get_gui_image("gui/hotbar")?,
            ),
            held_item_text: TextLine::new(
                Vector([0.5, 0.0]),
                Vector([1.0, 1.0, 1.0, 1.0]),
                Vector([0.0, 0.0, 0.0, 0.4]),
                String::new(),
            ),
            background_layer: MeshRenderer::create()?,
            item_layer: MeshRenderer::create()?,
            foreground_layer: MeshRenderer::create()?,
        })
    }

    pub fn anchor(&self) -> Vector<f32, 2> {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector<f32, 2>) {
        self.anchor = anchor;
    }

    pub fn offset(&self) -> Vector<f32, 2> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vector<f32, 2>) {
        self.offset = offset;
        self.invalidate();
    }

    pub fn set_held_item_text(&mut self, text: String) {
        self.held_item_text.set_text(text);
        self.background_layer.clear();
    }

    pub fn invalidate(&mut self) {
        self.background_layer.clear();
        self.item_layer.clear();
        self.foreground_layer.clear();
    }

    pub fn reload_assets(&mut self, assets: &mut AssetPool) -> Result<(), String> {
        self.background_image.set_atlas_region(assets.get_gui_image("gui/hotbar")?);
        self.held_item_text.invalidate();
        self.invalidate();
        Ok(())
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        if self.background_layer.is_empty() {
            self.background_image.append_to_mesh(
                self.background_layer.data_mut(),
                self.offset,
            );
            self.held_item_text.append_to_mesh(
                self.background_layer.data_mut(),
                self.offset + Vector([106.0, 32.0]),
                assets,
            );
            self.background_layer.upload_buffers();
        }
        assets.gui_texture().bind();
        assets.gui_shaders().set_uniform("anchor", self.anchor);
        self.background_layer.render();
    }
}
