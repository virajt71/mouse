use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::Engine;
use mouser_engine::config::Config;
use crate::theme;
use crate::views::customization::mappings::CustomizingButton;
use super::PopupView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThumbwheelOption {
    HorizontalScroll,
    Zoom,
    Volume,
    NavTabs,
    KeyboardShortcut,
    Brightness,
    DoNothing,
    ForwardBack,
    NavApps,
    NextPrev,
    SwitchDesktops,
}

impl ThumbwheelOption {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::HorizontalScroll => "Horizontal scroll",
            Self::Zoom => "Zoom in/out",
            Self::Volume => "Volume up/down",
            Self::NavTabs => "Navigate between tabs",
            Self::KeyboardShortcut => "Keyboard shortcut",
            Self::Brightness => "Brightness up/down",
            Self::DoNothing => "Do nothing",
            Self::ForwardBack => "Forward/Back",
            Self::NavApps => "Navigate between apps",
            Self::NextPrev => "Next/Previous",
            Self::SwitchDesktops => "Switch between desktops",
        }
    }
}

pub fn get_thumbwheel_option(mappings: &std::collections::HashMap<String, String>) -> ThumbwheelOption {
    if let Some(hscroll_val) = mappings.get("hscroll") {
        match hscroll_val.as_str() {
            "zoom" => return ThumbwheelOption::Zoom,
            "volume" => return ThumbwheelOption::Volume,
            "nav_tabs" => return ThumbwheelOption::NavTabs,
            "custom" => return ThumbwheelOption::KeyboardShortcut,
            "brightness" => return ThumbwheelOption::Brightness,
            "none" => return ThumbwheelOption::DoNothing,
            "forward_back" => return ThumbwheelOption::ForwardBack,
            "nav_apps" => return ThumbwheelOption::NavApps,
            "next_prev" => return ThumbwheelOption::NextPrev,
            "switch_desktops" => return ThumbwheelOption::SwitchDesktops,
            _ => {}
        }
    }
    ThumbwheelOption::HorizontalScroll
}

