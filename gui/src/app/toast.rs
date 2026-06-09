use super::MouserApp;
use eframe::egui;
use crate::theme;

pub const TOAST_DURATION_SECS: f32 = 2.5;

impl MouserApp {
    pub fn draw_toast(&mut self, ctx: &egui::Context) {
        if let (Some(msg), Some(shown_at)) = (&self.toast_message, self.toast_shown_at) {
            let elapsed = shown_at.elapsed().as_secs_f32();
            if elapsed < TOAST_DURATION_SECS {
                let alpha = ((TOAST_DURATION_SECS - elapsed) / 0.4).clamp(0.0, 1.0); // fade last 0.4s
                egui::Area::new(egui::Id::new("profile_toast"))
                    .order(egui::Order::Foreground)
                    .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-24.0, -24.0))
                    .show(ctx, |ui| {
                        let toast_bg = egui::Color32::from_rgba_unmultiplied(0x18, 0x18, 0x18, (220.0 * alpha) as u8);
                        let accent = theme::COLOR_ACCENT;
                        let border = egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), (180.0 * alpha) as u8);
                        let text_col = egui::Color32::from_rgba_unmultiplied(0xf0, 0xf0, 0xf0, (255.0 * alpha) as u8);
                        
                        egui::Frame::none()
                            .fill(toast_bg)
                            .stroke(egui::Stroke::new(1.0, border))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(msg.as_str()).color(text_col).size(13.0));
                            });
                    });
                ctx.request_repaint_after(std::time::Duration::from_millis(16));
            } else {
                self.toast_message = None;
                self.toast_shown_at = None;
            }
        }
    }
}
