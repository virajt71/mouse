#![allow(dead_code)]

use egui::{Color32, FontData, FontDefinitions, FontFamily};

// ── Light Theme Scale ────────────────────────────────────────────────────────
pub const LIGHT_BASE: Color32 = Color32::from_rgb(0xF7, 0xF7, 0xF6);
pub const LIGHT_APP_BG: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub const LIGHT_SURFACE: Color32 = Color32::from_rgb(0xF4, 0xF4, 0xF2);
pub const LIGHT_ELEVATED: Color32 = Color32::from_rgb(0xEB, 0xEB, 0xEA);
pub const LIGHT_HOVER: Color32 = Color32::from_rgb(0xE2, 0xE2, 0xE0);

pub const LIGHT_TEXT_PRIMARY: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
pub const LIGHT_TEXT_SECONDARY: Color32 = Color32::from_rgb(0x4A, 0x4A, 0x4A);
pub const LIGHT_TEXT_MUTED: Color32 = Color32::from_rgb(0x90, 0x90, 0x90);
pub const LIGHT_TEXT_DISABLED: Color32 = Color32::from_rgb(0xC4, 0xC4, 0xC4);

pub const LIGHT_ACCENT: Color32 = Color32::from_rgb(0x00, 0x89, 0x7B);
pub const LIGHT_ACCENT_DIM: Color32 = Color32::from_rgb(0xB2, 0xDF, 0xDB);
pub const LIGHT_DANGER: Color32 = Color32::from_rgb(0xD3, 0x2F, 0x2F);
pub const LIGHT_WARNING: Color32 = Color32::from_rgb(0xF5, 0x7C, 0x00);
pub const LIGHT_BORDER: Color32 = Color32::from_rgb(0xDC, 0xDC, 0xD8);

// ── Dark Theme Scale ─────────────────────────────────────────────────────────
pub const DARK_BASE: Color32 = Color32::from_rgb(0x07, 0x07, 0x08);
pub const DARK_APP_BG: Color32 = Color32::from_rgb(0x0C, 0x0C, 0x0D);
pub const DARK_SURFACE: Color32 = Color32::from_rgb(0x11, 0x11, 0x13);
pub const DARK_ELEVATED: Color32 = Color32::from_rgb(0x18, 0x18, 0x1B);
pub const DARK_HOVER: Color32 = Color32::from_rgb(0x20, 0x20, 0x24);

pub const DARK_TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF0, 0xF0, 0xF0);
pub const DARK_TEXT_SECONDARY: Color32 = Color32::from_rgb(0xA8, 0xA8, 0xA8);
pub const DARK_TEXT_MUTED: Color32 = Color32::from_rgb(0x60, 0x60, 0x60);
pub const DARK_TEXT_DISABLED: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);

pub const DARK_ACCENT: Color32 = Color32::from_rgb(0x00, 0xC8, 0xB0);
pub const DARK_ACCENT_DIM: Color32 = Color32::from_rgb(0x00, 0x6E, 0x60);
pub const DARK_DANGER: Color32 = Color32::from_rgb(0xFF, 0x4A, 0x4A);
pub const DARK_WARNING: Color32 = Color32::from_rgb(0xFF, 0x98, 0x00);
pub const DARK_BORDER: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x1E);

// ── Legacy Constants (Dark theme values, to keep references stable) ─────────
pub const COLOR_BG: Color32 = DARK_APP_BG;
pub const COLOR_SURFACE: Color32 = DARK_SURFACE;
pub const COLOR_TOP_BAR: Color32 = COLOR_BG;
pub const COLOR_ACCENT: Color32 = DARK_ACCENT;
pub const COLOR_ACCENT_DIM: Color32 = DARK_ACCENT_DIM;
pub const COLOR_DOT_RED: Color32 = DARK_DANGER;
pub const COLOR_DOT_DARK: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
pub const COLOR_INACTIVE_TEXT: Color32 = Color32::from_rgb(0x90, 0x90, 0x90);
pub const COLOR_DIVIDER: Color32 = Color32::from_rgb(0x22, 0x22, 0x22);
pub const COLOR_CARD_BORDER: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
pub const COLOR_FOOTER_ORANGE: Color32 = Color32::from_rgb(0xd9, 0xa7, 0x52);
pub const COLOR_SUBTLE_TEXT: Color32 = Color32::from_rgb(0x50, 0x50, 0x50);

// ── Window Sizing ────────────────────────────────────────────────────────────
pub const WINDOW_W: f32 = 1280.0;
pub const WINDOW_H: f32 = 720.0;

