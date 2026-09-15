mod tests;

use macroquad::prelude::*;
use shared_v2::*;

pub const COLS: usize = 10;
pub const ROWS: usize = 10;
pub const MINES: usize = 15;
const TILE: f32 = 44.0;
const GAP: f32 = 4.0;
const BOARD_W: f32 = COLS as f32 * TILE + (COLS + 1) as f32 * GAP;
const BOARD_H: f32 = ROWS as f32 * TILE + (ROWS + 1) as f32 * GAP;

const BG_COLOR: Color = Color::new(0.06, 0.06, 0.10, 1.0);
const BOARD_COLOR: Color = Color::new(0.10, 0.10, 0.16, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
const COVERED_COLOR: Color = Color::new(0.22, 0.24, 0.36, 1.0);
const HOVER_COLOR: Color = Color::new(0.31, 0.34, 0.50, 1.0);
const REVEALED_COLOR: Color = Color::new(0.13, 0.14, 0.20, 1.0);
const MINE_COLOR: Color = Color::new(0.90, 0.30, 0.28, 1.0);
const FLAG_COLOR: Color = Color::new(0.95, 0.65, 0.25, 1.0);
// Classic minesweeper number colors for counts 1..=8.
const NUMBER_COLORS: [Color; 8] = [
    Color::new(0.35, 0.55, 1.00, 1.0),
    Color::new(0.30, 0.80, 0.40, 1.0),
    Color::new(0.95, 0.40, 0.35, 1.0),
    Color::new(0.70, 0.50, 1.00, 1.0),
    Color::new(0.85, 0.55, 0.30, 1.0),
    Color::new(0.35, 0.80, 0.80, 1.0),
    Color::new(0.90, 0.90, 0.90, 1.0),
    Color::new(0.60, 0.60, 0.70, 1.0),
];

#[derive(Debug, Copy, Clone, Default)]
pub struct Cell {
    pub mine: bool,
    pub revealed: bool,
    pub flagged: bool,
}

pub type Cells = [[Cell; COLS]; ROWS];

// Resource: the whole game is this grid. Mines are placed on the first
// reveal so the first click is always safe.
pub struct MinesState {
    pub cells: Cells,
    pub mines_placed: bool,
    pub won: bool,
}

impl MinesState {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::default(); COLS]; ROWS],
            mines_placed: false,
            won: false,
        }
    }
}

pub fn neighbours(r: usize, c: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(8);
    for dr in -1i32..=1 {
        for dc in -1i32..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            let (nr, nc) = (r as i32 + dr, c as i32 + dc);
            if nr >= 0 && nr < ROWS as i32 && nc >= 0 && nc < COLS as i32 {
                out.push((nr as usize, nc as usize));
            }
        }
    }
    out
}

pub fn adjacent_mines(cells: &Cells, r: usize, c: usize) -> usize {
    neighbours(r, c)
        .into_iter()
        .filter(|&(nr, nc)| cells[nr][nc].mine)
        .count()
}

pub fn place_mines_at(state: &mut MinesState, mines: &[(usize, usize)]) {
    for &(r, c) in mines {
        state.cells[r][c].mine = true;
    }
    state.mines_placed = true;
}

// Random mine layout that keeps the first-clicked cell and its neighbours clear.
pub fn random_mines(safe_r: usize, safe_c: usize) -> Vec<(usize, usize)> {
    let safe = neighbours(safe_r, safe_c);
    let mut candidates: Vec<(usize, usize)> = (0..ROWS)
        .flat_map(|r| (0..COLS).map(move |c| (r, c)))
        .filter(|&(r, c)| (r, c) != (safe_r, safe_c) && !safe.contains(&(r, c)))
        .collect();
    for i in 0..MINES {
        let j = rand::gen_range(i as i32, candidates.len() as i32) as usize;
        candidates.swap(i, j);
    }
    candidates.truncate(MINES);
    candidates
}

// Reveals a cell, flooding out from zero-count cells. Returns true on a mine.
pub fn reveal(state: &mut MinesState, r: usize, c: usize) -> bool {
    if state.cells[r][c].revealed || state.cells[r][c].flagged {
        return false;
    }
    if state.cells[r][c].mine {
        state.cells[r][c].revealed = true;
        return true;
    }
    let mut stack = vec![(r, c)];
    while let Some((r, c)) = stack.pop() {
        if state.cells[r][c].revealed || state.cells[r][c].flagged {
            continue;
        }
        state.cells[r][c].revealed = true;
        if adjacent_mines(&state.cells, r, c) == 0 {
            for (nr, nc) in neighbours(r, c) {
                if !state.cells[nr][nc].revealed {
                    stack.push((nr, nc));
                }
            }
        }
    }
    false
}

pub fn toggle_flag(state: &mut MinesState, r: usize, c: usize) {
    let cell = &mut state.cells[r][c];
    if !cell.revealed {
        cell.flagged = !cell.flagged;
    }
}

pub fn revealed_count(cells: &Cells) -> usize {
    cells
        .iter()
        .flatten()
        .filter(|c| c.revealed && !c.mine)
        .count()
}

pub fn flag_count(cells: &Cells) -> usize {
    cells.iter().flatten().filter(|c| c.flagged).count()
}

pub fn all_safe_revealed(cells: &Cells) -> bool {
    cells.iter().flatten().all(|c| c.mine || c.revealed)
}

fn board_offset(screen_w: f32, screen_h: f32) -> (f32, f32) {
    ((screen_w - BOARD_W) / 2.0, (screen_h - BOARD_H) / 2.0)
}

