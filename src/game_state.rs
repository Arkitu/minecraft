use std::{borrow::Borrow, fs, path::Path};
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::dynamics::Velocity;
use crate::{BlocType, ChunkBlocs, ChunkNeighborsAreLinked, ChunkPos, Chunks, DefaultGenerator, Neighbors, PlayerMarker, PosInChunk, Render, CHUNK_X, CHUNK_Y, CHUNK_Z};
use serde::{Serialize, Deserialize};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
use web_time::{SystemTime, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
use base64::prelude::*;

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .init_resource::<ChunkSaves>()
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

#[derive(Resource, Default)]
pub struct GameState {
    pub chunks: HashMap<ChunkPos, ChunkTypes>
}

#[derive(Resource, Serialize, Deserialize, Clone, Default)]
pub struct ChunkSaves (pub HashMap<ChunkPos, ChunkSave>);

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ChunkSave {
    pub changes: HashMap<PosInChunk, BlocType>
}

#[derive(Serialize, Deserialize)]
pub struct GameSave {
    pub chunks: ChunkSaves,
    pub player_pos: Transform,
    pub player_linvel: Vec3,
    pub player_angvel: Vec3
}

pub fn save(
    keys: Res<Input<KeyCode>>,
    game_state: Res<GameState>,
    chunk_saves: Res<ChunkSaves>,
    player: Query<(&Transform, &Velocity), With<PlayerMarker>>
) {
    if !keys.just_pressed(KeyCode::T) {
        return
    }

    let (pos, vel) = player.single();
    let save = GameSave {
        chunks: chunk_saves.clone(),
        player_pos: *pos,
        player_linvel: vel.linvel,
        player_angvel: vel.angvel
    };

    let path = format!("saves/{:?}.save", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    let serialized = bincode::serialize(&save).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if !Path::new("saves").exists() {
            fs::create_dir("saves").unwrap();
        }

        fs::write(path, serialized).unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    unsafe {
        web_sys::window().unwrap().local_storage().unwrap().unwrap().set_item(&path, &BASE64_STANDARD.encode(&serialized)).unwrap();
    }
}

pub fn load(
    keys: Res<Input<KeyCode>>,
    mut chunks: ResMut<Chunks<DefaultGenerator>>,
    mut game_state: ResMut<GameState>,
    mut chunk_saves: ResMut<ChunkSaves>,
    mut player: Query<(&mut Transform, &mut Velocity), With<PlayerMarker>>,
    mut cmds: Commands,
    mut ev_render: EventWriter<Render>
) {
    if !keys.just_pressed(KeyCode::Y) {
        return
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    let content = {
        let ls = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let mut key = None;
        for i in 0..ls.length().unwrap() {
            let k = ls.key(i).unwrap().unwrap();
            if k.starts_with("saves/") && k.ends_with(".save") {
                let k = match k.trim_start_matches("saves/").trim_end_matches(".save").parse::<u128>() {
                    Err(_) => continue,
                    Ok(k) => k
                };

                match key {
                    None => key = Some(k),
                    Some(k1) => key = Some(k1.max(k))
                }
            }
        }
        let key = match key {
            None => {
                warn!("No save found");
                return
            },
            Some(k) => k
        };
        let string = ls.get_item(&format!("saves/{}.save", key)).unwrap().unwrap();
        let bytes = match BASE64_STANDARD.decode(string) {
            Ok(b) => b,
            Err(e) => {
                warn!("Invalid save (base64 error) : {}", e);
                return
            }
        };
        bytes
    };

    let game_save: GameSave = match bincode::deserialize(&content) {
        Err(e) => {
            warn!("Cannot deserialize save : {}", e);
            return;
        },
        Ok(gs) => gs
    };

    *chunk_saves = game_save.chunks;

    let old_loaded_chunks = chunks.inner.keys().map(|x|*x).clone().collect::<Vec<_>>();

    chunks.clear(&mut cmds);
    game_state.chunks.clear();

    for pos in old_loaded_chunks {
        chunks.load_or_generate(pos, &chunk_saves, &mut game_state, &mut cmds);
    }

    ev_render.send(Render);

    let (mut pos, mut vel) = player.single_mut();
    *pos = game_save.player_pos;
    *vel = Velocity { linvel: game_save.player_linvel, angvel: game_save.player_angvel };
}