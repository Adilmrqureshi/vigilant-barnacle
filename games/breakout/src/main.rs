mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const BOARD_W: f32 = 640.0;
const BOARD_H: f32 = 480.0;
const PADDLE_W: f32 = 96.0;
const PADDLE_H: f32 = 16.0;
const PADDLE_Y: f32 = BOARD_H - 40.0;
const PADDLE_SPEED: f32 = 420.0;
const BALL_SIZE: f32 = 12.0;
const BALL_SPEED: f32 = 300.0;
const MAX_DEFLECT: f32 = 260.0;
const BRICK_COLS: i32 = 8;
const BRICK_ROWS: i32 = 5;
const BRICK_W: f32 = 72.0;
const BRICK_H: f32 = 20.0;
const BRICK_GAP: f32 = 4.0;
const BRICK_TOP: f32 = 60.0;
const START_LIVES: u32 = 3;

const BG_COLOR: Color = Color::new(0.06, 0.06, 0.10, 1.0);
const BOARD_COLOR: Color = Color::new(0.10, 0.10, 0.16, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
const PADDLE_COLOR: Color = Color::new(0.75, 0.85, 1.00, 1.0);
const BALL_COLOR: Color = WHITE;
const ROW_COLORS: [Color; 5] = [
    Color::new(0.94, 0.30, 0.30, 1.0),
    Color::new(0.96, 0.60, 0.25, 1.0),
    Color::new(0.95, 0.85, 0.30, 1.0),
    Color::new(0.45, 0.85, 0.40, 1.0),
    Color::new(0.40, 0.65, 0.95, 1.0),
];

// Resource: lives and the win flag, which GameState doesn't model.
pub struct BreakoutState {
    pub lives: u32,
    pub won: bool,
}

fn reset_ball(ball: &mut Entity) {
    ball.transform.x = BOARD_W / 2.0 - BALL_SIZE / 2.0;
    ball.transform.y = PADDLE_Y - 60.0;
    if let Some(physics) = &mut ball.physics {
        physics.velocity.x = rand::gen_range(-120.0, 120.0);
        physics.velocity.y = -BALL_SPEED;
    }
}

fn paddle_input_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for paddle in world.with_tag_mut(Tag::Player) {
        if input.left {
            paddle.transform.x -= PADDLE_SPEED * input.dt;
        }
        if input.right {
            paddle.transform.x += PADDLE_SPEED * input.dt;
        }
        paddle.transform.x = paddle.transform.x.clamp(0.0, BOARD_W - paddle.transform.w);
    }
}

fn ball_system(world: &mut World, state: &mut GameState, input: &Input) {
    let paddle = world
        .with_tag(Tag::Player)
        .next()
        .map(|p| p.transform);

    let mut lost_ball = false;
    for ball in world.with_tag_mut(Tag::Ball) {
        let Some(physics) = &mut ball.physics else {
            continue;
        };
        ball.transform.x += physics.velocity.x * input.dt;
        ball.transform.y += physics.velocity.y * input.dt;

        // Side and top walls.
        if ball.transform.x <= 0.0 {
            ball.transform.x = 0.0;
            physics.velocity.x = physics.velocity.x.abs();
        }
        if ball.transform.x + ball.transform.w >= BOARD_W {
            ball.transform.x = BOARD_W - ball.transform.w;
            physics.velocity.x = -physics.velocity.x.abs();
        }
        if ball.transform.y <= 0.0 {
            ball.transform.y = 0.0;
            physics.velocity.y = physics.velocity.y.abs();
        }

        // Paddle: reflect upward, deflect by hit offset.
        if let Some(paddle) = paddle {
            if physics.velocity.y > 0.0 && ball.transform.overlaps(&paddle) {
                ball.transform.y = paddle.y - ball.transform.h;
                physics.velocity.y = -physics.velocity.y.abs();
                let ball_cx = ball.transform.x + ball.transform.w / 2.0;
                let paddle_cx = paddle.x + paddle.w / 2.0;
                physics.velocity.x = (ball_cx - paddle_cx) / (paddle.w / 2.0) * MAX_DEFLECT;
            }
        }

        // Bottom: lose a life.
        if ball.transform.y > BOARD_H {
            lost_ball = true;
            reset_ball(ball);
        }
    }

    if lost_ball {
        if let Some(breakout) = world.get_resource_mut::<BreakoutState>() {
            breakout.lives = breakout.lives.saturating_sub(1);
            if breakout.lives == 0 {
                state.game_over = true;
            }
        }
    }
}

