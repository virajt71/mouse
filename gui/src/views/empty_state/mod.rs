pub mod device_card;

pub use device_card::{show_known_device, DeviceCardAction};

use crate::theme;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, lang: &str) {
    ui.vertical_centered(|ui| {
        let height = ui.available_height();
        let content_height = 210.0;
        let top_padding = ((height - content_height) / 2.0).max(0.0);
        ui.add_space(top_padding);

        let (rect, _) = ui.allocate_exact_size(egui::vec2(300.0, 170.0), egui::Sense::hover());
        draw_illustration(ui, rect.center());

        ui.add_space(28.0);
        let caption = crate::translation::tr("connect_title", lang);
        draw_spaced_caption(ui, caption, 1.5);
    });
}

fn draw_spaced_caption(ui: &mut egui::Ui, text: &str, spacing: f32) {
    ui.vertical_centered(|ui| {
        let font_id = egui::FontId::proportional(11.5);
        let mut job = egui::text::LayoutJob::default();
        let mut chars = text.char_indices().peekable();
        while let Some((idx, _)) = chars.next() {
            let next_idx = chars.peek().map(|(n_idx, _)| *n_idx).unwrap_or(text.len());
            job.append(
                &text[idx..next_idx],
                spacing,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    color: theme::COLOR_SUBTLE_TEXT,
                    ..Default::default()
                },
            );
        }
        ui.label(job);
    });
}

fn draw_illustration(ui: &mut egui::Ui, center: egui::Pos2) {
    let painter = ui.painter();
    let accent = theme::accent_color(ui.ctx());
    let muted = theme::muted_text(ui.ctx());
    let border_col = theme::border_color(ui.ctx());

    // Define center for signal arcs (above the mouse)
    let signal_center = center - egui::vec2(0.0, 16.0);

    // Draw three concentric wireframe signal arcs (pointing upwards)
    let start_ang = -140.0_f32.to_radians();
    let end_ang = -40.0_f32.to_radians();

    for (i, r) in [24.0_f32, 40.0, 56.0].iter().enumerate() {
        let alpha = 140 - i as u8 * 40;
        let arc_color =
            egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha);

        let num_segments = 24;
        let mut points = Vec::with_capacity(num_segments + 1);
        for j in 0..=num_segments {
            let t = j as f32 / num_segments as f32;
            let angle = start_ang + (end_ang - start_ang) * t;
            points.push(signal_center + egui::vec2(angle.cos() * r, angle.sin() * r));
        }
        for j in 0..num_segments {
            painter.line_segment(
                [points[j], points[j + 1]],
                egui::Stroke::new(1.5, arc_color),
            );
        }
    }

    // Centered wireless dot at signal origin
    painter.circle_filled(signal_center, 4.0, accent);

    // Mouse wireframe silhouette (shifted lower)
    let mouse_w = 30.0;
    let mouse_h = 50.0;
    let mouse_body =
        egui::Rect::from_center_size(center + egui::vec2(0.0, 58.0), egui::vec2(mouse_w, mouse_h));

    // Draw subtle translucent mouse body fill
    let fill_color = if ui.visuals().dark_mode {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 4)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 3)
    };
    painter.rect_filled(mouse_body, 2.0, fill_color);

    // Wireframe outer stroke
    painter.rect_stroke(mouse_body, 2.0, egui::Stroke::new(1.2, muted));

    // Internal details (sleek wireframe style)
    let cy = mouse_body.center().y;
    // Left/right click split line
    painter.line_segment(
        [
            mouse_body.center_top() + egui::vec2(0.0, 1.0),
            egui::pos2(mouse_body.center().x, cy),
        ],
        egui::Stroke::new(1.0, border_col),
    );

    // Scroll wheel wireframe
    let wheel_rect = egui::Rect::from_center_size(
        mouse_body.center_top() + egui::vec2(0.0, 12.0),
        egui::vec2(6.0, 12.0),
    );
    painter.rect_filled(
        wheel_rect,
        2.0,
        if ui.visuals().dark_mode {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 10)
        },
    );
    painter.rect_stroke(wheel_rect, 2.0, egui::Stroke::new(1.0, accent));
}
