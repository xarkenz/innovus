use std::mem::offset_of;
use innovus::gfx::{Geometry, Vertex, VertexAttribute, VertexAttributeType};
use innovus::tools::Vector;
use crate::gui::cursor::CursorRenderer;
use crate::gui::text::StringRenderer;
use crate::tools::asset::AssetPool;
use crate::world::item::Item;
use crate::world::item::types::AIR;

pub mod cursor;
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
    cursor_position: Vector<f32, 2>,
    cursor_offset: Vector<f32, 2>,
    cursor_renderer: CursorRenderer,
    hotbar: Geometry<GuiVertex>,
    inventory: Geometry<GuiVertex>,
    inventory_shown: bool,
    fps_display: StringRenderer,
    player_info_display: StringRenderer,
    item_display: StringRenderer,
    input_test: StringRenderer,
}

impl GuiManager {
    pub fn new(viewport_size: Vector<f32, 2>, content_scale: Vector<f32, 2>, gui_scale: f32) -> Self {
        Self {
            viewport_size,
            content_scale,
            gui_scale,
            offset_scale: Self::compute_offset_scale(viewport_size, content_scale * gui_scale),
            cursor_position: Vector::zero(),
            cursor_offset: Vector::zero(),
            cursor_renderer: CursorRenderer::new(Vector::zero(), &AIR),
            hotbar: Geometry::new_render().unwrap(),
            inventory: Geometry::new_render().unwrap(),
            inventory_shown: false,
            fps_display: StringRenderer::new(
                Vector([-1.0, 1.0]),
                Vector([0.0, 0.0]),
                Vector([0.0, 1.0]),
                Vector([1.0, 1.0, 1.0, 1.0]),
                Vector([0.0, 0.0, 0.0, 0.4]),
                String::new(),
            ),
            player_info_display: StringRenderer::new(
                Vector([1.0, 1.0]),
                Vector([0.0, 0.0]),
                Vector([1.0, 1.0]),
                Vector([1.0, 1.0, 1.0, 1.0]),
                Vector([0.0, 0.0, 0.0, 0.4]),
                String::new(),
            ),
            item_display: StringRenderer::new(
                Vector([0.0, -1.0]),
                Vector([0.0, 32.0]),
                Vector([0.5, 0.0]),
                Vector([1.0, 1.0, 1.0, 1.0]),
                Vector([0.0, 0.0, 0.0, 0.4]),
                String::new(),
            ),
            input_test: StringRenderer::new(
                Vector([0.0, 0.5]),
                Vector([0.0, 0.0]),
                Vector([0.5, 0.5]),
                Vector([0.0, 0.0, 0.0, 1.0]),
                Vector([1.0, 1.0, 1.0, 1.0]),
                String::new(),
            ),
        }
    }

    pub fn viewport_size(&self) -> Vector<f32, 2> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector<f32, 2>) {
        self.viewport_size = viewport_size;
        self.offset_scale = Self::compute_offset_scale(viewport_size, self.content_scale * self.gui_scale);
        self.compute_cursor_offset();
    }

    pub fn content_scale(&self) -> Vector<f32, 2> {
        self.content_scale
    }

    pub fn set_content_scale(&mut self, content_scale: Vector<f32, 2>) {
        self.content_scale = content_scale;
        self.offset_scale = Self::compute_offset_scale(self.viewport_size, content_scale * self.gui_scale);
        self.compute_cursor_offset();
    }

    pub fn gui_scale(&self) -> f32 {
        self.gui_scale
    }

    pub fn set_gui_scale(&mut self, gui_scale: f32) {
        self.gui_scale = gui_scale;
        self.offset_scale = Self::compute_offset_scale(self.viewport_size, self.content_scale * gui_scale);
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
        self.cursor_offset = Vector([
            self.cursor_position.x() * 2.0,
            self.cursor_position.y() * -2.0,
        ]);
        self.cursor_offset /= self.content_scale * self.gui_scale;
        self.cursor_renderer.set_offset(self.cursor_offset);
    }

    pub fn inventory_shown(&self) -> bool {
        self.inventory_shown
    }

    pub fn set_inventory_shown(&mut self, shown: bool) {
        self.inventory_shown = shown;
    }

    pub fn invalidate_assets(&mut self) {
        self.hotbar.clear();
        self.inventory.clear();
    }

    pub fn update_fps_display(&mut self, average_fps: f32) {
        self.fps_display.set_string(format!("Average FPS: {average_fps:.1}"));
    }

    pub fn update_player_info_display(&mut self, position: Vector<f32, 2>, velocity: Vector<f32, 2>) {
        self.player_info_display.set_string(format!(
            "P=({:.0}, {:.0}); V=({:.1}, {:.1})",
            position.x().floor(),
            position.y().floor(),
            velocity.x(),
            velocity.y(),
        ));
    }

    pub fn update_item_display(&mut self, item: &Item, assets: &AssetPool) {
        self.cursor_renderer.set_item_type(item.item_type());
        if item.item_type() == &AIR {
            self.item_display.set_string(String::new());
        }
        else {
            let item_key = format!("item.{}", item.item_type());
            let item_name = assets.get_text_string(&item_key);
            self.item_display.set_string(match item.count() {
                1 => item_name.to_string(),
                count => format!("{item_name} ({count})")
            });
        }
    }

    pub fn enter_text(&mut self, text: &str) {
        let mut string = self.input_test.string().to_string();
        string.push_str(text);
        self.input_test.set_string(string);
    }

    pub fn backspace(&mut self) {
        let mut string = self.input_test.string().to_string();
        string.pop();
        self.input_test.set_string(string);
    }

    pub fn clear_text(&mut self) {
        self.input_test.set_string(String::new());
    }

    pub fn render(&mut self, assets: &mut AssetPool) {
        assets.gui_shaders().set_uniform("offset_scale", self.offset_scale);
        assets.gui_shaders().set_uniform("tex_atlas", assets.gui_texture());

        if self.inventory.is_empty() {
            let atlas_region = assets.get_gui_image("gui/inventory").unwrap();
            let anchor = Vector([0.0, 0.0]);
            let to_f32 = |x: u32| x as f32;
            self.inventory.add(
                &[
                    GuiVertex::new(anchor, Vector([-106.0, -60.0]), None, Some(atlas_region.min_x_max_y().map(to_f32))),
                    GuiVertex::new(anchor, Vector([-106.0, 60.0]), None, Some(atlas_region.min().map(to_f32))),
                    GuiVertex::new(anchor, Vector([106.0, 60.0]), None, Some(atlas_region.max_x_min_y().map(to_f32))),
                    GuiVertex::new(anchor, Vector([106.0, -60.0]), None, Some(atlas_region.max().map(to_f32))),
                ],
                &[
                    [0, 1, 2],
                    [2, 3, 0],
                ],
            );
        }
        if self.inventory_shown {
            assets.gui_texture().bind();
            self.inventory.render();
            self.input_test.render(assets);
        }

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
        assets.gui_texture().bind();
        self.hotbar.render();

        self.fps_display.render(assets);
        self.player_info_display.render(assets);
        self.item_display.render(assets);
        self.cursor_renderer.render(assets);
    }
}
