use crate::tools::*;
use innovus::tools::phys::Physics;
use crate::tools::input::{InputState, Key};
use crate::world::block::{BlockType, ChunkMap};
use crate::world::entity::{movement, Entity};
use crate::world::entity::render::{EntityRenderer};
use crate::world::particle::ParticleManager;

pub struct Spectator {
    uuid: Uuid,
    position: Vector<f32, 2>,
    velocity: Vector<f32, 2>,
    name: String,
    held_item: &'static BlockType,
    movement_accel: f32,
}

impl Spectator {
    pub fn new(position: Vector<f32, 2>, name: Option<String>) -> Self {
        Self {
            uuid: generate_uuid(),
            position,
            velocity: Vector::zero(),
            name: name.unwrap_or_else(|| "(anonymous)".into()),
            held_item: &crate::world::block::types::AIR,
            movement_accel: 32.0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Entity for Spectator {
    fn entity_type(&self) -> &'static str {
        "spectator"
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    fn held_item(&self) -> &'static BlockType {
        self.held_item
    }

    fn update(
        &mut self,
        dt: f32,
        inputs: &InputState,
        physics: &mut Physics,
        renderer: &mut EntityRenderer,
        chunks: &mut ChunkMap,
        particles: &mut ParticleManager,
    ) {
        let _ = (physics, renderer, chunks, particles);

        const SPEED_LIMIT: f32 = 20.0;

        if inputs.key_is_held(Key::A) {
            self.velocity.set_x((self.velocity.x() - self.movement_accel * dt).max(-SPEED_LIMIT));
        }
        if inputs.key_is_held(Key::D) {
            self.velocity.set_x((self.velocity.x() + self.movement_accel * dt).min(SPEED_LIMIT));
        }
        if inputs.key_is_held(Key::S) {
            self.velocity.set_y((self.velocity.y() - self.movement_accel * dt).max(-SPEED_LIMIT));
        }
        if inputs.key_is_held(Key::W) {
            self.velocity.set_y((self.velocity.y() + self.movement_accel * dt).min(SPEED_LIMIT));
        }

        self.velocity = self.velocity.map(|x| movement::apply_friction(
            x,
            dt,
            movement::DEFAULT_FRICTION_DECELERATION,
        ));

        self.position += self.velocity * dt;
    }
}
