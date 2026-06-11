use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, Stroke};
use mouser_engine::Engine;
use mouser_engine::config::Config;
use crate::theme;
use crate::views::customization::mappings::{
    CustomizingButton, UniversalButtonOption, save_button_option,
};
use crate::views::customization::popups::{
    PopupView, draw_record_shortcut_ui, draw_button_action_popup,
};

pub fn show_keyboard_keys_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &Config,
    keyboard_texture: &egui::TextureHandle,
    customizing_button: &mut Option<CustomizingButton>,
) {
    let rect = ui.max_rect();
    let center = rect.center() + vec2(0.0, -10.0);

    // 1. Draw Keyboard Image
    // Aspect ratio of standard keyboard is ~3.3 : 1
    let img_size = vec2(700.0, 210.0);
    let keyboard_rect = Rect::from_center_size(center, img_size);
    ui.put(
        keyboard_rect,
        egui::Image::new(keyboard_texture).fit_to_exact_size(img_size),
    );

    // Fetch active profile mappings
    let profile = config.get_profile(&config.active_app_profile).unwrap_or_else(|| {
        config.get_profile("global").unwrap()
    });

    // Defining coordinates (rx, ry, rw, rh) for 22 customizable keys:
    // 12 F-keys: F1 to F12
    // 6 Nav keys: Ins, Home, PgUp, Del, End, PgDn
    // 4 Numpad keys: Calculator, ScreenLock, Search, LockPower
    let key_mappings = &[
        (CustomizingButton::F1, 0.0755, 0.11, 0.032, 0.075),
        (CustomizingButton::F2, 0.124, 0.11, 0.032, 0.075),
        (CustomizingButton::F3, 0.1725, 0.11, 0.032, 0.075),
        (CustomizingButton::F4, 0.221, 0.11, 0.032, 0.075),
        (CustomizingButton::F5, 0.2695, 0.11, 0.032, 0.075),
        (CustomizingButton::F6, 0.3181, 0.11, 0.032, 0.075),
        (CustomizingButton::F7, 0.3665, 0.11, 0.032, 0.075),
        (CustomizingButton::F8, 0.415, 0.11, 0.032, 0.075),
        (CustomizingButton::F9, 0.4635, 0.11, 0.032, 0.075),
        (CustomizingButton::F10, 0.512, 0.11, 0.032, 0.075),
        (CustomizingButton::F11, 0.5605, 0.11, 0.032, 0.075),
        (CustomizingButton::F12, 0.609, 0.11, 0.032, 0.075),
        
        (CustomizingButton::Ins, 0.725, 0.31, 0.032, 0.075),
        (CustomizingButton::Home, 0.762, 0.31, 0.032, 0.075),
        (CustomizingButton::PgUp, 0.799, 0.31, 0.032, 0.075),
        (CustomizingButton::Del, 0.725, 0.47, 0.032, 0.075),
        (CustomizingButton::End, 0.762, 0.47, 0.032, 0.075),
        (CustomizingButton::PgDn, 0.799, 0.47, 0.032, 0.075),
        
        (CustomizingButton::Calculator, 0.842, 0.11, 0.032, 0.075),
        (CustomizingButton::ScreenLock, 0.879, 0.11, 0.032, 0.075),
        (CustomizingButton::Search, 0.916, 0.11, 0.032, 0.075),
        (CustomizingButton::LockPower, 0.953, 0.11, 0.032, 0.075),
    ];

    // Draw overlays on keyboard keys
    for &(btn, rx, ry, rw, rh) in key_mappings {
        let key_rect = Rect::from_min_max(
            keyboard_rect.min + vec2(rx * keyboard_rect.width(), ry * keyboard_rect.height()),
            keyboard_rect.min + vec2((rx + rw) * keyboard_rect.width(), (ry + rh) * keyboard_rect.height()),
        );

        let is_hovered = ui.rect_contains_pointer(key_rect);
        if is_hovered {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }

        let is_customizing = *customizing_button == Some(btn);

        // Check if this key is mapped (i.e. mapping is not "none")
        let (base_key, _, _) = btn.config_keys();
        let val = profile.mappings.get(base_key).map(|s| s.as_str()).unwrap_or("none");
        let is_mapped = val != "none";

        // Determine styling
        let stroke_color = if is_customizing {
            theme::accent_color(ui.ctx())
        } else if is_hovered {
            Color32::WHITE
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 120)
        };

        let fill_color = if is_customizing {
            theme::accent_color(ui.ctx()).linear_multiply(0.25)
        } else if is_hovered {
            Color32::from_rgba_unmultiplied(255, 255, 255, 40)
        } else if is_mapped {
            theme::accent_color(ui.ctx()).linear_multiply(0.15)
        } else {
            Color32::TRANSPARENT
        };

        let stroke_width = if is_customizing || is_hovered { 1.8 } else { 1.0 };

        ui.painter().rect(key_rect, 4.0, fill_color, Stroke::new(stroke_width, stroke_color));

        if is_hovered && ui.input(|i| i.pointer.any_click()) {
            *customizing_button = Some(btn);
        }
    }

    // Handle customizable button popovers
    if let Some(btn) = *customizing_button {
        // Find mapping layout entry for position
        if let Some(&(_, rx, ry, rw, rh)) = key_mappings.iter().find(|t| t.0 == btn) {
            let key_rect = Rect::from_min_max(
                keyboard_rect.min + vec2(rx * keyboard_rect.width(), ry * keyboard_rect.height()),
                keyboard_rect.min + vec2((rx + rw) * keyboard_rect.width(), (ry + rh) * keyboard_rect.height()),
            );

            let view_state_id = ui.id().with(btn).with("keyboard_popup_view");
            let current_view = ui.ctx().data(|d| d.get_temp(view_state_id)).unwrap_or(PopupView::ActionList);

            let popup_w = 300.0;
            let popup_h = 320.0;

            let mut popup_rect = if key_rect.center().x > center.x {
                Rect::from_min_size(
                    pos2(key_rect.min.x - popup_w - 10.0, key_rect.min.y - 40.0),
                    vec2(popup_w, popup_h),
                )
            } else {
                Rect::from_min_size(
                    pos2(key_rect.max.x + 10.0, key_rect.min.y - 40.0),
                    vec2(popup_w, popup_h),
                )
            };

            // Clamp inside boundary
            let canvas_min_x = rect.min.x;
            let canvas_max_x = rect.max.x;
            let canvas_min_y = rect.min.y;
            let canvas_max_y = rect.max.y;

            if popup_rect.min.x < canvas_min_x + 10.0 {
                popup_rect = popup_rect.translate(vec2(canvas_min_x + 10.0 - popup_rect.min.x, 0.0));
            }
            if popup_rect.max.x > canvas_max_x - 10.0 {
                popup_rect = popup_rect.translate(vec2((canvas_max_x - 10.0) - popup_rect.max.x, 0.0));
            }
            if popup_rect.min.y < canvas_min_y + 10.0 {
                popup_rect = popup_rect.translate(vec2(0.0, canvas_min_y + 10.0 - popup_rect.min.y));
            }
            if popup_rect.max.y > canvas_max_y - 10.0 {
                popup_rect = popup_rect.translate(vec2(0.0, (canvas_max_y - 10.0) - popup_rect.max.y));
            }

            let clicked_away = match &current_view {
                PopupView::RecordShortcut { target_key, display_label } => {
                    draw_record_shortcut_ui(
                        ui,
                        engine,
                        config,
                        btn,
                        target_key.clone(),
                        display_label.clone(),
                        popup_rect,
                        key_rect,
                        customizing_button,
                    )
                }
                PopupView::ActionList | _ => {
                    let mut selected_opt = None;
                    let res = draw_button_action_popup(
                        ui,
                        btn,
                        engine,
                        config,
                        popup_rect,
                        key_rect,
                        customizing_button,
                        &mut selected_opt,
                        view_state_id,
                    );

                    if let Some(opt) = selected_opt {
                        let mut mappings = profile.mappings.clone();
                        if opt == UniversalButtonOption::KeyboardShortcut {
                            let (click_key, _, _) = btn.config_keys();
                            let display_label = match btn {
                                CustomizingButton::F1 => "F1 Key",
                                CustomizingButton::F2 => "F2 Key",
                                CustomizingButton::F3 => "F3 Key",
                                CustomizingButton::F4 => "F4 Key",
                                CustomizingButton::F5 => "F5 Key",
                                CustomizingButton::F6 => "F6 Key",
                                CustomizingButton::F7 => "F7 Key",
                                CustomizingButton::F8 => "F8 Key",
                                CustomizingButton::F9 => "F9 Key",
                                CustomizingButton::F10 => "F10 Key",
                                CustomizingButton::F11 => "F11 Key",
                                CustomizingButton::F12 => "F12 Key",
                                CustomizingButton::Ins => "Insert Key",
                                CustomizingButton::Home => "Home Key",
                                CustomizingButton::PgUp => "Page Up Key",
                                CustomizingButton::Del => "Delete Key",
                                CustomizingButton::End => "End Key",
                                CustomizingButton::PgDn => "Page Down Key",
                                CustomizingButton::Calculator => "Calculator Key",
                                CustomizingButton::ScreenLock => "Screen Lock Key",
                                CustomizingButton::Search => "Search Key",
                                CustomizingButton::LockPower => "Lock/Power Key",
                                _ => "Key",
                            };
                            ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::RecordShortcut {
                                target_key: click_key.to_string(),
                                display_label: display_label.to_string(),
                            }));
                        } else {
                            save_button_option(btn, opt, &mut mappings);
                            let engine_bg = engine.clone();
                            let profile_bg = config.active_app_profile.clone();
                            std::thread::spawn(move || { engine_bg.update_profile_mappings(&profile_bg, mappings); });
                            *customizing_button = None;
                        }
                    }
                    res
                }
            };

            if clicked_away {
                *customizing_button = None;
            }
        }
    }
}

