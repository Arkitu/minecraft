use bevy::{prelude::*, utils::HashMap};
use arr_macro::arr;

pub const CHUNK_X: usize = 4; // Right
pub const CHUNK_Y: usize = 8; // Up
pub const CHUNK_Z: usize = 4; // Front

pub const SQUARE_UNIT: f32 = 8.0;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum BlocType {
    Dirt,
    Grass,
    Stone,
    Air
}

impl Into<&str> for &BlocType {
    fn into(self) -> &'static str {
        match self {
            BlocType::Dirt => "dirt",
            BlocType::Grass => "grass",
            BlocType::Stone => "stone",
            BlocType::Air => "air"
        }
    }
}
impl std::fmt::Display for BlocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}

/// Absolute position of a bloc
#[derive(Component, Debug, Clone)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

#[derive(Component, Debug)]
pub struct Neighbors {
    up: Option<Entity>,
    down: Option<Entity>,
    right: Option<Entity>,
    left: Option<Entity>,
    front: Option<Entity>,
    back: Option<Entity>
}
impl Neighbors {
    fn get_with_direction(&self, direction:&Direction) -> Option<Entity> {
        match direction {
            Direction::Up => self.up,
            Direction::Down => self.down,
            Direction::Right => self.right,
            Direction::Left => self.left,
            Direction::Front => self.front,
            Direction::Back => self.back
        }
    }
}

#[derive(Component, Debug)]
pub struct BlocFaces (Vec<Entity>);

#[derive(Component, Debug)]
pub struct Face;

impl Default for BlocFaces {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[derive(Bundle)]
pub struct Bloc {
    pos: Pos,
    neighbors: Neighbors,
    r#type: BlocType,
    faces: BlocFaces
}

impl Bloc {
    fn render(&mut self, asset_server: &Res<AssetServer>, bloc_types_query: &Query<&BlocType>, meshes: &mut ResMut<'_, Assets<Mesh>>, materials: &mut ResMut<'_, Assets<StandardMaterial>>, cmds: &mut Commands) {
        render_bloc(&self.pos, &self.neighbors, &self.r#type, &mut self.faces, asset_server, bloc_types_query, meshes, materials, cmds)
    }
}

pub fn render_bloc(
    pos: &Pos,
    neighbors: &Neighbors,
    r#type: &BlocType,
    old_faces: &mut BlocFaces,
    asset_server: &Res<AssetServer>,
    bloc_types_query: &Query<&BlocType>,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<StandardMaterial>>,
    cmds: &mut Commands
) {
    if let BlocType::Air = r#type {
        return
    }
    let mut faces: Vec<Entity> = Vec::new();
    for direction in Direction::list() {
        let neighbor = match neighbors.get_with_direction(&direction) {
            Some(n) => bloc_types_query.get(n).unwrap(),
            None => &BlocType::Air
        };
        if neighbor != &BlocType::Air {
            continue
        }
        // load the texture
        let texture_handle = asset_server.load(&format!("{}/{}.png", r#type.to_string(), direction.face_to_render_name()));
        // create a new quad mesh. this is what we will apply the texture to
        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            SQUARE_UNIT,
            SQUARE_UNIT
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
                (pos.x as f32 + x) * SQUARE_UNIT,
                (pos.y as f32 + y) * SQUARE_UNIT,
                (pos.z as f32 + z) * SQUARE_UNIT
            ).looking_to(direction.looking_to(), Vec3::ZERO),
            ..default()
        }).id();
        faces.push(id);
    }
    for f in old_faces.0.iter() {
        cmds.entity(*f).despawn()
    }
    *old_faces = BlocFaces(faces);
}

/// Bloc position relative to the chunk corner
#[derive(Component, Debug, Clone, Copy)]
pub struct PosInChunk {
    pub x: u8,
    pub y: u8,
    pub z: u8
}

