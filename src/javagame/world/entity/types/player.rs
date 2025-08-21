use crate::tools::*;
use innovus::tools::phys::Physics;
use crate::tools::asset::AssetPool;
use crate::tools::asset::entity::EntityImage;
use crate::tools::input::{InputState, Key};
use crate::world::entity::{movement, Entity};
use crate::world::entity::render::{EntityPiece, EntityPieceHandle, EntityRenderer};

struct PlayerAppearance {
    idle_image: EntityImage,
    run_image: EntityImage,
    jump_ascend_image: EntityImage,
    jump_descend_image: EntityImage,
    crouch_idle_image: EntityImage,
    crouch_walk_image: EntityImage,
    body: EntityPieceHandle,
}

const JUMP_COOLDOWN_SECONDS: f32 = 0.5;

pub struct Player {
    uuid: Uuid,
    position: Vector<f32, 2>,
    collider: Option<phys::ColliderHandle>,
    appearance: Option<PlayerAppearance>,
    name: String,
    crouching: bool,
    spawn_point: Option<Vector<i64, 2>>,
    movement_accel: f32,
    jump_speed: f32,
    jump_cooldown: f32,
}

impl Player {
    pub fn new(position: Vector<f32, 2>, name: Option<String>) -> Self {
        Self {
            uuid: generate_uuid(),
            position,
            collider: None,
            appearance: None,
            name: name.unwrap_or_else(|| "(anonymous)".into()),
            crouching: false,
            spawn_point: None,
            movement_accel: 32.0,
            jump_speed: 12.5,
            jump_cooldown: 0.0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn respawn(&mut self, physics: &mut Physics) {
        let collider = physics.get_collider_mut(self.collider.as_ref().unwrap()).unwrap();

        if let Some(Vector([x, y])) = self.spawn_point {
            collider.rectangle.shift_min_x_to(x as f32 + 0.5 - 0.5 * collider.rectangle.width());
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

    fn collider(&self) -> Option<&phys::ColliderHandle> {
        self.collider.as_ref()
    }

    fn init_collision(&mut self, physics: &mut Physics) {
        self.collider = Some(physics.add_collider(phys::Collider::new(
            Rectangle::from_size(Vector([self.position.x() - 0.375, self.position.y()]), Vector([0.75, 1.625])),
            Vector::zero(),
        )));
    }

    fn init_appearance(&mut self, assets: &mut AssetPool, renderer: &mut EntityRenderer) {
        if self.appearance.is_none() {
            let idle_image = assets.get_entity_image("player/idle").unwrap();
            let run_image = assets.get_entity_image("player/run").unwrap();
            let jump_ascend_image = assets.get_entity_image("player/jump_ascend").unwrap();
            let jump_descend_image = assets.get_entity_image("player/jump_descend").unwrap();
            let crouch_idle_image = assets.get_entity_image("player/crouch_idle").unwrap();
            let crouch_walk_image = assets.get_entity_image("player/crouch_walk").unwrap();

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
    }

    fn update(&mut self, dt: f32, inputs: &InputState, physics: &mut Physics, renderer: &mut EntityRenderer) {
        let mut velocity = Vector::zero();
        let mut touching_ground = true;

        if let Some(collider) = &self.collider {
            let collider = physics.get_collider_mut(collider).unwrap();
            velocity = collider.velocity;
            touching_ground = collider.hit_bottom;

            if !inputs.key_is_held(Key::Space) {
                self.jump_cooldown = 0.0;
            }
            if self.jump_cooldown <= 0.0 {
                if inputs.key_is_held(Key::Space) && touching_ground {
                    collider.velocity.set_y(self.jump_speed);
                    self.jump_cooldown += JUMP_COOLDOWN_SECONDS;
                }
            }
            else {
                self.jump_cooldown -= dt;
            }

            if inputs.key_is_held(Key::LeftShift) {
                self.crouching = true;
                collider.rectangle.set_max_y(collider.rectangle.min_y() + 1.4375); // 23 px
            }
            else {
                self.crouching = false;
                collider.rectangle.set_max_y(collider.rectangle.min_y() + 1.625); // 26 px
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

            movement::apply_gravity(collider, dt, movement::DEFAULT_GRAVITY_ACCELERATION, movement::DEFAULT_TERMINAL_VELOCITY);
            movement::apply_friction(collider, dt, movement::DEFAULT_FRICTION_DECELERATION);

            self.position.set_x(collider.rectangle.min_x() + 0.375);
            self.position.set_y(collider.rectangle.min_y());
        }

        if let Some(appearance) = &mut self.appearance {
            let body = renderer.get_piece_mut(&appearance.body);
            body.set_world_position(self.position);

            if velocity.x() != 0.0 {
                body.set_flip_x(velocity.x() < 0.0);
            }

            if touching_ground {
                if velocity.x() != 0.0 {
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
                if velocity.y() > 0.0 {
                    body.set_image(&appearance.jump_ascend_image);
                }
                else {
                    body.set_image(&appearance.jump_descend_image);
                }
            }
        }
    }

    fn destroy(&mut self, physics: &mut Physics, renderer: &mut EntityRenderer) {
        if let Some(collider) = self.collider.take() {
            physics.remove_collider(collider);
        }
        if let Some(appearance) = self.appearance.take() {
            renderer.remove_piece(&appearance.body);
        }
    }
}
