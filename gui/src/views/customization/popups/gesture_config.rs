use super::{PopupView, RecordingTarget, RECORDING_TARGET};
use crate::theme;
use crate::views::customization::mappings::{
    action_id_to_slot_display_name, get_button_keys, get_generic_action_id,
    resolve_generic_slot_action, CustomizingButton, GESTURE_PRESETS, SLOT_ACTIONS,
};
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::config::Config;
use mouser_engine::Engine;

pub fn draw_gesture_config_ui(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &Config,
    btn: CustomizingButton,
    rect: Rect,
    card_rect: Rect,
    customizing_button: &mut Option<CustomizingButton>,
) -> bool {
    let mut click_occurred = false;

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
        egui::Rounding {
            nw: 2.0,
            ne: 2.0,
            sw: 0.0,
            se: 0.0,
        },
        Color32::from_rgb(0, 245, 198), // Teal `#00f5c6`
    );

    let profile = config.get_profile(&config.active_app_profile).unwrap();
    let (_, _, up_key, down_key, left_key, right_key) = get_button_keys(btn);
    let click_key = get_button_keys(btn).0;

    let cur_left = profile
        .mappings
        .get(left_key)
        .map(|s| s.as_str())
        .unwrap_or("none");
    let cur_right = profile
        .mappings
        .get(right_key)
        .map(|s| s.as_str())
        .unwrap_or("none");
    let cur_up = profile
        .mappings
        .get(up_key)
        .map(|s| s.as_str())
        .unwrap_or("none");
    let cur_down = profile
        .mappings
        .get(down_key)
        .map(|s| s.as_str())
        .unwrap_or("none");
    let cur_click = profile
        .mappings
        .get(click_key)
        .map(|s| s.as_str())
        .unwrap_or("none");

    // Draw elements inside header_rect using child_ui
    let mut header_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect.shrink(6.0))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // Back Button (←)
    let (back_rect, back_res) =
        header_ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
    let back_hover_color = if back_res.hovered() {
        header_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(0, 0, 0, 20)
    } else {
        Color32::TRANSPARENT
    };
    header_ui
        .painter()
        .rect_filled(back_rect, 4.0, back_hover_color);

    let stroke = egui::Stroke::new(1.5, Color32::BLACK);
    let cx = back_rect.center().x;
    let cy = back_rect.center().y;
    header_ui
        .painter()
        .line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
    header_ui
        .painter()
        .line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy - 4.0)], stroke);
    header_ui
        .painter()
        .line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy + 4.0)], stroke);

    if back_res.clicked() {
        let view_state_id = ui.id().with(format!("popup_view_for_{:?}", btn));
        ui.ctx()
            .data_mut(|d| d.insert_temp(view_state_id, PopupView::ActionList));
        click_occurred = true;
    }

    header_ui.add_space(4.0);

    // Gestures Icon (circle outline with circle inside)
    let icon_center = header_ui.min_rect().min + vec2(28.0, 12.0);
    ui.painter().circle_filled(icon_center, 6.5, Color32::BLACK);
    ui.painter()
        .circle_filled(icon_center, 2.0, Color32::from_rgb(0, 245, 198));

    header_ui.add_space(12.0);

    // Title: Gestures
    header_ui.label(
        RichText::new("Gestures")
            .font(egui::FontId::proportional(12.0))
            .strong()
            .color(Color32::BLACK),
    );

    // Content container
    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(rect.min.x + 8.0, rect.min.y + header_h + 8.0),
                pos2(rect.max.x - 8.0, rect.max.y - 8.0),
            ))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    // Subheading description text
    content_ui.add(egui::Label::new(
        RichText::new("Choose a preset or select custom to create your own.")
            .font(egui::FontId::proportional(10.0))
            .color(theme::secondary_text(ui.ctx())),
    ));
    content_ui.add_space(8.0);

    // Determine active preset
    let mut active_preset_idx = 0; // Default to Custom
    for (idx, preset) in GESTURE_PRESETS.iter().enumerate().skip(1) {
        if cur_left == preset.left
            && cur_right == preset.right
            && cur_up == preset.up
            && cur_down == preset.down
            && cur_click == preset.click
        {
            active_preset_idx = idx;
            break;
        }
    }

    // Preset Selection Dropdown
    let combo_w = content_ui.available_width();
    let combo = egui::ComboBox::from_id_salt(ui.id().with(format!("preset_combo_{:?}", btn)))
        .width(combo_w)
        .selected_text(
            RichText::new(GESTURE_PRESETS[active_preset_idx].name)
                .font(egui::FontId::proportional(11.0))
                .color(Color32::WHITE),
        );

    let res = combo.show_ui(&mut content_ui, |ui| {
        let mut changed = false;
        let mut selected_idx = active_preset_idx;
        for (idx, preset) in GESTURE_PRESETS.iter().enumerate() {
            if ui
                .selectable_label(idx == active_preset_idx, preset.name)
                .clicked()
            {
                selected_idx = idx;
                changed = true;
            }
        }
        (changed, selected_idx)
    });

    if let Some((true, new_idx)) = res.inner {
        click_occurred = true;
        let preset = &GESTURE_PRESETS[new_idx];
        let mut mps = profile.mappings.clone();
        mps.insert(left_key.to_string(), preset.left.to_string());
        mps.insert(right_key.to_string(), preset.right.to_string());
        mps.insert(up_key.to_string(), preset.up.to_string());
        mps.insert(down_key.to_string(), preset.down.to_string());
        mps.insert(click_key.to_string(), preset.click.to_string());
        let engine_bg = engine.clone();
        let profile_bg = config.active_app_profile.clone();
        std::thread::spawn(move || {
            engine_bg.update_profile_mappings(&profile_bg, mps);
        });
    }

    content_ui.add_space(8.0);

    // 5 slots card container
    let container_rect = Rect::from_min_size(
        content_ui.cursor().min,
        vec2(content_ui.available_width(), 200.0),
    );

    // Frame background and stroke
    ui.painter()
        .rect_filled(container_rect, 4.0, theme::elevated_color(ui.ctx()));
    ui.painter().rect_stroke(
        container_rect,
        4.0,
        Stroke::new(1.0, theme::border_color(ui.ctx())),
    );

    let slots = &[
        ("left", "HOLD + MOVE LEFT", left_key, cur_left),
        ("right", "HOLD + MOVE RIGHT", right_key, cur_right),
        ("up", "HOLD + MOVE UP", up_key, cur_up),
        ("down", "HOLD + MOVE DOWN", down_key, cur_down),
        ("click", "CLICK", click_key, cur_click),
    ];

    for (i, &(dir, label, key_str, cur_val)) in slots.iter().enumerate() {
        if i > 0 {
            // Draw divider line between slots
            let y = container_rect.min.y + i as f32 * 40.0;
            ui.painter().line_segment(
                [
                    pos2(container_rect.min.x + 4.0, y),
                    pos2(container_rect.max.x - 4.0, y),
                ],
                Stroke::new(0.8, theme::border_color(ui.ctx())),
            );
        }

        let slot_rect = Rect::from_min_max(
            pos2(container_rect.min.x, container_rect.min.y + i as f32 * 40.0),
            pos2(
                container_rect.max.x,
                container_rect.min.y + (i + 1) as f32 * 40.0,
            ),
        );

        let row_id = ui.id().with(format!("slot_{:?}_{}", btn, dir));
        let response = ui.interact(slot_rect, row_id, egui::Sense::click());
        let is_hovered = response.hovered();

        if is_hovered {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            ui.painter()
                .rect_filled(slot_rect, 0.0, theme::hover_color(ui.ctx()));
        }

        // Left Icon
        let icon_pos = slot_rect.left_center() + vec2(14.0, 0.0);
        let icon_color = theme::primary_text(ui.ctx());
        match dir {
            "left" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter()
                    .line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
                ui.painter()
                    .line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy - 4.0)], stroke);
                ui.painter()
                    .line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy + 4.0)], stroke);
            }
            "right" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter()
                    .line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
                ui.painter()
                    .line_segment([pos2(cx + 5.0, cy), pos2(cx + 1.0, cy - 4.0)], stroke);
                ui.painter()
                    .line_segment([pos2(cx + 5.0, cy), pos2(cx + 1.0, cy + 4.0)], stroke);
            }
            "up" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter()
                    .line_segment([pos2(cx, cy - 5.0), pos2(cx, cy + 5.0)], stroke);
                ui.painter()
                    .line_segment([pos2(cx, cy - 5.0), pos2(cx - 4.0, cy - 1.0)], stroke);
                ui.painter()
                    .line_segment([pos2(cx, cy - 5.0), pos2(cx + 4.0, cy - 1.0)], stroke);
            }
            "down" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter()
                    .line_segment([pos2(cx, cy - 5.0), pos2(cx, cy + 5.0)], stroke);
                ui.painter()
                    .line_segment([pos2(cx, cy + 5.0), pos2(cx - 4.0, cy + 1.0)], stroke);
                ui.painter()
                    .line_segment([pos2(cx, cy + 5.0), pos2(cx + 4.0, cy + 1.0)], stroke);
            }
            _ => {
                ui.painter()
                    .circle_stroke(icon_pos, 4.0, Stroke::new(1.5, icon_color));
            }
        }

        // Text Labels: slot name and current action display name
        let label_pos = slot_rect.left_center() + vec2(28.0, -8.0);
        let action_pos = slot_rect.left_center() + vec2(28.0, 6.0);

        ui.painter().text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(7.5),
            theme::secondary_text(ui.ctx()),
        );

        let disp_name = action_id_to_slot_display_name(cur_val);
        ui.painter().text(
            action_pos,
            egui::Align2::LEFT_CENTER,
            &disp_name,
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );

        // Click handler to open dropdown action selector for this slot
        let generic_action = get_generic_action_id(cur_val);
        let dropdown_id = ui.id().with(format!("dropdown_{:?}_{}", btn, dir));

        // Hidden combobox over the row to handle slot customization
        let mut slot_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(slot_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        // Make the button transparent / frameless
        let widgets = &mut slot_ui.style_mut().visuals.widgets;
        widgets.inactive.bg_fill = Color32::TRANSPARENT;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.inactive.bg_stroke = Stroke::NONE;
        widgets.hovered.bg_fill = Color32::TRANSPARENT;
        widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
        widgets.hovered.bg_stroke = Stroke::NONE;
        widgets.active.bg_fill = Color32::TRANSPARENT;
        widgets.active.weak_bg_fill = Color32::TRANSPARENT;
        widgets.active.bg_stroke = Stroke::NONE;

        let slot_combo = egui::ComboBox::from_id_salt(dropdown_id)
            .icon(|_, _, _, _, _| {}) // Empty closure to remove default arrow icon
            .width(slot_rect.width());

        let res = slot_combo.show_ui(&mut slot_ui, |ui| {
            let mut changed = false;
            let mut selected_act = generic_action.to_string();
            for &(act_id, act_disp) in SLOT_ACTIONS {
                if ui
                    .selectable_label(generic_action == act_id, act_disp)
                    .clicked()
                {
                    selected_act = act_id.to_string();
                    changed = true;
                }
            }
            (changed, selected_act)
        });

        if let Some((true, act_val)) = res.inner {
            click_occurred = true;
            if act_val == "custom" {
                // Open keyboard recording modal
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
                if dir == "click" {
                    RECORDING_TARGET.with(|r| *r.borrow_mut() = Some(RecordingTarget::Button(btn)));
                } else {
                    RECORDING_TARGET.with(|r| {
                        *r.borrow_mut() = Some(RecordingTarget::Gesture(btn, dir.to_string()))
                    });
                }
                *customizing_button = None;
            } else {
                let resolved = resolve_generic_slot_action(&act_val, dir);
                let mut mps = profile.mappings.clone();
                mps.insert(key_str.to_string(), resolved);
                let engine_bg = engine.clone();
                let profile_bg = config.active_app_profile.clone();
                std::thread::spawn(move || {
                    engine_bg.update_profile_mappings(&profile_bg, mps);
                });
            }
        }
    }

    clicked_away && !click_occurred
}
