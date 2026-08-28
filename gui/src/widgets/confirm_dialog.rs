use crate::theme;
use eframe::egui;
use egui::{vec2, Color32, Rect, RichText, Stroke};

pub fn show_confirm_dialog(
    ctx: &egui::Context,
    title: &str,
    body: &str,
    confirm_text: &str,
    cancel_text: &str,
) -> Option<bool> {
    let mut clicked = None;

    egui::Area::new(egui::Id::new("confirm_dialog"))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let screen_r = ctx.screen_rect();
            // Allocate response to make this Area layer active under pointer and block fallthrough
            ui.allocate_rect(screen_r, egui::Sense::click_and_drag());
            // Dark overlay
            ui.painter()
                .rect_filled(screen_r, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 180));

            let card_w = 400.0;
            let card_h = 160.0;
            let card_rect = Rect::from_center_size(screen_r.center(), vec2(card_w, card_h));

            // Draw premium container matching standard overlay modal
            ui.painter()
                .rect_filled(card_rect, 4.0, Color32::from_rgb(0x16, 0x16, 0x16));
            ui.painter().rect_stroke(
                card_rect,
                4.0,
                Stroke::new(1.0, Color32::from_rgb(0x2d, 0x2d, 0x2d)),
            );
            theme::draw_tech_corners(ui.painter(), card_rect, theme::accent_color(ctx), 8.0);

            // Handle keyboard shortcuts
            ui.input(|i| {
                if i.key_pressed(egui::Key::Enter) {
                    clicked = Some(true);
                } else if i.key_pressed(egui::Key::Escape) {
                    clicked = Some(false);
                }
            });

            let mut modal_ui =
                ui.new_child(egui::UiBuilder::new().max_rect(card_rect.shrink(20.0)));

            modal_ui.vertical(|ui| {
                ui.label(
                    RichText::new(title)
                        .color(Color32::WHITE)
                        .size(14.0)
                        .strong(),
                );

                ui.add_space(8.0);

                ui.label(
                    RichText::new(body)
                        .color(theme::secondary_text(ctx))
                        .size(11.5),
                );

                ui.add_space(20.0);

                // Buttons at bottom right (using right-to-left layout)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(10.0, 0.0);

                    // Confirm button: filled with danger color
                    let confirm_btn = egui::Button::new(
                        RichText::new(confirm_text)
                            .color(Color32::WHITE)
                            .size(12.0)
                            .strong(),
                    )
                    .fill(theme::danger_color(ctx))
                    .min_size(vec2(100.0, 28.0));

                    if ui.add(confirm_btn).clicked() {
                        clicked = Some(true);
                    }

                    // Cancel button: filled with elevated background
                    let cancel_btn = egui::Button::new(
                        RichText::new(cancel_text)
                            .color(theme::primary_text(ctx))
                            .size(12.0),
                    )
                    .fill(theme::elevated_color(ctx))
                    .min_size(vec2(100.0, 28.0));

                    if ui.add(cancel_btn).clicked() {
                        clicked = Some(false);
                    }
                });
            });
        });

    clicked
}
