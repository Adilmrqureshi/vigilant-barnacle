mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const CELL: f32 = 40.0;
const COLS: i32 = 16;
const ROWS: i32 = 12;
const BOARD_W: f32 = COLS as f32 * CELL;
const BOARD_H: f32 = ROWS as f32 * CELL;
const START_ROW: i32 = 11;

const BG_COLOR: Color = Color::new(0.05, 0.06, 0.09, 1.0);
const ROAD_COLOR: Color = Color::new(0.10, 0.10, 0.14, 1.0);
const SAFE_COLOR: Color = Color::new(0.12, 0.17, 0.13, 1.0);
const GOAL_COLOR: Color = Color::new(0.16, 0.28, 0.16, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
const FROG_COLOR: Color = Color::new(0.49, 0.90, 0.23, 1.0);
const CAR_COLORS: [Color; 4] = [
    Color::new(0.90, 0.35, 0.30, 1.0),
    Color::new(0.35, 0.55, 0.90, 1.0),
    Color::new(0.90, 0.75, 0.30, 1.0),
    Color::new(0.75, 0.45, 0.85, 1.0),
];

// (row, speed px/s and direction, number of cars, car width in cells)
pub const LANES: [(i32, f32, i32, f32); 9] = [
    (1, -90.0, 3, 1.5),
    (2, 130.0, 2, 2.0),
    (3, -70.0, 3, 1.0),
    (4, 100.0, 2, 1.5),
    (5, -150.0, 2, 1.0),
    (7, 80.0, 3, 1.5),
    (8, -120.0, 2, 2.0),
    (9, 100.0, 3, 1.0),
    (10, -80.0, 2, 1.5),
];

pub fn frog_start() -> Rect {
    Rect {
        x: (COLS / 2) as f32 * CELL,
        y: START_ROW as f32 * CELL,
        w: CELL,
        h: CELL,
    }
}

fn frog_input_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for frog in world.with_tag_mut(Tag::Player) {
        if input.up {
            frog.transform.y -= CELL;
        }
        if input.down {
            frog.transform.y += CELL;
        }
        if input.left {
            frog.transform.x -= CELL;
        }
        if input.right {
            frog.transform.x += CELL;
        }
        frog.transform.x = frog.transform.x.clamp(0.0, BOARD_W - CELL);
        frog.transform.y = frog.transform.y.clamp(0.0, START_ROW as f32 * CELL);
    }
}

fn car_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for car in world.with_tag_mut(Tag::Enemy) {
        let Some(physics) = &car.physics else {
            continue;
        };
        car.transform.x += physics.velocity.x * input.dt;

        // Wrap around with the car fully off screen on both sides.
        let span = BOARD_W + car.transform.w * 2.0;
        if car.transform.x > BOARD_W + car.transform.w {
            car.transform.x -= span;
        }
        if car.transform.x < -car.transform.w * 2.0 {
            car.transform.x += span;
        }
    }
}

fn collision_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let Some(frog) = world.with_tag(Tag::Player).next().map(|f| f.transform) else {
        return;
    };
    // Shrink the frog's hitbox a little so grazing a car feels fair.
    let hitbox = Rect {
        x: frog.x + 6.0,
        y: frog.y + 6.0,
        w: frog.w - 12.0,
        h: frog.h - 12.0,
    };
    if world.with_tag(Tag::Enemy).any(|c| c.transform.overlaps(&hitbox)) {
        state.game_over = true;
    }
}

fn goal_system(world: &mut World, state: &mut GameState, _input: &Input) {
    for frog in world.with_tag_mut(Tag::Player) {
        if frog.transform.y < CELL / 2.0 {
            state.score += 1.0;
            frog.transform = frog_start();
        }
    }
}

fn board_offset() -> (f32, f32) {
    (
        (screen_width() - BOARD_W) / 2.0,
        (screen_height() - BOARD_H) / 2.0,
    )
}

fn render_board(world: &World, _state: &GameState) {
    let (ox, oy) = board_offset();

    for row in 0..ROWS {
        let color = match row {
            0 => GOAL_COLOR,
            6 | 11 => SAFE_COLOR,
            _ => ROAD_COLOR,
        };
        draw_rectangle(ox, oy + row as f32 * CELL, BOARD_W, CELL, color);
        // Lane markings on road rows.
        if color.r == ROAD_COLOR.r && row > 0 {
            let mut x = 8.0;
            while x < BOARD_W {
                draw_rectangle(
                    ox + x,
                    oy + row as f32 * CELL - 1.0,
                    16.0,
                    2.0,
                    Color::new(0.25, 0.25, 0.32, 1.0),
                );
                x += 48.0;
            }
        }
    }
    draw_rectangle_lines(ox - 2.0, oy - 2.0, BOARD_W + 4.0, BOARD_H + 4.0, 4.0, BORDER_COLOR);

    for e in &world.entities {
        let Some(render) = &e.render else {
            continue;
        };
        draw_rectangle(
            ox + e.transform.x + 2.0,
            oy + e.transform.y + 4.0,
            e.transform.w - 4.0,
            e.transform.h - 8.0,
            render.color,
        );
    }
}

fn ui_system(_world: &World, state: &GameState) {
    let (ox, oy) = board_offset();

    draw_text(&format!("CROSSED {}", state.score as i32), ox, oy - 14.0, 32.0, WHITE);

    ui::draw_help_line(
        "ARROWS or WASD: hop - dodge the traffic, reach the top",
        Rect::new(ox, oy, BOARD_W, BOARD_H),
    );

    if state.game_over {
        ui::draw_game_over_overlay("SQUASHED!", RED);
    }
}

pub fn spawn_initial(world: &mut World) {
    world.entities.push(
        Entity::new(frog_start())
            .with_tag(Tag::Player)
            .with_render(FROG_COLOR),
    );

    for (lane_index, (row, speed, count, width_cells)) in LANES.iter().enumerate() {
        let w = width_cells * CELL;
        for i in 0..*count {
            // Space cars evenly along the lane, staggered per lane.
            let x = i as f32 * (BOARD_W / *count as f32) + lane_index as f32 * 53.0;
            let mut car = Entity::new(Rect {
                x: x % BOARD_W,
                y: *row as f32 * CELL,
                w,
                h: CELL,
            })
            .with_tag(Tag::Enemy)
            .with_render(CAR_COLORS[lane_index % CAR_COLORS.len()])
            .with_physics(Physics::new());
            if let Some(physics) = &mut car.physics {
                physics.velocity.x = *speed;
            }
            world.entities.push(car);
        }
    }
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world);
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Frogger")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);

    let mut game = Game::new(world)
        .with_update_systems(vec![
            frog_input_system,
            car_system,
            collision_system,
            goal_system,
        ])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            up: is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
            down: is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
            left: is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
            right: is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
            screen_width: screen_width(),
            screen_height: screen_height(),
            ..Default::default()
        };

        clear_background(BG_COLOR);
        if !game.state.game_over {
            game.update(&input);
        } else {
            restart_game(&mut game, &input);
        }
        game.render();
        next_frame().await;
    }
}
