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
const EXPLORE_SPEED: f32 = 260.0;

const STATION_GAP: f32 = 900.0;
const FIRST_STATION: f32 = 700.0;
const ACTIVATION: f32 = 300.0;
const PLANET_Y: f32 = 96.0;
const PLANET_RADIUS: f32 = 34.0;
// The 64px generated sprites are drawn at 2x so their 17px planet radius
// lands exactly on PLANET_RADIUS.
const PLANET_DRAW: f32 = 128.0;
const POI_RANGE: f32 = 90.0;

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

// A point of interest on a planet surface: fly near it to read its story.
pub struct Poi {
    pub x: f32,
    pub y: f32,
    pub title: &'static str,
    pub lines: &'static [&'static str],
}

const fn poi(x: f32, y: f32, title: &'static str, lines: &'static [&'static str]) -> Poi {
    Poi { x, y, title, lines }
}

pub struct Station {
    pub at: f32,
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub pois: &'static [Poi],
}

const fn station(
    index: f32,
    title: &'static str,
    lines: &'static [&'static str],
    pois: &'static [Poi],
) -> Station {
    Station {
        at: FIRST_STATION + index * STATION_GAP,
        title,
        lines,
        pois,
    }
}

pub const STATIONS: [Station; 8] = [
    station(0.0, "HELLO, TRAVELLER", &[
        "I'm Mamunur Qureshi - software engineer.",
        "This is my career, laid out among the stars.",
        "Hold RIGHT to fly on. UP/DOWN to steer.",
    ], &[
        poi(400.0, 140.0, "THE SHIP", &[
            "A little Rust spaceship, 16 pixels wide.",
            "ARROWS or WASD to fly around down here.",
        ]),
        poi(160.0, 280.0, "THE MISSION", &[
            "Every planet on the route is a chapter",
            "of my CV. Fly close to a beacon to read",
            "it. Press E to head back to space.",
        ]),
        poi(640.0, 200.0, "THE ENGINE", &[
            "This site's games - including this one -",
            "run on a tiny ECS engine I wrote in Rust.",
        ]),
    ]),
    station(1.0, "LANGUAGES", &[
        "Python - TypeScript - JavaScript",
        "Rust - PHP - SQL",
    ], &[
        poi(150.0, 130.0, "DAILY DRIVERS", &[
            "Python and TypeScript do most of the",
            "heavy lifting day to day.",
        ]),
        poi(430.0, 290.0, "SYSTEMS SIDE", &[
            "Rust - including this game and the",
            "engine underneath it.",
        ]),
        poi(650.0, 150.0, "ALSO ON BOARD", &[
            "JavaScript, PHP and SQL.",
        ]),
    ]),
    station(2.0, "FRONT-END", &[
        "Next.js - React - React Native - Expo",
        "HTML5 / CSS3 - responsive design -",
        "accessibility",
    ], &[
        poi(170.0, 150.0, "FRAMEWORKS", &[
            "Next.js and React for the web.",
        ]),
        poi(420.0, 300.0, "MOBILE", &[
            "React Native and Expo.",
        ]),
        poi(640.0, 170.0, "CRAFT", &[
            "HTML5 / CSS3, responsive design,",
            "accessibility.",
        ]),
    ]),
    station(3.0, "BACK-END & CLOUD", &[
        "Django - Laravel - AWS - Docker",
        "CI/CD with GitHub Actions",
    ], &[
        poi(160.0, 140.0, "BACK-END", &[
            "Django and Laravel.",
        ]),
        poi(430.0, 280.0, "CLOUD", &[
            "AWS and Docker.",
        ]),
        poi(650.0, 160.0, "DELIVERY", &[
            "CI/CD pipelines with GitHub Actions.",
        ]),
    ]),
    station(4.0, "2020-2024 | EVALUCOM, LONDON", &[
        "Software engineer, alongside a part-time",
        "degree. Land to explore the projects.",
    ], &[
        poi(150.0, 130.0, "CAREPULSE", &[
            "Co-built a health and social care data",
            "app in Django, Node.js, TypeScript and",
            "Next.js.",
        ]),
        poi(420.0, 290.0, "EPROCUREMENT", &[
            "Lead developer on 2 of 3 modules:",
            "CSV-driven questionnaires and",
            "evaluation engines.",
        ]),
        poi(650.0, 150.0, "COST MODELS", &[
            "Led a cost-model analysis app -",
            "alongside a part-time degree.",
        ]),
    ]),
    station(5.0, "2024-2025 | FREELANCE, CAIRO & REMOTE", &[
        "End-to-end web and mobile products for",
        "international clients. Land to explore.",
    ], &[
        poi(160.0, 140.0, "FLUXOR", &[
            "Crypto-bot-as-a-service app in React",
            "Native + Expo, with subscriptions.",
        ]),
        poi(430.0, 300.0, "DCO YOUTH CLUB", &[
            "Community site with Stripe recurring",
            "donations.",
        ]),
        poi(650.0, 160.0, "END TO END", &[
            "Database design to cloud infrastructure",
            "for international clients.",
        ]),
    ]),
    station(6.0, "2025-NOW | CRONER-I, LONDON", &[
        "Decomposing a legacy monolith, building",
        "AI products. Land to explore the work.",
    ], &[
        poi(150.0, 140.0, "MICROSERVICES", &[
            "Decomposing a legacy Drupal monolith",
            "into Node.js microservices.",
        ]),
        poi(420.0, 290.0, "AI & RAG", &[
            "AI search and overviews on top of LLMs,",
            "RAG data pipelines in Python and",
            "Databricks. Lead developer on a",
            "flagship AI chatbot.",
        ]),
        poi(650.0, 150.0, "OPERATIONS", &[
            "AWS infrastructure, Docker, CI/CD - and",
            "the occasional cyber-attack fended off.",
        ]),
    ]),
    station(7.0, "THE END", &[
        "Thanks for flying with me.",
        "The games on the homepage are all written",
        "in Rust on my own little engine.",
        "Go play them!",
    ], &[
        poi(280.0, 160.0, "THANKS", &[
            "Thanks for flying with me.",
        ]),
        poi(560.0, 260.0, "THE GAMES", &[
            "Everything on the homepage is Rust on",
            "my own little engine. Go play!",
        ]),
    ]),
];

