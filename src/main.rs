use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod blocs;
use blocs::*;
pub mod player;
use player::*;
pub mod game_state;
use game_state::*;

fn setup<G: Generator>(
    mut cmds: Commands,
    mut chunks: ResMut<Chunks<G>>,
    mut game_state: ResMut<GameState>,
    chunk_saves: Res<ChunkSaves>,
    mut ev_render: EventWriter<Render>
) {
    // player
    Player::spawn(&mut cmds);

    // directional 'sun' light
    cmds.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            //shadows_enabled: true,
            illuminance: 3000.0,
            color: Color::rgb(1.0, 1.0, 0.75),
            ..default()
        },
        transform: Transform::default().looking_to(Vec3 { x: 0.5, y: -0.5, z: 0.3 }, Vec3::ZERO),
        ..default()
    });

    for x in -1..=1 {
        for z in -1..=1 {
            chunks.generate(ChunkPos { x, y: 0, z }, &chunk_saves, &mut game_state, &mut cmds);
        }
    }

    ev_render.send(Render);
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        // .insert_resource(AmbientLight {
        //     brightness: 0.4,
        //     ..Default::default()
        // })
        .add_plugins(bevy_editor_pls::EditorPlugin::default()) // for debug
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default()) // for debug
        .add_plugins(PlayerPlugin)
        .add_plugins(BlocAndChunkPlugin)
        .add_plugins(GameStatePlugin)
        .add_systems(Startup, setup::<DefaultGenerator>)
        .add_event::<Render>()
        .insert_resource(Chunks::<DefaultGenerator>::new(rand::random()));

    app.run();
}