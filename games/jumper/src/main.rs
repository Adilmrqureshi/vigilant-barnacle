use macroquad::prelude::*;

struct Shape {
    size: f32,
    force: f32,
    x: f32,
    y: f32,
    is_jumping: bool,
}

const DEFAULT_SIZE: f32 = 64.0;
const GRAVITY: f32 = 9.81;
const GROUND_HEIGHT: f32 = 100.0;

impl Shape {
    fn collides_with(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }
}

fn movement(player: &mut Shape) {
    if is_key_pressed(KeyCode::Space) {
        if !player.is_jumping {
            player.is_jumping = true;
            player.force = 6.0;
        }
    }
}

fn move_enemy(enemy: &mut Shape) {
    if enemy.x < -DEFAULT_SIZE {
        enemy.x = screen_width() + DEFAULT_SIZE * rand::gen_range(0, 10) as f32
    }
    enemy.x -= enemy.force;
}

fn render(player: &Shape, enemy: &Shape) {
    draw_rectangle(player.x, player.y, player.size, player.size, YELLOW);
    draw_rectangle(enemy.x, enemy.y, enemy.size, enemy.size, RED);
    draw_rectangle(
        0.0,
        screen_height() - GROUND_HEIGHT,
        screen_width(),
        GROUND_HEIGHT,
        BLUE,
    );
}

fn physics(player: &mut Shape, ground: &f32) {
    let time_delta = get_frame_time();
    player.y -= player.force;
    if player.y < *ground {
        player.force -= GRAVITY * time_delta;
    } else {
        player.y = *ground;
        player.is_jumping = false;
    }
}

#[macroquad::main("")]
async fn main() {
    let ground = screen_height() - GROUND_HEIGHT - DEFAULT_SIZE;
    let mut gameover = false;
    let mut player = Shape {
        x: 100.0,
        y: ground,
        size: DEFAULT_SIZE,
        force: 0.0,
        is_jumping: false,
    };

    let mut enemy = Shape {
        x: screen_width() + DEFAULT_SIZE,
        y: ground,
        size: DEFAULT_SIZE,
        force: 10.0,
        is_jumping: false,
    };

    let mut score = 0.0;
    loop {
        clear_background(DARKGREEN);

        if !gameover {
            score = get_time() * 1000.0;
            physics(&mut player, &ground);
            movement(&mut player);
            move_enemy(&mut enemy);
        }

        let text = format!("{:.0}", score);
        let text_dimensions = measure_text(&text, None, 50, 1.0);
        draw_text(&text, 10.0, 10.0 + text_dimensions.height, 32.0, WHITE);

        render(&player, &enemy);

        if enemy.collides_with(&player) {
            gameover = true;
        }

        if gameover {
            let text = "GAME OVER!";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            draw_text(
                text,
                screen_width() / 2.0 - text_dimensions.width / 2.0,
                screen_height() / 2.0 - text_dimensions.height / 2.0,
                50.0,
                RED,
            );
            if is_key_pressed(KeyCode::Space) {
                player.x = screen_width() / 2.0;
                player.y = screen_height() / 2.0;
                gameover = false;
            }
        }

        next_frame().await
    }
}
