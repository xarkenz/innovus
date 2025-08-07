use innovus::gfx::{Geometry, Vertex2D};
use innovus::tools::{Rectangle, Vector};

pub struct EntityImage {
    position: Rectangle<f32>,
    visible: bool,
    color: [f32; 4],
    uv_base: Rectangle<u32>,
    flip_x: bool,
    flip_y: bool,
    frame: u32,
    dirty: bool,
}

impl EntityImage {
    pub fn new(position: Rectangle<f32>, visible: bool, uv_base: Rectangle<u32>) -> Self {
        Self {
            position,
            visible,
            color: [1.0; 4],
            uv_base,
            flip_x: false,
            flip_y: true,
            frame: 0,
            dirty: true,
        }
    }

    pub fn position(&self) -> Rectangle<f32> {
        self.position
    }

    pub fn set_position(&mut self, position: Rectangle<f32>) {
        self.dirty = true;
        self.position = position;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.dirty = self.dirty || visible != self.visible;
        self.visible = visible;
    }

    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.dirty = self.dirty || color != self.color;
        self.color = color;
    }

    pub fn uv_base(&self) -> Rectangle<u32> {
        self.uv_base
    }

    pub fn set_uv_base(&mut self, uv_base: Rectangle<u32>) {
        self.dirty = true;
        self.uv_base = uv_base;
    }

    pub fn is_flip_x(&self) -> bool {
        self.flip_x
    }

    pub fn set_flip_x(&mut self, flip_x: bool) {
        self.dirty = self.dirty || flip_x != self.flip_x;
        self.flip_x = flip_x;
    }

    pub fn is_flip_y(&self) -> bool {
        self.flip_y
    }

    pub fn set_flip_y(&mut self, flip_y: bool) {
        self.dirty = self.dirty || flip_y != self.flip_y;
        self.flip_y = flip_y;
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }

    pub fn set_frame(&mut self, frame: u32) {
        self.dirty = self.dirty || frame != self.frame;
        self.frame = frame;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}

pub struct EntityRenderer {
    //
}

const IMAGES_PER_BATCH: usize = 512;

struct EntityRendererBatch {
    geometry: Geometry<Vertex2D>,
    images: [Option<EntityImage>]
}
