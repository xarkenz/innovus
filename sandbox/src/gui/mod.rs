use std::collections::VecDeque;
use glfw::Key;
use innovus::gfx::color::Color;
use innovus::tools::{Rectangle, Vector};
use crate::tools::asset::AssetPool;
use crate::world::item::Item;
use render::cursor::GuiCursor;
use render::text::{TextLine, TextLineRenderer};
use render::{GuiImage, GuiLayerMesh};
use render::text::TextBackground;
use crate::tools::asset::text::{TextAsset, TextElement};
use crate::tools::input::InputState;

pub mod render;
pub mod hotbar;
pub mod chat;

pub struct GuiManager {
    viewport_size: Vector<f32, 2>,
    content_scale: Vector<f32, 2>,
    gui_scale: f32,
    offset_scale: Vector<f32, 2>,
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

pub enum GuiRequest<'a> {
    HandleMouse(&'a InputState),
    HandleKeyboard(&'a InputState),
    ReloadAssets(&'a mut AssetPool),
    HeldItem(Item, &'a AssetPool),
    ChatMessage {
        content: TextElement,
        color: Color,
        assets: &'a AssetPool,
    },
    DebugFPS(f32),
    DebugPlayerInfo {
        position: Vector<f32, 2>,
        velocity: Vector<f32, 2>,
    },
}

pub enum GuiResponse {
    WorldHandleMouse,
    WorldHandleKeyboard,
    PlayerChat {
        content: String,
    },
    PlaySound {
        path: String,
    },
    AssetError {
        message: String,
    },
}

pub type GuiResponseQueue = VecDeque<GuiResponse>;

impl GuiManager {
    pub fn new(viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>, gui_scale: f32, assets: &mut AssetPool) -> Result<Self, String> {
        Ok(Self {
            viewport_size,
            content_scale,
            gui_scale,
            offset_scale: Self::compute_offset_scale(viewport_size, content_scale.mul(gui_scale)),
            cursor_position: Vector::zero(),
            cursor: GuiCursor::new(Vector::zero(), Vector::zero(), Default::default()),
            hotbar: hotbar::Hotbar::new(assets)?,
            chat_box: chat::ChatBox::new(20, 12.0, 0.4)?,
            inventory: GuiImage::new(
                Rectangle::new(Vector([-106.0, -62.0]), Vector([106.0, 62.0])),
                Color::White.into(),
                assets.get_gui_image("gui/inventory")?,
            ),
            inventory_layer: GuiLayerMesh::create(),
            inventory_shown: false,
            fps_display: TextLineRenderer::create(
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

    pub fn chat_box(&self) -> &chat::ChatBox {
        &self.chat_box
    }

    pub fn chat_box_mut(&mut self) -> &mut chat::ChatBox {
        &mut self.chat_box
    }

    pub fn update(&mut self, request: GuiRequest, responses: &mut GuiResponseQueue) {
        match request {
            GuiRequest::HandleMouse(inputs) => {
                self.handle_mouse(inputs, responses);
            }
            GuiRequest::HandleKeyboard(inputs) => {
                self.handle_keyboard(inputs, responses);
            }
            GuiRequest::ReloadAssets(assets) => {
                if let Err(message) = self.reload_assets(assets, responses) {
                    responses.push_back(GuiResponse::AssetError {
                        message,
                    });
                }
            }
            GuiRequest::HeldItem(item, assets) => {
                self.update_item_display(item, assets, responses);
            }
            GuiRequest::ChatMessage { content, color, assets } => {
                self.chat_message(content, color, assets, responses);
            }
            GuiRequest::DebugFPS(fps) => {
                self.update_fps_display(fps, responses);
            }
            GuiRequest::DebugPlayerInfo { position, velocity } => {
                self.update_player_info_display(position, velocity, responses);
            }
        }
    }

    fn handle_mouse(&mut self, inputs: &InputState, responses: &mut GuiResponseQueue) {
        self.set_cursor_position(inputs.cursor_pos().map(|x| x as f32));
        let cursor_offset = self.anchor_adjustment(self.cursor.anchor(), self.hotbar.anchor())
            + self.cursor.offset();

        let handled = self.hotbar.handle_cursor(cursor_offset, inputs);
        if !handled {
            responses.push_back(GuiResponse::WorldHandleMouse);
        }
    }

    fn handle_keyboard(&mut self, inputs: &InputState, responses: &mut GuiResponseQueue) {
        // TODO: probably need some kind of "focus" system... idk how exactly that should work
        let handled = self.chat_box.handle_keyboard(inputs, responses) ||
            {
                if inputs.key_was_pressed(Key::E) {
                    self.inventory_shown = !self.inventory_shown;
                    true
                }
                else {
                    false
                }
            };
        if !handled {
            responses.push_back(GuiResponse::WorldHandleKeyboard);
        }
    }

    fn reload_assets(&mut self, assets: &mut AssetPool, responses: &mut GuiResponseQueue) -> Result<(), String> {
        let _ = responses;
        self.hotbar.reload_assets(assets)?;
        self.chat_box.reload_assets();
        self.inventory.set_atlas_region(assets.get_gui_image("gui/inventory")?);
        self.inventory_layer.clear();
        Ok(())
    }

    fn update_item_display(&mut self, item: Item, assets: &AssetPool, responses: &mut GuiResponseQueue) {
        let _ = responses;
        self.cursor.set_item_type(item.item_type());
        if item.item_type().is_air() {
            self.hotbar.set_held_item_text(String::new());
        }
        else {
            let item_key = TextAsset::simple(format!("item.{}", item.item_type()));
            let item_name = assets.resolve_text(&item_key);
            self.hotbar.set_held_item_text(match item.count() {
                1 => item_name.to_string(),
                count => format!("{item_name} ({count})")
            });
        }
    }

    fn chat_message(&mut self, content: TextElement, color: Color, assets: &AssetPool, responses: &mut GuiResponseQueue) {
        let _ = responses;
        let text = content.resolve_text(assets);
        self.chat_box.add_plain_message(text, color);
    }

    fn update_fps_display(&mut self, min_fps: f32, responses: &mut GuiResponseQueue) {
        let _ = responses;
        self.fps_display.data_mut().set_text(format!("Min recent FPS: {min_fps:.1}"));
    }

    fn update_player_info_display(&mut self, position: Vector<f32, 2>, velocity: Vector<f32, 2>, responses: &mut GuiResponseQueue) {
        let _ = responses;
        self.player_info_display.data_mut().set_text(format!(
            "P=({:.0}, {:.0}); V=({:.1}, {:.1})",
            position.x().floor(),
            position.y().floor(),
            velocity.x(),
            velocity.y(),
        ));
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        assets.gui_shaders().set_uniform("offset_scale", &self.offset_scale);
        assets.gui_shaders().set_uniform("tex_atlas", assets.gui_texture());

        if self.inventory_shown {
            if self.inventory_layer.is_empty() {
                self.inventory.append_to_mesh(self.inventory_layer.data_mut(), Vector::zero());
                self.inventory_layer.upload_buffers();
            }
            assets.gui_texture().bind();
            assets.gui_shaders().set_uniform("anchor", &Vector([0.5f32, 0.5f32]));
            self.inventory_layer.render();
        }

        self.hotbar.render(assets);
        self.chat_box.render(assets);

        self.fps_display.render(assets);
        self.player_info_display.render(assets);
        self.cursor.render(assets);
    }
}
