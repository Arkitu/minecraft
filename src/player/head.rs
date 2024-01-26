use bevy::{input::common_conditions::input_pressed, prelude::*};
use bevy_rapier3d::prelude::*;
use crate::{render_bloc, BlocFaces, BlocType, Neighbors};

pub mod camera;
pub use camera::*;

const RANGE: f32 = 5.0;

pub struct HeadPlugin;
impl Plugin for HeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin);

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, destroy_bloc.run_if(input_pressed(MouseButton::Left)));

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, destroy_bloc);
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
    blocs_types_query: Query<&BlocType>,
    mut blocs: Query<(Entity,&mut Neighbors,&BlocType,&mut BlocFaces)>,
    mut cmds: Commands,
    mut bloc_being_destroyed: Query<&mut BlocBeingDestroyed, With<HeadMarker>>,
    time: Res<Time>,
    #[cfg(not(target_arch = "wasm32"))]
    mouse: Res<Input<MouseButton>>,
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

    let global_pos = head.single();
    let (selected_bloc, distance) = match rapier_ctx.cast_ray(
        global_pos.translation(),
        global_pos.forward(),
        RANGE,
        true,
        QueryFilter::default().groups(
            CollisionGroups::new(Group::ALL, Group::GROUP_1)
        )
    ) {
        None => return,
        Some(sb) => sb
    };

    let mut bloc_being_destroyed = bloc_being_destroyed.single_mut();
    let mut bbd = bloc_being_destroyed.0.unwrap_or((selected_bloc, 0.0));
    dbg!(bbd);
    
    if bbd.0 != selected_bloc {
        bbd = (selected_bloc, 0.0)
    }

    bbd.1 += time.delta_seconds();

    if bbd.1 >= 1.0 {
        let neighbors = blocs.get_mut(selected_bloc).unwrap().1.clone();
        if let Some(n) = &neighbors.up {
            let (n_bloc_entity, mut n_neighbors, n_type, mut n_faces) = blocs.get_mut(*n).unwrap();
            n_neighbors.down = None;
            render_bloc(n_bloc_entity, &mut n_neighbors, n_type, &mut n_faces, &asset_server, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
        }
        if let Some(n) = &neighbors.down {
            let (n_bloc_entity, mut n_neighbors, n_type, mut n_faces) = blocs.get_mut(*n).unwrap();
            n_neighbors.up = None;
            render_bloc(n_bloc_entity, &mut n_neighbors, n_type, &mut n_faces, &asset_server, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
        }
        if let Some(n) = &neighbors.left {
            let (n_bloc_entity, mut n_neighbors, n_type, mut n_faces) = blocs.get_mut(*n).unwrap();
            n_neighbors.right = None;
            render_bloc(n_bloc_entity, &mut n_neighbors, n_type, &mut n_faces, &asset_server, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
        }
        if let Some(n) = &neighbors.right {
            let (n_bloc_entity, mut n_neighbors, n_type, mut n_faces) = blocs.get_mut(*n).unwrap();
            n_neighbors.left = None;
            render_bloc(n_bloc_entity, &mut n_neighbors, n_type, &mut n_faces, &asset_server, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
        }
        if let Some(n) = &neighbors.front {
            let (n_bloc_entity, mut n_neighbors, n_type, mut n_faces) = blocs.get_mut(*n).unwrap();
            n_neighbors.back = None;
            render_bloc(n_bloc_entity, &mut n_neighbors, n_type, &mut n_faces, &asset_server, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
        }
        if let Some(n) = &neighbors.back {
            let (n_bloc_entity, mut n_neighbors, n_type, mut n_faces) = blocs.get_mut(*n).unwrap();
            n_neighbors.front = None;
            render_bloc(n_bloc_entity, &mut n_neighbors, n_type, &mut n_faces, &asset_server, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
        }

        let bloc_entity = blocs.get_mut(selected_bloc).unwrap().0;

        cmds.entity(bloc_entity).despawn_recursive();
        bloc_being_destroyed.0 = None;
    } else {
        bloc_being_destroyed.0 = Some(bbd);
    }
}