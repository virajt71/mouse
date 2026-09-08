use crate::theme;
use crate::views::customization::mappings::{
    get_button_keys, get_button_option, mapping_to_action, save_button_option, CustomizingButton,
    UniversalButtonOption,
};
use crate::views::customization::popups::{
    draw_button_action_popup, draw_gesture_config_ui, draw_record_shortcut_ui,
    draw_thumbwheel_action_popup, get_thumbwheel_option, save_thumbwheel_option, PopupView,
    ThumbwheelOption,
};
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, Stroke};
use mouser_engine::client::EngineClient as Engine;
use mouser_engine::config::Config;

pub fn show_buttons_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &Config,
    mouse_texture: &egui::TextureHandle,
    customizing_button: &mut Option<CustomizingButton>,
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

    // 2. Fetch active mappings
    let fallback_profile = mouser_engine::config::Profile::default();
    let profile = config
        .get_profile(&config.active_app_profile)
        .or_else(|| config.get_profile("global"))
        .unwrap_or(&fallback_profile);

    let middle_val = mapping_to_action(CustomizingButton::Middle, &profile.mappings);
    let top_val = mapping_to_action(CustomizingButton::Top, &profile.mappings);
    let forward_val = mapping_to_action(CustomizingButton::Forward, &profile.mappings);
    let back_val = mapping_to_action(CustomizingButton::Back, &profile.mappings);
    let thumbwheel_val = mapping_to_action(CustomizingButton::Thumbwheel, &profile.mappings);
    let thumb_val = mapping_to_action(CustomizingButton::Thumb, &profile.mappings);

    let items = &[
        (
            CustomizingButton::Middle,
            center + vec2(80.0, -190.0),
            center + vec2(300.0, -230.0),
            "Middle button",
            middle_val,
        ),
        (
            CustomizingButton::Top,
            center + vec2(112.5, -77.0),
            center + vec2(320.0, -40.0),
            "Top button",
            top_val,
        ),
        (
            CustomizingButton::Thumbwheel,
            center + vec2(0.0, -23.0),
            center + vec2(320.0, 60.0),
            "Thumb wheel",
            thumbwheel_val,
        ),
        (
            CustomizingButton::Forward,
            center + vec2(-50.0, -40.0),
            center + vec2(-320.0, -80.0),
            "Forward button",
            forward_val,
        ),
        (
            CustomizingButton::Back,
            center + vec2(-25.0, 40.0),
            center + vec2(-300.0, 160.0),
            "Back button",
            back_val,
        ),
        (
            CustomizingButton::Thumb,
            center + vec2(-160.0, 60.0),
            center + vec2(-320.0, 60.0),
            "Thumb button",
            thumb_val,
        ),
    ];

    let mut hovered_button = None;

    // Draw connecting lines and dots
    for &(_btn, dot_pos, card_pos, _title, _action) in items {
        let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 100));
        ui.painter().line_segment([dot_pos, card_pos], stroke);

        let card_rect = Rect::from_center_size(card_pos, vec2(145.0, 44.0));
        let is_card_hovered = ui.rect_contains_pointer(card_rect);
        let is_customizing = *customizing_button == Some(_btn);

        let t = ui.ctx().animate_bool(
            ui.id().with(_btn).with("dot_glow"),
            is_card_hovered || is_customizing,
        );
        let dot_color = theme::lerp_color(
            theme::accent_dim_color(ui.ctx()),
            theme::accent_color(ui.ctx()),
            t,
        );

        ui.painter()
            .circle_stroke(dot_pos, 8.0, Stroke::new(1.2, Color32::WHITE));
        ui.painter().circle_filled(dot_pos, 5.0, dot_color);
    }

    // Draw floating tooltip cards
    let mut popup_to_draw = None;

    for &(btn, _dot_pos, card_pos, _title, action) in items {
        let is_customizing = *customizing_button == Some(btn);
        let mut is_hovered = false;

        let subtitle = match btn {
            CustomizingButton::Middle => "Wheel button",
            CustomizingButton::Top => "Top button",
            CustomizingButton::Forward => "Forward button",
            CustomizingButton::Back => "Back button",
            CustomizingButton::Thumbwheel => "Thumb wheel",
            CustomizingButton::Thumb => "Thumb button",
            _ => "Key",
        };

        // Determine specific label
        let (base_key, _, _, _, _, _) = get_button_keys(btn);
        let mapping_str = profile
            .mappings
            .get(base_key)
            .map(|s| s.as_str())
            .unwrap_or("none");

        let primary_label = if btn == CustomizingButton::Thumbwheel {
            let opt = get_thumbwheel_option(&profile.mappings);
            if opt == ThumbwheelOption::KeyboardShortcut {
                let left_val = profile
                    .mappings
                    .get("hscroll_left")
                    .map(|s| s.as_str())
                    .unwrap_or("none");
                let right_val = profile
                    .mappings
                    .get("hscroll_right")
                    .map(|s| s.as_str())
                    .unwrap_or("none");
                let left_label = if left_val.starts_with("custom:") {
                    std::borrow::Cow::Owned(
                        left_val.strip_prefix("custom:").unwrap().to_uppercase(),
                    )
                } else {
                    std::borrow::Cow::Borrowed("NONE")
                };
                let right_label = if right_val.starts_with("custom:") {
                    std::borrow::Cow::Owned(
                        right_val.strip_prefix("custom:").unwrap().to_uppercase(),
                    )
                } else {
                    std::borrow::Cow::Borrowed("NONE")
                };
                std::borrow::Cow::Owned(format!("{} / {}", left_label, right_label))
            } else {
                std::borrow::Cow::Borrowed(opt.display_name())
            }
        } else {
            let opt = get_button_option(btn, &profile.mappings);
            if opt == UniversalButtonOption::KeyboardShortcut {
                if mapping_str.starts_with("custom:") {
                    std::borrow::Cow::Owned(
                        mapping_str.strip_prefix("custom:").unwrap().to_uppercase(),
                    )
                } else {
                    std::borrow::Cow::Borrowed("NONE")
                }
            } else {
                std::borrow::Cow::Borrowed(opt.display_name(btn))
            }
        };

        let card_res = draw_tooltip_card(
            ui,
            card_pos,
            &primary_label,
            subtitle,
            &mut is_hovered,
            is_customizing,
        );

        if is_hovered {
            hovered_button = Some(btn);
        }

        if card_res.clicked() {
            *customizing_button = Some(btn);
        }

        if is_customizing {
            popup_to_draw = Some((btn, card_pos, card_res.rect, action));
        }
    }

    if let Some(hovered) = hovered_button {
        if let Some(&(_, dot_pos, _, _, _)) = items.iter().find(|(b, _, _, _, _)| *b == hovered) {
            ui.painter().circle_stroke(
                dot_pos,
                14.0,
                Stroke::new(2.0, theme::accent_color(ui.ctx())),
            );
        }
    }

    // Draw Selection Popup Card if active
    if let Some((btn, card_pos, card_rect, _)) = popup_to_draw {
        let show_thumbwheel = btn == CustomizingButton::Thumbwheel;
        let show_thumb = btn == CustomizingButton::Thumb;
        let show_forward = btn == CustomizingButton::Forward;
        let show_back = btn == CustomizingButton::Back;
        let show_top = btn == CustomizingButton::Top;
        let show_wheel = btn == CustomizingButton::Middle;

        let is_gesture_active = if btn == CustomizingButton::Thumbwheel {
            false
        } else {
            get_button_option(btn, &profile.mappings) == UniversalButtonOption::Gesture
        };

        let view_state_id = ui.id().with(format!("popup_view_for_{:?}", btn));
        let current_view = ui
            .ctx()
            .data(|d| d.get_temp::<PopupView>(view_state_id))
            .unwrap_or({
                if is_gesture_active {
                    PopupView::GesturesConfig
                } else {
                    PopupView::ActionList
                }
            });

        let popup_w = match &current_view {
            PopupView::GesturesConfig => 240.0,
            PopupView::RecordShortcut { .. } => 200.0,
            PopupView::ActionList => {
                if show_thumbwheel
                    || show_thumb
                    || show_forward
                    || show_back
                    || show_top
                    || show_wheel
                {
                    200.0
                } else {
                    170.0
                }
            }
        };

        let popup_h = match &current_view {
            PopupView::GesturesConfig => 370.0,
            PopupView::RecordShortcut { .. } => 300.0,
            PopupView::ActionList => {
                if show_thumbwheel
                    || show_thumb
                    || show_forward
                    || show_back
                    || show_top
                    || show_wheel
                {
                    300.0
                } else {
                    190.0
                }
            }
        };

        let mut popup_rect = if card_pos.x > center.x {
            Rect::from_min_size(
                pos2(card_rect.max.x + 10.0, card_rect.min.y - 40.0),
                vec2(popup_w, popup_h),
            )
        } else {
            Rect::from_min_size(
                pos2(card_rect.min.x - popup_w - 10.0, card_rect.min.y - 40.0),
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
            PopupView::GesturesConfig => draw_gesture_config_ui(
                ui,
                engine,
                config,
                btn,
                popup_rect,
                card_rect,
                customizing_button,
            ),
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
                card_rect,
                customizing_button,
            ),
            PopupView::ActionList => {
                if show_thumbwheel {
                    let mut selected_opt = None;
                    let res = draw_thumbwheel_action_popup(
                        ui,
                        engine,
                        config,
                        popup_rect,
                        card_rect,
                        customizing_button,
                        &mut selected_opt,
                        view_state_id,
                    );

                    if let Some(opt) = selected_opt {
                        let mut mappings = profile.mappings.clone();
                        if opt == ThumbwheelOption::KeyboardShortcut {
                            // Leave it in ActionList view; inline buttons handle transitions.
                            save_thumbwheel_option(opt, &mut mappings);
                            engine.update_profile_mappings(&config.active_app_profile, mappings);
                        } else {
                            save_thumbwheel_option(opt, &mut mappings);
                            engine.update_profile_mappings(&config.active_app_profile, mappings);
                            *customizing_button = None;
                        }
                    }
                    res
                } else {
                    let mut selected_opt = None;
                    let res = draw_button_action_popup(
                        ui,
                        btn,
                        engine,
                        config,
                        popup_rect,
                        card_rect,
                        customizing_button,
                        &mut selected_opt,
                        view_state_id,
                    );

                    if let Some(opt) = selected_opt {
                        let mut mappings = profile.mappings.clone();
                        if opt == UniversalButtonOption::Gesture {
                            // Persist gesture_enabled = "true"
                            save_button_option(btn, opt, &mut mappings);
                            engine.update_profile_mappings(&config.active_app_profile, mappings);
                            // Then show the gesture configuration panel
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(view_state_id, PopupView::GesturesConfig)
                            });
                        } else if opt == UniversalButtonOption::KeyboardShortcut {
                            let (click_key, _, _, _, _, _) = get_button_keys(btn);
                            let display_label = match btn {
                                CustomizingButton::Thumb => "Thumb Button",
                                CustomizingButton::Forward => "Forward Button",
                                CustomizingButton::Back => "Back Button",
                                CustomizingButton::Top => "Top Button",
                                CustomizingButton::Middle => "Wheel Button",
                                _ => "",
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
                            engine.update_profile_mappings(&config.active_app_profile, mappings);
                            *customizing_button = None;
                        }
                    }
                    res
                }
            }
        };

        if clicked_away {
            *customizing_button = None;
        }
    }
}

