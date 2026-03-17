use super::*;
use crate::tools::asset::entity::EntityImage;
use crate::tools::input::{InputState, Key, MouseButtonLeft, MouseButtonMiddle, MouseButtonRight};
use crate::world::block::CHUNK_SIZE;
use crate::world::entity::render::{EntityPiece, EntityPieceHandle};
use crate::world::item::ITEM_TYPES;

const JUMP_COOLDOWN_SECONDS: f32 = 0.3;
const COYOTE_TIME_SECONDS: f32 = 0.1;
const SPECTATOR_MAX_SPEED: f32 = 20.0;

pub struct Player {
    uuid: Uuid,
    position: Vector<f32, 2>,
    velocity: Vector<f32, 2>,
    collider: Option<phys::ColliderHandle>,
    inputs: PlayerInputs,
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PlayerMode {
    Normal,
    Spectating,
}

#[derive(Clone, Default, Debug)]
struct PlayerInputs {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    jump: bool,
    crouch: bool,
    destroy_block: bool,
    use_item: bool,
    pick_block: bool,
    next_mode: bool,
    next_item: isize,
    block_pos: Option<(Vector<i64, 2>, BlockSide)>,
}

struct PlayerAppearance {
    idle_image: EntityImage,
    run_image: EntityImage,
    jump_ascend_image: EntityImage,
    jump_descend_image: EntityImage,
    crouch_idle_image: EntityImage,
    crouch_walk_image: EntityImage,
    body: EntityPieceHandle,
}

impl Player {
    pub fn new(uuid: Uuid, position: Vector<f32, 2>, name: Option<String>, mode: PlayerMode) -> Self {
        Self {
            uuid,
            position,
            velocity: Vector::zero(),
            collider: None,
            inputs: Default::default(),
            appearance: None,
            name: name.unwrap_or_else(|| "(anonymous)".into()),
            mode,
            held_item: Default::default(),
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

    pub fn handle_mouse(&mut self, inputs: &InputState, cursor_world_pos: Vector<f32, 2>) {
        self.inputs.destroy_block = inputs.button_is_held(MouseButtonLeft);
        self.inputs.use_item = inputs.button_is_held(MouseButtonRight);
        self.inputs.pick_block = inputs.button_is_held(MouseButtonMiddle);

        if self.inputs.destroy_block || self.inputs.use_item || self.inputs.pick_block {
            let block_pos = cursor_world_pos.map(|x| x.floor() as i64);

            if self.inputs.block_pos.is_some_and(|(pos, _)| block_pos == pos) {
                self.inputs.destroy_block = false;
                self.inputs.use_item = false;
                self.inputs.pick_block = false;
            }
            else {
                self.inputs.block_pos = Some((block_pos, BlockSide::from_position(cursor_world_pos)));
            }
        }
        else {
            self.inputs.block_pos = None;
        }
    }

    pub fn handle_keyboard(&mut self, inputs: &InputState) {
        self.inputs.left = inputs.key_is_held(Key::A);
        self.inputs.right = inputs.key_is_held(Key::D);
        self.inputs.up = inputs.key_is_held(Key::W);
        self.inputs.down = inputs.key_is_held(Key::S);
        self.inputs.jump = self.inputs.up || inputs.key_is_held(Key::Space);
        self.inputs.crouch = self.inputs.down || inputs.key_is_held(Key::LeftShift);
        self.inputs.next_mode ^= inputs.key_was_pressed(Key::F4);
        if inputs.key_was_repeated(Key::Tab) {
            self.inputs.next_item += if inputs.key_is_held(Key::LeftShift) { -1 } else { 1 };
        }
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

    fn detach_collision(&mut self, physics: &mut Physics) {
        if let Some(collider) = self.collider.take() {
            physics.remove_collider(collider);
        }
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

    fn detach_appearance(&mut self, renderer: &mut EntityRenderer) {
        if let Some(appearance) = self.appearance.take() {
            renderer.remove_piece(appearance.body);
        }
    }

    fn run_frame(&mut self, dt: f32, physics: &mut Physics, renderer: &mut EntityRenderer, responses: &mut EntityResponseQueue) {
        let _ = responses;
        let mut touching_ground = true;

        if self.mode == PlayerMode::Spectating {
            if self.inputs.left {
                self.velocity.set_x((self.velocity.x() - self.movement_accel * dt).max(-SPECTATOR_MAX_SPEED));
            }
            if self.inputs.right {
                self.velocity.set_x((self.velocity.x() + self.movement_accel * dt).min(SPECTATOR_MAX_SPEED));
            }
            if self.inputs.up {
                self.velocity.set_y((self.velocity.y() + self.movement_accel * dt).min(SPECTATOR_MAX_SPEED));
            }
            if self.inputs.down {
                self.velocity.set_y((self.velocity.y() - self.movement_accel * dt).max(-SPECTATOR_MAX_SPEED));
            }

            self.velocity = self.velocity.map(|x| apply_friction(
                x,
                dt,
                DEFAULT_FRICTION_DECELERATION,
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

            if !self.inputs.jump {
                self.jump_cooldown = 0.0;
            }
            if self.jump_cooldown <= 0.0 {
                if self.inputs.jump && touching_ground {
                    collider.velocity.set_y(self.jump_speed);
                    self.jump_cooldown += JUMP_COOLDOWN_SECONDS;
                    self.coyote_time = 0.0;
                }
            }
            else {
                self.jump_cooldown -= dt;
            }

            if self.inputs.crouch {
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

            if self.inputs.left {
                collider.velocity.set_x((collider.velocity.x() - self.movement_accel * dt).max(speed_multiplier * -5.0));
            }
            if self.inputs.right {
                collider.velocity.set_x((collider.velocity.x() + self.movement_accel * dt).min(speed_multiplier * 5.0));
            }

            collider.velocity.set_y(apply_gravity(
                collider.velocity.y(),
                dt,
                DEFAULT_GRAVITY_ACCELERATION,
                DEFAULT_TERMINAL_VELOCITY,
            ));
            collider.velocity.set_x(apply_friction(
                collider.velocity.x(),
                dt,
                DEFAULT_FRICTION_DECELERATION,
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

        self.inputs.left = false;
        self.inputs.right = false;
        self.inputs.up = false;
        self.inputs.down = false;
        self.inputs.jump = false;
        self.inputs.crouch = false;
    }

    fn tick(&mut self, chunks: &ChunkMap, responses: &mut EntityResponseQueue) {
        let _ = responses;

        if let Some((block_pos, side)) = self.inputs.block_pos {
            let chunk_location = block_pos.map(|x| x.div_euclid(CHUNK_SIZE as i64));
            let block_x = block_pos.x().rem_euclid(CHUNK_SIZE as i64) as usize;
            let block_y = block_pos.y().rem_euclid(CHUNK_SIZE as i64) as usize;

            if self.inputs.destroy_block {
                responses.push_back((self.uuid, EntityResponse::DestroyBlock {
                    chunk_location,
                    block_x,
                    block_y,
                }));
            }
            if self.inputs.use_item {
                responses.push_back((self.uuid, EntityResponse::UseHeldItem {
                    item: self.held_item.clone(),
                    chunk_location,
                    block_x,
                    block_y,
                    side,
                }));
            }
            if self.inputs.pick_block {
                let block_type = chunks
                    .get(chunk_location)
                    .map_or(Default::default(), |chunk| {
                        chunk.block_at(block_x, block_y).block_type()
                    });
                if let Some(item_type) = block_type.item_type() {
                    self.held_item = Item::with_max_count(item_type);
                }
            }
        }

        if self.inputs.next_item != 0 {
            let held_item_type = self.held_item.item_type();
            let item_index = ITEM_TYPES
                .iter()
                .position(|&item_type| item_type == held_item_type)
                .unwrap();
            let next_item_index = (item_index as isize + self.inputs.next_item).rem_euclid(ITEM_TYPES.len() as isize) as usize;
            self.held_item = Item::with_max_count(ITEM_TYPES[next_item_index]);
        }
        if self.inputs.next_mode {
            self.mode = match self.mode {
                PlayerMode::Normal => PlayerMode::Spectating,
                PlayerMode::Spectating => PlayerMode::Normal,
            };
        }

        self.inputs.destroy_block = false;
        self.inputs.use_item = false;
        self.inputs.pick_block = false;
        self.inputs.next_item = 0;
        self.inputs.next_mode = false;
    }

    fn set_held_item(&mut self, item: Item, responses: &mut EntityResponseQueue) {
        let _ = responses;
        self.held_item = item;
    }
}
