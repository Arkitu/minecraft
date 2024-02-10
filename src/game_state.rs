use bevy::{prelude::*, utils::HashMap};
use crate::{BlocType, ChunkPos, CHUNK_X, CHUNK_Y, CHUNK_Z};

struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        
    }
}

#[derive(Resource)]
pub struct GameState {
    chunks: HashMap<ChunkPos, [BlocType;CHUNK_X*CHUNK_Y*CHUNK_Z]>,
    player_pos: Transform
}