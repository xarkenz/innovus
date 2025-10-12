use innovus::tools::phys::Physics;
use crate::tools::*;
use crate::tools::asset::AssetPool;
use crate::tools::asset::entity::EntityImage;
use crate::tools::input::{InputState, Key};
use crate::world::block::ChunkMap;
use crate::world::entity::{movement, Entity};
use crate::world::entity::render::{EntityPiece, EntityPieceHandle, EntityRenderer};
use crate::world::item::Item;
use crate::world::particle::ParticleManager;
use super::*;

struct PlayerAppearance {
    idle_image: EntityImage,
    run_image: EntityImage,
    jump_ascend_image: EntityImage,
    jump_descend_image: EntityImage,
    crouch_idle_image: EntityImage,
    crouch_walk_image: EntityImage,
    body: EntityPieceHandle,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PlayerMode {
    Normal,
    Spectating,
}

const JUMP_COOLDOWN_SECONDS: f32 = 0.3;
const COYOTE_TIME_SECONDS: f32 = 0.1;

pub struct Player {
    uuid: Uuid,
    position: Vector<f32, 2>,
    velocity: Vector<f32, 2>,
    collider: Option<phys::ColliderHandle>,
    appearance: Option<PlayerAppearance>,
    name: String,
    mode: PlayerMode,
    held_item: Item,
    crouching: bool,
    spawn_point: Option<Vector<i64, 2>>,
    movement_accel: f32,
    jump_speed: f32,
    jump_cooldown: f32,
    coyote_time: f32,
}

impl Player {
    pub fn new(uuid: Uuid, position: Vector<f32, 2>, name: Option<String>, mode: PlayerMode) -> Self {
        Self {
            uuid,
            position,
            velocity: Vector::zero(),
            collider: None,
            appearance: None,
            name: name.unwrap_or_else(|| "(anonymous)".into()),
            mode,
            held_item: Item::new(&crate::world::item::types::AIR, 0),
            crouching: false,
            spawn_point: None,
            movement_accel: 32.0,
            jump_speed: 15.0,
            jump_cooldown: 0.0,
            coyote_time: COYOTE_TIME_SECONDS,
        }
    }

    pub fn velocity(&self) -> Vector<f32, 2> {
        self.velocity
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mode(&self) -> PlayerMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: PlayerMode) {
        self.mode = mode;
    }

    pub fn held_item(&self) -> &Item {
        &self.held_item
    }

    pub fn set_held_item(&mut self, item: Item) {
        self.held_item = item;
    }

    pub fn respawn(&mut self, physics: &mut Physics) {
        let collider = physics.get_collider_mut(self.collider.as_ref().unwrap()).unwrap();

        if let Some(Vector([x, y])) = self.spawn_point {
            collider.rectangle.shift_min_x_to(x as f32 + 0.5 - 0.5 * collider.rectangle.x_span());
            collider.rectangle.shift_min_y_to(y as f32);
            // if not colliding, return
        }

        // TODO
    }
}

impl Entity for Player {
    fn entity_type(&self) -> &'static str {
        "player"
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    fn attach_collision(&mut self, physics: &mut Physics) {
        self.collider = Some(physics.add_collider(phys::Collider::new(
            Rectangle::from_span(
                Vector([self.position.x() - pixels(5), self.position.y()]),
                Vector([pixels(10), pixels(26)]),
            ),
            Vector::zero(),
        )));
    }

    fn attach_appearance(&mut self, assets: &mut AssetPool, renderer: &mut EntityRenderer) {
        if let Some(appearance) = self.appearance.take() {
            renderer.remove_piece(appearance.body);
        }

        let idle_image = assets.get_entity_image("entity/player/idle").unwrap();
        let run_image = assets.get_entity_image("entity/player/run").unwrap();
        let jump_ascend_image = assets.get_entity_image("entity/player/jump_ascend").unwrap();
        let jump_descend_image = assets.get_entity_image("entity/player/jump_descend").unwrap();
        let crouch_idle_image = assets.get_entity_image("entity/player/crouch_idle").unwrap();
        let crouch_walk_image = assets.get_entity_image("entity/player/crouch_walk").unwrap();

        let body = EntityPiece::new(self.position, idle_image.clone());

        self.appearance = Some(PlayerAppearance {
            idle_image,
            run_image,
            jump_ascend_image,
            jump_descend_image,
            crouch_idle_image,
            crouch_walk_image,
            body: renderer.add_piece(body),
        });
    }

