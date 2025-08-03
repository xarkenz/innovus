use super::*;

use std::collections::BinaryHeap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollisionKind {
    // The ordinal values are important for ordering
    LeftRight = 0,
    TopBottom = 1,
    Corner = 2,
}

impl CollisionKind {
    pub fn from_times(x_time: f32, y_time: f32) -> Self {
        match x_time.total_cmp(&y_time) {
            Ordering::Less => Self::LeftRight,
            Ordering::Greater => Self::TopBottom,
            Ordering::Equal => Self::Corner,
        }
    }
}

#[derive(Debug)]
pub struct Collision {
    pub index_1: usize,
    pub index_2: usize,
    pub time: f32,
    pub kind: CollisionKind,
}

impl PartialEq for Collision {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.kind == other.kind
    }
}

impl Eq for Collision {}

impl PartialOrd for Collision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Since std::collections::BinaryHeap is a max-heap, and we want the earliest collisions
        // processed first, collisions with a greater time are treated as lesser priority
        // (hence the call to reverse()).
        // Comparing the collision kind ensures that when there is a tie for time,
        // corner hits are checked after edge hits.
        Some(self.time.partial_cmp(&other.time)?
            .then_with(|| self.kind.cmp(&other.kind))
            .reverse())
    }
}

impl Ord for Collision {
    fn cmp(&self, other: &Self) -> Ordering {
        // This shouldn't be problematic because self.time should be in the interval [0, 1]
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct Collider {
    pub fixed: bool,
    pub rectangle: Rectangle<f32>,
    pub velocity: Vector<f32, 2>,
}

impl Collider {
    pub fn new(rectangle: Rectangle<f32>, velocity: Vector<f32, 2>) -> Self {
        Self {
            fixed: false,
            rectangle,
            velocity,
        }
    }

    pub fn new_fixed(rect: Rectangle<f32>) -> Self {
        Self {
            fixed: true,
            rectangle: rect,
            velocity: Vector::zero(),
        }
    }

    pub fn stop(&mut self) {
        self.velocity = Vector::zero();
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.rectangle.intersects(&other.rectangle)
    }

    pub fn broad_phase(&self, dt: f32) -> Rectangle<f32> {
        let mut phase = self.rectangle;
        if !self.fixed {
            phase.expand_toward(self.velocity * dt);
        }
        phase
    }

    pub fn sweep_collision(&self, other: &Self, dt: f32) -> Option<(f32, CollisionKind)> {
        // Determine the time of collision in the x-direction, or infinity if no collision
        let mut x_time = match self.velocity.x().partial_cmp(&other.velocity.x()) {
            Some(Ordering::Less) => (other.rectangle.max_x() - self.rectangle.min_x())
                / (self.velocity.x() - other.velocity.x()),
            Some(Ordering::Greater) => (self.rectangle.max_x() - other.rectangle.min_x())
                / (other.velocity.x() - self.velocity.x()),
            _ => f32::INFINITY
        };
        if 0.0 <= x_time && x_time <= dt {
            // Ensure that collision occurs in the y-direction as well
            let self_dy = self.velocity.y() * x_time;
            let other_dy = other.velocity.y() * x_time;
            if self.rectangle.min_y() + self_dy >= other.rectangle.max_y() + other_dy
                || self.rectangle.max_y() + self_dy <= other.rectangle.min_y() + other_dy
            {
                x_time = f32::INFINITY;
            }
        }
        else {
            x_time = f32::INFINITY;
        }

        // Determine the time of collision in the y-direction, or infinity if no collision
        let mut y_time = match self.velocity.y().partial_cmp(&other.velocity.y()) {
            Some(Ordering::Less) => (other.rectangle.max_y() - self.rectangle.min_y())
                / (self.velocity.y() - other.velocity.y()),
            Some(Ordering::Greater) => (self.rectangle.max_y() - other.rectangle.min_y())
                / (other.velocity.y() - self.velocity.y()),
            _ => f32::NAN
        };
        if 0.0 <= y_time && y_time <= dt {
            // Ensure that collision occurs in the x-direction as well
            let self_dx = self.velocity.x() * y_time;
            let other_dx = other.velocity.x() * y_time;
            if self.rectangle.min_x() + self_dx >= other.rectangle.max_x() + other_dx
                || self.rectangle.max_x() + self_dx <= other.rectangle.min_x() + other_dx
            {
                y_time = f32::INFINITY;
            }
        }
        else {
            y_time = f32::INFINITY;
        }

        // Determine which direction collides first, if any
        let collision_time = x_time.min(y_time);
        if 0.0 <= collision_time && collision_time <= dt {
            Some((collision_time, CollisionKind::from_times(x_time, y_time)))
        }
        else {
            None
        }
    }

    pub fn collision_velocity(&self, other: &Self) -> Vector<f32, 2> {
        if self.fixed || other.fixed {
            Vector::zero()
        }
        else {
            (self.velocity + other.velocity) * 0.5
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ColliderHandle {
    slot: usize,
    version: usize,
}

#[derive(Debug)]
enum ColliderSlot {
    Open {
        next_open_slot_index: Option<usize>,
        version: usize,
    },
    Filled {
        collider: Collider,
        version: usize,
    },
}

impl ColliderSlot {
    fn current_collider(&self) -> Option<&Collider> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { collider, .. } => Some(collider),
        }
    }

    fn collider(&self, handle_version: usize) -> Option<&Collider> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { collider, version } => {
                (handle_version == *version).then_some(collider)
            },
        }
    }

    fn current_collider_mut(&mut self) -> Option<&mut Collider> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { collider, .. } => Some(collider),
        }
    }

    fn collider_mut(&mut self, handle_version: usize) -> Option<&mut Collider> {
        match self {
            Self::Open { .. } => None,
            Self::Filled { collider, version } => {
                (handle_version == *version).then_some(collider)
            },
        }
    }
}

