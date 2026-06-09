use crate::theme;
use crate::widgets::{draw_battery_widget, draw_connection_icon_mini, draw_trash_icon};
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCardAction {
    None,
    Unpair,
    Customize,
}

pub fn show_known_device(
    ui: &mut egui::Ui,
    name: &str,
    is_active: bool,
    texture: &egui::TextureHandle,
    conn_type: &str,
    battery_pct: &str,
    lang: &str,
) -> DeviceCardAction {
    let mut unpair_clicked = false;
    let mut action = DeviceCardAction::None;

    ui.vertical_centered(|ui| {
        let available = ui.available_height();
        // Content: image (300) + name gap (10) + name (20) + gap (16) + pill row (45) ≈ 391
        let content_height = 391.0;
        let top_padding = ((available - content_height) / 2.0 - 10.0).max(0.0);
        ui.add_space(top_padding);

        // ── 1. Mouse image ───────────────────────────────────────────────────
        let img_size = egui::vec2(300.0, 300.0);
        let (mouse_rect, mouse_res) = ui.allocate_exact_size(
            img_size,
            egui::Sense::click()
        );
        if mouse_res.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        if mouse_res.clicked() {
            action = DeviceCardAction::Customize;
        }

        // Only animate when the device is active/connected
        let t = if is_active {
            ui.ctx().animate_bool(mouse_res.id, mouse_res.hovered())
        } else {
            0.0
        };
        let scale = 1.0 + 0.08 * t; // Scale up by 8% on hover

        // Draw soft backglow centered on the mouse image
        if t > 0.0 {
            let glow_alpha = (25.0 * t) as u8;
            let glow_color = egui::Color32::from_rgba_unmultiplied(
                theme::COLOR_ACCENT.r(),
                theme::COLOR_ACCENT.g(),
                theme::COLOR_ACCENT.b(),
                glow_alpha,
            );
            for r in [82.5, 112.5, 142.5, 172.5] {
                ui.painter()
                    .circle_filled(mouse_rect.center(), r, glow_color);
            }
        }

        let anim_size = img_size * scale;
        let anim_rect = egui::Rect::from_center_size(mouse_rect.center(), anim_size);

        ui.put(
            anim_rect,
            egui::Image::new(texture).fit_to_exact_size(anim_size),
        );

        // ── 2. Device name ───────────────────────────────────────────────────
        ui.add_space(10.0);
        let name_color = if is_active {
            theme::primary_text(ui.ctx())
        } else {
            theme::muted_text(ui.ctx())
        };
        ui.add(
            egui::Label::new(
                egui::RichText::new(name)
                    .size(16.0)
                    .color(name_color)
                    .strong(),
            )
            .selectable(false),
        );

        // ── 3. Control row ───────────────────────────────────────────────────
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            // Centre the row manually
            let gap = 10.0;
            let pill_w = if is_active { 180.0 } else { 150.0 };
            let pill_h = 44.0;
            let trash_w = 44.0;
            let row_width = if is_active {
                pill_w
            } else {
                pill_w + gap + trash_w
            };
            let offset = (mouse_rect.width() - row_width) / 2.0;
            let cursor_x = ui.cursor().min.x;
            let target_x = mouse_rect.center().x - row_width / 2.0;
            let needed_space = (target_x - cursor_x).max(offset).max(0.0);
            ui.add_space(needed_space);

            if is_active {
                // ── Active: single pill with battery + connection ──────────
                let (pill_rect, pill_res) =
                    ui.allocate_exact_size(egui::vec2(pill_w, pill_h), egui::Sense::hover());
                let pill_res = pill_res.on_hover_text(format!("{}%", battery_pct));
                let t = ui.ctx().animate_bool(pill_res.id, pill_res.hovered());
                let bg = theme::lerp_color(
                    theme::card_bg(ui.ctx()),
                    if ui.visuals().dark_mode {
                        egui::Color32::from_rgb(0x1c, 0x1c, 0x1c)
                    } else {
                        egui::Color32::from_rgb(0xf3, 0xf4, 0xf6)
                    },
                    t,
                );
                let border =
                    theme::lerp_color(theme::card_border(ui.ctx()), theme::COLOR_ACCENT_DIM, t);

                let painter = ui.painter();
                // Subtle elevation shadow
                let shadow_rect = pill_rect
                    .expand2(egui::vec2(1.5, 2.0))
                    .translate(egui::vec2(0.0, 1.5));
                let shadow_color = if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60)
                } else {
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 12)
                };
                painter.rect_filled(shadow_rect, 3.0, shadow_color);

                painter.rect_filled(pill_rect, 2.0, bg);
                painter.rect_stroke(pill_rect, 2.0, egui::Stroke::new(1.0 + 0.5 * t, border));

                let cx = pill_rect.center().x;
                let cy = pill_rect.center().y;

                // Battery widget with percentage text
                let batt_rect = egui::Rect::from_min_max(
                    egui::pos2(pill_rect.min.x + 10.0, pill_rect.min.y),
                    egui::pos2(cx - 8.0, pill_rect.max.y),
                );
                let pct: f32 = battery_pct.parse::<f32>().unwrap_or(80.0) / 100.0;
                draw_battery_widget(painter, batt_rect, pct);

                // Thin vertical divider
                let div_x = cx;
                painter.line_segment(
                    [egui::pos2(div_x, cy - 12.0), egui::pos2(div_x, cy + 12.0)],
                    egui::Stroke::new(1.0, theme::COLOR_DIVIDER),
                );

                // Connection icon — right half
                let conn_center = egui::pos2(cx + (pill_rect.max.x - cx) / 2.0, cy);
                draw_connection_icon_mini(painter, conn_center, conn_type);
            } else {
                // ── Inactive: status pill + trash button ──────────────────
                let (pill_rect, pill_res) =
                    ui.allocate_exact_size(egui::vec2(pill_w, pill_h), egui::Sense::hover());
                let t = ui.ctx().animate_bool(pill_res.id, pill_res.hovered());
                let bg = theme::lerp_color(
                    theme::card_bg(ui.ctx()),
                    if ui.visuals().dark_mode {
                        egui::Color32::from_rgb(0x1a, 0x1a, 0x1a)
                    } else {
                        egui::Color32::from_rgb(0xf3, 0xf4, 0xf6)
                    },
                    t,
                );
                let border = theme::lerp_color(
                    theme::card_border(ui.ctx()),
                    if ui.visuals().dark_mode {
                        egui::Color32::from_rgb(0x38, 0x38, 0x38)
                    } else {
                        egui::Color32::from_rgb(0xd1, 0xd5, 0xdb)
                    },
                    t,
                );

                let painter = ui.painter();
                // Subtle elevation shadow
                let shadow_rect = pill_rect
                    .expand2(egui::vec2(1.5, 2.0))
                    .translate(egui::vec2(0.0, 1.5));
                let shadow_color = if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60)
                } else {
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 12)
                };
                painter.rect_filled(shadow_rect, 3.0, shadow_color);

                painter.rect_filled(pill_rect, 2.0, bg);
                painter.rect_stroke(pill_rect, 2.0, egui::Stroke::new(1.0, border));

                // Grey status dot (using elevated color instead of hardcoded COLOR_DOT_DARK)
                let dot_cx = pill_rect.min.x + 20.0;
                let dot_cy = pill_rect.center().y;
                painter.circle_filled(
                    egui::pos2(dot_cx, dot_cy),
                    4.0,
                    theme::elevated_color(ui.ctx()),
                );
                painter.circle_stroke(
                    egui::pos2(dot_cx, dot_cy),
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(0x44, 0x44, 0x44)),
                );

                // "Disconnected" label
                let label_font = egui::FontId::proportional(11.5);
                let text_start = egui::pos2(dot_cx + 12.0, dot_cy);
                let disc_text = crate::translation::tr("disconnected", lang);
                painter.text(
                    text_start,
                    egui::Align2::LEFT_CENTER,
                    disc_text,
                    label_font,
                    theme::COLOR_INACTIVE_TEXT,
                );

                ui.add_space(gap);

                // Trash button (Keyboard focus support)
                let (trash_rect, trash_res) = ui.allocate_exact_size(
                    egui::vec2(trash_w, pill_h),
                    egui::Sense::click().union(egui::Sense::focusable_noninteractive()),
                );
                if trash_res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }
                let tt = ui.ctx().animate_bool(trash_res.id, trash_res.hovered());
                let trash_bg = theme::lerp_color(
                    theme::card_bg(ui.ctx()),
                    if ui.visuals().dark_mode {
                        egui::Color32::from_rgb(0x28, 0x10, 0x10)
                    } else {
                        egui::Color32::from_rgb(0xfe, 0xe2, 0xe2)
                    },
                    tt,
                );
                let trash_border =
                    theme::lerp_color(theme::card_border(ui.ctx()), theme::COLOR_DOT_RED, tt);
                let trash_icon_col = theme::lerp_color(
                    if ui.visuals().dark_mode {
                        egui::Color32::from_rgb(0x60, 0x60, 0x60)
                    } else {
                        egui::Color32::from_rgb(0x9c, 0xa3, 0xaf)
                    },
                    theme::COLOR_DOT_RED,
                    tt,
                );

                let painter = ui.painter();
                painter.rect_filled(trash_rect, 2.0, trash_bg);
                painter.rect_stroke(trash_rect, 2.0, egui::Stroke::new(1.0, trash_border));
                draw_trash_icon(ui, trash_rect.center(), trash_icon_col);

                // Paint focus ring for trash button (A11y baseline)
                if trash_res.has_focus() {
                    painter.rect_stroke(
                        trash_rect.expand(2.0),
                        2.0,
                        egui::Stroke::new(2.0, theme::accent_color(ui.ctx())),
                    );
                }

                if trash_res.clicked() {
                    unpair_clicked = true;
                }
            }
        });
    });

    if unpair_clicked {
        DeviceCardAction::Unpair
    } else {
        action
    }
}
