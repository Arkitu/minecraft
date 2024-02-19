use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod head;
pub use head::{*, Head};

use crate::BLOCS_PHYSIC_GROUP;

const SPEED: f32 = 0.08;
const JUMP_SPEED: f32 = 3.0;
const PLAYER_HITBOX_RADIUS: f32 = 0.33;
const PLAYER_HITBOX_HEIGHT: f32 = 1.8;

const PLAYER_PHYSIC_GROUP: Group = Group::GROUP_2;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HeadPlugin)
            .add_systems(Update, move_player);
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

#[derive(Component, Default, Clone)]
pub struct PlayerVelocity (Vec3);

#[derive(Component)]
pub struct TouchedGroudLastFrame(bool);

#[derive(Bundle)]
pub struct Player {
    collider: Collider,
    // collider_mass_properties: ColliderMassProperties,
    // friction: Friction,
    // damping: Damping,
    gravity_scale: GravityScale,
    vel: PlayerVelocity,
    kcc: KinematicCharacterController,
    marker: PlayerMarker,
    spatial: SpatialBundle,
    rigid_body: RigidBody,
    velocity: Velocity,
    input_force: ExternalForce,
    jump_impulse: ExternalImpulse,
    sleeping: Sleeping,
    locked_axes: LockedAxes,
    keys: PlayerKeys,
    collision_groups: CollisionGroups,
    touched_groud_last_frame: TouchedGroudLastFrame
}
impl Player {
    pub fn new() -> Self {
        Self {
            collider: Collider::round_cylinder((PLAYER_HITBOX_HEIGHT/2.0)-0.1, PLAYER_HITBOX_RADIUS-0.1, 0.1),
            // collider_mass_properties: ColliderMassProperties::Density(1.0),
            // friction: Friction { coefficient: 2.5, combine_rule: CoefficientCombineRule::Multiply },
            // damping: Damping {
            //     linear_damping: 1.0,
            //     angular_damping: 0.0
            // },
            gravity_scale: GravityScale(4.0),
            vel: PlayerVelocity::default(),
            kcc: KinematicCharacterController {
                offset: CharacterLength::Absolute(0.05),
                ..Default::default()
            },
            marker: PlayerMarker,
            spatial: SpatialBundle::from_transform(Transform::from_xyz(0.0, 4.5, 0.0)),
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::default(),
            input_force: ExternalForce::default(),
            jump_impulse: ExternalImpulse::default(),
            sleeping: Sleeping::disabled(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            keys: PlayerKeys::default(),
            collision_groups: CollisionGroups::new(PLAYER_PHYSIC_GROUP, Group::ALL),
            touched_groud_last_frame: TouchedGroudLastFrame(false)
        }
    }
    pub fn spawn(cmds: &mut Commands) {
        cmds.spawn(Self::new())
            .with_children(|parent| {
                parent.spawn(Head::default());
            });
    }
}

pub fn move_player(
    mut player: Query<(&mut ExternalForce, &mut ExternalImpulse, &Transform, &PlayerKeys, &mut TouchedGroudLastFrame, &mut KinematicCharacterController, Option<&KinematicCharacterControllerOutput>, &mut PlayerVelocity, Entity), With<PlayerMarker>>,
    rapier_ctx: Res<RapierContext>,
    keys: Res<Input<KeyCode>>
) {
    let (mut input_force, mut jump_impulse, pos, player_keys, mut touched_groud_last_frame, mut kcc, kcc_out, mut vel, player) = player.single_mut();
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

    mov = mov.normalize_or_zero();

    let ground = rapier_ctx.intersection_with_shape(
        pos.translation + Vec3::new(0.0, -PLAYER_HITBOX_HEIGHT/2.0, 0.0),
        Quat::IDENTITY,
        &Collider::cylinder(0.1, PLAYER_HITBOX_RADIUS-0.1),
        QueryFilter::default().groups(
            CollisionGroups::new(PLAYER_PHYSIC_GROUP, BLOCS_PHYSIC_GROUP)
        )
    );

    let is_on_ground = match ground {
        None => false,
        Some(ground) => match rapier_ctx.contact_pair(ground, player) {
            None => false,
            Some(pair) => pair.has_any_active_contacts()
        }
    };

    kcc.translation = Some(mov*SPEED);

    if keys.pressed(player_keys.jump) {
        if is_on_ground && touched_groud_last_frame.0 {
            jump_impulse.impulse = Vec3::new(0.0, JUMP_SPEED, 0.0);
        } else {

        }
    } else {
        jump_impulse.impulse = Vec3::ZERO;
    }

    touched_groud_last_frame.0 = is_on_ground;
}