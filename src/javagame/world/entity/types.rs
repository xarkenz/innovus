use crate::tools::*;
use innovus::gfx::*;

use super::*;

const JUMP_COOLDOWN_SECONDS: f32 = 0.5;

pub struct Player {
    uuid: Uuid,
    position: Vector<f32, 2>,
    collider: phys::ColliderHandle,
    geometry: Geometry<Vertex2D>,

    name: String,
    facing: f32,
    health: f32,
    crouching: bool,
    spawn_point: Option<Vector<i64, 2>>,
    movement_speed: f32,
    jump_speed: f32,
    jump_cooldown: f32,
}

impl Player {
    fn new(physics: &mut phys::Physics, position: Vector<f32, 2>, name: Option<String>) -> Self {
        Self {
            uuid: generate_uuid(),
            position,
            collider: physics.add_collider(phys::Collider::new(
                Rectangle::from_size(Vector([position.x() - 6.0, position.y()]), Vector([12.0, 27.0])),
                Vector::zero(),
            )),
            geometry: Geometry::new_render().unwrap(),
            name: name.unwrap_or_else(|| String::from(Self::entity_type_str())),
            facing: 0.0,
            health: 100.0,
            crouching: false,
            spawn_point: None,
            movement_speed: 500.0,
            jump_speed: 200.0,
            jump_cooldown: 0.0,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn respawn(&mut self, physics: &mut phys::Physics) {
        let collider = physics.get_collider_mut(&self.collider).unwrap();

        if let Some(Vector([x, y])) = self.spawn_point {
            collider.rectangle.shift_min_x_to(x as f32 - 0.5 * collider.rectangle.width());
            collider.rectangle.shift_min_y_to(y as f32);
            // if not colliding, return
        }

        // TODO
    }
}

impl Entity for Player {
    fn entity_type_str() -> &'static str {
        "player"
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    fn collider(&self) -> &phys::ColliderHandle {
        &self.collider
    }

    fn geometry(&self) -> &Geometry<Vertex2D> {
        &self.geometry
    }

    fn update(&mut self, dt: f32, inputs: &input::InputState, physics: &mut phys::Physics) {
        let collider = physics.get_collider_mut(&self.collider).unwrap();

        if inputs.key_is_held(input::Key::Space) {
            self.jump_cooldown = 0.0;
        }
        if self.jump_cooldown <= 0.0 {
            if inputs.key_is_held(input::Key::Space) /*&& collider.is_grounded()*/ {
                collider.velocity.set_y(self.jump_speed);
                self.jump_cooldown = JUMP_COOLDOWN_SECONDS;
            }
        } else {
            self.jump_cooldown -= dt;
        }

        if inputs.key_is_held(input::Key::Z) {
            self.crouching = true;
            collider.rectangle.set_max_y(collider.rectangle.min_y() + 23.0);
        } else {
            self.crouching = false;
            collider.rectangle.set_max_y(collider.rectangle.min_y() + 27.0);
        }

        let speed_multiplier = if self.crouching /*&& collider.is_grounded()*/ {
            0.5
        } else {
            1.0
        };

        movement::apply_gravity(collider, dt, movement::DEFAULT_GRAVITY_ACCELERATION, movement::DEFAULT_TERMINAL_VELOCITY);
        movement::apply_friction(collider, dt, movement::DEFAULT_FRICTION_DECELERATION);
    }

    fn render(&mut self, dt: f32) {
        self.geometry.clear();
    }
}
