use super::*;

use std::collections::BinaryHeap;

// TODO: seems hacky. how else to deal with FP precision?
pub const COLLISION_TOLERANCE: f32 = 1.0e-5;

/// The side/corner of collider 1 which will collide with collider 2.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollisionSide {
    // The order doesn't really matter as long as the corners come last
    /// The left side of collider 1 will collide with the right side of collider 2.
    Left,
    /// The right side of collider 1 will collide with the left side of collider 2.
    Right,
    /// The bottom side of collider 1 will collide with the top side of collider 2.
    Bottom,
    /// The top side of collider 1 will collide with the bottom side of collider 2.
    Top,
    /// The bottom-left corner of collider 1 will collide with the top-right corner of collider 2.
    BottomLeft,
    /// The bottom-right corner of collider 1 will collide with the top-left corner of collider 2.
    BottomRight,
    /// The top-left corner of collider 1 will collide with the bottom-right corner of collider 2.
    TopLeft,
    /// The top-right corner of collider 1 will collide with the bottom-left corner of collider 2.
    TopRight,
}

impl CollisionSide {
    pub fn detect(x_time: f32, y_time: f32, relative_velocity: Vector<f32, 2>) -> Self {
        match x_time.total_cmp(&y_time) {
            Ordering::Less => {
                if relative_velocity.x() > 0.0 { Self::Right } else { Self::Left }
            }
            Ordering::Greater => {
                if relative_velocity.y() > 0.0 { Self::Top } else { Self::Bottom }
            }
            Ordering::Equal => {
                if relative_velocity.x() > 0.0 {
                    if relative_velocity.y() > 0.0 { Self::TopRight } else { Self::BottomRight }
                }
                else {
                    if relative_velocity.y() > 0.0 { Self::TopLeft } else { Self::BottomLeft }
                }
            }
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Top => Self::Bottom,
            Self::BottomLeft => Self::TopRight,
            Self::BottomRight => Self::TopLeft,
            Self::TopLeft => Self::BottomRight,
            Self::TopRight => Self::BottomLeft,
        }
    }
}

#[derive(Debug)]
pub struct Collision {
    pub index_1: usize,
    pub index_2: usize,
    pub time: f32,
    pub side: CollisionSide,
}

impl PartialEq for Collision {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.side == other.side
    }
}

impl Eq for Collision {}

impl PartialOrd for Collision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Since std::collections::BinaryHeap is a max-heap, and we want the earliest collisions
        // processed first, collisions with a greater time are treated as lesser priority
        // (hence the call to reverse()).
        // Comparing the collision side ensures that when there is a tie for time,
        // corner hits are checked after edge hits.
        Some(self.time.partial_cmp(&other.time)?
            .then_with(|| self.side.cmp(&other.side))
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
    pub hit_left: bool,
    pub hit_right: bool,
    pub hit_bottom: bool,
    pub hit_top: bool,
}

impl Collider {
    pub fn new(rectangle: Rectangle<f32>, velocity: Vector<f32, 2>) -> Self {
        Self {
            fixed: false,
            rectangle,
            velocity,
            hit_left: false,
            hit_right: false,
            hit_bottom: false,
            hit_top: false,
        }
    }

    pub fn new_fixed(rect: Rectangle<f32>) -> Self {
        Self {
            fixed: true,
            rectangle: rect,
            velocity: Vector::zero(),
            hit_left: false,
            hit_right: false,
            hit_bottom: false,
            hit_top: false,
        }
    }

    pub fn stop(&mut self) {
        self.velocity = Vector::zero();
    }

    pub fn clear_hit_flags(&mut self) {
        self.hit_left = false;
        self.hit_right = false;
        self.hit_bottom = false;
        self.hit_top = false;
    }

    pub fn set_hit_flags(&mut self, side: CollisionSide) {
        match side {
            CollisionSide::Left => {
                self.hit_left = true;
            }
            CollisionSide::Right => {
                self.hit_right = true;
            }
            CollisionSide::Bottom => {
                self.hit_bottom = true;
            }
            CollisionSide::Top => {
                self.hit_top = true;
            }
            CollisionSide::BottomLeft => {
                self.hit_left = true;
                self.hit_bottom = true;
            }
            CollisionSide::BottomRight => {
                self.hit_right = true;
                self.hit_bottom = true;
            }
            CollisionSide::TopLeft => {
                self.hit_left = true;
                self.hit_top = true;
            }
            CollisionSide::TopRight => {
                self.hit_right = true;
                self.hit_top = true;
            }
        }
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

    pub fn sweep_collision(&self, other: &Self, dt: f32) -> Option<(f32, CollisionSide)> {
        // Determine the time of collision in the x-direction, or infinity if no collision
        let mut x_time = match self.velocity.x().partial_cmp(&other.velocity.x()) {
            Some(Ordering::Less) => (other.rectangle.max_x() - self.rectangle.min_x())
                / (self.velocity.x() - other.velocity.x()),
            Some(Ordering::Greater) => (self.rectangle.max_x() - other.rectangle.min_x())
                / (other.velocity.x() - self.velocity.x()),
            _ => f32::INFINITY
        };
        if -COLLISION_TOLERANCE <= x_time && x_time <= dt {
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
        if -COLLISION_TOLERANCE <= y_time && y_time <= dt {
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
        if -COLLISION_TOLERANCE <= collision_time && collision_time <= dt {
            Some((collision_time, CollisionSide::detect(x_time, y_time, self.velocity - other.velocity)))
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
                let version = version.wrapping_add(1);
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
        // Clear all collider hit flags
        for slot in &mut self.slots {
            if let ColliderSlot::Filled { collider, .. } = slot {
                collider.clear_hit_flags();
            }
        }

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
                collider.velocity = match collision.side {
                    CollisionSide::Left |
                    CollisionSide::Right => {
                        Vector([collision_velocity.x(), collider.velocity.y()])
                    }
                    CollisionSide::Bottom |
                    CollisionSide::Top => {
                        Vector([collider.velocity.x(), collision_velocity.y()])
                    }
                    CollisionSide::BottomLeft |
                    CollisionSide::BottomRight |
                    CollisionSide::TopLeft |
                    CollisionSide::TopRight => {
                        // Subject to change
                        collision_velocity
                    }
                };
                // Set the corresponding hit flags in the collider
                collider.set_hit_flags(if index == collision.index_1 {
                    collision.side
                } else {
                    collision.side.opposite()
                });
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
                            collider_1.sweep_collision(collider_2, dt).map(|(time, side)| Collision {
                                index_1,
                                index_2,
                                time,
                                side,
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