impl PosInChunk {
    pub fn to_chunk_index(&self) -> usize {
        self.x as usize
        + (self.y as usize * CHUNK_X)
        + (self.z as usize * CHUNK_X * CHUNK_Y)
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
    pub fn to_neighbor(&self, dir: Direction) -> Self {
        dir.get_other_coordinates(self)
    }
}

/// Chunk position in chunk unit
#[derive(Component, Eq, Hash, PartialEq, Clone, Copy)]
pub struct ChunkPos {
    pub x: i16,
    pub y: i16,
    pub z: i16
}
impl Into<Pos> for ChunkPos {
    fn into(self) -> Pos {
        Pos {
            x: self.x as i32 * CHUNK_X as i32,
            y: self.y as i32 * CHUNK_Y as i32,
            z: self.z as i32 * CHUNK_Z as i32
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    Up, // +y
    Down, // -y
    Right, // +x
    Left, // -x
    Front, // +z
    Back // -z
}
impl Direction {
    fn get_other_coordinates(&self, pos:&PosInChunk) -> PosInChunk {
        let mut new = pos.to_owned();
        match self {
            Direction::Up => new.y += 1,
            Direction::Down => new.y -= 1,
            Direction::Right => new.x += 1,
            Direction::Left => new.x -= 1,
            Direction::Front => new.z += 1,
            Direction::Back => new.z -= 1
        }
        new
    }
    fn face_to_render_name(&self) -> &'static str {
        match self {
            Direction::Up => "top",
            Direction::Down => "bottom",
            Direction::Right => "right",
            Direction::Left => "left",
            Direction::Front => "front",
            Direction::Back => "back"
        }
    }
    fn looking_to(&self) -> Vec3 {
        match self {
            Direction::Up => Vec3::new(0.0, -1.0, 0.0),
            Direction::Down => Vec3::new(0.0, 1.0, 0.0),
            Direction::Right => Vec3::new(-1.0, 0.0, 0.0),
            Direction::Left => Vec3::new(1.0, 0.0, 0.0),
            Direction::Front => Vec3::new(0.0, 0.0, -1.0),
            Direction::Back => Vec3::new(0.0, 0.0, 1.0)
        }
    }
    fn transform(&self) -> (f32, f32, f32) {
        match self {
            Direction::Up => (0.0, 0.5, 0.0),
            Direction::Down => (0.0, -0.5, 0.0),
            Direction::Right => (0.5, 0.0, 0.0),
            Direction::Left => (-0.5, 0.0, 0.0),
            Direction::Front => (0.0, 0.0, 0.5),
            Direction::Back => (0.0, 0.0, -0.5)
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

#[derive(Component)]
pub struct ChunkBlocs ([Entity; CHUNK_X*CHUNK_Y*CHUNK_Z]);

impl ChunkBlocs {
    pub fn from_inner(inner: [Entity; CHUNK_X*CHUNK_Y*CHUNK_Z]) -> Self {
        Self(inner)
    }
    pub fn new(chunk_pos: ChunkPos, types: &[BlocType; CHUNK_X*CHUNK_Y*CHUNK_Z], cmds: &mut Commands) -> Self {
        let entities = arr![{
            cmds.spawn_empty().id()
        }; 128]; // CHUNK_X*CHUNK_Y*CHUNK_Z
        for x in 0..CHUNK_X as u8 {
            for y in 0..CHUNK_Y as u8 {
                for z in 0..CHUNK_Z as u8 {
                    let pos_in_chunk = PosInChunk {
                        x,
                        y,
                        z
                    };
                    let chunk_index = pos_in_chunk.to_chunk_index();
                    let mut pos: Pos = chunk_pos.into();
                    pos.x += x as i32;
                    pos.y += y as i32;
                    pos.z += z as i32;
                    let bloc = Bloc {
                        pos,
                        neighbors: Neighbors {
                            up: if y == (CHUNK_Y-1) as u8 {
                                None
                            } else {
                                Some(entities[pos_in_chunk.to_neighbor(Direction::Up).to_chunk_index()])
                            },
                            down: if y == 0 {
                                None
                            } else {
                                Some(entities[pos_in_chunk.to_neighbor(Direction::Down).to_chunk_index()])
                            },
                            right: if x == (CHUNK_X-1) as u8 {
                                None
                            } else {
                                Some(entities[pos_in_chunk.to_neighbor(Direction::Right).to_chunk_index()])
                            },
                            left: if x == 0 {
                                None
                            } else {
                                Some(entities[pos_in_chunk.to_neighbor(Direction::Left).to_chunk_index()])
                            },
                            front: if z == (CHUNK_Z-1) as u8 {
                                None
                            } else {
                                Some(entities[pos_in_chunk.to_neighbor(Direction::Front).to_chunk_index()])
                            },
                            back: if z == 0 {
                                None
                            } else {
                                Some(entities[pos_in_chunk.to_neighbor(Direction::Back).to_chunk_index()])
                            },
                        },
                        r#type: types[chunk_index],
                        faces: BlocFaces::default()
                    };
                    cmds.get_entity(entities[chunk_index]).unwrap().insert(bloc);
                }
            }
        }
        Self(entities)
    }
    pub fn new_empty(chunk_pos: ChunkPos, cmds: &mut Commands) -> Self {
        Self::new(chunk_pos, &[BlocType::Air; CHUNK_X*CHUNK_Y*CHUNK_Z], cmds)
    }
    pub fn get(&self, pos:&PosInChunk) -> Option<&Entity> {
        self.0.get(pos.to_chunk_index())
    }
    pub fn set(&mut self, pos:&PosInChunk, val: Entity) {
        self.0[pos.to_chunk_index()] = val;
    }
    pub fn render(&self, asset_server: &Res<AssetServer>, blocs: &mut Query<(&Pos,&Neighbors,&BlocType,&mut BlocFaces)>, bloc_types_query: &Query<&BlocType>, meshes: &mut ResMut<'_, Assets<Mesh>>, materials: &mut ResMut<'_, Assets<StandardMaterial>>, cmds: &mut Commands) {
        for bloc in self.0.iter() {
            let (pos,neighbors,r#type,mut faces) = blocs.get_mut(*bloc).expect("Cannot find bloc from chunk");
            render_bloc(pos, neighbors, r#type, &mut faces, asset_server, bloc_types_query, meshes, materials, cmds);
        }
        
        // let mut faces = Vec::new();
        // for (i, bloc) in self.0.iter().enumerate() {
        //     if bloc.is_some() {
        //         continue
        //     }
        //     let pos = PosInChunk::from_chunk_index(i);
        //     if pos.x == CHUNK_X as u8 || pos.z == CHUNK_Z as u8 {
        //         continue
        //     }
        //     for direction in Direction::list() {
        //         if pos.x == 0 && direction == Direction::Left
        //         || pos.x == CHUNK_X as u8 - 1 && direction == Direction::Right
        //         || pos.y == 0 && direction == Direction::Down
        //         || pos.y == CHUNK_Y as u8 - 1 && direction == Direction::Up
        //         || pos.z == 0 && direction == Direction::Back
        //         || pos.z == CHUNK_Z as u8 - 1 && direction == Direction::Front {
        //             continue
        //         }
        //         if let Some(other_id) = self.get(&direction.get_other_coordinates(&pos)) {
        //             let other_bloc = blocs.get(other_id).expect("Trying to render a deleted bloc");
        //             // load the texture
        //             let texture_handle = asset_server.load(&format!("{}/{}.png", other_bloc.to_string(), direction.face_to_render_name()));
        //             // create a new quad mesh. this is what we will apply the texture to
        //             let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        //                 SQUARE_UNIT,
        //                 SQUARE_UNIT
        //             ))));
        //             let material_handle = materials.add(StandardMaterial {
        //                 base_color_texture: Some(texture_handle.clone()),
        //                 ..default()
        //             });
        //             let (x, y, z) = direction.transform();
        //             let id = cmds.spawn(PbrBundle {
        //                 mesh: quad_handle.clone(),
        //                 material: material_handle,
        //                 transform: Transform::from_xyz(
        //                     ((chunk_pos.x*CHUNK_X as i16) as f32 + pos.x as f32 + x) * SQUARE_UNIT,
        //                     ((chunk_pos.y*CHUNK_Y as i16) as f32 + pos.y as f32 + y) * SQUARE_UNIT,
        //                     ((chunk_pos.z*CHUNK_Z as i16) as f32 + pos.z as f32 + z) * SQUARE_UNIT
        //                 ).looking_to(direction.looking_to(), Vec3::ZERO),
        //                 ..default()
        //             }).id();
        //             faces.push(id);
        //         }
        //     }
        // }

        // return ChunkFaces(faces)
    }
}

#[derive(Component, Default)]
pub struct ChunkFaces (Vec<Entity>);

#[derive(Bundle)]
pub struct Chunk {
    blocs: ChunkBlocs,
    pos: ChunkPos
}
impl Chunk {
    pub fn new_empty(pos: ChunkPos, cmds: &mut Commands) -> Self {
        Self {
            pos,
            blocs: ChunkBlocs::new_empty(pos, cmds)
        }
    }
    pub fn new_with_blocs(pos: ChunkPos, blocs: ChunkBlocs) -> Self {
        Self {
            pos,
            blocs
        }
    }
    pub fn get(&self, pos:&PosInChunk) -> Option<&Entity> {
        self.blocs.get(pos)
    }
    pub fn render(&self, asset_server: &Res<AssetServer>, blocs: &mut Query<(&Pos,&Neighbors,&BlocType,&mut BlocFaces)>, bloc_types_query: &Query<&BlocType>, meshes: &mut ResMut<'_, Assets<Mesh>>, materials: &mut ResMut<'_, Assets<StandardMaterial>>, cmds: &mut Commands) {
        self.blocs.render(asset_server, blocs, bloc_types_query, meshes, materials, cmds);
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
    pub fn get(&self, pos: ChunkPos) -> Option<&Entity> {
        self.inner.get(&pos)
    }
    pub fn generate(&mut self, pos: ChunkPos, cmds: &mut Commands, chunks_query: &Query<(&ChunkPos, &ChunkBlocs)>) {
        // return if there is already a chunk
        if let Some(_) = self.get(pos) {
            return
        }
        let mut types = [BlocType::Air; CHUNK_X*CHUNK_Y*CHUNK_Z];

        for x in 0..CHUNK_X as u8 {
            for z in 0..CHUNK_Z as u8 {
                for y in 1..3 {
                    types[PosInChunk { x, y, z }.to_chunk_index()] = BlocType::Stone;
                }
                for y in 3..4 {
                    types[PosInChunk { x, y, z }.to_chunk_index()] = BlocType::Grass;
                }
            }
        }
        let blocs = ChunkBlocs::new(pos, &types, cmds);

        let chunk = Chunk::new_with_blocs(pos, blocs);
        self.insert(pos, cmds.spawn(chunk).id());
    }
}