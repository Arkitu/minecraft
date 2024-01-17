use bevy::prelude::*;
use crate::bloc_and_chunk::SQUARE_UNIT;

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
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                ..Default::default()
            },
            marker: PlayerMarker
        }
    }
}