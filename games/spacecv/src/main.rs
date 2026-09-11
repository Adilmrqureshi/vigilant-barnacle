mod tests;

use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use shared_v2::*;

const BOARD_W: f32 = 800.0;
const BOARD_H: f32 = 480.0;

const SHIP_X: f32 = 160.0;
const SHIP_W: f32 = 48.0;
const SHIP_H: f32 = 72.0;
const SHIP_MIN_Y: f32 = 40.0;
const SHIP_MAX_Y: f32 = BOARD_H - SHIP_H - 24.0;
const STEER_SPEED: f32 = 220.0;

const CRUISE_SPEED: f32 = 120.0;
const BOOST_SPEED: f32 = 420.0;
const REVERSE_SPEED: f32 = 240.0;

const STATION_GAP: f32 = 900.0;
const FIRST_STATION: f32 = 700.0;
const ACTIVATION: f32 = 300.0;
const PLANET_Y: f32 = 96.0;
const PLANET_RADIUS: f32 = 34.0;

const BG_COLOR: Color = Color::new(0.03, 0.04, 0.08, 1.0);
const BOARD_COLOR: Color = Color::new(0.05, 0.06, 0.11, 1.0);
const BORDER_COLOR: Color = Color::new(0.35, 0.45, 0.60, 1.0);
const PANEL_COLOR: Color = Color::new(0.04, 0.05, 0.10, 0.88);
const ACCENT_COLOR: Color = Color::new(0.49, 0.90, 0.23, 1.0);
const STAR_COLORS: [Color; 3] = [
    Color::new(1.0, 1.0, 1.0, 0.9),
    Color::new(0.7, 0.8, 1.0, 0.6),
    Color::new(0.6, 0.6, 0.8, 0.35),
];
const PLANET_COLORS: [Color; 4] = [
    Color::new(0.85, 0.45, 0.35, 1.0),
    Color::new(0.40, 0.65, 0.90, 1.0),
    Color::new(0.80, 0.70, 0.35, 1.0),
    Color::new(0.55, 0.45, 0.85, 1.0),
];

pub struct Station {
    pub at: f32,
    pub title: &'static str,
    pub lines: &'static [&'static str],
}

const fn station(index: f32, title: &'static str, lines: &'static [&'static str]) -> Station {
    Station {
        at: FIRST_STATION + index * STATION_GAP,
        title,
        lines,
    }
}

pub const STATIONS: [Station; 8] = [
    station(0.0, "HELLO, TRAVELLER", &[
        "I'm Mamunur Qureshi - software engineer.",
        "This is my career, laid out among the stars.",
        "Hold RIGHT to fly on. UP/DOWN to steer.",
    ]),
    station(1.0, "LANGUAGES", &[
        "Python - TypeScript - JavaScript",
        "Rust - PHP - SQL",
    ]),
    station(2.0, "FRONT-END", &[
        "Next.js - React - React Native - Expo",
        "HTML5 / CSS3 - responsive design -",
        "accessibility",
    ]),
    station(3.0, "BACK-END & CLOUD", &[
        "Django - Laravel - AWS - Docker",
        "CI/CD with GitHub Actions",
    ]),
    station(4.0, "2020-2024 | EVALUCOM, LONDON", &[
        "Software engineer, alongside a part-time",
        "degree. Co-built CarePulse: a health and",
        "social care data app in Django, Node.js,",
        "TypeScript and Next.js. Lead developer on",
        "2 of 3 modules of an eProcurement app:",
        "CSV-driven questionnaires and evaluation",
        "engines. Led a cost-model analysis app.",
    ]),
    station(5.0, "2024-2025 | FREELANCE, CAIRO & REMOTE", &[
        "End-to-end web and mobile products for",
        "international clients, from database",
        "design to cloud infrastructure.",
        "Fluxor: crypto-bot-as-a-service app in",
        "React Native + Expo with subscriptions.",
        "DCO Youth Club: community site with Stripe",
        "recurring donations.",
    ]),
    station(6.0, "2025-NOW | CRONER-I, LONDON", &[
        "Decomposing a legacy Drupal monolith into",
        "Node.js microservices. Built AI search and",
        "overview on top of LLMs, plus RAG data",
        "pipelines in Python and Databricks. Lead",
        "developer on a flagship AI chatbot.",
        "AWS infrastructure, Docker, CI/CD - and",
        "the occasional cyber-attack fended off.",
    ]),
    station(7.0, "THE END", &[
        "Thanks for flying with me.",
        "The games on the homepage are all written",
        "in Rust on my own little engine.",
        "Go play them!",
    ]),
];

const JOURNEY_END: f32 = FIRST_STATION + 7.0 * STATION_GAP + 400.0;

// Resource: how far along the career route the ship has flown.
pub struct Journey {
    pub distance: f32,
}

pub fn active_station(distance: f32) -> Option<usize> {
    STATIONS
        .iter()
        .position(|s| (s.at - distance).abs() < ACTIVATION)
}

fn travel_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let speed = if input.right {
        BOOST_SPEED
    } else if input.left {
        -REVERSE_SPEED
    } else {
        CRUISE_SPEED
    };

    if let Some(journey) = world.get_resource_mut::<Journey>() {
        journey.distance = (journey.distance + speed * input.dt).clamp(0.0, JOURNEY_END);
    }

    for ship in world.with_tag_mut(Tag::Player) {
        if input.up {
            ship.transform.y -= STEER_SPEED * input.dt;
        }
        if input.down {
            ship.transform.y += STEER_SPEED * input.dt;
        }
        ship.transform.y = ship.transform.y.clamp(SHIP_MIN_Y, SHIP_MAX_Y);
    }
}

