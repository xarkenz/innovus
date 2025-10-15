use innovus::gfx::MeshRenderer;
use innovus::tools::{Rectangle, Vector};
use crate::gui::render::{GuiImage, GuiVertex};
use crate::gui::render::item::ItemGrid;
use crate::gui::render::text::{TextBackground, TextLine};
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use crate::world::item::{types, Item};

pub struct Hotbar {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    background_image: GuiImage,
    held_item_text: TextLine,
    item_grid: ItemGrid,
    background_layer: MeshRenderer<GuiVertex>,
    item_layer: MeshRenderer<GuiVertex>,
    foreground_layer: MeshRenderer<GuiVertex>,
}

impl Hotbar {
    const HELD_ITEM_TEXT_OFFSET: Vector<f32, 2> = Vector([106.0, 0.0]);
    const ITEM_GRID_OFFSET: Vector<f32, 2> = Vector([8.0, 8.0]);

    pub fn new(assets: &mut AssetPool) -> Result<Self, String> {
        Ok(Self {
            anchor: Vector([0.5, 1.0]),
            offset: Vector([-106.0, -32.0]),
            background_image: GuiImage::new(
                Rectangle::from_span(Vector::zero(), Vector([212.0, 32.0])),
                Vector::one(),
                assets.get_gui_image("gui/hotbar")?,
            ),
            held_item_text: TextLine::new(
                Vector([0.5, 1.0]),
                Vector([1.0, 1.0, 1.0, 1.0]),
                TextBackground::Rectangle {
                    color: Vector([0.0, 0.0, 0.0, 0.4]),
                    margin: Vector([1.0, 0.0]),
                },
                String::new(),
            ),
            item_grid: {
                let mut item_grid = ItemGrid::new(
                    10,
                    10,
                    Vector::filled(4.0)
                );
                item_grid.slot_mut(0).set_item(Item::new(&types::DIAMOND, 20));
                item_grid.slot_mut(1).set_item(Item::new(&types::DIAMOND_BLOCK, 7));
                item_grid.slot_mut(2).set_item(Item::new(&types::EMERALD, 5));
                item_grid.slot_mut(3).set_item(Item::new(&types::EMERALD_BLOCK, 2));
                item_grid.slot_mut(4).set_item(Item::new(&types::MAGMIUM_PICKAXE, 1));
                item_grid
            },
            background_layer: MeshRenderer::create(),
            item_layer: MeshRenderer::create(),
            foreground_layer: MeshRenderer::create(),
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
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.background_layer.clear();
        self.item_layer.clear();
        self.foreground_layer.clear();
    }

    pub fn reload_assets(&mut self, assets: &mut AssetPool) -> Result<(), String> {
        self.background_image.set_atlas_region(assets.get_gui_image("gui/hotbar")?);
        self.held_item_text.invalidate();
        self.item_grid.invalidate();
        self.invalidate();
        Ok(())
    }

    pub fn handle_input(&mut self, cursor_offset: Vector<f32, 2>, inputs: &InputState) -> bool {
        let cursor_offset = cursor_offset - self.offset;
        if self.background_image.bounds().contains_inclusive(cursor_offset) {
            self.item_grid.handle_input(cursor_offset - Self::ITEM_GRID_OFFSET, inputs);
            true
        }
        else {
            false
        }
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        if self.background_layer.is_empty() {
            self.item_layer.clear();
            self.foreground_layer.clear();

            self.background_image.append_to_mesh(
                self.background_layer.data_mut(),
                self.offset,
            );
            self.item_grid.append_to_mesh(
                self.item_layer.data_mut(),
                self.foreground_layer.data_mut(),
                self.offset + Self::ITEM_GRID_OFFSET,
                assets,
            );
            self.held_item_text.append_to_mesh(
                self.foreground_layer.data_mut(),
                self.offset + Self::HELD_ITEM_TEXT_OFFSET,
                assets,
            );

            self.background_layer.upload_buffers();
            self.item_layer.upload_buffers();
            self.foreground_layer.upload_buffers();
        }

        assets.gui_shaders().set_uniform("anchor", &self.anchor);
        assets.gui_texture().bind();
        self.background_layer.render();
        assets.item_texture().bind();
        self.item_layer.render();
        assets.gui_texture().bind();
        self.foreground_layer.render();
    }
}
