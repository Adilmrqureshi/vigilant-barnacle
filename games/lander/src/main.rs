mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const BOARD_W: f32 = 640.0;
const BOARD_H: f32 = 480.0;
const GROUND_Y: f32 = BOARD_H - 40.0;

const LANDER_W: f32 = 28.0;
const LANDER_H: f32 = 34.0;
const PAD_W: f32 = 110.0;

const GRAVITY: f32 = 50.0;
const MAIN_THRUST: f32 = 110.0;
const SIDE_THRUST: f32 = 70.0;
const START_FUEL: f32 = 100.0;
const MAIN_BURN: f32 = 22.0;
const SIDE_BURN: f32 = 8.0;
const MAX_LAND_VY: f32 = 60.0;
const MAX_LAND_VX: f32 = 40.0;

const BG_COLOR: Color = Color::new(0.03, 0.04, 0.08, 1.0);
const BOARD_COLOR: Color = Color::new(0.05, 0.06, 0.11, 1.0);
const BORDER_COLOR: Color = Color::new(0.35, 0.45, 0.60, 1.0);
const GROUND_COLOR: Color = Color::new(0.22, 0.22, 0.28, 1.0);
const PAD_COLOR: Color = Color::new(0.49, 0.90, 0.23, 1.0);
const LANDER_COLOR: Color = Color::new(0.90, 0.90, 0.95, 1.0);
const FLAME_COLOR: Color = Color::new(0.95, 0.60, 0.20, 1.0);

// Resource: fuel, where the pad is this round, and how the flight ended.
pub struct LanderState {
    pub fuel: f32,
    pub pad_x: f32,
    pub thrusting: bool,
    pub won: bool,
}

impl LanderState {
    pub fn new(pad_x: f32) -> Self {
        Self {
            fuel: START_FUEL,
            pad_x,
            thrusting: false,
            won: false,
        }
    }
}

pub fn random_pad_x() -> f32 {
    rand::gen_range(30.0, BOARD_W - PAD_W - 30.0)
}

fn control_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let (has_fuel, burn) = {
        let Some(lander) = world.get_resource::<LanderState>() else {
            return;
        };
        let mut burn = 0.0;
        if lander.fuel > 0.0 {
            if input.up {
                burn += MAIN_BURN;
            }
            if input.left || input.right {
                burn += SIDE_BURN;
            }
        }
        (lander.fuel > 0.0, burn * input.dt)
    };

    for ship in world.with_tag_mut(Tag::Player) {
        let Some(physics) = &mut ship.physics else {
            continue;
        };
        if has_fuel {
            if input.up {
                physics.velocity.y -= MAIN_THRUST * input.dt;
            }
            if input.left {
                physics.velocity.x -= SIDE_THRUST * input.dt;
            }
            if input.right {
                physics.velocity.x += SIDE_THRUST * input.dt;
            }
        }
    }

    if let Some(lander) = world.get_resource_mut::<LanderState>() {
        lander.fuel = (lander.fuel - burn).max(0.0);
        lander.thrusting = has_fuel && input.up;
    }
}

fn physics_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for ship in world.with_tag_mut(Tag::Player) {
        let Some(physics) = &mut ship.physics else {
            continue;
        };
        physics.velocity.y += GRAVITY * input.dt;
        ship.transform.x += physics.velocity.x * input.dt;
        ship.transform.y += physics.velocity.y * input.dt;

        // Side walls and ceiling stop the ship dead.
        if ship.transform.x < 0.0 || ship.transform.x + ship.transform.w > BOARD_W {
            ship.transform.x = ship.transform.x.clamp(0.0, BOARD_W - ship.transform.w);
            physics.velocity.x = 0.0;
        }
        if ship.transform.y < 0.0 {
            ship.transform.y = 0.0;
            physics.velocity.y = 0.0;
        }
    }
}

