use std::hash::{Hash, Hasher};
use fnv::FnvHasher;
use innovus::tools::Vector;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

pub fn smooth_step(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    (b - a) * (3.0 - 2.0 * t) * t * t + a
}

pub fn scramble(mut n: u64) -> u64 {
    n ^= n >> 33;
    n = n.wrapping_mul(0xFF51AFD7ED558CCD);
    n ^= n >> 33;
    n = n.wrapping_mul(0xC4CEB9FE1A85EC53);
    n ^= n >> 33;
    n
}

#[derive(Default)]
pub struct SimpleHasher {
    state: FnvHasher,
}

impl SimpleHasher {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            state: FnvHasher::with_key(seed),
        }
    }
}

impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.state.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.state.write(bytes);
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_be_bytes());
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_be_bytes());
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_be_bytes());
    }

    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_be_bytes());
    }

    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_be_bytes());
    }
}

#[derive(Debug)]
pub struct PerlinNoise1D {
    base_seed: u64,
}

impl PerlinNoise1D {
    pub fn new(base_seed: u64) -> Self {
        Self { base_seed }
    }

    pub fn base_seed(&self) -> u64 {
        self.base_seed
    }

    pub fn get_cell_seed(&self, cell: i64) -> u64 {
        let mut hasher = FnvHasher::with_key(self.base_seed);
        cell.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_cell(&self, cell: i64) -> PerlinNoiseCell1D {
        let seed_0 = self.get_cell_seed(cell);
        let seed_1 = self.get_cell_seed(cell + 1);

        let [grad_0, grad_1] = [seed_0, seed_1].map(|seed| {
            let theta = scramble(seed) as f32 / u64::MAX as f32 * std::f32::consts::TAU;
            theta.cos()
        });

        PerlinNoiseCell1D {
            grad_0,
            grad_1,
        }
    }
}

#[derive(Debug)]
pub struct PerlinNoiseCell1D {
    grad_0: f32,
    grad_1: f32,
}

impl PerlinNoiseCell1D {
    pub fn compute_value<F>(&self, offset: f32, interpolate: F) -> f32
    where
        F: Fn(f32, f32, f32) -> f32,
    {
        let offset_0 = offset;
        let offset_1 = offset - 1.0;

        let dot_0 = self.grad_0 * offset_0;
        let dot_1 = self.grad_1 * offset_1;

        interpolate(dot_0, dot_1, offset)
    }
}

#[derive(Debug)]
pub struct PerlinNoise2D {
    base_seed: u64,
}

impl PerlinNoise2D {
    pub fn new(base_seed: u64) -> Self {
        Self { base_seed }
    }

    pub fn base_seed(&self) -> u64 {
        self.base_seed
    }

    pub fn get_cell_seed(&self, cell: Vector<i64, 2>) -> u64 {
        let mut hasher = SimpleHasher::with_seed(self.base_seed);
        cell.x().hash(&mut hasher);
        cell.y().hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_cell(&self, cell: Vector<i64, 2>) -> PerlinNoiseCell2D {
        let seed_00 = self.get_cell_seed(cell);
        let seed_01 = self.get_cell_seed(cell + Vector([0, 1]));
        let seed_10 = self.get_cell_seed(cell + Vector([1, 0]));
        let seed_11 = self.get_cell_seed(cell + Vector([1, 1]));

        let [grad_00, grad_01, grad_10, grad_11] = [seed_00, seed_01, seed_10, seed_11].map(|seed| {
            let theta = scramble(seed) as f32 / u64::MAX as f32 * std::f32::consts::TAU;
            Vector([theta.cos(), theta.sin()])
        });

        PerlinNoiseCell2D {
            grad_00,
            grad_01,
            grad_10,
            grad_11,
        }
    }
}

#[derive(Debug)]
pub struct PerlinNoiseCell2D {
    pub grad_00: Vector<f32, 2>,
    pub grad_01: Vector<f32, 2>,
    pub grad_10: Vector<f32, 2>,
    pub grad_11: Vector<f32, 2>,
}

impl PerlinNoiseCell2D {
    pub fn compute_value<F>(&self, offset: Vector<f32, 2>, interpolate: F) -> f32
    where
        F: Fn(f32, f32, f32) -> f32,
    {
        let offset_00 = offset;
        let offset_01 = offset - Vector([0.0, 1.0]);
        let offset_10 = offset - Vector([1.0, 0.0]);
        let offset_11 = offset - Vector([1.0, 1.0]);

        let dot_00 = self.grad_00.dot(offset_00);
        let dot_01 = self.grad_01.dot(offset_01);
        let dot_10 = self.grad_10.dot(offset_10);
        let dot_11 = self.grad_11.dot(offset_11);

        interpolate(
            interpolate(dot_00, dot_10, offset.x()),
            interpolate(dot_01, dot_11, offset.x()),
            offset.y(),
        )
    }
}
