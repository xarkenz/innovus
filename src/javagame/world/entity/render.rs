use innovus::gfx::{Geometry, Vertex2D};
use innovus::tools::{Rectangle, Vector};
use innovus::tools::arena::{ArenaHandle, BoundedArena, UnboundedArena};
use crate::tools::asset::entity::EntityImage;

#[derive(Debug)]
pub struct EntityPiece {
    dirty: bool,
    atlas_region: Rectangle<u32>,
    world_position: Vector<f32, 2>,
    image: EntityImage,
    visible: bool,
    color: Vector<f32, 4>,
    flip_x: bool,
    flip_y: bool,
    frame: u32,
    frame_timer: u32,
}

impl EntityPiece {
    pub fn new(world_position: Vector<f32, 2>, initial_image: EntityImage) -> Self {
        Self {
            dirty: true,
            atlas_region: initial_image.atlas_base_region,
            world_position,
            image: initial_image,
            visible: true,
            color: Vector::one(),
            flip_x: false,
            flip_y: true,
            frame: 0,
            frame_timer: 0,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn atlas_region(&self) -> Rectangle<u32> {
        self.atlas_region
    }

    pub fn set_image(&mut self, image: &EntityImage) {
        if self.image != *image {
            self.dirty = true;
            self.image = image.clone();
            // Reset the animation for the new image
            self.frame = 0;
            self.frame_timer = self.image.animation
                .as_ref()
                .map_or(0, |animation| animation.frame_time);
        }
    }

    pub fn world_position(&self) -> Vector<f32, 2> {
        self.world_position
    }

    pub fn set_world_position(&mut self, position: Vector<f32, 2>) {
        self.dirty = self.dirty || position != self.world_position;
        self.world_position = position;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.dirty = self.dirty || visible != self.visible;
        self.visible = visible;
    }

    pub fn color(&self) -> Vector<f32, 4> {
        self.color
    }

    pub fn set_color(&mut self, color: Vector<f32, 4>) {
        self.dirty = self.dirty || color != self.color;
        self.color = color;
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

    pub fn get_world_bounds(&self) -> Rectangle<f32> {
        let mut bounds = self.image.world_offset;
        bounds.shift_by(self.world_position);
        bounds
    }

    pub fn tick(&mut self) {
        if let Some(animation) = &self.image.animation {
            self.frame_timer = self.frame_timer.saturating_sub(1);
            if self.frame_timer == 0 {
                // Advance to the next frame
                self.frame = (self.frame + 1) % animation.frame_count;
                self.dirty = true;
                // Reset the frame timer
                self.frame_timer = animation.frame_time;
            }
        }

        let mut region = self.image.atlas_base_region;
        region.shift_y_by(self.frame * region.height());
        if self.flip_x {
            region.flip_x();
        }
        if self.flip_y {
            region.flip_y();
        }
        self.dirty = self.dirty || region != self.atlas_region;
        self.atlas_region = region;
    }
}

pub struct EntityPieceHandle {
    batch: ArenaHandle,
    piece: ArenaHandle,
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

    pub fn add_piece(&mut self, mut piece: EntityPiece) -> EntityPieceHandle {
        for (batch_handle, batch) in self.batches.values_mut() {
            match batch.try_add_piece(piece) {
                Ok(piece_handle) => return EntityPieceHandle {
                    batch: batch_handle,
                    piece: piece_handle,
                },
                Err(returned_piece) => piece = returned_piece,
            }
        }

        let batch_handle = self.batches.insert(RendererBatch::new());
        let piece_handle = self.batches.get_current_mut(batch_handle.slot).unwrap()
            .try_add_piece(piece).unwrap();
        EntityPieceHandle {
            batch: batch_handle,
            piece: piece_handle,
        }
    }

    pub fn get_piece(&self, handle: &EntityPieceHandle) -> &EntityPiece {
        self.batches.get(handle.batch)
            .and_then(|batch| batch.get_piece(handle.piece))
            .expect("attempted to access entity piece after removal")
    }

    pub fn get_piece_mut(&mut self, handle: &EntityPieceHandle) -> &mut EntityPiece {
        self.batches.get_mut(handle.batch)
            .and_then(|batch| batch.get_piece_mut(handle.piece))
            .expect("attempted to access entity piece after removal")
    }

    pub fn remove_piece(&mut self, handle: EntityPieceHandle) -> Option<EntityPiece> {
        let result = self.batches.get_mut(handle.batch)?.remove_piece(handle.piece);
        if self.batches.get(handle.batch)?.is_empty() {
            self.batches.remove(handle.batch);
        }
        result
    }

    pub fn tick(&mut self) {
        for (_, batch) in self.batches.values_mut() {
            batch.tick();
        }
    }

    pub fn render_all(&mut self) {
        for (_, batch) in self.batches.values_mut() {
            batch.render();
        }
    }
}

const PIECES_PER_BATCH: usize = 512;

struct RendererBatch {
    geometry: Geometry<Vertex2D>,
    pieces: BoundedArena<EntityPiece>,
}

impl RendererBatch {
    pub fn new() -> Self {
        Self {
            geometry: Geometry::new_render().unwrap(),
            pieces: BoundedArena::new(PIECES_PER_BATCH),
        }
    }

    pub fn try_add_piece(&mut self, piece: EntityPiece) -> Result<ArenaHandle, EntityPiece> {
        self.pieces.try_insert(piece)
    }

    pub fn get_piece(&self, handle: ArenaHandle) -> Option<&EntityPiece> {
        self.pieces.get(handle)
    }

    pub fn get_piece_mut(&mut self, handle: ArenaHandle) -> Option<&mut EntityPiece> {
        self.pieces.get_mut(handle)
    }

    pub fn remove_piece(&mut self, handle: ArenaHandle) -> Option<EntityPiece> {
        self.pieces.remove(handle)
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.values().count() == 0
    }

    pub fn tick(&mut self) {
        for (_, piece) in self.pieces.values_mut() {
            piece.tick();
        }
    }

    pub fn render(&mut self) {
        if self.geometry.is_empty() {
            let mut vertices = Vec::new();
            let mut faces = Vec::new();
            for _ in 0..PIECES_PER_BATCH {
                let index = vertices.len() as u32;
                for _ in 0..4 {
                    vertices.push(Vertex2D::new(Vector::zero(), None, None));
                }
                faces.push([index + 0, index + 1, index + 2]);
                faces.push([index + 2, index + 3, index + 0]);
            }
            self.geometry.add(&vertices, &faces);
        }

        let mut any_changed = false;
        for (handle, piece) in self.pieces.values_mut() {
            if piece.is_dirty() {
                any_changed = true;
                let color = if piece.is_visible() { piece.color() } else { Vector::zero() };
                let bounds = piece.get_world_bounds();
                let atlas_region = piece.atlas_region();
                let vertices = [
                    (bounds.min(), atlas_region.min()),
                    (bounds.min_x_max_y(), atlas_region.min_x_max_y()),
                    (bounds.max(), atlas_region.max()),
                    (bounds.max_x_min_y(), atlas_region.max_x_min_y()),
                ];
                let mut vertex_index = handle.slot * 4;
                for (position, uv) in vertices {
                    let vertex = self.geometry.vertex_at_mut(vertex_index);
                    vertex.pos = position.with_z(1.0);
                    vertex.color = color;
                    vertex.uv = uv.map(|x| x as f32);
                    vertex_index += 1;
                }
                piece.set_dirty(false);
            }
        }

        if any_changed {
            self.geometry.update_vertex_buffer();
        }
        self.geometry.render();
    }
}
