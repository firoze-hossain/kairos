//! crates/kairos-ui/src/lib.rs
//!
//! # Kairos UI Engine (macOS + Ubuntu Synthesized)
//!
//! Sovereign 2D UI rasterizer, multi-view layout coordinator, and canvas
//! renderer for Kairos Calendar. Operates with zero external C dependencies,
//! rendering directly into raw pixel buffers.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use kairos_core::{
    days_in_month, day_of_week, format_date_long, CalendarState,
    EventCategory, ViewMode, DAY_NAMES, DAY_NAMES_SHORT, MONTH_NAMES, MONTH_NAMES_SHORT,
};

pub const DEFAULT_WINDOW_W: usize = 960;
pub const DEFAULT_WINDOW_H: usize = 640;

// ============================================================================
// 2D Geometric Primitives
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn blend(&self, bg: Color) -> Color {
        if self.a == 255 {
            return *self;
        }
        if self.a == 0 {
            return bg;
        }
        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;
        let r = ((self.r as u32 * alpha + bg.r as u32 * inv_alpha) / 255) as u8;
        let g = ((self.g as u32 * alpha + bg.g as u32 * inv_alpha) / 255) as u8;
        let b = ((self.b as u32 * alpha + bg.b as u32 * inv_alpha) / 255) as u8;
        Color::rgb(r, g, b)
    }
}

// ============================================================================
// Design System Theme Palette (macOS + Ubuntu Synthesis)
// ============================================================================

pub struct ThemePalette {
    pub bg: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub surface_hover: Color,
    pub border: Color,
    pub border_subtle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent_primary: Color,   // Apple Red #E11D48 / Royal Blue #2563EB
    pub accent_cyan: Color,      // Electric Cyan #06B6D4
    pub today_badge: Color,      // Iconic Red Circle #EF4444
    pub selected_ring: Color,    // Amber Ring #F59E0B
}

impl ThemePalette {
    pub fn dark() -> Self {
        Self {
            bg: Color::rgb(0x13, 0x15, 0x1C),
            surface: Color::rgb(0x1B, 0x1E, 0x28),
            surface_alt: Color::rgb(0x23, 0x27, 0x36),
            surface_hover: Color::rgb(0x2E, 0x34, 0x47),
            border: Color::rgb(0x35, 0x3C, 0x52),
            border_subtle: Color::rgb(0x27, 0x2C, 0x3C),
            text_primary: Color::rgb(0xF8, 0xFA, 0xFC),
            text_secondary: Color::rgb(0xCB, 0xD5, 0xE1),
            text_muted: Color::rgb(0x71, 0x7E, 0x95),
            accent_primary: Color::rgb(0x3B, 0x82, 0xF6),
            accent_cyan: Color::rgb(0x06, 0xB6, 0xD4),
            today_badge: Color::rgb(0xEF, 0x44, 0x44), // Apple Red Today Highlight
            selected_ring: Color::rgb(0xF5, 0x9E, 0x0B),
        }
    }
}

// ============================================================================
// Software Buffer Surface
// ============================================================================

