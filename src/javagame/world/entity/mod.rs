use crate::tools::*;
use innovus::gfx::*;

pub mod movement;
pub mod types;

pub trait Entity {
    fn entity_type_str() -> &'static str where Self: Sized;

    fn entity_type(&self) -> &'static str where Self: Sized {
        Self::entity_type_str()
    }

    fn uuid(&self) -> Uuid;
    fn position(&self) -> Vector<f32, 2>;
    fn collider(&self) -> Option<&phys::ColliderHandle>;
    fn geometry(&self) -> &Geometry<Vertex2D>;

    fn update(&mut self, dt: f32, inputs: &input::InputState, physics: &mut phys::Physics);
    fn render(&mut self, dt: f32);

    fn destroy(&mut self, physics: &mut phys::Physics);
}