fn landing_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let touchdown = {
        let Some(ship) = world.with_tag(Tag::Player).next() else {
            return;
        };
        if ship.transform.y + ship.transform.h < GROUND_Y {
            return;
        }
        let physics = ship.physics.as_ref();
        let vy = physics.map_or(0.0, |p| p.velocity.y);
        let vx = physics.map_or(0.0, |p| p.velocity.x);
        (ship.transform.x, vx, vy)
    };
    let (ship_x, vx, vy) = touchdown;

    let (fuel, on_pad) = {
        let Some(lander) = world.get_resource::<LanderState>() else {
            return;
        };
        let on_pad =
            ship_x >= lander.pad_x && ship_x + LANDER_W <= lander.pad_x + PAD_W;
        (lander.fuel, on_pad)
    };

    let soft = vy <= MAX_LAND_VY && vx.abs() <= MAX_LAND_VX;
    let won = on_pad && soft;

    for ship in world.with_tag_mut(Tag::Player) {
        ship.transform.y = GROUND_Y - ship.transform.h;
        if let Some(physics) = &mut ship.physics {
            physics.velocity.x = 0.0;
            physics.velocity.y = 0.0;
        }
    }
    if let Some(lander) = world.get_resource_mut::<LanderState>() {
        lander.won = won;
    }
    state.game_over = true;
    if won {
        state.score = fuel;
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

    // A static starfield, deterministic per index.
    for i in 0..50 {
        let x = ((i * 137) % 630) as f32 + 5.0;
        let y = ((i * 211) % 380) as f32 + 10.0;
        draw_rectangle(ox + x, oy + y, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.6));
    }

    draw_rectangle(ox, oy + GROUND_Y, BOARD_W, BOARD_H - GROUND_Y, GROUND_COLOR);

    let Some(lander) = world.get_resource::<LanderState>() else {
        return;
    };
    draw_rectangle(ox + lander.pad_x, oy + GROUND_Y - 6.0, PAD_W, 6.0, PAD_COLOR);

    for ship in world.with_tag(Tag::Player) {
        let t = ship.transform;
        draw_rectangle(ox + t.x, oy + t.y, t.w, t.h - 8.0, LANDER_COLOR);
        // Legs.
        draw_rectangle(ox + t.x, oy + t.y + t.h - 8.0, 5.0, 8.0, LANDER_COLOR);
        draw_rectangle(ox + t.x + t.w - 5.0, oy + t.y + t.h - 8.0, 5.0, 8.0, LANDER_COLOR);
        if lander.thrusting {
            draw_rectangle(
                ox + t.x + t.w / 2.0 - 4.0,
                oy + t.y + t.h,
                8.0,
                12.0,
                FLAME_COLOR,
            );
        }
    }
}

fn ui_system(world: &World, state: &GameState) {
    let (ox, oy) = board_offset();

    let Some(lander) = world.get_resource::<LanderState>() else {
        return;
    };
    draw_text(&format!("FUEL {:.0}", lander.fuel), ox, oy - 14.0, 32.0, WHITE);

    // Velocity readout, green when landing would be soft.
    if let Some(ship) = world.with_tag(Tag::Player).next() {
        if let Some(physics) = &ship.physics {
            let vy = physics.velocity.y;
            let color = if vy <= MAX_LAND_VY && physics.velocity.x.abs() <= MAX_LAND_VX {
                GREEN
            } else {
                RED
            };
            let label = format!("SPEED {:.0}", vy.max(0.0));
            let dims = measure_text(&label, None, 32, 1.0);
            draw_text(&label, ox + BOARD_W - dims.width, oy - 14.0, 32.0, color);
        }
    }

    ui::draw_help_line(
        "UP: thrust - LEFT/RIGHT: steer - land softly on the green pad",
        Rect::new(ox, oy, BOARD_W, BOARD_H),
    );

    if state.game_over {
        // Long title, so smaller than the standard 60pt overlay.
        let text = if lander.won { "THE EAGLE HAS LANDED!" } else { "CRASHED!" };
        ui::draw_game_over_title(text, 44, if lander.won { GREEN } else { RED });
        ui::draw_restart_hint("press SPACE to fly again");
    }
}

pub fn spawn_initial(world: &mut World, pad_x: f32) {
    let mut ship = Entity::new(Rect {
        x: BOARD_W / 2.0 - LANDER_W / 2.0,
        y: 40.0,
        w: LANDER_W,
        h: LANDER_H,
    })
    .with_tag(Tag::Player)
    .with_render(LANDER_COLOR)
    .with_physics(Physics::new());
    if let Some(physics) = &mut ship.physics {
        physics.velocity.x = rand::gen_range(-30.0, 30.0);
    }
    world.entities.push(ship);
    world.insert_resource(LanderState::new(pad_x));
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world, random_pad_x());
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Lunar Lander")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world, random_pad_x());

    let mut game = Game::new(world)
        .with_update_systems(vec![control_system, physics_system, landing_system])
        .with_render_systems(vec![render_board, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
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
