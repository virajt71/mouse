use eframe::egui;

pub fn draw_tech_corners(
    painter: &egui::Painter,
    rect: egui::Rect,
    color: egui::Color32,
    len: f32,
) {
    let stroke = egui::Stroke::new(1.0, color);
    // Top-left
    painter.line_segment(
        [rect.left_top(), rect.left_top() + egui::vec2(len, 0.0)],
        stroke,
    );
    painter.line_segment(
        [rect.left_top(), rect.left_top() + egui::vec2(0.0, len)],
        stroke,
    );
    // Top-right
    painter.line_segment(
        [rect.right_top(), rect.right_top() + egui::vec2(-len, 0.0)],
        stroke,
    );
    painter.line_segment(
        [rect.right_top(), rect.right_top() + egui::vec2(0.0, len)],
        stroke,
    );
    // Bottom-left
    painter.line_segment(
        [
            rect.left_bottom(),
            rect.left_bottom() + egui::vec2(len, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            rect.left_bottom(),
            rect.left_bottom() + egui::vec2(0.0, -len),
        ],
        stroke,
    );
    // Bottom-right
    painter.line_segment(
        [
            rect.right_bottom(),
            rect.right_bottom() + egui::vec2(-len, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            rect.right_bottom(),
            rect.right_bottom() + egui::vec2(0.0, -len),
        ],
        stroke,
    );
}
