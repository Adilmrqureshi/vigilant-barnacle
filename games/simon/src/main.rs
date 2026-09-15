mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const BOARD_W: f32 = 480.0;
const BOARD_H: f32 = 480.0;
const GAP: f32 = 14.0;
const SHOW_TIME: f32 = 0.55;
const FLASH_TIME: f32 = 0.35;

const BG_COLOR: Color = Color::new(0.05, 0.05, 0.09, 1.0);
const BOARD_COLOR: Color = Color::new(0.09, 0.09, 0.14, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
// Dim resting colors; a flashing quadrant is drawn brightened.
const QUADRANT_COLORS: [Color; 4] = [
    Color::new(0.15, 0.45, 0.20, 1.0),
    Color::new(0.55, 0.15, 0.15, 1.0),
    Color::new(0.55, 0.48, 0.12, 1.0),
    Color::new(0.15, 0.30, 0.55, 1.0),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Phase {
    Showing,
    Listening,
}

// Resource: the sequence and where we are in showing/repeating it.
pub struct SimonState {
    pub sequence: Vec<usize>,
    pub phase: Phase,
    pub show_index: usize,
    pub input_index: usize,
    pub timer: f32,
    pub flash: Option<(usize, f32)>,
}

impl SimonState {
    pub fn new() -> Self {
        Self {
            sequence: vec![rand::gen_range(0, 4) as usize],
            phase: Phase::Showing,
            show_index: 0,
            input_index: 0,
            timer: SHOW_TIME,
            flash: None,
        }
    }
}

// The board is drawn rotated 45° clockwise so the quadrants sit at the
// top / right / bottom / left, matching the directional controls.
const BOARD_ROTATION: f32 = std::f32::consts::FRAC_PI_4;

fn quadrant_rect(i: usize) -> Rect {
    // The 2x2 grid lives in the square inscribed in the board so that,
    // once rotated, the diamond exactly fills the BOARD_W x BOARD_H box.
    let inner = BOARD_W / std::f32::consts::SQRT_2;
    let margin = (BOARD_W - inner) / 2.0;
    let side = (inner - GAP * 3.0) / 2.0;
    Rect {
        x: margin + GAP + (i % 2) as f32 * (side + GAP),
        y: margin + GAP + (i / 2) as f32 * (side + GAP),
        w: side,
        h: side,
    }
}

// Rotates a board-space point by BOARD_ROTATION around the board centre.
fn rotate_about_center(x: f32, y: f32) -> (f32, f32) {
    let (cx, cy) = (BOARD_W / 2.0, BOARD_H / 2.0);
    let (dx, dy) = (x - cx, y - cy);
    let (sin, cos) = BOARD_ROTATION.sin_cos();
    (cx + dx * cos - dy * sin, cy + dx * sin + dy * cos)
}

fn brighten(c: Color) -> Color {
    Color::new(
        (c.r * 1.9).min(1.0),
        (c.g * 1.9).min(1.0),
        (c.b * 1.9).min(1.0),
        1.0,
    )
}

// Maps the frame's directional input to a quadrant press. On the rotated
// board: Up = top, Right = right, Left = left, Down = bottom.
pub fn pressed_quadrant(input: &Input) -> Option<usize> {
    if input.up {
        Some(0)
    } else if input.right {
        Some(1)
    } else if input.left {
        Some(2)
    } else if input.down {
        Some(3)
    } else {
        None
    }
}

fn flash_system(world: &mut World, _state: &mut GameState, input: &Input) {
    if let Some(simon) = world.get_resource_mut::<SimonState>() {
        if let Some((q, t)) = simon.flash {
            let t = t - input.dt;
            simon.flash = if t > 0.0 { Some((q, t)) } else { None };
        }
    }
}

fn show_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let Some(simon) = world.get_resource_mut::<SimonState>() else {
        return;
    };
    if simon.phase != Phase::Showing {
        return;
    }

    simon.timer -= input.dt;
    if simon.timer > 0.0 {
        return;
    }

    if simon.show_index >= simon.sequence.len() {
        simon.phase = Phase::Listening;
        simon.input_index = 0;
    } else {
        simon.flash = Some((simon.sequence[simon.show_index], FLASH_TIME));
        simon.show_index += 1;
        simon.timer = SHOW_TIME;
    }
}

fn listen_system(world: &mut World, state: &mut GameState, input: &Input) {
    let Some(pressed) = pressed_quadrant(input) else {
        return;
    };
    let Some(simon) = world.get_resource_mut::<SimonState>() else {
        return;
    };
    if simon.phase != Phase::Listening {
        return;
    }

    simon.flash = Some((pressed, FLASH_TIME));

    if pressed != simon.sequence[simon.input_index] {
        state.game_over = true;
        return;
    }

    simon.input_index += 1;
    if simon.input_index >= simon.sequence.len() {
        state.score = simon.sequence.len() as f32;
        simon.sequence.push(rand::gen_range(0, 4) as usize);
        simon.phase = Phase::Showing;
        simon.show_index = 0;
        simon.timer = SHOW_TIME;
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
    let (cx, cy) = (ox + BOARD_W / 2.0, oy + BOARD_H / 2.0);

    // Background and border as a diamond (a square rotated by BOARD_ROTATION;
    // draw_poly with 4 sides and 0° starts its first vertex pointing right).
    draw_poly(cx, cy, 4, BOARD_W / 2.0, 0.0, BOARD_COLOR);
    draw_poly_lines(cx, cy, 4, BOARD_W / 2.0 + 3.0, 0.0, 4.0, BORDER_COLOR);

    let flashing = world
        .get_resource::<SimonState>()
        .and_then(|s| s.flash)
        .map(|(q, _)| q);

    for (i, e) in world.entities.iter().enumerate() {
        let Some(render) = &e.render else {
            continue;
        };
        let color = if flashing == Some(i) {
            brighten(render.color)
        } else {
            render.color
        };
        let (qx, qy) = rotate_about_center(
            e.transform.x + e.transform.w / 2.0,
            e.transform.y + e.transform.h / 2.0,
        );
        draw_rectangle_ex(
            ox + qx,
            oy + qy,
            e.transform.w,
            e.transform.h,
            DrawRectangleParams {
                offset: vec2(0.5, 0.5),
                rotation: BOARD_ROTATION,
                color,
            },
        );
    }
}

fn ui_system(world: &World, state: &GameState) {
    let (ox, oy) = board_offset();

    draw_text(&format!("SCORE {}", state.score as i32), ox, oy - 14.0, 32.0, WHITE);

    let label = match world.get_resource::<SimonState>().map(|s| s.phase) {
        Some(Phase::Showing) => "WATCH...",
        Some(Phase::Listening) => "YOUR TURN",
        None => "",
    };
    let dims = measure_text(label, None, 26, 1.0);
    draw_text(label, ox + BOARD_W - dims.width, oy - 14.0, 26.0, GRAY);

    ui::draw_help_line(
        "repeat the sequence with the ARROW KEYS or WASD",
        Rect::new(ox, oy, BOARD_W, BOARD_H),
    );

    if state.game_over {
        ui::draw_game_over_overlay("GAME OVER!", RED);
    }
}

pub fn spawn_initial(world: &mut World) {
    for i in 0..4 {
        world
            .entities
            .push(Entity::new(quadrant_rect(i)).with_render(QUADRANT_COLORS[i]));
    }
    world.insert_resource(SimonState::new());
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world);
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Simon")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);

    let mut game = Game::new(world)
        .with_update_systems(vec![flash_system, show_system, listen_system])
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
