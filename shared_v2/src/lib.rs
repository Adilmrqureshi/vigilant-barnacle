mod tests;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use macroquad::{
    color::{Color, WHITE},
    math::Rect,
    prelude::animation::AnimatedSprite,
    shapes::draw_rectangle_lines,
    texture::Texture2D,
};

#[derive(Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct Sprite {
    pub texture: Texture2D,
    pub sprite: AnimatedSprite,
    pub tag: AnimationTag,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Player,
    Enemy,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AnimationTag {
    Movement,
    Attack,
}

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

pub struct Physics {
    pub is_grounded: bool,
    pub velocity: Velocity,
}

pub struct Attack {
    pub animation: usize,
    pub is_attacking: bool,
}

pub struct Entity {
    pub transform: Rect,

    pub tag: Option<Tag>,

    pub render: Option<Render>,
    pub current_sprite: Option<usize>,
    pub original_sprite: Option<usize>,

    pub physics: Option<Physics>,
    pub attack: Option<Attack>,
}

pub struct World {
    pub entities: Vec<Entity>,
    pub sprites: Vec<Sprite>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

pub struct GameState {
    pub score: f32,
    pub game_over: bool,
}

pub struct Input {
    pub dt: f32,
    pub spacebar: bool,
    pub a: bool,
}

pub struct Systems {
    pub update: Vec<fn(&mut World, &mut GameState, &Input)>,
    pub render: Vec<fn(&World, &GameState)>,
}

pub struct Game {
    pub world: World,
    pub state: GameState,
    pub systems: Systems,
}

// Resources
pub struct EnemyManager {
    pub active_enemy: Option<usize>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            sprites: vec![],
            resources: HashMap::new(),
        }
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

    pub fn add_sprite(mut self, sprite: Sprite) -> Self {
        self.sprites.push(sprite);
        self
    }

    pub fn spawn(mut self, entity: Entity) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
    }

    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|r| r.downcast_mut::<T>())
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

// pub fn debug_sprites(entities: &[Entity]) {
//     for e in entities {
//         let Some(sprite) = &e.current_sprite else {
//             continue;
//         };
//         let frame = sprite.sprite.frame();
//         draw_rectangle_lines(
//             frame.source_rect.x,
//             frame.source_rect.y,
//             48.0 * 2.0,
//             48.0 * 2.0,
//             2.0,
//             WHITE,
//         );
//     }
// }

impl Entity {
    pub fn new(rect: Rect) -> Self {
        Self {
            transform: rect,
            tag: None,
            render: None,
            current_sprite: None,
            original_sprite: None,
            physics: None,
            attack: None,
        }
    }

    pub fn with_attack(mut self, animation: usize) -> Self {
        self.attack = Some(Attack {
            is_attacking: false,
            animation,
        });
        self
    }

    pub fn with_render(mut self, color: Color) -> Self {
        self.render = Some(Render { color });
        self
    }

    pub fn with_sprite(mut self, sprite: usize) -> Self {
        self.original_sprite = Some(sprite);
        self.current_sprite = Some(sprite);
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
            system(&self.world, &self.state);
        }
    }
}

impl Game {
    pub fn with_update_system(mut self, system: fn(&mut World, &mut GameState, &Input)) -> Self {
        self.systems.update.push(system);
        self
    }

    pub fn with_render_system(mut self, system: fn(&World, &GameState)) -> Self {
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

    pub fn with_render_systems(mut self, systems: Vec<fn(&World, &GameState)>) -> Self {
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
