use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::Engine;
use mouser_engine::config::Config;
use crate::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointScrollPopup {
    PointerSpeed,
    ThumbwheelSpeed,
}

thread_local! {
    pub static ACTIVE_POINT_SCROLL_POPUP: std::cell::RefCell<Option<PointScrollPopup>> = const { std::cell::RefCell::new(None) };
}

pub fn show_point_scroll_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    mouse_texture: &egui::TextureHandle,
) {
    let rect = ui.max_rect();
    let center = rect.center() + vec2(30.0, -10.0);

    // 1. Draw Clean Mouse Image
    let img_size = vec2(600.0, 600.0);
    let mouse_rect = Rect::from_center_size(center, img_size);
    ui.put(
        mouse_rect,
        egui::Image::new(mouse_texture).fit_to_exact_size(img_size),
    );

    let mut settings_dirty = false;

    // Define card and dot positions
    let scroll_wheel_dot = center + vec2(80.0, -190.0);
    let scroll_wheel_card_pos = center + vec2(270.0, -140.0);

    let thumb_wheel_dot = center + vec2(0.0, -23.0);
    let thumb_wheel_card_pos = center + vec2(-180.0, 30.0);

    let pointer_dot = center + vec2(110.0, 40.0);
    let pointer_card_pos = center + vec2(240.0, 40.0);

    // 2. Draw annotation rings/dots (white stroke circles)
    ui.painter().circle_stroke(scroll_wheel_dot, 8.0, Stroke::new(1.5, Color32::WHITE));
    ui.painter().circle_stroke(thumb_wheel_dot, 8.0, Stroke::new(1.5, Color32::WHITE));
    ui.painter().circle_stroke(pointer_dot, 8.0, Stroke::new(1.5, Color32::WHITE));

    // Get current settings
    let invert_v = config.settings.invert_vscroll;
    let invert_h = config.settings.invert_hscroll;
    let smart_shift = config.settings.smart_shift_enabled;
    let dpi = config.settings.dpi;

    let pointer_speed_pct = ((dpi.clamp(200, 4000) - 200) as f32 * 100.0 / 3800.0).round() as i32;
    let thumb_speed_pct = ((11 - config.settings.hscroll_threshold.clamp(1, 10)) * 10).clamp(10, 100);

    // Draw Card 1: Scroll wheel
    let scroll_dir_str = if invert_v { "Scroll direction: Inverted" } else { "Scroll direction: Standard" };
    let smart_shift_str = if smart_shift { "SmartShift: On" } else { "SmartShift: Off" };
    let scroll_rows = &[
        scroll_dir_str,
        "Smooth scrolling: Off",
        smart_shift_str,
    ];
    let mut scroll_hovered_row = None;
    let _scroll_card_rect = Rect::from_center_size(scroll_wheel_card_pos, vec2(160.0, 72.0));
    let scroll_res = draw_point_scroll_card(
        ui,
        scroll_wheel_card_pos,
        160.0,
        72.0,
        "Scroll wheel",
        scroll_rows,
        &mut scroll_hovered_row,
    );

    if scroll_res.clicked() {
        if let Some(row_idx) = scroll_hovered_row {
            match row_idx {
                0 => {
                    config.settings.invert_vscroll = !config.settings.invert_vscroll;
                    settings_dirty = true;
                }
                2 => {
                    config.settings.smart_shift_enabled = !config.settings.smart_shift_enabled;
                    settings_dirty = true;
                }
                _ => {}
            }
        }
    }

    // Draw Card 2: Thumb wheel
    let thumb_dir_str = if invert_h { "Scroll direction: Inverted" } else { "Scroll direction: Default" };
    let thumb_speed_str = format!("Speed: {}%", thumb_speed_pct);
    let thumb_rows = &[
        thumb_speed_str.as_str(),
        thumb_dir_str,
    ];
    let mut thumb_hovered_row = None;
    let thumb_card_rect = Rect::from_center_size(thumb_wheel_card_pos, vec2(160.0, 58.0));
    let thumb_res = draw_point_scroll_card(
        ui,
        thumb_wheel_card_pos,
        160.0,
        58.0,
        "Thumb wheel",
        thumb_rows,
        &mut thumb_hovered_row,
    );

    if thumb_res.clicked() {
        if let Some(row_idx) = thumb_hovered_row {
            match row_idx {
                0 => {
                    ACTIVE_POINT_SCROLL_POPUP.with(|p| {
                        let mut val = p.borrow_mut();
                        if *val == Some(PointScrollPopup::ThumbwheelSpeed) {
                            *val = None;
                        } else {
                            *val = Some(PointScrollPopup::ThumbwheelSpeed);
                        }
                    });
                }
                1 => {
                    config.settings.invert_hscroll = !config.settings.invert_hscroll;
                    settings_dirty = true;
                }
                _ => {}
            }
        }
    }

    // Draw Card 3: Pointer speed
    let pointer_speed_str = format!("Speed: {}%", pointer_speed_pct);
    let pointer_rows = &[
        pointer_speed_str.as_str(),
    ];
    let mut pointer_hovered_row = None;
    let pointer_card_rect = Rect::from_center_size(pointer_card_pos, vec2(145.0, 44.0));
    let pointer_res = draw_point_scroll_card(
        ui,
        pointer_card_pos,
        145.0,
        44.0,
        "Pointer speed",
        pointer_rows,
        &mut pointer_hovered_row,
    );

    if pointer_res.clicked() {
        ACTIVE_POINT_SCROLL_POPUP.with(|p| {
            let mut val = p.borrow_mut();
            if *val == Some(PointScrollPopup::PointerSpeed) {
                *val = None;
            } else {
                *val = Some(PointScrollPopup::PointerSpeed);
            }
        });
    }

    // Render active popups
    let active_popup = ACTIVE_POINT_SCROLL_POPUP.with(|p| *p.borrow());
    if let Some(popup) = active_popup {
        match popup {
            PointScrollPopup::PointerSpeed => {
                let popup_pos = pointer_card_pos + vec2(0.0, 70.0);
                let mut current_dpi = config.settings.dpi;
                let (changed, should_close) = draw_slider_popup(
                    ui,
                    popup_pos,
                    &mut current_dpi,
                    200..=4000,
                    "Pointer speed (DPI)",
                    pointer_card_rect,
                );
                if changed {
                    config.settings.dpi = current_dpi;
                    settings_dirty = true;
                }
                if should_close {
                    ACTIVE_POINT_SCROLL_POPUP.with(|p| *p.borrow_mut() = None);
                }
            }
            PointScrollPopup::ThumbwheelSpeed => {
                let popup_pos = thumb_wheel_card_pos + vec2(0.0, 75.0);
                let mut current_speed = thumb_speed_pct;
                let (changed, should_close) = draw_slider_popup(
                    ui,
                    popup_pos,
                    &mut current_speed,
                    10..=100,
                    "Thumb Wheel Speed (%)",
                    thumb_card_rect,
                );
                if changed {
                    let rounded_speed = ((current_speed as f32 / 10.0).round() * 10.0) as i32;
                    let threshold = 11 - (rounded_speed / 10);
                    config.settings.hscroll_threshold = threshold.clamp(1, 10);
                    settings_dirty = true;
                }
                if should_close {
                    ACTIVE_POINT_SCROLL_POPUP.with(|p| *p.borrow_mut() = None);
                }
            }
        }
    }

    if settings_dirty {
        engine.update_global_settings(
            config.settings.dpi as u32,
            config.settings.smart_shift_mode.clone(),
            config.settings.smart_shift_enabled,
            config.settings.smart_shift_threshold as u8,
            config.settings.invert_hscroll,
            config.settings.invert_vscroll,
            config.settings.gesture_threshold,
            config.settings.gesture_deadzone,
            config.settings.accent_color.clone(),
        );
    }
}