fn brick_collision_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let balls: Vec<Rect> = world.with_tag(Tag::Ball).map(|b| b.transform).collect();

    let mut hits = 0;
    for brick in world.with_tag_mut(Tag::Brick) {
        if balls.iter().any(|b| b.overlaps(&brick.transform)) {
            brick.alive = false;
            hits += 1;
        }
    }

    if hits > 0 {
        state.score += hits as f32;
        // A simple vertical reflection is enough for a first version.
        for ball in world.with_tag_mut(Tag::Ball) {
            if let Some(physics) = &mut ball.physics {
                physics.velocity.y = -physics.velocity.y;
            }
        }
        world.despawn_dead();
    }
}

fn win_system(world: &mut World, state: &mut GameState, _input: &Input) {
    if world.with_tag(Tag::Brick).next().is_none() {
        if let Some(breakout) = world.get_resource_mut::<BreakoutState>() {
            breakout.won = true;
        }
        state.game_over = true;
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

    draw_rectangle(ox, oy, BOARD_W, BOARD_H, BOARD_COLOR);
    draw_rectangle_lines(ox - 2.0, oy - 2.0, BOARD_W + 4.0, BOARD_H + 4.0, 4.0, BORDER_COLOR);

    for e in &world.entities {
        let Some(render) = &e.render else {
            continue;
        };
        draw_rectangle(
            ox + e.transform.x,
            oy + e.transform.y,
            e.transform.w,
            e.transform.h,
            render.color,
        );
    }
}

fn ui_system(world: &World, state: &GameState) {
    let (ox, oy) = board_offset();

    draw_text(&format!("SCORE {}", state.score as i32), ox, oy - 14.0, 32.0, WHITE);

    if let Some(breakout) = world.get_resource::<BreakoutState>() {
        let lives = format!("LIVES {}", breakout.lives);
        let dims = measure_text(&lives, None, 32, 1.0);
        draw_text(&lives, ox + BOARD_W - dims.width, oy - 14.0, 32.0, WHITE);

        ui::draw_help_line(
            "LEFT/RIGHT or A/D: move - clear every brick",
            Rect::new(ox, oy, BOARD_W, BOARD_H),
        );

        if state.game_over {
            let text = if breakout.won { "YOU WIN!" } else { "GAME OVER!" };
            ui::draw_game_over_overlay(text, if breakout.won { GREEN } else { RED });
        }
    }
}

fn spawn_initial(world: &mut World) {
    world.entities.push(
        Entity::new(Rect {
            x: BOARD_W / 2.0 - PADDLE_W / 2.0,
            y: PADDLE_Y,
            w: PADDLE_W,
            h: PADDLE_H,
        })
        .with_tag(Tag::Player)
        .with_render(PADDLE_COLOR),
    );

    let grid_w = BRICK_COLS as f32 * BRICK_W + (BRICK_COLS - 1) as f32 * BRICK_GAP;
    let left = (BOARD_W - grid_w) / 2.0;
    for row in 0..BRICK_ROWS {
        for col in 0..BRICK_COLS {
            world.entities.push(
                Entity::new(Rect {
                    x: left + col as f32 * (BRICK_W + BRICK_GAP),
                    y: BRICK_TOP + row as f32 * (BRICK_H + BRICK_GAP),
                    w: BRICK_W,
                    h: BRICK_H,
                })
                .with_tag(Tag::Brick)
                .with_render(ROW_COLORS[row as usize % ROW_COLORS.len()]),
            );
        }
    }

    let mut ball = Entity::new(Rect {
        x: 0.0,
        y: 0.0,
        w: BALL_SIZE,
        h: BALL_SIZE,
    })
    .with_tag(Tag::Ball)
    .with_render(BALL_COLOR)
    .with_physics(Physics::new());
    reset_ball(&mut ball);
    world.entities.push(ball);
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world);
        game.world.insert_resource(BreakoutState {
            lives: START_LIVES,
            won: false,
        });
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Breakout")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);
    world.insert_resource(BreakoutState {
        lives: START_LIVES,
        won: false,
    });

    let mut game = Game::new(world)
        .with_update_systems(vec![
            paddle_input_system,
            ball_system,
            brick_collision_system,
            win_system,
        ])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
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
