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
        Rectangle::new(
            self.rect.x() + self.vel.x().min(0.0) * dt,
            self.rect.y() + self.vel.y().min(0.0) * dt,
            self.rect.width() + self.vel.x().abs() * dt,
            self.rect.height() + self.vel.y().abs() * dt,
        )
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ColliderHandle {
    slot: usize,
}

#[derive(Debug)]
enum ColliderSlot {
    Open(Option<usize>),
    Filled(Collider),
}

#[derive(Debug)]
pub struct Physics {
    slots: Vec<ColliderSlot>,
    next_open_slot: Option<usize>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            next_open_slot: None,
        }
    }

    pub fn add(&mut self, collider: Collider) -> ColliderHandle {
        if let Some(slot) = self.next_open_slot {
            if let ColliderSlot::Open(next_open_slot) = self.slots[slot] {
                self.next_open_slot = next_open_slot;
                self.slots[slot] = ColliderSlot::Filled(collider);
                ColliderHandle { slot }
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

    pub fn get(&self, handle: ColliderHandle) -> Option<&Collider> {
        if let Some(ColliderSlot::Filled(collider)) = self.slots.get(handle.slot) {
            Some(collider)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        if let Some(ColliderSlot::Filled(collider)) = self.slots.get_mut(handle.slot) {
            Some(collider)
        } else {
            None
        }
    }

    pub fn remove(&mut self, handle: ColliderHandle) -> Option<Collider> {
        if handle.slot >= self.slots.len() {
            None
        } else {
            match std::mem::replace(
                &mut self.slots[handle.slot],
                ColliderSlot::Open(self.next_open_slot),
            ) {
                ColliderSlot::Filled(collider) => {
                    self.next_open_slot = Some(handle.slot);
                    Some(collider)
                }
                open_slot => {
                    self.slots[handle.slot] = open_slot;
                    None
                }
            }
        }
    }
}
