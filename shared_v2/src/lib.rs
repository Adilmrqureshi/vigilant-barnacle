mod tests;
pub mod ui;
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
    Body,
    Food,
    Ball,
    Brick,
    Bullet,
    /// Experience pickup dropped by defeated enemies.
    Xp,
}

/// Hit points. `hp` may go negative for a frame; systems that care should
/// treat `hp <= 0.0` as dead and clear `alive`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub hp: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { hp: max, max }
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.hp / self.max).clamp(0.0, 1.0)
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}

/// Countdown that fires every `period` seconds. `tick` returns true on the
/// frame it fires and re-arms itself, so callers never repeat the
/// subtract-compare-reset dance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timer {
    pub remaining: f32,
    pub period: f32,
}

impl Timer {
    pub fn new(period: f32) -> Self {
        Self {
            remaining: period,
            period,
        }
    }

    /// Like `new`, but fires on the very first tick.
    pub fn ready(period: f32) -> Self {
        Self {
            remaining: 0.0,
            period,
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.remaining -= dt;
        if self.remaining <= 0.0 {
            self.remaining += self.period;
            if self.remaining <= 0.0 {
                self.remaining = self.period;
            }
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.remaining = self.period;
    }

    /// 0.0 just after firing, 1.0 just before the next fire.
    pub fn fraction(&self) -> f32 {
        if self.period <= 0.0 {
            1.0
        } else {
            (1.0 - self.remaining / self.period).clamp(0.0, 1.0)
        }
    }
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
    // Stable identity, assigned by World::add. 0 means "never registered";
    // entities pushed straight onto the Vec keep it and can't be found by id.
    pub id: usize,
    pub transform: Rect,
    pub alive: bool,

    pub tag: Option<Tag>,
    /// Game-defined sub-type within a tag (enemy variety, projectile owner,
    /// pickup value...). The engine never interprets it.
    pub kind: Option<usize>,

    pub render: Option<Render>,
    pub current_sprite: Option<usize>,
    pub original_sprite: Option<usize>,

    pub physics: Option<Physics>,
    pub attack: Option<Attack>,
    pub health: Option<Health>,
}

pub struct World {
    pub entities: Vec<Entity>,
    pub sprites: Vec<Sprite>,
    resources: HashMap<TypeId, Box<dyn Any>>,
    next_id: usize,
    event_clearers: Vec<fn(&mut World)>,
}

// One-frame message queue. Systems emit during an update; any number of later
// systems can read the same events; Game::update clears every queue at the
// start of the next frame.
pub struct Events<E> {
    pub queue: Vec<E>,
}

pub struct GameState {
    pub score: f32,
    pub game_over: bool,
}

pub struct Input {
    pub dt: f32,
    pub spacebar: bool,
    pub a: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    // Mouse position in the same pixel space as screen_width/screen_height.
    pub mouse_x: f32,
    pub mouse_y: f32,
    /// Left button went down this frame.
    pub mouse_pressed: bool,
    /// Left button currently held.
    pub mouse_down: bool,
    /// Right button went down this frame.
    pub mouse_right_pressed: bool,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            dt: 0.0,
            spacebar: false,
            a: false,
            up: false,
            down: false,
            left: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_pressed: false,
            mouse_down: false,
            mouse_right_pressed: false,
            right: false,
            screen_width: 0.0,
            screen_height: 0.0,
        }
    }
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
    // The entity id (see World::add) of the enemy currently in play.
    pub active_enemy: Option<usize>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            sprites: vec![],
            resources: HashMap::new(),
            next_id: 1,
            event_clearers: vec![],
        }
    }

    // Registers an entity with a stable id and returns it. Unlike a raw
    // `entities.push`, entities added here can be looked up with find/find_mut
    // even after despawn_dead() reshuffles indices.
    pub fn add(&mut self, mut entity: Entity) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        entity.id = id;
        self.entities.push(entity);
        id
    }

    pub fn find(&self, id: usize) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    pub fn find_mut(&mut self, id: usize) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    pub fn emit<E: 'static>(&mut self, event: E) {
        if self.get_resource::<Events<E>>().is_none() {
            self.insert_resource(Events::<E> { queue: vec![] });
            self.event_clearers.push(|world: &mut World| {
                if let Some(events) = world.get_resource_mut::<Events<E>>() {
                    events.queue.clear();
                }
            });
        }
        self.get_resource_mut::<Events<E>>()
            .unwrap()
            .queue
            .push(event);
    }

    pub fn events<E: 'static>(&self) -> &[E] {
        self.get_resource::<Events<E>>()
            .map(|e| e.queue.as_slice())
            .unwrap_or(&[])
    }

    pub fn clear_events(&mut self) {
        let clearers = self.event_clearers.clone();
        for clear in clearers {
            clear(self);
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
        self.add(entity);
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

    // Removes every entity whose `alive` flag has been cleared. Call once per
    // frame after update systems; indices held across this call are invalid.
    pub fn despawn_dead(&mut self) {
        self.entities.retain(|e| e.alive);
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
            id: 0,
            transform: rect,
            alive: true,
            tag: None,
            kind: None,
            render: None,
            current_sprite: None,
            original_sprite: None,
            physics: None,
            attack: None,
            health: None,
        }
    }

    pub fn with_kind(mut self, kind: usize) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_health(mut self, max: f32) -> Self {
        self.health = Some(Health::new(max));
        self
    }

    /// Physics component with the given starting velocity.
    pub fn with_velocity(mut self, x: f32, y: f32) -> Self {
        self.physics = Some(Physics {
            is_grounded: false,
            velocity: Velocity { x, y },
        });
        self
    }

    pub fn center(&self) -> (f32, f32) {
        (
            self.transform.x + self.transform.w / 2.0,
            self.transform.y + self.transform.h / 2.0,
        )
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
        // Events emitted last frame have been seen by every system by now.
        self.world.clear_events();
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
