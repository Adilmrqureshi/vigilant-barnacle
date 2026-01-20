use macroquad::prelude::*;

use crate::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

struct Tick {
    pub texture: Texture2D,
    pub tick: f32,
}
pub struct Parallex(Tick, Tick, Tick, Tick, Tick);

pub async fn load_background_assets() -> Parallex {
    let texture_1: Texture2D = load_texture("1.png").await.expect("Couldn't load file");
    texture_1.set_filter(FilterMode::Nearest);
    let texture_2: Texture2D = load_texture("2.png").await.expect("Couldn't load file");
    texture_2.set_filter(FilterMode::Nearest);
    let texture_3: Texture2D = load_texture("3.png").await.expect("Couldn't load file");
    texture_3.set_filter(FilterMode::Nearest);
    let texture_4: Texture2D = load_texture("4.png").await.expect("Couldn't load file");
    texture_4.set_filter(FilterMode::Nearest);
    let texture_5: Texture2D = load_texture("5.png").await.expect("Couldn't load file");
    texture_5.set_filter(FilterMode::Nearest);
    Parallex(
        Tick {
            texture: texture_1,
            tick: 0.0,
        },
        Tick {
            texture: texture_2,
            tick: 0.0,
        },
        Tick {
            texture: texture_3,
            tick: 0.0,
        },
        Tick {
            texture: texture_4,
            tick: 0.0,
        },
        Tick {
            texture: texture_5,
            tick: 0.0,
        },
    )
}

pub fn render_paralax_background(para: &mut Parallex) {
    let time = get_frame_time() as f32;

    if para.0.tick >= VIRTUAL_WIDTH {
        para.0.tick -= VIRTUAL_WIDTH;
    }

    if para.1.tick >= VIRTUAL_WIDTH {
        para.1.tick -= VIRTUAL_WIDTH;
    }

    if para.2.tick >= VIRTUAL_WIDTH {
        para.2.tick -= VIRTUAL_WIDTH;
    }

    if para.3.tick >= VIRTUAL_WIDTH {
        para.3.tick -= VIRTUAL_WIDTH;
    }

    if para.4.tick >= VIRTUAL_WIDTH {
        para.4.tick -= VIRTUAL_WIDTH;
    }

    para.0.tick += time * 3.0;
    para.1.tick += time * 9.0;
    para.2.tick += time * 27.0;
    para.3.tick += time * 81.0;
    para.4.tick += time * 243.0;

    draw_texture_ex(
        &para.0.texture,
        -para.0.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.0.texture,
        VIRTUAL_WIDTH - (para.0.tick + 4.0),
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.1.texture,
        VIRTUAL_WIDTH - para.1.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.1.texture,
        -para.1.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.2.texture,
        -para.2.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.2.texture,
        VIRTUAL_WIDTH - para.2.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.3.texture,
        -para.3.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.3.texture,
        VIRTUAL_WIDTH - para.3.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.4.texture,
        -para.4.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &para.4.texture,
        VIRTUAL_WIDTH - para.4.tick,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
            flip_y: true,
            ..Default::default()
        },
    );
}
