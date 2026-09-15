mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const CELL: f32 = 32.0;
const GRID_W: i32 = 20;
const GRID_H: i32 = 15;
const BOARD_W: f32 = GRID_W as f32 * CELL;
const BOARD_H: f32 = GRID_H as f32 * CELL;
const STEP_TIME: f32 = 0.12;

const BG_COLOR: Color = Color::new(0.05, 0.07, 0.05, 1.0);
const BOARD_COLOR: Color = Color::new(0.09, 0.12, 0.09, 1.0);
const BORDER_COLOR: Color = Color::new(0.35, 0.65, 0.35, 1.0);
const HEAD_COLOR: Color = Color::new(0.64, 0.90, 0.21, 1.0);
const BODY_COLOR: Color = Color::new(0.29, 0.68, 0.31, 1.0);
const FOOD_COLOR: Color = Color::new(0.94, 0.27, 0.27, 1.0);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn delta(self) -> (f32, f32) {
        match self {
            Dir::Up => (0.0, -CELL),
            Dir::Down => (0.0, CELL),
            Dir::Left => (-CELL, 0.0),
            Dir::Right => (CELL, 0.0),
        }
    }

    fn opposite(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

// Resource holding everything about the snake that isn't per-entity.
pub struct SnakeState {
    pub dir: Dir,
    pub next_dir: Dir,
    pub step_timer: f32,
    pub pending_growth: usize,
}

impl SnakeState {
    pub fn new() -> Self {
        Self {
            dir: Dir::Right,
            next_dir: Dir::Right,
            step_timer: 0.0,
            pending_growth: 0,
        }
    }
}

fn cell(x: i32, y: i32) -> Rect {
    Rect {
        x: x as f32 * CELL,
        y: y as f32 * CELL,
        w: CELL,
        h: CELL,
    }
}

// Grid positions are multiples of CELL, so half a cell of tolerance
// identifies "same cell" without relying on float equality.
fn same_cell(a: &Rect, b: &Rect) -> bool {
    (a.x - b.x).abs() < CELL / 2.0 && (a.y - b.y).abs() < CELL / 2.0
}

fn is_segment(e: &Entity) -> bool {
    e.tag == Some(Tag::Player) || e.tag == Some(Tag::Body)
}

fn direction_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let Some(snake) = world.get_resource_mut::<SnakeState>() else {
        return;
    };

    let wanted = if input.up {
        Some(Dir::Up)
    } else if input.down {
        Some(Dir::Down)
    } else if input.left {
        Some(Dir::Left)
    } else if input.right {
        Some(Dir::Right)
    } else {
        None
    };

    if let Some(dir) = wanted {
        if dir != snake.dir.opposite() {
            snake.next_dir = dir;
        }
    }
}

fn movement_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let dir = {
        let Some(snake) = world.get_resource_mut::<SnakeState>() else {
            return;
        };
        snake.step_timer += input.dt;
        if snake.step_timer < STEP_TIME {
            return;
        }
        snake.step_timer -= STEP_TIME;
        snake.dir = snake.next_dir;
        snake.dir
    };

    // Entity order is head first, then body oldest-to-newest, because new
    // segments are always pushed to the back of the entity list.
    let segments: Vec<usize> = world
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| is_segment(e))
        .map(|(i, _)| i)
        .collect();

    let (dx, dy) = dir.delta();
    let mut prev: Option<Rect> = None;
    for idx in segments {
        let e = &mut world.entities[idx];
        let old = e.transform;
        match prev {
            None => {
                e.transform.x += dx;
                e.transform.y += dy;
            }
            Some(p) => {
                e.transform.x = p.x;
                e.transform.y = p.y;
            }
        }
        prev = Some(old);
    }

    let grow = world
        .get_resource_mut::<SnakeState>()
        .map(|snake| {
            if snake.pending_growth > 0 {
                snake.pending_growth -= 1;
                true
            } else {
                false
            }
        })
        .unwrap_or(false);

    if grow {
        if let Some(tail) = prev {
            world.entities.push(
                Entity::new(tail)
                    .with_tag(Tag::Body)
                    .with_render(BODY_COLOR),
            );
        }
    }
}

fn food_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let Some(head) = world.with_tag(Tag::Player).next().map(|e| e.transform) else {
        return;
    };

    let eaten = world
        .with_tag(Tag::Food)
        .any(|f| same_cell(&f.transform, &head));
    if !eaten {
        return;
    }

    state.score += 1.0;
    if let Some(snake) = world.get_resource_mut::<SnakeState>() {
        snake.pending_growth += 1;
    }

    let occupied: Vec<Rect> = world
        .entities
        .iter()
        .filter(|e| is_segment(e))
        .map(|e| e.transform)
        .collect();

    let mut spot = cell(0, 0);
    for _ in 0..200 {
        spot = cell(rand::gen_range(0, GRID_W), rand::gen_range(0, GRID_H));
        if !occupied.iter().any(|o| same_cell(o, &spot)) {
            break;
        }
    }

    for food in world.with_tag_mut(Tag::Food) {
        food.transform.x = spot.x;
        food.transform.y = spot.y;
    }
}

fn snake_collision_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let Some(head) = world.with_tag(Tag::Player).next().map(|e| e.transform) else {
        return;
    };

    let out_of_bounds = head.x < -0.5
        || head.y < -0.5
        || head.x > BOARD_W - CELL + 0.5
        || head.y > BOARD_H - CELL + 0.5;

    let hits_body = world
        .with_tag(Tag::Body)
        .any(|b| same_cell(&b.transform, &head));

    if out_of_bounds || hits_body {
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
            ox + e.transform.x + 1.0,
            oy + e.transform.y + 1.0,
            e.transform.w - 2.0,
            e.transform.h - 2.0,
            render.color,
        );
    }
}

fn ui_system(_world: &World, state: &GameState) {
    let (ox, oy) = board_offset();

    draw_text(&format!("SCORE {}", state.score as i32), ox, oy - 14.0, 32.0, WHITE);

    ui::draw_help_line(
        "ARROWS or WASD: steer - eat food, avoid walls and yourself",
        Rect::new(ox, oy, BOARD_W, BOARD_H),
    );

    if state.game_over {
        ui::draw_game_over_overlay("GAME OVER!", RED);
    }
}

fn spawn_initial(world: &mut World) {
    world.entities.push(
        Entity::new(cell(GRID_W / 2, GRID_H / 2))
            .with_tag(Tag::Player)
            .with_render(HEAD_COLOR),
    );
    for i in 1..=2 {
        world.entities.push(
            Entity::new(cell(GRID_W / 2 - i, GRID_H / 2))
                .with_tag(Tag::Body)
                .with_render(BODY_COLOR),
        );
    }
    world.entities.push(
        Entity::new(cell(GRID_W / 2 + 5, GRID_H / 2))
            .with_tag(Tag::Food)
            .with_render(FOOD_COLOR),
    );
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world);
        if let Some(snake) = game.world.get_resource_mut::<SnakeState>() {
            *snake = SnakeState::new();
        }
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Snake")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);
    world.insert_resource(SnakeState::new());

    let mut game = Game::new(world)
        .with_update_systems(vec![
            direction_system,
            movement_system,
            food_system,
            snake_collision_system,
        ])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            a: is_key_pressed(KeyCode::A),
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
