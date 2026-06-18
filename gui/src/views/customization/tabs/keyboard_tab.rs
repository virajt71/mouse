use crate::theme;
use crate::views::customization::mappings::{
    save_button_option, CustomizingButton, UniversalButtonOption,
};
use crate::views::customization::popups::{
    draw_button_action_popup, draw_record_shortcut_ui, PopupView,
};
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::config::Config;
use mouser_engine::Engine;

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
    let img_size = vec2(800.0, 266.0);
    let keyboard_rect = Rect::from_center_size(center, img_size);
    ui.put(
        keyboard_rect,
        egui::Image::new(keyboard_texture).fit_to_exact_size(img_size),
    );

    // Fetch active profile mappings
    let profile = config
        .get_profile(&config.active_app_profile)
        .unwrap_or_else(|| config.get_profile("global").unwrap());

    // Defining coordinates (rx, ry, rw, rh) for 22 customizable keys:
    // 12 F-keys: F1 to F12
    // 6 Nav keys: Ins, Home, PgUp, Del, End, PgDn
    // 4 Numpad keys: Calculator, ScreenLock, Search, LockPower
    let key_mappings = &[
        (CustomizingButton::F1, 0.055, 0.15, 0.033, 0.10),
        (CustomizingButton::F2, 0.100, 0.15, 0.033, 0.10),
        (CustomizingButton::F3, 0.142, 0.15, 0.033, 0.10),
        (CustomizingButton::F4, 0.190, 0.15, 0.033, 0.10),
        (CustomizingButton::F5, 0.235, 0.15, 0.033, 0.10),
        (CustomizingButton::F6, 0.280, 0.15, 0.033, 0.10),
        (CustomizingButton::F7, 0.315, 0.15, 0.033, 0.10),
        (CustomizingButton::F8, 0.360, 0.15, 0.033, 0.10),
        (CustomizingButton::F9, 0.400, 0.15, 0.033, 0.10),
        (CustomizingButton::F10, 0.445, 0.15, 0.033, 0.10),
        (CustomizingButton::F11, 0.490, 0.15, 0.033, 0.10),
        (CustomizingButton::F12, 0.535, 0.15, 0.033, 0.10),
        (CustomizingButton::Ins, 0.725, 0.31, 0.032, 0.075),
        (CustomizingButton::Home, 0.762, 0.31, 0.032, 0.075),
        (CustomizingButton::PgUp, 0.799, 0.31, 0.032, 0.075),
        (CustomizingButton::Del, 0.725, 0.47, 0.032, 0.075),
        (CustomizingButton::End, 0.762, 0.47, 0.032, 0.075),
        (CustomizingButton::PgDn, 0.799, 0.47, 0.032, 0.075),
        (CustomizingButton::Calculator, 0.842, 0.15, 0.033, 0.10),
        (CustomizingButton::ScreenLock, 0.879, 0.15, 0.033, 0.10),
        (CustomizingButton::Search, 0.916, 0.15, 0.033, 0.10),
        (CustomizingButton::LockPower, 0.953, 0.15, 0.033, 0.10),
    ];

    // Draw overlays on keyboard keys
    for &(btn, rx, ry, rw, rh) in key_mappings {
        let key_rect = Rect::from_min_max(
            keyboard_rect.min + vec2(rx * keyboard_rect.width(), ry * keyboard_rect.height()),
            keyboard_rect.min
                + vec2(
                    (rx + rw) * keyboard_rect.width(),
                    (ry + rh) * keyboard_rect.height(),
                ),
        );

        let is_hovered = ui.rect_contains_pointer(key_rect);
        if is_hovered {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }

        let is_customizing = *customizing_button == Some(btn);

        // Check if this key is mapped (i.e. mapping is not "none")
        let (base_key, _, _) = btn.config_keys();
        let val = profile
            .mappings
            .get(base_key)
            .map(|s| s.as_str())
            .unwrap_or("none");
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

        let stroke_width = if is_customizing || is_hovered {
            1.8
        } else {
            1.0
        };

        ui.painter().rect(
            key_rect,
            4.0,
            fill_color,
            Stroke::new(stroke_width, stroke_color),
        );

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
                keyboard_rect.min
                    + vec2(
                        (rx + rw) * keyboard_rect.width(),
                        (ry + rh) * keyboard_rect.height(),
                    ),
            );

            let view_state_id = ui.id().with(btn).with("keyboard_popup_view");
            let current_view = ui
                .ctx()
                .data(|d| d.get_temp(view_state_id))
                .unwrap_or(PopupView::ActionList);

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
                popup_rect =
                    popup_rect.translate(vec2(canvas_min_x + 10.0 - popup_rect.min.x, 0.0));
            }
            if popup_rect.max.x > canvas_max_x - 10.0 {
                popup_rect =
                    popup_rect.translate(vec2((canvas_max_x - 10.0) - popup_rect.max.x, 0.0));
            }
            if popup_rect.min.y < canvas_min_y + 10.0 {
                popup_rect =
                    popup_rect.translate(vec2(0.0, canvas_min_y + 10.0 - popup_rect.min.y));
            }
            if popup_rect.max.y > canvas_max_y - 10.0 {
                popup_rect =
                    popup_rect.translate(vec2(0.0, (canvas_max_y - 10.0) - popup_rect.max.y));
            }

            let clicked_away = match &current_view {
                PopupView::RecordShortcut {
                    target_key,
                    display_label,
                } => draw_record_shortcut_ui(
                    ui,
                    engine,
                    config,
                    btn,
                    target_key.clone(),
                    display_label.clone(),
                    popup_rect,
                    key_rect,
                    customizing_button,
                ),
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
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(
                                    view_state_id,
                                    PopupView::RecordShortcut {
                                        target_key: click_key.to_string(),
                                        display_label: display_label.to_string(),
                                    },
                                )
                            });
                        } else {
                            save_button_option(btn, opt, &mut mappings);
                            let engine_bg = engine.clone();
                            let profile_bg = config.active_app_profile.clone();
                            std::thread::spawn(move || {
                                engine_bg.update_profile_mappings(&profile_bg, mappings);
                            });
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

fn toggle_switch(ui: &mut egui::Ui, enabled: &mut bool) -> egui::Response {
    let desired_size = vec2(36.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *enabled = !*enabled;
        response.mark_changed();
    }
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *enabled, ""));

    let how_on = ui.ctx().animate_bool(response.id, *enabled);
    let painter = ui.painter();
    let track_color = if *enabled {
        theme::accent_color(ui.ctx())
    } else {
        Color32::from_rgb(0x33, 0x33, 0x33)
    };
    painter.rect_filled(rect, 10.0, track_color);

    let circle_radius = 8.0;
    let x = rect.min.x + circle_radius + 2.0 + how_on * (rect.width() - 2.0 * circle_radius - 4.0);
    let y = rect.center().y;
    painter.circle_filled(pos2(x, y), circle_radius, Color32::WHITE);

    response
}

