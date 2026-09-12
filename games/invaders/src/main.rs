mod tests;

use macroquad::prelude::*;
use shared_v2::*;

const BOARD_W: f32 = 640.0;
const BOARD_H: f32 = 480.0;
const EDGE_MARGIN: f32 = 16.0;

const SHIP_W: f32 = 48.0;
const SHIP_H: f32 = 16.0;
const SHIP_Y: f32 = BOARD_H - 40.0;
const SHIP_SPEED: f32 = 300.0;

const INVADER_COLS: i32 = 8;
const INVADER_ROWS: i32 = 4;
const INVADER_W: f32 = 32.0;
const INVADER_H: f32 = 24.0;
const INVADER_X_GAP: f32 = 56.0;
const INVADER_Y_GAP: f32 = 44.0;
const INVADER_TOP: f32 = 48.0;
const FLEET_SPEED: f32 = 40.0;
const FLEET_SPEEDUP_PER_KILL: f32 = 4.0;
const FLEET_DROP: f32 = 16.0;

const BULLET_W: f32 = 4.0;
const BULLET_H: f32 = 12.0;
const PLAYER_BULLET_SPEED: f32 = 380.0;
const ENEMY_BULLET_SPEED: f32 = 180.0;
const ENEMY_FIRE_INTERVAL: f32 = 1.2;

const BG_COLOR: Color = Color::new(0.04, 0.05, 0.09, 1.0);
const BOARD_COLOR: Color = Color::new(0.07, 0.08, 0.13, 1.0);
const BORDER_COLOR: Color = Color::new(0.35, 0.45, 0.60, 1.0);
const SHIP_COLOR: Color = Color::new(0.45, 0.95, 0.55, 1.0);
const INVADER_COLOR: Color = Color::new(0.85, 0.85, 0.95, 1.0);
const PLAYER_BULLET_COLOR: Color = Color::new(0.55, 0.95, 0.65, 1.0);
const ENEMY_BULLET_COLOR: Color = Color::new(0.95, 0.45, 0.45, 1.0);

// Resource: fleet direction, enemy fire cooldown, and the win flag.
pub struct Fleet {
    pub dir: f32,
    pub fire_timer: f32,
    pub kills: u32,
    pub won: bool,
}

impl Fleet {
    pub fn new() -> Self {
        Self {
            dir: 1.0,
            fire_timer: ENEMY_FIRE_INTERVAL,
            kills: 0,
            won: false,
        }
    }
}

fn spawn_bullet(world: &mut World, x: f32, y: f32, vy: f32, color: Color) {
    let mut bullet = Entity::new(Rect {
        x: x - BULLET_W / 2.0,
        y,
        w: BULLET_W,
        h: BULLET_H,
    })
    .with_tag(Tag::Bullet)
    .with_render(color)
    .with_physics(Physics::new());
    if let Some(physics) = &mut bullet.physics {
        physics.velocity.y = vy;
    }
    world.add(bullet);
}

fn bullet_vy(e: &Entity) -> f32 {
    e.physics.as_ref().map(|p| p.velocity.y).unwrap_or(0.0)
}

fn player_input_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for ship in world.with_tag_mut(Tag::Player) {
        if input.left {
            ship.transform.x -= SHIP_SPEED * input.dt;
        }
        if input.right {
            ship.transform.x += SHIP_SPEED * input.dt;
        }
        ship.transform.x = ship.transform.x.clamp(0.0, BOARD_W - ship.transform.w);
    }
}

fn player_shoot_system(world: &mut World, _state: &mut GameState, input: &Input) {
    if !input.spacebar {
        return;
    }

    // Classic rule: only one player bullet on screen at a time.
    let has_player_bullet = world
        .with_tag(Tag::Bullet)
        .any(|b| bullet_vy(b) < 0.0);
    if has_player_bullet {
        return;
    }

    let Some(ship) = world.with_tag(Tag::Player).next().map(|s| s.transform) else {
        return;
    };
    spawn_bullet(
        world,
        ship.x + ship.w / 2.0,
        ship.y - BULLET_H,
        -PLAYER_BULLET_SPEED,
        PLAYER_BULLET_COLOR,
    );
}