pub struct CanvasSurface {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl CanvasSurface {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
        }
    }

    #[inline]
    pub fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = (y as usize) * self.width + (x as usize);
            if color.a == 255 {
                self.pixels[idx] = color.to_u32();
            } else {
                let current = self.pixels[idx];
                let bg = Color::rgb(
                    ((current >> 16) & 0xFF) as u8,
                    ((current >> 8) & 0xFF) as u8,
                    (current & 0xFF) as u8,
                );
                self.pixels[idx] = color.blend(bg).to_u32();
            }
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.clamp(0, self.width as i32);
        let y0 = rect.y.clamp(0, self.height as i32);
        let x1 = (rect.x + rect.w).clamp(0, self.width as i32);
        let y1 = (rect.y + rect.h).clamp(0, self.height as i32);

        for y in y0..y1 {
            let row_offset = (y as usize) * self.width;
            for x in x0..x1 {
                self.pixels[row_offset + (x as usize)] = color.to_u32();
            }
        }
    }

    pub fn fill_rounded_rect(&mut self, rect: Rect, r: i32, color: Color) {
        let r = r.clamp(0, rect.w / 2).min(rect.h / 2);
        if r <= 0 {
            self.fill_rect(rect, color);
            return;
        }

        let inner_rect = Rect::new(rect.x + r, rect.y, rect.w - 2 * r, rect.h);
        self.fill_rect(inner_rect, color);

        let left_rect = Rect::new(rect.x, rect.y + r, r, rect.h - 2 * r);
        let right_rect = Rect::new(rect.x + rect.w - r, rect.y + r, r, rect.h - 2 * r);
        self.fill_rect(left_rect, color);
        self.fill_rect(right_rect, color);

        let corners = [
            (rect.x + r, rect.y + r),
            (rect.x + rect.w - r - 1, rect.y + r),
            (rect.x + r, rect.y + rect.h - r - 1),
            (rect.x + rect.w - r - 1, rect.y + rect.h - r - 1),
        ];

        for &(cx, cy) in &corners {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r {
                        self.put_pixel(cx + dx, cy + dy, color);
                    }
                }
            }
        }
    }

    pub fn stroke_rounded_rect(&mut self, rect: Rect, r: i32, thickness: i32, color: Color) {
        let x0 = rect.x;
        let y0 = rect.y;
        let w = rect.w;
        let h = rect.h;

        // Top & Bottom
        self.fill_rect(Rect::new(x0 + r, y0, w - 2 * r, thickness), color);
        self.fill_rect(Rect::new(x0 + r, y0 + h - thickness, w - 2 * r, thickness), color);

        // Left & Right
        self.fill_rect(Rect::new(x0, y0 + r, thickness, h - 2 * r), color);
        self.fill_rect(Rect::new(x0 + w - thickness, y0 + r, thickness, h - 2 * r), color);
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let r2 = radius * radius;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= r2 {
                    self.put_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }

    /// Renders text using an embedded 5x8 ASCII monospace font.
    pub fn draw_text(&mut self, text: &str, mut x: i32, y: i32, color: Color) {
        for ch in text.bytes() {
            if ch >= 32 && ch <= 126 {
                let glyph = get_font_glyph(ch);
                for row in 0..8 {
                    let bits = glyph[row as usize];
                    for col in 0..5 {
                        if (bits & (1 << (4 - col))) != 0 {
                            self.put_pixel(x + col, y + row, color);
                        }
                    }
                }
            }
            x += 6;
        }
    }

    pub fn draw_text_bold(&mut self, text: &str, x: i32, y: i32, color: Color) {
        self.draw_text(text, x, y, color);
        self.draw_text(text, x + 1, y, color);
    }
}

// 5x8 Bitmap Font glyph lookup
fn get_font_glyph(ascii: u8) -> [u8; 8] {
    match ascii {
        b'0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E, 0x00],
        b'1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E, 0x00],
        b'2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F, 0x00],
        b'3' => [0x1F, 0x02, 0x04, 0x0E, 0x01, 0x11, 0x0E, 0x00],
        b'4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02, 0x00],
        b'5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E, 0x00],
        b'6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E, 0x00],
        b'7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08, 0x00],
        b'8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E, 0x00],
        b'9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C, 0x00],
        b':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00, 0x00],
        b'-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00, 0x00],
        b'/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10, 0x00],
        b'<' => [0x02, 0x04, 0x08, 0x10, 0x08, 0x04, 0x02, 0x00],
        b'>' => [0x08, 0x04, 0x02, 0x01, 0x02, 0x04, 0x08, 0x00],
        b'+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00, 0x00],
        b'A' | b'a' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11, 0x00],
        b'B' | b'b' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E, 0x00],
        b'C' | b'c' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E, 0x00],
        b'D' | b'd' => [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C, 0x00],
        b'E' | b'e' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F, 0x00],
        b'F' | b'f' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10, 0x00],
        b'G' | b'g' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F, 0x00],
        b'H' | b'h' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11, 0x00],
        b'I' | b'i' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E, 0x00],
        b'J' | b'j' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C, 0x00],
        b'K' | b'k' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11, 0x00],
        b'L' | b'l' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F, 0x00],
        b'M' | b'm' => [0x11, 0x1B, 0x15, 0x11, 0x11, 0x11, 0x11, 0x00],
        b'N' | b'n' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11, 0x00],
        b'O' | b'o' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E, 0x00],
        b'P' | b'p' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10, 0x00],
        b'Q' | b'q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D, 0x00],
        b'R' | b'r' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11, 0x00],
        b'S' | b's' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E, 0x00],
        b'T' | b't' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x00],
        b'U' | b'u' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E, 0x00],
        b'V' | b'v' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04, 0x00],
        b'W' | b'w' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11, 0x00],
        b'X' | b'x' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11, 0x00],
        b'Y' | b'y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04, 0x00],
        b'Z' | b'z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F, 0x00],
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }
}

