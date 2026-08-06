//! A tiny software rasterizer for headless egui frames.
//!
//! `Context::tessellate` turns a frame's shapes into textured triangle meshes —
//! exactly what a GPU backend uploads. Walking those triangles on the CPU and
//! sampling egui's own font atlas reproduces the real rendered pixels (panels,
//! hairlines, rounded corners, and readable glyphs) without a window, a GPU, or
//! the macOS screen-capture permission. That makes it the objective answer to
//! "what does the workbench actually look like?".

use std::collections::HashMap;

use egui::{
    epaint::{image::ImageData, Primitive, Vertex},
    Color32, Context, FullOutput, Pos2, Rect, TextureId, Vec2,
};
use image::{ImageBuffer, Rgba, RgbaImage};

/// A CPU copy of one egui texture (the font atlas, plus any user textures).
struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Color32>,
}

impl Texture {
    /// Nearest-neighbour sample in normalised uv space.
    fn sample(&self, u: f32, v: f32) -> Color32 {
        if self.width == 0 || self.height == 0 {
            return Color32::WHITE;
        }
        let x = ((u * self.width as f32) as isize).clamp(0, self.width as isize - 1) as usize;
        let y = ((v * self.height as f32) as isize).clamp(0, self.height as isize - 1) as usize;
        self.pixels[y * self.width + x]
    }
}

/// Applies the texture uploads from every frame in `frames` — the font atlas is
/// built incrementally, so a later frame's delta only patches what earlier
/// frames already allocated — then rasterises the LAST frame's tessellated
/// meshes onto a `base`-filled canvas of `screen` points.
pub fn rasterize(
    context: &Context,
    frames: Vec<FullOutput>,
    screen: Vec2,
    base: Color32,
) -> RgbaImage {
    let mut textures: HashMap<TextureId, Texture> = HashMap::new();
    let mut shapes = Vec::new();
    let mut pixels_per_point = 1.0;
    for output in frames {
        for (id, delta) in &output.textures_delta.set {
            apply_delta(&mut textures, *id, delta);
        }
        shapes = output.shapes;
        pixels_per_point = output.pixels_per_point;
    }

    let mut img: RgbaImage = ImageBuffer::from_pixel(
        screen.x as u32,
        screen.y as u32,
        Rgba([base.r(), base.g(), base.b(), 255]),
    );
    let screen_rect = Rect::from_min_size(Pos2::ZERO, screen);

    for clipped in context.tessellate(shapes, pixels_per_point) {
        let Primitive::Mesh(mesh) = clipped.primitive else {
            continue;
        };
        let clip = clipped.clip_rect.intersect(screen_rect);
        if !clip.is_positive() {
            continue;
        }
        let texture = textures.get(&mesh.texture_id);
        for triangle in mesh.indices.chunks_exact(3) {
            let vertices = [
                mesh.vertices[triangle[0] as usize],
                mesh.vertices[triangle[1] as usize],
                mesh.vertices[triangle[2] as usize],
            ];
            draw_triangle(&mut img, clip, &vertices, texture);
        }
    }
    img
}

fn apply_delta(
    textures: &mut HashMap<TextureId, Texture>,
    id: TextureId,
    delta: &egui::epaint::ImageDelta,
) {
    let ImageData::Color(source) = &delta.image;
    let [src_w, src_h] = source.size;
    match delta.pos {
        None => {
            textures.insert(
                id,
                Texture {
                    width: src_w,
                    height: src_h,
                    pixels: source.pixels.clone(),
                },
            );
        }
        Some([x0, y0]) => {
            let Some(target) = textures.get_mut(&id) else {
                return;
            };
            for row in 0..src_h {
                for col in 0..src_w {
                    let (x, y) = (x0 + col, y0 + row);
                    if x < target.width && y < target.height {
                        target.pixels[y * target.width + x] = source.pixels[row * src_w + col];
                    }
                }
            }
        }
    }
}

/// Signed area of the triangle `(a, b, c)` — the standard edge function.
fn edge(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn draw_triangle(img: &mut RgbaImage, clip: Rect, v: &[Vertex; 3], texture: Option<&Texture>) {
    let area = edge(v[0].pos, v[1].pos, v[2].pos);
    if area.abs() < f32::EPSILON {
        return;
    }
    let min_x = v[0].pos.x.min(v[1].pos.x).min(v[2].pos.x).max(clip.min.x);
    let max_x = v[0].pos.x.max(v[1].pos.x).max(v[2].pos.x).min(clip.max.x);
    let min_y = v[0].pos.y.min(v[1].pos.y).min(v[2].pos.y).max(clip.min.y);
    let max_y = v[0].pos.y.max(v[1].pos.y).max(v[2].pos.y).min(clip.max.y);
    if min_x > max_x || min_y > max_y {
        return;
    }

    for y in (min_y.floor() as i64)..=(max_y.ceil() as i64) {
        for x in (min_x.floor() as i64)..=(max_x.ceil() as i64) {
            let point = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
            if !clip.contains(point) {
                continue;
            }
            let w = [
                edge(v[1].pos, v[2].pos, point) / area,
                edge(v[2].pos, v[0].pos, point) / area,
                edge(v[0].pos, v[1].pos, point) / area,
            ];
            if w.iter().any(|weight| *weight < 0.0) {
                continue;
            }
            let source = shade(v, &w, texture);
            blend(img, x, y, source);
        }
    }
}

/// Interpolates vertex colour and uv, then multiplies by the sampled texel.
/// Both egui vertex colours and atlas texels carry premultiplied alpha, so the
/// componentwise product is the premultiplied source colour.
fn shade(v: &[Vertex; 3], w: &[f32; 3], texture: Option<&Texture>) -> [f32; 4] {
    let mut colour = [0.0_f32; 4];
    for (index, weight) in w.iter().enumerate() {
        let c = v[index].color;
        colour[0] += weight * c.r() as f32;
        colour[1] += weight * c.g() as f32;
        colour[2] += weight * c.b() as f32;
        colour[3] += weight * c.a() as f32;
    }
    let Some(texture) = texture else {
        return colour;
    };
    let u = w[0] * v[0].uv.x + w[1] * v[1].uv.x + w[2] * v[2].uv.x;
    let vv = w[0] * v[0].uv.y + w[1] * v[1].uv.y + w[2] * v[2].uv.y;
    let texel = texture.sample(u, vv);
    [
        colour[0] * texel.r() as f32 / 255.0,
        colour[1] * texel.g() as f32 / 255.0,
        colour[2] * texel.b() as f32 / 255.0,
        colour[3] * texel.a() as f32 / 255.0,
    ]
}

/// `source over destination` for premultiplied source colour.
fn blend(img: &mut RgbaImage, x: i64, y: i64, source: [f32; 4]) {
    if x < 0 || y < 0 || source[3] <= 0.0 {
        return;
    }
    let Some(pixel) = img.get_pixel_mut_checked(x as u32, y as u32) else {
        return;
    };
    let inverse = 1.0 - (source[3] / 255.0).clamp(0.0, 1.0);
    for channel in 0..3 {
        let value = source[channel] + pixel[channel] as f32 * inverse;
        pixel[channel] = value.clamp(0.0, 255.0) as u8;
    }
    pixel[3] = 255;
}
