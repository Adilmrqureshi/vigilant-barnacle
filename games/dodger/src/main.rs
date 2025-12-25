use macroquad::prelude::*;

const MOVEMENT_SPEED: f32 = 200.0;
const RADIUS: f32 = 16.0;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    hit: bool,
}

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

#[macroquad::main("My game")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let mut gameover = false;
    let screen_boundary_x = screen_width() - RADIUS;
    let screen_boundary_y = screen_height() - RADIUS;
    let mut squares: Vec<Shape> = vec![];
    let mut bullets: Vec<Shape> = vec![];
    let mut circle = Shape {
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        hit: false,
    };

    loop {
        clear_background(DARKPURPLE);

        let delta_time = get_frame_time();
        if !gameover {
            if is_key_down(KeyCode::Right) {
                circle.x += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Left) {
                circle.x -= MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Down) {
                circle.y += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Up) {
                circle.y -= MOVEMENT_SPEED * delta_time;
            }
            if is_key_pressed(KeyCode::Space) {
                bullets.push(Shape {
                    x: circle.x,
                    y: circle.y,
                    speed: circle.speed * 2.0,
                    size: 5.0,
                    hit: false,
                })
            }
            circle.x = clamp(circle.x, circle.size / 2.0, screen_boundary_x);
            circle.y = clamp(circle.y, circle.size / 2.0, screen_boundary_y);

            if rand::gen_range(0, 99) >= 95 {
                let size = rand::gen_range(16.0, 64.0);
                squares.push(Shape {
                    size,
                    speed: rand::gen_range(50.0, 150.0),
                    x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                    y: -size,
                    hit: false,
                });
            }
            for square in &mut squares {
                square.y += square.speed * delta_time;
            }
            for bullet in &mut bullets {
                bullet.y -= bullet.speed * delta_time;
            }
            if squares.iter().any(|square| circle.collides_with(square)) {
                gameover = true;
            }
            for bullet in bullets.iter_mut() {
                for square in squares.iter_mut() {
                    if bullet.collides_with(square) {
                        bullet.hit = true;
                        square.hit = true;
                    }
                }
            }
        }

        squares.retain(|square| square.y < screen_height() + square.size);
        bullets.retain(|bullet| bullet.y > 0.0);

        squares.retain(|square| !square.hit);
        bullets.retain(|bullet| !bullet.hit);

        for square in &squares {
            draw_rectangle(
                square.x - square.size / 2.0,
                square.y - square.size / 2.0,
                square.size,
                square.size,
                GREEN,
            );
        }
        for bullet in &bullets {
            draw_circle(bullet.x, bullet.y, bullet.size, BLUE)
        }
        draw_circle(circle.x, circle.y, circle.size / 2.0, YELLOW);

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
                squares.clear();
                bullets.clear();
                circle.x = screen_width() / 2.0;
                circle.y = screen_height() / 2.0;
                gameover = false;
            }
        }

        next_frame().await
    }
}
