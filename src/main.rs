use bevy::{prelude::*, window::ApplicationLifetime};

mod bloc_and_chunk;
use bloc_and_chunk::*;

#[derive(Component)]
struct CameraMarker;

#[derive(Event)]
struct Render;

fn setup(
    mut cmds: Commands,
    mut chunks: ResMut<Chunks>
) {
    // camera
    cmds.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(80.0, -96.0, 128.0)
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
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let mut blocs = ChunkBlocs::default();
    let bloc = cmds.spawn(BlocType::Grass).id();
    blocs.set(&PosInChunk { x: 0, y: 0, z: 1 }, Some(bloc));

    // chunks
    let base_chunk = Chunk::new_with_blocs(
        ChunkPos { x: 0, y: 0, z: 0 },
        blocs
    );
    let base_chunk_id = cmds.spawn(base_chunk).id();
    chunks.insert(ChunkPos { x: 0, y: 0, z: 0 }, base_chunk_id);
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
        .add_systems(Startup, setup)
        .add_systems(Update, render)
        .add_event::<Render>()
        .insert_resource(Chunks::new(0))
        .run();
}