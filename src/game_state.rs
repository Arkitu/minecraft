use std::{borrow::Borrow, fs, path::Path, time::{SystemTime, UNIX_EPOCH}};

use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::dynamics::Velocity;
use crate::{BlocType, ChunkBlocs, ChunkNeighborsAreLinked, ChunkPos, Chunks, DefaultGenerator, Neighbors, PlayerMarker, Render, CHUNK_X, CHUNK_Y, CHUNK_Z};
use serde::{Serialize, Deserialize};

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState {
            chunks: HashMap::new()
        })
            .add_systems(Update, save)
            .add_systems(Update, load);
    }
}

#[derive(Clone)]
pub struct ChunkTypes(pub [BlocType;CHUNK_X*CHUNK_Y*CHUNK_Z]);

impl Serialize for ChunkTypes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        self.0.to_vec().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ChunkTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        Ok(Self(Vec::deserialize(deserializer)?.try_into().unwrap()))
    }
}

#[derive(Resource)]
pub struct GameState {
    pub chunks: HashMap<ChunkPos, ChunkTypes>
}



#[derive(Serialize, Deserialize)]
pub struct GameSave {
    pub chunks: HashMap<ChunkPos, ChunkTypes>,
    pub player_pos: Transform,
    pub player_linvel: Vec3,
    pub player_angvel: Vec3
}

pub fn save(
    keys: Res<Input<KeyCode>>,
    game_state: Res<GameState>,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>
) {
    if !keys.just_pressed(KeyCode::T) {
        return
    }

    let (pos, vel) = player.single();
    let save = GameSave {
        chunks: game_state.chunks.clone(),
        player_pos: *pos,
        player_linvel: vel.linvel,
        player_angvel: vel.angvel
    };

    if !Path::new("saves").exists() {
        fs::create_dir("saves").unwrap();
    }

    fs::write(format!("saves/{:?}.save", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()), bincode::serialize(&save).unwrap()).unwrap();
}

pub fn load(
    keys: Res<Input<KeyCode>>,
    mut chunks: ResMut<Chunks<DefaultGenerator>>,
    mut game_state: ResMut<GameState>,
    mut player: Query<(&mut Transform, &mut Velocity), With<PlayerMarker>>,
    mut cmds: Commands,
    mut ev_render: EventWriter<Render>
) {
    if !keys.just_pressed(KeyCode::Y) {
        return
    }

    let content = match fs::read_dir("saves") {
        Err(e) => {
            warn!("Cannot read saves : {}", e);
            return;
        },
        Ok(rd) => {
            let filename = rd.filter_map(|x|{
                match x {
                    Ok(x) => {
                        let filename = x.file_name();
                        let filename = filename.to_string_lossy();
                        if filename.ends_with(".save") {
                            filename.trim_end_matches(".save").parse::<u128>().ok()
                        } else {
                            None
                        }
                    },
                    Err(_) => None
                }
            }).max();
            match filename {
                None => {
                    warn!("No save found");
                    return
                },
                Some(filename) => {
                    match fs::read(format!("saves/{}.save", filename)) {
                        Err(e) => {
                            warn!("Cannot read save : {}", e);
                            return;
                        },
                        Ok(c) => c
                    }
                }
            }
        }
    };

    let game_save: GameSave = match bincode::deserialize(&content) {
        Err(e) => {
            warn!("Cannot deserialize save : {}", e);
            return;
        },
        Ok(gs) => gs
    };

    game_state.chunks = game_save.chunks;

    let old_loaded_chunks = chunks.inner.keys().map(|x|*x).clone().collect::<Vec<_>>();

    chunks.clear(&mut cmds);

    for pos in old_loaded_chunks {
        if game_state.chunks.contains_key(&pos) {
            chunks.load(pos, &game_state, &mut cmds);
        }
    }

    ev_render.send(Render);

    let (mut pos, mut vel) = player.single_mut();
    *pos = game_save.player_pos;
    *vel = Velocity { linvel: game_save.player_linvel, angvel: game_save.player_angvel };
}