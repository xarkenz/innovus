use innovus::gfx::color::Color;
use innovus::gfx::Gfx;
use innovus::gfx::pipeline::BindGroup;
use innovus::tools::{Rectangle, Vector};
use crate::gui::render::cursor::GuiCursor;
use crate::gui::render::text::{TextLine, TextLineRenderer};
use crate::gui::render::{GuiImage, GuiLayerMesh, GuiParams};
use crate::gui::render::text::TextBackground;
use crate::script::ScriptingEngine;
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use crate::world::World;
use crate::world::item::Item;

pub mod render;
pub mod hotbar;
pub mod chat;

pub struct GuiManager {
    params: GuiParams,
    cursor_position: Vector<f32, 2>,
    cursor: GuiCursor,
    hotbar: hotbar::Hotbar,
    chat_box: chat::ChatBox,
    inventory: GuiImage,
    inventory_layer: GuiLayerMesh,
    inventory_shown: bool,
    fps_display: TextLineRenderer,
    player_info_display: TextLineRenderer,
}

impl GuiManager {
    pub fn create(gfx: &Gfx, viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>, gui_scale: f32, assets: &mut AssetPool) -> Result<Self, String> {
        Ok(Self {
            params: GuiParams::new(
                gfx,
                &GuiParams::create_layout(gfx.device()),
                viewport_size,
                content_scale,
                gui_scale,
            ),
            cursor_position: Vector::zero(),
            cursor: GuiCursor::create(gfx, Vector::zero(), Vector::zero(), &crate::world::item::types::AIR),
            hotbar: hotbar::Hotbar::create(gfx, assets)?,
            chat_box: chat::ChatBox::create(gfx, 20, 12.0, 0.4)?,
            inventory: GuiImage::new(
                Rectangle::new(Vector([-106.0, -62.0]), Vector([106.0, 62.0])),
                Color::White.into(),
                assets.get_gui_image("gui/inventory")?,
            ),
            inventory_layer: GuiLayerMesh::create(gfx),
            inventory_shown: false,
            fps_display: TextLineRenderer::create(
                gfx,
                TextLine::new(
                    Vector([0.0, 0.0]),
                    Color::White.into(),
                    TextBackground::Rectangle {
                        color: Color::Black.with_alpha(0.4),
                        margin: Vector([1.0, 0.0]),
                    },
                    String::new(),
                ),
                Vector([0.0, 0.0]),
                Vector([1.0, 0.0]),
            ),
            player_info_display: TextLineRenderer::create(
                gfx,
                TextLine::new(
                    Vector([1.0, 0.0]),
                    Color::White.into(),
                    TextBackground::Rectangle {
                        color: Color::Black.with_alpha(0.4),
                        margin: Vector([1.0, 0.0]),
                    },
                    String::new(),
                ),
                Vector([1.0, 0.0]),
                Vector([-1.0, 0.0]),
            ),
        })
    }

    pub fn viewport_size(&self) -> Vector<f32, 2> {
        self.params.viewport_size()
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector<f32, 2>) {
        self.params.set_viewport_size(viewport_size);
        self.compute_cursor_offset();
    }

    pub fn content_scale(&self) -> Vector<f32, 2> {
        self.params.content_scale()
    }

    pub fn set_content_scale(&mut self, content_scale: Vector<f32, 2>) {
        self.params.set_content_scale(content_scale);
        self.compute_cursor_offset();
    }

    pub fn gui_scale(&self) -> f32 {
        self.params.gui_scale()
    }

    pub fn set_gui_scale(&mut self, gui_scale: f32) {
        self.params.set_gui_scale(gui_scale);
        self.compute_cursor_offset();
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
            / self.content_scale().mul(self.gui_scale()));
    }

    pub fn anchor_adjustment(&self, from_anchor: Vector<f32, 2>, to_anchor: Vector<f32, 2>) -> Vector<f32, 2> {
        self.params.anchor_adjustment(from_anchor, to_anchor)
    }

    pub fn hotbar(&self) -> &hotbar::Hotbar {
        &self.hotbar
    }

    pub fn hotbar_mut(&mut self) -> &mut hotbar::Hotbar {
        &mut self.hotbar
    }

    pub fn chat_box(&self) -> &chat::ChatBox {
        &self.chat_box
    }

    pub fn chat_box_mut(&mut self) -> &mut chat::ChatBox {
        &mut self.chat_box
    }

    pub fn reload_assets(&mut self, assets: &mut AssetPool) -> Result<(), String> {
        self.hotbar.reload_assets(assets)?;
        self.chat_box.reload_assets();
        self.inventory.set_atlas_region(assets.get_gui_image("gui/inventory")?);
        self.inventory_layer.clear();
        Ok(())
    }

    pub fn update_fps_display(&mut self, min_fps: f32) {
        self.fps_display.data_mut().set_text(format!("Min recent FPS: {min_fps:.1}"));
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

    pub fn handle_cursor(&mut self, inputs: &InputState, scripting: &ScriptingEngine, world: &mut World, assets: &AssetPool) -> bool {
        let _ = (scripting, world, assets);
        let cursor_offset = self.anchor_adjustment(self.cursor.anchor(), self.hotbar.anchor())
            + self.cursor.offset();

        self.hotbar.handle_cursor(cursor_offset, inputs)
    }

    pub fn handle_keyboard(&mut self, inputs: &InputState, scripting: &ScriptingEngine, world: &mut World, assets: &AssetPool) -> bool {
        // TODO: probably need some kind of "focus" system... idk how exactly that should work
        self.chat_box.handle_keyboard(inputs, scripting, world, assets) ||
            {
                if inputs.key_was_pressed(Key::E) {
                    self.inventory_shown = !self.inventory_shown;
                    true
                }
                else {
                    false
                }
            }
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, assets: &mut AssetPool) {
        render_pass.set_pipeline(assets.gui_pipeline());
        render_pass.set_bind_group(0, assets.gui_texture().bind_group(), &[]);
        render_pass.set_bind_group(2, self.params.bind_group(), &[]);

        if self.inventory_shown {
            if self.inventory_layer.is_empty() {
                self.inventory.append_to_mesh(self.inventory_layer.mesh_mut(), Vector([0.5, 0.5]), Vector::zero());
                self.inventory_layer.upload_buffers();
            }
            self.inventory_layer.render(render_pass);
        }

        self.hotbar.render(render_pass, assets);
        self.chat_box.render(render_pass, assets);

        self.fps_display.render(render_pass, assets);
        self.player_info_display.render(render_pass, assets);
        self.cursor.render(render_pass, assets);
    }
}
