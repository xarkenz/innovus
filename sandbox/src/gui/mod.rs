use innovus::gfx::MeshRenderer;
use innovus::tools::Vector;
use crate::tools::asset::AssetPool;
use crate::world::item::Item;
use render::GuiVertex;
use render::cursor::GuiCursor;
use render::text::{TextLine, TextLineRenderer};
use crate::gui::render::text::TextBackground;
use crate::tools::input::InputState;

pub mod render;
pub mod hotbar;

pub struct GuiManager {
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    gui_scale: f32,
    offset_scale: Vector<f32, 2>,
    cursor_position: Vector<f32, 2>,
    cursor: GuiCursor,
    hotbar: hotbar::Hotbar,
    inventory: MeshRenderer<GuiVertex>,
    inventory_shown: bool,
    fps_display: TextLineRenderer,
    player_info_display: TextLineRenderer,
    input_test: TextLineRenderer,
}

impl GuiManager {
    pub fn new(viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>, gui_scale: f32, assets: &mut AssetPool) -> Result<Self, String> {
        Ok(Self {
            viewport_size,
            content_scale,
            gui_scale,
            offset_scale: Self::compute_offset_scale(viewport_size, content_scale.mul(gui_scale)),
            cursor_position: Vector::zero(),
            cursor: GuiCursor::new(Vector::zero(), Vector::zero(), &crate::world::item::types::AIR),
            hotbar: hotbar::Hotbar::new(assets)?,
            inventory: MeshRenderer::create(),
            inventory_shown: false,
            fps_display: TextLineRenderer::create(
                TextLine::new(
                    Vector([0.0, 0.0]),
                    Vector([1.0, 1.0, 1.0, 1.0]),
                    TextBackground::Rectangle {
                        color: Vector([0.0, 0.0, 0.0, 0.4]),
                        margin: Vector([1.0, 0.0]),
                    },
                    String::new(),
                ),
                Vector([0.0, 0.0]),
                Vector([0.0, 0.0]),
            ),
            player_info_display: TextLineRenderer::create(
                TextLine::new(
                    Vector([1.0, 0.0]),
                    Vector([1.0, 1.0, 1.0, 1.0]),
                    TextBackground::Rectangle {
                        color: Vector([0.0, 0.0, 0.0, 0.4]),
                        margin: Vector([1.0, 0.0]),
                    },
                    String::new(),
                ),
                Vector([1.0, 0.0]),
                Vector([0.0, 0.0]),
            ),
            input_test: TextLineRenderer::create(
                TextLine::new(
                    Vector([0.5, 0.5]),
                    Vector([1.0, 1.0, 1.0, 1.0]),
                    TextBackground::Rectangle {
                        color: Vector([0.0, 0.0, 0.0, 0.4]),
                        margin: Vector([1.0, 0.0]),
                    },
                    String::new(),
                ),
                Vector([0.5, 0.25]),
                Vector([0.0, 0.0]),
            ),
        })
    }

