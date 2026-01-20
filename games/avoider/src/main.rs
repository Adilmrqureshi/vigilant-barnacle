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
    let scale = (screen_w / VIRTUAL_WIDTH).min(0.1).floor().max(1.0);

    let viewport_w = VIRTUAL_WIDTH * scale;
    let viewport_h = VIRTUAL_HEIGHT * scale;

    let offset_x = (screen_w - viewport_w) * 0.5;
    let offset_y = (screen_h - viewport_h) * 0.5;

    let mut camera = Camera2D::default();

    camera.zoom = vec2(2.0 / viewport_w, -2.0 / viewport_h);
    camera.target = vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5);
    camera.offset = vec2(offset_x / scale, offset_y / scale);

    set_camera(&camera);
}

fn gravity_engine(world: &mut World, state: &mut GameState, input: &Input) {
    for e in &mut world.entities {
        let Some(ref mut physics) = e.physics else {
            continue;
        };
        const GRAVITY: f32 = 800.0;
        const JUMP_STRENGTH: f32 = 450.0;

        if input.spacebar && physics.is_grounded {
            e.transform.y = GROUND;
            physics.velocity.y += JUMP_STRENGTH;
            physics.is_grounded = false;
        }
        if !physics.is_grounded {
            physics.velocity.y -= GRAVITY * input.dt;
            if e.transform.y.ceil() < GROUND {
                physics.is_grounded = true;
                physics.velocity.y = 0.0;
            }
        }
        e.transform.y += physics.velocity.y * input.dt;
    }
}

fn render_sprites(world: &World) {
    for entity in &world.entities {
        let Some(ref player) = entity.sprite else {
            continue;
        };
        let frame = player.sprite.frame();
        draw_texture_ex(
            &player.texture,
            entity.transform.x - ORIGINAL_SPRITE_SIZE,
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
            return;
        };
        if physics.is_grounded {
            player.sprite.update();
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
    build_textures_atlas();
    let screen_w = screen_width();
    let screen_h = screen_height();

    let entity = Entity::new(Rect {
        x: GAME_SPRITE_SIZE,
        y: 40.0,
        w: GAME_SPRITE_SIZE,
        h: GAME_SPRITE_SIZE,
    })
    .with_render(BLACK)
    .with_physics(Physics::new())
    .with_sprite(texture, sprite);

    let world = World::new().spawn(entity);
    let mut game = Game::new(world)
        .with_update_system(gravity_engine)
        .with_render_systems(vec![render_sprites]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
        };
        normalise_camera(screen_w, screen_h);
        background::render_paralax_background(&mut para);
        game.render();
        game.update(&input);
        update_sprites(&mut game.world);
        next_frame().await;
    }
}
