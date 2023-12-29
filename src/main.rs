use std::f32::consts::PI;
use bevy::prelude::*;

mod bloc_and_chunk;
use bloc_and_chunk::*;

#[derive(Component)]
struct CameraMarker;

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunks: ResMut<Chunks>
) {
    // camera
    cmds.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 12.0, 16.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraMarker,
    ));

    // sun light
    cmds.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // chunks
    let base_chunk = cmds.spawn(Chunk::new(
        ChunkPos { x: 0, y: 0, z: 0 }
    )).id();
    chunks.insert(ChunkPos { x: 0, y: 0, z: 0 }, base_chunk);

    // a random cube
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb_u8(124, 144, 255).into()),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..default()
    // });
}

fn spawn_mesh_for_air(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk: &Chunk,
    pos: &PosInChunk
) {
    if let Some(_) = chunk.get(pos) {
        return
    }
    for pos in [
        PosInChunk {
            x: pos.x-1,
            y: pos.y,
            z: pos.z
        },
        PosInChunk {
            x: pos.x+1,
            y: pos.y,
            z: pos.z
        },
        PosInChunk {
            x: pos.x,
            y: pos.y-1,
            z: pos.z
        },
        PosInChunk {
            x: pos.x,
            y: pos.y+1,
            z: pos.z
        },
        PosInChunk {
            x: pos.x,
            y: pos.y,
            z: pos.z-1
        },
        PosInChunk {
            x: pos.x,
            y: pos.y,
            z: pos.z+1
        }
    ] {
        let bloc = match chunk.get(&pos) {
            Some(b) => b,
            None => continue
        };
        // let bloc = ;
        // let texture_handle = asset_server.load();
        
    }

    // load a texture
    let texture_handle = asset_server.load("default_grass_side.png");

    // create a new quad mesh. this is what we will apply the texture to
    let quad_width = 8.0;
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        quad_width,
        quad_width
    ))));

    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: quad_handle.clone(),
        material: material_handle,
        transform: Transform::from_xyz(0.0, 0.0, 1.5)
            .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
        ..default()
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .insert_resource(Chunks::new(0))
        //.add_systems(Update, spawn_mesh_for_air)
        .run();
}