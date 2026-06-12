use crate::theme;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCardAction {
    None,
    Unpair,
    Customize,
}

pub fn draw_horizontal_battery_widget(painter: &egui::Painter, rect: egui::Rect, level: f32) {
    let stroke_color = egui::Color32::from_rgb(0x44, 0x44, 0x44);
    let fill_color = if level <= 0.20 {
        theme::COLOR_DOT_RED
    } else {
        egui::Color32::from_rgb(0x00, 0xd4, 0xc8) // Cyan/teal
    };

    let cx = rect.center().x;
    let cy = rect.center().y;

    let w = 22.0;
    let h = 12.0;
    let body = egui::Rect::from_center_size(egui::pos2(cx - 1.0, cy), egui::vec2(w, h));

    // Outer body outline
    painter.rect_stroke(body, 3.0, egui::Stroke::new(1.5, stroke_color));

    // Battery tip on the right edge
    let tip_w = 2.0;
    let tip_h = 4.0;
    let tip = egui::Rect::from_min_max(
        egui::pos2(body.max.x, cy - tip_h / 2.0),
        egui::pos2(body.max.x + tip_w, cy + tip_h / 2.0),
    );
    painter.rect_filled(tip, 0.5, stroke_color);

    // Inner fill representing level
    let max_fill_w = w - 4.0;
    let fill_w = max_fill_w * level.clamp(0.0, 1.0);
    if fill_w > 0.0 {
        let fill = egui::Rect::from_min_max(
            egui::pos2(body.min.x + 2.0, body.min.y + 2.0),
            egui::pos2(body.min.x + 2.0 + fill_w, body.max.y - 2.0),
        );
        painter.rect_filled(fill, 1.0, fill_color);
    }
}

pub fn draw_bluetooth_icon_clean(
    painter: &egui::Painter,
    center: egui::Pos2,
    _color: egui::Color32,
) {
    // Draw filled blue circle background
    let circle_color = egui::Color32::from_rgb(0, 122, 255); // Solid vibrant blue
    painter.circle_filled(center, 11.0, circle_color);

    // Draw white bluetooth icon on top
    let stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    let (cx, cy) = (center.x, center.y);
    painter.line_segment([egui::pos2(cx, cy - 5.0), egui::pos2(cx, cy + 5.0)], stroke);
    painter.line_segment([egui::pos2(cx, cy), egui::pos2(cx + 2.5, cy - 2.5)], stroke);
    painter.line_segment(
        [egui::pos2(cx + 2.5, cy - 2.5), egui::pos2(cx, cy - 5.0)],
        stroke,
    );
    painter.line_segment([egui::pos2(cx, cy), egui::pos2(cx + 2.5, cy + 2.5)], stroke);
    painter.line_segment(
        [egui::pos2(cx + 2.5, cy + 2.5), egui::pos2(cx, cy + 5.0)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(cx, cy - 2.5), egui::pos2(cx - 2.5, cy - 5.0)],
        stroke,
    );
    painter.line_segment(
        [egui::pos2(cx, cy + 2.5), egui::pos2(cx - 2.5, cy + 5.0)],
        stroke,
    );
}

pub fn draw_unifying_icon_clean(
    painter: &egui::Painter,
    center: egui::Pos2,
    _color: egui::Color32,
) {
    // Draw filled orange circle background
    let circle_color = egui::Color32::from_rgb(249, 115, 22);
    painter.circle_filled(center, 11.0, circle_color);

    // Draw white unifying logo in the center
    painter.circle_filled(center, 2.0, egui::Color32::WHITE);
    let stroke = egui::Stroke::new(1.2, egui::Color32::WHITE);
    for i in 0..6 {
        let angle = (i as f32) * std::f32::consts::TAU / 6.0;
        painter.line_segment(
            [
                center,
                center + egui::vec2(angle.cos() * 5.0, angle.sin() * 5.0),
            ],
            stroke,
        );
    }
}