// ============================================================================
// Kairos UI Master Renderer
// ============================================================================

pub struct KairosUiRenderer;

impl KairosUiRenderer {
    pub fn render(surface: &mut CanvasSurface, state: &CalendarState) {
        let p = ThemePalette::dark();
        let w = surface.width as i32;
        let h = surface.height as i32;

        // 1. Clear desktop background
        surface.fill_rect(Rect::new(0, 0, w, h), p.bg);

        let top_h = 46;
        let bottom_h = 24;
        let content_h = h - top_h - bottom_h;

        // 2. Render Top Header Bar (macOS Toolbar + GNOME Segmented Buttons)
        Self::render_header_bar(surface, state, &p, top_h);

        // 3. Layout: 3-pane split
        let sidebar_w = 210;
        let right_pane_w = 240;
        let center_w = w - sidebar_w - right_pane_w;

        let sidebar_rect = Rect::new(0, top_h, sidebar_w, content_h);
        let center_rect = Rect::new(sidebar_w, top_h, center_w, content_h);
        let right_rect = Rect::new(sidebar_w + center_w, top_h, right_pane_w, content_h);

        // Sidebar
        Self::render_sidebar(surface, state, &p, sidebar_rect);

        // Center Viewports
        match state.view_mode {
            ViewMode::Month => Self::render_month_grid(surface, state, &p, center_rect),
            ViewMode::Week => Self::render_week_view(surface, state, &p, center_rect),
            ViewMode::Day => Self::render_day_view(surface, state, &p, center_rect),
            ViewMode::Agenda => Self::render_agenda_view(surface, state, &p, center_rect),
            ViewMode::Year => Self::render_year_matrix(surface, state, &p, center_rect),
        }

        // Right Inspector Pane
        Self::render_inspector(surface, state, &p, right_rect);

        // 4. Status Bar at Bottom
        let status_rect = Rect::new(0, h - bottom_h, w, bottom_h);
        Self::render_status_bar(surface, state, &p, status_rect);
    }

    fn render_header_bar(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, h: i32) {
        let w = surface.width as i32;
        surface.fill_rect(Rect::new(0, 0, w, h), p.surface);
        surface.fill_rect(Rect::new(0, h - 1, w, 1), p.border);

        // Left: Navigation [<] [Today] [>]
        let btn_prev = Rect::new(14, 11, 26, 24);
        surface.fill_rounded_rect(btn_prev, 4, p.surface_alt);
        surface.stroke_rounded_rect(btn_prev, 4, 1, p.border);
        surface.draw_text("<", 24, 19, p.text_primary);

        let btn_today = Rect::new(46, 11, 52, 24);
        surface.fill_rounded_rect(btn_today, 4, p.surface_alt);
        surface.stroke_rounded_rect(btn_today, 4, 1, p.border);
        surface.draw_text_bold("Today", 54, 19, p.text_primary);

        let btn_next = Rect::new(104, 11, 26, 24);
        surface.fill_rounded_rect(btn_next, 4, p.surface_alt);
        surface.stroke_rounded_rect(btn_next, 4, 1, p.border);
        surface.draw_text(">", 114, 19, p.text_primary);

        // Center-Left: Month and Year Title (e.g., "November 2026")
        let title = format!(
            "{} {}",
            MONTH_NAMES[(state.view_month.clamp(1, 12) - 1) as usize],
            state.view_year
        );
        surface.draw_text_bold(&title, 144, 19, p.text_primary);

        // Center: macOS-style Segmented View Switcher: [ Month | Week | Day | Agenda | Year ]
        let seg_w = 320;
        let seg_x = (w - seg_w) / 2;
        let seg_rect = Rect::new(seg_x, 10, seg_w, 26);
        surface.fill_rounded_rect(seg_rect, 6, p.surface_alt);
        surface.stroke_rounded_rect(seg_rect, 6, 1, p.border);

        let item_w = seg_w / 5;
        for (i, mode) in ViewMode::all().iter().enumerate() {
            let item_rect = Rect::new(seg_x + (i as i32) * item_w + 1, 11, item_w - 2, 24);
            if *mode == state.view_mode {
                surface.fill_rounded_rect(item_rect, 5, p.accent_primary);
                surface.draw_text_bold(mode.name(), item_rect.x + (item_w - 24) / 2 - 8, 19, Color::rgb(255, 255, 255));
            } else {
                surface.draw_text(mode.name(), item_rect.x + (item_w - 24) / 2 - 8, 19, p.text_secondary);
            }
        }

        // Right side: [+ Event] button (Ubuntu GNOME primary action style)
        let add_btn = Rect::new(w - 100, 11, 88, 24);
        surface.fill_rounded_rect(add_btn, 4, Color::rgb(0x16, 0xA3, 0x4A)); // Forest Green
        surface.stroke_rounded_rect(add_btn, 4, 1, Color::rgb(0x22, 0xC5, 0x5E));
        surface.draw_text_bold("+ Event", w - 82, 19, Color::rgb(255, 255, 255));
    }

