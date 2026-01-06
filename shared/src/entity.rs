use macroquad::color::Color;

use crate::*;

#[derive(Debug)]
pub struct Entity {
    pub id: i32,
    pub transform: Rect,
    pub default_position: Transform,

    pub render: Option<components::Render>,
    pub jump: Option<components::Jump>,
    pub collide: Option<components::Collide>,
    pub movement: Option<components::Movement>,
}

impl Entity {
    pub fn new(id: i32, x: f32, y: f32) -> Self {
        Self {
            id,
            jump: None,
            collide: None,
            render: None,
            movement: None,
            default_position: Transform { x, y },
            transform: Rect {
                x,
                y,
                w: 64.0,
                h: 64.0,
            },
        }
    }

    pub fn is_collided(&self) -> bool {
        let Some(ref c) = self.collide else {
            return false;
        };
        println!("Hi {}", c.is_collided);
        return c.is_collided;
    }

    pub fn set_dimensions(&mut self, w: f32, h: f32) {
        self.transform.w = w;
        self.transform.h = h;
    }

    pub fn reset(&mut self) {
        let Some(ref mut c) = self.collide else {
            return;
        };
        c.is_collided = false;
        self.transform.x = self.default_position.x;
        self.transform.y = self.default_position.y;
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.transform.x = x;
        self.transform.y = y;
    }

    pub fn with_jump(mut self, force: f32, ground_level: f32) -> Self {
        self.jump = Some(components::Jump {
            force,
            // change these
            gravity: 600.0,
            ground_level,
            velocity: components::Velocity { x: 0.0, y: 0.0 },
        });
        self
    }

    pub fn with_collide(mut self) -> Self {
        self.collide = Some(components::Collide { is_collided: false });
        self
    }

    pub fn with_move(mut self, x: f32, y: f32) -> Self {
        self.movement = Some(components::Movement {
            velocity: components::Velocity { x, y },
        });
        self
    }

    pub fn with_render(mut self, w: f32, h: f32, color: Color) -> Self {
        self.render = Some(Render {
            shape: Shape { w, h },
            color,
        });
        self
    }
}
