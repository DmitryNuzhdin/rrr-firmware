use bevy::math::{Quat, Vec3};

struct Rocket {
    mass: f32,
    moments_of_inertia: Vec3
}

struct Environment {
    g: f32,
    wind: Vec3
}

struct RocketState {
    position: Vec3,
    velocity: Vec3,
    orientation: Quat
}