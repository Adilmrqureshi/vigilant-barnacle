pub struct Input {
    dt: f32,
    is_jump: bool,
}

pub struct Transform {
    pub x: f32,
    pub y: f32,
}

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

pub struct Jump {
    pub force: f32,
    pub ground_level: f32,
    pub gravity: f32,
}

pub fn jump_system(entities: &mut [Entity], input: &Input) {
    for entity in entities {
        let Some(jump) = &mut entity.jump else {
            continue;
        };

        if input.is_jump && entity.transform.y <= jump.ground_level {
            entity.velocity.y = jump.force;
        } else {
            entity.velocity.y -= jump.gravity * input.dt;
        }
    }
}

pub struct Entity {
    pub transform: Transform,
    pub velocity: Velocity,

    pub jump: Option<Jump>,
}

impl Entity {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            jump: None,
            transform: Transform { x, y },
            velocity: Velocity { x: 0.0, y: 0.0 },
        }
    }

    pub fn with_jump(mut self, force: f32) -> Self {
        self.jump = Some(Jump {
            force,
            // change these
            gravity: 9.81,
            ground_level: 0.0,
        });
        self
    }
}

pub struct World {
    pub entites: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entites: Vec::new(),
        }
    }

    pub fn spawn(&mut self, entity: Entity) {
        self.entites.push(entity);
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
