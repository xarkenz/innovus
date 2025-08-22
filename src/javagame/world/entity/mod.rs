use crate::tools::*;

pub mod movement;
pub mod types;
pub mod render;

pub trait Entity {
    fn entity_type(&self) -> &'static str;

    fn uuid(&self) -> Uuid;

    fn position(&self) -> Vector<f32, 2>;

    fn collider(&self) -> Option<&phys::ColliderHandle> {
        // No collision by default
        None
    }

    fn init_collision(&mut self, physics: &mut phys::Physics) {
        // Do nothing by default
        let _ = physics;
    }

    fn init_appearance(&mut self, assets: &mut asset::AssetPool, renderer: &mut render::EntityRenderer) {
        // Do nothing by default
        let _ = (assets, renderer);
    }

    fn update(&mut self, dt: f32, inputs: &input::InputState, physics: &mut phys::Physics, renderer: &mut render::EntityRenderer);

    fn destroy(&mut self, physics: &mut phys::Physics, renderer: &mut render::EntityRenderer) {
        // Do nothing by default
        let _ = (physics, renderer);
    }
}
