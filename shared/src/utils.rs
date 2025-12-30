use macroquad::{color::WHITE, shapes::draw_rectangle_lines};

use crate::Entity;

pub fn debug(entities: &[Entity]) {
    for e in entities {
        draw_rectangle_lines(
            e.transform.x,
            e.transform.y,
            e.transform.w,
            e.transform.h,
            2.0,
            WHITE,
        );
    }
}