fn board_offset() -> (f32, f32) {
    (
        (screen_width() - BOARD_W) / 2.0,
        (screen_height() - BOARD_H) / 2.0,
    )
}

fn render_space(world: &World, _state: &GameState) {
    let (ox, oy) = board_offset();
    let distance = world.get_resource::<Journey>().map_or(0.0, |j| j.distance);

    draw_rectangle(ox, oy, BOARD_W, BOARD_H, BOARD_COLOR);
    draw_rectangle_lines(ox - 2.0, oy - 2.0, BOARD_W + 4.0, BOARD_H + 4.0, 4.0, BORDER_COLOR);

    // Three parallax star layers, generated deterministically from the index.
    for i in 0..90 {
        let layer = i % 3;
        let parallax = [1.0, 0.5, 0.25][layer];
        let x = ((i * 137) as f32 - distance * parallax).rem_euclid(BOARD_W);
        let y = ((i * 211) % 460) as f32 + 10.0;
        let size = [2.5, 2.0, 1.5][layer];
        draw_rectangle(ox + x, oy + y, size, size, STAR_COLORS[layer]);
    }

    // Station planets drift by as the ship travels.
    for (i, s) in STATIONS.iter().enumerate() {
        let x = SHIP_X + (s.at - distance);
        if x < -PLANET_RADIUS || x > BOARD_W + PLANET_RADIUS {
            continue;
        }
        let color = PLANET_COLORS[i % PLANET_COLORS.len()];
        draw_circle(ox + x, oy + PLANET_Y, PLANET_RADIUS, color);
        draw_circle_lines(ox + x, oy + PLANET_Y, PLANET_RADIUS + 8.0, 2.0, BORDER_COLOR);
    }

    // The ship, rotated to face its direction of travel.
    for ship in world.with_tag(Tag::Player) {
        let Some(sprite_index) = ship.current_sprite else {
            continue;
        };
        let Some(sprite) = world.sprites.get(sprite_index) else {
            continue;
        };
        let frame = sprite.sprite.frame();
        draw_texture_ex(
            &sprite.texture,
            ox + ship.transform.x,
            oy + ship.transform.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SHIP_W, SHIP_H)),
                source: Some(frame.source_rect),
                rotation: std::f32::consts::FRAC_PI_2,
                ..Default::default()
            },
        );
    }

    // CV panel for the station currently in range.
    if let Some(index) = active_station(distance) {
        let s = &STATIONS[index];
        let px = ox + 60.0;
        let py = oy + 180.0;
        let pw = BOARD_W - 120.0;
        let ph = BOARD_H - 220.0;
        draw_rectangle(px, py, pw, ph, PANEL_COLOR);
        draw_rectangle_lines(px, py, pw, ph, 3.0, ACCENT_COLOR);
        draw_text(s.title, px + 24.0, py + 44.0, 30.0, ACCENT_COLOR);
        for (i, line) in s.lines.iter().enumerate() {
            draw_text(line, px + 24.0, py + 80.0 + i as f32 * 26.0, 22.0, WHITE);
        }
    }
}

fn ui_system(world: &World, _state: &GameState) {
    let (ox, oy) = board_offset();
    let distance = world.get_resource::<Journey>().map_or(0.0, |j| j.distance);
    let pct = (distance / JOURNEY_END * 100.0).round() as i32;

    draw_text(&format!("JOURNEY {pct}%"), ox, oy - 14.0, 32.0, WHITE);

    let hint = "RIGHT: fly  LEFT: back  UP/DOWN: steer";
    let dims = measure_text(hint, None, 20, 1.0);
    draw_text(hint, ox + BOARD_W - dims.width, oy - 14.0, 20.0, GRAY);
}

pub fn spawn_initial(world: &mut World) {
    world.entities.push(
        Entity::new(Rect {
            x: SHIP_X,
            y: BOARD_H / 2.0,
            w: SHIP_W,
            h: SHIP_H,
        })
        .with_tag(Tag::Player)
        .with_sprite(0),
    );
    world.insert_resource(Journey { distance: 0.0 });
}

#[macroquad::main("Get to know me")]
async fn main() {
    set_pc_assets_folder("./assets");
    let texture: Texture2D = load_texture("ship.png").await.expect("Couldn't load file");
    texture.set_filter(FilterMode::Nearest);
    build_textures_atlas();

    let ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[Animation {
            name: "idle".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    let mut world = World::new().add_sprite(Sprite {
        texture,
        sprite: ship_sprite,
        tag: AnimationTag::Movement,
    });
    spawn_initial(&mut world);

    let mut game = Game::new(world)
        .with_update_systems(vec![travel_system])
        .with_render_systems(vec![render_space, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: false,
            a: false,
            up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
            down: is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
            screen_width: screen_width(),
            screen_height: screen_height(),
        };

        clear_background(BG_COLOR);
        game.update(&input);
        for sprite in &mut game.world.sprites {
            sprite.sprite.update();
        }
        game.render();
        next_frame().await;
    }
}