fn enemy_fire_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let fire = {
        let Some(fleet) = world.get_resource_mut::<Fleet>() else {
            return;
        };
        fleet.fire_timer -= input.dt;
        if fleet.fire_timer > 0.0 {
            return;
        }
        fleet.fire_timer = ENEMY_FIRE_INTERVAL;
        true
    };
    if !fire {
        return;
    }

    let shooters: Vec<Rect> = world.with_tag(Tag::Enemy).map(|e| e.transform).collect();
    if shooters.is_empty() {
        return;
    }
    let shooter = shooters[rand::gen_range(0, shooters.len() as i32) as usize];
    spawn_bullet(
        world,
        shooter.x + shooter.w / 2.0,
        shooter.y + shooter.h,
        ENEMY_BULLET_SPEED,
        ENEMY_BULLET_COLOR,
    );
}

fn fleet_system(world: &mut World, state: &mut GameState, input: &Input) {
    let (dir, speed) = {
        let Some(fleet) = world.get_resource::<Fleet>() else {
            return;
        };
        (
            fleet.dir,
            FLEET_SPEED + fleet.kills as f32 * FLEET_SPEEDUP_PER_KILL,
        )
    };

    let mut hit_edge = false;
    let mut reached_ship = false;
    for invader in world.with_tag_mut(Tag::Enemy) {
        invader.transform.x += dir * speed * input.dt;
        if (dir > 0.0 && invader.transform.x + invader.transform.w > BOARD_W - EDGE_MARGIN)
            || (dir < 0.0 && invader.transform.x < EDGE_MARGIN)
        {
            hit_edge = true;
        }
        if invader.transform.y + invader.transform.h >= SHIP_Y {
            reached_ship = true;
        }
    }

    if hit_edge {
        if let Some(fleet) = world.get_resource_mut::<Fleet>() {
            fleet.dir = -dir;
        }
        for invader in world.with_tag_mut(Tag::Enemy) {
            invader.transform.y += FLEET_DROP;
        }
    }

    if reached_ship {
        state.game_over = true;
    }
}

fn bullet_system(world: &mut World, _state: &mut GameState, input: &Input) {
    for bullet in world.with_tag_mut(Tag::Bullet) {
        let Some(physics) = &bullet.physics else {
            continue;
        };
        bullet.transform.y += physics.velocity.y * input.dt;
        if bullet.transform.y + bullet.transform.h < 0.0 || bullet.transform.y > BOARD_H {
            bullet.alive = false;
        }
    }
    world.despawn_dead();
}

// Events: detection announces what happened; the consumers below decide
// what it means. Anything else (sound, particles) can subscribe later
// without touching detection.
pub struct BulletHit {
    pub bullet: usize,
    pub invader: usize,
}
pub struct PlayerHit;

fn hit_detection_system(world: &mut World, _state: &mut GameState, _input: &Input) {
    let player_shots: Vec<(usize, Rect)> = world
        .with_tag(Tag::Bullet)
        .filter(|b| bullet_vy(b) < 0.0)
        .map(|b| (b.id, b.transform))
        .collect();
    let enemy_shots: Vec<Rect> = world
        .with_tag(Tag::Bullet)
        .filter(|b| bullet_vy(b) > 0.0)
        .map(|b| b.transform)
        .collect();

    let mut hits: Vec<BulletHit> = vec![];
    for invader in world.with_tag(Tag::Enemy) {
        let already_spent = |id: usize| hits.iter().any(|h| h.bullet == id);
        if let Some((bullet, _)) = player_shots
            .iter()
            .find(|(id, s)| !already_spent(*id) && s.overlaps(&invader.transform))
        {
            hits.push(BulletHit {
                bullet: *bullet,
                invader: invader.id,
            });
        }
    }
    for hit in hits {
        world.emit(hit);
    }

    let player_hit = world
        .with_tag(Tag::Player)
        .any(|ship| enemy_shots.iter().any(|s| s.overlaps(&ship.transform)));
    if player_hit {
        world.emit(PlayerHit);
    }
}

