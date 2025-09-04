use innovus::gfx::color::RGBColor;
use innovus::gfx::{Geometry, Vertex2D};
use innovus::tools::Vector;

pub struct ParticleInfo {
    pub position: Vector<f32, 2>,
    pub velocity: Vector<f32, 2>,
    pub acceleration: Vector<f32, 2>,
    pub lifetime: f32,
    pub color: RGBColor,
}

impl Default for ParticleInfo {
    fn default() -> Self {
        Self {
            position: Vector::zero(),
            velocity: Vector::zero(),
            acceleration: Vector([0.0, -32.0]),
            lifetime: 1.0,
            color: RGBColor::black(),
        }
    }
}

pub struct ParticleManager {
    active_particles: Vec<ParticleInfo>,
    geometry: Geometry<Vertex2D>,
}

impl ParticleManager {
    pub fn new() -> Self {
        Self {
            active_particles: Vec::new(),
            geometry: Geometry::new_render().unwrap(),
        }
    }

    pub fn create_particle(&mut self, info: ParticleInfo) {
        self.active_particles.push(info);
    }

    pub fn update(&mut self, dt: f32) {
        self.active_particles.retain_mut(|particle| {
            particle.lifetime -= dt;
            if particle.lifetime <= 0.0 {
                false
            }
            else {
                particle.position += particle.velocity * dt;
                particle.velocity += particle.acceleration * dt;
                true
            }
        })
    }

    pub fn render(&mut self) {
        const OFFSETS: [Vector<f32, 2>; 4] = [
            Vector([0.0, 0.0]), // Bottom left
            Vector([0.0, 1.0]), // Top left
            Vector([1.0, 1.0]), // Top right
            Vector([1.0, 0.0]), // Bottom right
        ];

        self.geometry.clear();
        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        for particle in &self.active_particles {
            let index = vertices.len() as u32;
            let opacity = particle.lifetime.min(1.0);
            for offset in OFFSETS {
                let position = particle.position + offset / 16.0;
                vertices.push(Vertex2D::new(
                    [position.x(), position.y(), 0.0],
                    Some([particle.color.r(), particle.color.g(), particle.color.b(), opacity]),
                    None,
                ));
            }
            faces.push([index + 0, index + 1, index + 2]);
            faces.push([index + 2, index + 3, index + 0]);
        }
        self.geometry.add(&vertices, &faces);
        self.geometry.render();
    }
}
