// Generates the pixel-art planet sprites and planet-surface backdrops for the
// space CV. Pure std: PNGs are written by hand using stored (uncompressed)
// deflate blocks, so no image or compression crates are needed.
//
// Usage, from the repo root:
//   cargo run -p spacecv --bin genart --features genart
// Writes planet0..7.png (64x64) and surface0..7.png (200x120) into
// games/spacecv/assets (or the directory given as the first argument).
// Both are drawn at low resolution and upscaled with nearest-neighbour
// filtering in-game to match the ship's pixel-art style.

const PLANET_SIZE: usize = 64;
const PLANET_R: f32 = 17.0;
const SURF_W: usize = 200;
const SURF_H: usize = 120;

// ---------------------------------------------------------------------------
// PNG encoding
// ---------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { 0xEDB8_8320 ^ (crc >> 1) } else { crc >> 1 };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn push_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    out.extend(kind);
    out.extend(data);
    let mut crc_input = kind.to_vec();
    crc_input.extend(data);
    out.extend(crc32(&crc_input).to_be_bytes());
}

fn encode_png(w: usize, h: usize, rgba: &[u8]) -> Vec<u8> {
    // Every scanline gets filter byte 0 (no filtering).
    let mut raw = Vec::with_capacity((w * 4 + 1) * h);
    for row in rgba.chunks(w * 4) {
        raw.push(0);
        raw.extend(row);
    }

    // A valid zlib stream may consist entirely of stored deflate blocks.
    let mut idat = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65535).peekable();
    while let Some(block) = blocks.next() {
        idat.push(if blocks.peek().is_none() { 1 } else { 0 });
        idat.extend((block.len() as u16).to_le_bytes());
        idat.extend((!(block.len() as u16)).to_le_bytes());
        idat.extend(block);
    }
    idat.extend(adler32(&raw).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend((w as u32).to_be_bytes());
    ihdr.extend((h as u32).to_be_bytes());
    ihdr.extend([8, 6, 0, 0, 0]); // 8-bit RGBA
    push_chunk(&mut png, b"IHDR", &ihdr);
    push_chunk(&mut png, b"IDAT", &idat);
    push_chunk(&mut png, b"IEND", &[]);
    png
}

// ---------------------------------------------------------------------------
// Canvas, RNG and noise
// ---------------------------------------------------------------------------

struct Img {
    w: usize,
    h: usize,
    px: Vec<[u8; 4]>,
}

impl Img {
    fn new(w: usize, h: usize) -> Self {
        Self { w, h, px: vec![[0, 0, 0, 0]; w * h] }
    }

    fn put(&mut self, x: i32, y: i32, c: [u8; 4]) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.w + x as usize] = c;
        }
    }

    fn bytes(&self) -> Vec<u8> {
        self.px.iter().flatten().copied().collect()
    }
}

struct Rng(u32);

impl Rng {
    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (self.next() & 0xFFFF) as f32 / 65535.0 * (hi - lo)
    }
}

fn hash2(ix: i32, iy: i32, seed: u32) -> f32 {
    let mut h = (ix as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((iy as u32).wrapping_mul(668_265_263))
        ^ seed.wrapping_mul(2_246_822_519);
    h ^= h >> 13;
    h = h.wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h & 0xFFFF) as f32 / 65535.0
}

fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - ix as f32, y - iy as f32);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let n0 = hash2(ix, iy, seed) + (hash2(ix + 1, iy, seed) - hash2(ix, iy, seed)) * sx;
    let n1 =
        hash2(ix, iy + 1, seed) + (hash2(ix + 1, iy + 1, seed) - hash2(ix, iy + 1, seed)) * sx;
    n0 + (n1 - n0) * sy
}

fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    0.65 * noise(x, y, seed) + 0.35 * noise(x * 2.3, y * 2.3, seed ^ 0x9E37)
}

// ---------------------------------------------------------------------------
// Palettes: 4 shades, dark to light, matching the game's station colours
// ---------------------------------------------------------------------------

const PALETTES: [[[u8; 3]; 4]; 8] = [
    // 0: rust-red gas giant
    [[84, 32, 34], [140, 58, 48], [217, 115, 89], [245, 180, 130]],
    // 1: ocean world
    [[18, 42, 84], [38, 84, 150], [102, 166, 230], [180, 220, 245]],
    // 2: cratered desert
    [[95, 74, 38], [150, 118, 56], [204, 178, 89], [235, 215, 150]],
    // 3: ringed purple giant
    [[52, 38, 95], [88, 66, 150], [140, 115, 217], [200, 180, 245]],
    // 4: ice world
    [[70, 90, 130], [120, 145, 180], [190, 205, 225], [240, 248, 255]],
    // 5: lava world (rock shades; glow handled separately)
    [[40, 24, 28], [70, 40, 40], [110, 60, 50], [150, 90, 65]],
    // 6: mossy moon
    [[30, 70, 45], [55, 110, 60], [95, 160, 80], [160, 210, 120]],
    // 7: teal gas giant
    [[16, 66, 74], [30, 110, 115], [60, 170, 165], [140, 225, 210]],
];