    fn render_sidebar(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, rect: Rect) {
        surface.fill_rect(rect, p.surface);
        surface.fill_rect(Rect::new(rect.x + rect.w - 1, rect.y, 1, rect.h), p.border);

        // 1. Hero Date Badge (e.g. "24" in large card, "Tuesday, Nov 24, 2026")
        let hero_rect = Rect::new(rect.x + 12, rect.y + 12, rect.w - 24, 72);
        surface.fill_rounded_rect(hero_rect, 6, p.surface_alt);
        surface.stroke_rounded_rect(hero_rect, 6, 1, p.border);

        let day_str = format!("{:02}", state.now_day);
        surface.draw_text_bold(&day_str, hero_rect.x + 16, hero_rect.y + 18, p.today_badge);

        let m_name = MONTH_NAMES_SHORT[(state.now_month.clamp(1, 12) - 1) as usize];
        let sub_str = format!("{}, {}", m_name, state.now_year);
        surface.draw_text_bold(&sub_str, hero_rect.x + 46, hero_rect.y + 18, p.text_primary);

        let dow = day_of_week(state.now_year, state.now_month, state.now_day);
        let dow_str = DAY_NAMES[(dow % 7) as usize];
        surface.draw_text(dow_str, hero_rect.x + 46, hero_rect.y + 36, p.text_muted);

        // 2. Calendars / Categories list header
        surface.draw_text_bold("CALENDARS", rect.x + 14, rect.y + 102, p.text_muted);

        let cat_y0 = rect.y + 120;
        for (i, cat) in EventCategory::all().iter().enumerate() {
            let cy = cat_y0 + (i as i32) * 26;
            if cy + 24 > rect.y + rect.h {
                break;
            }

            let is_enabled = state.is_category_enabled(*cat);
            let (cr, cg, cb) = cat.color_rgb();
            let cat_color = Color::rgb(cr, cg, cb);

            // Checkbox pill
            let chk_rect = Rect::new(rect.x + 14, cy + 4, 14, 14);
            if is_enabled {
                surface.fill_rounded_rect(chk_rect, 3, cat_color);
            } else {
                surface.stroke_rounded_rect(chk_rect, 3, 1, p.border);
            }

            // Category name & count
            let count = state.events.iter().filter(|e| e.category == *cat).count();
            let text_color = if is_enabled { p.text_primary } else { p.text_muted };
            surface.draw_text(cat.name(), rect.x + 36, cy + 7, text_color);

            let count_str = format!("{}", count);
            surface.draw_text(&count_str, rect.x + rect.w - 28, cy + 7, p.text_muted);
        }

        // 3. Mini Month Calendar Matrix at bottom of sidebar
        let mini_y = rect.y + rect.h - 180;
        if mini_y > cat_y0 + (EventCategory::all().len() as i32) * 26 + 20 {
            surface.fill_rect(Rect::new(rect.x + 10, mini_y - 10, rect.w - 20, 1), p.border);
            let mini_title = format!("{} {}", MONTH_NAMES_SHORT[(state.view_month.clamp(1, 12) - 1) as usize], state.view_year);
            surface.draw_text_bold(&mini_title, rect.x + 14, mini_y, p.text_secondary);

            let cell_w = (rect.w - 24) / 7;
            for (col, d) in DAY_NAMES_SHORT.iter().enumerate() {
                surface.draw_text(&d[0..1], rect.x + 12 + (col as i32) * cell_w + 4, mini_y + 18, p.text_muted);
            }

            let start_dow = day_of_week(state.view_year, state.view_month, 1);
            let max_d = days_in_month(state.view_year, state.view_month);
            for d in 1..=max_d {
                let cell_idx = (start_dow as usize) + (d as usize) - 1;
                let row = (cell_idx / 7) as i32;
                let col = (cell_idx % 7) as i32;
                let cx = rect.x + 12 + col * cell_w;
                let cy = mini_y + 32 + row * 16;
                if cy + 14 < rect.y + rect.h {
                    if d == state.selected_day {
                        surface.fill_rounded_rect(Rect::new(cx + 1, cy - 2, cell_w - 2, 14), 2, p.accent_primary);
                        surface.draw_text(&format!("{:2}", d), cx + 2, cy, Color::rgb(255, 255, 255));
                    } else if d == state.now_day && state.view_month == state.now_month && state.view_year == state.now_year {
                        surface.fill_circle(cx + cell_w / 2, cy + 4, 6, p.today_badge);
                        surface.draw_text(&format!("{:2}", d), cx + 2, cy, Color::rgb(255, 255, 255));
                    } else {
                        surface.draw_text(&format!("{:2}", d), cx + 2, cy, p.text_secondary);
                    }
                }
            }
        }
    }