fn draw_custom_radio(
    ui: &mut egui::Ui,
    selected: &mut String,
    value: &str,
    label: &str,
) -> egui::Response {
    let is_selected = *selected == value;
    let size = vec2(180.0, 26.0);
    let (rect, mut res) = ui.allocate_exact_size(size, egui::Sense::click());
    if res.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }
    if res.clicked() {
        *selected = value.to_string();
        res.mark_changed();
    }

    let ctx = ui.ctx();
    let active_color = theme::accent_color(ctx);
    let inactive_color = Color32::from_rgb(0x66, 0x66, 0x66);

    let circle_center = pos2(rect.min.x + 10.0, rect.center().y);
    let circle_radius = 6.0;

    // Draw outer circle
    let circle_color = if is_selected {
        active_color
    } else {
        inactive_color
    };
    ui.painter()
        .circle_stroke(circle_center, circle_radius, Stroke::new(1.5, circle_color));

    // Draw inner dot if selected
    if is_selected {
        ui.painter().circle_filled(circle_center, 3.0, active_color);
    }

    // Draw label text
    let text_color = if is_selected {
        active_color
    } else {
        theme::primary_text(ctx)
    };
    let text_pos = pos2(rect.min.x + 24.0, rect.center().y);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.5),
        text_color,
    );

    res
}