    fn detach_collision(&mut self, physics: &mut Physics) {
        if let Some(collider) = self.collider.take() {
            physics.remove_collider(collider);
        }
    }

    fn detach_appearance(&mut self, renderer: &mut EntityRenderer) {
        if let Some(appearance) = self.appearance.take() {
            renderer.remove_piece(appearance.body);
        }
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
        let _ = (chunks, particles);

        let mut touching_ground = true;

        if self.mode == PlayerMode::Spectating {
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

            self.position += self.velocity.mul(dt);
        }
        else if let Some(collider) = &self.collider {
            let collider = physics.get_collider_mut(collider).unwrap();
            self.velocity = collider.velocity;

            if collider.hit_bottom {
                self.coyote_time = COYOTE_TIME_SECONDS;
            }
            else if self.coyote_time > 0.0 {
                self.coyote_time -= dt;
            }
            else {
                touching_ground = false;
            }

            let jump_held = inputs.key_is_held(Key::W) || inputs.key_is_held(Key::Space);
            if !jump_held {
                self.jump_cooldown = 0.0;
            }
            if self.jump_cooldown <= 0.0 {
                if jump_held && touching_ground {
                    collider.velocity.set_y(self.jump_speed);
                    self.jump_cooldown += JUMP_COOLDOWN_SECONDS;
                    self.coyote_time = 0.0;
                }
            }
            else {
                self.jump_cooldown -= dt;
            }

            let crouch_held = inputs.key_is_held(Key::S) || inputs.key_is_held(Key::LeftShift);
            if crouch_held {
                self.crouching = true;
                collider.rectangle.max.set_y(collider.rectangle.min.y() + pixels(23));
            }
            else {
                self.crouching = false;
                collider.rectangle.max.set_y(collider.rectangle.min.y() + pixels(26));
            }

            let speed_multiplier = if self.crouching && touching_ground {
                0.5
            } else {
                1.0
            };

            if inputs.key_is_held(Key::A) {
                collider.velocity.set_x((collider.velocity.x() - self.movement_accel * dt).max(speed_multiplier * -5.0));
            }
            if inputs.key_is_held(Key::D) {
                collider.velocity.set_x((collider.velocity.x() + self.movement_accel * dt).min(speed_multiplier * 5.0));
            }

            collider.velocity.set_y(movement::apply_gravity(
                collider.velocity.y(),
                dt,
                movement::DEFAULT_GRAVITY_ACCELERATION,
                movement::DEFAULT_TERMINAL_VELOCITY,
            ));
            collider.velocity.set_x(movement::apply_friction(
                collider.velocity.x(),
                dt,
                movement::DEFAULT_FRICTION_DECELERATION,
            ));

            self.position.set_x(collider.rectangle.min.x() + pixels(5));
            self.position.set_y(collider.rectangle.min.y());
        }

        if let Some(appearance) = &mut self.appearance {
            let body = renderer.get_piece_mut(&appearance.body);
            body.set_world_position(self.position);

            if self.velocity.x() != 0.0 {
                body.set_flip_x(self.velocity.x() < 0.0);
            }

            if touching_ground {
                if self.velocity.x() != 0.0 {
                    if self.crouching {
                        body.set_image(&appearance.crouch_walk_image);
                    }
                    else {
                        body.set_image(&appearance.run_image);
                    }
                }
                else {
                    if self.crouching {
                        body.set_image(&appearance.crouch_idle_image);
                    }
                    else {
                        body.set_image(&appearance.idle_image);
                    }
                }
            }
            else {
                if self.velocity.y() > 0.0 {
                    body.set_image(&appearance.jump_ascend_image);
                }
                else {
                    body.set_image(&appearance.jump_descend_image);
                }
            }
        }
    }
}
