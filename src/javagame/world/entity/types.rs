use innovus::{gfx::*, tools::*};

use super::*;

pub struct Player {
    uuid: Uuid,
    position: Vector<f32, 2>,
    collider: phys::ColliderHandle,
    geometry: Geometry<Vertex2D>,

    facing: f32,
    health: f32,
    crouching: bool,
    spawn_point: Option<Vector<i64, 2>>,
}

impl Entity for Player {
    fn new(physics: &mut phys::Physics, position: Vector<f32, 2>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            position,
            collider: physics.add(phys::Collider::new(
                Rectangle::new(position.x() - 6.0, position.y(), 12.0, 27.0),
                Vector::zero(),
            )),
            geometry: Geometry::new_render().unwrap(),
            facing: 0.0,
            health: 100.0,
            crouching: false,
            spawn_point: None,
        }
    }

    fn entity_type(&self) -> &'static str {
        "player"
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn position(&self) -> Vector<f32, 2> {
        self.position
    }

    fn collider(&self) -> phys::ColliderHandle {
        self.collider
    }

    fn geometry(&self) -> &Geometry<Vertex2D> {
        &self.geometry
    }

    fn update(&mut self, physics: &mut phys::Physics, dt: f32) {}

    fn render(&self) {}
}
