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

    pub fn spawn(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn update(&mut self, input: &components::Input) {
        jump_system(&mut self.entities, input);
        collide_system(&mut self.entities);
        movement_system(&mut self.entities, input);
    }
}
