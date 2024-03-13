use bevy::prelude::*;
use super::{Generator as GeneratorTrait, BlocType, CHUNK_X, CHUNK_Y, CHUNK_Z, ChunkPos, PosInChunk};
use noise::{Fbm, Perlin};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};

pub enum Biome {
    Plain,
    Forest,
    Desert,
    SnowyPlain
}
impl Biome {
    pub fn new(temp: f32, rain: f32) -> Self {
        if temp < 0.2 {
            Self::SnowyPlain
        } else if temp > 0.7 {
            Self::Desert
        } else if rain < 0.5 {
            Self::Plain
        } else {
            Self::Forest
        }
    }
    pub fn top_block(&self) -> BlocType {
        match self {
            Self::Plain => BlocType::Dirt,
            Self::Forest => BlocType::Grass,
            Self::Desert => BlocType::Sand,
            Self::SnowyPlain => BlocType::SnowyDirt
        }
    }
    /// Average height between 0.0 and 1.0
    pub fn avg_height(temp: f32, rain: f32) -> f32 {
        0.35 - (temp * 0.05) + (rain * 0.05)
    }
    /// Indicate if the height vary a lot or not between 0.0 and +Infinity
    pub fn height_variance(temp: f32, rain: f32) -> f32 {
        let p = Vec2::new(temp, rain);
        0.5 - ((0.7 - Vec2::new(0.5, 0.0).distance(p)) * (1.0/0.7)).max(0.0) + ((0.7 - Vec2::new(0.5, 1.0).distance(p)) * (1.0/0.7)).max(0.0)
    }
}

pub struct Generator {
    seed: u32,
    height_noise: Fbm<Perlin>,
    temp_noise: Fbm<Perlin>,
    rain_noise: Fbm<Perlin>
}
impl GeneratorTrait for Generator {
    fn new(seed: u32) -> Self {
        let mut height_noise = Fbm::<Perlin>::new(seed);
        height_noise.frequency *= 0.1;
        height_noise.octaves = 6;
        let mut temp_noise = Fbm::<Perlin>::new(seed+1);
        temp_noise.frequency *= 0.1;
        temp_noise.octaves = 1;
        let mut rain_noise = Fbm::<Perlin>::new(seed+2);
        rain_noise.frequency *= 0.1;
        rain_noise.octaves = 1;
        Self {
            seed,
            height_noise,
            temp_noise,
            rain_noise
        }
    }
    fn generate(&self, pos: ChunkPos) -> [BlocType; CHUNK_X*CHUNK_Y*CHUNK_Z] {
        let height_noise_map = PlaneMapBuilder::<_, 2>::new(&self.height_noise)
            .set_size(CHUNK_X, CHUNK_Z)
            .set_x_bounds((pos.x as f64) - 0.5, (pos.x as f64) + 0.5)
            .set_y_bounds((pos.z as f64) - 0.5, (pos.z as f64) + 0.5)
            .build();
        let temp_noise_map = PlaneMapBuilder::<_, 2>::new(&self.temp_noise)
            .set_size(CHUNK_X, CHUNK_Z)
            .set_x_bounds((pos.x as f64) - 0.5, (pos.x as f64) + 0.5)
            .set_y_bounds((pos.z as f64) - 0.5, (pos.z as f64) + 0.5)
            .build();
        let rain_noise_map = PlaneMapBuilder::<_, 2>::new(&self.rain_noise)
            .set_size(CHUNK_X, CHUNK_Z)
            .set_x_bounds((pos.x as f64) - 0.5, (pos.x as f64) + 0.5)
            .set_y_bounds((pos.z as f64) - 0.5, (pos.z as f64) + 0.5)
            .build();

        let mut types = [BlocType::Air; CHUNK_X*CHUNK_Y*CHUNK_Z];
        for x in 0..CHUNK_X as u8 {
            for z in 0..CHUNK_Z as u8 {
                let temp = ((temp_noise_map.get_value(x as usize, z as usize) + 1.0)/2.0) as f32;
                let rain = ((rain_noise_map.get_value(x as usize, z as usize) + 1.0)/2.0) as f32;
                let biome = Biome::new(temp, rain);
                let mut h = Biome::avg_height(temp, rain);
                h += ((height_noise_map.get_value(x as usize, z as usize) + 1.0)/2.0) as f32 * (CHUNK_Y as f32) * Biome::height_variance(temp, rain);
                let h = (h as u8).min(CHUNK_Y as u8);
                for y in 0..h.saturating_sub(2) {
                    types[PosInChunk { x,y,z }.to_chunk_index()] = BlocType::Stone;
                }
                types[PosInChunk { x, y:h.saturating_sub(2) ,z }.to_chunk_index()] = BlocType::Dirt;
                types[PosInChunk { x, y:h.saturating_sub(1) ,z }.to_chunk_index()] = biome.top_block();
            }
        }
        types
    }
}