pub fn draw_bolt_icon_clean(painter: &egui::Painter, center: egui::Pos2, _color: egui::Color32) {
    // Draw filled yellow circle background
    let circle_color = egui::Color32::from_rgb(254, 240, 138); // light yellow
    painter.circle_filled(center, 11.0, circle_color);

    // Draw dark grey bolt glyph in the center
    let glyph_color = egui::Color32::from_rgb(0x11, 0x11, 0x11);
    let pts = vec![
        center + egui::vec2(0.8, -4.5),
        center + egui::vec2(2.5, -1.0),
        center + egui::vec2(0.3, -1.0),
        center + egui::vec2(1.2, 4.5),
        center + egui::vec2(-1.2, 1.0),
        center + egui::vec2(0.3, 1.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        pts,
        glyph_color,
        egui::Stroke::NONE,
    ));
}

pub fn draw_connection_icon_clean(
    painter: &egui::Painter,
    center: egui::Pos2,
    conn_type: &str,
    color: egui::Color32,
) {
    match conn_type {
        "bluetooth" => draw_bluetooth_icon_clean(painter, center, color),
        "unifying" => draw_unifying_icon_clean(painter, center, color),
        "bolt" => draw_bolt_icon_clean(painter, center, color),
        _ => {}
    }
}

pub fn show_known_device(
    ui: &mut egui::Ui,
    name: &str,
    is_active: bool,
    texture: &egui::TextureHandle,
    conn_type: &str,
    battery_pct: &str,
    lang: &str,
) -> DeviceCardAction {
    let mut unpair_clicked = false;
    let mut action = DeviceCardAction::None;

    ui.vertical(|ui| {
        // ── 1. Device image (size adjusted by type: mouse vs keyboard) ───────
        let layout_key = crate::app::get_layout_key_from_name(name);
        let is_kbd = layout_key.contains("keys") || layout_key.contains("mechanical");

        let img_size = if is_kbd {
            egui::vec2(480.0, 200.0)
        } else {
            egui::vec2(220.0, 220.0)
        };

        let (mouse_rect, mouse_res) = ui.allocate_exact_size(img_size, egui::Sense::click());
        if mouse_res.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        if mouse_res.clicked() {
            action = DeviceCardAction::Customize;
        }

        // Only animate when the device is active/connected
        let t = if is_active {
            ui.ctx().animate_bool(mouse_res.id, mouse_res.hovered())
        } else {
            0.0
        };
        let scale = 1.0 + 0.05 * t; // Scale up by 5% on hover

        // Draw soft backglow centered on the image
        if t > 0.0 {
            let glow_alpha = (25.0 * t) as u8;
            let glow_color = egui::Color32::from_rgba_unmultiplied(
                theme::COLOR_ACCENT.r(),
                theme::COLOR_ACCENT.g(),
                theme::COLOR_ACCENT.b(),
                glow_alpha,
            );
            for r in [60.0, 90.0, 120.0, 150.0] {
                ui.painter()
                    .circle_filled(mouse_rect.center(), r, glow_color);
            }
        }

        let anim_size = img_size * scale;
        let anim_rect = egui::Rect::from_center_size(mouse_rect.center(), anim_size);

        ui.put(
            anim_rect,
            egui::Image::new(texture).fit_to_exact_size(anim_size),
        );

        // ── 2. Control row ───────────────────────────────────────────────────
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            // Centre the row manually under the image
            let gap = 10.0;
            let pill_w = if is_active { 96.0 } else { 110.0 };
            let pill_h = 44.0;
            let trash_w = 44.0;

            let row_width = if is_active {
                pill_w
            } else {
                pill_w + gap + trash_w
            };

            let offset = (mouse_rect.width() - row_width) / 2.0;
            let cursor_x = ui.cursor().min.x;
            let target_x = mouse_rect.center().x - row_width / 2.0;
            let needed_space = (target_x - cursor_x).max(offset).max(0.0);
            ui.add_space(needed_space);

            if is_active {
                // ── Active: single pill with battery + connection ──────────
                let (pill_rect, pill_res) =
                    ui.allocate_exact_size(egui::vec2(pill_w, pill_h), egui::Sense::hover());
                let pill_res = pill_res.on_hover_text(format!("{}%", battery_pct));
                let pt = ui.ctx().animate_bool(pill_res.id, pill_res.hovered());

                // Very dark theme background
                let bg = egui::Color32::from_rgb(0x0a, 0x0a, 0x0a);
                let border = theme::lerp_color(
                    egui::Color32::from_rgb(0x1c, 0x1c, 0x1c),
                    theme::COLOR_ACCENT_DIM,
                    pt,
                );

                let painter = ui.painter();
                painter.rect_filled(pill_rect, 6.0, bg);
                painter.rect_stroke(pill_rect, 6.0, egui::Stroke::new(1.0 + 0.5 * pt, border));

                let cx = pill_rect.center().x;
                let cy = pill_rect.center().y;

                // Horizontal battery widget centered in the left half
                let batt_center = egui::pos2(pill_rect.min.x + (cx - pill_rect.min.x) / 2.0, cy);
                let batt_w = 22.0;
                let batt_h = 12.0;
                let batt_rect =
                    egui::Rect::from_center_size(batt_center, egui::vec2(batt_w, batt_h));
                let pct: f32 = battery_pct.parse::<f32>().unwrap_or(80.0) / 100.0;
                draw_horizontal_battery_widget(painter, batt_rect, pct);

                // Thin vertical divider line segment between battery and connection icon
                painter.line_segment(
                    [egui::pos2(cx, cy - 10.0), egui::pos2(cx, cy + 10.0)],
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(0x22, 0x22, 0x22)),
                );

                // Connection icon (centered in the right half, now with solid colored circle background)
                let conn_center = egui::pos2(cx + (pill_rect.max.x - cx) / 2.0, cy);
                draw_connection_icon_clean(painter, conn_center, conn_type, egui::Color32::WHITE);
            } else {
                // ── Inactive: status pill + trash button ──────────────────
                let (pill_rect, pill_res) =
                    ui.allocate_exact_size(egui::vec2(pill_w, pill_h), egui::Sense::hover());
                let pt = ui.ctx().animate_bool(pill_res.id, pill_res.hovered());

                let bg = egui::Color32::from_rgb(0x0a, 0x0a, 0x0a);
                let border = theme::lerp_color(
                    egui::Color32::from_rgb(0x1c, 0x1c, 0x1c),
                    egui::Color32::from_rgb(0x38, 0x38, 0x38),
                    pt,
                );

                let painter = ui.painter();
                painter.rect_filled(pill_rect, 6.0, bg);
                painter.rect_stroke(pill_rect, 6.0, egui::Stroke::new(1.0, border));

                // "INACTIVE" label
                let label_font = egui::FontId::proportional(11.0);
                let disc_text = crate::translation::tr("inactive", lang);
                painter.text(
                    pill_rect.center() - egui::vec2(0.0, 1.0),
                    egui::Align2::CENTER_CENTER,
                    disc_text,
                    label_font,
                    egui::Color32::from_rgb(0xdd, 0xdd, 0xdd),
                );

                ui.add_space(gap);

                // Trash button
                let (trash_rect, trash_res) = ui.allocate_exact_size(
                    egui::vec2(trash_w, pill_h),
                    egui::Sense::click().union(egui::Sense::focusable_noninteractive()),
                );
                if trash_res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                let tt = ui.ctx().animate_bool(trash_res.id, trash_res.hovered());
                let trash_bg = theme::lerp_color(
                    egui::Color32::from_rgb(0x0a, 0x0a, 0x0a),
                    egui::Color32::from_rgb(0x28, 0x10, 0x10),
                    tt,
                );
                let trash_border = theme::lerp_color(
                    egui::Color32::from_rgb(0x1c, 0x1c, 0x1c),
                    theme::COLOR_DOT_RED,
                    tt,
                );
                let trash_icon_col = theme::lerp_color(
                    egui::Color32::from_rgb(0xdd, 0xdd, 0xdd),
                    theme::COLOR_DOT_RED,
                    tt,
                );

                let painter = ui.painter();
                painter.rect_filled(trash_rect, 6.0, trash_bg);
                painter.rect_stroke(trash_rect, 6.0, egui::Stroke::new(1.0, trash_border));
                crate::widgets::draw_trash_icon(ui, trash_rect.center(), trash_icon_col);

                // Paint focus ring for trash button
                if trash_res.has_focus() {
                    painter.rect_stroke(
                        trash_rect.expand(2.0),
                        2.0,
                        egui::Stroke::new(2.0, theme::accent_color(ui.ctx())),
                    );
                }

                if trash_res.clicked() {
                    unpair_clicked = true;
                }
            }
        });
    });

    if unpair_clicked {
        DeviceCardAction::Unpair
    } else {
        action
    }
}
