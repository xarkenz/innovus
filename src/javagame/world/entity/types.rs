use crate::tools::*;
use innovus::gfx::*;
use super::*;

const JUMP_COOLDOWN_SECONDS: f32 = 0.5;

pub struct Player {
    uuid: Uuid,
    position: Vector<f32, 2>,
    collider: Option<phys::ColliderHandle>,
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
    pub fn new(physics: &mut phys::Physics, position: Vector<f32, 2>, name: Option<String>) -> Self {
        Self {
            uuid: generate_uuid(),
            position,
            collider: Some(physics.add_collider(phys::Collider::new(
                Rectangle::from_size(Vector([position.x() - 6.0, position.y()]), Vector([12.0, 27.0])),
                Vector::zero(),
            ))),
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
        let collider = physics.get_collider_mut(self.collider.as_ref().unwrap()).unwrap();

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

    fn collider(&self) -> Option<&phys::ColliderHandle> {
        self.collider.as_ref()
    }

    fn geometry(&self) -> &Geometry<Vertex2D> {
        &self.geometry
    }

    fn update(&mut self, dt: f32, inputs: &input::InputState, physics: &mut phys::Physics) {
        let collider = physics.get_collider_mut(self.collider.as_ref().unwrap()).unwrap();

        if inputs.key_is_held(input::Key::Space) {
            self.jump_cooldown = 0.0;
        }
        if self.jump_cooldown <= 0.0 {
            if inputs.key_is_held(input::Key::Space) /*&& collider.is_grounded()*/ {
                collider.velocity.set_y(self.jump_speed);
                self.jump_cooldown = JUMP_COOLDOWN_SECONDS;
            }
        }
        else {
            self.jump_cooldown -= dt;
        }

        if inputs.key_is_held(input::Key::Z) {
            self.crouching = true;
            collider.rectangle.set_max_y(collider.rectangle.min_y() + 23.0);
        }
        else {
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

    fn destroy(&mut self, physics: &mut phys::Physics) {
        if let Some(collider) = self.collider.take() {
            physics.remove_collider(collider);
        }
    }
}

pub struct TestBox {
    uuid: Uuid,
    position: Vector<f32, 2>,
    collider: Option<phys::ColliderHandle>,
    geometry: Geometry<Vertex2D>,

    controllable: bool,
    initial_rectangle: Rectangle<f32>,
    initial_velocity: Vector<f32, 2>,
    rectangle: Rectangle<f32>,
    color: [f32; 4],
}

impl TestBox {
    pub fn new(physics: &mut phys::Physics, collider: phys::Collider, color: [f32; 4], controllable: bool) -> Self {
        Self {
            uuid: generate_uuid(),
            position: collider.rectangle.center(),
            initial_rectangle: collider.rectangle,
            initial_velocity: collider.velocity,
            rectangle: collider.rectangle,
            collider: Some(physics.add_collider(collider)),
            geometry: Geometry::new_render().unwrap(),
            controllable,
            color,
        }
    }

    pub fn respawn(&mut self, physics: &mut phys::Physics) {
        let collider = physics.get_collider_mut(self.collider.as_ref().unwrap()).unwrap();

        collider.rectangle = self.initial_rectangle;
        collider.velocity = self.initial_velocity;
        self.rectangle = self.initial_rectangle;
    }
}

impl Entity for TestBox {
    fn entity_type_str() -> &'static str {
        "test_box"
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

    fn geometry(&self) -> &Geometry<Vertex2D> {
        &self.geometry
    }

    fn update(&mut self, dt: f32, inputs: &input::InputState, physics: &mut phys::Physics) {
        if inputs.key_is_held(input::Key::Enter) {
            self.respawn(physics);
        }
        else {
            let collider = physics.get_collider_mut(self.collider.as_ref().unwrap()).unwrap();

            if self.controllable {
                if inputs.key_is_held(input::Key::Left) {
                    collider.velocity.set_x(collider.velocity.x() - 5.0 * dt);
                }
                if inputs.key_is_held(input::Key::Right) {
                    collider.velocity.set_x(collider.velocity.x() + 5.0 * dt);
                }
                if inputs.key_is_held(input::Key::Down) {
                    collider.velocity.set_y(collider.velocity.y() - 5.0 * dt);
                }
                if inputs.key_is_held(input::Key::Up) {
                    collider.velocity.set_y(collider.velocity.y() + 5.0 * dt);
                }
            }

            self.rectangle = collider.rectangle;
        }
    }

    fn render(&mut self, dt: f32) {
        self.geometry.clear();
        self.geometry.add(&[
            Vertex2D::new([self.rectangle.min_x(), self.rectangle.min_y(), 1.0], Some(self.color), None),
            Vertex2D::new([self.rectangle.min_x(), self.rectangle.max_y(), 1.0], Some(self.color), None),
            Vertex2D::new([self.rectangle.max_x(), self.rectangle.max_y(), 1.0], Some(self.color), None),
            Vertex2D::new([self.rectangle.max_x(), self.rectangle.min_y(), 1.0], Some(self.color), None),
        ], &[
            [0, 1, 2],
            [2, 3, 0],
        ]);
        self.geometry.render();
    }

    fn destroy(&mut self, physics: &mut phys::Physics) {
        if let Some(collider) = self.collider.take() {
            physics.remove_collider(collider);
        }
    }
}
