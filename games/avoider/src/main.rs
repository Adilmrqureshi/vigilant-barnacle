mod background;
mod tests;

use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use shared_v2::*;

const ORIGINAL_SPRITE_SIZE: f32 = 48.0;
const GAME_SPRITE_SIZE: f32 = 48.0 * 2.0;
const GROUND: f32 = 40.0;

pub const VIRTUAL_WIDTH: f32 = 800.0;
pub const VIRTUAL_HEIGHT: f32 = 600.0;

fn normalise_camera(screen_w: f32, screen_h: f32) {
    let scale = (screen_w / VIRTUAL_WIDTH).min(0.1).floor();

    let viewport_w = VIRTUAL_WIDTH * scale;
    let viewport_h = VIRTUAL_HEIGHT * scale;

    let offset_x = (screen_w - viewport_w) * 0.5;
    let offset_y = (screen_h - viewport_h) * 0.5;

    let mut camera = Camera2D::default();

    camera.zoom = vec2(2.0 / viewport_w, -2.0 / viewport_h);
    camera.target = vec2(viewport_w * 0.5, viewport_h * 0.5);
    camera.offset = vec2(offset_x / scale, offset_y / scale);

    set_camera(&camera);
}

fn gravity_engine(world: &mut World, _state: &mut GameState, input: &Input) {
    for e in &mut world.entities {
        let Some(ref mut physics) = e.physics else {
            continue;
        };
        const GRAVITY: f32 = 800.0;
        const JUMP_STRENGTH: f32 = 450.0;

        if input.spacebar && physics.is_grounded {
            physics.velocity.y += JUMP_STRENGTH;
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
    for entity in &world.entities {
        let Some(ref player) = entity.sprite else {
            continue;
        };
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
                dest_size: Some(frame.dest_size * 3.0),
                source: Some(frame.source_rect),
                flip_x: false,
                flip_y: true,
                ..Default::default()
            },
        );
    }
}

fn update_sprites(world: &mut World) {
    for entity in &mut world.entities {
        let Some(ref mut player) = entity.sprite else {
            continue;
        };
        let Some(ref mut physics) = entity.physics else {
            player.sprite.update();
            continue;
        };
        if physics.is_grounded {
            player.sprite.update();
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
    for e in world.with_tag_mut(Tag::Enemy) {
        e.transform.x -= 300.0 * input.dt;
        if e.transform.x < -GAME_SPRITE_SIZE {
            e.transform.x += GAME_SPRITE_SIZE + VIRTUAL_WIDTH * rand::gen_range(1.0, 3.0);
        }
    }
}

fn load_player_sprite() -> AnimatedSprite {
    AnimatedSprite::new(
        ORIGINAL_SPRITE_SIZE as u32,
        ORIGINAL_SPRITE_SIZE as u32,
        &[Animation {
            name: "step_3".to_string(),
            row: 0,
            frames: 6,
            fps: 12,
        }],
        true,
    )
}

fn load_enemy_sprite() -> AnimatedSprite {
    AnimatedSprite::new(
        ORIGINAL_SPRITE_SIZE as u32,
        ORIGINAL_SPRITE_SIZE as u32,
        &[Animation {
            name: "imma_snake".to_string(),
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
    let sprite = load_player_sprite();
    let enemy_texture = load_player("snake_walk.png").await;
    let enemy_sprite = load_enemy_sprite();
    build_textures_atlas();
    let screen_w = screen_width();
    let screen_h = screen_height();

    let entity = Entity::new(Rect {
        x: GAME_SPRITE_SIZE,
        y: GROUND,
        w: GAME_SPRITE_SIZE,
        h: GAME_SPRITE_SIZE,
    })
    .with_tag(Tag::Player)
    .with_render(BLACK)
    .with_physics(Physics::new())
    .with_sprite(texture, sprite);

    let enemy = Entity::new(Rect {
        x: VIRTUAL_WIDTH * 2.0,
        y: GROUND,
        w: ORIGINAL_SPRITE_SIZE / 2.0,
        h: ORIGINAL_SPRITE_SIZE / 2.0,
    })
    .with_sprite(enemy_texture, enemy_sprite)
    .with_tag(Tag::Enemy);

    let world = World::new().spawn(entity).spawn(enemy);
    let mut game = Game::new(world)
        .with_update_systems(vec![gravity_engine, move_enemy_system, collision_system])
        .with_render_systems(vec![render_sprites, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
        };
        normalise_camera(screen_w, screen_h);
        background::render_paralax_background(&mut para, game.state.game_over);
        if !game.state.game_over {
            game.update(&input);
            update_sprites(&mut game.world);
        }
        game.render();
        next_frame().await;
    }
}
