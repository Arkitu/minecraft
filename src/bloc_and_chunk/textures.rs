use bevy::{prelude::*, utils::HashMap};
use image;
use crate::{BlocType, DestructionLevel, bloc_and_chunk::Direction};

#[derive(PartialEq, Eq, Hash)]
pub struct TextureId {
    r#type: BlocType,
    dir: Direction,
    lvl: DestructionLevel
}
impl From<(BlocType, Direction, DestructionLevel)> for TextureId {
    fn from(value: (BlocType, Direction, DestructionLevel)) -> Self {
        Self {
            r#type: value.0,
            dir: value.1,
            lvl: value.2
        }
    }
}

#[derive(Resource)]
pub struct TexturesManager (HashMap<TextureId, Handle<StandardMaterial>>);
impl Default for TexturesManager {
    fn default() -> Self {
        Self (HashMap::new())
    }
}

impl TexturesManager {
    pub fn get_id_raw(&self, id: &TextureId) -> Option<&Handle<StandardMaterial>> {
        self.0.get(id)
    }
    pub fn get_id(&mut self, id: &TextureId) -> &Handle<StandardMaterial> {
        if let Some(h) = self.get_id_raw(id) {
            return h
        }

        let img = image::open(format!("{}/{}.png", id.r#type.to_string(), id.dir.face_to_render_name())).unwrap()
    }
    pub fn get_raw(&self, r#type: BlocType, dir: Direction, lvl: DestructionLevel) -> Option<&Handle<StandardMaterial>> {
        self.get_id_raw(&(r#type, dir, lvl).into())
    }
}