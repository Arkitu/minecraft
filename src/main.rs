use bevy::{prelude::*, window::ApplicationLifetime};
use bevy_rapier3d::prelude::*;

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
    Player::spawn(&mut cmds);

    // directional 'sun' light
    cmds.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            color: Color::rgb(1.0, 1.0, 0.75),
            ..default()
        },
        transform: Transform::default().looking_to(Vec3 { x: 0.5, y: -0.5, z: 0.3 }, Vec3::ZERO),
        ..default()
    });

    for x in -1..=1 {
        for z in -1..=1 {
            chunks.generate(ChunkPos { x, y: 0, z }, &mut cmds, &chunks_query);
        }
    }
}

fn render_all(
    mut ev_app_lifetime: EventReader<ApplicationLifetime>,
    mut chunks_query: Query<&ChunkBlocs>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    blocs_types_query: Query<&BlocType>,
    mut blocs_query: Query<(Entity, &Neighbors, &BlocType, &mut BlocFaces)>
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
   let mut app = App::new();
   app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_editor_pls::EditorPlugin::default()) // for debug
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, render_all)
        .add_event::<Render>()
        .insert_resource(Chunks::new(0));

    app.run();
}