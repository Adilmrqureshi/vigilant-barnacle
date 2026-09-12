mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const BOARD_W: f32 = 480.0;
const BOARD_H: f32 = 640.0;

const PLAYER_SIZE: f32 = 32.0;
const PLATFORM_W: f32 = 72.0;
const PLATFORM_H: f32 = 14.0;
const PLATFORM_COUNT: i32 = 8;
const PLATFORM_SPACING: f32 = 80.0;
const PLATFORM_SPAN: f32 = PLATFORM_COUNT as f32 * PLATFORM_SPACING;

const GRAVITY: f32 = 900.0;
const JUMP_VY: f32 = -560.0;
const MOVE_SPEED: f32 = 280.0;
const SCROLL_LINE: f32 = 240.0;

const BG_COLOR: Color = Color::new(0.05, 0.06, 0.10, 1.0);
const BOARD_COLOR: Color = Color::new(0.08, 0.09, 0.14, 1.0);
const BORDER_COLOR: Color = Color::new(0.40, 0.40, 0.60, 1.0);
const PLAYER_COLOR: Color = Color::new(0.95, 0.85, 0.30, 1.0);
const PLATFORM_COLOR: Color = Color::new(0.35, 0.75, 0.40, 1.0);

// Resource: how far the world has scrolled, i.e. how high we've climbed.
pub struct Climb {
    pub height: f32,
}

fn steer_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for player in world.with_tag_mut(Tag::Player) {
        if input.left {
            player.transform.x -= MOVE_SPEED * input.dt;
        }
        if input.right {
            player.transform.x += MOVE_SPEED * input.dt;
        }
        // Wrap around the side edges, doodle-jump style.
        if player.transform.x + player.transform.w < 0.0 {
            player.transform.x = BOARD_W;
        }
        if player.transform.x > BOARD_W {
            player.transform.x = -player.transform.w;
        }
    }
}

fn physics_system(world: &mut World, state: &mut GameState, input: &Input) {
    let platforms: Vec<Rect> = world.with_tag(Tag::Brick).map(|p| p.transform).collect();

    for player in world.with_tag_mut(Tag::Player) {
        let Some(physics) = &mut player.physics else {
            continue;
        };
        physics.velocity.y += GRAVITY * input.dt;
        let old_bottom = player.transform.y + player.transform.h;
        player.transform.y += physics.velocity.y * input.dt;
        let new_bottom = player.transform.y + player.transform.h;

        // Bounce when falling through the top of a platform this frame.
        if physics.velocity.y > 0.0 {
            for platform in &platforms {
                let overlaps_x = player.transform.x + player.transform.w > platform.x
                    && player.transform.x < platform.x + platform.w;
                let crossed_top = old_bottom <= platform.y && new_bottom >= platform.y;
                if overlaps_x && crossed_top {
                    player.transform.y = platform.y - player.transform.h;
                    physics.velocity.y = JUMP_VY;
                    break;
                }
            }
        }

        if player.transform.y > BOARD_H {
            state.game_over = true;
        }
    }
}

fn scroll_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let lift = {
        let Some(player) = world.with_tag(Tag::Player).next() else {
            return;
        };
        SCROLL_LINE - player.transform.y
    };
    if lift <= 0.0 {
        return;
    }

    for player in world.with_tag_mut(Tag::Player) {
        player.transform.y = SCROLL_LINE;
    }
    for platform in world.with_tag_mut(Tag::Brick) {
        platform.transform.y += lift;
        // Recycle platforms that scrolled off the bottom back to the top.
        if platform.transform.y > BOARD_H {
            platform.transform.y -= PLATFORM_SPAN;
            platform.transform.x = rand::gen_range(0.0, BOARD_W - PLATFORM_W);
        }
    }

    if let Some(climb) = world.get_resource_mut::<Climb>() {
        climb.height += lift;
        state.score = (climb.height / 10.0).floor();
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
        // Skip anything scrolled outside the board.
        if e.transform.y > BOARD_H || e.transform.y + e.transform.h < 0.0 {
            continue;
        }
        draw_rectangle(
            ox + e.transform.x,
            oy + e.transform.y,
            e.transform.w,
            e.transform.h,
            render.color,
        );
    }
}

fn ui_system(_world: &World, state: &GameState) {
    let (ox, oy) = board_offset();

    draw_text(&format!("HEIGHT {}", state.score as i32), ox, oy - 14.0, 32.0, WHITE);

    let help = "LEFT/RIGHT or A/D: steer - bounce up, don't fall";
    let dims = measure_text(help, None, 20, 1.0);
    draw_text(help, ox + (BOARD_W - dims.width) / 2.0, oy + BOARD_H + 26.0, 20.0, GRAY);

    if state.game_over {
        let text = "YOU FELL!";
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

pub fn spawn_initial(world: &mut World) {
    // A guaranteed platform under the player's starting position.
    let start_x = BOARD_W / 2.0 - PLATFORM_W / 2.0;
    for i in 0..PLATFORM_COUNT {
        let last = i == PLATFORM_COUNT - 1;
        world.entities.push(
            Entity::new(Rect {
                x: if last {
                    start_x
                } else {
                    rand::gen_range(0.0, BOARD_W - PLATFORM_W)
                },
                y: 40.0 + i as f32 * PLATFORM_SPACING,
                w: PLATFORM_W,
                h: PLATFORM_H,
            })
            .with_tag(Tag::Brick)
            .with_render(PLATFORM_COLOR),
        );
    }

    world.entities.push(
        Entity::new(Rect {
            x: BOARD_W / 2.0 - PLAYER_SIZE / 2.0,
            y: 40.0 + (PLATFORM_COUNT - 1) as f32 * PLATFORM_SPACING - PLAYER_SIZE,
            w: PLAYER_SIZE,
            h: PLAYER_SIZE,
        })
        .with_tag(Tag::Player)
        .with_render(PLAYER_COLOR)
        .with_physics(Physics::new()),
    );

    world.insert_resource(Climb { height: 0.0 });
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world);
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Sky Climber")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);

    let mut game = Game::new(world)
        .with_update_systems(vec![steer_system, physics_system, scroll_system])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            a: false,
            up: false,
            down: false,
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
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
