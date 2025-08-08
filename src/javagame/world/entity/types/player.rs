use crate::tools::*;
use innovus::tools::phys::Physics;
use crate::tools::asset::AssetPool;
use crate::tools::input::{InputState, Key};
use crate::world::entity::{movement, Entity};
use crate::world::entity::render::{EntityImage, EntityImageHandle, EntityRenderer};

struct PlayerAppearance {
    uv_idle: Rectangle<u32>,
    uv_run: Rectangle<u32>,
    body_image: EntityImageHandle,
    frame_timer: f32,
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
            Rectangle::from_size(Vector([self.position.x() - 0.375, self.position.y()]), Vector([0.75, 1.6875])),
            Vector::zero(),
        )));
    }

    fn init_appearance(&mut self, assets: &mut AssetPool, renderer: &mut EntityRenderer) {
        if self.appearance.is_none() {
            let uv_idle = assets.get_entity_image_uv("player/idle").unwrap();
            let mut uv_run = assets.get_entity_image_uv("player/run").unwrap();
            uv_run.set_max_y(uv_run.min_y() + uv_idle.height());

            let body_image = EntityImage::new(
                self.position,
                Rectangle::from_size(Vector([-0.75, 0.0]), Vector([1.5, 2.0])),
                true,
                uv_idle,
            );

            self.appearance = Some(PlayerAppearance {
                uv_idle,
                uv_run,
                body_image: renderer.add_image(body_image),
                frame_timer: 0.0,
            });
        }
    }

    fn update(&mut self, dt: f32, inputs: &InputState, physics: &mut Physics, renderer: &mut EntityRenderer) {
        let mut velocity = Vector::zero();

        if let Some(collider) = &self.collider {
            let collider = physics.get_collider_mut(collider).unwrap();
            velocity = collider.velocity;

            if collider.rectangle.min_y() < -128.0 {
                collider.rectangle.shift_min_to(Vector([-0.375, 0.0]));
                collider.stop();
            }

            if !inputs.key_is_held(Key::Space) {
                self.jump_cooldown = 0.0;
            }
            if self.jump_cooldown <= 0.0 {
                if inputs.key_is_held(Key::Space) && collider.hit_bottom {
                    collider.velocity.set_y(self.jump_speed);
                    self.jump_cooldown += JUMP_COOLDOWN_SECONDS;
                }
            }
            else {
                self.jump_cooldown -= dt;
            }

            if inputs.key_is_held(Key::LeftShift) {
                self.crouching = true;
                collider.rectangle.set_max_y(collider.rectangle.min_y() + 1.4375);
            }
            else {
                self.crouching = false;
                collider.rectangle.set_max_y(collider.rectangle.min_y() + 1.6875);
            }

            let speed_multiplier = if self.crouching && collider.hit_bottom {
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
            let body_image = renderer.get_image_mut(&appearance.body_image);
            body_image.set_position(self.position);

            if velocity.x() == 0.0 {
                body_image.set_uv_base(appearance.uv_idle);
                body_image.set_frame(0);
                appearance.frame_timer = 0.1;
            }
            else {
                body_image.set_uv_base(appearance.uv_run);
                body_image.set_flip_x(velocity.x() < 0.0);
                if appearance.frame_timer <= 0.0 {
                    body_image.set_frame((body_image.frame() + 1) % 6);
                    appearance.frame_timer += 0.1;
                }
                else {
                    appearance.frame_timer -= dt;
                }
            }
        }
    }

    fn destroy(&mut self, physics: &mut Physics, renderer: &mut EntityRenderer) {
        if let Some(collider) = self.collider.take() {
            physics.remove_collider(collider);
        }
        if let Some(appearance) = self.appearance.take() {
            renderer.remove_image(&appearance.body_image);
        }
    }
}
