use bevy::{prelude::*, window::ApplicationLifetime};

mod bloc_and_chunk;
use bloc_and_chunk::*;

#[derive(Component)]
struct CameraMarker;

#[derive(Event)]
struct Render;

fn setup(
    mut cmds: Commands,
    mut chunks: ResMut<Chunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // camera
    cmds.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(40.0, 48.0, 64.0)
                .looking_at(Vec3::ZERO, Vec3::ZERO),
            
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
        transform: Transform::from_xyz(8.0, 8.0, 8.0),
        ..default()
    });

    let mut blocs = ChunkBlocs::default();
    let bloc = cmds.spawn(BlocType::Grass).id();
    blocs.set(&PosInChunk { x: 1, y: 1, z: 1 }, Some(bloc));

    // chunks
    let base_chunk = Chunk::new_with_blocs(
        ChunkPos { x: 0, y: 0, z: 0 },
        blocs
    );
    let base_chunk_id = cmds.spawn(base_chunk).id();
    chunks.insert(ChunkPos { x: 0, y: 0, z: 0 }, base_chunk_id);

    // Spawn point at origin for debug
    cmds.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(1.0, 1.0, 1.0),
        ..default()
    });
        
}

fn render(
    mut ev_app_lifetime: EventReader<ApplicationLifetime>,
    mut chunks_query: Query<(&ChunkBlocs, &mut ChunkFaces, &ChunkPos)>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    blocs_query: Query<&BlocType>,
) {
    let mut skip = true;
    for e in ev_app_lifetime.read() {
        if let ApplicationLifetime::Started = e {
            skip = false;
            break;
        }
    }
    if skip {
        return
    }
    dbg!("render");
    for (blocs, mut faces, pos) in chunks_query.iter_mut() {
        *faces = blocs.render(pos, &asset_server, &blocs_query, &mut meshes, &mut materials, &mut cmds);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        //.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new()) // for debug
        .add_plugins(bevy_editor_pls::EditorPlugin::default()) // for debug
        .add_systems(Startup, setup)
        .add_systems(Update, render)
        .add_event::<Render>()
        .insert_resource(Chunks::new(0))
        .run();
}