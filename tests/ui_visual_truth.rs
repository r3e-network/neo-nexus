//! Pixel-truth verification for the rendered UI.
//!
//! Screen capture (`screencapture`) is NOT a reliable way to verify this app's
//! rendering: on a Retina / scaled macOS desktop it captures the whole virtual
//! workspace (often larger than the screen), the app window may only occupy a
//! corner, and the rest is dark desktop — so a "screenshot" mostly shows empty
//! black space, not the app. Automated vision tools also fabricate precise
//! measurements (e.g. claiming "75% magenta" when pixel counting shows 0%).
//!
//! This test instead renders a real 1280x820 frame headlessly (the same path
//! the contract tests use) and rasterizes every painted fill rectangle into a
//! PNG. That PNG is the OBJECTIVE truth of what the app paints — every panel,
//! card, and surface background — with zero desktop / window-position /
//! screen-capture interference.
//!
//! It is `#[ignore]`d so it never runs in CI; run it on demand:
//!
//!     cargo test --release --test ui_visual_truth -- --nocapture --ignored
//!
//! Then inspect the PNGs at /tmp/neonexus_truth_light.png and _dark.png.
//!
//! For brightness/tier analysis (e.g. confirming the dark theme is not a wall
//! of near-black), count pixels with a quick script:
//!
//!     python3 -c "
//!     from PIL import Image
//!     from collections import Counter
//!     im=Image.open('/tmp/neonexus_truth_dark.png').convert('RGBA'); px=im.load()
//!     c=Counter()
//!     for y in range(0,im.size[1],3):
//!         for x in range(0,im.size[0],3):
//!             r,g,b,a=px[x,y]; c[(r//4*4,g//4*4,b//4*4)]+=1
//!     for (r,g,b),n in c.most_common(5):
//!         print(f'rgb({r:3},{g:3},{b:3}) lum={(r+g+b)//3:3}  {100*n/sum(c.values()):5.1f}%')
//!     "
//!
//! The bright-magenta base option below makes UNPAINTED regions visible: any
//! pixel that stays magenta means the app painted nothing there (a hole that
//! would render as black in a real window).

use std::path::Path;

use egui::{Color32, Pos2, Rect, Vec2};
use image::{ImageBuffer, Rgba, RgbaImage};
use neo_nexus::{repository::Repository, NeoNexusApp};

/// Design window size, matching the geometry contract (tests/ui_geometry.rs).
const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

/// When true, the raster base is bright magenta so any region the app leaves
/// unpainted stays magenta (visible as a "hole"). When false, the base is the
/// workspace colour so the PNG reads like a normal (fill-only) screenshot.
const MAGENTA_HOLE_DETECTOR: bool = false;

#[test]
#[ignore]
fn rasterize_both_themes() {
    for dark in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("neonexus.db");
        let repo = Repository::open(&path).unwrap();
        repo.save_app_dark_mode(dark).unwrap();
        drop(repo);
        let repo = Repository::open(&path).unwrap();
        let mut app = NeoNexusApp::new(repo);

        let ctx = egui::Context::default();
        let raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            ..Default::default()
        };
        let output = ctx.run(raw, |ctx| app.render_headless_frame(ctx));

        let base = if MAGENTA_HOLE_DETECTOR {
            Color32::from_rgb(255, 0, 255)
        } else if dark {
            Color32::from_rgb(20, 20, 23)
        } else {
            Color32::from_rgb(236, 236, 240)
        };
        let mut img: RgbaImage = ImageBuffer::from_pixel(
            SCREEN.x as u32,
            SCREEN.y as u32,
            Rgba([base.r(), base.g(), base.b(), 255]),
        );

        let mut fills = 0u32;
        for clipped in &output.shapes {
            if let egui::Shape::Rect(r) = &clipped.shape {
                if r.fill.a() == 0 {
                    continue;
                }
                paint_rect(&mut img, &r.rect, r.fill);
                fills += 1;
            }
        }

        let name = if dark { "dark" } else { "light" };
        let out = format!("/tmp/neonexus_truth_{name}.png");
        img.save(Path::new(&out)).unwrap();
        println!(
            "\n===== TRUTH [{name}] -> {out} (1280x820, {fills} fills, holes={}) =====",
            if MAGENTA_HOLE_DETECTOR { "ON" } else { "off" }
        );
    }
    println!("\nInspect /tmp/neonexus_truth_light.png and _dark.png");
}

fn paint_rect(img: &mut RgbaImage, rect: &Rect, color: Color32) {
    let x0 = rect.min.x.max(0.0) as i32;
    let y0 = rect.min.y.max(0.0) as i32;
    let x1 = (rect.max.x as i32).min(SCREEN.x as i32);
    let y1 = (rect.max.y as i32).min(SCREEN.y as i32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            if let Some(px) = img.get_pixel_mut_checked(x as u32, y as u32) {
                let sa = color.a() as f32 / 255.0;
                px[0] = (px[0] as f32 * (1.0 - sa) + color.r() as f32 * sa) as u8;
                px[1] = (px[1] as f32 * (1.0 - sa) + color.g() as f32 * sa) as u8;
                px[2] = (px[2] as f32 * (1.0 - sa) + color.b() as f32 * sa) as u8;
                px[3] = 255;
            }
        }
    }
}
