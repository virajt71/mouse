use eframe::egui;

pub fn draw_connection_icon_mini(painter: &egui::Painter, center: egui::Pos2, conn_type: &str) {
    match conn_type {
        "bluetooth" => {
            painter.circle_filled(center, 11.0, egui::Color32::from_rgb(0x00, 0x7a, 0xff));
            let stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
            let (cx, cy) = (center.x, center.y);
            painter.line_segment([egui::pos2(cx, cy - 6.0), egui::pos2(cx, cy + 6.0)], stroke);
            painter.line_segment([egui::pos2(cx, cy), egui::pos2(cx + 2.8, cy - 3.0)], stroke);
            painter.line_segment(
                [egui::pos2(cx + 2.8, cy - 3.0), egui::pos2(cx, cy - 6.0)],
                stroke,
            );
            painter.line_segment([egui::pos2(cx, cy), egui::pos2(cx + 2.8, cy + 3.0)], stroke);
            painter.line_segment(
                [egui::pos2(cx + 2.8, cy + 3.0), egui::pos2(cx, cy + 6.0)],
                stroke,
            );
            painter.line_segment(
                [egui::pos2(cx, cy - 3.0), egui::pos2(cx - 2.8, cy - 6.0)],
                stroke,
            );
            painter.line_segment(
                [egui::pos2(cx, cy + 3.0), egui::pos2(cx - 2.8, cy + 6.0)],
                stroke,
            );
        }
        "unifying" => {
            painter.circle_filled(center, 11.0, egui::Color32::from_rgb(0xff, 0x98, 0x00));
            let color = egui::Color32::WHITE;
            painter.circle_filled(center, 2.0, color);
            let stroke = egui::Stroke::new(1.5, color);
            for i in 0..6 {
                let angle = (i as f32) * std::f32::consts::TAU / 6.0;
                painter.line_segment(
                    [
                        center,
                        center + egui::vec2(angle.cos() * 6.5, angle.sin() * 6.5),
                    ],
                    stroke,
                );
            }
        }
        "bolt" => {
            painter.circle_filled(center, 11.0, egui::Color32::from_rgb(0xcc, 0xff, 0x00));
            let color = egui::Color32::BLACK;
            let pts = vec![
                center + egui::vec2(0.8, -4.5),
                center + egui::vec2(2.5, -1.0),
                center + egui::vec2(0.3, -1.0),
                center + egui::vec2(1.2, 4.5),
                center + egui::vec2(-1.2, 1.0),
                center + egui::vec2(0.3, 1.0),
            ];
            painter.add(egui::Shape::convex_polygon(pts, color, egui::Stroke::NONE));
        }
        _ => {}
    }
}
