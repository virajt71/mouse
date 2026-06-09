use eframe::egui;
use egui::{vec2, Color32, Rect, Stroke, pos2};
use mouser_engine::config::Config;
use mouser_engine::Engine;
use crate::translation::tr;
use crate::theme;
use crate::widgets::{draw_rgb_palette_icon, draw_tech_corners};
use super::{section_card, render_spaced_header};

pub fn render_section_theme(ui: &mut egui::Ui, _ctx: &egui::Context, config: &mut Config, engine: &Engine) {
    section_card(ui, |ui| {
        // ── Section header ──
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            draw_rgb_palette_icon(ui, icon_rect);
            ui.add_space(6.0);
            render_spaced_header(
                ui,
                tr("theme_label", &config.settings.language),
                14.0,
                theme::primary_text(ui.ctx()),
            );
        });

        ui.add_space(14.0);

        let total_w = ui.available_width();
        let gap = 12.0;
        let card_w = ((total_w - 2.0 * gap) / 3.0).max(140.0);
        let card_h = 170.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);

            // Card 1: Follow OS Theme
            let label1 = tr("follow_os", &config.settings.language);
            if draw_theme_card(
                ui,
                card_w,
                card_h,
                label1,
                config.settings.appearance_mode == "system",
                |painter, rect| {
                    // Split background: left periwinkle, right navy
                    let mid_x = rect.center().x;
                    let left_bg = Rect::from_min_max(rect.min, pos2(mid_x, rect.max.y));
                    let right_bg = Rect::from_min_max(pos2(mid_x, rect.min.y), rect.max);
                    painter.rect_filled(
                        left_bg,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 0.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0xb5, 0xc4, 0xf5),
                    );
                    painter.rect_filled(
                        right_bg,
                        egui::Rounding {
                            nw: 0.0,
                            ne: 2.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0x14, 0x19, 0x23),
                    );

                    // Draw mock panel in the middle
                    let panel_w = 110.0;
                    let panel_h = 80.0;
                    let panel_rect = Rect::from_center_size(
                        pos2(rect.center().x, rect.bottom() - 10.0),
                        vec2(panel_w, panel_h),
                    );

                    // Left half of panel (White, rounded top-left)
                    let panel_mid_x = panel_rect.center().x;
                    let left_panel =
                        Rect::from_min_max(panel_rect.min, pos2(panel_mid_x, panel_rect.max.y));
                    painter.rect_filled(
                        left_panel,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 0.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::WHITE,
                    );

                    // Right half of panel (Dark charcoal, rounded top-right)
                    let right_panel =
                        Rect::from_min_max(pos2(panel_mid_x, panel_rect.min.y), panel_rect.max);
                    painter.rect_filled(
                        right_panel,
                        egui::Rounding {
                            nw: 0.0,
                            ne: 2.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::from_rgb(0x1a, 0x1a, 0x1a),
                    );

                    // Draw window controls dots (Red, Yellow, Green)
                    let dot_y = left_panel.top() + 8.0;
                    painter.circle_filled(
                        pos2(left_panel.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0x5f, 0x56),
                    );
                    painter.circle_filled(
                        pos2(left_panel.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0xbd, 0x2e),
                    );
                    painter.circle_filled(
                        pos2(left_panel.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x27, 0xc9, 0x3f),
                    );

                    painter.circle_filled(
                        pos2(right_panel.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x8f, 0x3a, 0x35),
                    );
                    painter.circle_filled(
                        pos2(right_panel.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x8f, 0x6e, 0x1d),
                    );
                    painter.circle_filled(
                        pos2(right_panel.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x1a, 0x75, 0x27),
                    );

                    // Draw mock UI lines
                    let line_y1 = panel_rect.top() + 18.0;
                    let line_y2 = panel_rect.top() + 32.0;
                    let line_y3 = panel_rect.top() + 46.0;

                    // Draw mock UI lines (Gray bars + orange accent pill + muted pill)
                    let stroke_gray = Stroke::new(2.5, Color32::from_rgb(0xdd, 0xdd, 0xdd));
                    painter.line_segment(
                        [
                            pos2(left_panel.left() + 8.0, line_y1),
                            pos2(left_panel.right() - 8.0, line_y1),
                        ],
                        stroke_gray,
                    );

                    // Preview accent pill
                    let preview_accent = Color32::from_rgb(0x00, 0xC8, 0xB0);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(left_panel.left() + 8.0, line_y2 - 2.5),
                            pos2(left_panel.right() - 12.0, line_y2 + 2.5),
                        ),
                        1.0,
                        preview_accent,
                    );

                    // Muted pill
                    let muted_color = Color32::from_rgb(0x60, 0x60, 0x60);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(left_panel.left() + 8.0, line_y3 - 2.5),
                            pos2(left_panel.right() - 16.0, line_y3 + 2.5),
                        ),
                        1.0,
                        muted_color,
                    );

                    // Right side details (Gray bars)
                    let stroke_dark_gray = Stroke::new(2.5, Color32::from_rgb(0x44, 0x44, 0x44));
                    painter.line_segment(
                        [
                            pos2(right_panel.left() + 8.0, line_y1),
                            pos2(right_panel.right() - 8.0, line_y1),
                        ],
                        stroke_dark_gray,
                    );
                    painter.line_segment(
                        [
                            pos2(right_panel.left() + 8.0, line_y2),
                            pos2(right_panel.right() - 12.0, line_y2),
                        ],
                        stroke_dark_gray,
                    );
                    painter.line_segment(
                        [
                            pos2(right_panel.left() + 8.0, line_y3),
                            pos2(right_panel.right() - 20.0, line_y3),
                        ],
                        stroke_dark_gray,
                    );
                },
            ) {
                config.settings.appearance_mode = "system".to_string();
                let _ = config.save();
                engine.reload_config();
            }

            // Card 2: Light Theme
            let label2 = tr("light_theme", &config.settings.language);
            if draw_theme_card(
                ui,
                card_w,
                card_h,
                label2,
                config.settings.appearance_mode == "light",
                |painter, rect| {
                    // Background: soft periwinkle blue
                    painter.rect_filled(
                        rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0xb5, 0xc4, 0xf5),
                    );

                    // Draw mock panel
                    let panel_w = 110.0;
                    let panel_h = 80.0;
                    let panel_rect = Rect::from_center_size(
                        pos2(rect.center().x, rect.bottom() - 10.0),
                        vec2(panel_w, panel_h),
                    );

                    // Panel (White, rounded top)
                    painter.rect_filled(
                        panel_rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::WHITE,
                    );

                    // Draw window controls dots (Red, Yellow, Green)
                    let dot_y = panel_rect.top() + 8.0;
                    painter.circle_filled(
                        pos2(panel_rect.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0x5f, 0x56),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0xbd, 0x2e),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x27, 0xc9, 0x3f),
                    );

                    // Mock UI lines
                    let line_y1 = panel_rect.top() + 18.0;
                    let line_y2 = panel_rect.top() + 32.0;
                    let line_y3 = panel_rect.top() + 46.0;

                    let stroke_gray = Stroke::new(2.5, Color32::from_rgb(0xdd, 0xdd, 0xdd));
                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y1),
                            pos2(panel_rect.right() - 8.0, line_y1),
                        ],
                        stroke_gray,
                    );

                    // Preview accent pill
                    let preview_accent = Color32::from_rgb(0x00, 0x89, 0x7B);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(panel_rect.left() + 8.0, line_y2 - 2.5),
                            pos2(panel_rect.right() - 20.0, line_y2 + 2.5),
                        ),
                        1.0,
                        preview_accent,
                    );

                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y3),
                            pos2(panel_rect.right() - 16.0, line_y3),
                        ],
                        stroke_gray,
                    );
                },
            ) {
                config.settings.appearance_mode = "light".to_string();
                let _ = config.save();
                engine.reload_config();
            }

            // Card 3: Dark Theme
            let label3 = tr("dark_theme", &config.settings.language);
            if draw_theme_card(
                ui,
                card_w,
                card_h,
                label3,
                config.settings.appearance_mode == "dark",
                |painter, rect| {
                    // Background: dark navy
                    painter.rect_filled(
                        rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0x14, 0x19, 0x23),
                    );

                    // Draw mock panel
                    let panel_w = 110.0;
                    let panel_h = 80.0;
                    let panel_rect = Rect::from_center_size(
                        pos2(rect.center().x, rect.bottom() - 10.0),
                        vec2(panel_w, panel_h),
                    );

                    // Panel (Dark charcoal, rounded top)
                    painter.rect_filled(
                        panel_rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::from_rgb(0x1a, 0x1a, 0x1a),
                    );

                    // Draw window controls dots (Red, Yellow, Green)
                    let dot_y = panel_rect.top() + 8.0;
                    painter.circle_filled(
                        pos2(panel_rect.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0x5f, 0x56),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0xbd, 0x2e),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x27, 0xc9, 0x3f),
                    );

                    // Mock UI lines
                    let line_y1 = panel_rect.top() + 18.0;
                    let line_y2 = panel_rect.top() + 32.0;
                    let line_y3 = panel_rect.top() + 46.0;

                    let stroke_dark_gray = Stroke::new(2.5, Color32::from_rgb(0x44, 0x44, 0x44));
                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y1),
                            pos2(panel_rect.right() - 8.0, line_y1),
                        ],
                        stroke_dark_gray,
                    );

                    // Preview accent pill
                    let preview_accent = Color32::from_rgb(0x00, 0xC8, 0xB0);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(panel_rect.left() + 8.0, line_y2 - 2.5),
                            pos2(panel_rect.right() - 20.0, line_y2 + 2.5),
                        ),
                        1.0,
                        preview_accent,
                    );

                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y3),
                            pos2(panel_rect.right() - 16.0, line_y3),
                        ],
                        stroke_dark_gray,
                    );
                },
            ) {
                config.settings.appearance_mode = "dark".to_string();
                let _ = config.save();
                engine.reload_config();
            }
        });
    }); // section_card
}

