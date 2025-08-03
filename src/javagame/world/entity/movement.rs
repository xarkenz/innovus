use crate::tools::*;

pub const DEFAULT_TERMINAL_VELOCITY: f32 = 128.0;

pub const DEFAULT_GRAVITY_ACCELERATION: f32 = 32.0;
pub const DEFAULT_FRICTION_DECELERATION: f32 = 16.0;

pub fn apply_gravity(collider: &mut phys::Collider, dt: f32, acceleration: f32, terminal_velocity: f32) {
    if !collider.fixed {
        if collider.velocity.y() - acceleration * dt < -terminal_velocity {
            collider.velocity.set_y(-terminal_velocity);
        }
        else {
            collider.velocity.set_y(collider.velocity.y() - acceleration * dt);
        }
    }
}

pub fn apply_friction(collider: &mut phys::Collider, dt: f32, deceleration: f32) {
    if !collider.fixed && collider.velocity.x() != 0.0 {
        let x_sign = collider.velocity.x().signum();
        collider.velocity.set_x(collider.velocity.x() - x_sign * deceleration * dt);
        if collider.velocity.x().signum() != x_sign {
            collider.velocity.set_x(0.0);
        }
    }
}
