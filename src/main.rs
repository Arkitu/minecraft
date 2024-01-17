use bevy::{prelude::*, window::ApplicationLifetime};

pub mod bloc_and_chunk;
use bloc_and_chunk::*;
pub mod player;
use player::*;

#[derive(Event)]
struct Render;

fn setup(
    mut cmds: Commands,
    mut chunks: ResMut<Chunks>,
    chunks_query: Query<(&ChunkPos, &ChunkBlocs)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // player
    cmds.spawn(Player::new());

    // directional 'sun' light
    cmds.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            color: Color::rgb(1.0, 1.0, 0.75),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            ..default()
        }.looking_to(Vec3 { x: 0.5, y: -0.5, z: 0.3 }, Vec3::ZERO),
        ..default()
    });

    chunks.generate(ChunkPos { x: 0, y: 0, z: 0 }, &mut cmds, &chunks_query);

    //let mut blocs = ChunkBlocs::new_empty(ChunkPos { x: 0, y: 0, z: 0 }, &mut cmds);

    // for x in 0..(CHUNK_X as u8)-1 {
    //     for z in 0..(CHUNK_Z as u8)-1 {
    //         for y in 1..3 {
    //             blocs.set(&PosInChunk { x, y, z }, Some(cmds.spawn(BlocType::Stone).id()));
    //         }
    //         for y in 3..4 {
    //             blocs.set(&PosInChunk { x, y, z }, Some(cmds.spawn(BlocType::Grass).id()));
    //         }
    //     }
    // }

    // let bloc = cmds.spawn(BlocType::Grass).id();
    // blocs.set(&PosInChunk { x: 1, y: 1, z: 1 }, Some(bloc));

    // chunks
    // let base_chunk = Chunk::new_with_blocs(
    //     ChunkPos { x: 0, y: 0, z: 0 },
    //     blocs
    // );
    // let base_chunk_id = cmds.spawn(base_chunk).id();
    // chunks.insert(ChunkPos { x: 0, y: 0, z: 0 }, base_chunk_id);

    // Spawn point at origin for debug
    cmds.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(1.0, 1.0, 1.0),
        ..default()
    });
}

fn render_all(
    mut ev_app_lifetime: EventReader<ApplicationLifetime>,
    mut chunks_query: Query<&ChunkBlocs>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    blocs_types_query: Query<&BlocType>,
    mut blocs_query: Query<(&Pos, &Neighbors, &BlocType, &mut BlocFaces)>
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
    for blocs in chunks_query.iter_mut() {
        blocs.render(&asset_server, &mut blocs_query, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        //.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new()) // for debug
        .add_plugins(bevy_editor_pls::EditorPlugin::default()) // for debug
        .add_systems(Startup, setup)
        .add_systems(Update, render_all)
        .add_event::<Render>()
        .insert_resource(Chunks::new(0))
        .run();
}