pub fn show_keyboard_backlighting_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &Config,
) {
    let profile = config.get_profile(&config.active_app_profile).unwrap_or_else(|| {
        config.get_profile("global").unwrap()
    });

    // Load current values or defaults
    let brightness_str = profile.mappings.get("backlight_brightness").map(|s| s.as_str()).unwrap_or("80");
    let mut brightness = brightness_str.parse::<f32>().unwrap_or(80.0);

    let enabled_str = profile.mappings.get("backlight_enabled").map(|s| s.as_str()).unwrap_or("true");
    let mut enabled = enabled_str == "true";

    let smart_str = profile.mappings.get("backlight_smart").map(|s| s.as_str()).unwrap_or("true");
    let mut smart = smart_str == "true";

    let save_power_str = profile.mappings.get("backlight_save_power").map(|s| s.as_str()).unwrap_or("false");
    let mut save_power = save_power_str == "true";

    let mut changed = false;

    ui.vertical(|ui| {
        ui.add_space(20.0);
        ui.heading("Backlighting Settings");
        ui.add_space(20.0);

        let bg = theme::surface_color(ui.ctx());
        let border = theme::border_color(ui.ctx());
        
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0, border))
            .rounding(4.0)
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Brightness:");
                        let slider = ui.add(egui::Slider::new(&mut brightness, 0.0..=100.0).text("%"));
                        if slider.changed() {
                            changed = true;
                        }
                    });
                    ui.add_space(15.0);

                    let toggle1 = ui.checkbox(&mut enabled, "Enable Backlighting");
                    if toggle1.changed() {
                        changed = true;
                    }
                    ui.add_space(10.0);

                    let toggle2 = ui.checkbox(&mut smart, "Smart Backlighting (Ambient Light)");
                    if toggle2.changed() {
                        changed = true;
                    }
                    ui.add_space(10.0);

                    let toggle3 = ui.checkbox(&mut save_power, "Save Power Mode (Disable below 10% battery)");
                    if toggle3.changed() {
                        changed = true;
                    }
                });
            });
    });

    if changed {
        let mut mappings = profile.mappings.clone();
        mappings.insert("backlight_brightness".to_string(), format!("{:.0}", brightness));
        mappings.insert("backlight_enabled".to_string(), enabled.to_string());
        mappings.insert("backlight_smart".to_string(), smart.to_string());
        mappings.insert("backlight_save_power".to_string(), save_power.to_string());

        let engine_bg = engine.clone();
        let profile_bg = config.active_app_profile.clone();
        std::thread::spawn(move || {
            engine_bg.update_profile_mappings(&profile_bg, mappings);
        });
    }
}

