use bevy::prelude::*;

#[derive(Bundle)]
pub struct Player {
    cam: Camera3dBundle,
    pos: Transform
}
impl Player {
    pub fn new() -> Self {
        Self {
            cam: Camera3dBundle::default(),
            pos: Transform::from_xyz(40.0, 48.0, 64.0)
                .looking_at(Vec3::ZERO, Vec3::ZERO)
        }
    }
}