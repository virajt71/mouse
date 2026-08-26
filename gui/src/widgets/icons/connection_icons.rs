use eframe::egui;

pub fn draw_unifying_icon(ui: &egui::Ui, center: egui::Pos2) {
    let painter = ui.painter();

    // USB connector (silver metal part at top)
    let metal_rect =
        egui::Rect::from_center_size(center - egui::vec2(0.0, 4.0), egui::vec2(10.0, 8.0));
    painter.rect_filled(metal_rect, 1.0, egui::Color32::from_rgb(0xb0, 0xb0, 0xb0));

    // Two small slots in metal connector
    let hole1 = egui::Rect::from_center_size(center - egui::vec2(2.5, 5.5), egui::vec2(2.0, 2.5));
    let hole2 = egui::Rect::from_center_size(center - egui::vec2(-2.5, 5.5), egui::vec2(2.0, 2.5));
    painter.rect_filled(hole1, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));
    painter.rect_filled(hole2, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));

    // USB plastic collar (bottom base)
    let base_rect =
        egui::Rect::from_center_size(center + egui::vec2(0.0, 3.5), egui::vec2(15.0, 7.0));
    painter.rect_filled(base_rect, 1.5, egui::Color32::from_rgb(0x38, 0x38, 0x38));

    // Unifying orange dot logo
    painter.circle_filled(
        center + egui::vec2(0.0, 3.5),
        1.8,
        egui::Color32::from_rgb(0xff, 0x98, 0x00),
    );
}

pub fn draw_bolt_icon(ui: &egui::Ui, center: egui::Pos2) {
    let painter = ui.painter();

    // USB connector (silver metal part at top)
    let metal_rect =
        egui::Rect::from_center_size(center - egui::vec2(0.0, 4.0), egui::vec2(10.0, 8.0));
    painter.rect_filled(metal_rect, 1.0, egui::Color32::from_rgb(0xb0, 0xb0, 0xb0));

    // Two small slots in metal connector
    let hole1 = egui::Rect::from_center_size(center - egui::vec2(2.5, 5.5), egui::vec2(2.0, 2.5));
    let hole2 = egui::Rect::from_center_size(center - egui::vec2(-2.5, 5.5), egui::vec2(2.0, 2.5));
    painter.rect_filled(hole1, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));
    painter.rect_filled(hole2, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));

    // USB plastic collar (bottom base)
    let base_rect =
        egui::Rect::from_center_size(center + egui::vec2(0.0, 3.5), egui::vec2(15.0, 7.0));
    painter.rect_filled(base_rect, 1.5, egui::Color32::from_rgb(0x38, 0x38, 0x38));

    // Bolt neon yellow-green lightning logo
    let bolt_center = center + egui::vec2(0.0, 3.5);
    let bolt_color = egui::Color32::from_rgb(0xcc, 0xff, 0x00);

    let pts = vec![
        bolt_center + egui::vec2(0.5, -2.5),
        bolt_center + egui::vec2(1.5, -0.5),
        bolt_center + egui::vec2(0.2, -0.5),
        bolt_center + egui::vec2(0.8, 2.5),
        bolt_center + egui::vec2(-0.8, 0.5),
        bolt_center + egui::vec2(0.2, 0.5),
    ];

    painter.add(egui::Shape::convex_polygon(
        pts,
        bolt_color,
        egui::Stroke::NONE,
    ));
}

pub fn draw_bluetooth_icon(ui: &egui::Ui, center: egui::Pos2) {
    let painter = ui.painter();

    // Blue background circle + white bluetooth rune
    painter.circle_filled(center, 12.0, egui::Color32::from_rgb(0x00, 0x7a, 0xff));
    crate::widgets::draw_bluetooth_rune(painter, center, 12.0, egui::Color32::WHITE);
}
