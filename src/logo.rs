use egui::{Color32, FontId, Painter, Pos2, Stroke, Vec2, Align2};

// ── Brand palette (const so they can be used as default args) ─────────────────

const BG:          Color32 = Color32::from_rgb(22, 22, 42);
const WAND_DARK:   Color32 = Color32::from_rgb(100, 72, 20);
const WAND_MID:    Color32 = Color32::from_rgb(200, 168, 75);
const SPARKLE:     Color32 = Color32::from_rgb(255, 217, 64);
const SPARKLE_DIM: Color32 = Color32::from_rgba_premultiplied(255, 217, 64, 140);
const C_GET:       Color32 = Color32::from_rgb(74, 159, 90);
const C_POST:      Color32 = Color32::from_rgb(58, 111, 204);
const C_DELETE:    Color32 = Color32::from_rgb(184, 64, 64);
const BAR_BG:      Color32 = Color32::from_rgb(42, 42, 64);
const TEXT_DIM:    Color32 = Color32::from_rgb(144, 144, 184);
const DIVIDER:     Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 28);
const TRAIL:       Color32 = Color32::from_rgba_premultiplied(255, 217, 64, 45);

// ── Welcome-screen logo ───────────────────────────────────────────────────────

/// Draws the wand + API logo into a 300×170 area allocated inside `ui`.
pub fn draw_logo(ui: &mut egui::Ui) {
    let desired = Vec2::new(300.0, 170.0);
    let (resp, painter) = ui.allocate_painter(desired, egui::Sense::hover());
    let r = resp.rect;
    let w = r.width();
    let h = r.height();

    // Background
    painter.rect_filled(r, 10.0, BG);

    // ── Wand ─────────────────────────────────────────────────────────────────
    let base = r.min + Vec2::new(w * 0.08, h * 0.87);
    let tip  = r.min + Vec2::new(w * 0.30, h * 0.13);

    // Shaft — dark core + lighter overlay gives a two-tone depth
    painter.line_segment([base, tip], Stroke::new(w * 0.033, WAND_DARK));
    painter.line_segment([base, tip], Stroke::new(w * 0.016, WAND_MID));

    // ── Sparkle at wand tip ───────────────────────────────────────────────────
    draw_sparkle(&painter, tip, w * 0.063, SPARKLE);

    // Small satellite dots
    draw_dot(&painter, tip + Vec2::new( w * 0.077, -h * 0.082), w * 0.016, SPARKLE_DIM);
    draw_dot(&painter, tip + Vec2::new(-w * 0.058, -h * 0.072), w * 0.012, SPARKLE_DIM);
    draw_dot(&painter, tip + Vec2::new( w * 0.082,  h * 0.076), w * 0.009, SPARKLE_DIM);

    // ── Divider ───────────────────────────────────────────────────────────────
    let div_x = r.min.x + w * 0.44;
    painter.line_segment(
        [Pos2::new(div_x, r.min.y + h * 0.07), Pos2::new(div_x, r.min.y + h * 0.93)],
        Stroke::new(1.0, DIVIDER),
    );

    // ── Magic trail from tip toward the API section ───────────────────────────
    let trail_start = tip + Vec2::new(w * 0.025, h * 0.065);
    let trail_end   = Pos2::new(div_x - 4.0, r.min.y + h * 0.52);
    painter.line_segment([trail_start, trail_end], Stroke::new(0.9, TRAIL));

    // ── API Endpoint bars ─────────────────────────────────────────────────────
    let bar_x    = r.min.x + w * 0.46;
    let badge_w  = w * 0.195;
    let path_w   = w * 0.325;
    let bar_h    = h * 0.145;
    let gap      = h * 0.075;
    let rows_top = r.min.y + h * 0.175;
    let font_sz  = bar_h * 0.56;

    let endpoints: &[(&str, Color32, &str)] = &[
        ("GET",    C_GET,    "/users"),
        ("POST",   C_POST,   "/users"),
        ("DELETE", C_DELETE, "/users/{id}"),
    ];

    for (i, (method, color, path)) in endpoints.iter().enumerate() {
        let y = rows_top + i as f32 * (bar_h + gap);

        // Method badge
        let badge = egui::Rect::from_min_size(
            Pos2::new(bar_x, y),
            Vec2::new(badge_w, bar_h),
        );
        painter.rect_filled(badge, 3.0, *color);
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            *method,
            FontId::monospace(font_sz),
            Color32::WHITE,
        );

        // Path bar
        let path_rect = egui::Rect::from_min_size(
            Pos2::new(bar_x + badge_w + 2.0, y),
            Vec2::new(path_w, bar_h),
        );
        painter.rect_filled(path_rect, 3.0, BAR_BG);
        painter.text(
            path_rect.center(),
            Align2::CENTER_CENTER,
            *path,
            FontId::monospace(font_sz * 0.88),
            TEXT_DIM,
        );
    }
}

// ── Painter helpers ───────────────────────────────────────────────────────────

