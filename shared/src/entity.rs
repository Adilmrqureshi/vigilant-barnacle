use crate::*;

#[derive(Debug)]
pub struct Entity {
    pub transform: Rect,

    pub jump: Option<components::Jump>,
    pub collide: Option<components::Collide>,
    pub movement: Option<components::Movement>,
}

impl Entity {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            jump: None,
            collide: None,
            movement: None,
            transform: Rect {
                x,
                y,
                w: 64.0,
                h: 64.0,
            },
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.transform.x = x;
        self.transform.y = y;
    }

    pub fn with_jump(mut self, force: f32, ground_level: f32) -> Self {
        self.jump = Some(components::Jump {
            force,
            // change these
            gravity: 450.0,
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
}
