mod tests;
use std::fmt::Debug;

use macroquad::experimental::animation::AnimatedSprite;
use macroquad::{
    color::{Color, WHITE},
    math::Rect,
    shapes::draw_rectangle_lines,
    texture::Texture2D,
};

#[derive(Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Shape {
    pub w: f32,
    pub h: f32,
}

#[derive(Debug)]
pub struct Render {
    pub color: Color,
}

pub struct Sprite {
    pub texture: Texture2D,
    pub sprite: AnimatedSprite,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Player,
}

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

pub struct Physics {
    pub is_grounded: bool,
    pub velocity: Velocity,
}

pub struct Entity {
    pub transform: Rect,

    pub tag: Option<Tag>,

    pub render: Option<Render>,
    pub sprite: Option<Sprite>,

    pub physics: Option<Physics>,
}

// entities and components
pub struct World {
    pub entities: Vec<Entity>,
}

pub struct GameState {
    pub score: f32,
    pub game_over: bool,
}

pub struct Input {
    pub dt: f32,
    pub spacebar: bool,
}

pub struct Systems {
    pub update: Vec<fn(&mut World, &mut GameState, &Input)>,
    pub render: Vec<fn(&World)>,
}

pub struct Game {
    pub world: World,
    pub state: GameState,
    pub systems: Systems,
}

impl World {
    pub fn new() -> Self {
        Self { entities: vec![] }
    }

    pub fn with_tag(&self, tag: Tag) -> impl Iterator<Item = &Entity> {
        self.entities
            .iter()
            .filter(move |entity| entity.tag.is_some_and(|e| e == tag))
    }

    pub fn with_tag_mut(&mut self, tag: Tag) -> impl Iterator<Item = &mut Entity> {
        self.entities
            .iter_mut()
            .filter(move |entity| entity.tag.is_some_and(|e| e == tag))
    }

    pub fn spawn(mut self, entity: Entity) -> Self {
        self.entities.push(entity);
        self
    }
}

impl Physics {
    pub fn new() -> Physics {
        Self {
            is_grounded: true,
            velocity: Velocity { x: 0.0, y: 0.0 },
        }
    }
}

pub fn debug(entities: &[Entity]) {
    for e in entities {
        draw_rectangle_lines(
            e.transform.x,
            e.transform.y,
            e.transform.w,
            e.transform.h,
            2.0,
            WHITE,
        );
    }
}

pub fn debug_sprites(entities: &[Entity]) {
    for e in entities {
        let Some(sprite) = &e.sprite else {
            continue;
        };
        let frame = sprite.sprite.frame();
        draw_rectangle_lines(
            frame.source_rect.x,
            frame.source_rect.y,
            48.0 * 2.0,
            48.0 * 2.0,
            2.0,
            WHITE,
        );
    }
}

impl Entity {
    pub fn new(rect: Rect) -> Self {
        Self {
            transform: rect,
            tag: None,
            render: None,
            sprite: None,
            physics: None,
        }
    }

    pub fn with_render(mut self, color: Color) -> Self {
        self.render = Some(Render { color });
        self
    }

    pub fn with_sprite(mut self, texture: Texture2D, sprite: AnimatedSprite) -> Self {
        self.sprite = Some(Sprite { texture, sprite });
        self
    }

    pub fn with_tag(mut self, tag: Tag) -> Entity {
        self.tag = Some(tag);
        self
    }

    pub fn with_physics(mut self, physics: Physics) -> Entity {
        self.physics = Some(physics);
        self
    }
}

impl Systems {
    pub fn new() -> Self {
        Self {
            update: vec![],
            render: vec![],
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            score: 0.0,
            game_over: false,
        }
    }
}

impl Game {
    pub fn update(&mut self, input: &Input) {
        for system in &self.systems.update {
            system(&mut self.world, &mut self.state, input);
        }
    }

    pub fn render(&self) {
        for system in &self.systems.render {
            system(&self.world);
        }
    }
}

impl Game {
    pub fn with_update_system(mut self, system: fn(&mut World, &mut GameState, &Input)) -> Self {
        self.systems.update.push(system);
        self
    }

    pub fn with_render_system(mut self, system: fn(&World)) -> Self {
        self.systems.render.push(system);
        self
    }

    pub fn with_update_systems(
        mut self,
        systems: Vec<fn(&mut World, &mut GameState, &Input)>,
    ) -> Self {
        self.systems.update.extend(systems);
        self
    }

    pub fn with_render_systems(mut self, systems: Vec<fn(&World)>) -> Self {
        self.systems.render.extend(systems);
        self
    }
}

impl Game {
    pub fn new(world: World) -> Self {
        Self {
            world,
            state: GameState::new(),
            systems: Systems::new(),
        }
    }
}
