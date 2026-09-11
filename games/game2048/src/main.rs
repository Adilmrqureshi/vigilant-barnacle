mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const SIZE: usize = 4;
const TILE: f32 = 110.0;
const GAP: f32 = 12.0;
const BOARD_W: f32 = SIZE as f32 * TILE + (SIZE + 1) as f32 * GAP;
const BOARD_H: f32 = BOARD_W;

const BG_COLOR: Color = Color::new(0.06, 0.06, 0.10, 1.0);
const BOARD_COLOR: Color = Color::new(0.10, 0.10, 0.16, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
const EMPTY_COLOR: Color = Color::new(0.14, 0.14, 0.21, 1.0);
// Tile colors by exponent: 2, 4, 8, ... 2048 and beyond.
const TILE_COLORS: [Color; 11] = [
    Color::new(0.55, 0.51, 0.47, 1.0),
    Color::new(0.60, 0.53, 0.42, 1.0),
    Color::new(0.85, 0.58, 0.35, 1.0),
    Color::new(0.88, 0.49, 0.30, 1.0),
    Color::new(0.90, 0.42, 0.31, 1.0),
    Color::new(0.90, 0.35, 0.24, 1.0),
    Color::new(0.85, 0.70, 0.35, 1.0),
    Color::new(0.85, 0.68, 0.28, 1.0),
    Color::new(0.85, 0.66, 0.22, 1.0),
    Color::new(0.85, 0.64, 0.16, 1.0),
    Color::new(0.49, 0.90, 0.23, 1.0),
];

pub type Cells = [[u32; SIZE]; SIZE];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

// Resource: the whole game is this grid.
pub struct Board2048 {
    pub cells: Cells,
}

// Slides one row towards index 0, merging equal neighbours once each.
pub fn slide_row(row: [u32; SIZE]) -> ([u32; SIZE], f32) {
    let mut out = [0u32; SIZE];
    let mut score = 0.0;
    let mut pos = 0;
    let mut last_merged = false;

    for v in row.into_iter().filter(|v| *v != 0) {
        if pos > 0 && out[pos - 1] == v && !last_merged {
            out[pos - 1] = v * 2;
            score += (v * 2) as f32;
            last_merged = true;
        } else {
            out[pos] = v;
            pos += 1;
            last_merged = false;
        }
    }
    (out, score)
}

fn row_of(cells: &Cells, dir: Dir, i: usize) -> [u32; SIZE] {
    let mut row = [0u32; SIZE];
    for j in 0..SIZE {
        row[j] = match dir {
            Dir::Left => cells[i][j],
            Dir::Right => cells[i][SIZE - 1 - j],
            Dir::Up => cells[j][i],
            Dir::Down => cells[SIZE - 1 - j][i],
        };
    }
    row
}

fn write_row(cells: &mut Cells, dir: Dir, i: usize, row: [u32; SIZE]) {
    for j in 0..SIZE {
        match dir {
            Dir::Left => cells[i][j] = row[j],
            Dir::Right => cells[i][SIZE - 1 - j] = row[j],
            Dir::Up => cells[j][i] = row[j],
            Dir::Down => cells[SIZE - 1 - j][i] = row[j],
        }
    }
}

// Applies a move to the grid. Returns (gained score, whether anything moved).
pub fn apply_move(cells: &mut Cells, dir: Dir) -> (f32, bool) {
    let before = *cells;
    let mut score = 0.0;
    for i in 0..SIZE {
        let (row, gained) = slide_row(row_of(cells, dir, i));
        write_row(cells, dir, i, row);
        score += gained;
    }
    (score, *cells != before)
}

pub fn spawn_tile(cells: &mut Cells) {
    let empty: Vec<(usize, usize)> = (0..SIZE)
        .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
        .filter(|&(r, c)| cells[r][c] == 0)
        .collect();
    if empty.is_empty() {
        return;
    }
    let (r, c) = empty[rand::gen_range(0, empty.len() as i32) as usize];
    cells[r][c] = if rand::gen_range(0, 10) == 0 { 4 } else { 2 };
}

pub fn has_moves(cells: &Cells) -> bool {
    for r in 0..SIZE {
        for c in 0..SIZE {
            if cells[r][c] == 0 {
                return true;
            }
            if c + 1 < SIZE && cells[r][c] == cells[r][c + 1] {
                return true;
            }
            if r + 1 < SIZE && cells[r][c] == cells[r + 1][c] {
                return true;
            }
        }
    }
    false
}

pub fn pressed_dir(input: &Input) -> Option<Dir> {
    if input.up {
        Some(Dir::Up)
    } else if input.down {
        Some(Dir::Down)
    } else if input.left {
        Some(Dir::Left)
    } else if input.right {
        Some(Dir::Right)
    } else {
        None
    }
}

fn move_system(world: &mut World, state: &mut GameState, input: &Input) {
    let Some(dir) = pressed_dir(input) else {
        return;
    };
    let Some(board) = world.get_resource_mut::<Board2048>() else {
        return;
    };

    let (gained, moved) = apply_move(&mut board.cells, dir);
    if !moved {
        return;
    }
    state.score += gained;
    spawn_tile(&mut board.cells);
    if !has_moves(&board.cells) {
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

    let Some(board) = world.get_resource::<Board2048>() else {
        return;
    };

    for r in 0..SIZE {
        for c in 0..SIZE {
            let x = ox + GAP + c as f32 * (TILE + GAP);
            let y = oy + GAP + r as f32 * (TILE + GAP);
            let v = board.cells[r][c];
            let color = if v == 0 {
                EMPTY_COLOR
            } else {
                let exp = (v.ilog2() as usize - 1).min(TILE_COLORS.len() - 1);
                TILE_COLORS[exp]
            };
            draw_rectangle(x, y, TILE, TILE, color);
            if v > 0 {
                let label = v.to_string();
                let font_size = if v < 1000 { 44 } else { 34 };
                let dims = measure_text(&label, None, font_size, 1.0);
                draw_text(
                    &label,
                    x + TILE / 2.0 - dims.width / 2.0,
                    y + TILE / 2.0 + dims.height / 2.0,
                    font_size as f32,
                    WHITE,
                );
            }
        }
    }
}

fn ui_system(_world: &World, state: &GameState) {
    let (ox, oy) = board_offset();

    draw_text(&format!("SCORE {}", state.score as i32), ox, oy - 14.0, 32.0, WHITE);

    if state.game_over {
        let text = "GAME OVER!";
        let dims = measure_text(text, None, 50, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - dims.width / 2.0,
            screen_height() / 2.0 - dims.height / 2.0,
            60.0,
            RED,
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

pub fn new_board() -> Board2048 {
    let mut cells = [[0u32; SIZE]; SIZE];
    spawn_tile(&mut cells);
    spawn_tile(&mut cells);
    Board2048 { cells }
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.insert_resource(new_board());
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("2048")]
async fn main() {
    let mut world = World::new();
    world.insert_resource(new_board());

    let mut game = Game::new(world)
        .with_update_systems(vec![move_system])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            a: false,
            up: is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W),
            down: is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S),
            left: is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A),
            right: is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D),
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
