use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod camera;
pub use camera::{*, Camera};

const SPEED: f32 = 0.3;

#[derive(Component)]
pub struct PlayerMarker;

#[derive(Component)]
pub struct PlayerKeys {
    forward: KeyCode,
    backward: KeyCode,
    right: KeyCode,
    left: KeyCode,
    jump: KeyCode
}
impl Default for PlayerKeys {
    fn default() -> Self {
        Self {
            forward: KeyCode::Z,
            backward: KeyCode::S,
            right: KeyCode::D,
            left: KeyCode::Q,
            jump: KeyCode::Space
        }
    }
}

#[derive(Bundle)]
pub struct Player {
    collider: Collider,
    marker: PlayerMarker,
    spatial: SpatialBundle,
    controller: KinematicCharacterController,
    keys: PlayerKeys
}
impl Player {
    pub fn new() -> Self {
        Self {
            collider: Collider::cylinder(0.9, 1.0/3.0),
            marker: PlayerMarker,
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0.0, 4.5, 0.0)),
            controller: KinematicCharacterController::default(),
            keys: PlayerKeys::default()
        }
    }
    pub fn spawn(cmds: &mut Commands) {
        cmds.spawn(Self::new())
            .with_children(|parent| {
                Camera::spawn(parent);
            });
    }
}

pub fn move_player(
    mut player_controller: Query<(&mut KinematicCharacterController, &Transform, &PlayerKeys), With<PlayerMarker>>,
    keys: Res<Input<KeyCode>>
) {
    let (mut player_controller, pos, player_keys) = player_controller.single_mut();
    let mut mov = Vec3::ZERO;
    if keys.pressed(player_keys.forward) {
        mov -= pos.local_z()
    } else if keys.pressed(player_keys.backward) {
        mov += pos.local_z()
    } else if keys.pressed(player_keys.right) {
        mov += pos.local_x()
    } else if keys.pressed(player_keys.left) {
        mov -= pos.local_x()
    }
    mov = mov.normalize_or_zero() * SPEED;
    player_controller.translation = Some(mov);
}