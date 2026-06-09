use eframe::egui::{self, Color32, Rect, Stroke, pos2, vec2};

pub fn draw_equalizer_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    for i in 0..3 {
        let x = center.x - 5.0 + i as f32 * 5.0;
        painter.line_segment([pos2(x, center.y - 5.5), pos2(x, center.y + 5.5)], stroke);
        let knob_y = match i {
            0 => center.y - 2.0,
            1 => center.y + 2.5,
            _ => center.y - 3.5,
        };
        painter.circle_filled(pos2(x, knob_y), 1.8, color);
    }
}

pub fn draw_mouse_outline_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    let r = Rect::from_center_size(center, vec2(10.0, 14.0));
    painter.rect_stroke(r, 4.0, stroke);
    painter.line_segment(
        [pos2(center.x, r.min.y), pos2(center.x, center.y - 1.0)],
        stroke,
    );
}

pub fn draw_flow_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    let r1 = Rect::from_center_size(center - vec2(2.5, 2.5), vec2(8.0, 6.0));
    painter.rect_stroke(r1, 1.0, stroke);

    let r2 = Rect::from_center_size(center + vec2(2.5, 2.5), vec2(8.0, 6.0));
    painter.rect_filled(r2, 1.0, Color32::from_rgb(0x11, 0x11, 0x11));
    painter.rect_stroke(r2, 1.0, stroke);
}

pub fn draw_hamburger_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    painter.line_segment(
        [
            pos2(center.x - 6.0, center.y - 3.5),
            pos2(center.x + 6.0, center.y - 3.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x - 6.0, center.y),
            pos2(center.x + 6.0, center.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x - 6.0, center.y + 3.5),
            pos2(center.x + 6.0, center.y + 3.5),
        ],
        stroke,
    );
}