fn cell_pos(r: usize, c: usize) -> (f32, f32) {
    (
        GAP + c as f32 * (TILE + GAP),
        GAP + r as f32 * (TILE + GAP),
    )
}

pub fn cell_at(x: f32, y: f32, screen_w: f32, screen_h: f32) -> Option<(usize, usize)> {
    let (ox, oy) = board_offset(screen_w, screen_h);
    let (x, y) = (x - ox - GAP, y - oy - GAP);
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let (c, r) = ((x / (TILE + GAP)) as usize, (y / (TILE + GAP)) as usize);
    // Reject clicks that land in the gap past a tile's edge.
    if r >= ROWS || c >= COLS || x - c as f32 * (TILE + GAP) > TILE || y - r as f32 * (TILE + GAP) > TILE {
        return None;
    }
    Some((r, c))
}

fn play_system(world: &mut World, state: &mut GameState, input: &Input) {
    let Some(ms) = world.get_resource_mut::<MinesState>() else {
        return;
    };
    let Some((r, c)) = cell_at(input.mouse_x, input.mouse_y, input.screen_width, input.screen_height)
    else {
        return;
    };

    if input.mouse_right_pressed || input.spacebar {
        toggle_flag(ms, r, c);
    }

    if input.mouse_pressed {
        if !ms.mines_placed {
            let mines = random_mines(r, c);
            place_mines_at(ms, &mines);
        }
        if reveal(ms, r, c) {
            // Hit a mine: show the rest of them.
            for cell in ms.cells.iter_mut().flatten() {
                if cell.mine {
                    cell.revealed = true;
                }
            }
            state.game_over = true;
        } else {
            state.score = revealed_count(&ms.cells) as f32;
            if all_safe_revealed(&ms.cells) {
                ms.won = true;
                state.game_over = true;
            }
        }
    }
}

fn draw_flag(x: f32, y: f32) {
    let (cx, cy) = (x + TILE / 2.0, y + TILE / 2.0);
    draw_line(cx - 2.0, cy - 10.0, cx - 2.0, cy + 11.0, 3.0, FLAG_COLOR);
    draw_triangle(
        vec2(cx - 2.0, cy - 10.0),
        vec2(cx - 2.0, cy - 1.0),
        vec2(cx + 11.0, cy - 5.5),
        FLAG_COLOR,
    );
}

fn render_board(world: &World, state: &GameState) {
    let (ox, oy) = board_offset(screen_width(), screen_height());
    draw_rectangle(ox, oy, BOARD_W, BOARD_H, BOARD_COLOR);
    draw_rectangle_lines(ox - 2.0, oy - 2.0, BOARD_W + 4.0, BOARD_H + 4.0, 4.0, BORDER_COLOR);

    let Some(ms) = world.get_resource::<MinesState>() else {
        return;
    };
    let (mx, my) = mouse_position();
    let hovered = if state.game_over {
        None
    } else {
        cell_at(mx, my, screen_width(), screen_height())
    };

    for r in 0..ROWS {
        for c in 0..COLS {
            let (x, y) = cell_pos(r, c);
            let (x, y) = (ox + x, oy + y);
            let cell = ms.cells[r][c];

            let color = if cell.revealed {
                REVEALED_COLOR
            } else if hovered == Some((r, c)) {
                HOVER_COLOR
            } else {
                COVERED_COLOR
            };
            draw_rectangle(x, y, TILE, TILE, color);

            if cell.revealed && cell.mine {
                draw_circle(x + TILE / 2.0, y + TILE / 2.0, TILE * 0.26, MINE_COLOR);
            } else if cell.revealed {
                let n = adjacent_mines(&ms.cells, r, c);
                if n > 0 {
                    let text = format!("{n}");
                    let dims = measure_text(&text, None, 28, 1.0);
                    draw_text(
                        &text,
                        x + (TILE - dims.width) / 2.0,
                        y + (TILE + dims.height) / 2.0,
                        28.0,
                        NUMBER_COLORS[n - 1],
                    );
                }
            } else if cell.flagged {
                draw_flag(x, y);
            }
        }
    }
}

fn ui_system(world: &World, state: &GameState) {
    let (ox, oy) = board_offset(screen_width(), screen_height());
    let Some(ms) = world.get_resource::<MinesState>() else {
        return;
    };

    let left = MINES as i32 - flag_count(&ms.cells) as i32;
    draw_text(&format!("MINES {left}"), ox, oy - 14.0, 32.0, WHITE);

    let label = "CLEARED";
    let text = format!("{label} {}", state.score as i32);
    let dims = measure_text(&text, None, 26, 1.0);
    draw_text(&text, ox + BOARD_W - dims.width, oy - 14.0, 26.0, GRAY);

    ui::draw_help_line(
        "LEFT CLICK reveal   RIGHT CLICK or SPACE flag",
        Rect::new(ox, oy, BOARD_W, BOARD_H),
    );

    if state.game_over {
        let (text, color) = if ms.won {
            ("YOU WIN!", GREEN)
        } else {
            ("GAME OVER!", RED)
        };
        ui::draw_game_over_overlay(text, color);
    }
}

pub fn spawn_initial(world: &mut World) {
    world.insert_resource(MinesState::new());
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        spawn_initial(&mut game.world);
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Minesweeper")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);

    let mut game = Game::new(world)
        .with_update_systems(vec![play_system])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let (mouse_x, mouse_y) = mouse_position();
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
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
