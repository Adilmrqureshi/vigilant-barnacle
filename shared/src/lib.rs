mod components;
mod entity;
mod systems;
mod tests;

pub use crate::components::*;
pub use crate::entity::*;
pub use crate::systems::*;

use macroquad::math::Rect;

pub struct World {
    pub entities: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn find(&self, id: i32) -> Option<&Entity> {
        self.entities.iter().find(|ent| id == ent.id)
    }

    pub fn find_mut(&mut self, id: i32) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|ent| id == ent.id)
    }

    pub fn spawn(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn update(&mut self, input: &components::Input) {
        jump_system(&mut self.entities, input);
        collide_system(&mut self.entities);
        movement_system(&mut self.entities, input);
        render_system(&mut self.entities);
    }
}
