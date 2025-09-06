use innovus::tools::phys::{ColliderHandle, Physics};
use crate::tools::*;
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use crate::world::block::{BlockType, ChunkMap};
use crate::world::entity::render::EntityRenderer;
use crate::world::particle::ParticleManager;

pub mod movement;
pub mod types;
pub mod render;

pub trait Entity {
    fn entity_type(&self) -> &'static str;

    fn uuid(&self) -> Uuid;

    fn position(&self) -> Vector<f32, 2>;

    fn collider(&self) -> Option<&ColliderHandle> {
        // No collision by default
        None
    }

    fn held_item(&self) -> &'static BlockType {
        &crate::world::block::types::AIR
    }

    fn set_held_item(&mut self, block_type: &'static BlockType) {
        // Do nothing by default
        let _ = block_type;
    }

    fn init_collision(&mut self, physics: &mut Physics) {
        // Do nothing by default
        let _ = physics;
    }

    fn init_appearance(&mut self, assets: &mut AssetPool, renderer: &mut EntityRenderer) {
        // Do nothing by default
        let _ = (assets, renderer);
    }

    fn update(
        &mut self,
        dt: f32,
        inputs: &InputState,
        physics: &mut Physics,
        renderer: &mut EntityRenderer,
        chunks: &mut ChunkMap,
        particles: &mut ParticleManager,
    );

    fn destroy(&mut self, physics: &mut Physics, renderer: &mut EntityRenderer) {
        // Do nothing by default
        let _ = (physics, renderer);
    }
}