const LAND: [[u8; 3]; 2] = [[60, 120, 70], [110, 170, 90]];
const GLOW: [[u8; 3]; 2] = [[255, 120, 40], [255, 200, 80]];
const RING: [[u8; 3]; 2] = [[130, 110, 88], [225, 205, 160]];

fn rgb(c: [u8; 3]) -> [u8; 4] {
    [c[0], c[1], c[2], 255]
}

fn mul(c: [u8; 3], f: f32) -> [u8; 4] {
    [
        (c[0] as f32 * f) as u8,
        (c[1] as f32 * f) as u8,
        (c[2] as f32 * f) as u8,
        255,
    ]
}

fn shade(pal: &[[u8; 3]; 4], v: f32) -> [u8; 4] {
    rgb(pal[(v.clamp(0.0, 0.999) * 4.0) as usize])
}

// ---------------------------------------------------------------------------
// Planet sprites
// ---------------------------------------------------------------------------

fn draw_planet(style: usize) -> Img {
    let mut img = Img::new(PLANET_SIZE, PLANET_SIZE);
    let mut rng = Rng(0x9E37_79B9 ^ (style as u32).wrapping_mul(2_654_435_761));
    let seed = rng.next();
    let c = (PLANET_SIZE as f32 - 1.0) / 2.0;
    let pal = &PALETTES[style];

    // Craters in planet-space coordinates, for the rocky styles.
    let craters: Vec<(f32, f32, f32)> = (0..6)
        .map(|_| {
            (
                rng.range(-0.7, 0.7),
                rng.range(-0.7, 0.7),
                rng.range(0.12, 0.3),
            )
        })
        .collect();

    for y in 0..PLANET_SIZE as i32 {
        for x in 0..PLANET_SIZE as i32 {
            let dx = (x as f32 - c) / PLANET_R;
            let dy = (y as f32 - c) / PLANET_R;
            let d2 = dx * dx + dy * dy;
            if d2 > 1.0 {
                continue;
            }
            let nz = (1.0 - d2).sqrt();
            let lum = (-0.45 * dx - 0.5 * dy + 0.74 * nz).max(0.0);
            let dither = ((x + y) & 1) as f32 * 0.07 - 0.035;
            let mut v = lum * 0.85 + 0.15 + dither;

            // Style-specific surface features.
            let mut special: Option<[u8; 4]> = None;
            match style {
                0 | 3 | 7 => {
                    // Latitude bands with a little wobble.
                    let freq = if style == 7 { 6.5 } else { 5.0 };
                    let amp = if style == 3 { 0.16 } else { 0.26 };
                    v += (dy * freq + 0.9 * (dx * 2.1 + style as f32).sin()).sin() * amp;
                    if style == 7 {
                        // A great storm spot.
                        let (sx, sy) = (dx + 0.32, dy + 0.28);
                        let d = (sx * sx + sy * sy * 3.0).sqrt();
                        if d < 0.28 {
                            v += 0.3;
                        } else if d < 0.38 {
                            v -= 0.15;
                        }
                    }
                }
                1 => {
                    // Continents on an ocean.
                    let n = fbm(x as f32 * 0.14, y as f32 * 0.14, seed);
                    if n > 0.55 {
                        special = Some(rgb(LAND[if lum > 0.55 { 1 } else { 0 }]));
                    } else {
                        v += (n - 0.5) * 0.3;
                    }
                }
                2 | 6 => {
                    v += (fbm(x as f32 * 0.18, y as f32 * 0.18, seed) - 0.5) * 0.3;
                    for &(cx2, cy2, r2) in &craters {
                        let d = ((dx - cx2).powi(2) + (dy - cy2).powi(2)).sqrt();
                        if d < r2 {
                            v -= 0.28;
                        } else if d < r2 * 1.4 {
                            v += 0.14;
                        }
                    }
                }
                4 => {
                    let n = fbm(x as f32 * 0.2, y as f32 * 0.2, seed);
                    v += (n - 0.5) * 0.5;
                    if dy.abs() > 0.55 + (n - 0.5) * 0.2 {
                        // Polar caps, blue-tinted so they read against the body.
                        special = Some(if lum > 0.5 {
                            [235, 245, 255, 255]
                        } else {
                            [150, 180, 220, 255]
                        });
                    }
                }
                5 => {
                    // Glowing cracks across dark rock.
                    let n = fbm(x as f32 * 0.16, y as f32 * 0.16, seed);
                    if (n - 0.5).abs() < 0.045 {
                        special = Some(rgb(GLOW[if lum > 0.5 { 1 } else { 0 }]));
                    } else {
                        v = v * 0.8;
                    }
                }
                _ => {}
            }

            if d2 > 0.86 {
                v -= 0.3; // limb darkening for a rounded look
            }
            img.put(x, y, special.unwrap_or_else(|| shade(pal, v)));
        }
    }

    // The ringed planet gets its ring drawn last so the front half overlaps.
    if style == 3 {
        let (s, co) = (-0.3f32).sin_cos();
        for y in 0..PLANET_SIZE as i32 {
            for x in 0..PLANET_SIZE as i32 {
                let px = x as f32 - c;
                let py = y as f32 - c;
                let (u, w) = (px * co + py * s, -px * s + py * co);
                let e = (u / 29.0).powi(2) + (w / 9.0).powi(2);
                if !(0.5..=1.0).contains(&e) {
                    continue;
                }
                let dx = px / PLANET_R;
                let dy = py / PLANET_R;
                // The half of the ring passing behind the planet is hidden.
                if dx * dx + dy * dy < 1.0 && w < 0.0 {
                    continue;
                }
                let band = if !(0.62..=0.9).contains(&e) { 0 } else { 1 };
                img.put(x, y, rgb(RING[band]));
            }
        }
    }

    img
}

