// Pure drawing helpers for the UI chrome every game repeats: the help line
// under the board and the game-over overlay. No World or GameState knowledge;
// data in, pixels out.
use macroquad::color::{Color, GRAY, WHITE};
use macroquad::math::Rect;
use macroquad::text::{draw_text, measure_text};
use macroquad::window::{screen_height, screen_width};

/// Draws `text` horizontally centred on the screen, baseline at `y`.
pub fn draw_text_centered(text: &str, y: f32, font_size: u16, color: Color) {
    let dims = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - dims.width / 2.0,
        y,
        font_size as f32,
        color,
    );
}

/// One-line control hint, centred just below the playing field.
pub fn draw_help_line(text: &str, board: Rect) {
    let dims = measure_text(text, None, 20, 1.0);
    draw_text(
        text,
        board.x + (board.w - dims.width) / 2.0,
        board.y + board.h + 26.0,
        20.0,
        GRAY,
    );
}

/// Big end-of-game title, centred on the middle of the screen.
pub fn draw_game_over_title(text: &str, font_size: u16, color: Color) {
    let dims = measure_text(text, None, font_size, 1.0);
    draw_text_centered(
        text,
        screen_height() / 2.0 - dims.height / 2.0,
        font_size,
        color,
    );
}

/// Restart instruction shown under the game-over title.
pub fn draw_restart_hint(text: &str) {
    draw_text_centered(text, screen_height() / 2.0 + 40.0, 30, WHITE);
}

/// The standard game-over screen: 60pt title plus "press SPACE to restart".
pub fn draw_game_over_overlay(title: &str, title_color: Color) {
    draw_game_over_title(title, 60, title_color);
    draw_restart_hint("press SPACE to restart");
}
