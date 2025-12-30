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
    pub velocity: Velocity,
}
