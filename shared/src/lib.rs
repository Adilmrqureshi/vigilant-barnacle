use macroquad::math::Rect;

#[derive(Debug)]
pub struct Input {
    pub dt: f32,
    pub is_jump: bool,
}

#[derive(Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Jump {
    pub force: f32,
    pub ground_level: f32,
    pub gravity: f32,
    pub velocity: Velocity,
}

#[derive(Debug)]
pub struct Collide {
    pub is_collided: bool,
}

#[derive(Debug)]
pub struct Movement {
    velocity: Velocity,
}

pub fn movement_system(entities: &mut [Entity], input: &Input) {
    for entity in entities {
        let Some(ref movement) = entity.movement else {
            continue;
        };
        let should_not_move = matches!(entity.collide, Some(ref c) if c.is_collided);

        if !should_not_move {
            entity.transform.x += movement.velocity.x * input.dt;
            entity.transform.y += movement.velocity.y * input.dt;
        };
    }
}

pub fn collide_system(entities: &mut [Entity]) {
    let len = entities.len();

    for i in 0..len {
        for j in i + 1..len {
            let (a, b) = {
                let (left, right) = entities.split_at_mut(j);
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
                col_1.is_collided = true;
                col_2.is_collided = true;
            }
        }
    }
}

pub fn jump_system(entities: &mut [Entity], input: &Input) {
    for entity in entities {
        let Some(jump) = &mut entity.jump else {
            continue;
        };

        if entity.transform.y >= jump.ground_level {
            if input.is_jump {
                jump.velocity.y = jump.force;
            } else {
                jump.velocity.y = 0.0;
            }
        } else if entity.transform.y < jump.ground_level {
            jump.velocity.y -= jump.gravity * input.dt;
        }
        entity.transform.x += jump.velocity.x * input.dt;
        entity.transform.y -= jump.velocity.y * input.dt;
    }
}

#[derive(Debug)]
pub struct Entity {
    pub transform: Rect,

    pub jump: Option<Jump>,
    pub collide: Option<Collide>,
    pub movement: Option<Movement>,
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
        self.jump = Some(Jump {
            force,
            // change these
            gravity: 450.0,
            ground_level,
            velocity: Velocity { x: 0.0, y: 0.0 },
        });
        self
    }

    pub fn with_collide(mut self) -> Self {
        self.collide = Some(Collide { is_collided: false });
        self
    }

    pub fn with_move(mut self, x: f32, y: f32) -> Self {
        self.movement = Some(Movement {
            velocity: Velocity { x, y },
        });
        self
    }
}

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

    pub fn update(&mut self, input: &Input) {
        jump_system(&mut self.entities, input);
        collide_system(&mut self.entities);
        movement_system(&mut self.entities, input);
    }
}

#[cfg(test)]
mod test_utils {
    use super::*;

    pub fn input(dt: f32, jump: bool) -> Input {
        Input { dt, is_jump: jump }
    }

    pub fn entity_at(x: f32, y: f32) -> Entity {
        Entity::new(x, y)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn movement_moves_entity_when_not_colliding() {
        let mut entities = vec![Entity::new(0.0, 0.0).with_move(10.0, 0.0)];

        let input = Input {
            dt: 1.0,
            is_jump: false,
        };

        movement_system(&mut entities, &input);

        assert_eq!(entities[0].transform.x, 10.0);
        assert_eq!(entities[0].transform.y, 0.0);
    }

    #[test]
    fn movement_stops_when_collided() {
        let mut entity = Entity::new(0.0, 0.0).with_move(10.0, 0.0).with_collide();

        entity.collide.as_mut().unwrap().is_collided = true;

        let mut entities = vec![entity];
        let input = Input {
            dt: 1.0,
            is_jump: false,
        };

        movement_system(&mut entities, &input);

        assert_eq!(entities[0].transform.x, 0.0);
    }

    #[test]
    fn jump_applies_force_when_grounded() {
        let mut entities = vec![Entity::new(0.0, 100.0).with_jump(300.0, 100.0)];

        let input = Input {
            dt: 1.0,
            is_jump: true,
        };

        jump_system(&mut entities, &input);

        // y decreases because you subtract velocity.y
        assert!(entities[0].transform.y < 100.0);
    }

    #[test]
    fn gravity_applies_when_airborne() {
        let mut entity = Entity::new(0.0, 90.0).with_jump(300.0, 100.0);

        // Simulate upward velocity
        entity.jump.as_mut().unwrap().velocity.y = 100.0;

        let mut entities = vec![entity];
        let input = Input {
            dt: 1.0,
            is_jump: false,
        };

        jump_system(&mut entities, &input);

        let jump = entities[0].jump.as_ref().unwrap();
        assert!(jump.velocity.y < 100.0);
    }

    #[test]
    fn collision_detects_overlap() {
        let mut entities = vec![
            Entity::new(0.0, 0.0).with_collide(),
            Entity::new(32.0, 32.0).with_collide(), // overlaps 64x64
        ];

        collide_system(&mut entities);

        assert!(entities[0].collide.as_ref().unwrap().is_collided);
        assert!(entities[1].collide.as_ref().unwrap().is_collided);
    }

    #[test]
    fn collision_not_detected_when_separated() {
        let mut entities = vec![
            Entity::new(0.0, 0.0).with_collide(),
            Entity::new(200.0, 200.0).with_collide(),
        ];

        collide_system(&mut entities);

        assert!(!entities[0].collide.as_ref().unwrap().is_collided);
        assert!(!entities[1].collide.as_ref().unwrap().is_collided);
    }

    #[test]
    fn world_update_runs_all_systems() {
        let mut world = World::new();

        world.spawn(
            Entity::new(0.0, 100.0)
                .with_jump(200.0, 100.0)
                .with_move(10.0, 0.0)
                .with_collide(),
        );

        let input = Input {
            dt: 1.0,
            is_jump: true,
        };

        world.update(&input);

        let e = &world.entities[0];
        assert!(e.transform.x > 0.0);
        assert!(e.transform.y < 100.0);
    }
}