pub fn show_keyboard_easy_switch_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &Config,
) {
    let profile = config.get_profile(&config.active_app_profile).unwrap_or_else(|| {
        config.get_profile("global").unwrap()
    });

    let mut changed = false;

    // Load or set default channel names
    let mut ch1 = profile.mappings.get("easy_switch_ch1").cloned().unwrap_or_else(|| "Laptop".to_string());
    let mut ch2 = profile.mappings.get("easy_switch_ch2").cloned().unwrap_or_else(|| "Desktop".to_string());
    let mut ch3 = profile.mappings.get("easy_switch_ch3").cloned().unwrap_or_else(|| "iPad".to_string());

    ui.vertical(|ui| {
        ui.add_space(20.0);
        ui.heading("Easy-Switch Channels");
        ui.add_space(20.0);

        for ch_idx in 1..=3 {
            let label = match ch_idx {
                1 => "Channel 1",
                2 => "Channel 2",
                _ => "Channel 3",
            };
            let val = match ch_idx {
                1 => &mut ch1,
                2 => &mut ch2,
                _ => &mut ch3,
            };

            let bg = theme::surface_color(ui.ctx());
            let border = theme::border_color(ui.ctx());
            
            egui::Frame::none()
                .fill(bg)
                .stroke(Stroke::new(1.0, border))
                .rounding(4.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let is_active_channel = ch_idx == 1; // Simulation
                        let indicator_color = if is_active_channel {
                            theme::accent_color(ui.ctx())
                        } else {
                            Color32::from_rgb(100, 100, 100)
                        };
                        let (dot_rect, _) = ui.allocate_exact_size(vec2(15.0, 15.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 5.0, indicator_color);
                        
                        ui.add_space(5.0);
                        ui.label(label);
                        
                        ui.add_space(30.0);
                        ui.label("Device name:");
                        let text_edit = ui.text_edit_singleline(val);
                        if text_edit.changed() {
                            changed = true;
                        }
                    });
                });
            ui.add_space(10.0);
        }
    });

    if changed {
        let mut mappings = profile.mappings.clone();
        mappings.insert("easy_switch_ch1".to_string(), ch1);
        mappings.insert("easy_switch_ch2".to_string(), ch2);
        mappings.insert("easy_switch_ch3".to_string(), ch3);

        let engine_bg = engine.clone();
        let profile_bg = config.active_app_profile.clone();
        std::thread::spawn(move || {
            engine_bg.update_profile_mappings(&profile_bg, mappings);
        });
    }
}
