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

#[derive(Debug, PartialEq)]
pub enum GameState {
    Running,
    GameOver,
}

pub struct World {
    pub entities: Vec<Entity>,
    pub origin: Transform,
    pub state: GameState,
}

impl World {
    pub fn new() -> Self {
        Self {
            origin: Transform { x: 0.0, y: 0.0 },
            entities: Vec::new(),
            state: GameState::Running,
        }
    }

    pub fn reset(&mut self) {
        self.entities.iter_mut().for_each(|e| e.reset());
    }

    pub fn despawn(&mut self, id: i32) {
        let index = self.entities.iter().position(|x| x.id == id).unwrap();
        self.entities.remove(index);
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

    pub fn collide_system(&mut self) {
        let len = self.entities.len();

        for i in 0..len {
            for j in i + 1..len {
                let (a, b) = {
                    let (left, right) = self.entities.split_at_mut(j);
                    (&mut left[i], &mut right[0])
                };

                let Some(ref mut col_1) = a.collide else {
                    continue;
                };
                let Some(ref mut col_2) = b.collide else {
                    continue;
                };
                col_1.is_collided = false;
                col_2.is_collided = false;

                if a.transform.overlaps(&b.transform) {
                    println!("{} {}", a.id, b.id);
                    if a.id == 1 || b.id == 1 {
                        self.state = GameState::GameOver;
                    }
                    col_1.is_collided = true;
                    col_2.is_collided = true;
                }
            }
        }
    }

    pub fn update(&mut self, input: &components::Input) {
        collide_system(&mut self.entities);
        jump_system(&mut self.entities, input);
        movement_system(&mut self.entities, input);
    }

    pub fn new_update(&mut self, input: &components::Input) {
        self.collide_system();
        jump_system(&mut self.entities, input);
        movement_system(&mut self.entities, input);
    }
}