    fn render_month_grid(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, rect: Rect) {
        let col_w = rect.w / 7;
        let dow_h = 24;

        // Day of Week Header row
        surface.fill_rect(Rect::new(rect.x, rect.y, rect.w, dow_h), p.surface_alt);
        surface.fill_rect(Rect::new(rect.x, rect.y + dow_h - 1, rect.w, 1), p.border);

        for (col, name) in DAY_NAMES.iter().enumerate() {
            let cx = rect.x + (col as i32) * col_w;
            surface.draw_text_bold(name, cx + 12, rect.y + 8, p.text_muted);
        }

        let grid_y = rect.y + dow_h;
        let grid_h = rect.h - dow_h;
        let row_h = grid_h / 6;

        let start_dow = day_of_week(state.view_year, state.view_month, 1);
        let max_days = days_in_month(state.view_year, state.view_month);

        for row in 0..6 {
            let ry = grid_y + row * row_h;
            surface.fill_rect(Rect::new(rect.x, ry + row_h - 1, rect.w, 1), p.border_subtle);

            for col in 0..7 {
                let rx = rect.x + col * col_w;
                surface.fill_rect(Rect::new(rx + col_w - 1, ry, 1, row_h), p.border_subtle);

                let cell_idx = (row * 7 + col) as i32;
                let day_num = cell_idx - (start_dow as i32) + 1;

                if day_num >= 1 && day_num <= max_days as i32 {
                    let d = day_num as u8;
                    let is_today = d == state.now_day && state.view_month == state.now_month && state.view_year == state.now_year;
                    let is_selected = d == state.selected_day;

                    if is_selected {
                        surface.fill_rect(Rect::new(rx + 1, ry + 1, col_w - 2, row_h - 2), p.surface_hover);
                    }

                    // Day Number badge
                    if is_today {
                        surface.fill_circle(rx + 18, ry + 14, 10, p.today_badge);
                        surface.draw_text_bold(&format!("{}", d), rx + 14, ry + 10, Color::rgb(255, 255, 255));
                    } else {
                        surface.draw_text_bold(&format!("{}", d), rx + 10, ry + 10, p.text_primary);
                    }

                    // Render event pill indicators
                    let events = state.events_for_date(state.view_year, state.view_month, d);
                    let mut pill_y = ry + 28;
                    for ev in events.iter().take(3) {
                        let (cr, cg, cb) = ev.category.color_rgb();
                        let pill_rect = Rect::new(rx + 6, pill_y, col_w - 12, 16);
                        surface.fill_rounded_rect(pill_rect, 3, Color::rgba(cr, cg, cb, 40));
                        surface.stroke_rounded_rect(pill_rect, 3, 1, Color::rgb(cr, cg, cb));

                        let max_chars = ((col_w - 18) / 6).max(2) as usize;
                        let trunc_title = if ev.title.len() > max_chars {
                            &ev.title[0..max_chars]
                        } else {
                            &ev.title
                        };
                        surface.draw_text(trunc_title, rx + 10, pill_y + 4, Color::rgb(cr, cg, cb));
                        pill_y += 18;
                    }
                    if events.len() > 3 {
                        let more_str = format!("+{} more", events.len() - 3);
                        surface.draw_text(&more_str, rx + 10, pill_y + 2, p.text_muted);
                    }
                }
            }
        }
    }

