use macroquad::prelude::*;

// let texture_1: Texture2D = load_texture("1.png").await.expect("Couldn't load file");
// texture_1.set_filter(FilterMode::Nearest);
// let texture_2: Texture2D = load_texture("2.png").await.expect("Couldn't load file");
// texture_2.set_filter(FilterMode::Nearest);
// let texture_3: Texture2D = load_texture("3.png").await.expect("Couldn't load file");
// texture_3.set_filter(FilterMode::Nearest);
// let texture_4: Texture2D = load_texture("4.png").await.expect("Couldn't load file");
// texture_4.set_filter(FilterMode::Nearest);
// let texture_5: Texture2D = load_texture("5.png").await.expect("Couldn't load file");
// texture_5.set_filter(FilterMode::Nearest);
// Parallex(
//     Tick {
//         texture: texture_1,
//         tick: 0.0,
//     },
//     Tick {
//         texture: texture_2,
//         tick: 0.0,
//     },
//     Tick {
//         texture: texture_3,
//         tick: 0.0,
//     },
//     Tick {
//         texture: texture_4,
//         tick: 0.0,
//     },
//     Tick {
//         texture: texture_5,
//         tick: 0.0,
//     },
// )

// if para.0.tick >= min_width {
//     para.0.tick -= min_width;
// }
//
// if para.1.tick >= min_width {
//     para.1.tick -= min_width;
// }
//
// if para.2.tick >= min_width {
//     para.2.tick -= min_width;
// }
//
// if para.3.tick >= min_width {
//     para.3.tick -= min_width;
// }
//
// if para.4.tick >= min_width {
//     para.4.tick -= min_width;
// }

// if !game_over {
//     para.0.tick += time * 3.0;
//     para.1.tick += time * 9.0;
//     para.2.tick += time * 27.0;
//     para.3.tick += time * 81.0;
//     para.4.tick += time * 243.0;
// }

struct Layer {
    pub texture: Texture2D,
    pub delta: f32,
    pub speed: i32,
}
pub struct Parallex {
    pub layers: Vec<Layer>,
}

pub async fn load_background_assets() -> Parallex {
    let files = ["1.png", "2.png", "3.png", "4.png", "5.png"];
    let speeds = [3, 9, 27, 81, 243];
    let mut para = Parallex { layers: vec![] };

    for (file, speed) in files.iter().zip(speeds) {
        let texture: Texture2D = load_texture(file).await.expect("Couldn't load file");
        texture.set_filter(FilterMode::Nearest);

        let layer = Layer {
            texture,
            delta: 0.0,
            speed,
        };

        para.layers.push(layer);
    }

    para
}

pub fn render_paralax_background(para: &mut Parallex, game_over: bool) {
    let dt = get_frame_time() as f32;
    let w = screen_width();
    let h = screen_height();

    for layer in &mut para.layers {
        if layer.delta >= w {
            layer.delta -= w;
        }

        if !game_over {
            layer.delta += dt * layer.speed as f32;
        }

        let x1 = -layer.delta;
        let x2 = w - layer.delta;

        let params = DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            flip_y: true,
            ..Default::default()
        };

        draw_texture_ex(&layer.texture, x1, 0.0, WHITE, params.clone());
        draw_texture_ex(&layer.texture, x2, 0.0, WHITE, params);
    }

    //
    // draw_texture_ex(
    //     &para.0.texture,
    //     -para.0.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.0.texture,
    //     min_width - para.0.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.1.texture,
    //     min_width - para.1.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.1.texture,
    //     -para.1.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.2.texture,
    //     -para.2.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.2.texture,
    //     min_width - para.2.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.3.texture,
    //     -para.3.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.3.texture,
    //     min_width - para.3.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.4.texture,
    //     -para.4.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
    //
    // draw_texture_ex(
    //     &para.4.texture,
    //     min_width - para.4.tick,
    //     0.0,
    //     WHITE,
    //     DrawTextureParams {
    //         dest_size: Some(vec2(min_width, min_height)),
    //         flip_y: true,
    //         ..Default::default()
    //     },
    // );
}