// ── Dynamic Theme Colors ──────────────────────────────────────────────────────
pub fn base_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_BASE
    } else {
        LIGHT_BASE
    }
}

pub fn app_bg(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_APP_BG
    } else {
        LIGHT_APP_BG
    }
}

pub fn surface_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_SURFACE
    } else {
        LIGHT_SURFACE
    }
}

pub fn elevated_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_ELEVATED
    } else {
        LIGHT_ELEVATED
    }
}

pub fn hover_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_HOVER
    } else {
        LIGHT_HOVER
    }
}

pub fn primary_text(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_TEXT_PRIMARY
    } else {
        LIGHT_TEXT_PRIMARY
    }
}

pub fn secondary_text(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_TEXT_SECONDARY
    } else {
        LIGHT_TEXT_SECONDARY
    }
}

pub fn muted_text(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_TEXT_MUTED
    } else {
        LIGHT_TEXT_MUTED
    }
}

pub fn disabled_text(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_TEXT_DISABLED
    } else {
        LIGHT_TEXT_DISABLED
    }
}

pub fn accent_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_ACCENT
    } else {
        LIGHT_ACCENT
    }
}

pub fn accent_dim_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_ACCENT_DIM
    } else {
        LIGHT_ACCENT_DIM
    }
}

pub fn danger_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_DANGER
    } else {
        LIGHT_DANGER
    }
}

pub fn warning_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_WARNING
    } else {
        LIGHT_WARNING
    }
}

pub fn border_color(ctx: &egui::Context) -> Color32 {
    if ctx.style().visuals.dark_mode {
        DARK_BORDER
    } else {
        LIGHT_BORDER
    }
}

// ── Compatibility Mappings ───────────────────────────────────────────────────
pub fn card_bg(ctx: &egui::Context) -> Color32 {
    surface_color(ctx)
}

pub fn card_border(ctx: &egui::Context) -> Color32 {
    border_color(ctx)
}

pub fn divider_color(ctx: &egui::Context) -> Color32 {
    border_color(ctx)
}

// ── Window Sizing Helpers ────────────────────────────────────────────────────
pub fn compute_window_size(monitor_size: egui::Vec2) -> egui::Vec2 {
    let screen_w = monitor_size.x;
    let screen_h = monitor_size.y;

    if screen_w > 2560.0 {
        egui::vec2(1792.0, 1008.0)
    } else {
        egui::vec2(screen_w * 0.70, screen_h * 0.70)
    }
}

pub fn center_window(ctx: &egui::Context, monitor: egui::Vec2) {
    let size = compute_window_size(monitor);
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));

    let pos_x = (monitor.x - size.x) / 2.0;
    let pos_y = (monitor.y - size.y) / 2.0;
    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
        pos_x, pos_y,
    )));
}

pub fn lerp_color(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    egui::Color32::from_rgba_unmultiplied(
        lerp(from.r(), to.r()),
        lerp(from.g(), to.g()),
        lerp(from.b(), to.b()),
        lerp(from.a(), to.a()),
    )
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "inter-regular".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.ttf")),
    );
    fonts.font_data.insert(
        "inter-bold".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/Inter-Bold.ttf")),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "inter-regular".to_owned());

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("inter-regular".to_owned());

    ctx.set_fonts(fonts);
}

pub fn draw_tech_corners(
    painter: &egui::Painter,
    rect: egui::Rect,
    color: egui::Color32,
    len: f32,
) {
    let stroke = egui::Stroke::new(1.0, color);
    // Top-left
    painter.line_segment(
        [rect.left_top(), rect.left_top() + egui::vec2(len, 0.0)],
        stroke,
    );
    painter.line_segment(
        [rect.left_top(), rect.left_top() + egui::vec2(0.0, len)],
        stroke,
    );
    // Top-right
    painter.line_segment(
        [rect.right_top(), rect.right_top() + egui::vec2(-len, 0.0)],
        stroke,
    );
    painter.line_segment(
        [rect.right_top(), rect.right_top() + egui::vec2(0.0, len)],
        stroke,
    );
    // Bottom-left
    painter.line_segment(
        [
            rect.left_bottom(),
            rect.left_bottom() + egui::vec2(len, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            rect.left_bottom(),
            rect.left_bottom() + egui::vec2(0.0, -len),
        ],
        stroke,
    );
    // Bottom-right
    painter.line_segment(
        [
            rect.right_bottom(),
            rect.right_bottom() + egui::vec2(-len, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            rect.right_bottom(),
            rect.right_bottom() + egui::vec2(0.0, -len),
        ],
        stroke,
    );
}
