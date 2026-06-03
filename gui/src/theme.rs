#![allow(dead_code)]

use egui::{Color32, FontData, FontDefinitions, FontFamily};
use tray_icon::Icon;

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

fn line_dist(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let l2 = dx * dx + dy * dy;
    if l2 == 0.0 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / l2;
    let t = t.clamp(0.0, 1.0);
    let proj_x = ax + t * dx;
    let proj_y = ay + t * dy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

pub fn create_bluetooth_tray_icon() -> Icon {
    let width = 32u32;
    let height = 32u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let cx = 15.5f32;
    let cy = 15.5f32;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let idx = (((y * width) + x) * 4) as usize;

            let dist = (dx * dx + dy * dy).sqrt();
            let circle_alpha = if dist <= 12.0 {
                255
            } else if dist >= 13.5 {
                0
            } else {
                ((13.5 - dist) / 1.5 * 255.0) as u8
            };

            let mut min_line_dist = f32::MAX;
            for &(ax, ay, bx, by) in &[
                (15.5, 6.5, 15.5, 24.5), // vertical
                (15.5, 15.5, 21.0, 11.0), // upper diag 1
                (21.0, 11.0, 15.5, 6.5),  // upper diag 2
                (15.5, 15.5, 21.0, 20.0), // lower diag 1
                (21.0, 20.0, 15.5, 24.5), // lower diag 2
                (15.5, 11.0, 10.0, 6.5),  // upper ear
                (15.5, 20.0, 10.0, 24.5),  // lower ear
            ] {
                min_line_dist = min_line_dist.min(line_dist(x as f32, y as f32, ax, ay, bx, by));
            }

            let line_alpha = if min_line_dist <= 0.75 {
                255
            } else if min_line_dist >= 1.75 {
                0
            } else {
                ((1.75 - min_line_dist) * 255.0) as u8
            };

            let bg_r = 0.0f32;
            let bg_g = 122.0f32;
            let bg_b = 255.0f32;

            let a_circle = circle_alpha as f32 / 255.0;
            let a_line = line_alpha as f32 / 255.0;

            let a_out = a_line + a_circle * (1.0 - a_line);
            if a_out > 0.0 {
                let r_out = (255.0 * a_line + bg_r * a_circle * (1.0 - a_line)) / a_out;
                let g_out = (255.0 * a_line + bg_g * a_circle * (1.0 - a_line)) / a_out;
                let b_out = (255.0 * a_line + bg_b * a_circle * (1.0 - a_line)) / a_out;

                rgba[idx] = r_out.round() as u8;
                rgba[idx + 1] = g_out.round() as u8;
                rgba[idx + 2] = b_out.round() as u8;
                rgba[idx + 3] = (a_out * 255.0).round() as u8;
            } else {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }
    Icon::from_rgba(rgba, width, height).unwrap()
}

pub fn create_mouse_tray_icon() -> Icon {
    let width = 32u32;
    let height = 32u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    let cx = 15.5f32; // Center offset slightly to align on grid
    let cy = 15.5f32;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let idx = (((y * width) + x) * 4) as usize;

            // Tapered width based on vertical position (narrower top, wider palm)
            let w_y = 7.5f32 - 1.0f32 * (dy / 11.0f32);

            // Normalized coordinates for squircle shape
            let dx_norm = dx / w_y;
            let dy_norm = dy / 11.0f32;
            let d_val = dx_norm.powi(4) + dy_norm.powi(4);
            let val = d_val.powf(0.25f32);

            // Anti-aliased outer mouse body edge
            let edge = (val - 1.0f32) * 11.0f32;
            let alpha = if edge <= -0.5f32 {
                255
            } else if edge >= 0.5f32 {
                0
            } else {
                ((0.5f32 - edge) * 255.0f32) as u8
            };

            if alpha > 0 {
                // Scroll wheel: vertical pill segment from (0.0, -8.0) to (0.0, -3.0)
                let wheel_x = 0.0f32;
                let wheel_y_min = -7.5f32;
                let wheel_y_max = -2.5f32;

                let t = ((dy - wheel_y_min) / (wheel_y_max - wheel_y_min)).clamp(0.0f32, 1.0f32);
                let proj_y = wheel_y_min + t * (wheel_y_max - wheel_y_min);
                let w_dx = dx - wheel_x;
                let w_dy = dy - proj_y;
                let dist_to_wheel = (w_dx * w_dx + w_dy * w_dy).sqrt();

                // Scroll wheel shape & gap mask around it
                let wheel_val = dist_to_wheel - 1.0f32;
                let gap_val = dist_to_wheel - 2.0f32;

                let wheel_alpha = (0.5f32 - wheel_val).clamp(0.0f32, 1.0f32);
                let wheel_gap_mask = (gap_val + 0.5f32).clamp(0.0f32, 1.0f32);

                // Cutout masks for buttons:
                let mut gap_mask = 1.0f32;

                // 1. Vertical button divider (dy < -1.0, |dx| <= 0.6)
                if dy < -1.0f32 {
                    let dist = dx.abs();
                    let edge_dist = dist - 0.6f32;
                    let factor = (edge_dist + 0.5f32).clamp(0.0f32, 1.0f32);
                    gap_mask = gap_mask.min(factor);
                }

                // 2. Horizontal button separation arc (dy close to -1.0, |dx| <= w_y * 0.9)
                let horizontal_line_factor = {
                    let dist_to_line = (dy - (-1.0f32)).abs();
                    let edge_dist = dist_to_line - 0.6f32;
                    let line_intensity = (edge_dist + 0.5f32).clamp(0.0f32, 1.0f32);
                    // Smooth transition at the ends of the horizontal line
                    let end_dist = w_y * 0.9f32 - dx.abs();
                    let end_intensity = end_dist.clamp(0.0f32, 1.0f32);
                    line_intensity * end_intensity + (1.0f32 - end_intensity)
                };
                gap_mask = gap_mask.min(horizontal_line_factor);

                // Apply all gap masks to body alpha
                let final_body_alpha = (alpha as f32) * gap_mask * wheel_gap_mask;

                if wheel_alpha > 0.0f32 {
                    // Blend scroll wheel (White) on top of the mouse body/background
                    let r_body = 0.0f32;
                    let g_body = 191.0f32;
                    let b_body = 165.0f32;

                    let a_wheel = wheel_alpha;
                    let a_body = final_body_alpha / 255.0f32;

                    let a_out = a_wheel + a_body * (1.0f32 - a_wheel);
                    if a_out > 0.0f32 {
                        let r_out =
                            (255.0f32 * a_wheel + r_body * a_body * (1.0f32 - a_wheel)) / a_out;
                        let g_out =
                            (255.0f32 * a_wheel + g_body * a_body * (1.0f32 - a_wheel)) / a_out;
                        let b_out =
                            (255.0f32 * a_wheel + b_body * a_body * (1.0f32 - a_wheel)) / a_out;

                        rgba[idx] = r_out.round() as u8;
                        rgba[idx + 1] = g_out.round() as u8;
                        rgba[idx + 2] = b_out.round() as u8;
                        rgba[idx + 3] = (a_out * 255.0f32).round() as u8;
                    } else {
                        rgba[idx] = 0;
                        rgba[idx + 1] = 0;
                        rgba[idx + 2] = 0;
                        rgba[idx + 3] = 0;
                    }
                } else if final_body_alpha > 0.0f32 {
                    // Draw mouse body (Teal: #00BFA5)
                    rgba[idx] = 0;
                    rgba[idx + 1] = 191;
                    rgba[idx + 2] = 165;
                    rgba[idx + 3] = final_body_alpha.round() as u8;
                } else {
                    rgba[idx] = 0;
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 0;
                    rgba[idx + 3] = 0;
                }
            } else {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }
    Icon::from_rgba(rgba, width, height).unwrap()
}
