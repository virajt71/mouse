use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::Engine;
use mouser_engine::config::Config;
use crate::theme;
use crate::views::customization::mappings::{
    CustomizingButton, get_button_keys, egui_key_to_string, is_valid_combo,
};
use super::PopupView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingTarget {
    Button(CustomizingButton),
    Gesture(CustomizingButton, String),
}

thread_local! {
    pub static RECORDING_TARGET: std::cell::RefCell<Option<RecordingTarget>> = const { std::cell::RefCell::new(None) };
    pub static RECORDED_KEYS: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

#[allow(clippy::too_many_arguments)]
pub fn draw_record_shortcut_ui(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &Config,
    btn: CustomizingButton,
    target_key: String,
    display_label: String,
    rect: Rect,
    card_rect: Rect,
    _customizing_button: &mut Option<CustomizingButton>,
) -> bool {
    let _response = ui.allocate_rect(rect, egui::Sense::click());

    // Check click away
    let clicked_away = ui.input(|i| i.pointer.any_click())
        && !ui.rect_contains_pointer(rect)
        && !ui.rect_contains_pointer(card_rect);

    let bg = theme::surface_color(ui.ctx());
    let border = theme::border_color(ui.ctx());

    // Draw drop shadow
    let shadow_rect = rect.expand2(vec2(2.0, 3.0)).translate(vec2(0.0, 2.0));
    ui.painter().rect_filled(
        shadow_rect,
        3.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );

    // Outer popup container
    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border));
    theme::draw_tech_corners(ui.painter(), rect, theme::accent_color(ui.ctx()), 6.0);

    // Teal Header Bar
    let header_h = 36.0;
    let header_rect = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + header_h));
    ui.painter().rect_filled(
        header_rect,
        egui::Rounding { nw: 2.0, ne: 2.0, sw: 0.0, se: 0.0 },
        Color32::from_rgb(0, 245, 198), // Teal `#00f5c6`
    );

    let mut header_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect.shrink(6.0))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    let view_state_id = ui.id().with(format!("popup_view_for_{:?}", btn));

    // Back Button (←)
    let (back_rect, back_res) = header_ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
    let back_hover_color = if back_res.hovered() {
        header_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(0, 0, 0, 20)
    } else {
        Color32::TRANSPARENT
    };
    header_ui.painter().rect_filled(back_rect, 4.0, back_hover_color);

    let stroke = egui::Stroke::new(1.5, Color32::BLACK);
    let cx = back_rect.center().x;
    let cy = back_rect.center().y;
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy - 4.0)], stroke);
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy + 4.0)], stroke);

    let mut click_occurred = false;

    if back_res.clicked() {
        // Return to parent view
        let next_view = if target_key.contains("_gesture_") || target_key.starts_with("gesture_") {
            PopupView::GesturesConfig
        } else {
            PopupView::ActionList
        };
        ui.ctx().data_mut(|d| d.insert_temp(view_state_id, next_view));
        RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
        click_occurred = true;
    }

    header_ui.add_space(8.0);
    header_ui.label(
        RichText::new("Record Shortcut")
            .font(egui::FontId::proportional(12.0))
            .strong()
            .color(Color32::BLACK),
    );

    // Content area
    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(rect.min.x + 8.0, rect.min.y + header_h + 8.0),
                pos2(rect.max.x - 8.0, rect.max.y - 8.0),
            ))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );

    content_ui.add_space(8.0);
    content_ui.label(
        RichText::new(format!("Target: {}", display_label))
            .font(egui::FontId::proportional(11.0))
            .strong()
            .color(Color32::WHITE),
    );
    content_ui.add_space(8.0);

    content_ui.add(egui::Label::new(
        RichText::new("Press any key combination on your keyboard.\nPress Enter to save, or Escape to cancel.")
            .font(egui::FontId::proportional(10.0))
            .color(theme::secondary_text(ui.ctx())),
    ));

    // Listen to keyboard keys
    let mut escape_pressed = false;
    let mut enter_pressed = false;
    let mut new_keys_recorded = None;

    ui.input(|i| {
        let mods = i.modifiers;
        let has_modifiers = mods.ctrl || mods.shift || mods.alt || mods.mac_cmd;
        let mut parts = Vec::new();
        if mods.ctrl { parts.push("ctrl".to_string()); }
        if mods.shift { parts.push("shift".to_string()); }
        if mods.alt { parts.push("alt".to_string()); }
        if mods.mac_cmd { parts.push("meta".to_string()); }

        for event in &i.events {
            let detected_key = match event {
                egui::Event::Key { key, pressed: true, .. } => Some(*key),
                egui::Event::Copy => Some(egui::Key::C),
                egui::Event::Cut => Some(egui::Key::X),
                egui::Event::Paste(_) => Some(egui::Key::V),
                _ => None,
            };

            if let Some(key) = detected_key {
                if key == egui::Key::Escape && !has_modifiers {
                    escape_pressed = true;
                } else if key == egui::Key::Enter && !has_modifiers {
                    enter_pressed = true;
                } else {
                    let name = egui_key_to_string(key);
                    if !name.is_empty() && name != "ctrl" && name != "shift" && name != "alt" && name != "meta" && name != "tab" {
                        parts.push(name);
                        new_keys_recorded = Some(parts.join("+"));
                    }
                }
            }
        }
    });

    if let Some(keys) = new_keys_recorded {
        RECORDED_KEYS.with(|rk| *rk.borrow_mut() = keys);
    }

    if escape_pressed {
        let next_view = if target_key.contains("_gesture_") || target_key.starts_with("gesture_") {
            PopupView::GesturesConfig
        } else {
            PopupView::ActionList
        };
        ui.ctx().data_mut(|d| d.insert_temp(view_state_id, next_view));
        RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
    } else if enter_pressed {
        let recorded = RECORDED_KEYS.with(|rk| rk.borrow().clone());
        if is_valid_combo(&recorded) {
            let profile_name = &config.active_profile;
            if let Some(profile) = config.profiles.get(profile_name).or_else(|| config.profiles.get("default")) {
                let mut mappings = profile.mappings.clone();
                let action_str = format!("custom:{}", recorded);

                mappings.insert(target_key.clone(), action_str);

                if !target_key.contains("_gesture_") && !target_key.starts_with("gesture_") && target_key != "hscroll_left" && target_key != "hscroll_right" {
                    let (_, gesture_enabled_key, up_k, down_k, left_k, right_k) = get_button_keys(btn);
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                    for dir_key in [up_k, down_k, left_k, right_k] {
                        mappings.insert(dir_key.to_string(), "none".to_string());
                    }
                }

                let engine_bg = engine.clone();
                let profile_name_bg = profile_name.clone();
                std::thread::spawn(move || {
                    engine_bg.update_profile_mappings(&profile_name_bg, mappings);
                });
            }
        }
        let next_view = if target_key.contains("_gesture_") || target_key.starts_with("gesture_") {
            PopupView::GesturesConfig
        } else {
            PopupView::ActionList
        };
        ui.ctx().data_mut(|d| d.insert_temp(view_state_id, next_view));
        RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
    }

    let current_pressed = RECORDED_KEYS.with(|rk| rk.borrow().clone());

    content_ui.add_space(20.0);

    let display_combo = if current_pressed.is_empty() {
        "Press keys...".to_string()
    } else {
        current_pressed.to_uppercase()
    };

    let text_color = if current_pressed.is_empty() {
        theme::muted_text(ui.ctx())
    } else {
        theme::accent_color(ui.ctx())
    };

    // Keystroke combo preview box inside popup content
    let preview_w = content_ui.available_width() - 16.0;
    let (preview_rect, _) = content_ui.allocate_exact_size(vec2(preview_w, 36.0), egui::Sense::hover());
    ui.painter().rect_filled(preview_rect, 2.0, theme::elevated_color(ui.ctx()));
    ui.painter().rect_stroke(preview_rect, 2.0, Stroke::new(1.0, theme::border_color(ui.ctx())));
    ui.painter().text(
        preview_rect.center(),
        egui::Align2::CENTER_CENTER,
        &display_combo,
        egui::FontId::monospace(12.0),
        text_color,
    );

    clicked_away && !click_occurred
}