#[derive(Debug)]
pub struct Physics {
    slots: Vec<ColliderSlot>,
    first_open_slot_index: Option<usize>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            first_open_slot_index: None,
        }
    }

    pub fn add_collider(&mut self, collider: Collider) -> ColliderHandle {
        if let Some(first_open_slot_index) = self.first_open_slot_index {
            if let ColliderSlot::Open { next_open_slot_index, version } = self.slots[first_open_slot_index] {
                let version = version + 1;
                self.first_open_slot_index = next_open_slot_index;
                self.slots[first_open_slot_index] = ColliderSlot::Filled {
                    collider,
                    version,
                };
                ColliderHandle {
                    slot: first_open_slot_index,
                    version,
                }
            }
            else {
                unreachable!("physics collider data found in slot marked as open")
            }
        }
        else {
            let handle = ColliderHandle {
                slot: self.slots.len(),
                version: 0,
            };
            self.slots.push(ColliderSlot::Filled {
                collider,
                version: 0,
            });
            handle
        }
    }

    pub fn get_collider(&self, handle: &ColliderHandle) -> Option<&Collider> {
        self.slots.get(handle.slot).and_then(|slot| slot.collider(handle.version))
    }

    pub fn get_collider_mut(&mut self, handle: &ColliderHandle) -> Option<&mut Collider> {
        self.slots.get_mut(handle.slot).and_then(|slot| slot.collider_mut(handle.version))
    }

    pub fn remove_collider(&mut self, handle: ColliderHandle) -> Option<Collider> {
        if handle.slot >= self.slots.len() {
            None
        }
        else if let ColliderSlot::Filled { version, .. } = self.slots[handle.slot] {
            if handle.version != version {
                None
            }
            else if let ColliderSlot::Filled { collider, .. } = std::mem::replace(
                &mut self.slots[handle.slot],
                ColliderSlot::Open {
                    next_open_slot_index: self.first_open_slot_index,
                    version,
                },
            ) {
                self.first_open_slot_index = Some(handle.slot);
                Some(collider)
            }
            else {
                unreachable!()
            }
        }
        else {
            None
        }
    }

    pub fn step_simulation(&mut self, dt: f32) {
        // FIXME: something is still a little broken here. objects sometimes clip through each other
        // Keep track of how much time has been used by advancing to collision sites
        let mut time_used = Vec::with_capacity(self.slots.len());
        time_used.resize(self.slots.len(), 0.0);

        // Incrementally handle all swept collisions, earliest first
        let mut collisions = self.get_collisions(dt);
        while let Some(collision) = collisions.pop() {
            let collider_1 = self.slots[collision.index_1].current_collider().unwrap();
            let collider_2 = self.slots[collision.index_2].current_collider().unwrap();

            // Double-check that there is still a broad phase intersection between the colliders.
            // This is necessary because the colliders' velocities may have changed since the
            // initial sweep (due to previous collisions).
            if !collider_1.broad_phase(dt).intersects(&collider_2.broad_phase(dt)) {
                continue;
            }

            let collision_velocity = collider_1.collision_velocity(collider_2);

            for index in [collision.index_1, collision.index_2] {
                let collider = self.slots[index].current_collider_mut().unwrap();
                // Advance to the collision site
                collider.rectangle.shift_by(collider.velocity * (collision.time - time_used[index]));
                // Record the amount of time passed once collision site is reached
                time_used[index] = collision.time;
                // Set the post-collision velocity of the collider
                collider.velocity = match collision.kind {
                    CollisionKind::LeftRight => Vector([
                        collision_velocity.x(),
                        collider.velocity.y(),
                    ]),
                    CollisionKind::TopBottom => Vector([
                        collider.velocity.x(),
                        collision_velocity.y(),
                    ]),
                    CollisionKind::Corner => collision_velocity, // Subject to change
                };
            }
        }

        // Advance all colliders to their final position
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if let Some(collider) = slot.current_collider_mut() {
                collider.rectangle.shift_by(collider.velocity * (dt - time_used[index]));
            }
        }
    }

    fn get_collisions(&self, dt: f32) -> BinaryHeap<Collision> {
        // TODO: this is a fairly naive approach
        let indexed_colliders = self.slots.iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.current_collider()
                .map(|collider| (index, collider)));

        indexed_colliders.clone()
            .enumerate()
            .flat_map(move |(checked_count, (index_1, collider_1))| {
                // Skipping checked_count + 1 elements starts the below loop right after this one
                // within indexed_colliders, which ensures that:
                // a) no collider is checked against itself
                // b) there is only one check performed on each pair
                // c) for each pair (index_1, index_2) it holds that index_1 < index_2
                indexed_colliders.clone()
                    .skip(checked_count + 1)
                    .filter_map(move |(index_2, collider_2)| {
                        // Before checking for actual collision between a pair, we will first
                        // check if there is broad phase intersection. The broad phase is a
                        // rectangle encompassing the projected motion of a collider, so if their
                        // broad phases don't intersect, we can discard the pair.
                        if collider_1.broad_phase(dt).intersects(&collider_2.broad_phase(dt)) {
                            // Now, check for an actual collision between the two colliders
                            collider_1.sweep_collision(collider_2, dt).map(|(time, kind)| Collision {
                                index_1,
                                index_2,
                                time,
                                kind,
                            })
                        }
                        else {
                            None
                        }
                    })
            })
            .collect()
    }
}
