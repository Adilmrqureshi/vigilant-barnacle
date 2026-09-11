use macroquad::prelude::*;
use shared::{Entity, Input, Transform, World, render_text};

const DEFAULT_SIZE: f32 = 64.0;
const MARGIN: f32 = 24.0;

const BG_COLOR: Color = Color::new(0.06, 0.06, 0.10, 1.0);
const BOARD_COLOR: Color = Color::new(0.10, 0.10, 0.16, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
const GROUND_COLOR: Color = Color::new(0.16, 0.19, 0.30, 1.0);

// Enemy speed as a fraction of screen width per second: starts at a medium
// pace and ramps up the longer a run lasts, capped so it stays beatable.
const START_SPEED_RATIO: f32 = 0.4;
const MAX_SPEED_RATIO: f32 = 0.85;
const SPEED_RAMP_PER_SEC: f32 = 0.0075;

#[macroquad::main("My game")]
async fn main() {
    let mut gameover = false;
    let player_id = 1;
    let enemy_id = 2;
    let edge = screen_width() / 2.0;

    let player_ent = Entity::new(player_id, -screen_width() / 4.0, 0.0);
    let enemy_ent = Entity::new(
        enemy_id,
        // 3.0 is to add some padding so the user has time
        // to react at the start of the game
        screen_width(),
        0.0,
    );

    let mut world = World::new();

    world.set_origin(0.0, 200.0);
    world.spawn(player_ent.with_jump(350.0, 0.0).with_collide().with_render(
        DEFAULT_SIZE,
        DEFAULT_SIZE,
        YELLOW,
    ));
    world.spawn(
        enemy_ent
            .with_collide()
            // By making the speed a factor of screen width, the speed is proportional to the size
            // of the screen
            .with_move(-screen_width() * START_SPEED_RATIO, 0.0)
            .with_render(DEFAULT_SIZE, DEFAULT_SIZE, RED),
    );

    let mut score = 0.0;
    let mut elapsed = 0.0;

    // Playfield frame in world coordinates: the visible area inset by MARGIN.
    // The camera is centred on the world origin set above (0, 200), y-up.
    let field = Rect {
        x: -screen_width() / 2.0 + MARGIN,
        y: 200.0 - screen_height() / 2.0 + MARGIN,
        w: screen_width() - MARGIN * 2.0,
        h: screen_height() - MARGIN * 2.0,
    };

    loop {
        clear_background(BG_COLOR);
        draw_rectangle(field.x, field.y, field.w, field.h, BOARD_COLOR);
        draw_rectangle_lines(
            field.x - 2.0,
            field.y - 2.0,
            field.w + 4.0,
            field.h + 4.0,
            4.0,
            BORDER_COLOR,
        );

        if !gameover {
            let dt = get_frame_time();
            score += dt * 100.0;
            elapsed += dt;
            let input = Input {
                dt,
                is_jump: !gameover && is_key_pressed(KeyCode::Space),
            };

            let speed_ratio =
                (START_SPEED_RATIO + SPEED_RAMP_PER_SEC * elapsed).min(MAX_SPEED_RATIO);
            if let Some(ref mut movement) = world.find_mut(enemy_id).unwrap().movement {
                movement.velocity.x = -screen_width() * speed_ratio;
            }

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

        world.render();
        // Ground: from the bottom of the frame up to y = 0.
        draw_rectangle(field.x, field.y, field.w, -field.y, GROUND_COLOR);

        let text = format!("{:.0}", score);
        let text_dimensions = measure_text(&text, None, 50, 1.0);
        let score_pos = Transform {
            x: MARGIN + 10.0,
            y: MARGIN + 10.0 + text_dimensions.height,
        };
        render_text(&mut world, &text, 40.0, &score_pos, WHITE);

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
                elapsed = 0.0;
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
