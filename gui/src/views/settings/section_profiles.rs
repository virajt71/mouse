use super::{render_spaced_header, section_card};
use crate::theme;
use crate::widgets::draw_profiles_icon_settings;
use eframe::egui;
use egui::{vec2, RichText};
use mouser_engine::client::EngineClient as Engine;
use mouser_engine::config::Config;

thread_local! {
    pub static SELECTED_EDIT_GROUP: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static NEW_GROUP_NAME: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static CONFIRM_DELETE_GROUP: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

pub fn render_section_profiles(ui: &mut egui::Ui, config: &mut Config, engine: &Engine) {
    section_card(ui, |ui| {
        // ── Section header ──
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            draw_profiles_icon_settings(ui, icon_rect, theme::accent_color(ui.ctx()));
            ui.add_space(6.0);
            render_spaced_header(ui, "PROFILE GROUPS", 14.0, theme::primary_text(ui.ctx()));
        });

        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.label("Active Profile Group:");
            let mut sorted_keys: Vec<String> = config.profile_groups.keys().cloned().collect();
            sorted_keys.sort();

            let mut editing_group = SELECTED_EDIT_GROUP.with(|g| g.borrow().clone());
            if editing_group.is_empty() {
                editing_group = config.active_group.clone();
                SELECTED_EDIT_GROUP.with(|g| *g.borrow_mut() = editing_group.clone());
            }

            let combo = egui::ComboBox::from_id_salt("settings_editing_group_combo")
                .selected_text(RichText::new(&editing_group).color(theme::primary_text(ui.ctx())));
            let res = combo.show_ui(ui, |ui| {
                let mut changed = false;
                for g_name in &sorted_keys {
                    let label = if *g_name == config.active_group {
                        format!("★ {}", g_name)
                    } else {
                        g_name.clone()
                    };
                    if ui
                        .selectable_value(&mut editing_group, g_name.clone(), label)
                        .clicked()
                    {
                        changed = true;
                    }
                }
                changed
            });

            if let Some(true) = res.inner {
                SELECTED_EDIT_GROUP.with(|g| *g.borrow_mut() = editing_group.clone());
            }

            // Button to activate if not active
            if editing_group != config.active_group {
                if ui.button("Activate").clicked() {
                    engine.select_profile_group(&editing_group);
                    config.active_group = editing_group.clone();
                    config.active_app_profile = "global".to_string();
                }
            } else {
                ui.label(
                    RichText::new("Active")
                        .size(11.0)
                        .color(theme::accent_color(ui.ctx())),
                );
            }
        });

        ui.add_space(14.0);

        let editing_group = SELECTED_EDIT_GROUP.with(|g| g.borrow().clone());
        if !editing_group.is_empty() {
            // Delete group
            if editing_group == "default" {
                ui.add_enabled(false, egui::Button::new("Delete Group"));
                ui.label(
                    RichText::new("The 'default' group cannot be deleted.")
                        .size(10.0)
                        .color(theme::muted_text(ui.ctx())),
                );
            } else {
                if ui
                    .button(
                        RichText::new("Delete Profile Group").color(theme::danger_color(ui.ctx())),
                    )
                    .clicked()
                {
                    CONFIRM_DELETE_GROUP.with(|c| *c.borrow_mut() = Some(editing_group.clone()));
                }
            }
        }

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(14.0);

        // Add group form
        ui.label("Create New Profile Group:");
        ui.horizontal(|ui| {
            NEW_GROUP_NAME.with(|name_cell| {
                let mut name_ref = name_cell.borrow_mut();
                ui.text_edit_singleline(&mut *name_ref);
                if ui.button("Add Group").clicked() {
                    let clean = name_ref.trim();
                    if !clean.is_empty() {
                        engine.add_profile_group(clean);
                        name_ref.clear();
                    }
                }
            });
        });
    });

    let mut group_to_delete = None;
    CONFIRM_DELETE_GROUP.with(|c| {
        if let Some(ref group) = *c.borrow() {
            group_to_delete = Some(group.clone());
        }
    });

    if let Some(group) = group_to_delete {
        let title = format!("Delete \"{}\"?", group);
        let body =
            "This removes all button and gesture mappings in this group. This can't be undone.";
        if let Some(confirmed) = crate::widgets::show_confirm_dialog(
            ui.ctx(),
            &title,
            body,
            "Delete group",
            "Keep group",
        ) {
            if confirmed {
                engine.delete_profile_group(&group);
                SELECTED_EDIT_GROUP.with(|g| g.borrow_mut().clear());
            }
            CONFIRM_DELETE_GROUP.with(|c| *c.borrow_mut() = None);
        }
    }
}
