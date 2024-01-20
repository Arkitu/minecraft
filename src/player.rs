use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod camera;
pub use camera::{*, Camera};

const SPEED: f32 = 0.3;
const PLAYER_HITBOX_RADIUS: f32 = 0.33;
const PLAYER_HITBOX_HEIGHT: f32 = 1.8;

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

#[derive(Component)]
pub struct FeetMarker;

#[derive(Bundle)]
pub struct Feet {
    marker: FeetMarker,
    collider: Collider,
    transform: TransformBundle,
    sensor: Sensor
}
impl Feet {
    pub fn spawn(parent: &mut ChildBuilder) {
        parent.spawn(Self::default());
    }
}
impl Default for Feet {
    fn default() -> Self {
        Self {
            marker: FeetMarker,
            collider: Collider::cylinder(0.1, PLAYER_HITBOX_RADIUS),
            sensor: Sensor,
            transform: TransformBundle::from_transform(Transform::from_xyz(0.0, -PLAYER_HITBOX_HEIGHT/2.0, 0.0))
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
            collider: Collider::cylinder(PLAYER_HITBOX_HEIGHT/2.0, PLAYER_HITBOX_RADIUS),
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
                Feet::spawn(parent);
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
    }
    if keys.pressed(player_keys.backward) {
        mov += pos.local_z()
    }
    if keys.pressed(player_keys.right) {
        mov += pos.local_x()
    }
    if keys.pressed(player_keys.left) {
        mov -= pos.local_x()
    }
    mov = mov.normalize_or_zero() * SPEED;
    player_controller.translation = Some(mov);
}