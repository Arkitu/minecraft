use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod camera;
pub use camera::{*, Camera};

const SPEED: f32 = 0.4;
const PLAYER_HITBOX_RADIUS: f32 = 0.33;
const PLAYER_HITBOX_HEIGHT: f32 = 1.8;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            .add_systems(Update, move_player);
            // .add_systems(Update, gravity)
            // .add_systems(Update, log);
    }
}

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

#[derive(Component)]
pub struct Feet {
    marker: FeetMarker,
    collider: Collider,
    sensor: Sensor,
    transform: TransformBundle
}
impl Default for Feet {
    fn default() -> Self {
        Self {
            marker: FeetMarker,
            collider: Collider::cylinder(0.1, PLAYER_HITBOX_RADIUS-0.01),
            sensor: Sensor,
            transform: TransformBundle::from_transform(Transform::from_xyz(0.0, -PLAYER_HITBOX_HEIGHT/2.0, 0.0))
        }
    }
}

#[derive(Bundle)]
pub struct Player {
    collider: Collider,
    collider_mass_properties: ColliderMassProperties,
    damping: Damping,
    gravity_scale: GravityScale,
    marker: PlayerMarker,
    spatial: SpatialBundle,
    rigid_body: RigidBody,
    input_force: ExternalForce,
    jump_impulse: ExternalImpulse,
    sleeping: Sleeping,
    locked_axes: LockedAxes,
    keys: PlayerKeys
}
impl Player {
    pub fn new() -> Self {
        Self {
            collider: Collider::capsule_y(PLAYER_HITBOX_HEIGHT/2.0, PLAYER_HITBOX_RADIUS), // ::cylinder(PLAYER_HITBOX_HEIGHT/2.0, PLAYER_HITBOX_RADIUS),
            collider_mass_properties: ColliderMassProperties::Density(0.01),
            damping: Damping {
                linear_damping: 3.0,
                angular_damping: 0.0
            },
            gravity_scale: GravityScale(5.0),
            marker: PlayerMarker,
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0.0, 4.5, 0.0)),
            rigid_body: RigidBody::Dynamic,
            input_force: ExternalForce::default(),
            jump_impulse: ExternalImpulse::default(),
            sleeping: Sleeping::disabled(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            keys: PlayerKeys::default(),
        }
    }
    pub fn spawn(cmds: &mut Commands) {
        cmds.spawn(Self::new())
            .with_children(|parent| {
                parent.spawn(Camera::default());
                parent.spawn(Feet::default());
            });
    }
}

pub fn move_player(
    mut player: Query<(&mut ExternalForce, &mut ExternalImpulse, &Transform, &PlayerKeys, Entity), With<PlayerMarker>>,
    rapier_ctx: Res<RapierContext>,
    feet: Query<Entity, With<FeetMarker>>,
    keys: Res<Input<KeyCode>>
) {
    let (mut input_force, mut jump_impulse, pos, player_keys, player) = player.single_mut();
    let mut mov = Vec3::ZERO;
    if keys.pressed(player_keys.forward) || keys.just_pressed(player_keys.forward) {
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

    input_force.force = mov;

    let feet = feet.single();

    for (e1, e2, is_intersecting) in rapier_ctx.intersections_with(feet) {
        if !is_intersecting {
            continue
        }
        let other = if e1 == feet {
            e2
        } else {
            e1
        };
        
    }
    for pair in rapier_ctx.contacts_with(entity) {
        pair
    }

    if keys.just_pressed(player_keys.jump) {
        jump_impulse.impulse = Vec3::new(0.0, 0.15, 0.0);
    } else {
        jump_impulse.impulse = Vec3::ZERO;
    }

    // if let Ok(output) = output.get_single() {
    //     if keys.pressed(player_keys.jump) && output.grounded {
    //     }
    // }
}

// pub fn gravity(
//     mut player_controller: Query<&mut KinematicCharacterController, With<PlayerMarker>>
// ) {
//     let mut player_controller = player_controller.single_mut();
//     match player_controller.translation {
//         Some(ref mut t) => {
//             t.y -= 0.2;
//         },
//         None => {
//             player_controller.translation = Some(Vec3::new(0.0, -0.2, 0.0));
//         }
//     }
// }

// pub fn log(
//     output: Query<&KinematicCharacterControllerOutput, With<PlayerMarker>>
// ) {
//     let output = match output.get_single() {
//         Ok(o) => o,
//         Err(_) => return
//     };
//     dbg!(output.desired_translation, output.effective_translation, output.grounded);
// }