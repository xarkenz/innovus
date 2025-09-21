pub const DEFAULT_TERMINAL_VELOCITY: f32 = 128.0;

pub const DEFAULT_GRAVITY_ACCELERATION: f32 = 32.0;
pub const DEFAULT_FRICTION_DECELERATION: f32 = 16.0;

pub fn apply_gravity(velocity: f32, dt: f32, acceleration: f32, terminal_velocity: f32) -> f32 {
    if velocity - acceleration * dt < -terminal_velocity {
        -terminal_velocity
    }
    else {
        velocity - acceleration * dt
    }
}

pub fn apply_friction(mut velocity: f32, dt: f32, deceleration: f32) -> f32 {
    if velocity != 0.0 {
        let sign = velocity.signum();
        velocity = velocity - sign * deceleration * dt;
        if velocity.signum() != sign {
            velocity = 0.0;
        }
    }
    velocity
}
