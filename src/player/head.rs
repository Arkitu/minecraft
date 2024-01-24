use bevy::prelude::*;

pub mod camera;
pub use camera::*;

#[derive(Component)]
pub struct HeadMarker;

#[derive(Bundle)]
pub struct Head {
    marker: HeadMarker,
    cam: Camera3dBundle,
    config: CameraConfig
}
impl Default for Head {
    fn default() -> Self {
        Self {
            marker: HeadMarker,
            cam: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            },
            config: CameraConfig::default()
        }
    }
}
