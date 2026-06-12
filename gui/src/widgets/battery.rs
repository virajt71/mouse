use crate::theme;
use eframe::egui;

pub fn draw_battery_widget(painter: &egui::Painter, rect: egui::Rect, level: f32) {
    let stroke_color = egui::Color32::from_rgb(0x66, 0x66, 0x66);
    let fill_color = if level <= 0.20 {
        theme::COLOR_DOT_RED
    } else {
        theme::COLOR_ACCENT
    };

    let cy = rect.center().y;
    let icon_w = 20.0;
    let icon_h = 10.0;
    let cap_w = 2.0;

    let content_w = icon_w + cap_w;
    let x0 = rect.center().x - content_w / 2.0;

    // Body outline
    let body = egui::Rect::from_min_max(
        egui::pos2(x0, cy - icon_h / 2.0),
        egui::pos2(x0 + icon_w, cy + icon_h / 2.0),
    );
    painter.rect_stroke(body, 2.0, egui::Stroke::new(1.2, stroke_color));

    // Cap
    let cap = egui::Rect::from_min_max(
        egui::pos2(x0 + icon_w + 0.5, cy - 2.5),
        egui::pos2(x0 + icon_w + cap_w, cy + 2.5),
    );
    painter.rect_filled(cap, 1.0, stroke_color);

    // Fill
    let fill_w = (icon_w - 4.0) * level.clamp(0.0, 1.0);
    if fill_w > 0.0 {
        let fill = egui::Rect::from_min_max(
            egui::pos2(x0 + 2.0, cy - (icon_h / 2.0 - 2.0)),
            egui::pos2(x0 + 2.0 + fill_w, cy + (icon_h / 2.0 - 2.0)),
        );
        painter.rect_filled(fill, 1.0, fill_color);
    }
}
