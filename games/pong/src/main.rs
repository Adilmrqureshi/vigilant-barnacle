mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const COURT_W: f32 = 800.0;
const COURT_H: f32 = 480.0;
const PADDLE_W: f32 = 16.0;
const PADDLE_H: f32 = 96.0;
const PADDLE_MARGIN: f32 = 24.0;
const PADDLE_SPEED: f32 = 380.0;
const AI_SPEED: f32 = 260.0;
const AI_DECISION_INTERVAL: f32 = 0.25;
const AI_ERROR_CHANCE: f32 = 0.05;
const AI_ERROR_DURATION: f32 = 0.4;
const BALL_SIZE: f32 = 16.0;
const BALL_SPEED: f32 = 320.0;
const BALL_SPEEDUP: f32 = 1.05;
const MAX_DEFLECT: f32 = 280.0;
const WIN_SCORE: u32 = 7;

const BG_COLOR: Color = Color::new(0.07, 0.07, 0.10, 1.0);
const COURT_COLOR: Color = Color::new(0.10, 0.10, 0.15, 1.0);
const LINE_COLOR: Color = Color::new(0.35, 0.35, 0.50, 1.0);
const PLAYER_COLOR: Color = Color::new(0.45, 0.80, 1.00, 1.0);
const CPU_COLOR: Color = Color::new(1.00, 0.55, 0.45, 1.0);
const BALL_COLOR: Color = WHITE;

// Resource: pong keeps two scores, so GameState.score alone isn't enough.
pub struct Scores {
    pub player: u32,
    pub cpu: u32,
}

// Resource: makes the CPU fallible. Every AI_DECISION_INTERVAL it has an
// AI_ERROR_CHANCE of chasing the wrong direction for AI_ERROR_DURATION.
pub struct AiBrain {
    pub decision_timer: f32,
    pub error_timer: f32,
}

impl AiBrain {
    pub fn new() -> Self {
        Self {
            decision_timer: AI_DECISION_INTERVAL,
            error_timer: 0.0,
        }
    }
}

fn clamp_paddle(rect: &mut Rect) {
    rect.y = rect.y.clamp(0.0, COURT_H - rect.h);
}

fn serve_ball(ball: &mut Entity, toward_player: bool) {
    ball.transform.x = COURT_W / 2.0 - BALL_SIZE / 2.0;
    ball.transform.y = COURT_H / 2.0 - BALL_SIZE / 2.0;
    if let Some(physics) = &mut ball.physics {
        physics.velocity.x = if toward_player { -BALL_SPEED } else { BALL_SPEED };
        physics.velocity.y = rand::gen_range(-120.0, 120.0);
    }
}

fn player_input_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for paddle in world.with_tag_mut(Tag::Player) {
        if input.up {
            paddle.transform.y -= PADDLE_SPEED * input.dt;
        }
        if input.down {
            paddle.transform.y += PADDLE_SPEED * input.dt;
        }
        clamp_paddle(&mut paddle.transform);
    }
}

fn ai_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let Some(ball_cy) = world
        .with_tag(Tag::Ball)
        .next()
        .map(|b| b.transform.y + b.transform.h / 2.0)
    else {
        return;
    };

    let confused = world
        .get_resource_mut::<AiBrain>()
        .map(|brain| {
            brain.decision_timer -= input.dt;
            if brain.decision_timer <= 0.0 {
                brain.decision_timer = AI_DECISION_INTERVAL;
                if brain.error_timer <= 0.0 && rand::gen_range(0.0, 1.0) < AI_ERROR_CHANCE {
                    brain.error_timer = AI_ERROR_DURATION;
                }
            }
            if brain.error_timer > 0.0 {
                brain.error_timer -= input.dt;
                true
            } else {
                false
            }
        })
        .unwrap_or(false);

    for paddle in world.with_tag_mut(Tag::Enemy) {
        let paddle_cy = paddle.transform.y + paddle.transform.h / 2.0;
        let mut diff = ball_cy - paddle_cy;
        if confused {
            diff = -diff;
        }
        let step = AI_SPEED * input.dt;
        paddle.transform.y += diff.clamp(-step, step);
        clamp_paddle(&mut paddle.transform);
    }
}

fn ball_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let paddles: Vec<Rect> = world
        .entities
        .iter()
        .filter(|e| e.tag == Some(Tag::Player) || e.tag == Some(Tag::Enemy))
        .map(|e| e.transform)
        .collect();

    for ball in world.with_tag_mut(Tag::Ball) {
        let Some(physics) = &mut ball.physics else {
            continue;
        };
        ball.transform.x += physics.velocity.x * input.dt;
        ball.transform.y += physics.velocity.y * input.dt;

        // Top and bottom walls.
        if ball.transform.y <= 0.0 {
            ball.transform.y = 0.0;
            physics.velocity.y = physics.velocity.y.abs();
        }
        if ball.transform.y + ball.transform.h >= COURT_H {
            ball.transform.y = COURT_H - ball.transform.h;
            physics.velocity.y = -physics.velocity.y.abs();
        }

        for paddle in &paddles {
            if !ball.transform.overlaps(paddle) {
                continue;
            }
            let moving_toward = if paddle.x < COURT_W / 2.0 {
                physics.velocity.x < 0.0
            } else {
                physics.velocity.x > 0.0
            };
            if !moving_toward {
                continue;
            }

            // Reflect, speed up, and deflect based on where the paddle was hit.
            let speed = physics.velocity.x.abs() * BALL_SPEEDUP;
            if paddle.x < COURT_W / 2.0 {
                ball.transform.x = paddle.x + paddle.w;
                physics.velocity.x = speed;
            } else {
                ball.transform.x = paddle.x - ball.transform.w;
                physics.velocity.x = -speed;
            }
            let ball_cy = ball.transform.y + ball.transform.h / 2.0;
            let paddle_cy = paddle.y + paddle.h / 2.0;
            physics.velocity.y = (ball_cy - paddle_cy) / (paddle.h / 2.0) * MAX_DEFLECT;
        }
    }
}