fn hit_scoring_system(world: &mut World, state: &mut GameState, _input: &Input) {
    let kills = world.events::<BulletHit>().len() as u32;
    if kills == 0 {
        return;
    }
    state.score += kills as f32;
    if let Some(fleet) = world.get_resource_mut::<Fleet>() {
        fleet.kills += kills;
    }
}

fn hit_despawn_system(world: &mut World, _state: &mut GameState, _input: &Input) {
    let casualties: Vec<usize> = world
        .events::<BulletHit>()
        .iter()
        .flat_map(|h| [h.bullet, h.invader])
        .collect();
    if casualties.is_empty() {
        return;
    }
    for id in casualties {
        if let Some(e) = world.find_mut(id) {
            e.alive = false;
        }
    }
    world.despawn_dead();
}

fn player_hit_system(world: &mut World, state: &mut GameState, _input: &Input) {
    if !world.events::<PlayerHit>().is_empty() {
        state.game_over = true;
    }
}

fn win_system(world: &mut World, state: &mut GameState, _input: &Input) {
    if world.with_tag(Tag::Enemy).next().is_none() {
        if let Some(fleet) = world.get_resource_mut::<Fleet>() {
            fleet.won = true;
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

    let help = "LEFT/RIGHT or A/D: move - SPACE: shoot - clear the fleet";
    let dims = measure_text(help, None, 20, 1.0);
    draw_text(help, ox + (BOARD_W - dims.width) / 2.0, oy + BOARD_H + 26.0, 20.0, GRAY);

    if state.game_over {
        let won = world.get_resource::<Fleet>().is_some_and(|f| f.won);
        let text = if won { "YOU WIN!" } else { "GAME OVER!" };
        let dims = measure_text(text, None, 50, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - dims.width / 2.0,
            screen_height() / 2.0 - dims.height / 2.0,
            60.0,
            if won { GREEN } else { RED },
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
    world.add(
        Entity::new(Rect {
            x: BOARD_W / 2.0 - SHIP_W / 2.0,
            y: SHIP_Y,
            w: SHIP_W,
            h: SHIP_H,
        })
        .with_tag(Tag::Player)
        .with_render(SHIP_COLOR),
    );

    let grid_w = (INVADER_COLS - 1) as f32 * INVADER_X_GAP + INVADER_W;
    let left = (BOARD_W - grid_w) / 2.0;
    for row in 0..INVADER_ROWS {
        for col in 0..INVADER_COLS {
            world.add(
                Entity::new(Rect {
                    x: left + col as f32 * INVADER_X_GAP,
                    y: INVADER_TOP + row as f32 * INVADER_Y_GAP,
                    w: INVADER_W,
                    h: INVADER_H,
                })
                .with_tag(Tag::Enemy)
                .with_render(INVADER_COLOR),
            );
        }
    }
}

fn restart_game(game: &mut Game, input: &Input) {
    if input.spacebar {
        game.world.entities.clear();
        spawn_initial(&mut game.world);
        game.world.insert_resource(Fleet::new());
        game.state.score = 0.0;
        game.state.game_over = false;
    }
}

#[macroquad::main("Invaders")]
async fn main() {
    let mut world = World::new();
    spawn_initial(&mut world);
    world.insert_resource(Fleet::new());

    let mut game = Game::new(world)
        .with_update_systems(vec![
            player_input_system,
            player_shoot_system,
            enemy_fire_system,
            fleet_system,
            bullet_system,
            hit_detection_system,
            hit_scoring_system,
            hit_despawn_system,
            player_hit_system,
            win_system,
        ])
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
