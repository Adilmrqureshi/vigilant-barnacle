use macroquad::prelude::*;
use shared::{Entity, Input, Transform, World, render_text};

const DEFAULT_SIZE: f32 = 64.0;

#[macroquad::main("My game")]
async fn main() {
    let mut gameover = false;
    let player_id = 1;
    let enemy_id = 2;
    let edge = screen_width() / 2.0;

    let player_ent = Entity::new(player_id, -100.0, 0.0);
    let enemy_ent = Entity::new(
        enemy_id,
        // 3.0 is to add some padding so the user has time
        // to react at the start of the game
        screen_width(),
        0.0,
    );

    let mut world = World::new();

    world.spawn(player_ent.with_jump(300.0, 0.0).with_collide().with_render(
        DEFAULT_SIZE,
        DEFAULT_SIZE,
        YELLOW,
    ));
    world.spawn(
        enemy_ent
            .with_collide()
            // By making the speed a factor of screen width, the speed is proportional to the size
            // of the screen
            .with_move(-screen_width(), 0.0)
            .with_render(DEFAULT_SIZE, DEFAULT_SIZE, RED),
    );

    let time = get_frame_time();
    let mut score = 0.0;

    loop {
        clear_background(DARKGREEN);
        world.set_origin(0.0, 200.0);

        if !gameover {
            score += get_frame_time() * 100.0;
            let input = Input {
                dt: time,
                is_jump: !gameover && is_key_pressed(KeyCode::Space),
            };
            let Some(ref c) = world.find(player_id).unwrap().collide else {
                continue;
            };

            if c.is_collided {
                gameover = true;
                continue;
            }

            world.update(&input);

            if world.find(enemy_id).unwrap().transform.x < -edge - DEFAULT_SIZE {
                world.find_mut(enemy_id).unwrap().set_position(
                    screen_width() + DEFAULT_SIZE * rand::gen_range(2, 10) as f32,
                    0.0 + (DEFAULT_SIZE * 2.0) * rand::gen_range(0, 2) as f32,
                );
            }
        }

        let text = format!("{:.0}", score);
        let text_dimensions = measure_text(&text, None, 50, 1.0);
        let score_pos = Transform {
            x: 10.0,
            y: 10.0 + text_dimensions.height,
        };

        world.render();
        draw_rectangle(-screen_width() / 2.0, -100.0, screen_width(), 100.0, BLUE);

        render_text(&mut world, &text, 20.0, &score_pos, WHITE);

        if gameover {
            let text = "GAME OVER!";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            let pos = Transform {
                x: screen_width() / 2.0 - text_dimensions.width / 2.0,
                y: screen_height() / 2.0 - text_dimensions.height / 2.0,
            };

            render_text(&mut world, text, 50.0, &pos, RED);
            if is_key_pressed(KeyCode::Space) {
                score = 0.0;
                gameover = false;
                world.find_mut(player_id).unwrap().set_position(-100.0, 0.0);
                world
                    .find_mut(enemy_id)
                    .unwrap()
                    .set_position(screen_width() + DEFAULT_SIZE, 0.0);
                world.reset();
            }
        }

        next_frame().await
    }
}
