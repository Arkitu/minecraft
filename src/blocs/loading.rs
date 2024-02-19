use bevy::prelude::*;
use crate::{blocs::*, PlayerMarker};

// Do not put a value higher than 2^31 (with margin)
const RENDER_DISTANCE: u32 = 5;

pub struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_loaded_chunks)
            .add_systems(Update, render_added_chunks);
    }
}

/// Load/unload chunks around the player
pub fn update_loaded_chunks(
    player: Query<&Transform, With<PlayerMarker>>,
    mut chunks: ResMut<Chunks<DefaultGenerator>>,
    mut chunks_query: Query<(&ChunkBlocs, &mut ChunkNeighborsAreLinked)>,
    mut blocs_query: Query<&mut Neighbors>,
    chunk_saves: Res<ChunkSaves>,
    mut game_state: ResMut<GameState>,
    mut cmds: Commands
) {
    //dbg!("update_loaded_chunks");
    let player_pos = player.single();

    let player_chunk = ChunkPos {
        x: (player_pos.translation.x / (CHUNK_X as f32*SQUARE_UNIT)).round() as i32,
        y: (player_pos.translation.y / (CHUNK_Y as f32*SQUARE_UNIT)).round() as i32,
        z: (player_pos.translation.z / (CHUNK_Z as f32*SQUARE_UNIT)).round() as i32
    };

    // Dispawn chunks
    for pos in chunks.inner.keys().map(|x|*x).collect::<Vec<_>>() {
        if (pos.x - player_chunk.x).saturating_pow(2) as u32 + (pos.z - player_chunk.z).saturating_pow(2) as u32 > RENDER_DISTANCE.pow(2) {
            chunks.unload(pos, &mut chunks_query, &mut blocs_query, &mut cmds);
        }
    }

    // Spawn chunks
    for x in -(RENDER_DISTANCE as i32)+player_chunk.x..(RENDER_DISTANCE as i32)+player_chunk.x {
        for z in -(RENDER_DISTANCE as i32)+player_chunk.z..(RENDER_DISTANCE as i32)+player_chunk.z {
            if (x - player_chunk.x).saturating_pow(2) as u32 + (z - player_chunk.z).saturating_pow(2) as u32 > RENDER_DISTANCE.pow(2) {
                continue
            }
            chunks.load_or_generate(ChunkPos { x, z, y: 0 }, &chunk_saves, &mut game_state, &mut cmds);

        }
    }
}

pub fn render_added_chunks(
    mut chunks_query: Query<(&ChunkBlocs, &ChunkPos), Added<ChunkBlocs>>,
    player: Query<&Transform, With<PlayerMarker>>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    blocs_types_query: Query<&BlocType>,
    mut blocs_query: Query<(Entity, &Neighbors, &mut BlocFaces)>
) {
    let player_pos = player.single();
    let player_chunk = ChunkPos {
        x: (player_pos.translation.x / (CHUNK_X as f32*SQUARE_UNIT)).round() as i32,
        y: (player_pos.translation.y / (CHUNK_Y as f32*SQUARE_UNIT)).round() as i32,
        z: (player_pos.translation.z / (CHUNK_Z as f32*SQUARE_UNIT)).round() as i32
    };
    for (blocs, pos) in chunks_query.iter_mut() {
        if (pos.x - player_chunk.x).saturating_pow(2) as u32 + (pos.z - player_chunk.z).saturating_pow(2) as u32 > RENDER_DISTANCE.pow(2) {
            continue
        }
        blocs.render(&asset_server, &mut blocs_query, &blocs_types_query, &mut meshes, &mut materials, &mut cmds);
    }
}