    fn render_week_view(surface: &mut CanvasSurface, _state: &CalendarState, p: &ThemePalette, rect: Rect) {
        surface.fill_rect(rect, p.surface);
        let col_w = (rect.w - 50) / 7;
        let time_col_w = 50;

        // Header with 7 days
        surface.fill_rect(Rect::new(rect.x, rect.y, rect.w, 36), p.surface_alt);
        surface.fill_rect(Rect::new(rect.x, rect.y + 35, rect.w, 1), p.border);

        for col in 0..7 {
            let cx = rect.x + time_col_w + col * col_w;
            surface.draw_text(DAY_NAMES_SHORT[col as usize], cx + 8, rect.y + 8, p.text_muted);
            let d_num = format!("{:02}", col + 18);
            surface.draw_text_bold(&d_num, cx + 8, rect.y + 20, p.text_primary);
        }

        // Hourly rows
        let row_h = (rect.h - 36) / 12;
        for h in 0..12 {
            let ry = rect.y + 36 + h * row_h;
            surface.fill_rect(Rect::new(rect.x, ry + row_h - 1, rect.w, 1), p.border_subtle);
            let hour_str = format!("{:02}:00", h + 8);
            surface.draw_text(&hour_str, rect.x + 8, ry + 6, p.text_muted);
        }
    }

    fn render_day_view(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, rect: Rect) {
        surface.fill_rect(rect, p.surface);
        let head_h = 48;
        surface.fill_rect(Rect::new(rect.x, rect.y, rect.w, head_h), p.surface_alt);
        surface.fill_rect(Rect::new(rect.x, rect.y + head_h - 1, rect.w, 1), p.border);

        let date_str = format_date_long(state.view_year, state.view_month, state.selected_day);
        surface.draw_text_bold(&date_str, rect.x + 20, rect.y + 18, p.text_primary);

        let events = state.events_for_selected_date();
        let mut ey = rect.y + head_h + 16;
        for ev in events {
            if ey + 64 > rect.y + rect.h {
                break;
            }
            let card_rect = Rect::new(rect.x + 20, ey, rect.w - 40, 54);
            surface.fill_rounded_rect(card_rect, 6, p.surface_alt);
            let (cr, cg, cb) = ev.category.color_rgb();
            surface.fill_rect(Rect::new(card_rect.x, card_rect.y, 4, card_rect.h), Color::rgb(cr, cg, cb));

            surface.draw_text_bold(&ev.title, card_rect.x + 16, card_rect.y + 10, p.text_primary);
            let time_loc = format!("{} - {} | {}", ev.start_time, ev.end_time, ev.location);
            surface.draw_text(&time_loc, card_rect.x + 16, card_rect.y + 26, p.text_muted);

            ey += 64;
        }
    }

    fn render_agenda_view(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, rect: Rect) {
        surface.fill_rect(rect, p.surface);
        surface.draw_text_bold("CHRONOLOGICAL AGENDA FEED", rect.x + 20, rect.y + 16, p.text_muted);

        let events = state.events_for_month(state.view_year, state.view_month);
        let mut y = rect.y + 42;
        for ev in events {
            if y + 48 > rect.y + rect.h {
                break;
            }
            let card_rect = Rect::new(rect.x + 20, y, rect.w - 40, 42);
            surface.fill_rounded_rect(card_rect, 4, p.surface_alt);
            let (cr, cg, cb) = ev.category.color_rgb();
            surface.fill_circle(card_rect.x + 14, card_rect.y + 21, 5, Color::rgb(cr, cg, cb));

            let date_badge = format!("{:02}/{:02}", ev.month, ev.day);
            surface.draw_text_bold(&date_badge, card_rect.x + 28, card_rect.y + 16, p.text_primary);
            surface.draw_text_bold(&ev.title, card_rect.x + 80, card_rect.y + 16, p.text_primary);
            surface.draw_text(&ev.start_time, card_rect.x + card_rect.w - 80, card_rect.y + 16, p.text_muted);

            y += 48;
        }
    }