pub fn save_thumbwheel_option(opt: ThumbwheelOption, mappings: &mut std::collections::HashMap<String, String>) {
    match opt {
        ThumbwheelOption::HorizontalScroll => {
            mappings.insert("hscroll".to_string(), "hscroll".to_string());
            mappings.insert("hscroll_left".to_string(), "none".to_string());
            mappings.insert("hscroll_right".to_string(), "none".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::Zoom => {
            mappings.insert("hscroll".to_string(), "zoom".to_string());
            mappings.insert("hscroll_left".to_string(), "zoom_out".to_string());
            mappings.insert("hscroll_right".to_string(), "zoom_in".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::Volume => {
            mappings.insert("hscroll".to_string(), "volume".to_string());
            mappings.insert("hscroll_left".to_string(), "volume_down".to_string());
            mappings.insert("hscroll_right".to_string(), "volume_up".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::NavTabs => {
            mappings.insert("hscroll".to_string(), "nav_tabs".to_string());
            mappings.insert("hscroll_left".to_string(), "tab_prev".to_string());
            mappings.insert("hscroll_right".to_string(), "tab_next".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::KeyboardShortcut => {
            mappings.insert("hscroll".to_string(), "custom".to_string());
            if !mappings.contains_key("hscroll_left") {
                mappings.insert("hscroll_left".to_string(), "none".to_string());
            }
            if !mappings.contains_key("hscroll_right") {
                mappings.insert("hscroll_right".to_string(), "none".to_string());
            }
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::Brightness => {
            mappings.insert("hscroll".to_string(), "brightness".to_string());
            mappings.insert("hscroll_left".to_string(), "brightness_down".to_string());
            mappings.insert("hscroll_right".to_string(), "brightness_up".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::DoNothing => {
            mappings.insert("hscroll".to_string(), "none".to_string());
            mappings.insert("hscroll_left".to_string(), "none".to_string());
            mappings.insert("hscroll_right".to_string(), "none".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::ForwardBack => {
            mappings.insert("hscroll".to_string(), "forward_back".to_string());
            mappings.insert("hscroll_left".to_string(), "browser_back".to_string());
            mappings.insert("hscroll_right".to_string(), "browser_forward".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::NavApps => {
            mappings.insert("hscroll".to_string(), "nav_apps".to_string());
            mappings.insert("hscroll_left".to_string(), "app_prev".to_string());
            mappings.insert("hscroll_right".to_string(), "app_next".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::NextPrev => {
            mappings.insert("hscroll".to_string(), "next_prev".to_string());
            mappings.insert("hscroll_left".to_string(), "prev_track".to_string());
            mappings.insert("hscroll_right".to_string(), "next_track".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::SwitchDesktops => {
            mappings.insert("hscroll".to_string(), "switch_desktops".to_string());
            mappings.insert("hscroll_left".to_string(), "space_left".to_string());
            mappings.insert("hscroll_right".to_string(), "space_right".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_thumbwheel_action_popup(
    ui: &mut egui::Ui,
    _engine: &Engine,
    config: &Config,
    rect: Rect,
    card_rect: Rect,
    customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<ThumbwheelOption>,
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

    let profile = config.profiles.get(&config.active_profile).unwrap();
    let current_opt = get_thumbwheel_option(&profile.mappings);

    let mut click_occurred = false;

    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::ScrollArea::vertical()
        .id_salt("thumbwheel_scroll")
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

            let recommended = &[
                ThumbwheelOption::HorizontalScroll,
                ThumbwheelOption::Zoom,
                ThumbwheelOption::Volume,
                ThumbwheelOption::NavTabs,
                ThumbwheelOption::KeyboardShortcut,
            ];

            for &opt in recommended {
                if draw_thumbwheel_item(ui, opt, current_opt, profile, customizing_button, selected_option, view_state_id) {
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

            let other = &[
                ThumbwheelOption::Brightness,
                ThumbwheelOption::DoNothing,
                ThumbwheelOption::ForwardBack,
                ThumbwheelOption::NavApps,
                ThumbwheelOption::NextPrev,
                ThumbwheelOption::SwitchDesktops,
            ];

            for &opt in other {
                if draw_thumbwheel_item(ui, opt, current_opt, profile, customizing_button, selected_option, view_state_id) {
                    click_occurred = true;
                }
            }

            ui.add_space(4.0);
        });

    clicked_away && !click_occurred
}

#[allow(clippy::too_many_arguments)]
fn draw_thumbwheel_item(
    ui: &mut egui::Ui,
    opt: ThumbwheelOption,
    current_opt: ThumbwheelOption,
    profile: &mouser_engine::config::Profile,
    _customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<ThumbwheelOption>,
    view_state_id: egui::Id,
) -> bool {
    let is_selected = opt == current_opt;
    let item_h = 24.0;

    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), item_h), egui::Sense::click());
    let is_hovered = response.hovered();

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(rect, 0.0, theme::hover_color(ui.ctx()));
    }

    let bullet_center = pos2(rect.min.x + 14.0, rect.center().y);
    if is_selected {
        let accent = theme::accent_color(ui.ctx());
        ui.painter().circle_filled(bullet_center, 6.0, accent);
        ui.painter().circle_filled(bullet_center, 2.0, theme::surface_color(ui.ctx()));
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

    let label_text = opt.display_name();
    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            label_text.to_string(),
            egui::FontId::proportional(11.0),
            text_color,
        ))
    });
    let text_y = rect.center().y - galley.size().y / 2.0;
    ui.painter().galley(pos2(rect.min.x + 28.0, text_y), galley, text_color);

    let mut clicked = false;
    if response.clicked() {
        *selected_option = Some(opt);
        clicked = true;
    }

    if opt == ThumbwheelOption::KeyboardShortcut && is_selected {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(28.0);

            let left_val = profile.mappings.get("hscroll_left").map(|s| s.as_str()).unwrap_or("none");
            let left_text = if left_val.starts_with("custom:") {
                std::borrow::Cow::Owned(left_val.strip_prefix("custom:").unwrap().to_uppercase())
            } else {
                std::borrow::Cow::Borrowed("Record Left")
            };

            let btn_left = ui.add(egui::Button::new(
                RichText::new(format!("Left: {}", left_text)).size(10.0)
            ));
            if btn_left.clicked() {
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
                ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::RecordShortcut {
                    target_key: "hscroll_left".to_string(),
                    display_label: "Scroll Left".to_string(),
                }));
                clicked = true;
            }

            let right_val = profile.mappings.get("hscroll_right").map(|s| s.as_str()).unwrap_or("none");
            let right_text = if right_val.starts_with("custom:") {
                std::borrow::Cow::Owned(right_val.strip_prefix("custom:").unwrap().to_uppercase())
            } else {
                std::borrow::Cow::Borrowed("Record Right")
            };

            let btn_right = ui.add(egui::Button::new(
                RichText::new(format!("Right: {}", right_text)).size(10.0)
            ));
            if btn_right.clicked() {
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
                ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::RecordShortcut {
                    target_key: "hscroll_right".to_string(),
                    display_label: "Scroll Right".to_string(),
                }));
                clicked = true;
            }
        });
        ui.add_space(6.0);
    }

    clicked
}
