use macroquad::prelude::*;

pub struct Layer {
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
}
