use eframe::egui;

/// Draw the canonical Lucide "bluetooth" glyph (OpenLogi ships the same SVG)
/// as a single stroked polyline, scaled to radius `r`. The path
/// `m7 7 10 10-5 5V2l5 5L7 17` is a 24×24 viewBox hexagon — one continuous
/// stroke, round joins/caps like the source.
pub fn draw_bluetooth_rune(
    painter: &egui::Painter,
    center: egui::Pos2,
    r: f32,
    color: egui::Color32,
) {
    // ViewBox-relative points of the lucide path, centered on (12,12).
    const PTS: [(f32, f32); 6] = [
        (-5.0, -5.0), // m7 7
        (5.0, 5.0),   // l10 10
        (0.0, 10.0),  // l-5 5
        (0.0, -10.0), // V2
        (5.0, -5.0),  // l5 5
        (-5.0, 5.0),  // L7 17
    ];
    // Glyph reaches ~±10 units vertically in a 24-unit viewBox; scale so it
    // spans ~1.6R, and mirror the SVG's stroke-width of 2/24.
    let s = r / 7.5;
    let stroke_w = (r / 12.0).max(1.6);
    let points: Vec<egui::Pos2> = PTS
        .iter()
        .map(|(x, y)| center + egui::vec2(x * s, y * s))
        .collect();
    painter.add(egui::Shape::line(points, egui::Stroke::new(stroke_w, color)));
}

pub fn draw_connection_icon_mini(painter: &egui::Painter, center: egui::Pos2, conn_type: &str) {
    match conn_type {
        "bluetooth" => {
            painter.circle_filled(center, 11.0, egui::Color32::from_rgb(0x00, 0x7a, 0xff));
            draw_bluetooth_rune(painter, center, 11.0, egui::Color32::WHITE);
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
