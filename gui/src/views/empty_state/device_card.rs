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
    // Blue circle background + the shared white bluetooth rune
    let circle_color = egui::Color32::from_rgb(0, 122, 255); // Solid vibrant blue
    painter.circle_filled(center, 11.0, circle_color);
    crate::widgets::draw_bluetooth_rune(painter, center, 11.0, egui::Color32::WHITE);
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
        // Fixed-height card: identical for every device so the control row is
        // always at the same vertical level (mouse vs keyboard, active vs inactive).
        let layout_key = crate::app::get_layout_key_from_name(name);
        let is_kbd = layout_key.contains("keys") || layout_key.contains("mechanical");

        let card_w = if is_kbd { 560.0 } else { 220.0 };
        let img_h = 220.0;
        let img_gap = 20.0;
        let pill_h = 44.0;
        let card_h = img_h + img_gap + pill_h; // 284 for all devices

        let (card_rect, card_res) =
            ui.allocate_exact_size(egui::vec2(card_w, card_h), egui::Sense::hover());
        let img_area = egui::Rect::from_min_max(
            card_rect.min,
            egui::pos2(card_rect.max.x, card_rect.min.y + img_h),
        );

        // ── 1. Device image (fit-centered in the fixed image area) ──────────
        let mouse_res = ui.interact(img_area, card_res.id, egui::Sense::click());
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
                ui.painter().circle_filled(img_area.center(), r, glow_color);
            }
        }

        let aspect_ratio = texture.size()[0] as f32 / texture.size()[1] as f32;
        let anim_size = if aspect_ratio > (card_w / img_h) {
            let w = card_w * scale;
            egui::vec2(w, w / aspect_ratio)
        } else {
            let h = img_h * scale;
            egui::vec2(h * aspect_ratio, h)
        };
        let anim_rect = egui::Rect::from_center_size(img_area.center(), anim_size);

        ui.put(
            anim_rect,
            egui::Image::new(texture).fit_to_exact_size(anim_size),
        );

        // ── 2. Control row (pinned at fixed offset from card top) ───────────
        let gap = 10.0;
        let pill_w = if is_active { 96.0 } else { 110.0 };
        let trash_w = 44.0;
        let row_w = if is_active {
            pill_w
        } else {
            pill_w + gap + trash_w
        };

        // Pill and trash share the same top → always at the same level.
        let row_top = card_rect.min.y + img_h + img_gap;
        let row_min_x = img_area.center().x - row_w / 2.0;
        let pill_rect =
            egui::Rect::from_min_size(egui::pos2(row_min_x, row_top), egui::vec2(pill_w, pill_h));
        let trash_rect = egui::Rect::from_min_size(
            egui::pos2(row_min_x + pill_w + gap, row_top),
            egui::vec2(trash_w, pill_h),
        );

        // ── Pill (active or inactive) ───────────────────────────────────────
        let pill_id = ui.make_persistent_id(("dev_pill", name));
        let pill_res = ui
            .interact(pill_rect, pill_id, egui::Sense::hover())
            .on_hover_text(format!("{}%", battery_pct));
        let pt = ui.ctx().animate_bool(pill_id, pill_res.hovered());

        let pill_bg = egui::Color32::from_rgb(0x0a, 0x0a, 0x0a);
        let pill_border = if is_active {
            theme::lerp_color(
                egui::Color32::from_rgb(0x1c, 0x1c, 0x1c),
                theme::COLOR_ACCENT_DIM,
                pt,
            )
        } else {
            theme::lerp_color(
                egui::Color32::from_rgb(0x1c, 0x1c, 0x1c),
                egui::Color32::from_rgb(0x38, 0x38, 0x38),
                pt,
            )
        };

        let painter = ui.painter();
        painter.rect_filled(pill_rect, 6.0, pill_bg);
        painter.rect_stroke(
            pill_rect,
            6.0,
            egui::Stroke::new(1.0 + 0.5 * pt, pill_border),
        );

        let cx = pill_rect.center().x;
        let cy = pill_rect.center().y;

        if is_active {
            // Horizontal battery widget centered in the left half
            let batt_center = egui::pos2(pill_rect.min.x + (cx - pill_rect.min.x) / 2.0, cy);
            let batt_w = 22.0;
            let batt_h = 12.0;
            let batt_rect = egui::Rect::from_center_size(batt_center, egui::vec2(batt_w, batt_h));
            let pct: f32 = battery_pct.parse::<f32>().unwrap_or(80.0) / 100.0;
            draw_horizontal_battery_widget(painter, batt_rect, pct);

            // Thin vertical divider line segment between battery and connection icon
            painter.line_segment(
                [egui::pos2(cx, cy - 10.0), egui::pos2(cx, cy + 10.0)],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x22, 0x22, 0x22)),
            );

            // Connection icon (centered in the right half)
            let conn_center = egui::pos2(cx + (pill_rect.max.x - cx) / 2.0, cy);
            draw_connection_icon_clean(painter, conn_center, conn_type, egui::Color32::WHITE);
        } else {
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
        }

        if !is_active {
            // ── Trash button (same top as pill) ─────────────────────────────
            let trash_id = ui.make_persistent_id(("dev_trash", name));
            let trash_res = ui.interact(
                trash_rect,
                trash_id,
                egui::Sense::click().union(egui::Sense::focusable_noninteractive()),
            );
            if trash_res.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
            let tt = ui.ctx().animate_bool(trash_id, trash_res.hovered());
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

    if unpair_clicked {
        DeviceCardAction::Unpair
    } else {
        action
    }
}
