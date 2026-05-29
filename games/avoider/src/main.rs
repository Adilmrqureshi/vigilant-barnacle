mod background;
mod tests;

use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use shared_v2::*;

const ORIGINAL_SPRITE_SIZE: f32 = 48.0;
const GAME_SPRITE_SIZE: f32 = 48.0 * 2.0;
const GROUND: f32 = 40.0;
const DEFAULT_PLAYER_POSITION: f32 = 96.0;
const SPEED_SCREEN_RATO: f32 = 0.4;
const CHARACTER_SCALE: f32 = 4.0;
const FLYING_HEIGHT: f32 = 250.0; // const GRAVITY: f32 = 800.0;
const DESIRED_JUMP_HEIGHT: f32 = 250.0;
const GRAVITY: f32 = 800.0;

fn normalise_camera() {
    let mut camera = Camera2D::default();

    camera.zoom = vec2(2.0 / screen_width(), -2.0 / screen_height());
    camera.target = vec2(screen_width() * 0.5, screen_height() * 0.5);
    camera.offset = vec2(0.0, 0.0);

    set_camera(&camera);
}

fn gravity_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for e in &mut world.entities {
        let Some(ref mut physics) = e.physics else {
            continue;
        };

        let jump_strength: f32 = (2.0 * GRAVITY * DESIRED_JUMP_HEIGHT).sqrt();

        if input.spacebar && physics.is_grounded {
            physics.velocity.y += jump_strength;
            physics.is_grounded = false;
        }
        if !physics.is_grounded {
            physics.velocity.y -= GRAVITY * input.dt;
            if e.transform.y.ceil() < GROUND {
                e.transform.y = GROUND;
                physics.is_grounded = true;
                physics.velocity.y = 0.0;
            }
        }
        e.transform.y += physics.velocity.y * input.dt;
    }
}

fn render_sprites(world: &World, _state: &GameState) {
    let sprites = &world.sprites;
    for entity in &world.entities {
        let Some(player_index) = entity.current_sprite else {
            continue;
        };
        let player = &sprites[player_index];
        let frame = player.sprite.frame();
        let direction = if entity.tag.is_some() && entity.tag.unwrap() == Tag::Enemy {
            -1.0
        } else {
            1.0
        };
        draw_texture_ex(
            &player.texture,
            entity.transform.x + ORIGINAL_SPRITE_SIZE * direction,
            entity.transform.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(frame.dest_size * CHARACTER_SCALE),
                source: Some(frame.source_rect),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}

fn update_sprites(world: &mut World) {
    let sprites = &mut world.sprites;
    for entity in &mut world.entities {
        let Some(sprite_index) = entity.current_sprite else {
            continue;
        };
        let player_sprite = &mut sprites[sprite_index];
        let Some(ref mut physics) = entity.physics else {
            player_sprite.sprite.update();
            continue;
        };

        match player_sprite.tag {
            AnimationTag::Movement => {
                if physics.is_grounded {
                    player_sprite.sprite.update();
                }
            }
            AnimationTag::Attack => {
                let Some(ref mut attack) = entity.attack else {
                    continue;
                };
                player_sprite.sprite.update();
                if player_sprite.sprite.is_last_frame() {
                    entity.current_sprite = entity.original_sprite;
                    attack.is_attacking = false;
                }
            }
        }
    }
}

fn collision_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let len = world.entities.len();
    for i in 0..len {
        for j in i + 1..len {
            let (a, b) = {
                let (left, right) = world.entities.split_at_mut(j);
                (&mut left[i], &mut right[0])
            };

            if a.tag.is_some()
                && a.tag.unwrap() == Tag::Player
                && a.transform.overlaps(&b.transform)
            {
                state.game_over = true;
            }
        }
    }
}

fn ui_system(_world: &World, state: &GameState) {
    set_default_camera();

    if state.game_over {
        let text = "GAME OVER!";
        let text_dimensions = measure_text(text, None, 50, 1.0);
        let pos = Transform {
            x: screen_width() / 2.0 - text_dimensions.width / 2.0,
            y: screen_height() / 2.0 - text_dimensions.height / 2.0,
        };
        draw_text(text, pos.x, pos.y, 60.0, RED);
    }
}

fn move_enemy_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let speed = SPEED_SCREEN_RATO * screen_width();
    for e in world.with_tag_mut(Tag::Enemy) {
        e.transform.x -= speed * input.dt;
    }
}

fn enemy_spawn_system(world: &mut World, _state: &mut GameState, _input: &Input) {
    let screen_w = screen_width();

    // Step 1: read active_enemy WITHOUT holding mutable borrow
    let active_enemy = {
        world
            .get_resource::<EnemyManager>()
            .and_then(|m| m.active_enemy)
    };

    // Step 2: check if it’s still visible
    let still_active = if let Some(idx) = active_enemy {
        let e = &world.entities[idx];
        e.transform.x > -GAME_SPRITE_SIZE 
    } else {
        false
    };

    if still_active {
        return;
    }

    // Step 3: gather candidates (no mutable borrow yet)
    let enemy_indices: Vec<_> = world
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.tag == Some(Tag::Enemy))
        .map(|(i, _)| i)
        .collect();

    if enemy_indices.is_empty() {
        return;
    }

    let choice = rand::gen_range(0, enemy_indices.len() as i32) as usize;
    let idx = enemy_indices[choice];

    // Step 4: mutate entity
    {
        let enemy = &mut world.entities[idx];
        enemy.transform.x = screen_w + rand::gen_range(0.0, screen_w);
    }

    // Step 5: NOW mutate the resource (separate borrow)
    {
        let manager = world.get_resource_mut::<EnemyManager>().unwrap();
        manager.active_enemy = Some(idx);
    }
}

