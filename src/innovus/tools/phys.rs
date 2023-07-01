use super::*;

#[derive(Clone, Debug)]
pub struct Collider {
    pub fixed: bool,
    pub rect: Rectangle<f32>,
    pub vel: Vector<f32, 2>,
}

impl Collider {
    pub fn new(rect: Rectangle<f32>, vel: Vector<f32, 2>) -> Self {
        Self {
            fixed: false,
            rect,
            vel,
        }
    }

    pub fn new_fixed(rect: Rectangle<f32>) -> Self {
        Self {
            fixed: true,
            rect,
            vel: Vector::zero(),
        }
    }

    pub fn stop(&mut self) {
        self.vel = Vector::zero();
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.rect.intersects(&other.rect)
    }

    pub fn broad_phase(&self, dt: f32) -> Rectangle<f32> {
        if self.fixed {
            self.rect.clone()
        } else {
            Rectangle::new(
                self.rect.x() + self.vel.x().min(0.0) * dt,
                self.rect.y() + self.vel.y().min(0.0) * dt,
                self.rect.width() + self.vel.x().abs() * dt,
                self.rect.height() + self.vel.y().abs() * dt,
            )
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ColliderHandle {
    slot: usize,
}

#[derive(Debug)]
enum ColliderSlot {
    Open(Option<usize>),
    Filled(Collider),
}

impl ColliderSlot {
    fn collider(&self) -> Option<&Collider> {
        if let Self::Filled(collider) = self {
            Some(collider)
        } else {
            None
        }
    }

    fn collider_mut(&mut self) -> Option<&mut Collider> {
        if let Self::Filled(collider) = self {
            Some(collider)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Collision {
    index_1: usize,
    index_2: usize,
    time: f32,
}

#[derive(Debug)]
pub struct Physics {
    slots: Vec<ColliderSlot>,
    next_open_slot_index: Option<usize>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            next_open_slot_index: None,
        }
    }

    pub fn add_collider(&mut self, collider: Collider) -> ColliderHandle {
        if let Some(next_open_slot_index) = self.next_open_slot_index {
            if let ColliderSlot::Open(next_open_slot) = self.slots[next_open_slot_index] {
                self.next_open_slot_index = next_open_slot;
                self.slots[next_open_slot_index] = ColliderSlot::Filled(collider);
                ColliderHandle { slot: next_open_slot_index }
            } else {
                panic!("physics collider data found in slot marked as open")
            }
        } else {
            let handle = ColliderHandle {
                slot: self.slots.len(),
            };
            self.slots.push(ColliderSlot::Filled(collider));
            handle
        }
    }

    pub fn get_collider(&self, handle: &ColliderHandle) -> Option<&Collider> {
        self.slots.get(handle.slot).and_then(|slot| slot.collider())
    }

    pub fn get_collider_mut(&mut self, handle: &ColliderHandle) -> Option<&mut Collider> {
        self.slots.get_mut(handle.slot).and_then(|slot| slot.collider_mut())
    }

    pub fn remove_collider(&mut self, handle: ColliderHandle) -> Option<Collider> {
        if handle.slot >= self.slots.len() {
            None
        } else {
            match std::mem::replace(
                &mut self.slots[handle.slot],
                ColliderSlot::Open(self.next_open_slot_index),
            ) {
                ColliderSlot::Filled(collider) => {
                    self.next_open_slot_index = Some(handle.slot);
                    Some(collider)
                }
                open_slot => {
                    self.slots[handle.slot] = open_slot;
                    None
                }
            }
        }
    }

    pub fn step_simulation(&mut self, dt: f32) {
        let pairs = self.possible_collision_pairs(dt);
        for (index_1, index_2) in pairs {
            self.resolve_collision(index_1, index_2);
        }
    }

    fn possible_collision_pairs(&self, dt: f32) -> Vec<(usize, usize)> {
        let indexed_broad_phases = self.slots.iter()
            .enumerate()
            .filter_map(move |(index, slot)| slot.collider()
                .map(|collider| (index, collider.broad_phase(dt))));

        indexed_broad_phases.clone().flat_map(move |(index_1, broad_phase_1)| {
            indexed_broad_phases.clone().filter_map(move |(index_2, broad_phase_2)| {
                // index_1 < index_2 ensures that:
                //   a) no collider is checked against itself
                //   b) there is only one entry for each pair
                if index_1 < index_2 && broad_phase_1.intersects(&broad_phase_2) {
                    Some((index_1, index_2))
                } else {
                    None
                }
            })
        }).collect()
    }

    fn resolve_collision(&mut self, index_1: usize, index_2: usize) {
        let collider_1 = self.slots[index_1].collider().unwrap();
        let collider_2 = self.slots[index_2].collider().unwrap();
    }
}