const JOURNEY_END: f32 = FIRST_STATION + 7.0 * STATION_GAP + 400.0;

// Resource: how far along the career route the ship has flown.
pub struct Journey {
    pub distance: f32,
}

// Resource: whether we're flying the route or exploring a planet surface.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Scene {
    Space,
    Planet(usize),
}

// Resource: the generated planet sprites and surface backdrops. Absent in
// tests, so rendering falls back to flat shapes when it's missing.
pub struct Art {
    pub planets: Vec<Texture2D>,
    pub surfaces: Vec<Texture2D>,
}

pub fn scene(world: &World) -> Scene {
    world
        .get_resource::<Scene>()
        .copied()
        .unwrap_or(Scene::Space)
}

pub fn active_station(distance: f32) -> Option<usize> {
    STATIONS
        .iter()
        .position(|s| (s.at - distance).abs() < ACTIVATION)
}

pub fn active_poi(station: usize, x: f32, y: f32) -> Option<usize> {
    STATIONS[station]
        .pois
        .iter()
        .position(|p| ((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt() < POI_RANGE)
}

fn ship_center(world: &World) -> Option<(f32, f32)> {
    world
        .with_tag(Tag::Player)
        .next()
        .map(|s| (s.transform.x + SHIP_W / 2.0, s.transform.y + SHIP_H / 2.0))
}

// SPACE lands on the station in range; E (mapped onto Input::a) lifts off.
fn scene_system(world: &mut World, _state: &mut GameState, input: &Input) {
    match scene(world) {
        Scene::Space => {
            if !input.spacebar {
                return;
            }
            let distance = world.get_resource::<Journey>().map_or(0.0, |j| j.distance);
            let Some(index) = active_station(distance) else {
                return;
            };
            if let Some(s) = world.get_resource_mut::<Scene>() {
                *s = Scene::Planet(index);
            }
            for ship in world.with_tag_mut(Tag::Player) {
                ship.transform.x = (BOARD_W - SHIP_W) / 2.0;
                ship.transform.y = SHIP_MAX_Y - 40.0;
            }
        }
        Scene::Planet(_) => {
            if !input.a {
                return;
            }
            if let Some(s) = world.get_resource_mut::<Scene>() {
                *s = Scene::Space;
            }
            for ship in world.with_tag_mut(Tag::Player) {
                ship.transform.x = SHIP_X;
                ship.transform.y = BOARD_H / 2.0;
            }
        }
    }
}

fn travel_system(world: &mut World, _state: &mut GameState, input: &Input) {
    if scene(world) != Scene::Space {
        return;
    }

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

// Free flight over the planet surface.
fn explore_system(world: &mut World, _state: &mut GameState, input: &Input) {
    let Scene::Planet(_) = scene(world) else {
        return;
    };

    for ship in world.with_tag_mut(Tag::Player) {
        if input.up {
            ship.transform.y -= EXPLORE_SPEED * input.dt;
        }
        if input.down {
            ship.transform.y += EXPLORE_SPEED * input.dt;
        }
        if input.left {
            ship.transform.x -= EXPLORE_SPEED * input.dt;
        }
        if input.right {
            ship.transform.x += EXPLORE_SPEED * input.dt;
        }
        ship.transform.x = ship.transform.x.clamp(0.0, BOARD_W - SHIP_W);
        ship.transform.y = ship.transform.y.clamp(SHIP_MIN_Y, SHIP_MAX_Y);
    }
}

fn board_offset() -> (f32, f32) {
    (
        (screen_width() - BOARD_W) / 2.0,
        (screen_height() - BOARD_H) / 2.0,
    )
}

fn draw_ship(world: &World, ox: f32, oy: f32) {
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
}

fn draw_panel(ox: f32, oy: f32, py: f32, title: &str, lines: &[&str]) {
    let px = ox + 60.0;
    let pw = BOARD_W - 120.0;
    let ph = 64.0 + lines.len() as f32 * 26.0;
    draw_rectangle(px, oy + py, pw, ph, PANEL_COLOR);
    draw_rectangle_lines(px, oy + py, pw, ph, 3.0, ACCENT_COLOR);
    draw_text(title, px + 24.0, oy + py + 40.0, 30.0, ACCENT_COLOR);
    for (i, line) in lines.iter().enumerate() {
        draw_text(line, px + 24.0, oy + py + 74.0 + i as f32 * 26.0, 22.0, WHITE);
    }
}

fn render_space(world: &World, _state: &GameState) {
    if scene(world) != Scene::Space {
        return;
    }
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
    let active = active_station(distance);
    for (i, s) in STATIONS.iter().enumerate() {
        let x = SHIP_X + (s.at - distance);
        if x < -PLANET_DRAW / 2.0 || x > BOARD_W + PLANET_DRAW / 2.0 {
            continue;
        }
        if let Some(art) = world.get_resource::<Art>() {
            draw_texture_ex(
                &art.planets[i % art.planets.len()],
                ox + x - PLANET_DRAW / 2.0,
                oy + PLANET_Y - PLANET_DRAW / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(PLANET_DRAW, PLANET_DRAW)),
                    ..Default::default()
                },
            );
        } else {
            draw_circle(ox + x, oy + PLANET_Y, PLANET_RADIUS, PLANET_COLORS[i % 4]);
        }
        // A pulsing ring marks the planet you can land on.
        if active == Some(i) {
            let pulse = ((get_time() as f32 * 3.0).sin() + 1.0) * 3.0;
            draw_circle_lines(ox + x, oy + PLANET_Y, PLANET_RADIUS + 10.0 + pulse, 2.0, ACCENT_COLOR);
        }
    }

    draw_ship(world, ox, oy);

    // CV panel for the station currently in range.
    if let Some(index) = active {
        let s = &STATIONS[index];
        let mut lines: Vec<&str> = s.lines.to_vec();
        lines.push("");
        lines.push(">> SPACE: land and explore");
        draw_panel(ox, oy, 180.0, s.title, &lines);
    }
}

fn render_planet(world: &World, _state: &GameState) {
    let Scene::Planet(index) = scene(world) else {
        return;
    };
    let (ox, oy) = board_offset();
    let s = &STATIONS[index];

    if let Some(art) = world.get_resource::<Art>() {
        draw_texture_ex(
            &art.surfaces[index % art.surfaces.len()],
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(BOARD_W, BOARD_H)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(ox, oy, BOARD_W, BOARD_H, BOARD_COLOR);
    }
    draw_rectangle_lines(ox - 2.0, oy - 2.0, BOARD_W + 4.0, BOARD_H + 4.0, 4.0, BORDER_COLOR);

    // Pulsing beacons mark the points of interest.
    let t = get_time() as f32;
    for (i, p) in s.pois.iter().enumerate() {
        let pulse = ((t * 2.4 + i as f32 * 1.7).sin() + 1.0) * 4.0;
        draw_circle(ox + p.x, oy + p.y, 6.0, ACCENT_COLOR);
        draw_circle_lines(ox + p.x, oy + p.y, 12.0 + pulse, 2.0, ACCENT_COLOR);
    }

    draw_ship(world, ox, oy);

    // Fly near a beacon and its story appears at the bottom of the board.
    if let Some((sx, sy)) = ship_center(world) {
        if let Some(pi) = active_poi(index, sx, sy) {
            let p = &s.pois[pi];
            let ph = 64.0 + p.lines.len() as f32 * 26.0;
            draw_panel(ox, oy, BOARD_H - ph - 16.0, p.title, p.lines);
        }
    }
}

fn ui_system(world: &World, _state: &GameState) {
    let (ox, oy) = board_offset();

    match scene(world) {
        Scene::Space => {
            let distance = world.get_resource::<Journey>().map_or(0.0, |j| j.distance);
            let pct = (distance / JOURNEY_END * 100.0).round() as i32;
            draw_text(&format!("JOURNEY {pct}%"), ox, oy - 14.0, 32.0, WHITE);

            let hint = "RIGHT: fly  LEFT: back  UP/DOWN: steer";
            let dims = measure_text(hint, None, 20, 1.0);
            draw_text(hint, ox + BOARD_W - dims.width, oy - 14.0, 20.0, GRAY);
        }
        Scene::Planet(index) => {
            let title = format!("EXPLORING: {}", STATIONS[index].title);
            draw_text(&title, ox, oy - 14.0, 32.0, ACCENT_COLOR);

            let hint = "ARROWS: fly  E: leave orbit";
            let dims = measure_text(hint, None, 20, 1.0);
            draw_text(hint, ox + BOARD_W - dims.width, oy - 14.0, 20.0, GRAY);
        }
    }
}

pub fn spawn_initial(world: &mut World) {
    world.add(
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
    world.insert_resource(Scene::Space);
}

#[macroquad::main("Get to know me")]
async fn main() {
    set_pc_assets_folder("./assets");
    let texture: Texture2D = load_texture("ship.png").await.expect("Couldn't load file");
    texture.set_filter(FilterMode::Nearest);

    // Planet sprites and surface backdrops, generated by `cargo run --bin genart`.
    let mut planets = Vec::new();
    let mut surfaces = Vec::new();
    for i in 0..8 {
        let planet = load_texture(&format!("planet{i}.png"))
            .await
            .expect("Couldn't load planet - run `cargo run --bin genart`");
        planet.set_filter(FilterMode::Nearest);
        planets.push(planet);

        let surface = load_texture(&format!("surface{i}.png"))
            .await
            .expect("Couldn't load surface - run `cargo run --bin genart`");
        surface.set_filter(FilterMode::Nearest);
        surfaces.push(surface);
    }
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
    world.insert_resource(Art { planets, surfaces });

    let mut game = Game::new(world)
        .with_update_systems(vec![scene_system, travel_system, explore_system])
        .with_render_systems(vec![render_space, render_planet, ui_system]);

    loop {
        let input = Input {
            dt: get_frame_time(),
            spacebar: is_key_pressed(KeyCode::Space),
            // The engine's Input has no `e` field; E rides in the unused `a` slot.
            a: is_key_pressed(KeyCode::E),
            up: is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
            down: is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
            left: is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            right: is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
            screen_width: screen_width(),
            screen_height: screen_height(),
            ..Default::default()
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