pub fn show_keyboard_backlighting_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &Config,
    keyboard_texture: &egui::TextureHandle,
) {
    let profile = config
        .get_profile(&config.active_app_profile)
        .unwrap_or_else(|| config.get_profile("global").unwrap());

    let enabled_str = profile
        .mappings
        .get("backlight_enabled")
        .map(|s| s.as_str())
        .unwrap_or("true");
    let mut enabled = enabled_str == "true";

    let effect = profile
        .mappings
        .get("backlight_effect")
        .map(|s| s.as_str())
        .unwrap_or("Static")
        .to_string();
    let mut selected_effect = effect;

    let mut changed = false;

    ui.vertical(|ui| {
        ui.add_space(20.0);

        // Header with switch and Select Effect sub-tab
        ui.horizontal(|ui| {

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(20.0); // gap between toggle and SELECT EFFECT

                // 2. SELECT EFFECT label and gear icon (clickable)
                let (eff_rect, response) = ui.allocate_exact_size(vec2(130.0, 24.0), egui::Sense::click());
                if response.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }

                let popup_id = egui::Id::new("backlight_effect_popup");
                let mut popup_open = ui.ctx().data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
                if response.clicked() {
                    popup_open = !popup_open;
                    ui.ctx().data_mut(|d| d.insert_temp(popup_id, popup_open));
                }

                let icon_center = pos2(eff_rect.min.x + 12.0, eff_rect.center().y);
                let active_color = theme::accent_color(ui.ctx());

                // Draw gear icon next to SELECT EFFECT
                let painter = ui.painter();
                let stroke = Stroke::new(1.2, active_color);
                painter.circle_stroke(icon_center, 4.0, stroke);
                for i in 0..8 {
                    let angle = (i as f32 * 45.0).to_radians();
                    let p1 = icon_center + vec2(angle.cos() * 4.0, angle.sin() * 4.0);
                    let p2 = icon_center + vec2(angle.cos() * 6.5, angle.sin() * 6.5);
                    painter.line_segment([p1, p2], stroke);
                }

                let text_pos = pos2(eff_rect.min.x + 28.0, eff_rect.center().y);
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_CENTER,
                    "SELECT EFFECT",
                    egui::FontId::proportional(11.5),
                    active_color,
                );

                // Active tab underline
                let bar_y = eff_rect.max.y + 4.0;
                painter.line_segment(
                    [pos2(eff_rect.min.x, bar_y), pos2(eff_rect.max.x, bar_y)],
                    Stroke::new(2.0, active_color),
                );

                // Draw popup if open
                if popup_open {
                    let popup_pos = eff_rect.left_bottom() + vec2(0.0, 8.0);

                    egui::Area::new(egui::Id::new("backlight_popup_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(popup_pos)
                        .show(ui.ctx(), |ui| {
                            // Draw container frame
                            let frame = egui::Frame::none()
                                .fill(Color32::from_rgb(0x13, 0x13, 0x13))
                                .stroke(Stroke::new(1.0, theme::border_color(ui.ctx())))
                                .rounding(4.0)
                                .inner_margin(12.0);

                            let response_frame = frame.show(ui, |ui| {
                                ui.set_width(260.0);
                                ui.vertical(|ui| {
                                    ui.add(egui::Label::new(
                                        RichText::new("Select Backlight Effect")
                                            .color(Color32::WHITE)
                                            .size(13.0)
                                            .strong(),
                                    ));
                                    ui.add_space(6.0);
                                    ui.add(egui::Label::new(
                                        RichText::new("Backlighting Effects")
                                            .color(theme::primary_text(ui.ctx()))
                                            .size(11.0)
                                            .strong(),
                                    ));
                                    ui.add_space(4.0);
                                    ui.add(egui::Label::new(
                                        RichText::new("Choose between different backlighting effects for your keyboard by pressing \"Fn + Lightbulb\" keys or choose from the list below")
                                            .color(theme::muted_text(ui.ctx()))
                                            .size(10.0)
                                    ));
                                    ui.add_space(10.0);

                                    // Options list
                                    if draw_custom_radio(ui, &mut selected_effect, "Static", "Static").changed() {
                                        changed = true;
                                    }
                                    ui.add_space(6.0);
                                    if draw_custom_radio(ui, &mut selected_effect, "Contrast", "Contrast").changed() {
                                        changed = true;
                                    }
                                    ui.add_space(6.0);
                                    if draw_custom_radio(ui, &mut selected_effect, "Breathing", "Breathing").changed() {
                                        changed = true;
                                    }
                                    ui.add_space(6.0);
                                    if draw_custom_radio(ui, &mut selected_effect, "Waves", "Waves").changed() {
                                        changed = true;
                                    }
                                    ui.add_space(6.0);
                                    if draw_custom_radio(ui, &mut selected_effect, "Reaction", "Reaction").changed() {
                                        changed = true;
                                    }
                                    ui.add_space(6.0);
                                    if draw_custom_radio(ui, &mut selected_effect, "Random", "Random").changed() {
                                        changed = true;
                                    }
                                });
                            });

                            // Check click away to close
                            let interact_pos = ui.input(|i| i.pointer.interact_pos());
                            if ui.input(|i| i.pointer.any_click()) {
                                if let Some(pos) = interact_pos {
                                    let clicked_inside_frame = response_frame.response.rect.contains(pos);
                                    let clicked_inside_button = eff_rect.contains(pos);
                                    if !clicked_inside_frame && !clicked_inside_button {
                                        ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
                                    }
                                }
                            }
                        });
                }

                ui.add_space(20.0);

                ui.label(
                    RichText::new("BACKLIGHTING")
                        .color(Color32::WHITE)
                        .size(12.0)
                        .strong(),
                );

                ui.add_space(8.0);

                // 1. Toggle switch at the right edge
                if toggle_switch(ui, &mut enabled).changed() {
                    changed = true;
                }
            });
        });

        ui.add_space(20.0);

        // Main layout container: Keyboard Image
        let max_rect = ui.max_rect();
        let center = max_rect.center() + vec2(0.0, -10.0);
        let img_size = vec2(800.0, 266.0);
        let keyboard_rect = Rect::from_center_size(center, img_size);

        ui.put(
            keyboard_rect,
            egui::Image::new(keyboard_texture).fit_to_exact_size(img_size),
        );

        // Highlighted keys outlines on standard keyboard image
        // 1. Bulb key highlight (to the right of F12)
        let b_rx = 0.655;
        let b_ry = 0.20;
        let b_rw = 0.032;
        let b_rh = 0.075;
        let bulb_rect = Rect::from_min_max(
            keyboard_rect.min + vec2(b_rx * keyboard_rect.width(), b_ry * keyboard_rect.height()),
            keyboard_rect.min + vec2((b_rx + b_rw) * keyboard_rect.width(), (b_ry + b_rh) * keyboard_rect.height()),
        );
        ui.painter().rect_stroke(
            bulb_rect.expand(1.0),
            3.0,
            Stroke::new(1.8, Color32::WHITE),
        );

        // 2. Fn key highlight (bottom row)
        let fn_rx = 0.44;
        let fn_ry = 0.85;
        let fn_rw = 0.032;
        let fn_rh = 0.075;
        let fn_rect = Rect::from_min_max(
            keyboard_rect.min + vec2(fn_rx * keyboard_rect.width(), fn_ry * keyboard_rect.height()),
            keyboard_rect.min + vec2((fn_rx + fn_rw) * keyboard_rect.width(), (fn_ry + fn_rh) * keyboard_rect.height()),
        );
        ui.painter().rect_stroke(
            fn_rect.expand(1.0),
            3.0,
            Stroke::new(1.8, Color32::WHITE),
        );
    });

    if changed {
        let mut mappings = profile.mappings.clone();
        mappings.insert("backlight_enabled".to_string(), enabled.to_string());
        mappings.insert("backlight_effect".to_string(), selected_effect);

        let engine_bg = engine.clone();
        let profile_bg = config.active_app_profile.clone();
        std::thread::spawn(move || {
            engine_bg.update_profile_mappings(&profile_bg, mappings);
        });
    }
}

pub fn show_keyboard_easy_switch_tab(ui: &mut egui::Ui, engine: &Engine, config: &Config) {
    let profile = config
        .get_profile(&config.active_app_profile)
        .unwrap_or_else(|| config.get_profile("global").unwrap());

    let mut changed = false;

    // Load or set default channel names
    let mut ch1 = profile
        .mappings
        .get("easy_switch_ch1")
        .cloned()
        .unwrap_or_else(|| "Laptop".to_string());
    let mut ch2 = profile
        .mappings
        .get("easy_switch_ch2")
        .cloned()
        .unwrap_or_else(|| "Desktop".to_string());
    let mut ch3 = profile
        .mappings
        .get("easy_switch_ch3")
        .cloned()
        .unwrap_or_else(|| "iPad".to_string());

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
                        let (dot_rect, _) =
                            ui.allocate_exact_size(vec2(15.0, 15.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 5.0, indicator_color);

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
