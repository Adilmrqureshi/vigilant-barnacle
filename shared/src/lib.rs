mod components;
mod entity;
mod systems;
mod tests;
mod ui;
mod utils;

pub use crate::components::*;
pub use crate::entity::*;
pub use crate::systems::*;
pub use crate::ui::*;
pub use crate::utils::*;

use macroquad::camera::Camera2D;
use macroquad::camera::set_camera;
use macroquad::math::Rect;
use macroquad::math::vec2;
use macroquad::window::screen_height;
use macroquad::window::screen_width;

pub struct World {
    pub entities: Vec<Entity>,
    pub origin: Transform,
}

impl World {
    pub fn new() -> Self {
        Self {
            origin: Transform { x: 0.0, y: 0.0 },
            entities: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.entities.iter_mut().for_each(|e| e.reset());
    }

    // Add pixels_per_unit: f32
    pub fn set_origin(&mut self, origin_x: f32, origin_y: f32) {
        let mut camera = Camera2D::default();

        camera.target = vec2(origin_x, origin_y);
        // Add (screen_width() / pixels_per_unit)
        camera.zoom = vec2(2.0 / screen_width(), -2.0 / screen_height());
        self.origin.x = origin_x;
        self.origin.y = origin_y;
        set_camera(&camera);
    }

    pub fn set_default_origin(&mut self) {
        self.set_origin(self.origin.x, self.origin.y);
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

    pub fn render(&mut self) {
        render_system(&mut self.entities);
    }

    pub fn update(&mut self, input: &components::Input) {
        collide_system(&mut self.entities);
        jump_system(&mut self.entities, input);
        movement_system(&mut self.entities, input);
    }
}
