use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Bundle)]
pub struct Player {
    cam: Camera3dBundle,
    marker: PlayerMarker
}
impl Player {
    pub fn new() -> Self {
        Self {
            cam: Camera3dBundle {
                transform: Transform::from_xyz(40.0, 48.0, 64.0)
                    .looking_at(Vec3::ZERO, Vec3::ZERO),
                
                ..default()
            },
            marker: PlayerMarker
        }
    }
}