fn score_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let conceded = {
        let Some(ball) = world.with_tag(Tag::Ball).next() else {
            return;
        };
        if ball.transform.x + ball.transform.w < 0.0 {
            Some(true) // past the player's edge: cpu scores
        } else if ball.transform.x > COURT_W {
            Some(false) // past the cpu's edge: player scores
        } else {
            None
        }
    };

    let Some(player_conceded) = conceded else {
        return;
    };

    let game_over = {
        let Some(scores) = world.get_resource_mut::<Scores>() else {
            return;
        };
        if player_conceded {
            scores.cpu += 1;
        } else {
            scores.player += 1;
        }
        scores.player >= WIN_SCORE || scores.cpu >= WIN_SCORE
    };

    if game_over {
        state.game_over = true;
    }

    for ball in world.with_tag_mut(Tag::Ball) {
        serve_ball(ball, player_conceded);
    }
}

fn court_offset() -> (f32, f32) {
    (
        (screen_width() - COURT_W) / 2.0,
        (screen_height() - COURT_H) / 2.0,
    )
}

fn render_court(world: &World, _state: &GameState) {
    let (ox, oy) = court_offset();

    draw_rectangle(ox, oy, COURT_W, COURT_H, COURT_COLOR);
    draw_rectangle_lines(ox - 2.0, oy - 2.0, COURT_W + 4.0, COURT_H + 4.0, 4.0, LINE_COLOR);

    // Dashed center line.
    let mut y = 8.0;
    while y < COURT_H {
        draw_rectangle(ox + COURT_W / 2.0 - 2.0, oy + y, 4.0, 16.0, LINE_COLOR);
        y += 32.0;
    }

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
    let (ox, oy) = court_offset();

    let Some(scores) = world.get_resource::<Scores>() else {
        return;
    };

    draw_text(
        &format!("{}", scores.player),
        ox + COURT_W / 4.0,
        oy - 14.0,
        48.0,
        PLAYER_COLOR,
    );
    draw_text(
        &format!("{}", scores.cpu),
        ox + COURT_W * 3.0 / 4.0,
        oy - 14.0,
        48.0,
        CPU_COLOR,
    );

    if state.game_over {
        let text = if scores.player > scores.cpu {
            "YOU WIN!"
        } else {
            "CPU WINS!"
        };
        let dims = measure_text(text, None, 50, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - dims.width / 2.0,
            screen_height() / 2.0 - dims.height / 2.0,
            60.0,
            WHITE,
        );

        let hint = "press SPACE to restart";
        let hint_dims = measure_text(hint, None, 30, 1.0);
        draw_text(
            hint,
            screen_width() / 2.0 - hint_dims.width / 2.0,
            screen_height() / 2.0 + 40.0,
            30.0,
            WHITE,
        );
    }
}

fn spawn_initial(world: &mut World) {
    world.entities.push(
        Entity::new(Rect {
            x: PADDLE_MARGIN,
            y: COURT_H / 2.0 - PADDLE_H / 2.0,
            w: PADDLE_W,
            h: PADDLE_H,
        })
        .with_tag(Tag::Player)
        .with_render(PLAYER_COLOR),
    );
    world.entities.push(
        Entity::new(Rect {
            x: COURT_W - PADDLE_MARGIN - PADDLE_W,
            y: COURT_H / 2.0 - PADDLE_H / 2.0,
            w: PADDLE_W,
            h: PADDLE_H,
        })
        .with_tag(Tag::Enemy)
        .with_render(CPU_COLOR),
    );

    let mut ball = Entity::new(Rect {
        x: 0.0,
        y: 0.0,
        w: BALL_SIZE,
        h: BALL_SIZE,
    })
    .with_tag(Tag::Ball)
    .with_render(BALL_COLOR)
    .with_physics(Physics::new());
    serve_ball(&mut ball, false);
    world.entities.push(ball);
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world);
        game.world.insert_resource(Scores { player: 0, cpu: 0 });
        game.world.insert_resource(AiBrain::new());
        game.state.game_over = false;
    }
}

#[macroquad::main("Pong")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);
    world.insert_resource(Scores { player: 0, cpu: 0 });
    world.insert_resource(AiBrain::new());

    let mut game = Game::new(world)
        .with_update_systems(vec![
            player_input_system,
            ai_system,
            ball_system,
            score_system,
        ])
        .with_render_systems(vec![render_court, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            a: false,
            up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
            down: is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
            left: false,
            right: false,
            screen_width: screen_width(),
            screen_height: screen_height(),
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