fn draw_point_scroll_card(
    ui: &mut egui::Ui,
    pos: egui::Pos2,
    width: f32,
    height: f32,
    title: &str,
    rows: &[&str],
    hovered_row_idx: &mut Option<usize>,
) -> egui::Response {
    let rect = Rect::from_center_size(pos, vec2(width, height));
    let response = ui.allocate_rect(rect, egui::Sense::click());

    let is_hovered = response.hovered();
    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    // Card background & border
    let bg_fill = if is_hovered {
        Color32::from_rgb(0x24, 0x24, 0x24)
    } else {
        Color32::from_rgb(0x16, 0x16, 0x16)
    };

    let border_color = if is_hovered {
        Color32::from_rgb(0x3c, 0x3c, 0x3c)
    } else {
        Color32::from_rgb(0x26, 0x26, 0x26)
    };

    ui.painter().rect_filled(rect, 2.0, bg_fill);
    ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.0, border_color));

    // Tech corners animation
    let is_hov_anim = ui.ctx().animate_bool(response.id.with("hov"), is_hovered);
    let corner_color = theme::lerp_color(
        Color32::TRANSPARENT,
        theme::accent_color(ui.ctx()),
        is_hov_anim,
    );
    theme::draw_tech_corners(ui.painter(), rect, corner_color, 4.0);

    // Draw lines of text
    let left_x = rect.left() + 10.0;
    let mut current_y = rect.top() + 7.0;

    // Draw title
    let title_color = theme::primary_text(ui.ctx());
    let title_galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            title.to_string(),
            egui::FontId::proportional(10.5),
            title_color,
        ))
    });
    ui.painter().galley(pos2(left_x, current_y), title_galley, title_color);
    current_y += 14.0;

    // Draw rows
    let mouse_pos = ui.input(|i| i.pointer.interact_pos());
    for (idx, &row_text) in rows.iter().enumerate() {
        let row_rect = Rect::from_min_max(
            pos2(rect.left(), current_y),
            pos2(rect.right(), current_y + 14.0),
        );
        let is_row_hovered = if let Some(m_pos) = mouse_pos {
            row_rect.contains(m_pos) && is_hovered
        } else {
            false
        };

        if is_row_hovered {
            *hovered_row_idx = Some(idx);
            // Draw a subtle row hover background highlight
            ui.painter().rect_filled(
                row_rect.shrink2(vec2(4.0, 0.0)),
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 10),
            );
        }

        let text_color = if is_row_hovered {
            theme::accent_color(ui.ctx())
        } else {
            theme::muted_text(ui.ctx())
        };

        let row_galley = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                row_text.to_string(),
                egui::FontId::proportional(9.0),
                text_color,
            ))
        });
        ui.painter().galley(pos2(left_x, current_y + 1.0), row_galley, text_color);
        current_y += 14.0;
    }

    response
}