fn enemy_take_damage(world: &mut World, _state: &mut GameState, input: &Input) {
    let len = world.entities.len();
    for i in 0..len {
        for j in i + 1..len {
            let (a, b) = {
                let (left, right) = world.entities.split_at_mut(j);
                (&mut left[i], &mut right[0])
            };

            let Some(attack) = &a.attack else {
                continue;
            };

            if a.tag.is_some() && a.tag.unwrap() == Tag::Player && attack.is_attacking {
                let dist = a.transform.right() - b.transform.left();
                if dist.powi(2) < 100.0 {
                    b.transform.x += GAME_SPRITE_SIZE + screen_width() * rand::gen_range(2.0, 5.0);
                }
            }
        }
    }
}

fn attack_system(world: &mut World, _state: &mut GameState, input: &Input) {
    if input.a {
        for player in world.with_tag_mut(Tag::Player) {
            let Some(ref mut attack) = player.attack else {
                continue;
            };
            player.current_sprite = Some(attack.animation.clone());
        }
    }
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        for player in game.world.with_tag_mut(Tag::Player) {
            player.transform.x = DEFAULT_PLAYER_POSITION;
            player.transform.y = GROUND;
        }

        for enemy in game.world.with_tag_mut(Tag::Enemy) {
            enemy.transform.x += screen_width();
        }
        game.state.game_over = false;
    }
}

fn load_sprite(name: String) -> AnimatedSprite {
    AnimatedSprite::new(
        ORIGINAL_SPRITE_SIZE as u32,
        ORIGINAL_SPRITE_SIZE as u32,
        &[Animation {
            name,
            row: 0,
            frames: 4,
            fps: 12,
        }],
        true,
    )
}

async fn load_player(file_name: &str) -> Texture2D {
    let texture: Texture2D = load_texture(file_name).await.expect("Couldn't load file");
    texture.set_filter(FilterMode::Linear);
    texture
}

#[macroquad::main("Death avoider")]
async fn main() {
    set_pc_assets_folder("./assets");
    let mut para = background::load_background_assets().await;
    let texture = load_player("boy_walk.png").await;
    let sprite = load_sprite("player".to_string());
    let snake_texture = load_player("snake_walk.png").await;
    let snake_sprite = load_sprite("imma_snake".to_string());
    let vulture_texture = load_player("vulture_walk.png").await;
    let vulture_sprite = load_sprite("vulture".to_string());
    let attack_texture = load_player("boy_attack.png").await;
    let attack_animation = load_sprite("attack!".to_string());
    build_textures_atlas();

    let entity = Entity::new(Rect {
        x: DEFAULT_PLAYER_POSITION,
        y: GROUND,
        w: GAME_SPRITE_SIZE,
        h: GAME_SPRITE_SIZE,
    })
    .with_tag(Tag::Player)
    .with_render(BLACK)
    .with_physics(Physics::new())
    .with_sprite(0)
    .with_attack(1);

    let snake = Entity::new(Rect {
        x: screen_width() * 2.0,
        y: GROUND,
        w: ORIGINAL_SPRITE_SIZE / 2.0,
        h: ORIGINAL_SPRITE_SIZE / 2.0,
    })
    .with_sprite(2)
    .with_tag(Tag::Enemy);

    let vulture = Entity::new(Rect {
        x: screen_width() * 4.0,
        y: FLYING_HEIGHT,
        w: ORIGINAL_SPRITE_SIZE / 2.0,
        h: ORIGINAL_SPRITE_SIZE / 2.0,
    })
    .with_sprite(3)
    .with_tag(Tag::Enemy);

    let mut world = World::new()
        .add_sprite(Sprite {
            texture,
            sprite,
            tag: AnimationTag::Movement,
        })
        .add_sprite(Sprite {
            texture: attack_texture,
            sprite: attack_animation,
            tag: AnimationTag::Attack,
        })
        .add_sprite(Sprite {
            texture: snake_texture,
            sprite: snake_sprite,
            tag: AnimationTag::Movement,
        })
        .add_sprite(Sprite {
            texture: vulture_texture,
            sprite: vulture_sprite,
            tag: AnimationTag::Movement,
        })
        .spawn(entity)
        .spawn(snake)
        .spawn(vulture);

    world.insert_resource(EnemyManager { active_enemy: None });

    let mut game = Game::new(world)
        .with_update_systems(vec![
            gravity_system,
            move_enemy_system,
            collision_system,
            attack_system,
            enemy_spawn_system,
        ])
        .with_render_systems(vec![render_sprites, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            a: is_key_pressed(KeyCode::A),
        };
        normalise_camera();
        background::render_paralax_background(&mut para, game.state.game_over);
        if !game.state.game_over {
            game.update(&input);
            update_sprites(&mut game.world);
        } else {
            restart_game(&mut game, &input);
        }
        game.render();
        next_frame().await;
    }
}
