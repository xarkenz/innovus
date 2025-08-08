use innovus::gfx::{Geometry, Vertex2D};
use innovus::tools::{Rectangle, Vector};
use innovus::tools::arena::{ArenaHandle, BoundedArena, UnboundedArena};

#[derive(Debug)]
pub struct EntityImage {
    position: Vector<f32, 2>,
    offset: Rectangle<f32>,
    visible: bool,
    color: [f32; 4],
    uv_base: Rectangle<u32>,
    flip_x: bool,
    flip_y: bool,
    frame: u32,
    dirty: bool,
}

impl EntityImage {
    pub fn new(position: Vector<f32, 2>, offset: Rectangle<f32>, visible: bool, uv_base: Rectangle<u32>) -> Self {
        Self {
            position,
            offset,
            visible,
            color: [1.0; 4],
            uv_base,
            flip_x: false,
            flip_y: true,
            frame: 0,
            dirty: true,
        }
    }

    pub fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector<f32, 2>) {
        self.dirty = self.dirty || position != self.position;
        self.position = position;
    }

    pub fn offset(&self) -> Rectangle<f32> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Rectangle<f32>) {
        self.dirty = true;
        self.offset = offset;
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

    pub fn get_bounds(&self) -> Rectangle<f32> {
        let mut bounds = self.offset;
        bounds.shift_by(self.position);
        bounds
    }

    pub fn get_uv(&self) -> Rectangle<u32> {
        let mut uv = self.uv_base;
        uv.shift_y_by(self.frame * uv.height());
        if self.flip_x {
            uv.flip_x();
        }
        if self.flip_y {
            uv.flip_y();
        }
        uv
    }
}

pub struct EntityImageHandle {
    batch: ArenaHandle,
    image: ArenaHandle,
}

pub struct EntityRenderer {
    batches: UnboundedArena<RendererBatch>,
}

impl EntityRenderer {
    pub fn new() -> Self {
        Self {
            batches: UnboundedArena::new(),
        }
    }

    pub fn add_image(&mut self, mut image: EntityImage) -> EntityImageHandle {
        for (batch_handle, batch) in self.batches.values_mut() {
            match batch.try_add_image(image) {
                Ok(image_handle) => return EntityImageHandle {
                    batch: batch_handle,
                    image: image_handle,
                },
                Err(returned_image) => image = returned_image,
            }
        }

        let batch_handle = self.batches.insert(RendererBatch::new());
        let image_handle = self.batches.get_current_mut(batch_handle.slot).unwrap()
            .try_add_image(image).unwrap();
        EntityImageHandle {
            batch: batch_handle,
            image: image_handle,
        }
    }

    pub fn get_image(&self, handle: &EntityImageHandle) -> &EntityImage {
        self.batches.get(handle.batch)
            .and_then(|batch| batch.get_image(handle.image))
            .expect("attempted to access entity image after removal")
    }

    pub fn get_image_mut(&mut self, handle: &EntityImageHandle) -> &mut EntityImage {
        self.batches.get_mut(handle.batch)
            .and_then(|batch| batch.get_image_mut(handle.image))
            .expect("attempted to access entity image after removal")
    }

    pub fn remove_image(&mut self, handle: &EntityImageHandle) -> Option<EntityImage> {
        let result = self.batches.get_mut(handle.batch)?.remove_image(handle.image);
        if self.batches.get(handle.batch)?.is_empty() {
            self.batches.remove(handle.batch);
        }
        result
    }

    pub fn render_all(&mut self) {
        for (_, batch) in self.batches.values_mut() {
            batch.render();
        }
    }
}

const IMAGES_PER_BATCH: usize = 512;

struct RendererBatch {
    geometry: Geometry<Vertex2D>,
    images: BoundedArena<EntityImage>,
}

impl RendererBatch {
    pub fn new() -> Self {
        Self {
            geometry: Geometry::new_render().unwrap(),
            images: BoundedArena::new(IMAGES_PER_BATCH),
        }
    }

    pub fn try_add_image(&mut self, image: EntityImage) -> Result<ArenaHandle, EntityImage> {
        self.images.try_insert(image)
    }

    pub fn get_image(&self, handle: ArenaHandle) -> Option<&EntityImage> {
        self.images.get(handle)
    }

    pub fn get_image_mut(&mut self, handle: ArenaHandle) -> Option<&mut EntityImage> {
        self.images.get_mut(handle)
    }

    pub fn remove_image(&mut self, handle: ArenaHandle) -> Option<EntityImage> {
        self.images.remove(handle)
    }

    pub fn is_empty(&self) -> bool {
        self.images.values().count() == 0
    }

    pub fn render(&mut self) {
        if self.geometry.is_empty() {
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            for _ in 0..IMAGES_PER_BATCH as u32 {
                let index = vertices.len() as u32;
                for _ in 0..4 {
                    vertices.push(Vertex2D::new([0.0; 3], Some([0.0; 4]), None));
                }
                faces.push([index + 0, index + 1, index + 2]);
                faces.push([index + 2, index + 3, index + 0]);
            }
            self.geometry.add(&vertices, &faces);
        }

        let mut any_changed = false;
        for (handle, image) in self.images.values_mut() {
            if image.is_dirty() {
                any_changed = true;
                let color = if image.is_visible() { image.color() } else { [0.0; 4] };
                let bounds = image.get_bounds();
                let uv = image.get_uv();
                let vertices = [
                    (bounds.min(), uv.min()),
                    (bounds.min_x_max_y(), uv.min_x_max_y()),
                    (bounds.max(), uv.max()),
                    (bounds.max_x_min_y(), uv.max_x_min_y()),
                ];
                let mut vertex_index = handle.slot * 4;
                for (position, uv_offset) in vertices {
                    self.geometry.set_vertex(vertex_index, &Vertex2D::new(
                        [position.x(), position.y(), 1.0],
                        Some(color),
                        Some([uv_offset.x() as f32, uv_offset.y() as f32]),
                    ));
                    vertex_index += 1;
                }
                image.set_dirty(false);
            }
        }

        if any_changed {
            self.geometry.update_vertex_buffer();
        }
        self.geometry.render();
    }
}
