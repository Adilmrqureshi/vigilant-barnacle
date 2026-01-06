use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use shared::{Entity, GameState, Input, Transform, World, render_text};

const FRAGMENT_SHADER: &str = include_str!("./assets/starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

const MOVEMENT_SPEED: f32 = 100.0;

fn create_user() -> Entity {
    let mut entity = Entity::new(1, 0.0, 0.0).with_collide();
    entity.set_dimensions(32.0, 48.0);
    entity
}

#[macroquad::main("Shooter")]
async fn main() {
    console_error_panic_hook::set_once();
    let mut world = World::new();
    world.spawn(create_user());
    set_pc_assets_folder("./assets");
    // Texture2D stores image data in GPU (Image uses the CPU)
    let ship_texture: Texture2D = load_texture("ship.png").await.expect("Couldn't load file");
    ship_texture.set_filter(FilterMode::Linear);
    let bullet_texture: Texture2D = load_texture("laser-bolts.png")
        .await
        .expect("Couldn't load file");
    bullet_texture.set_filter(FilterMode::Linear);
    let enemy_small_texture: Texture2D = load_texture("enemy-small.png")
        .await
        .expect("Couldn't load file");
    enemy_small_texture.set_filter(FilterMode::Nearest);
    build_textures_atlas();
    let mut direction_modifier: f32 = 0.0;

    let render_target = render_target(320, 150);

    render_target.texture.set_filter(FilterMode::Nearest);
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("iResolution", UniformType::Float2),
                UniformDesc::new("direction_modifier", UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .unwrap();
    let mut ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[
            Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left".to_string(),
                row: 2,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right".to_string(),
                row: 4,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    let mut enemy_small_sprite = AnimatedSprite::new(
        17,
        16,
        &[Animation {
            name: "enemy_small".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );
    loop {
        clear_background(BLACK);
        set_default_camera();
        material.set_uniform("iResolution", (screen_width(), screen_height()));
        material.set_uniform("direction_modifier", direction_modifier);
        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();
        world.set_default_origin();

        let delta_time = get_frame_time();
        if world.state == GameState::Running {
            let input = Input {
                dt: delta_time,
                is_jump: false,
            };
            world.new_update(&input);
            ship_sprite.set_animation(0);
            if is_key_down(KeyCode::Right) {
                direction_modifier += 0.05 * delta_time;
                ship_sprite.set_animation(1);
                world.find_mut(1).unwrap().transform.x += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Left) {
                ship_sprite.set_animation(2);
                direction_modifier -= 0.05 * delta_time;
                world.find_mut(1).unwrap().transform.x -= MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Down) {
                world.find_mut(1).unwrap().transform.y -= MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Up) {
                world.find_mut(1).unwrap().transform.y += MOVEMENT_SPEED * delta_time;
            }
            if rand::gen_range(0, 99) >= 95 {
                let mut entity = Entity::new(
                    world.entities.len() as i32,
                    rand::gen_range(-screen_width() / 2.0 + 32.0, screen_width() / 2.0 - 32.0),
                    screen_width() / 2.0,
                )
                .with_move(0.0, -100.0)
                .with_collide();
                entity.set_dimensions(32.0, 32.0);
                world.spawn(entity);
            }

            for i in 0..world.entities.len() - 1 {
                if world.entities[i].id != 1 {
                    if world.entities[i].transform.y < -screen_width() {
                        world.despawn(world.entities[i].id);
                    }
                }
            }
        }

        let enemy_frame = enemy_small_sprite.frame();
        for i in 1..world.entities.len() - 1 {
            draw_texture_ex(
                &enemy_small_texture,
                world.entities[i].transform.x,
                world.entities[i].transform.y + 32.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(32.0, -32.0)),
                    source: Some(enemy_frame.source_rect),
                    ..Default::default()
                },
            )
        }

        if world.state == GameState::GameOver {
            let text = "GAME OVER!";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            let pos = Transform {
                x: screen_width() / 2.0 - text_dimensions.width / 2.0,
                y: screen_height() / 2.0 - text_dimensions.height / 2.0,
            };

            render_text(&mut world, text, 50.0, &pos, RED);

            if is_key_pressed(KeyCode::Space) {
                world.entities = vec![];
                world.spawn(create_user());
                world.state = GameState::Running;
                world.reset();
            }
        }

        let ship_frame = ship_sprite.frame();
        draw_texture_ex(
            &ship_texture,
            world.find_mut(1).unwrap().transform.x + ship_frame.dest_size.x * 2.0,
            world.find_mut(1).unwrap().transform.y + ship_frame.dest_size.y * 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(ship_frame.dest_size * -2.0),
                source: Some(ship_frame.source_rect),
                ..Default::default()
            },
        );

        ship_sprite.update();
        enemy_small_sprite.update();
        next_frame().await
    }
}
