use innovus::{gfx::*, tools::*};

pub mod types;

pub trait Entity {
    fn new(physics: &mut phys::Physics, position: Vector<f32, 2>) -> Self
    where
        Self: Sized;

    fn entity_type(&self) -> &'static str;
    fn uuid(&self) -> Uuid;
    fn position(&self) -> Vector<f32, 2>;
    fn collider(&self) -> phys::ColliderHandle;
    fn geometry(&self) -> &Geometry<Vertex2D>;

    fn update(&mut self, physics: &mut phys::Physics, dt: f32);
    fn render(&self);
}
