use bevy::prelude::*;
use crate::{blocs::*, PlayerMarker};

const RENDER_DISTANCE: u32 = 3;

pub struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_loaded_chunks);
    }
}

/// Load/unload chunks around the player
pub fn update_loaded_chunks(
    player: Query<&Transform, With<PlayerMarker>>
) {
    let player_pos = player.single();

    let player_chunk = ChunkPos {
        x: (player_pos.translation.x / (CHUNK_X as f32*SQUARE_UNIT)).round() as i32,
        y: (player_pos.translation.y / (CHUNK_Y as f32*SQUARE_UNIT)).round() as i32,
        z: (player_pos.translation.z / (CHUNK_Z as f32*SQUARE_UNIT)).round() as i32
    };

    
}