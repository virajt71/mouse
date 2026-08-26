use crate::theme;
use crate::widgets::{draw_battery_widget, draw_connection_icon_mini};
use eframe::egui;
use egui::{pos2, vec2, Rect, Stroke};

pub fn draw_status_pill(
    ui: &mut egui::Ui,
    status_rect: Rect,
    is_connected: bool,
    battery_pct: &str,
    battery_status: &str,
    conn_type: &str,
    lang: &str,
    is_sidebar: bool,
) {
    let ctx = ui.ctx();
    let id = ui.make_persistent_id(format!("status_pill_{:?}", status_rect.min));
    let status_res = ui.interact(status_rect, id, egui::Sense::hover());
    let status_res = if is_connected {
        if battery_status.is_empty() {
            status_res.on_hover_text(format!("{}%", battery_pct))
        } else {
            status_res.on_hover_text(format!("{}% · {}", battery_pct, battery_status))
        }
    } else {
        status_res
    };
    let t = ctx.animate_bool(status_res.id, status_res.hovered());

    let bg = theme::lerp_color(
        theme::surface_color(ctx),
        if ctx.style().visuals.dark_mode {
            egui::Color32::from_rgb(0x1c, 0x1c, 0x1c)
        } else {
            egui::Color32::from_rgb(0xf3, 0xf4, 0xf6)
        },
        t,
    );
    let border = theme::lerp_color(
        theme::border_color(ctx),
        if is_connected {
            theme::COLOR_ACCENT_DIM
        } else {
            theme::border_color(ctx)
        },
        t,
    );

    let painter = ui.painter();
    // Subtle elevation shadow
    let shadow_rect = status_rect
        .expand2(egui::vec2(1.5, 2.0))
        .translate(egui::vec2(0.0, 1.5));
    let shadow_color = if ctx.style().visuals.dark_mode {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 12)
    };
    painter.rect_filled(shadow_rect, 3.0, shadow_color);

    painter.rect_filled(status_rect, 2.0, bg);
    painter.rect_stroke(status_rect, 2.0, Stroke::new(1.0 + 0.5 * t, border));

    let cx = status_rect.center().x;
    let cy = status_rect.center().y;

    if is_connected {
        let level = battery_pct.parse::<f32>().unwrap_or(100.0) / 100.0;
        let batt_rect = if is_sidebar {
            Rect::from_center_size(pos2(cx - 19.0, cy), vec2(20.0, 10.0))
        } else {
            Rect::from_min_max(
                pos2(status_rect.min.x + 10.0, status_rect.min.y),
                pos2(cx - 8.0, status_rect.max.y),
            )
        };
        draw_battery_widget(painter, batt_rect, level);

        // Charging bolt when the HID++ status says so
        let is_charging = battery_status.contains("charging") || battery_status.contains("recharging");
        if is_charging && battery_status != "charging_error" {
            let bolt_color = if level <= 0.20 {
                theme::COLOR_DOT_RED
            } else {
                theme::COLOR_ACCENT
            };
            let bolt_center = pos2(batt_rect.max.x + 5.0, cy);
            painter.text(
                bolt_center,
                egui::Align2::CENTER_CENTER,
                "⚡",
                egui::FontId::proportional(10.0),
                bolt_color,
            );
        }

        // Thin vertical divider
        let div_x = cx;
        painter.line_segment(
            [
                pos2(div_x, cy - (if is_sidebar { 9.0 } else { 12.0 })),
                pos2(div_x, cy + (if is_sidebar { 9.0 } else { 12.0 })),
            ],
            Stroke::new(1.0, theme::divider_color(ctx)),
        );

        // Connection icon
        let conn_center = if is_sidebar {
            pos2(cx + 19.0, cy)
        } else {
            pos2(cx + (status_rect.max.x - cx) / 2.0, cy)
        };
        draw_connection_icon_mini(painter, conn_center, conn_type);
    } else {
        // "INACTIVE" label centered
        let label_font = egui::FontId::proportional(if is_sidebar { 10.0 } else { 11.5 });
        let inactive_text = crate::translation::tr("inactive", lang);
        painter.text(
            status_rect.center() - egui::vec2(0.0, 1.0),
            egui::Align2::CENTER_CENTER,
            inactive_text,
            label_font,
            theme::COLOR_INACTIVE_TEXT,
        );
    }
}
