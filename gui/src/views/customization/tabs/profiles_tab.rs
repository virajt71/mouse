use eframe::egui;
use egui::{Color32, RichText};
use mouser_engine::Engine;
use mouser_engine::config::Config;
use crate::theme;

thread_local! {
    pub static NEW_GROUP_NAME: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static SELECTED_EDIT_GROUP: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static NEW_PROFILE_NAME: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static APP_BINDINGS_BUFFER: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static SELECTED_EDIT_PROFILE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static BINDINGS_ERROR: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

pub fn show_profiles_settings_tab(ui: &mut egui::Ui, engine: &Engine, config: &mut Config) {
    // Ensure scanned apps are loaded
    let needs_scan = crate::views::customization::popups::add_app_modal::SCANNED_APPS.with(|apps| apps.borrow().is_none());
    if needs_scan {
        crate::views::customization::popups::add_app_modal::SCANNED_APPS.with(|apps| {
            *apps.borrow_mut() = Some(crate::desktop_apps::scan_all_applications());
        });
    }

    ui.horizontal(|ui| {
        ui.add_space(40.0);
        ui.vertical(|ui| {
            ui.add_space(20.0);
            ui.add(egui::Label::new(
                RichText::new("PROFILES MANAGEMENT")
                    .color(Color32::WHITE)
                    .size(18.0)
                    .strong(),
            ));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Active Group:").size(11.0).color(theme::secondary_text(ui.ctx())));
                ui.label(RichText::new(&config.active_group).strong().size(11.0).color(theme::accent_color(ui.ctx())));
            });
            ui.add(egui::Label::new(
                RichText::new("Switch, create, or delete groups in the Profiles section under settings.")
                    .size(10.0)
                    .color(theme::muted_text(ui.ctx())),
            ));
            ui.add_space(15.0);

            let active_group = config.active_group.clone();
            if let Some(group_data) = config.profile_groups.get(&active_group) {
                let mut p_keys: Vec<String> = group_data.profiles.keys().cloned().collect();
                p_keys.sort();

                let mut edit_profile = SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());

                let scroll_h = 160.0;
                egui::ScrollArea::vertical().id_salt("profile_under_group_scroll").max_height(scroll_h).show(ui, |ui| {
                    for p_name in p_keys {
                        let is_active = p_name == config.active_app_profile;
                        let is_editing = p_name == edit_profile;

                        ui.horizontal(|ui| {
                            let label = if is_active {
                                RichText::new(format!("★ {}", p_name)).strong().color(theme::accent_color(ui.ctx()))
                            } else {
                                RichText::new(&p_name).color(theme::secondary_text(ui.ctx()))
                            };

                            if ui.selectable_label(is_editing, label).clicked() {
                                edit_profile = p_name.clone();
                                SELECTED_EDIT_PROFILE.with(|p| *p.borrow_mut() = p_name.clone());
                                if let Some(prof) = group_data.profiles.get(&p_name) {
                                    APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = prof.apps.first().cloned().unwrap_or_default());
                                }
                                BINDINGS_ERROR.with(|e| *e.borrow_mut() = None);
                            }

                            if is_active {
                                ui.label(RichText::new("(Active)").size(10.0).color(theme::muted_text(ui.ctx())));
                            } else {
                                if ui.button("Activate").clicked() {
                                    engine.select_profile(&p_name);
                                    config.active_app_profile = p_name.clone();
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }
                });

                // Add Profile form
                ui.add_space(10.0);
                ui.label("Add New Profile to Group:");
                ui.horizontal(|ui| {
                    NEW_PROFILE_NAME.with(|name_cell| {
                        let mut name_ref = name_cell.borrow_mut();
                        ui.text_edit_singleline(&mut *name_ref);
                        if ui.button("Add Profile").clicked() {
                            let clean = name_ref.trim();
                            if !clean.is_empty() {
                                engine.add_profile(clean);
                                name_ref.clear();
                            }
                        }
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Edit profile bindings
                let sel_profile = SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());
                if !sel_profile.is_empty() {
                    ui.label(RichText::new(format!("Managing Profile: {}", sel_profile)).strong().size(11.0));
                    ui.add_space(4.0);

                    if sel_profile == "global" {
                        ui.add_enabled(false, egui::Button::new("Delete Profile"));
                        ui.label(RichText::new("The 'global' profile cannot be deleted.").size(9.5).color(theme::muted_text(ui.ctx())));
                    } else {
                        if ui.button(RichText::new("Delete Profile").color(theme::COLOR_DOT_RED)).clicked() {
                            engine.delete_profile(&sel_profile);
                            SELECTED_EDIT_PROFILE.with(|p| p.borrow_mut().clear());
                        }
                    }

                    ui.add_space(10.0);
                    ui.label(RichText::new("Target Application Executable:").strong());
                    ui.label(RichText::new("Enter the single process name of the application (e.g. brave, spotify)")
                        .size(9.5)
                        .color(theme::muted_text(ui.ctx())));

                    ui.horizontal(|ui| {
                        // Text input for executable name
                        APP_BINDINGS_BUFFER.with(|buff_cell| {
                            let mut buff_ref = buff_cell.borrow_mut();
                            ui.text_edit_singleline(&mut *buff_ref);
                        });

                        // Dropdown picker from detected apps
                        let scanned_opt = crate::views::customization::popups::add_app_modal::SCANNED_APPS.with(|apps| apps.borrow().clone());
                        if let Some(scanned_apps) = scanned_opt {
                            let mut selected_app_exec = String::new();
                            let combo = egui::ComboBox::from_id_salt("scanned_apps_combo_picker")
                                .selected_text("Quick Select Detected App...");
                            let combo_res = combo.show_ui(ui, |ui| {
                                let mut clicked_val = None;
                                for app in &scanned_apps {
                                    // Check if already assigned
                                    let mut assigned_profile = None;
                                    for (pname, pdata) in &group_data.profiles {
                                        if pdata.apps.contains(&app.exec) {
                                            assigned_profile = Some(pname.clone());
                                            break;
                                        }
                                    }

                                    let label_text = if let Some(p) = assigned_profile {
                                        format!("{} (Mapped to {})", app.name, p)
                                    } else {
                                        app.name.clone()
                                    };

                                    if ui.selectable_value(&mut selected_app_exec, app.exec.clone(), label_text).clicked() {
                                        clicked_val = Some(app.exec.clone());
                                    }
                                }
                                clicked_val
                            });

                            if let Some(Some(val)) = combo_res.inner {
                                APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = val);
                            }
                        }
                    });

                    ui.add_space(4.0);

                    // Show validation error if any
                    let err_opt = BINDINGS_ERROR.with(|e| e.borrow().clone());
                    if let Some(err) = err_opt {
                        ui.label(RichText::new(&err).color(theme::COLOR_DOT_RED).size(10.0));
                        ui.add_space(4.0);
                    }

                    if ui.button("Save App Trigger").clicked() {
                        let raw_input = APP_BINDINGS_BUFFER.with(|b| b.borrow().clone());
                        let clean_exe = raw_input.trim().to_string();

                        let mut validation_ok = true;
                        let mut error_msg = None;

                        if clean_exe.contains(',') || clean_exe.contains(' ') {
                            validation_ok = false;
                            error_msg = Some("Please specify exactly one executable name (no commas or spaces).".to_string());
                        } else if !clean_exe.is_empty() {
                            // Check for duplicates
                            for (pname, pdata) in &group_data.profiles {
                                if pname != &sel_profile {
                                    if pdata.apps.contains(&clean_exe) {
                                        validation_ok = false;
                                        error_msg = Some(format!("Executable '{}' is already assigned to profile '{}'.", clean_exe, pname));
                                        break;
                                    }
                                }
                            }
                        }

                        if validation_ok {
                            BINDINGS_ERROR.with(|e| *e.borrow_mut() = None);
                            engine.update_app_bindings(&sel_profile, &clean_exe);
                        } else {
                            BINDINGS_ERROR.with(|e| *e.borrow_mut() = error_msg);
                        }
                    }
                }
            } else {
                ui.label("Active profile group not found.");
            }
        });
    });
}
