use bevy::{prelude::*, utils::HashMap};

pub const CHUNK_X: usize = 16;
pub const CHUNK_Y: usize = 16;
pub const CHUNK_Z: usize = 256;

#[derive(Component, Clone, Copy)]
pub enum BlocType {
    Dirt,
    Stone
}

impl BlocType {
    pub fn get_asset_path(&self) -> &str {
        match self {
            &Self::Dirt => "default_grass_side.png",
            &Self::Stone => "default_stone_bloc.png"
        }
    }
}

/// Bloc position relative to the chunk corner
#[derive(Component)]
pub struct PosInChunk {
    pub x: u8,
    pub y: u8,
    pub z: u8
}

impl PosInChunk {
    pub fn to_chunk_index(&self) -> usize {
        (self.x as usize)
        + ((self.y as usize) * CHUNK_X)
        + ((self.z as usize) * CHUNK_X * CHUNK_Y)
    }
}

/// Chunk position in chunk unit
#[derive(Component, Eq, Hash, PartialEq)]
pub struct ChunkPos {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

#[derive(Component)]
pub struct ChunkBlocs ([Option<Entity>; CHUNK_X*CHUNK_Y*CHUNK_Z]);

impl Default for ChunkBlocs {
    fn default() -> Self {
        Self ([None; CHUNK_X*CHUNK_Y*CHUNK_Z])
    }
}

#[derive(Component, Default)]
pub struct ChunkFaces (Vec<Entity>);

#[derive(Bundle)]
pub struct Chunk {
    blocs: ChunkBlocs,
    faces: ChunkFaces,
    pos: ChunkPos
}
impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            faces: ChunkFaces::default(),
            blocs: ChunkBlocs::default()
        }
    }
    pub fn get(&self, pos:&PosInChunk) -> Option<Entity> {
        self.blocs.0[pos.to_chunk_index()]
    }
    pub fn render(&self) {
        
    }
}

#[derive(Resource)]
pub struct Chunks {
    inner: HashMap<ChunkPos, Entity>,
    seed: u64
}
impl Chunks {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: HashMap::new(),
            seed
        }
    }
    pub fn insert(&mut self, pos: ChunkPos, chunk: Entity) {
        self.inner.insert(pos, chunk);
    }
}