fn draw_tooltip_card(
    ui: &mut egui::Ui,
    pos: egui::Pos2,
    title: &str,
    subtitle: &str,
    is_hovered: &mut bool,
    is_active: bool,
) -> egui::Response {
    let card_w = 145.0;
    let card_h = 44.0;
    let rect = Rect::from_center_size(pos, vec2(card_w, card_h));
    let response = ui.allocate_rect(rect, egui::Sense::click());

    *is_hovered = response.hovered();
    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    let bg_fill = if is_active || response.hovered() {
        Color32::from_rgb(0x24, 0x24, 0x24)
    } else {
        Color32::from_rgb(0x16, 0x16, 0x16)
    };

    let border_color = if is_active {
        theme::accent_color(ui.ctx())
    } else if response.hovered() {
        Color32::from_rgb(0x3c, 0x3c, 0x3c)
    } else {
        Color32::from_rgb(0x26, 0x26, 0x26)
    };

    ui.painter().rect_filled(rect, 2.0, bg_fill);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border_color));

    let is_sel_anim = ui.ctx().animate_bool(response.id, is_active);
    let is_hov_anim = ui
        .ctx()
        .animate_bool(response.id.with("hov"), response.hovered());
    let corner_color = theme::lerp_color(
        Color32::TRANSPARENT,
        theme::accent_color(ui.ctx()),
        is_sel_anim.max(is_hov_anim),
    );
    theme::draw_tech_corners(ui.painter(), rect, corner_color, 4.0);

    let text_color = if is_active {
        theme::accent_color(ui.ctx())
    } else {
        theme::primary_text(ui.ctx())
    };

    let title_pos = pos2(rect.left() + 10.0, rect.top() + 7.0);
    let title_galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            title.to_string(),
            egui::FontId::proportional(10.5),
            text_color,
        ))
    });
    ui.painter().galley(title_pos, title_galley, text_color);

    let sub_pos = pos2(rect.left() + 10.0, rect.top() + 21.0);
    let sub_color = theme::muted_text(ui.ctx());
    let sub_galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            subtitle.to_string(),
            egui::FontId::proportional(9.0),
            sub_color,
        ))
    });
    ui.painter()
        .galley(sub_pos, sub_galley, theme::muted_text(ui.ctx()));

    response
}
