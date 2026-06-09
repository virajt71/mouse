use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use crate::theme;

pub fn show_flow_tab(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        ui.vertical(|ui| {
            ui.add_space(40.0);
            ui.add(egui::Label::new(
                RichText::new("FLOW")
                    .color(Color32::WHITE)
                    .size(18.0)
                    .strong(),
            ));
            ui.add_space(20.0);
            ui.add(egui::Label::new(
                RichText::new("Control multiple computers seamlessly with a single mouse.")
                    .color(theme::muted_text(ui.ctx()))
                    .size(12.5),
            ));

            ui.add_space(30.0);

            let cx = rect.center().x - 100.0;
            let cy = rect.center().y;

            let s_w = 120.0;
            let s_h = 80.0;

            let screen1 = Rect::from_center_size(pos2(cx - 70.0, cy), vec2(s_w, s_h));
            let screen2 = Rect::from_center_size(pos2(cx + 70.0, cy), vec2(s_w, s_h));

            // Screen 1: active
            ui.painter()
                .rect_filled(screen1, 2.0, theme::app_bg(ui.ctx()));
            ui.painter().rect_stroke(
                screen1,
                2.0,
                Stroke::new(1.5, theme::accent_color(ui.ctx())),
            );
            ui.painter().text(
                screen1.center(),
                egui::Align2::CENTER_CENTER,
                "Computer 1",
                egui::FontId::proportional(11.0),
                theme::primary_text(ui.ctx()),
            );

            // Screen 2: inactive
            ui.painter()
                .rect_filled(screen2, 2.0, theme::app_bg(ui.ctx()));
            ui.painter().rect_stroke(
                screen2,
                2.0,
                Stroke::new(1.0, theme::border_color(ui.ctx())),
            );
            ui.painter().text(
                screen2.center(),
                egui::Align2::CENTER_CENTER,
                "Computer 2",
                egui::FontId::proportional(11.0),
                theme::muted_text(ui.ctx()),
            );
        });
    });
}
