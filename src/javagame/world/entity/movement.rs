use crate::tools::*;

pub const DEFAULT_TERMINAL_VELOCITY: f32 = 2048.0;

pub const DEFAULT_GRAVITY_ACCELERATION: f32 = 512.0;
pub const DEFAULT_FRICTION_DECELERATION: f32 = 256.0;

pub fn apply_gravity(collider: &mut phys::Collider, dt: f32, acceleration: f32, terminal_velocity: f32) {
    if !collider.fixed {
        if collider.vel.y() - acceleration * dt < -terminal_velocity {
            collider.vel.set_y(-terminal_velocity);
        } else {
            collider.vel.set_y(collider.vel.y() - acceleration * dt);
        }
    }
}

pub fn apply_friction(collider: &mut phys::Collider, dt: f32, deceleration: f32) {
    if !collider.fixed && collider.vel.x() != 0.0 {
        let x_sign = collider.vel.x().signum();
        collider.vel.set_x(collider.vel.x() - x_sign * deceleration * dt);
        if collider.vel.x().signum() != x_sign {
            collider.vel.set_x(0.0);
        }
    }
}