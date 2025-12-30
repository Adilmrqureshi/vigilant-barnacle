use macroquad::prelude::*;
use shared::{Entity, Input, World};

const DEFAULT_SIZE: f32 = 64.0;
const GROUND_HEIGHT: f32 = 100.0;

#[macroquad::main("My game")]
async fn main() {
    let ground = screen_height() - GROUND_HEIGHT;
    let mut gameover = false;
    let player_id = 1;
    let enemy_id = 2;
    let player_ent = Entity::new(player_id, 100.0, screen_height() - GROUND_HEIGHT);
    let enemy_ent = Entity::new(
        enemy_id,
        // 3.0 is to add some padding so the user has time
        // to react at the start of the game
        screen_width() + DEFAULT_SIZE * 3.0,
        screen_height() - GROUND_HEIGHT,
    );

    let mut world = World::new();

    world.spawn(
        player_ent
            .with_jump(300.0, ground)
            .with_collide()
            .with_render(DEFAULT_SIZE, DEFAULT_SIZE, YELLOW),
    );
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

        if !gameover {
            score += get_frame_time() * 100.0;
            let input = Input {
                dt: time,
                is_jump: is_key_pressed(KeyCode::Space),
            };
            let Some(ref c) = world.entities[0].collide else {
                continue;
            };
            if c.is_collided {
                gameover = true;
            }
            world.update(&input);
            if world.find(enemy_id).unwrap().transform.x < -DEFAULT_SIZE {
                world.find_mut(enemy_id).unwrap().set_position(
                    screen_width() + DEFAULT_SIZE * rand::gen_range(2, 10) as f32,
                    ground - (DEFAULT_SIZE * 2.0) * rand::gen_range(0, 2) as f32,
                );
            }
        }

        let text = format!("{:.0}", score);
        let text_dimensions = measure_text(&text, None, 50, 1.0);
        draw_text(&text, 10.0, 10.0 + text_dimensions.height, 32.0, WHITE);

        draw_rectangle(
            world.find(player_id).unwrap().transform.x,
            world.find(player_id).unwrap().transform.y,
            DEFAULT_SIZE,
            DEFAULT_SIZE,
            YELLOW,
        );
        draw_rectangle(
            world.find(enemy_id).unwrap().transform.x,
            world.find(enemy_id).unwrap().transform.y,
            DEFAULT_SIZE,
            DEFAULT_SIZE,
            RED,
        );
        draw_rectangle(0.0, ground + DEFAULT_SIZE, screen_width(), 100.0, BLUE);

        if gameover {
            let text = "GAME OVER!";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            draw_text(
                text,
                screen_width() / 2.0 - text_dimensions.width / 2.0,
                screen_height() / 2.0 - text_dimensions.height / 2.0,
                50.0,
                RED,
            );
        }

        next_frame().await
    }
}
