use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::{remove_bloc, BaseMaterial, BlocFaces, BlocType, ChunkPos, ChunkSaves, Cracks, DestructionLevel, FaceMarker, GameState, Neighbors, NextMaterial, PosInChunk, BLOCS_PHYSIC_GROUP};

pub mod camera;
pub use camera::*;

const RANGE: f32 = 5.0;

pub struct HeadPlugin;
impl Plugin for HeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            .add_systems(Update, destroy_bloc)
            .add_systems(Update, reset_destruction_lvl);
    }
}

#[derive(Component, Default)]
/// (bloc_entity, advancement_between_0_and_1)
pub struct BlocBeingDestroyed(Option<(Entity, f32)>);

#[derive(Component)]
pub struct HeadMarker;

#[derive(Bundle)]
pub struct Head {
    marker: HeadMarker,
    cam: Camera3dBundle,
    config: CameraConfig,
    bloc_being_destroyed: BlocBeingDestroyed
}
impl Default for Head {
    fn default() -> Self {
        Self {
            marker: HeadMarker,
            cam: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            },
            config: CameraConfig::default(),
            bloc_being_destroyed: BlocBeingDestroyed(None)
        }
    }
}

pub fn destroy_bloc(
    head: Query<&GlobalTransform, With<HeadMarker>>,
    rapier_ctx: Res<RapierContext>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    blocs: (Query<'_, '_, &'_ mut BlocType>, Query<(Entity,&mut Neighbors,&mut BlocFaces)>, Query<(&PosInChunk, &Parent), With<BlocType>>, Query<&ChunkPos>),
    // mut blocs_types_query: Query<'_, '_, &'_ mut BlocType>,
    // mut blocs: Query<(Entity,&mut Neighbors,&mut BlocFaces)>,
    // blocs_pos_parent_query: Query<(&PosInChunk, &Parent), With<BlocType>>,
    // chunk_pos_query: Query<&ChunkPos>,
    mut game_state: ResMut<GameState>,
    mut changes: ResMut<ChunkSaves>,
    mut faces: Query<(&mut Handle<StandardMaterial>, &BaseMaterial, &mut NextMaterial, &mut DestructionLevel), With<FaceMarker>>,
    mut cmds: Commands,
    mut bloc_being_destroyed: Query<&mut BlocBeingDestroyed, With<HeadMarker>>,
    time: Res<Time>,
    images: Res<Assets<Image>>,
    cracks: Res<Cracks>,
    #[cfg(not(target_arch = "wasm32"))]
    mouse: Res<ButtonInput<MouseButton>>,
    #[cfg(target_arch = "wasm32")]
    wasm_mouse_tracker: Res<WasmMouseTracker>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    if !mouse.pressed(MouseButton::Left) {
        bloc_being_destroyed.single_mut().0 = None;
        return;
    }
    #[cfg(target_arch = "wasm32")]
    if !wasm_mouse_tracker.is_mouse_down() {
        bloc_being_destroyed.single_mut().0 = None;
        return;
    }

    let (mut blocs_types_query, mut blocs, blocs_pos_parent_query, chunk_pos_query) = blocs;

    let global_pos = head.single();
    let (selected_bloc, distance) = match rapier_ctx.cast_ray(
        global_pos.translation(),
        global_pos.forward(),
        RANGE,
        true,
        QueryFilter::default().groups(
            CollisionGroups::new(Group::ALL, BLOCS_PHYSIC_GROUP)
        )
    ) {
        None => return,
        Some(sb) => sb
    };

    let mut bloc_being_destroyed = bloc_being_destroyed.single_mut();
    let (bbd, crack_lvl) = match bloc_being_destroyed.0 {
        None => {
            ((selected_bloc, 0.0),Some(0))
        },
        Some(mut bbd) => {
            if bbd.0 != selected_bloc {
                ((selected_bloc, 0.0), Some(0))
            } else {
                let old_time = bbd.1;
                bbd.1 += time.delta_seconds();
                let crack = if old_time < 0.2 && bbd.1 >= 0.2 {
                    Some(1)
                } else if old_time < 0.4 && bbd.1 >= 0.4 {
                    Some(2)
                } else if old_time < 0.6 && bbd.1 >= 0.6 {
                    Some(3)
                } else if old_time < 0.8 && bbd.1 >= 0.8 {
                    Some(4)
                } else {
                    None
                };
                (bbd, crack)
            }
        }
    };

    if let Some(crack) = crack_lvl {
        let crack = images.get(cracks.0[crack].id()).unwrap();
        let bloc = blocs.get_mut(selected_bloc).unwrap();
        for x in bloc.2.0.iter() {
            let (_, base_mat, mut next_mat, mut destruction_lvl) = faces.get_mut(*x).unwrap();
            let mut material = materials.get(base_mat.0.id()).unwrap().clone();
            let mut img = match images.get(material.base_color_texture.unwrap().id()) {
                Some(img) => img.clone(),
                None => continue
            };
            for (i, c) in img.data.chunks_mut(4).zip(crack.data.chunks(4)) {
                let c3 = c[3] as u16;
                *i.get_mut(0).unwrap() = (((i[0] as u16 * (255-c3)) + (c[0] as u16 * c3)) / (255)) as u8;
                *i.get_mut(1).unwrap() = (((i[1] as u16 * (255-c3)) + (c[1] as u16 * c3)) / (255)) as u8;
                *i.get_mut(2).unwrap() = (((i[2] as u16 * (255-c3)) + (c[2] as u16 * c3)) / (255)) as u8;
            }
            material.base_color_texture = Some(asset_server.add(img));
            next_mat.0 = Some(asset_server.add(material));
            *destruction_lvl = match crack_lvl {
                None => DestructionLevel::Zero,
                Some(0) => DestructionLevel::One,
                Some(1) => DestructionLevel::Two,
                Some(2) => DestructionLevel::Three,
                Some(3) => DestructionLevel::Four,
                Some(4) => DestructionLevel::Five,
                _ => unreachable!()
            };
        }
    }

    if bbd.1 >= 1.0 {
        let neighbors = blocs.get_mut(selected_bloc).unwrap().1.clone();
        remove_bloc(selected_bloc, &neighbors, &mut blocs, &mut blocs_types_query, &blocs_pos_parent_query, &chunk_pos_query, &mut game_state, &mut changes, &mut cmds, &asset_server, &mut meshes, &mut materials);
        bloc_being_destroyed.0 = None;
    } else {
        bloc_being_destroyed.0 = Some(bbd);
    }
}

pub fn reset_destruction_lvl(
    mut faces: Query<(&Parent, &mut DestructionLevel, &BaseMaterial, &mut NextMaterial), With<FaceMarker>>,
    bbd: Query<&BlocBeingDestroyed, With<HeadMarker>>
) {
    let bbd = bbd.single();
    for (parent, mut lvl, base_mat, mut next_mat) in faces.iter_mut() {
        if lvl.as_ref() != &DestructionLevel::Zero {
            match bbd.0 {
                Some(bbd) => {
                    if parent.get() != bbd.0 {
                        next_mat.0 = Some(base_mat.0.clone());
                        *lvl = DestructionLevel::Zero;
                    }
                },
                None => {
                    next_mat.0 = Some(base_mat.0.clone());
                    *lvl = DestructionLevel::Zero;
                }
            }
        }
    }
}