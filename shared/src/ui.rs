use macroquad::{camera::set_default_camera, color::Color, text::draw_text};

use crate::{Transform, World};

pub fn render_text(world: &mut World, text: &str, font_size: f32, pos: &Transform, color: Color) {
    set_default_camera();
    draw_text(text, pos.x, pos.y, font_size, color);
    world.set_default_origin();
}
