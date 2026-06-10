use eframe::egui;
use egui::{Color32, RichText};
use mouser_engine::Engine;
use mouser_engine::config::Config;
use crate::theme;

thread_local! {
    pub static NEW_PROFILE_NAME: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static APP_BINDINGS_BUFFER: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static SELECTED_EDIT_PROFILE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

pub fn show_profiles_settings_tab(ui: &mut egui::Ui, engine: &Engine, config: &mut Config) {
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        ui.vertical(|ui| {
            ui.add_space(40.0);
            ui.add(egui::Label::new(
                RichText::new("PROFILES MANAGEMENT")
                    .color(Color32::WHITE)
                    .size(18.0)
                    .strong(),
            ));
            ui.add_space(20.0);

            // Left Sidebar of Profile names, Right pane for Selected Profile setup
            ui.horizontal(|ui| {
                // Left list: Profiles
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(RichText::new("Existing Profiles").strong().size(12.0)));
                    ui.add_space(6.0);

                    let mut sorted_keys: Vec<String> = config.profiles.keys().cloned().collect();
                    sorted_keys.sort();

                    let scroll_h = 180.0;
                    egui::ScrollArea::vertical().max_height(scroll_h).show(ui, |ui| {
                        for p_name in sorted_keys {
                            let is_active = p_name == config.active_profile;
                            let is_editing = SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone()) == p_name;

                            ui.horizontal(|ui| {
                                let label = if is_active {
                                    RichText::new(format!("★ {}", p_name)).strong().color(theme::accent_color(ui.ctx()))
                                } else {
                                    RichText::new(&p_name).color(theme::secondary_text(ui.ctx()))
                                };

                                let select_res = ui.selectable_label(is_editing, label);
                                if select_res.clicked() {
                                    SELECTED_EDIT_PROFILE.with(|p| *p.borrow_mut() = p_name.clone());
                                    // Initialize edit buffer
                                    if let Some(prof) = config.profiles.get(&p_name) {
                                        APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = prof.apps.join(", "));
                                    }
                                }

                                if is_active {
                                    ui.label(RichText::new("(Active)").size(10.0).color(theme::muted_text(ui.ctx())));
                                } else {
                                    // Button to select it as active
                                    if ui.button("Activate").clicked() {
                                        engine.select_profile(&p_name);
                                        config.active_profile = p_name.clone();
                                    }
                                }
                            });
                            ui.add_space(4.0);
                        }
                    });

                    // Add Profile Form
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label("Add New Profile:");
                    ui.horizontal(|ui| {
                        NEW_PROFILE_NAME.with(|name_cell| {
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

                ui.add_space(40.0);
                ui.separator();
                ui.add_space(40.0);

                // Right Pane: Edit profile app mappings & deletion
                ui.vertical(|ui| {
                    let sel_profile = SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());
                    if sel_profile.is_empty() {
                        ui.label("Select a profile from the list to manage its settings.");
                    } else {
                        ui.label(RichText::new(format!("Managing Profile: {}", sel_profile)).strong().size(13.0));
                        ui.add_space(14.0);

                        // Deletion (Disabled for default)
                        if sel_profile == "default" {
                            ui.add_enabled(false, egui::Button::new("Delete Profile"));
                            ui.label(RichText::new("The 'default' profile cannot be deleted.").size(10.0).color(theme::muted_text(ui.ctx())));
                        } else {
                            if ui.button(RichText::new("Delete Profile").color(theme::COLOR_DOT_RED)).clicked() {
                                engine.delete_profile(&sel_profile);
                                SELECTED_EDIT_PROFILE.with(|p| p.borrow_mut().clear());
                            }
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(20.0);

                        // App Process Rules
                        ui.label("Target Application Process Names:");
                        ui.label(RichText::new("Comma-separated list of execution processes (e.g. chrome, code, slack)")
                            .size(10.5)
                            .color(theme::muted_text(ui.ctx())));
                        ui.add_space(6.0);

                        APP_BINDINGS_BUFFER.with(|buff_cell| {
                            let mut buff_ref = buff_cell.borrow_mut();
                            ui.text_edit_singleline(&mut *buff_ref);

                            ui.add_space(8.0);
                            if ui.button("Save App Triggers").clicked() {
                                engine.update_app_bindings(&sel_profile, &buff_ref);
                            }
                        });
                    }
                });
            });
        });
    });
}