// ---------------------------------------------------------------------------
// Surface backdrops
// ---------------------------------------------------------------------------

fn draw_surface(style: usize) -> Img {
    let mut img = Img::new(SURF_W, SURF_H);
    let mut rng = Rng(0xC0FF_EE00 ^ (style as u32).wrapping_mul(2_654_435_761));
    let seed = rng.next();
    let pal = &PALETTES[style];

    // Sky gradient, brightening towards the horizon.
    for y in 0..SURF_H {
        let t = (y as f32 / 95.0).min(1.0);
        let f = 0.3 + t * 0.45;
        for x in 0..SURF_W {
            img.put(x as i32, y as i32, mul(pal[0], f));
        }
    }

    // Stars.
    for _ in 0..80 {
        let x = rng.range(0.0, SURF_W as f32) as i32;
        let y = rng.range(0.0, 65.0) as i32;
        let bright = if rng.next() & 3 == 0 { 255 } else { 170 };
        img.put(x, y, [bright, bright, bright, 255]);
    }

    // A moon or parent planet hanging in the sky.
    let (mx, my, mr) = (rng.range(30.0, 170.0), rng.range(12.0, 32.0), rng.range(7.0, 11.0));
    for y in (my - mr) as i32..=(my + mr) as i32 {
        for x in (mx - mr) as i32..=(mx + mr) as i32 {
            let dx = (x as f32 - mx) / mr;
            let dy = (y as f32 - my) / mr;
            let d2 = dx * dx + dy * dy;
            if d2 > 1.0 {
                continue;
            }
            let lum = (-0.45 * dx - 0.5 * dy + 0.74 * (1.0 - d2).sqrt()).max(0.0);
            img.put(x, y, shade(&PALETTES[(style + 3) % 8], lum * 0.8 + 0.2));
        }
    }

    // Distant hills (dark silhouette with a lit ridge), then textured ground.
    for x in 0..SURF_W as i32 {
        let hill = (78.0 - fbm(x as f32 * 0.045, 0.0, seed) * 16.0) as i32;
        img.put(x, hill, mul(pal[2], 0.85));
        for y in hill + 1..SURF_H as i32 {
            img.put(x, y, mul(pal[0], 0.55));
        }
        let ground = (96.0 - fbm(x as f32 * 0.08, 40.0, seed ^ 0xABCD) * 7.0) as i32;
        for y in ground..SURF_H as i32 {
            let speck = hash2(x, y, seed ^ 0x5EED);
            let f = if speck > 0.92 {
                0.85
            } else if speck < 0.1 {
                0.5
            } else {
                0.65
            };
            img.put(x, y, mul(pal[1], f));
        }
    }

    img
}

// ---------------------------------------------------------------------------

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "games/spacecv/assets".to_string());

    for style in 0..8 {
        let planet = draw_planet(style);
        let path = format!("{out_dir}/planet{style}.png");
        std::fs::write(&path, encode_png(planet.w, planet.h, &planet.bytes()))
            .unwrap_or_else(|e| panic!("couldn't write {path}: {e}"));

        let surface = draw_surface(style);
        let path = format!("{out_dir}/surface{style}.png");
        std::fs::write(&path, encode_png(surface.w, surface.h, &surface.bytes()))
            .unwrap_or_else(|e| panic!("couldn't write {path}: {e}"));
    }
    println!("wrote 8 planets + 8 surfaces to {out_dir}");
}
