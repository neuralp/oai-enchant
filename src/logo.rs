use egui::{Color32, Vec2};

// 8-bit palette
const BG:      Color32 = Color32::from_rgb(8,   8,  20);
const WAND:    Color32 = Color32::from_rgb(160, 100, 40);
const WAND_HI: Color32 = Color32::from_rgb(210, 155, 70);
const STAR:    Color32 = Color32::from_rgb(255, 210,  0);
const STAR_HI: Color32 = Color32::from_rgb(255, 255, 160);
const SPARK:   Color32 = Color32::from_rgb(255, 230, 60);

// ── Welcome-screen logo ───────────────────────────────────────────────────────

/// Draws the wand pixel-art logo into a 300×170 area inside `ui`.
pub fn draw_logo(ui: &mut egui::Ui) {
    const PS: f32 = 5.0; // one "pixel" = 5×5 screen points
    let desired = Vec2::new(300.0, 170.0);
    let (resp, painter) = ui.allocate_painter(desired, egui::Sense::hover());
    let r  = resp.rect;
    let ox = r.min.x;
    let oy = r.min.y;

    painter.rect_filled(r, 0.0, BG);

    // Draw one grid "pixel" at (gx, gy)
    let pix = |gx: i32, gy: i32, color: Color32| {
        let min = egui::Pos2::new(ox + gx as f32 * PS, oy + gy as f32 * PS);
        painter.rect_filled(egui::Rect::from_min_size(min, Vec2::splat(PS)), 0.0, color);
    };

    // Wand shaft via Bresenham: base (8, 30) → tip (18, 8), 2px wide
    {
        let (mut x, mut y) = (8i32, 30i32);
        let (ex, ey) = (18i32, 8i32);
        let adx = (ex - x).abs();
        let ady = (ey - y).abs();
        let sx  = if ex > x { 1i32 } else { -1 };
        let sy  = if ey > y { 1i32 } else { -1 };
        let mut err = adx - ady;
        loop {
            pix(x,     y, WAND);
            pix(x + 1, y, WAND_HI);
            if x == ex && y == ey { break; }
            let e2 = 2 * err;
            if e2 > -ady { err -= ady; x += sx; }
            if e2 <  adx { err += adx; y += sy; }
        }
    }

    // 5×5 pixel-cross star, centre (19, 7)
    for &(gx, gy, c) in &[
        (19i32, 5i32, STAR_HI),
        (18, 6, STAR),  (19, 6, STAR_HI), (20, 6, STAR),
        (17, 7, STAR),  (18, 7, STAR),    (19, 7, STAR_HI), (20, 7, STAR), (21, 7, STAR),
        (18, 8, STAR),  (19, 8, STAR_HI), (20, 8, STAR),
        (19, 9, STAR),
    ] {
        pix(gx, gy, c);
    }

    // Scattered sparkle pixels — magic particles radiating from the wand tip
    for &(gx, gy) in &[
        // Near the star
        (22, 5), (23, 3), (24, 9), (21, 11),
        // Mid-canvas spread
        (27, 4), (29, 8), (31, 3), (33, 6), (35, 10), (36, 4),
        (39, 7), (41, 4), (43, 9), (45, 5), (47, 3), (48, 8),
        // Far right
        (51, 5), (53, 7), (55, 4), (57, 9), (58, 6),
        // Lower-half background stars
        (26, 16), (32, 14), (38, 17), (44, 13), (50, 15), (56, 18),
        // Above the wand shaft
        (11, 2), (13, 4), (7,  6),
        // Deep field
        (30, 21), (42, 23), (54, 20),
    ] {
        pix(gx, gy, SPARK);
    }
}

// ── Window icon ───────────────────────────────────────────────────────────────

/// Generates a 128×128 RGBA pixel-art icon.
pub fn make_icon() -> egui::IconData {
    const SIZE: usize = 128;
    const PS:   f32   = 5.0;
    let mut buf = vec![0u8; SIZE * SIZE * 4];

    fill_rect_f(&mut buf, SIZE, 0.0, 0.0, SIZE as f32, SIZE as f32, 8, 8, 20, 255);

    // Wand shaft via Bresenham: base (2, 21) → tip (12, 4), 2px wide
    {
        let (mut x, mut y) = (2i32, 21i32);
        let (ex, ey) = (12i32, 4i32);
        let adx = (ex - x).abs();
        let ady = (ey - y).abs();
        let sx  = if ex > x { 1i32 } else { -1 };
        let sy  = if ey > y { 1i32 } else { -1 };
        let mut err = adx - ady;
        loop {
            icon_pix(&mut buf, SIZE, x,     y, PS, 160, 100, 40);
            icon_pix(&mut buf, SIZE, x + 1, y, PS, 210, 155, 70);
            if x == ex && y == ey { break; }
            let e2 = 2 * err;
            if e2 > -ady { err -= ady; x += sx; }
            if e2 <  adx { err += adx; y += sy; }
        }
    }

    // 5×5 pixel-cross star, centre (13, 4)
    for &(gx, gy, r, g, b) in &[
        (13i32, 2i32, 255u8, 255u8, 160u8),
        (12, 3, 255, 210, 0), (13, 3, 255, 255, 160), (14, 3, 255, 210, 0),
        (11, 4, 255, 210, 0), (12, 4, 255, 210, 0), (13, 4, 255, 255, 160), (14, 4, 255, 210, 0), (15, 4, 255, 210, 0),
        (12, 5, 255, 210, 0), (13, 5, 255, 255, 160), (14, 5, 255, 210, 0),
        (13, 6, 255, 210, 0),
    ] {
        icon_pix(&mut buf, SIZE, gx, gy, PS, r, g, b);
    }

    // Scatter sparkles
    for &(gx, gy) in &[
        (16, 2), (17, 5), (18, 2), (20, 4), (22, 6),
        (10, 2), (9,  5), (8,  8),
        (19, 8), (21, 10), (23, 7), (24, 9),
        (14, 9), (15, 12), (19, 13),
    ] {
        icon_pix(&mut buf, SIZE, gx, gy, PS, 255, 230, 60);
    }

    egui::IconData { rgba: buf, width: SIZE as u32, height: SIZE as u32 }
}

// ── Pixel-buffer helpers ──────────────────────────────────────────────────────

fn icon_pix(buf: &mut [u8], size: usize, gx: i32, gy: i32, ps: f32,
            r: u8, g: u8, b: u8) {
    if gx < 0 || gy < 0 { return; }
    let x0 = gx as f32 * ps;
    let y0 = gy as f32 * ps;
    fill_rect_f(buf, size, x0, y0, x0 + ps, y0 + ps, r, g, b, 255);
}

fn fill_rect_f(buf: &mut [u8], size: usize,
               x0: f32, y0: f32, x1: f32, y1: f32,
               red: u8, grn: u8, blu: u8, alpha: u8) {
    let ix0 = (x0.floor() as usize).min(size - 1);
    let iy0 = (y0.floor() as usize).min(size - 1);
    let ix1 = (x1.ceil()  as usize).min(size - 1);
    let iy1 = (y1.ceil()  as usize).min(size - 1);
    for y in iy0..=iy1 {
        for x in ix0..=ix1 {
            let i = (y * size + x) * 4;
            buf[i] = red; buf[i+1] = grn; buf[i+2] = blu; buf[i+3] = alpha;
        }
    }
}