    fn render_year_matrix(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, rect: Rect) {
        surface.fill_rect(rect, p.surface);
        let cell_w = rect.w / 4;
        let cell_h = rect.h / 3;

        for m in 1..=12 {
            let row = ((m - 1) / 4) as i32;
            let col = ((m - 1) % 4) as i32;
            let mx = rect.x + col * cell_w;
            let my = rect.y + row * cell_h;

            surface.stroke_rounded_rect(Rect::new(mx + 6, my + 6, cell_w - 12, cell_h - 12), 4, 1, p.border_subtle);
            surface.draw_text_bold(MONTH_NAMES[(m - 1) as usize], mx + 16, my + 14, p.text_primary);

            let ev_count = state.events_for_month(state.view_year, m).len();
            let count_str = format!("{} events", ev_count);
            surface.draw_text(&count_str, mx + 16, my + 30, p.text_muted);
        }
    }

    fn render_inspector(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, rect: Rect) {
        surface.fill_rect(rect, p.surface_alt);
        surface.fill_rect(Rect::new(rect.x, rect.y, 1, rect.h), p.border);

        surface.draw_text_bold("EVENT INSPECTOR", rect.x + 16, rect.y + 16, p.text_muted);

        let selected = state
            .selected_event_id
            .and_then(|id| state.events.iter().find(|e| e.id == id))
            .or_else(|| state.events.first());

        if let Some(ev) = selected {
            let card_rect = Rect::new(rect.x + 14, rect.y + 40, rect.w - 28, rect.h - 60);
            surface.fill_rounded_rect(card_rect, 6, p.surface);
            surface.stroke_rounded_rect(card_rect, 6, 1, p.border);

            let (cr, cg, cb) = ev.category.color_rgb();
            let cat_pill = Rect::new(card_rect.x + 14, card_rect.y + 14, 100, 20);
            surface.fill_rounded_rect(cat_pill, 4, Color::rgba(cr, cg, cb, 50));
            surface.draw_text_bold(ev.category.name(), cat_pill.x + 8, cat_pill.y + 6, Color::rgb(cr, cg, cb));

            surface.draw_text_bold(&ev.title, card_rect.x + 14, card_rect.y + 44, p.text_primary);

            let time_str = format!("{} - {}", ev.start_time, ev.end_time);
            surface.draw_text(&time_str, card_rect.x + 14, card_rect.y + 64, p.text_secondary);

            surface.draw_text_bold("LOCATION", card_rect.x + 14, card_rect.y + 92, p.text_muted);
            surface.draw_text(&ev.location, card_rect.x + 14, card_rect.y + 106, p.text_primary);

            surface.draw_text_bold("NOTES", card_rect.x + 14, card_rect.y + 132, p.text_muted);
            surface.draw_text(&ev.description, card_rect.x + 14, card_rect.y + 148, p.text_secondary);

            let rep_str = format!("Repeat: {}", ev.recurrence.name());
            surface.draw_text(&rep_str, card_rect.x + 14, card_rect.y + 180, p.text_muted);
        }
    }

    fn render_status_bar(surface: &mut CanvasSurface, state: &CalendarState, p: &ThemePalette, rect: Rect) {
        surface.fill_rect(rect, p.surface);
        surface.fill_rect(Rect::new(rect.x, rect.y, rect.w, 1), p.border);

        let total_events = state.events.len();
        let stat_text = format!("Total: {} events | Timezone: {}", total_events, state.timezone_name);
        surface.draw_text(&stat_text, rect.x + 14, rect.y + 8, p.text_muted);

        let ver_str = "Kairos Calendar v1.0.0 (macOS + Ubuntu Synthesized)";
        surface.draw_text(ver_str, rect.x + rect.w - 380, rect.y + 8, p.text_muted);
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_primitives() {
        let mut canvas = CanvasSurface::new(100, 100);
        canvas.fill_rect(Rect::new(10, 10, 20, 20), Color::rgb(255, 0, 0));
        assert_eq!(canvas.pixels[10 * 100 + 10], 0x00FF0000);

        canvas.fill_circle(50, 50, 10, Color::rgb(0, 255, 0));
        assert_eq!(canvas.pixels[50 * 100 + 50], 0x0000FF00);
    }

    #[test]
    fn test_all_views_rasterization() {
        let state = CalendarState::new(2026, 11, 24);
        let mut canvas = CanvasSurface::new(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H);

        for mode in ViewMode::all() {
            let mut s = state.clone();
            s.view_mode = mode;
            KairosUiRenderer::render(&mut canvas, &s);
            let non_zero = canvas.pixels.iter().filter(|&&px| px != 0).count();
            assert!(non_zero > 10000, "ViewMode {:?} must draw pixels", mode);
        }
    }
}
