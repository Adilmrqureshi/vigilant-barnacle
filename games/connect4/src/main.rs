mod tests;

use macroquad::prelude::*;
use shared_v2::*;

pub const COLS: usize = 7;
pub const ROWS: usize = 6;
pub const AI_DEPTH: u32 = 6;
const TILE: f32 = 68.0;
const GAP: f32 = 8.0;
const BOARD_W: f32 = COLS as f32 * TILE + (COLS + 1) as f32 * GAP;
const BOARD_H: f32 = ROWS as f32 * TILE + (ROWS + 1) as f32 * GAP;
const CPU_DELAY: f32 = 0.45;
// Search columns centre-out; it makes alpha-beta prune far more aggressively.
const COL_ORDER: [usize; COLS] = [3, 2, 4, 1, 5, 0, 6];

const BG_COLOR: Color = Color::new(0.06, 0.06, 0.10, 1.0);
const BOARD_COLOR: Color = Color::new(0.14, 0.22, 0.48, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
const HOLE_COLOR: Color = Color::new(0.06, 0.06, 0.10, 1.0);
const HUMAN_COLOR: Color = Color::new(0.90, 0.33, 0.28, 1.0);
const CPU_COLOR: Color = Color::new(0.93, 0.79, 0.25, 1.0);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Piece {
    Human,
    Cpu,
}

// grid[0] is the top row; pieces fall towards grid[ROWS - 1].
pub type Grid = [[Option<Piece>; COLS]; ROWS];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Outcome {
    HumanWin,
    CpuWin,
    Draw,
}

// Resource: board, whose turn it is, and the column the player is aiming at.
pub struct C4State {
    pub grid: Grid,
    pub turn: Piece,
    pub hover_col: usize,
    pub cpu_timer: f32,
    pub outcome: Option<Outcome>,
}

impl C4State {
    pub fn new() -> Self {
        Self {
            grid: [[None; COLS]; ROWS],
            turn: Piece::Human,
            hover_col: COLS / 2,
            cpu_timer: 0.0,
            outcome: None,
        }
    }
}

// Drops a piece into a column. Returns the row it landed in, or None if full.
pub fn drop_piece(grid: &mut Grid, col: usize, piece: Piece) -> Option<usize> {
    if col >= COLS {
        return None;
    }
    (0..ROWS).rev().find(|&r| grid[r][col].is_none()).map(|r| {
        grid[r][col] = Some(piece);
        r
    })
}

pub fn is_full(grid: &Grid) -> bool {
    grid[0].iter().all(|c| c.is_some())
}

fn windows(grid: &Grid) -> Vec<[Option<Piece>; 4]> {
    let mut out = Vec::with_capacity(69);
    let at = |r: i32, c: i32| grid[r as usize][c as usize];
    for r in 0..ROWS as i32 {
        for c in 0..COLS as i32 {
            for (dr, dc) in [(0, 1), (1, 0), (1, 1), (-1, 1)] {
                let (er, ec) = (r + 3 * dr, c + 3 * dc);
                if er >= 0 && er < ROWS as i32 && ec >= 0 && ec < COLS as i32 {
                    out.push([
                        at(r, c),
                        at(r + dr, c + dc),
                        at(r + 2 * dr, c + 2 * dc),
                        at(er, ec),
                    ]);
                }
            }
        }
    }
    out
}

pub fn find_winner(grid: &Grid) -> Option<Piece> {
    for w in windows(grid) {
        if let Some(p) = w[0] {
            if w.iter().all(|&x| x == Some(p)) {
                return Some(p);
            }
        }
    }
    None
}

// Heuristic for non-terminal positions, from the CPU's point of view.
fn evaluate(grid: &Grid) -> i32 {
    let mut score = 0;
    for w in windows(grid) {
        let cpu = w.iter().filter(|&&x| x == Some(Piece::Cpu)).count();
        let human = w.iter().filter(|&&x| x == Some(Piece::Human)).count();
        score += match (cpu, human) {
            (3, 0) => 50,
            (2, 0) => 10,
            (0, 3) => -80,
            (0, 2) => -10,
            _ => 0,
        };
    }
    // Centre column control is worth a little on its own.
    for r in 0..ROWS {
        score += match grid[r][COLS / 2] {
            Some(Piece::Cpu) => 6,
            Some(Piece::Human) => -6,
            None => 0,
        };
    }
    score
}

fn minimax(grid: &Grid, depth: u32, mut alpha: i32, mut beta: i32, maximising: bool) -> i32 {
    if let Some(winner) = find_winner(grid) {
        // Prefer quicker wins and slower losses.
        return match winner {
            Piece::Cpu => 1_000_000 + depth as i32,
            Piece::Human => -1_000_000 - depth as i32,
        };
    }
    if depth == 0 || is_full(grid) {
        return evaluate(grid);
    }

    let (piece, mut best) = if maximising {
        (Piece::Cpu, i32::MIN)
    } else {
        (Piece::Human, i32::MAX)
    };
    for col in COL_ORDER {
        let mut next = *grid;
        if drop_piece(&mut next, col, piece).is_none() {
            continue;
        }
        let score = minimax(&next, depth - 1, alpha, beta, !maximising);
        if maximising {
            best = best.max(score);
            alpha = alpha.max(best);
        } else {
            best = best.min(score);
            beta = beta.min(best);
        }
        if beta <= alpha {
            break;
        }
    }
    best
}

// Picks the CPU's column with an alpha-beta minimax search.
pub fn best_move(grid: &Grid, depth: u32) -> usize {
    let mut best = (i32::MIN, COLS / 2);
    for col in COL_ORDER {
        let mut next = *grid;
        if drop_piece(&mut next, col, Piece::Cpu).is_none() {
            continue;
        }
        let score = minimax(&next, depth.saturating_sub(1), i32::MIN + 1, i32::MAX - 1, false);
        if score > best.0 {
            best = (score, col);
        }
    }
    best.1
}

fn settle_move(c4: &mut C4State, state: &mut GameState, mover: Piece) {
    if let Some(winner) = find_winner(&c4.grid) {
        c4.outcome = Some(match winner {
            Piece::Human => Outcome::HumanWin,
            Piece::Cpu => Outcome::CpuWin,
        });
        if winner == Piece::Human {
            state.score = 1.0;
        }
        state.game_over = true;
    } else if is_full(&c4.grid) {
        c4.outcome = Some(Outcome::Draw);
        state.game_over = true;
    } else {
        c4.turn = match mover {
            Piece::Human => Piece::Cpu,
            Piece::Cpu => Piece::Human,
        };
        c4.cpu_timer = CPU_DELAY;
    }
}

fn board_offset(screen_w: f32, screen_h: f32) -> (f32, f32) {
    ((screen_w - BOARD_W) / 2.0, (screen_h - BOARD_H) / 2.0)
}

pub fn column_at(x: f32, screen_w: f32) -> Option<usize> {
    let ox = (screen_w - BOARD_W) / 2.0;
    let c = ((x - ox - GAP) / (TILE + GAP)).floor();
    (c >= 0.0 && c < COLS as f32).then_some(c as usize)
}

fn human_system(world: &mut World, state: &mut GameState, input: &Input) {
    let Some(c4) = world.get_resource_mut::<C4State>() else {
        return;
    };
    if c4.turn != Piece::Human {
        return;
    }

    if let Some(col) = column_at(input.mouse_x, input.screen_width) {
        c4.hover_col = col;
    }
    if input.left && c4.hover_col > 0 {
        c4.hover_col -= 1;
    }
    if input.right && c4.hover_col < COLS - 1 {
        c4.hover_col += 1;
    }

    let wants_drop =
        input.spacebar || (input.mouse_pressed && column_at(input.mouse_x, input.screen_width).is_some());
    if wants_drop && drop_piece(&mut c4.grid, c4.hover_col, Piece::Human).is_some() {
        settle_move(c4, state, Piece::Human);
    }
}

fn cpu_system(world: &mut World, state: &mut GameState, input: &Input) {
    let Some(c4) = world.get_resource_mut::<C4State>() else {
        return;
    };
    if c4.turn != Piece::Cpu {
        return;
    }
    c4.cpu_timer -= input.dt;
    if c4.cpu_timer > 0.0 {
        return;
    }

    let col = best_move(&c4.grid, AI_DEPTH);
    drop_piece(&mut c4.grid, col, Piece::Cpu);
    settle_move(c4, state, Piece::Cpu);
}

fn piece_color(piece: Piece) -> Color {
    match piece {
        Piece::Human => HUMAN_COLOR,
        Piece::Cpu => CPU_COLOR,
    }
}

fn render_board(world: &World, state: &GameState) {
    let (ox, oy) = board_offset(screen_width(), screen_height());
    let Some(c4) = world.get_resource::<C4State>() else {
        return;
    };

    // Preview piece hovering above the chosen column.
    if !state.game_over && c4.turn == Piece::Human {
        let x = ox + GAP + c4.hover_col as f32 * (TILE + GAP) + TILE / 2.0;
        draw_circle(x, oy - TILE / 2.0, TILE * 0.42, HUMAN_COLOR);
    }

    draw_rectangle(ox, oy, BOARD_W, BOARD_H, BOARD_COLOR);
    draw_rectangle_lines(ox - 2.0, oy - 2.0, BOARD_W + 4.0, BOARD_H + 4.0, 4.0, BORDER_COLOR);

    for r in 0..ROWS {
        for c in 0..COLS {
            let x = ox + GAP + c as f32 * (TILE + GAP) + TILE / 2.0;
            let y = oy + GAP + r as f32 * (TILE + GAP) + TILE / 2.0;
            let color = c4.grid[r][c].map(piece_color).unwrap_or(HOLE_COLOR);
            draw_circle(x, y, TILE * 0.42, color);
        }
    }
}

fn ui_system(world: &World, state: &GameState) {
    let (ox, oy) = board_offset(screen_width(), screen_height());
    let Some(c4) = world.get_resource::<C4State>() else {
        return;
    };

    let label = match (state.game_over, c4.turn) {
        (true, _) => "",
        (false, Piece::Human) => "YOUR TURN",
        (false, Piece::Cpu) => "CPU IS THINKING...",
    };
    draw_text(label, ox, oy - TILE - 24.0, 32.0, WHITE);

    ui::draw_help_line(
        "CLICK a column or use LEFT/RIGHT + SPACE",
        Rect::new(ox, oy, BOARD_W, BOARD_H),
    );

    if state.game_over {
        let (text, color) = match c4.outcome {
            Some(Outcome::HumanWin) => ("YOU WIN!", GREEN),
            Some(Outcome::CpuWin) => ("CPU WINS!", RED),
            _ => ("IT'S A DRAW!", WHITE),
        };
        ui::draw_game_over_overlay(text, color);
    }
}

pub fn spawn_initial(world: &mut World) {
    world.insert_resource(C4State::new());
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        spawn_initial(&mut game.world);
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Connect 4")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);

    let mut game = Game::new(world)
        .with_update_systems(vec![human_system, cpu_system])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let (mouse_x, mouse_y) = mouse_position();
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            left: is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
            right: is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
            mouse_x,
            mouse_y,
            mouse_pressed: is_mouse_button_pressed(MouseButton::Left),
            mouse_down: is_mouse_button_down(MouseButton::Left),
            mouse_right_pressed: is_mouse_button_pressed(MouseButton::Right),
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
