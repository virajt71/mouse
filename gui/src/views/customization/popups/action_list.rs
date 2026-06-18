use super::PopupView;
use crate::theme;
use crate::views::customization::mappings::{
    get_button_option, CustomizingButton, UniversalButtonOption,
};
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::config::Config;
use mouser_engine::Engine;

#[allow(clippy::too_many_arguments)]
pub fn draw_button_action_popup(
    ui: &mut egui::Ui,
    btn: CustomizingButton,
    _engine: &Engine,
    config: &Config,
    rect: Rect,
    card_rect: Rect,
    customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<UniversalButtonOption>,
    view_state_id: egui::Id,
) -> bool {
    let _response = ui.allocate_rect(rect, egui::Sense::click());

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

    let profile = config.get_profile(&config.active_app_profile).unwrap();
    let current_opt = get_button_option(btn, &profile.mappings);

    let mut click_occurred = false;

    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    let salt = match btn {
        CustomizingButton::Thumb => "thumb_button_scroll",
        CustomizingButton::Forward => "forward_button_scroll",
        CustomizingButton::Back => "back_button_scroll",
        CustomizingButton::Top => "top_button_scroll",
        CustomizingButton::Middle => "wheel_button_scroll",
        _ => "button_scroll",
    };

    egui::ScrollArea::vertical()
        .id_salt(salt)
        .show(&mut child_ui, |ui| {
            ui.add_space(4.0);

            // RECOMMENDED header
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("RECOMMENDED")
                        .font(egui::FontId::proportional(9.0))
                        .strong()
                        .color(theme::secondary_text(ui.ctx())),
                );
            });
            ui.add_space(4.0);

            let recommended = btn.recommended_options();

            for &opt in recommended {
                if draw_button_item(
                    ui,
                    btn,
                    opt,
                    current_opt,
                    profile,
                    customizing_button,
                    selected_option,
                    view_state_id,
                ) {
                    click_occurred = true;
                }
            }

            ui.add_space(8.0);

            // OTHER ACTIONS header
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("OTHER ACTIONS")
                        .font(egui::FontId::proportional(9.0))
                        .strong()
                        .color(theme::secondary_text(ui.ctx())),
                );
            });
            ui.add_space(4.0);

            let other = btn.other_options();

            for &opt in other {
                if draw_button_item(
                    ui,
                    btn,
                    opt,
                    current_opt,
                    profile,
                    customizing_button,
                    selected_option,
                    view_state_id,
                ) {
                    click_occurred = true;
                }
            }

            ui.add_space(4.0);
        });

    clicked_away && !click_occurred
}

#[allow(clippy::too_many_arguments)]
fn draw_button_item(
    ui: &mut egui::Ui,
    btn: CustomizingButton,
    opt: UniversalButtonOption,
    current_opt: UniversalButtonOption,
    profile: &mouser_engine::config::Profile,
    _customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<UniversalButtonOption>,
    view_state_id: egui::Id,
) -> bool {
    let is_selected = opt == current_opt;
    let item_h = 24.0;

    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), item_h), egui::Sense::click());
    let is_hovered = response.hovered();

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        ui.painter()
            .rect_filled(rect, 0.0, theme::hover_color(ui.ctx()));
    }

    let bullet_center = pos2(rect.min.x + 14.0, rect.center().y);
    if is_selected {
        let accent = theme::accent_color(ui.ctx());
        ui.painter().circle_filled(bullet_center, 6.0, accent);
        ui.painter()
            .circle_filled(bullet_center, 2.0, theme::surface_color(ui.ctx()));
    } else {
        let bullet_color = Color32::from_gray(60);
        ui.painter().circle_filled(bullet_center, 6.0, bullet_color);
    }

    let text_color = if is_selected {
        theme::accent_color(ui.ctx())
    } else if is_hovered {
        theme::primary_text(ui.ctx())
    } else {
        theme::secondary_text(ui.ctx())
    };

    let label_text = opt.display_name(btn);
    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            label_text.to_string(),
            egui::FontId::proportional(11.0),
            text_color,
        ))
    });
    let text_y = rect.center().y - galley.size().y / 2.0;
    ui.painter()
        .galley(pos2(rect.min.x + 28.0, text_y), galley, text_color);

    let mut clicked = false;
    if response.clicked() {
        *selected_option = Some(opt);
        clicked = true;
    }

    if opt == UniversalButtonOption::KeyboardShortcut && is_selected {
        let (base_key, _, _) = btn.config_keys();
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(28.0);

            let val = profile
                .mappings
                .get(base_key)
                .map(|s| s.as_str())
                .unwrap_or("none");
            let keys_text = if val.starts_with("custom:") {
                std::borrow::Cow::Owned(val.strip_prefix("custom:").unwrap().to_uppercase())
            } else {
                std::borrow::Cow::Borrowed("Record Keystroke")
            };

            let btn_rec = ui.add(egui::Button::new(RichText::new(keys_text).size(10.0)));
            if btn_rec.clicked() {
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
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
                            target_key: base_key.to_string(),
                            display_label: display_label.to_string(),
                        },
                    )
                });
                clicked = true;
            }
        });
        ui.add_space(6.0);
    }

    clicked
}
