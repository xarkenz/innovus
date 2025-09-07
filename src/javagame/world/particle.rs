use innovus::gfx::color::RGBColor;
use innovus::gfx::{Geometry, Vertex2D};
use innovus::tools::Vector;
use crate::tools::noise::scramble;

pub fn pseudo_random() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    scramble(nanos as u64)
}

pub fn random_unit_vector() -> Vector<f32, 2> {
    let theta = pseudo_random() as f32 / u64::MAX as f32 * std::f32::consts::TAU;
    Vector([theta.cos(), theta.sin()])
}

pub fn choose_random<T>(elements: &[T]) -> Option<&T> {
    (!elements.is_empty()).then(|| {
        &elements[pseudo_random() as usize % elements.len()]
    })
}

pub struct ParticleInfo {
    pub position: Vector<f32, 2>,
    pub velocity: Vector<f32, 2>,
    pub acceleration: Vector<f32, 2>,
    pub lifetime: f32,
    pub color: RGBColor,
    pub size: f32,
}

impl Default for ParticleInfo {
    fn default() -> Self {
        Self {
            position: Vector::zero(),
            velocity: Vector::zero(),
            acceleration: Vector([0.0, -32.0]),
            lifetime: 1.0,
            color: RGBColor::black(),
            size: 1.0,
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
        let mut triangles = Vec::new();
        for particle in &self.active_particles {
            let index = vertices.len() as u32;
            let opacity = particle.lifetime.min(1.0);
            for offset in OFFSETS {
                vertices.push(Vertex2D::new(
                    (particle.position + offset * particle.size / 16.0).with_z(0.0),
                    Some(particle.color.0.with_w(opacity)),
                    None,
                ));
            }
            triangles.push([index + 0, index + 1, index + 2]);
            triangles.push([index + 2, index + 3, index + 0]);
        }

        if !vertices.is_empty() {
            self.geometry.add(&vertices, &triangles);
            self.geometry.render();
        }
    }
}