fn draw_theme_card<F>(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    label: &str,
    is_selected: bool,
    draw_preview: F,
) -> bool
where
    F: FnOnce(&egui::Painter, Rect),
{
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click());

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    // Smooth border and background color hover transitions
    let is_sel_anim = ui.ctx().animate_bool(response.id, is_selected);
    let is_hov_anim = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());

    let base_border = theme::border_color(ui.ctx());
    let hover_border = if ui.visuals().dark_mode {
        Color32::from_rgb(0x55, 0x55, 0x55)
    } else {
        Color32::from_rgb(0xaa, 0xaa, 0xaa)
    };
    let selected_border = theme::accent_color(ui.ctx());

    // Calculate animated border color and thickness
    let border_color = if is_selected {
        selected_border
    } else {
        theme::lerp_color(base_border, hover_border, is_hov_anim)
    };
    let border_thickness = if is_selected {
        1.5
    } else {
        1.0 + 0.5 * is_hov_anim
    };

    // Calculate animated background color using surface and hover colors from active theme
    let bg_fill = theme::lerp_color(
        theme::card_bg(ui.ctx()),
        theme::hover_color(ui.ctx()),
        is_hov_anim,
    );
    ui.painter().rect_filled(rect, 2.0, bg_fill);

    // Split layout for preview (top 110px) and label (bottom 70px)
    let preview_rect = Rect::from_min_max(
        rect.min + vec2(1.0, 1.0),
        pos2(rect.max.x - 1.0, rect.min.y + 110.0),
    );

    // Draw preview
    let painter = ui.painter();
    draw_preview(&painter.with_clip_rect(preview_rect), preview_rect);

    // Bottom area
    let label_rect = Rect::from_min_max(pos2(rect.min.x, rect.min.y + 110.0), rect.max);

    // Minimalist Square Checkbox (14x14 px)
    let check_size = 14.0;
    let check_rect = Rect::from_center_size(
        pos2(label_rect.left() + 24.0, label_rect.center().y),
        vec2(check_size, check_size),
    );

    let inactive_border = theme::border_color(ui.ctx());
    let active_border = theme::accent_color(ui.ctx());
    let check_border_color = theme::lerp_color(inactive_border, active_border, is_sel_anim);

    // Draw outer checkbox border (2px corner radius)
    ui.painter()
        .rect_stroke(check_rect, 1.0, Stroke::new(1.0, check_border_color));

    // Draw filled Signal Orange inner square (8x8 px) when selected
    let inner_size = 8.0 * is_sel_anim;
    if inner_size > 0.0 {
        let inner_rect = Rect::from_center_size(check_rect.center(), vec2(inner_size, inner_size));
        ui.painter()
            .rect_filled(inner_rect, 0.0, theme::accent_color(ui.ctx()));
    }

    // Label text
    let text_pos = pos2(check_rect.center().x + 20.0, label_rect.center().y);
    let text_color = theme::primary_text(ui.ctx());

    // Draw text with support for multi-line (using splitting on newline if present)
    let lines: Vec<&str> = label.split('\n').collect();
    if lines.len() == 2 {
        let galley1 = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                lines[0].to_string(),
                egui::FontId::proportional(13.0),
                text_color,
            ))
        });
        let galley2 = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                lines[1].to_string(),
                egui::FontId::proportional(13.0),
                text_color,
            ))
        });
        ui.painter()
            .galley(text_pos - vec2(0.0, 14.0), galley1, text_color);
        ui.painter()
            .galley(text_pos - vec2(0.0, -2.0), galley2, text_color);
    } else {
        let galley = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                label.to_string(),
                egui::FontId::proportional(13.0),
                text_color,
            ))
        });
        let text_y = text_pos.y - galley.size().y / 2.0;
        ui.painter()
            .galley(pos2(text_pos.x, text_y), galley, text_color);
    }

    // Draw border around the entire card
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(border_thickness, border_color));

    // Tech corner brackets (which glow in orange on hover or selection)
    let corner_color = theme::lerp_color(
        theme::border_color(ui.ctx()),
        theme::accent_color(ui.ctx()),
        is_sel_anim.max(is_hov_anim),
    );
    draw_tech_corners(ui.painter(), rect, corner_color, 6.0);

    response.clicked()
}
