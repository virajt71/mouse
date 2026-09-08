use eframe::egui::{self, pos2, vec2, Color32, Rect, Stroke};

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

pub fn draw_keys_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);

    // Draw outer keyboard body
    let body = Rect::from_center_size(center, vec2(15.0, 10.0));
    painter.rect_stroke(body, 1.5, stroke);

    // Draw inner key dividing lines
    painter.line_segment(
        [
            pos2(body.min.x + 2.0, center.y - 1.5),
            pos2(body.max.x - 2.0, center.y - 1.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(body.min.x + 2.0, center.y + 1.5),
            pos2(body.max.x - 2.0, center.y + 1.5),
        ],
        stroke,
    );
    // Vertical ticks
    painter.line_segment(
        [
            pos2(center.x - 3.0, center.y - 1.5),
            pos2(center.x - 3.0, body.max.y - 1.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x + 3.0, center.y - 1.5),
            pos2(center.x + 3.0, body.max.y - 1.5),
        ],
        stroke,
    );
}

pub fn draw_backlighting_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);

    // Bulb body: circle + bottom connector
    let bulb_center = center - vec2(0.0, 1.5);
    painter.circle_stroke(bulb_center, 4.0, stroke);

    // Base lines
    painter.line_segment(
        [
            pos2(center.x - 2.0, center.y + 3.5),
            pos2(center.x + 2.0, center.y + 3.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x - 1.0, center.y + 5.5),
            pos2(center.x + 1.0, center.y + 5.5),
        ],
        stroke,
    );

    // Rays (top, left, right, top-left, top-right)
    let rays = [
        (-90.0f32, 5.0, 7.0),
        (-45.0f32, 5.0, 7.0),
        (-135.0f32, 5.0, 7.0),
        (0.0f32, 5.0, 7.0),
        (180.0f32, 5.0, 7.0),
    ];
    for &(angle_deg, r1, r2) in &rays {
        let rad = angle_deg.to_radians();
        let start = bulb_center + vec2(rad.cos(), rad.sin()) * r1;
        let end = bulb_center + vec2(rad.cos(), rad.sin()) * r2;
        painter.line_segment([start, end], Stroke::new(1.0, color));
    }
}

pub fn draw_settings_slider_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);

    for i in 0..3 {
        let y = center.y - 4.0 + i as f32 * 4.0;
        // Draw track
        painter.line_segment([pos2(center.x - 7.0, y), pos2(center.x + 7.0, y)], stroke);
        // Draw knob
        let knob_x = match i {
            0 => center.x - 3.0,
            1 => center.x + 3.0,
            _ => center.x - 1.0,
        };
        painter.circle_filled(pos2(knob_x, y), 2.0, color);
    }
}
pub fn draw_ring_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);

    // Outer ring circle
    painter.circle_stroke(center, 7.0, stroke);

    // Inner dot (center of ring)
    painter.circle_filled(center, 2.0, color);

    // Small bubbles around the ring (4 dots at N/E/S/W positions)
    let bubble_r = 1.5;
    let ring_r = 7.0;
    let positions = [
        (0.0, -ring_r),   // top
        (ring_r, 0.0),    // right
        (0.0, ring_r),    // bottom
        (-ring_r, 0.0),   // left
    ];
    for (dx, dy) in positions {
        painter.circle_filled(
            pos2(center.x + dx as f32, center.y + dy as f32),
            bubble_r,
            color,
        );
    }
}
