use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::Engine;
use mouser_engine::config::{Config, FlowPeer};
use crate::theme;

pub fn show_flow_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
) {
    let mut settings_dirty = false;

    ui.horizontal(|ui| {
        ui.add_space(20.0);
        ui.vertical(|ui| {
            ui.add_space(20.0);
            
            // Header
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    RichText::new("LOGITECH FLOW")
                        .color(Color32::WHITE)
                        .size(18.0)
                        .strong(),
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut enabled = config.settings.flow_enabled;
                    // Custom toggle row for premium feel
                    let response = ui.allocate_response(vec2(40.0, 20.0), egui::Sense::click());
                    if response.clicked() {
                        enabled = !enabled;
                        config.settings.flow_enabled = enabled;
                        settings_dirty = true;
                    }
                    let t = ui.ctx().animate_bool(response.id.with("flow_enable_toggle"), enabled);
                    let color = theme::lerp_color(Color32::from_gray(50), theme::accent_color(ui.ctx()), t);
                    ui.painter().rect_filled(response.rect, 10.0, color);
                    let knob_x = response.rect.min.x + 10.0 + t * (response.rect.width() - 20.0);
                    ui.painter().circle_filled(pos2(knob_x, response.rect.center().y), 8.0, Color32::WHITE);
                });
            });

            ui.add_space(8.0);
            ui.add(egui::Label::new(
                RichText::new("Seamlessly control multiple computers, copy and paste files, and sync clipboards over your local network.")
                    .color(theme::secondary_text(ui.ctx()))
                    .size(12.0),
            ));
            
            ui.add_space(20.0);

            // Two-column layout: Left is layout grid, Right is Settings / Peers
            ui.columns(2, |columns| {
                // Column 0: Visual Screen Grid Layout Configurator
                let ui_grid = &mut columns[0];
                ui_grid.vertical(|ui| {
                    ui.label(
                        RichText::new("MONITOR ARRANGEMENT")
                            .color(theme::primary_text(ui.ctx()))
                            .size(11.0)
                            .strong(),
                    );
                    ui.add_space(10.0);

                    // Draw a 3x3 layout grid representing monitors
                    // Center is local computer, surrounding are peers
                    let grid_center = ui.cursor().left_top() + vec2(150.0, 100.0);
                    let box_w = 90.0;
                    let box_h = 60.0;

                    let mut clicked_slot = None;

                    let mut draw_screen = |ui: &mut egui::Ui, offset_x: i32, offset_y: i32, label: &str, is_local: bool| {
                        let center_pos = grid_center + vec2(offset_x as f32 * (box_w + 10.0), offset_y as f32 * (box_h + 10.0));
                        let r = Rect::from_center_size(center_pos, vec2(box_w, box_h));
                        let res = ui.allocate_rect(r, egui::Sense::click());

                        let bg = if is_local {
                            theme::accent_dim_color(ui.ctx())
                        } else if res.hovered() {
                            theme::hover_color(ui.ctx())
                        } else {
                            theme::surface_color(ui.ctx())
                        };

                        let border = if is_local {
                            theme::accent_color(ui.ctx())
                        } else if !label.is_empty() {
                            theme::accent_color(ui.ctx())
                        } else {
                            theme::border_color(ui.ctx())
                        };

                        ui.painter().rect_filled(r, 4.0, bg);
                        ui.painter().rect_stroke(r, 4.0, Stroke::new(1.2, border));
                        theme::draw_tech_corners(ui.painter(), r, border, 5.0);

                        let display_label = if label.is_empty() {
                            if is_local { config.settings.flow_local_name.clone() } else { "[Empty]".to_string() }
                        } else {
                            label.to_string()
                        };

                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            &display_label,
                            egui::FontId::proportional(10.0),
                            theme::primary_text(ui.ctx()),
                        );

                        if !is_local && res.clicked() {
                            clicked_slot = Some((offset_x, offset_y));
                        }
                    };

                    // Render 3x3 layout cross
                    // Local computer in center
                    draw_screen(ui, 0, 0, "", true);

                    // Check which peers are in which positions
                    let mut left_peer = "".to_string();
                    let mut right_peer = "".to_string();
                    let mut top_peer = "".to_string();
                    let mut bottom_peer = "".to_string();

                    for peer in &config.settings.flow_peers {
                        if peer.layout_x == -1 && peer.layout_y == 0 { left_peer = peer.name.clone(); }
                        else if peer.layout_x == 1 && peer.layout_y == 0 { right_peer = peer.name.clone(); }
                        else if peer.layout_x == 0 && peer.layout_y == -1 { top_peer = peer.name.clone(); }
                        else if peer.layout_x == 0 && peer.layout_y == 1 { bottom_peer = peer.name.clone(); }
                    }

                    draw_screen(ui, -1, 0, &left_peer, false);
                    draw_screen(ui, 1, 0, &right_peer, false);
                    draw_screen(ui, 0, -1, &top_peer, false);
                    draw_screen(ui, 0, 1, &bottom_peer, false);

                    if let Some((offset_x, offset_y)) = clicked_slot {
                        let mut available = vec!["".to_string()];
                        for peer in &config.settings.flow_peers {
                            if peer.paired {
                                available.push(peer.name.clone());
                            }
                        }
                        
                        let current = config.settings.flow_peers.iter()
                            .find(|p| p.layout_x == offset_x && p.layout_y == offset_y)
                            .map(|p| p.name.clone())
                            .unwrap_or_default();
                        
                        let current_idx = available.iter().position(|x| x == &current).unwrap_or(0);
                        let next_idx = (current_idx + 1) % available.len();
                        let new_peer_name = &available[next_idx];
                        
                        // 1. Clear whatever was previously at this position
                        for p in &mut config.settings.flow_peers {
                            if p.layout_x == offset_x && p.layout_y == offset_y {
                                p.layout_x = 99;
                                p.layout_y = 99;
                            }
                        }
                        
                        // 2. Assign the chosen peer to this position
                        if !new_peer_name.is_empty() {
                            for p in &mut config.settings.flow_peers {
                                if p.name == *new_peer_name {
                                    p.layout_x = offset_x;
                                    p.layout_y = offset_y;
                                }
                            }
                        }
                        settings_dirty = true;
                    }

                    ui.add_space(200.0); // reserve space for grid height
                });

                // Column 1: Config Parameters & Peer pairing listing
                let ui_params = &mut columns[1];
                ui_params.vertical(|ui| {
                    ui.label(
                        RichText::new("FLOW SETTINGS")
                            .color(theme::primary_text(ui.ctx()))
                            .size(11.0)
                            .strong(),
                    );
                    ui.add_space(10.0);

                    // Local Name Input
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("This Computer:").color(theme::secondary_text(ui.ctx())).size(11.0));
                        let mut local_name = config.settings.flow_local_name.clone();
                        if ui.text_edit_singleline(&mut local_name).changed() {
                            config.settings.flow_local_name = local_name;
                            settings_dirty = true;
                        }
                    });
                    ui.add_space(8.0);

                    // Mouse Mode Dropdown/Combo
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Switching Method:").color(theme::secondary_text(ui.ctx())).size(11.0));
                        let current_mode = config.settings.flow_mouse_mode.clone();
                        egui::ComboBox::from_id_salt("mouse_redirection_mode_combobox")
                            .selected_text(if current_mode == "hardware" { "Hardware (HID++ Channel Switch)" } else { "Software Redirection (Instant)" })
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut config.settings.flow_mouse_mode, "software".to_string(), "Software Redirection (Instant)").clicked() {
                                    settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_mouse_mode, "hardware".to_string(), "Hardware (HID++ Channel Switch)").clicked() {
                                    settings_dirty = true;
                                }
                            });
                    });
                    ui.add_space(8.0);

                    // Hold key Dropdown/Combo
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hold Key to Transition:").color(theme::secondary_text(ui.ctx())).size(11.0));
                        let hold_key = config.settings.flow_hold_key.clone();
                        egui::ComboBox::from_id_salt("hold_key_combobox")
                            .selected_text(hold_key.to_uppercase())
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "none".to_string(), "NONE").clicked() {
                                    settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "ctrl".to_string(), "CTRL").clicked() {
                                    settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "alt".to_string(), "ALT").clicked() {
                                    settings_dirty = true;
                                }
                                if ui.selectable_value(&mut config.settings.flow_hold_key, "shift".to_string(), "SHIFT").clicked() {
                                    settings_dirty = true;
                                }
                            });
                    });
                    ui.add_space(8.0);

                    // Keyboard Linking checkbox
                    ui.checkbox(&mut config.settings.flow_keyboard_linking, "Link keyboard input redirection");
                    ui.add_space(10.0);

                    // Resolution inputs for Wayland coordinate accumulation
                    ui.label(
                        RichText::new("Screen Resolution (required for Wayland)")
                            .color(theme::secondary_text(ui.ctx()))
                            .size(10.5),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        let mut w_str = config.settings.flow_screen_width.to_string();
                        if ui.add(egui::TextEdit::singleline(&mut w_str).desired_width(50.0)).changed() {
                            if let Ok(w) = w_str.parse::<i32>() {
                                config.settings.flow_screen_width = w;
                                settings_dirty = true;
                            }
                        }
                        ui.label("Height:");
                        let mut h_str = config.settings.flow_screen_height.to_string();
                        if ui.add(egui::TextEdit::singleline(&mut h_str).desired_width(50.0)).changed() {
                            if let Ok(h) = h_str.parse::<i32>() {
                                config.settings.flow_screen_height = h;
                                settings_dirty = true;
                            }
                        }
                    });

                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("DISCOVERED COMPUTERS")
                            .color(theme::primary_text(ui.ctx()))
                            .size(11.0)
                            .strong(),
                    );
                    ui.add_space(8.0);

                    // Display list of discovered peers from network
                    let discovered = mouser_engine::flow::network::DISCOVERED_PEERS.read().unwrap();
                    if discovered.is_empty() {
                        ui.label(RichText::new("No computers found on local subnet. Make sure they are running Mouser-RS and connected to the same network.").color(theme::muted_text(ui.ctx())).size(10.5));
                    } else {
                        for (name, (ip, _)) in discovered.iter() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{} ({})", name, ip)).color(theme::primary_text(ui.ctx())).size(11.0));
                                
                                // Check if this peer is already paired
                                let paired = config.settings.flow_peers.iter().any(|p| p.name == *name && p.paired);
                                if paired {
                                    ui.label(RichText::new("Paired").color(theme::accent_color(ui.ctx())).size(10.5));
                                } else {
                                    if ui.button("Pair").clicked() {
                                        // Simple pairing simulation: add to peers config as paired
                                        config.settings.flow_peers.push(FlowPeer {
                                            name: name.clone(),
                                            ip: ip.clone(),
                                            port: 50520,
                                            layout_x: 1, // Default to right
                                            layout_y: 0,
                                            paired: true,
                                            fingerprint: "".to_string(),
                                        });
                                        settings_dirty = true;
                                    }
                                }
                            });
                        }
                    }
                });
            });
        });
    });

    if settings_dirty {
        let _ = config.save();
        // Trigger Engine reload to apply new resolution or settings
        engine.increment_config_generation();
    }
}
