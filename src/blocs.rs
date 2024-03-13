use bevy::{ecs::query::{QueryEntityError}, prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;
use arr_macro::arr;

pub const CHUNK_X: usize = 8; // Right
pub const CHUNK_Y: usize = 16; // Up
pub const CHUNK_Z: usize = 8; // Front

pub const SQUARE_UNIT: f32 = 1.0;

pub const BLOCS_PHYSIC_GROUP: Group = Group::GROUP_1;

pub type DefaultGenerator = generation::Generator;

pub mod cracks;
pub use cracks::*;
pub mod loading;
pub use loading::*;
pub mod generation;
pub use generation::*;

use serde::{Deserialize, Serialize};

use crate::{ChunkSave, ChunkSaves, ChunkTypes, GameState};

pub struct BlocAndChunkPlugin;
impl Plugin for BlocAndChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CracksPlugin)
            .add_plugins(LoadingPlugin)
            .add_systems(Update, apply_next_material)
            .add_systems(Update, link_chunks::<DefaultGenerator>);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlocType {
    Dirt,
    Grass,
    Stone,
    Sand,
    SnowyDirt,
    Air,
}

impl Into<&str> for &BlocType {
    fn into(self) -> &'static str {
        match self {
            BlocType::Dirt => "dirt",
            BlocType::Grass => "grass",
            BlocType::Stone => "stone",
            BlocType::Sand => "sand",
            BlocType::SnowyDirt => "snowy_dirt",
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
impl Into<Transform> for Pos {
    fn into(self) -> Transform {
        Transform::from_xyz(self.x as f32, self.y as f32, self.z as f32)
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct Neighbors {
    pub up: Option<Entity>,
    pub down: Option<Entity>,
    pub right: Option<Entity>,
    pub left: Option<Entity>,
    pub front: Option<Entity>,
    pub back: Option<Entity>
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
    fn list(&self) -> [&Option<Entity>; 6] {
        [
            &self.up,
            &self.down,
            &self.right,
            &self.left,
            &self.front,
            &self.back
        ]
    }
}

#[derive(Component, Debug)]
pub struct BlocFaces (pub Vec<Entity>);

#[derive(Component, Debug)]
pub struct Face;

impl Default for BlocFaces {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[derive(Component)]
pub struct FaceMarker;

#[derive(Component)]
pub struct BaseMaterial(pub Handle<StandardMaterial>);

#[derive(Component)]
pub struct NextMaterial(pub Option<Handle<StandardMaterial>>);

#[derive(Component, PartialEq, Eq, Hash)]
pub enum DestructionLevel {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five
}

#[derive(Bundle)]
pub struct Bloc {
    pos_in_chunk: PosInChunk,
    neighbors: Neighbors,
    r#type: BlocType,
    faces: BlocFaces,
    rigid_body: RigidBody,
    spatial: SpatialBundle,
    collision_groups: CollisionGroups
}

pub enum BlocTypeQuery<'a, 'b, 'world, 'state> {
    Simple(&'a Query<'world, 'state, &'b BlocType>),
    Mut(&'a Query<'world, 'state, &'b mut BlocType>),
    Complex1(&'a Query<'world, 'state, (Entity,&'b mut Neighbors,&'b mut BlocType,&'b mut BlocFaces)>),
    Complex2(&'a Query<'world, 'state, (Entity,&'b Neighbors,&'b BlocType,&'b mut BlocFaces)>)
}
impl BlocTypeQuery<'_, '_, '_, '_> {
    pub fn get(&self, entity: Entity) -> Result<&BlocType, QueryEntityError> {
        match self {
            BlocTypeQuery::Simple(q) => q.get(entity),
            BlocTypeQuery::Mut(q) => q.get(entity),
            BlocTypeQuery::Complex1(q) => q.get(entity).map(|e|{e.2}),
            BlocTypeQuery::Complex2(q) => q.get(entity).map(|e|{e.2})
        }
    }
}

pub fn render_bloc(
    bloc_entity: Entity,
    neighbors: &Neighbors,
    old_faces: &mut BlocFaces,
    asset_server: &Res<AssetServer>,
    bloc_types_query: BlocTypeQuery,
    meshes: &mut ResMut<'_, Assets<Mesh>>,
    materials: &mut ResMut<'_, Assets<StandardMaterial>>,
    cmds: &mut Commands,
    // None: don't touch to the physic / Some<true>: render the physic / Some<false>: don't render the physic
    overwrite_physic: Option<bool>
) {
    let r#type = bloc_types_query.get(bloc_entity).unwrap();
    cmds.entity(bloc_entity).despawn_descendants();
    // if let Some(false) = overwrite_physic {
    //     cmds.entity(bloc_entity).remove::<Collider>();
    // }
    if let BlocType::Air = r#type {
        // if let Some(true) = overwrite_physic {
        //     cmds.entity(bloc_entity).remove::<Collider>();
        // }
        return
    }
    let mut faces: Vec<Entity> = Vec::new();
    // Add collider for outside blocs only for optimization
    let mut needs_new_collider = false;
    for direction in Direction::list() {
        let neighbor = match neighbors.get_with_direction(&direction) {
            Some(n) => bloc_types_query.get(n).unwrap(),
            None => if direction == Direction::Up {
                &BlocType::Air
            } else {
                continue
            }
        };
        if neighbor != &BlocType::Air {
            continue
        }
        needs_new_collider = overwrite_physic.unwrap_or(false);
        // load the texture
        let texture_handle = asset_server.load(&format!("{}/{}.png", r#type.to_string(), direction.face_to_render_name()));
        
        // create a new quad mesh. this is what we will apply the texture to
        let quad_handle = meshes.add(
            Mesh::from(
                Rectangle::new(SQUARE_UNIT, SQUARE_UNIT)
            )
        );
        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            ..default()
        });
        let (x, y, z) = direction.transform();
        let id = cmds.spawn((PbrBundle {
            mesh: quad_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(
                x * SQUARE_UNIT,
                y * SQUARE_UNIT,
                z * SQUARE_UNIT
            ).looking_to(direction.looking_to(), Vec3::ZERO),
            ..default()
        }, FaceMarker, DestructionLevel::Zero, BaseMaterial(material_handle), NextMaterial(None))).id();
        faces.push(id);
    }
    let mut cmd = cmds.entity(bloc_entity);
    cmd.push_children(&faces);
    *old_faces = BlocFaces(faces);

    // if needs_new_collider {
    //     cmd.insert(Collider::cuboid(SQUARE_UNIT/2.0, SQUARE_UNIT/2.0, SQUARE_UNIT/2.0));
    // }
}

pub fn load_physic(
    bloc_entity: Entity,
    neighbors: &Neighbors,
    bloc_types_query: BlocTypeQuery,
    cmds: &mut Commands
) {
    let r#type = bloc_types_query.get(bloc_entity).unwrap();
    if let BlocType::Air = r#type {
        cmds.entity(bloc_entity).remove::<Collider>();
        return
    }
    // Add collider for outside blocs only for optimization
    for direction in Direction::list() {
        let neighbor = match neighbors.get_with_direction(&direction) {
            Some(n) => bloc_types_query.get(n).unwrap(),
            None => if direction == Direction::Up {
                &BlocType::Air
            } else {
                continue
            }
        };
        if neighbor != &BlocType::Air {
            continue
        }
        cmds.entity(bloc_entity).insert(Collider::cuboid(SQUARE_UNIT/2.0, SQUARE_UNIT/2.0, SQUARE_UNIT/2.0));
        return
    }
    cmds.entity(bloc_entity).remove::<Collider>();
}
pub fn unload_physic(
    bloc_entity: Entity,
    cmds: &mut Commands
) {
    cmds.entity(bloc_entity).remove::<Collider>();
}

pub fn remove_bloc(
    entity: Entity,
    neighbors: &Neighbors,
    blocs: &mut Query<(Entity,&mut Neighbors,&mut BlocFaces)>,
    blocs_types_query: &mut Query<&mut BlocType>,
    blocs_pos_parent_query: &Query<(&PosInChunk, &Parent), With<BlocType>>,
    chunk_pos_query: &Query<&ChunkPos>,
    game_state: &mut ResMut<GameState>,
    chunk_saves: &mut ResMut<ChunkSaves>,
    cmds: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let (bloc_entity, mut neighbors_mut, mut faces) = blocs.get_mut(entity).unwrap();
    *blocs_types_query.get_mut(entity).unwrap() = BlocType::Air;
    render_bloc(bloc_entity, &mut neighbors_mut, &mut faces, asset_server, BlocTypeQuery::Mut(blocs_types_query), meshes, materials, cmds, Some(true));

    let (pos, parent) = blocs_pos_parent_query.get(entity).unwrap();
    let chunk_pos = chunk_pos_query.get(parent.get()).unwrap();
    game_state.chunks.get_mut(chunk_pos).unwrap().0[pos.to_chunk_index()] = BlocType::Air;

    match chunk_saves.0.get_mut(chunk_pos) {
        Some(entry) => {
            entry.changes.insert(*pos, BlocType::Air);
        },
        None => {
            let mut entry = ChunkSave::default();
            entry.changes.insert(*pos, BlocType::Air);
            chunk_saves.0.insert(*chunk_pos, entry);
        }
    }

    if let Some(n) = &neighbors.up {
        let (n_bloc_entity, mut n_neighbors, mut n_faces) = blocs.get_mut(*n).unwrap();
        render_bloc(n_bloc_entity, &mut n_neighbors, &mut n_faces, &asset_server, BlocTypeQuery::Mut(blocs_types_query), meshes, materials, cmds, None);
    }
    if let Some(n) = &neighbors.down {
        let (n_bloc_entity, mut n_neighbors, mut n_faces) = blocs.get_mut(*n).unwrap();
        render_bloc(n_bloc_entity, &mut n_neighbors, &mut n_faces, &asset_server, BlocTypeQuery::Mut(blocs_types_query), meshes, materials, cmds, None);
    }
    if let Some(n) = &neighbors.left {
        let (n_bloc_entity, mut n_neighbors, mut n_faces) = blocs.get_mut(*n).unwrap();
        render_bloc(n_bloc_entity, &mut n_neighbors, &mut n_faces, &asset_server, BlocTypeQuery::Mut(blocs_types_query), meshes, materials, cmds, None);
    }
    if let Some(n) = &neighbors.right {
        let (n_bloc_entity, mut n_neighbors, mut n_faces) = blocs.get_mut(*n).unwrap();
        render_bloc(n_bloc_entity, &mut n_neighbors, &mut n_faces, &asset_server, BlocTypeQuery::Mut(blocs_types_query), meshes, materials, cmds, None);
    }
    if let Some(n) = &neighbors.front {
        let (n_bloc_entity, mut n_neighbors, mut n_faces) = blocs.get_mut(*n).unwrap();
        render_bloc(n_bloc_entity, &mut n_neighbors, &mut n_faces, &asset_server, BlocTypeQuery::Mut(blocs_types_query), meshes, materials, cmds, None);
    }
    if let Some(n) = &neighbors.back {
        let (n_bloc_entity, mut n_neighbors, mut n_faces) = blocs.get_mut(*n).unwrap();
        render_bloc(n_bloc_entity, &mut n_neighbors, &mut n_faces, &asset_server, BlocTypeQuery::Mut(blocs_types_query), meshes, materials, cmds, None);
    }
}

/// Bloc position relative to the chunk corner
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PosInChunk {
    pub x: u8,
    pub y: u8,
    pub z: u8
}
impl Into<Transform> for PosInChunk {
    fn into(self) -> Transform {
        Transform::from_xyz(
            self.x as f32 * SQUARE_UNIT,
            self.y as f32 * SQUARE_UNIT,
            self.z as f32 * SQUARE_UNIT
        )
    }
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
#[derive(Component, Eq, Hash, PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32
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
impl Into<Transform> for ChunkPos {
    fn into(self) -> Transform {
        Into::<Pos>::into(self).into()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

#[derive(Component, Debug, Clone)]
pub struct ChunkBlocs ([Entity; CHUNK_X*CHUNK_Y*CHUNK_Z]);

impl ChunkBlocs {
    pub fn from_inner(inner: [Entity; CHUNK_X*CHUNK_Y*CHUNK_Z]) -> Self {
        Self(inner)
    }
    pub fn new(chunk_pos: ChunkPos, types: &[BlocType; CHUNK_X*CHUNK_Y*CHUNK_Z], cmds: &mut Commands) -> Self {
        let entities = arr![{
            cmds.spawn_empty().id()
        }; 1024]; // CHUNK_X*CHUNK_Y*CHUNK_Z
        for x in 0..CHUNK_X as u8 {
            for y in 0..CHUNK_Y as u8 {
                for z in 0..CHUNK_Z as u8 {
                    let pos_in_chunk = PosInChunk {
                        x,
                        y,
                        z
                    };
                    let chunk_index = pos_in_chunk.to_chunk_index();
                    let bloc = Bloc {
                        pos_in_chunk: pos_in_chunk.clone(),
                        spatial: SpatialBundle::from_transform(pos_in_chunk.into()),
                        rigid_body: RigidBody::Fixed,
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
                        faces: BlocFaces::default(),
                        collision_groups: CollisionGroups::new(BLOCS_PHYSIC_GROUP, Group::complement(BLOCS_PHYSIC_GROUP))
                    };
                    let mut entity = cmds.get_entity(entities[chunk_index]).unwrap();
                    entity.insert(bloc);
                    if types[chunk_index] != BlocType::Air {
                        entity.insert(Collider::cuboid(SQUARE_UNIT/2.0, SQUARE_UNIT/2.0, SQUARE_UNIT/2.0));
                    }
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
    pub fn render(&self, asset_server: &Res<AssetServer>, blocs: &mut Query<(Entity,&Neighbors,&mut BlocFaces)>, bloc_types_query: &Query<&BlocType>, meshes: &mut ResMut<'_, Assets<Mesh>>, materials: &mut ResMut<'_, Assets<StandardMaterial>>, cmds: &mut Commands,  physic_overwride: Option<bool>) {
        for bloc in self.0.iter() {
            let (bloc_entity, neighbors,mut faces) = blocs.get_mut(*bloc).expect("Cannot find bloc from chunk");
            render_bloc(bloc_entity,  neighbors, &mut faces, asset_server, BlocTypeQuery::Simple(bloc_types_query), meshes, materials, cmds, physic_overwride);
        }
    }
    pub fn load_physic(&self, blocs: &Query<(Entity,&Neighbors)>, bloc_types_query: &Query<&BlocType>, cmds: &mut Commands) {
        for bloc in self.0.iter() {
            let (bloc_entity, neighbors) = blocs.get(*bloc).expect("Cannot find bloc from chunk");
            load_physic(bloc_entity,  neighbors, BlocTypeQuery::Simple(bloc_types_query), cmds);
        }
    }
    pub fn unload_physic(&self, cmds: &mut Commands) {
        for bloc in self.0.iter() {
            unload_physic(*bloc, cmds);
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ChunkNeighborsAreLinked {
    up: bool,
    right: bool,
    front: bool,
}
impl Default for ChunkNeighborsAreLinked {
    fn default() -> Self {
        Self {
            up: false,
            right: false,
            front: false
        }
    }
}

#[derive(Bundle)]
pub struct Chunk {
    spatial: SpatialBundle,
    blocs: ChunkBlocs,
    pos: ChunkPos,
    neighbors_are_linked: ChunkNeighborsAreLinked
}
impl Chunk {
    pub fn new_empty(pos: ChunkPos, cmds: &mut Commands) -> Self {
        Self {
            spatial: SpatialBundle::from_transform(pos.into()),
            pos,
            blocs: ChunkBlocs::new_empty(pos, cmds),
            neighbors_are_linked: ChunkNeighborsAreLinked::default()
        }
    }
    pub fn new_with_blocs(pos: ChunkPos, blocs: ChunkBlocs) -> Self {
        Self {
            spatial: SpatialBundle::from_transform(pos.into()),
            pos,
            blocs,
            neighbors_are_linked: ChunkNeighborsAreLinked::default()
        }
    }
    pub fn get(&self, pos:&PosInChunk) -> Option<&Entity> {
        self.blocs.get(pos)
    }
    pub fn render(&self, asset_server: &Res<AssetServer>, blocs: &mut Query<(Entity,&Neighbors,&mut BlocFaces)>, bloc_types_query: &Query<&BlocType>, meshes: &mut ResMut<'_, Assets<Mesh>>, materials: &mut ResMut<'_, Assets<StandardMaterial>>, cmds: &mut Commands, physic_overwride: Option<bool>) {
        self.blocs.render(asset_server, blocs, bloc_types_query, meshes, materials, cmds, physic_overwride);
    }
}

pub trait Generator: Send + std::marker::Sync + 'static {
    fn new(seed: u32) -> Self;
    fn generate(&self, pos: ChunkPos) -> [BlocType; CHUNK_X*CHUNK_Y*CHUNK_Z];
}

pub struct FlatWordGenerator;
impl Default for FlatWordGenerator {
    fn default() -> Self {
        Self
    }
}
impl Generator for FlatWordGenerator {
    fn new(_: u32) -> Self {
        Self::default()
    }
    fn generate(&self, _: ChunkPos) -> [BlocType; CHUNK_X*CHUNK_Y*CHUNK_Z] {
        let mut types = [BlocType::Air; CHUNK_X*CHUNK_Y*CHUNK_Z];
        for x in 0..CHUNK_X as u8 {
            for z in 0..CHUNK_Z as u8 {
                for y in 0..3 {
                    types[PosInChunk { x, y, z }.to_chunk_index()] = BlocType::Stone;
                }
                for y in 3..4 {
                    types[PosInChunk { x, y, z }.to_chunk_index()] = BlocType::Grass;
                }
            }
        }
        types[PosInChunk { x:1, y:4, z:1 }.to_chunk_index()] = BlocType::Stone;
        return types
    }
}

#[derive(Resource)]
pub struct Chunks<G: Generator> {
    pub inner: HashMap<ChunkPos, Entity>,
    pub generator: G
}
impl<G: Generator> Chunks<G> {
    pub fn new(seed: u32) -> Self {
        Self {
            inner: HashMap::new(),
            generator: G::new(seed)
        }
    }
    pub fn insert(&mut self, pos: ChunkPos, chunk: Entity) {
        self.inner.insert(pos, chunk);
    }
    pub fn get(&self, pos: ChunkPos) -> Option<&Entity> {
        self.inner.get(&pos)
    }
    pub fn clear(&mut self, cmds: &mut Commands) {
        for (_, entity) in self.inner.iter() {
            cmds.entity(*entity).despawn_recursive();
        }
        self.inner.clear()
    }
    pub fn load_types(&mut self, pos: ChunkPos, types: &[BlocType; CHUNK_X*CHUNK_Y*CHUNK_Z], cmds: &mut Commands) {
        if let Some(_) = self.get(pos) {
            return
        }
        let blocs = ChunkBlocs::new(pos, &types, cmds);

        let mut cmd = cmds.spawn_empty();
        cmd.push_children(&blocs.0);
        let chunk = Chunk::new_with_blocs(pos, blocs);
        cmd.insert(chunk);
        self.insert(pos, cmd.id());
    }
    /// Panics if game_state doesn't contain the chunk bloc types
    pub fn load(&mut self, pos: ChunkPos, game_state: &GameState, cmds: &mut Commands) {
        if let Some(_) = self.get(pos) {
            return
        }
        let types = game_state.chunks.get(&pos).unwrap().0;
        self.load_types(pos, &types, cmds);
    }
    pub fn generate(&mut self, pos: ChunkPos, chunk_saves: &ChunkSaves, game_state: &mut GameState, cmds: &mut Commands) {
        // return if there is already a chunk
        if let Some(_) = self.get(pos) {
            return
        }
        let mut types = self.generator.generate(pos);
        if let Some(save) = chunk_saves.0.get(&pos) {
            for (pos, r#type) in save.changes.iter() {
                types[pos.to_chunk_index()] = *r#type;
            }
        }
        game_state.chunks.insert(pos, ChunkTypes(types));
        self.load_types(pos, &types, cmds);
    }
    pub fn load_or_generate(&mut self, pos: ChunkPos, chunk_saves: &ChunkSaves, game_state: &mut GameState, cmds: &mut Commands) {
        if let Some(_) = self.get(pos) {
            return
        }
        let types = match game_state.chunks.get(&pos) {
            Some(types) => types.0,
            None => {
                let mut types = self.generator.generate(pos);
                if let Some(save) = chunk_saves.0.get(&pos) {
                    for (pos, r#type) in save.changes.iter() {
                        types[pos.to_chunk_index()] = *r#type;
                    }
                }
                game_state.chunks.insert(pos, ChunkTypes(types));
                types
            }
        };
        self.load_types(pos, &types, cmds);
    }
    pub fn unload(&mut self, pos: ChunkPos, chunks_query: &mut Query<(&ChunkBlocs, &mut ChunkNeighborsAreLinked)>, blocs_query: &mut Query<&mut Neighbors>, cmds: &mut Commands) {
        let entity = *self.get(pos).unwrap();
        let (blocs, nal) = chunks_query.get(entity).unwrap();
        let blocs = (*blocs).clone();
        let nal = *nal;
        // up
        if nal.up {
            let mut pos2 = pos;
            pos2.y += 1;
            let entity2 = *self.get(pos2).unwrap();
            let blocs2 = chunks_query.get(entity2).unwrap().0;
            self.unlink(pos, pos2, &blocs, blocs2, blocs_query);
        }
        // right
        if nal.right {
            let mut pos2 = pos;
            pos2.x += 1;
            let entity2 = *self.get(pos2).unwrap();
            let blocs2 = chunks_query.get(entity2).unwrap().0;
            self.unlink(pos, pos2, &blocs, blocs2, blocs_query);
        }
        // front
        if nal.front {
            let mut pos2 = pos;
            pos2.z += 1;
            let entity2 = *self.get(pos2).unwrap();
            let blocs2 = chunks_query.get(entity2).unwrap().0;
            self.unlink(pos, pos2, &blocs, blocs2, blocs_query);
        }
        // down
        let mut pos2 = pos;
        pos2.y -= 1;
        if let Some(entity2) = self.get(pos2) {
            if let Ok((blocs2, mut nal2)) = chunks_query.get_mut(*entity2) {
                if nal2.up {
                    self.unlink(pos, pos2, &blocs, blocs2, blocs_query);
                    nal2.up = false;
                }
            }
        }
        // left
        let mut pos2 = pos;
        pos2.x -= 1;
        if let Some(entity2) = self.get(pos2) {
            if let Ok((blocs2, mut nal2)) = chunks_query.get_mut(*entity2) {
                if nal2.right {
                    self.unlink(pos, pos2, &blocs, blocs2, blocs_query);
                    nal2.right = false;
                }
            }
        }
        // back
        let mut pos2 = pos;
        pos2.z -= 1;
        if let Some(entity2) = self.get(pos2) {
            if let Ok((blocs2, mut nal2)) = chunks_query.get_mut(*entity2) {
                if nal2.front {
                    self.unlink(pos, pos2, &blocs, blocs2, blocs_query);
                    nal2.front = false;
                }
            }
        }

        cmds.entity(entity).despawn_recursive();
        self.inner.remove(&pos);
    }
    /// * Fill the neighbors of the edge blocs for each chunk
    /// * /!\ This assumes that the blocs are already spawned and that the chunks are neighbors
    /// * Errors if the chunks are not spawned
    pub fn link(&self, pos1: ChunkPos, pos2: ChunkPos, blocs1: &ChunkBlocs, blocs2: &ChunkBlocs, blocs_query: &mut Query<(&mut Neighbors, &mut BlocFaces)>, asset_server: &Res<AssetServer>, bloc_types_query: &Query<&BlocType>, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, cmds: &mut Commands) {
        let x_iter = if pos1.x < pos2.x {
            (CHUNK_X as u8-1..=CHUNK_X as u8-1).into_iter().zip(0..=0)
        } else if pos1.x > pos2.x {
            (0..=0).into_iter().zip(CHUNK_X as u8-1..=CHUNK_X as u8-1)
        } else {
            (0..=CHUNK_X as u8-1).into_iter().zip(0..=CHUNK_X as u8-1)
        };
        let y_iter = if pos1.y < pos2.y {
            (CHUNK_Y as u8-1..=CHUNK_Y as u8-1).into_iter().zip(0..=0)
        } else if pos1.y > pos2.y {
            (0..=0).into_iter().zip(CHUNK_Y as u8-1..=CHUNK_Y as u8-1)
        } else {
            (0..=CHUNK_Y as u8-1).into_iter().zip(0..=CHUNK_Y as u8-1)
        };
        let z_iter = if pos1.z < pos2.z {
            (CHUNK_Z as u8-1..=CHUNK_Z as u8-1).into_iter().zip(0..=0)
        } else if pos1.z > pos2.z {
            (0..=0).into_iter().zip(CHUNK_Z as u8-1..=CHUNK_Z as u8-1)
        } else {
            (0..=CHUNK_Z as u8-1).into_iter().zip(0..=CHUNK_Z as u8-1)
        };
        for x in x_iter {
            for y in y_iter.clone() {
                for z in z_iter.clone() {
                    let entities = (
                        *blocs1.get(&PosInChunk { x: x.0, y: y.0, z: z.0 }).expect("Cannot find bloc 1"),
                        *blocs2.get(&PosInChunk { x: x.1, y: y.1, z: z.1 }).expect("Cannot find bloc 2")
                    );
                    let mut blocs = blocs_query.get_many_mut([entities.0, entities.1]).expect("Cannot find neighbors");
                    if pos1.x < pos2.x {
                        blocs[0].0.right = Some(entities.1);
                        blocs[1].0.left = Some(entities.0);
                    } else if pos1.x > pos2.x {
                        blocs[0].0.left = Some(entities.1);
                        blocs[1].0.right = Some(entities.0);
                    } else if pos1.y < pos2.y {
                        blocs[0].0.up = Some(entities.1);
                        blocs[1].0.down = Some(entities.0);
                    } else if pos1.y > pos2.y {
                        blocs[0].0.down = Some(entities.1);
                        blocs[1].0.up = Some(entities.0);
                    } else if pos1.z < pos2.z {
                        blocs[0].0.front = Some(entities.1);
                        blocs[1].0.back = Some(entities.0);
                    } else if pos1.z > pos2.z {
                        blocs[0].0.back = Some(entities.1);
                        blocs[1].0.front = Some(entities.0);
                    }
                    render_bloc(entities.0, &blocs[0].0, &mut blocs[0].1, asset_server, BlocTypeQuery::Simple(bloc_types_query), meshes, materials, cmds, None);
                    render_bloc(entities.1, &blocs[1].0, &mut blocs[1].1, asset_server, BlocTypeQuery::Simple(bloc_types_query), meshes, materials, cmds, None)
                }
            }
        }
    }
    pub fn unlink(&self, pos1: ChunkPos, pos2: ChunkPos, blocs1: &ChunkBlocs, blocs2: &ChunkBlocs, blocs_query: &mut Query<&mut Neighbors>) {
        let x_iter = if pos1.x < pos2.x {
            (CHUNK_X as u8-1..=CHUNK_X as u8-1).into_iter().zip(0..=0)
        } else if pos1.x > pos2.x {
            (0..=0).into_iter().zip(CHUNK_X as u8-1..=CHUNK_X as u8-1)
        } else {
            (0..=CHUNK_X as u8-1).into_iter().zip(0..=CHUNK_X as u8-1)
        };
        let y_iter = if pos1.y < pos2.y {
            (CHUNK_Y as u8-1..=CHUNK_Y as u8-1).into_iter().zip(0..=0)
        } else if pos1.y > pos2.y {
            (0..=0).into_iter().zip(CHUNK_Y as u8-1..=CHUNK_Y as u8-1)
        } else {
            (0..=CHUNK_Y as u8-1).into_iter().zip(0..=CHUNK_Y as u8-1)
        };
        let z_iter = if pos1.z < pos2.z {
            (CHUNK_Z as u8-1..=CHUNK_Z as u8-1).into_iter().zip(0..=0)
        } else if pos1.z > pos2.z {
            (0..=0).into_iter().zip(CHUNK_Z as u8-1..=CHUNK_Z as u8-1)
        } else {
            (0..=CHUNK_Z as u8-1).into_iter().zip(0..=CHUNK_Z as u8-1)
        };
        for x in x_iter {
            for y in y_iter.clone() {
                for z in z_iter.clone() {
                    let blocs = (
                        *blocs1.get(&PosInChunk { x: x.0, y: y.0, z: z.0 }).expect("Cannot find bloc 1"),
                        *blocs2.get(&PosInChunk { x: x.1, y: y.1, z: z.1 }).expect("Cannot find bloc 2")
                    );
                    let mut neighbors = blocs_query.get_many_mut([blocs.0, blocs.1]).expect("Cannot find neighbors");
                    if pos1.x < pos2.x {
                        neighbors[0].right = None;
                        neighbors[1].left = None
                    } else if pos1.x > pos2.x {
                        neighbors[0].left = None;
                        neighbors[1].right = None;
                    } else if pos1.y < pos2.y {
                        neighbors[0].up = None;
                        neighbors[1].down = None;
                    } else if pos1.y > pos2.y {
                        neighbors[0].down = None;
                        neighbors[1].up = None;
                    } else if pos1.z < pos2.z {
                        neighbors[0].front = None;
                        neighbors[1].back = None;
                    } else if pos1.z > pos2.z {
                        neighbors[0].back = None;
                        neighbors[1].front = None;
                    }
                }
            }
        }
    }
}

pub fn apply_next_material(
    mut faces: Query<(&mut Handle<StandardMaterial>, &mut NextMaterial, &BaseMaterial), With<FaceMarker>>,
    asset_server: Res<AssetServer>
) {
    for (mut face, mut next_mat, base_mat) in faces.iter_mut() {
        if let Some(nm) = &next_mat.0 {
            if asset_server.is_loaded_with_dependencies(nm.id()) || nm == &base_mat.0 {
                *face = nm.clone();
                next_mat.0 = None;
            }
        }
    }
}

pub fn link_chunks<G: Generator>(
    chunks: Res<Chunks<G>>,
    mut nal_query: Query<(Entity, &mut ChunkNeighborsAreLinked)>,
    chunks_query: Query<(&ChunkPos, &ChunkBlocs)>,
    mut blocs_query: Query<(&mut Neighbors, &mut BlocFaces)>,
    asset_server: Res<AssetServer>,
    bloc_types_query: Query<&BlocType>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cmds: Commands
) {
    for (id, mut nal) in nal_query.iter_mut() {
        if nal.up && nal.right && nal.front {
            continue
        }
        let (pos1, blocs1) = chunks_query.get(id).unwrap();
        let mut neighbors = Vec::new();
        if !nal.up {
            neighbors.push((ChunkPos {
                x: pos1.x,
                y: pos1.y + 1,
                z: pos1.z
            }, 0));
        }
        if !nal.right {
            neighbors.push((ChunkPos {
                x: pos1.x + 1,
                y: pos1.y,
                z: pos1.z
            }, 1));
        }
        if !nal.front {
            neighbors.push((ChunkPos {
                x: pos1.x,
                y: pos1.y,
                z: pos1.z + 1
            }, 2));
        }
        for (pos2, val_to_change) in neighbors {
            let (_, blocs2) = match chunks_query.get(
                *match chunks.get(pos2) {
                    Some(c) => c,
                    None => continue
                }
            ) {
                Ok(x) => x,
                Err(_) => continue
            };
            chunks.link(*pos1, pos2, blocs1, blocs2, &mut blocs_query, &asset_server, &bloc_types_query, &mut meshes, &mut materials, &mut cmds);
            match val_to_change {
                0 => nal.up = true,
                1 => nal.right = true,
                2 => nal.front = true,
                _ => {}
            }
        }
    }
}