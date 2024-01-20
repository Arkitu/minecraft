use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::bloc_and_chunk::SQUARE_UNIT;

mod camera;
pub use camera::{*, Camera};

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Bundle)]
pub struct Player {
    collider: Collider,
    marker: PlayerMarker,
    spatial: SpatialBundle
}
impl Player {
    pub fn new() -> Self {
        Self {
            collider: Collider::cuboid(SQUARE_UNIT/3.0, SQUARE_UNIT*0.9, SQUARE_UNIT/3.0),
            marker: PlayerMarker,
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0.0, 5.0, 0.0))
        }
    }
    pub fn spawn(cmds: &mut Commands) {
        cmds.spawn(Self::new())
            .with_children(|parent| {
                Camera::spawn(parent);
            });
    }
}


