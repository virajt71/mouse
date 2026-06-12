use crate::theme;
use eframe::egui;

pub fn draw_trash_icon(ui: &egui::Ui, center: egui::Pos2, color: egui::Color32) {
    let painter = ui.painter();
    let stroke = egui::Stroke::new(1.5, color);
    // Lid bar
    painter.line_segment(
        [
            center + egui::vec2(-7.0, -5.5),
            center + egui::vec2(7.0, -5.5),
        ],
        stroke,
    );
    // Lid handle
    painter.line_segment(
        [
            center + egui::vec2(-2.5, -5.5),
            center + egui::vec2(-2.5, -8.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(-2.5, -8.5),
            center + egui::vec2(2.5, -8.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(2.5, -8.5),
            center + egui::vec2(2.5, -5.5),
        ],
        stroke,
    );
    // Body: left, bottom, right
    painter.line_segment(
        [
            center + egui::vec2(-5.5, -5.5),
            center + egui::vec2(-4.5, 7.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(-4.5, 7.0),
            center + egui::vec2(4.5, 7.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(4.5, -5.5),
            center + egui::vec2(4.5, 7.0),
        ],
        stroke,
    );
    // Interior lines
    painter.line_segment(
        [
            center + egui::vec2(-1.8, -2.5),
            center + egui::vec2(-1.8, 4.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(1.8, -2.5),
            center + egui::vec2(1.8, 4.0),
        ],
        stroke,
    );
}

pub fn draw_settings_gear_icon(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let center = rect.center();
    let r_out = 6.5; // Outer radius of the gear body
    let r_in = 4.0; // Inner radius of the gear body
    let r_hole = 2.0; // Center hole radius
    let num_teeth = 8;

    // Draw outer body outline & filled area
    painter.circle_stroke(center, r_out, egui::Stroke::new(1.5, color));
    painter.circle_filled(center, r_out, color);

    // Draw center hole to cut out the inner area
    let bg_color = theme::app_bg(ui.ctx());
    painter.circle_filled(center, r_hole, bg_color);

    // Draw teeth/spokes
    let stroke = egui::Stroke::new(2.2, color);
    for i in 0..num_teeth {
        let angle = (i as f32) * std::f32::consts::TAU / (num_teeth as f32);
        let direction = egui::vec2(angle.cos(), angle.sin());
        let p_start = center + direction * r_in;
        let p_end = center + direction * (r_out + 2.0);
        painter.line_segment([p_start, p_end], stroke);
    }

    // Re-draw center hole to keep it clean (in case spokes overlap it)
    painter.circle_filled(center, r_hole, bg_color);
}
