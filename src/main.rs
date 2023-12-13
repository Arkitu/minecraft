use bevy::{prelude::*, utils::HashMap};

#[derive(Component)]
struct CameraMarker;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 12.0, 16.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraMarker,
    ));

    // sun light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // a random cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}

enum BlocType {
    Dirt,
    Stone
}

struct BlocPosition {
    x: i32,
    y: i32,
    z: i32
}

struct ChunkPosition {
    x: i32,
    y: i32,
    z: i32
}

/// use Option<Bloc>, there isn't any "air" bloc
struct Bloc {
    r#type: BlocType,
    pos: BlocPosition
}

#[derive(Component)]
struct Chunk {
    inner: [[[Option<Entity>; 16]; 16]; 256],
    pos: ChunkPosition
}
impl Chunk {
    pub fn new(pos: ChunkPosition) -> Self {
        Self {
            inner: [[[None; 16]; 16]; 256],
            pos
        }
    }
}

#[derive(Resource)]
struct Chunks {
    inner: HashMap<ChunkPosition, Entity>,
    seed: u64
}
impl Chunks {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: HashMap::new(),
            seed
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .insert_resource(Chunks::new(0))
        .run();
}