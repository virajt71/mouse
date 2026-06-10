use eframe::egui;
use egui::{vec2, RichText};
use mouser_engine::config::Config;
use mouser_engine::Engine;
use crate::theme;
use crate::widgets::draw_profiles_icon_settings;
use super::{section_card, render_spaced_header};

pub fn render_section_profiles(ui: &mut egui::Ui, config: &mut Config, engine: &Engine) {
    section_card(ui, |ui| {
        // ── Section header ──
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            draw_profiles_icon_settings(ui, icon_rect, theme::accent_color(ui.ctx()));
            ui.add_space(6.0);
            render_spaced_header(
                ui,
                "PROFILES",
                14.0,
                theme::primary_text(ui.ctx()),
            );
        });

        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.label("Selected Profile:");
            let mut sorted_keys: Vec<String> = config.profiles.keys().cloned().collect();
            sorted_keys.sort();

            let mut editing_profile = crate::views::customization::SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());
            if editing_profile.is_empty() {
                editing_profile = config.active_profile.clone();
                crate::views::customization::SELECTED_EDIT_PROFILE.with(|p| *p.borrow_mut() = editing_profile.clone());
                if let Some(prof) = config.profiles.get(&editing_profile) {
                    crate::views::customization::APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = prof.apps.join(", "));
                }
            }

            let combo = egui::ComboBox::from_id_salt("settings_editing_profile_combo")
                .selected_text(RichText::new(&editing_profile).color(theme::primary_text(ui.ctx())));
            let res = combo.show_ui(ui, |ui| {
                let mut changed = false;
                for p_name in &sorted_keys {
                    let label = if *p_name == config.active_profile {
                        format!("★ {}", p_name)
                    } else {
                        p_name.clone()
                    };
                    if ui.selectable_value(&mut editing_profile, p_name.clone(), label).clicked() {
                        changed = true;
                    }
                }
                changed
            });

            if let Some(true) = res.inner {
                crate::views::customization::SELECTED_EDIT_PROFILE.with(|p| *p.borrow_mut() = editing_profile.clone());
                if let Some(prof) = config.profiles.get(&editing_profile) {
                    crate::views::customization::APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = prof.apps.join(", "));
                }
            }

            // Button to activate if not active
            if editing_profile != config.active_profile {
                if ui.button("Activate").clicked() {
                    engine.select_profile(&editing_profile);
                    config.active_profile = editing_profile.clone();
                }
            } else {
                ui.label(RichText::new("Active").size(11.0).color(theme::accent_color(ui.ctx())));
            }
        });

        ui.add_space(14.0);

        let editing_profile = crate::views::customization::SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());
        if !editing_profile.is_empty() {
            // Delete profile
            if editing_profile == "default" {
                ui.add_enabled(false, egui::Button::new("Delete Profile"));
                ui.label(RichText::new("The 'default' profile cannot be deleted.").size(10.0).color(theme::muted_text(ui.ctx())));
            } else {
                if ui.button(RichText::new("Delete Profile").color(theme::danger_color(ui.ctx()))).clicked() {
                    engine.delete_profile(&editing_profile);
                    crate::views::customization::SELECTED_EDIT_PROFILE.with(|p| p.borrow_mut().clear());
                }
            }
        }

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(14.0);

        // Add profile form
        ui.label("Create New Profile:");
        ui.horizontal(|ui| {
            crate::views::customization::NEW_PROFILE_NAME.with(|name_cell| {
                let mut name_ref = name_cell.borrow_mut();
                ui.text_edit_singleline(&mut *name_ref);
                if ui.button("Add").clicked() {
                    let clean = name_ref.trim();
                    if !clean.is_empty() {
                        engine.add_profile(clean);
                        name_ref.clear();
                    }
                }
            });
        });
    });
}
