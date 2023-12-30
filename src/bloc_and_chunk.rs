use bevy::{prelude::*, utils::HashMap};

pub const CHUNK_X: usize = 4;
pub const CHUNK_Y: usize = 4;
pub const CHUNK_Z: usize = 8;

#[derive(Component, Clone, Copy)]
pub enum BlocType {
    Dirt,
    Grass,
    Stone
}

impl Into<&str> for &BlocType {
    fn into(self) -> &'static str {
        match self {
            BlocType::Dirt => "dirt",
            BlocType::Grass => "grass",
            BlocType::Stone => "stone"
        }
    }
}
impl std::fmt::Display for BlocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}

/// Bloc position relative to the chunk corner
#[derive(Component, Debug)]
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
    pub fn from_chunk_index(chunk_index: usize) -> Self {
        let z = chunk_index / (CHUNK_Y*CHUNK_X);
        let y = (chunk_index - (CHUNK_Y*CHUNK_X*z)) / CHUNK_X;
        let x = chunk_index - (CHUNK_Y*CHUNK_X*z) - (CHUNK_X*y);
        PosInChunk {
            x: x as u8,
            y: y as u8,
            z: z as u8
        }
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

enum Direction {
    Up,
    Down,
    Right,
    Left,
    Front,
    Back
}
impl Direction {
    /// Return a tuple (to_add, to_remove) because it's usize and we can't return negative values
    fn index_difference(&self) -> (usize, usize) {
        match self {
            Direction::Up => (CHUNK_X*CHUNK_Y, 0),
            Direction::Down => (0, CHUNK_X*CHUNK_Y),
            Direction::Right => (1, 0),
            Direction::Left => (0, 1),
            Direction::Front => (CHUNK_X, 0),
            Direction::Back => (0, CHUNK_X)
        }
    }
    fn face_to_render_name(&self) -> &'static str {
        match self {
            Direction::Up => "bottom",
            Direction::Down => "top",
            Direction::Right => "left",
            Direction::Left => "right",
            Direction::Front => "back",
            Direction::Back => "front"
        }
    }
    fn looking_to(&self) -> Vec3 {
        match self {
            _ => Vec3::ZERO,
            Direction::Up => Vec3::new(0.0, 1.0, 0.0),
            _ => Vec3::ZERO
        }
    }
    fn transform(&self) -> (f32, f32, f32) {
        match self {
            _ => (0.0, 0.0, 0.0)
        }
    }
    pub fn list() -> [Direction; 6] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Right,
            Direction::Left,
            Direction::Front,
            Direction::Back
        ]
    }
}

impl Default for ChunkBlocs {
    fn default() -> Self {
        Self ([None; CHUNK_X*CHUNK_Y*CHUNK_Z])
    }
}
impl ChunkBlocs {
    pub fn get(&self, pos:&PosInChunk) -> Option<Entity> {
        self.get_raw(pos.to_chunk_index())
    }
    pub fn get_raw(&self, index:usize) -> Option<Entity> {
        match self.0.get(index) {
            None | Some(None) => None,
            Some(Some(b)) => Some(*b)
        }
    }
    pub fn set(&mut self, pos:&PosInChunk, val: Option<Entity>) {
        self.0[pos.to_chunk_index()] = val
    }
    pub fn render(&self, chunk_pos:&ChunkPos, asset_server: &Res<AssetServer>, blocs: &Query<&BlocType>, meshes: &mut ResMut<'_, Assets<Mesh>>, materials: &mut ResMut<'_, Assets<StandardMaterial>>, cmds: &mut Commands) -> ChunkFaces {
        let mut faces = Vec::new();
        for (i, bloc) in self.0.iter().enumerate() {
            if bloc.is_some() {
                continue
            }
            let pos = PosInChunk::from_chunk_index(i);
            for direction in Direction::list() {
                let (to_add, to_remove) = direction.index_difference();
                let mut index = pos.to_chunk_index();
                if to_remove > index {
                    continue
                }
                index += to_add;
                index -= to_remove;
                if index >= self.0.len() {
                    continue
                }
                if let Some(other_id) = self.get_raw(index) {
                    let other_bloc = blocs.get(other_id).expect("Trying to render a deleted bloc");
                    // load the texture
                    let texture_handle = asset_server.load(&format!("{}/{}.png", other_bloc.to_string(), direction.face_to_render_name()));
                    // create a new quad mesh. this is what we will apply the texture to
                    let quad_width = 8.0;
                    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                        quad_width,
                        quad_width
                    ))));
                    let material_handle = materials.add(StandardMaterial {
                        base_color_texture: Some(texture_handle.clone()),
                        ..default()
                    });
                    let (x, y, z) = direction.transform();
                    let id = cmds.spawn(PbrBundle {
                        mesh: quad_handle.clone(),
                        material: material_handle,
                        transform: Transform::from_xyz(
                            (chunk_pos.x as f32 + pos.x as f32 + x) * quad_width,
                            (chunk_pos.y as f32 + pos.y as f32 + y) * quad_width,
                            (chunk_pos.z as f32 + pos.z as f32 + z) * quad_width
                        ).looking_to(direction.looking_to(), Vec3::ZERO),
                        ..default()
                    }).id();
                    faces.push(id);
                }
            }
        }

        return ChunkFaces(faces)
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
    pub fn new_empty(pos: ChunkPos) -> Self {
        Self {
            pos,
            faces: ChunkFaces::default(),
            blocs: ChunkBlocs::default()
        }
    }
    pub fn new_with_blocs(pos: ChunkPos, blocs: ChunkBlocs) -> Self {
        Self {
            pos,
            blocs,
            faces: ChunkFaces::default()
        }
    }
    pub fn get(&self, pos:&PosInChunk) -> Option<Entity> {
        self.blocs.get(pos)
    }
    pub fn render(&mut self, asset_server: &Res<AssetServer>, blocs: &Query<&BlocType>, meshes: &mut ResMut<'_, Assets<Mesh>>, materials: &mut ResMut<'_, Assets<StandardMaterial>>, cmds: &mut Commands) {
        self.faces = self.blocs.render(&self.pos, asset_server, blocs, meshes, materials, cmds);
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
    pub fn get(&self, pos: &ChunkPos) -> Option<&Entity> {
        self.inner.get(pos)
    }
}