/// 8-arm sparkle (cross + X, thick arms + thin diagonals).
fn draw_sparkle(painter: &Painter, center: Pos2, size: f32, color: Color32) {
    let thick = Stroke::new(size * 0.40, color);
    let thin  = Stroke::new(size * 0.22, color);
    painter.line_segment([center - Vec2::new(size, 0.0), center + Vec2::new(size, 0.0)], thick);
    painter.line_segment([center - Vec2::new(0.0, size), center + Vec2::new(0.0, size)], thick);
    let d = size * 0.60;
    painter.line_segment([center - Vec2::new(d, d),  center + Vec2::new(d, d)],  thin);
    painter.line_segment([center + Vec2::new(-d, d), center + Vec2::new(d, -d)], thin);
}

fn draw_dot(painter: &Painter, pos: Pos2, r: f32, color: Color32) {
    painter.circle_filled(pos, r, color);
}

// ── Window icon ───────────────────────────────────────────────────────────────

/// Generates a 128×128 RGBA icon matching the logo's visual language.
pub fn make_icon() -> egui::IconData {
    const SIZE: usize = 128;
    let mut buf = vec![0u8; SIZE * SIZE * 4];
    let s = SIZE as f32;

    // Dark circular background
    fill_circle(&mut buf, SIZE, s * 0.5, s * 0.5, s * 0.47, 22, 22, 42, 255);

    // Wand shaft (dark core then lighter overlay)
    paint_line(&mut buf, SIZE, s*0.17, s*0.84, s*0.53, s*0.21, s*0.055, 100, 72, 20);
    paint_line(&mut buf, SIZE, s*0.17, s*0.84, s*0.53, s*0.21, s*0.026, 200, 168, 75);

    // Sparkle at tip (68, 27) in 128-space
    let tx = s * 0.53;
    let ty = s * 0.21;
    let sr = s * 0.115;
    // Horizontal arm
    paint_line(&mut buf, SIZE, tx - sr, ty,      tx + sr, ty,      s*0.042, 255, 217, 64);
    // Vertical arm
    paint_line(&mut buf, SIZE, tx,      ty - sr, tx,      ty + sr, s*0.042, 255, 217, 64);
    // Diagonal arms (thinner)
    let sd = sr * 0.62;
    paint_line(&mut buf, SIZE, tx - sd, ty - sd, tx + sd, ty + sd, s*0.024, 255, 217, 64);
    paint_line(&mut buf, SIZE, tx + sd, ty - sd, tx - sd, ty + sd, s*0.024, 255, 217, 64);

    // Satellite sparkle dots
    fill_circle(&mut buf, SIZE, tx + sr*1.4, ty - sr*0.7, s*0.018, 255, 217, 64, 200);
    fill_circle(&mut buf, SIZE, tx - sr*1.2, ty - sr*0.9, s*0.014, 255, 217, 64, 160);

    // Three API dots + stub lines (right side)
    let dot_x  = s * 0.73;
    let dot_r  = s * 0.055;
    let bar_x0 = s * 0.80;
    let bar_x1 = s * 0.91;
    let bar_t  = s * 0.025;

    // GET (green)
    fill_circle(&mut buf, SIZE, dot_x, s*0.34, dot_r, 74, 159, 90, 255);
    paint_line( &mut buf, SIZE, bar_x0, s*0.34, bar_x1, s*0.34, bar_t, 74, 159, 90);

    // POST (blue)
    fill_circle(&mut buf, SIZE, dot_x, s*0.52, dot_r, 58, 111, 204, 255);
    paint_line( &mut buf, SIZE, bar_x0, s*0.52, bar_x1, s*0.52, bar_t, 58, 111, 204);

    // DELETE (red)
    fill_circle(&mut buf, SIZE, dot_x, s*0.70, dot_r, 184, 64, 64, 255);
    paint_line( &mut buf, SIZE, bar_x0, s*0.70, bar_x1, s*0.70, bar_t, 184, 64, 64);

    egui::IconData { rgba: buf, width: SIZE as u32, height: SIZE as u32 }
}

// ── Pixel-buffer helpers ──────────────────────────────────────────────────────

fn fill_circle(buf: &mut [u8], size: usize, cx: f32, cy: f32, r: f32,
               red: u8, grn: u8, blu: u8, alpha: u8) {
    let x0 = ((cx - r).floor() as i32).max(0) as usize;
    let x1 = ((cx + r).ceil()  as i32).min(size as i32 - 1) as usize;
    let y0 = ((cy - r).floor() as i32).max(0) as usize;
    let y1 = ((cy + r).ceil()  as i32).min(size as i32 - 1) as usize;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r * r {
                let i = (y * size + x) * 4;
                buf[i] = red; buf[i+1] = grn; buf[i+2] = blu; buf[i+3] = alpha;
            }
        }
    }
}

/// Draws a thick line by stamping circles along its length (round caps).
fn paint_line(buf: &mut [u8], size: usize,
              x0: f32, y0: f32, x1: f32, y1: f32,
              thickness: f32, red: u8, grn: u8, blu: u8) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-6 { return; }
    let steps = (len * 1.5) as usize + 2;
    let r = thickness / 2.0;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        fill_circle(buf, size, x0 + t * dx, y0 + t * dy, r, red, grn, blu, 255);
    }
}
