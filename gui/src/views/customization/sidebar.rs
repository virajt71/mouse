use crate::theme;
use crate::views::customization::mappings::CustomizingButton;
use crate::widgets::{
    draw_backlighting_icon, draw_equalizer_icon, draw_flow_icon,
    draw_hamburger_icon, draw_keys_icon, draw_mouse_outline_icon, draw_settings_slider_icon,
};
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Buttons,
    PointAndScroll,
    Flow,
    Settings,
}

pub fn draw_sidebar(
    ui: &mut egui::Ui,
    sidebar_rect: Rect,
    customization_tab: &mut SidebarTab,
    customizing_button: &mut Option<CustomizingButton>,
    is_keyboard: bool,
) {
    let mut sidebar_ui = ui.new_child(egui::UiBuilder::new().max_rect(sidebar_rect));

    sidebar_ui.vertical(|ui| {
        ui.add_space(40.0);

        let tab_width = 170.0;
        let tab_height = 36.0;

        let tabs = if is_keyboard {
            &[
                (SidebarTab::Buttons, "KEYS"),
                (SidebarTab::PointAndScroll, "BACKLIGHTING"),
                (SidebarTab::Settings, "SETTINGS"),
            ][..]
        } else {
            &[
                (SidebarTab::Buttons, "BUTTONS"),
                (SidebarTab::PointAndScroll, "POINT AND SCROLL"),
                (SidebarTab::Flow, "FLOW"),
                (SidebarTab::Settings, "SETTINGS"),
            ][..]
        };

        for &(tab, label) in tabs {
            let is_active = tab == *customization_tab;

            ui.horizontal(|ui| {
                ui.add_space(20.0);

                let (tab_rect, tab_res) =
                    ui.allocate_exact_size(vec2(tab_width, tab_height), egui::Sense::click());
                if tab_res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }

                let bg_color = if is_active {
                    theme::accent_color(ui.ctx())
                } else if tab_res.hovered() {
                    theme::hover_color(ui.ctx())
                } else {
                    Color32::TRANSPARENT
                };

                ui.painter().rect_filled(tab_rect, 2.0, bg_color);

                let tab_hov_anim = ui
                    .ctx()
                    .animate_bool(tab_res.id.with("tab_hov"), tab_res.hovered());
                let tab_act_anim = ui.ctx().animate_bool(tab_res.id.with("tab_act"), is_active);
                let tab_t = tab_hov_anim.max(tab_act_anim);
                let tab_corner_color =
                    theme::lerp_color(Color32::TRANSPARENT, theme::accent_color(ui.ctx()), tab_t);
                theme::draw_tech_corners(ui.painter(), tab_rect, tab_corner_color, 4.0);

                let text_color = if is_active {
                    Color32::BLACK
                } else {
                    theme::primary_text(ui.ctx())
                };

                let icon_center = pos2(tab_rect.min.x + 18.0, tab_rect.center().y);
                if is_keyboard {
                    match tab {
                        SidebarTab::Buttons => draw_keys_icon(ui, icon_center, text_color),
                        SidebarTab::PointAndScroll => {
                            draw_backlighting_icon(ui, icon_center, text_color)
                        }
                        SidebarTab::Flow => {}
                        SidebarTab::Settings => {
                            draw_settings_slider_icon(ui, icon_center, text_color)
                        }
                    }
                } else {
                    match tab {
                        SidebarTab::Buttons => draw_equalizer_icon(ui, icon_center, text_color),
                        SidebarTab::PointAndScroll => {
                            draw_mouse_outline_icon(ui, icon_center, text_color)
                        }
                        SidebarTab::Flow => draw_flow_icon(ui, icon_center, text_color),
                        SidebarTab::Settings => draw_hamburger_icon(ui, icon_center, text_color),
                    }
                }

                let text_pos = pos2(tab_rect.min.x + 36.0, tab_rect.center().y);
                let text_galley = ui.fonts(|f| {
                    f.layout_job(egui::text::LayoutJob::simple_singleline(
                        label.to_string(),
                        egui::FontId::proportional(11.0),
                        text_color,
                    ))
                });
                ui.painter().galley(
                    pos2(text_pos.x, text_pos.y - text_galley.size().y / 2.0),
                    text_galley,
                    text_color,
                );

                if tab_res.clicked() {
                    *customization_tab = tab;
                    *customizing_button = None;
                }
            });
            ui.add_space(12.0);
        }
    });
}