    pub fn viewport_size(&self) -> Vector<f32, 2> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector<f32, 2>) {
        self.viewport_size = viewport_size;
        self.offset_scale = Self::compute_offset_scale(viewport_size, self.content_scale.mul(self.gui_scale));
        self.compute_cursor_offset();
    }

    pub fn content_scale(&self) -> Vector<f32, 2> {
        self.content_scale
    }

    pub fn set_content_scale(&mut self, content_scale: Vector<f32, 2>) {
        self.content_scale = content_scale;
        self.offset_scale = Self::compute_offset_scale(self.viewport_size, content_scale.mul(self.gui_scale));
        self.compute_cursor_offset();
    }

    pub fn gui_scale(&self) -> f32 {
        self.gui_scale
    }

    pub fn set_gui_scale(&mut self, gui_scale: f32) {
        self.gui_scale = gui_scale;
        self.offset_scale = Self::compute_offset_scale(self.viewport_size, self.content_scale.mul(gui_scale));
        self.compute_cursor_offset();
    }

    fn compute_offset_scale(viewport_size: Vector<f32, 2>, scale: Vector<f32, 2>) -> Vector<f32, 2> {
        scale / viewport_size
    }

    pub fn cursor_position(&self) -> Vector<f32, 2> {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, position: Vector<f32, 2>) {
        self.cursor_position = position;
        self.compute_cursor_offset();
    }

    fn compute_cursor_offset(&mut self) {
        self.cursor.set_offset(self.cursor_position.mul(2.0)
            / self.content_scale.mul(self.gui_scale));
    }

    pub fn anchor_adjustment(&self, from_anchor: Vector<f32, 2>, to_anchor: Vector<f32, 2>) -> Vector<f32, 2> {
        (from_anchor - to_anchor).mul(2.0) / self.offset_scale
    }

    pub fn hotbar(&self) -> &hotbar::Hotbar {
        &self.hotbar
    }

    pub fn hotbar_mut(&mut self) -> &mut hotbar::Hotbar {
        &mut self.hotbar
    }

    pub fn inventory_shown(&self) -> bool {
        self.inventory_shown
    }

    pub fn set_inventory_shown(&mut self, shown: bool) {
        self.inventory_shown = shown;
    }

    pub fn reload_assets(&mut self, assets: &mut AssetPool) -> Result<(), String> {
        self.hotbar.reload_assets(assets)?;
        self.inventory.clear();
        Ok(())
    }

    pub fn update_fps_display(&mut self, average_fps: f32) {
        self.fps_display.data_mut().set_text(format!("Average FPS: {average_fps:.1}"));
    }

    pub fn update_player_info_display(&mut self, position: Vector<f32, 2>, velocity: Vector<f32, 2>) {
        self.player_info_display.data_mut().set_text(format!(
            "P=({:.0}, {:.0}); V=({:.1}, {:.1})",
            position.x().floor(),
            position.y().floor(),
            velocity.x(),
            velocity.y(),
        ));
    }

    pub fn update_item_display(&mut self, item: &Item, assets: &AssetPool) {
        self.cursor.set_item_type(item.item_type());
        if item.item_type().is_air() {
            self.hotbar.set_held_item_text(String::new());
        }
        else {
            let item_key = format!("item.{}", item.item_type());
            let item_name = assets.get_text(&item_key);
            self.hotbar.set_held_item_text(match item.count() {
                1 => item_name.to_string(),
                count => format!("{item_name} ({count})")
            });
        }
    }

    pub fn entered_text(&self) -> &str {
        self.input_test.data().text()
    }

    pub fn enter_text(&mut self, text: &str) {
        let mut string = self.input_test.data().text().to_string();
        string.push_str(text);
        self.input_test.data_mut().set_text(string);
    }

    pub fn backspace(&mut self) {
        let mut string = self.input_test.data().text().to_string();
        string.pop();
        self.input_test.data_mut().set_text(string);
    }

    pub fn clear_text(&mut self) {
        self.input_test.data_mut().set_text(String::new());
    }

    pub fn handle_input(&mut self, inputs: &InputState) -> bool {
        let cursor_offset = self.anchor_adjustment(self.cursor.anchor(), self.hotbar.anchor())
            + self.cursor.offset();
        if self.hotbar.handle_input(cursor_offset, inputs) {
            return true;
        }
        false
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        assets.gui_shaders().set_uniform("offset_scale", &self.offset_scale);
        assets.gui_shaders().set_uniform("tex_atlas", assets.gui_texture());

        if self.inventory.is_empty() {
            let atlas_region = assets.get_gui_image("gui/inventory").unwrap();
            let to_f32 = |x: u32| x as f32;
            self.inventory.add(
                &[
                    GuiVertex::new(Vector([-106.0, -62.0]), None, Some(atlas_region.min.map(to_f32))),
                    GuiVertex::new(Vector([-106.0, 62.0]), None, Some(atlas_region.min_x_max_y().map(to_f32))),
                    GuiVertex::new(Vector([106.0, 62.0]), None, Some(atlas_region.max.map(to_f32))),
                    GuiVertex::new(Vector([106.0, -62.0]), None, Some(atlas_region.max_x_min_y().map(to_f32))),
                ],
                &[
                    [0, 1, 2],
                    [2, 3, 0],
                ],
            );
        }
        if self.inventory_shown {
            assets.gui_texture().bind();
            assets.gui_shaders().set_uniform("anchor", &Vector([0.5f32, 0.5f32]));
            self.inventory.render();
            self.input_test.render(assets);
        }

        self.hotbar.render(assets);

        self.fps_display.render(assets);
        self.player_info_display.render(assets);
        self.cursor.render(assets);
    }
}