fn draw_slider_popup(
    ui: &mut egui::Ui,
    pos: egui::Pos2,
    value: &mut i32,
    range: std::ops::RangeInclusive<i32>,
    label: &str,
    card_rect: Rect,
) -> (bool, bool) {
    let popup_rect = Rect::from_center_size(pos, vec2(180.0, 60.0));

    ui.painter().rect_filled(popup_rect, 6.0, Color32::from_rgb(0x1a, 0x1a, 0x1a));
    ui.painter().rect_stroke(popup_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(0x2a, 0x2a, 0x2a)));

    let mut popup_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(popup_rect.shrink(10.0))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );

    popup_ui.label(RichText::new(label).color(Color32::WHITE).size(10.5));
    popup_ui.add_space(4.0);

    let mut changed = false;
    popup_ui.horizontal(|ui| {
        let res = ui.add(egui::Slider::new(value, range).show_value(true));
        if res.changed() {
            changed = true;
        }
    });

    let mut should_close = false;
    if ui.input(|i| i.pointer.any_click()) {
        if let Some(m_pos) = ui.input(|i| i.pointer.interact_pos()) {
            if !popup_rect.contains(m_pos) && !card_rect.contains(m_pos) {
                should_close = true;
            }
        }
    }

    (changed, should_close)
}
