use eframe::egui::{self, pos2, Color32, Stroke};

pub fn draw_globe_icon(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let center = rect.center();
    let r = 6.0;
    let stroke = egui::Stroke::new(1.2, color);
    painter.circle_stroke(center, r, stroke);

    // Horizontal equator
    painter.line_segment(
        [center - egui::vec2(r, 0.0), center + egui::vec2(r, 0.0)],
        stroke,
    );
    // Vertical meridian
    painter.line_segment(
        [center - egui::vec2(0.0, r), center + egui::vec2(0.0, r)],
        stroke,
    );

    // Oval vertical meridian
    let num_pts = 16;
    for i in 0..num_pts {
        let t1 = i as f32 / num_pts as f32;
        let t2 = (i + 1) as f32 / num_pts as f32;
        let ang1 = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * t1;
        let ang2 = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * t2;

        let p1 = center + egui::vec2(ang1.cos() * r * 0.5, ang1.sin() * r);
        let p2 = center + egui::vec2(ang2.cos() * r * 0.5, ang2.sin() * r);
        painter.line_segment([p1, p2], stroke);
    }
}

pub fn draw_rgb_palette_icon(ui: &mut egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let center = rect.center();
    let r = 4.0;
    let offset = 2.5;

    // Three overlapping circles (Red, Green, Blue)
    let c_red = center + egui::vec2(0.0, -offset);
    let c_green = center + egui::vec2(-offset * 0.866, offset * 0.5);
    let c_blue = center + egui::vec2(offset * 0.866, offset * 0.5);

    let alpha = 150;
    painter.circle_filled(
        c_red,
        r,
        egui::Color32::from_rgba_unmultiplied(255, 0, 0, alpha),
    );
    painter.circle_filled(
        c_green,
        r,
        egui::Color32::from_rgba_unmultiplied(0, 255, 0, alpha),
    );
    painter.circle_filled(
        c_blue,
        r,
        egui::Color32::from_rgba_unmultiplied(0, 0, 255, alpha),
    );

    // Add thin outlines
    let stroke = egui::Stroke::new(0.8, egui::Color32::WHITE);
    painter.circle_stroke(c_red, r, stroke);
    painter.circle_stroke(c_green, r, stroke);
    painter.circle_stroke(c_blue, r, stroke);
}

pub fn draw_refresh_icon(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let center = rect.center();
    let r = 5.5;
    let stroke = egui::Stroke::new(1.2, color);

    // Draw arc for circular arrow (about 270 degrees)
    let start_ang = -45.0_f32.to_radians();
    let end_ang = 225.0_f32.to_radians();
    let num_segments = 16;
    let mut points = Vec::with_capacity(num_segments + 1);
    for j in 0..=num_segments {
        let t = j as f32 / num_segments as f32;
        let angle = start_ang + (end_ang - start_ang) * t;
        points.push(center + egui::vec2(angle.cos() * r, angle.sin() * r));
    }
    for j in 0..num_segments {
        painter.line_segment([points[j], points[j + 1]], stroke);
    }

    // Draw arrow head at start_ang
    let arrow_pos = points[0];
    let arrow_stroke = egui::Stroke::new(1.2, color);
    painter.line_segment(
        [arrow_pos, arrow_pos + egui::vec2(-2.0, -3.0)],
        arrow_stroke,
    );
    painter.line_segment([arrow_pos, arrow_pos + egui::vec2(-3.0, 2.0)], arrow_stroke);
}

pub fn draw_profiles_icon_settings(ui: &egui::Ui, rect: egui::Rect, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    let center = rect.center();
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
