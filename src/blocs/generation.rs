use bevy::prelude::*;
use rand::SeedableRng;
use super::{Generator as GeneratorTrait, BlocType, CHUNK_X, CHUNK_Y, CHUNK_Z, ChunkPos, PosInChunk};
use noise::{Fbm, Perlin};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use rand::rngs::StdRng;

pub struct Generator {
    seed: u32,
    height_noise: Fbm<Perlin>
}
impl GeneratorTrait for Generator {
    fn new(seed: u32) -> Self {
        Self {
            seed,
            height_noise: Fbm::<Perlin>::new(seed)
        }
    }
    fn generate(&self, pos: ChunkPos) -> [BlocType; CHUNK_X*CHUNK_Y*CHUNK_Z] {
        let height_noise_map = PlaneMapBuilder::<_, 2>::new(&self.height_noise)
          .set_size(CHUNK_X, CHUNK_Z)
          .set_x_bounds((pos.x as f64) - 0.5, (pos.x as f64) + 0.5)
          .set_y_bounds((pos.z as f64) - 0.5, (pos.z as f64) + 0.5)
          .build();
        let mut types = [BlocType::Air; CHUNK_X*CHUNK_Y*CHUNK_Z];
        for x in 0..CHUNK_X as u8 {
            for z in 0..CHUNK_Z as u8 {
                let h = ((((height_noise_map.get_value(x as usize, z as usize) + 1.0)/2.0) * (CHUNK_Y as f64)) as u8).min(CHUNK_Y as u8);
                for y in 0..h.saturating_sub(2) {
                    types[PosInChunk { x,y,z }.to_chunk_index()] = BlocType::Stone;
                }
                types[PosInChunk { x, y:h.saturating_sub(2) ,z }.to_chunk_index()] = BlocType::Dirt;
                types[PosInChunk { x, y:h.saturating_sub(1) ,z }.to_chunk_index()] = BlocType::Grass;
            }
        }
        types
    }
}