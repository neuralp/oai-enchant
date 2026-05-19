use egui::Vec2;

// ── Welcome-screen logo ───────────────────────────────────────────────────────

/// Renders the pixel-art SVG logo into a 300×170 area inside `ui`.
pub fn draw_logo(ui: &mut egui::Ui) {
    let bytes: &[u8] = include_bytes!("../assets/logo.svg");
    let img = egui::Image::from_bytes("bytes://oai-enchant-logo.svg", bytes)
        .fit_to_exact_size(Vec2::new(300.0, 170.0));
    ui.add(img);
}

// ── Window / taskbar icon ─────────────────────────────────────────────────────

/// Generates a 128×128 RGBA pixel-art icon matching the logo design.
/// Grid: 32×32 pixels at PS=4. Left half = wand; right half = API document.
pub fn make_icon() -> egui::IconData {
    const SIZE: usize = 128;
    const PS:   f32   = 4.0;
    let mut buf = vec![0u8; SIZE * SIZE * 4];

    // Background
    fill_rect_f(&mut buf, SIZE, 0.0, 0.0, SIZE as f32, SIZE as f32, 8, 8, 20, 255);

    // ── Wand star (centre gx=7, gy=4) ────────────────────────────────────────
    for &(gx, gy, r, g, b) in &[
        (7i32, 2i32, 255u8, 255u8, 160u8),                                              // top spike
        (6, 3, 255, 210, 0), (7, 3, 255, 255, 160), (8, 3, 255, 210, 0),               // row 3
        (5, 4, 255, 210, 0), (6, 4, 255, 210, 0), (7, 4, 255, 255, 160),               // row 4
        (8, 4, 255, 210, 0), (9, 4, 255, 210, 0),
        (6, 5, 255, 210, 0), (7, 5, 255, 255, 160), (8, 5, 255, 210, 0),               // row 5
        (7, 6, 255, 255, 160),                                                           // bottom spike
    ] {
        icon_pix(&mut buf, SIZE, gx, gy, PS, r, g, b);
    }

    // ── Wand shaft: 2-pixel-wide staircase from (gx=7,gy=7) to (gx=2,gy=17) ─
    // Each row: (gy, gx_dark, gx_light)
    for &(gy, gx_d, gx_l) in &[
        (7i32, 6i32, 7i32), (8,  6, 7),
        (9,  5, 6),         (10, 5, 6),
        (11, 4, 5),         (12, 4, 5),
        (13, 3, 4),         (14, 3, 4),
        (15, 2, 3),         (16, 2, 3),
        (17, 2, 3),
    ] {
        icon_pix(&mut buf, SIZE, gx_d, gy, PS, 160, 100, 40);
        icon_pix(&mut buf, SIZE, gx_l, gy, PS, 210, 155, 70);
    }

    // ── Sparkles ──────────────────────────────────────────────────────────────
    for &(gx, gy) in &[
        (10i32, 2i32), (12, 3), (11, 5), (13, 4),
        (9, 7), (11, 8), (12, 10), (10, 12), (12, 14),
        (0, 6), (0, 9), (1, 12), (0, 15), (1, 18), (0, 21),
        (4, 20), (3, 23), (5, 25), (2, 27),
        (13, 17), (12, 20), (13, 23), (12, 27),
    ] {
        icon_pix(&mut buf, SIZE, gx, gy, PS, 255, 224, 48);
    }

    // ── API document page body: cols 14-29, rows 2-28 ────────────────────────
    fill_rect_f(&mut buf, SIZE, 56.0, 8.0, 120.0, 116.0, 184, 196, 216, 255);

    // Corner fold: 3-step staircase cut at top-right
    fill_rect_f(&mut buf, SIZE, 108.0,  8.0, 120.0, 12.0, 8, 8, 20, 255); // row 2: cut cols 27-29
    fill_rect_f(&mut buf, SIZE, 112.0, 12.0, 120.0, 16.0, 8, 8, 20, 255); // row 3: cut cols 28-29
    fill_rect_f(&mut buf, SIZE, 116.0, 16.0, 120.0, 20.0, 8, 8, 20, 255); // row 4: cut col 29

    // Fold-face pixels (one per staircase step)
    icon_pix(&mut buf, SIZE, 26, 2, PS, 120, 136, 160);
    icon_pix(&mut buf, SIZE, 27, 3, PS, 120, 136, 160);
    icon_pix(&mut buf, SIZE, 28, 4, PS, 120, 136, 160);

    // Header bar: rows 3-4, cols 15-25
    fill_rect_f(&mut buf, SIZE, 60.0, 12.0, 104.0, 20.0, 136, 152, 176, 255);

    // Separator rule: row 6, cols 15-28
    fill_rect_f(&mut buf, SIZE, 60.0, 24.0, 116.0, 28.0, 154, 170, 187, 255);

    // GET row (rows 8-9, y=32..40)
    fill_rect_f(&mut buf, SIZE,  60.0, 32.0,  72.0, 40.0,  58, 128,  64, 255);
    fill_rect_f(&mut buf, SIZE,  76.0, 32.0, 112.0, 40.0,  96, 104, 120, 255);

    // POST row (rows 12-13, y=48..56)
    fill_rect_f(&mut buf, SIZE,  60.0, 48.0,  72.0, 56.0,  42,  80, 168, 255);
    fill_rect_f(&mut buf, SIZE,  76.0, 48.0, 112.0, 56.0,  96, 104, 120, 255);

    // DELETE row (rows 16-17, y=64..72)
    fill_rect_f(&mut buf, SIZE,  60.0, 64.0,  72.0, 72.0, 168,  48,  48, 255);
    fill_rect_f(&mut buf, SIZE,  76.0, 64.0, 112.0, 72.0,  96, 104, 120, 255);

    // PATCH row (rows 20-21, y=80..88)
    fill_rect_f(&mut buf, SIZE,  60.0, 80.0,  72.0, 88.0, 168,  96,  32, 255);
    fill_rect_f(&mut buf, SIZE,  76.0, 80.0, 112.0, 88.0,  96, 104, 120, 255);

    egui::IconData { rgba: buf, width: SIZE as u32, height: SIZE as u32 }
}

// ── Pixel-buffer helpers ──────────────────────────────────────────────────────

fn icon_pix(buf: &mut [u8], size: usize, gx: i32, gy: i32, ps: f32, r: u8, g: u8, b: u8) {
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
