use super::MouserApp;
use crate::theme;
use crate::views::ActiveView;
use eframe::egui;

impl eframe::App for MouserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keep in sync with the engine's config state periodically
        self.reload_config();

        // ── Startup centering / update check ────────────────────────────────
        if !self.window_initialized {
            if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                theme::center_window(ctx, monitor);
                self.window_initialized = true;

                if self.config.settings.install_updates {
                    self.updater.check_for_updates(ctx.clone(), true);
                }
            }
        }

        // ── Close → hide to tray ────────────────────────────────────────────
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // ── Receive decoded images from background thread ───────────────────
        // Once received, store the ColorImage so the next texture request can
        // upload it to the GPU. Drop the channel afterwards to free memory.
        if let Some(img_rx) = &self.img_rx {
            if let Ok((main, cust)) = img_rx.try_recv() {
                self.preloaded_mouse_image = Some(main);
                self.preloaded_customization_mouse_image = Some(cust);
                self.img_rx = None; // channel is no longer needed
                ctx.request_repaint(); // ensure textures are uploaded next frame
            }
        }

        // ── Receive hardware state updates from background thread ───────────
        while let Ok(update) = self.rx.try_recv() {
            self.unifying_receiver_connected = Some(update.unifying_receiver_connected);
            self.bolt_receiver_connected = Some(update.bolt_receiver_connected);
            self.bluetooth_available = update.bluetooth_available;
            self.paired_devices = update.paired_devices;
            self.battery_pct = update.battery_pct;
            self.battery_status = update.battery_status;
            self.has_active_hidpp_battery = Some(update.has_active_hidpp_battery);
        }

        // Evaluate and lock/freeze connection type on device connect
        let is_device_connected = if !self.paired_devices.is_empty() {
            self.paired_devices.iter().any(|(_, _, is_conn)| *is_conn)
        } else {
            self.has_active_hidpp_battery.unwrap_or(false)
        };

        if is_device_connected {
            if self.current_connection_type.is_none() {
                let conn = if self.paired_devices.iter().any(|(_, _, is_conn)| *is_conn) {
                    "bluetooth"
                } else if self.bolt_receiver_connected.unwrap_or(false) {
                    "bolt"
                } else if self.unifying_receiver_connected.unwrap_or(false) {
                    "unifying"
                } else {
                    "bluetooth"
                };
                self.current_connection_type = Some(conn.to_string());
            }
        } else {
            self.current_connection_type = None;
        }

        // Update system tray icon based on connection type transition
        let active_conn = "mouse";

        if active_conn != self.current_tray_icon_type {
            if let Some(ref tray) = self.tray_icon {
                let new_icon = crate::theme::create_mouse_tray_icon();
                tray.set_icon(new_icon);
            }
            self.current_tray_icon_type = active_conn.to_string();
        }

        // ── Theme ────────────────────────────────────────────────────────────
        let is_light = match self.config.settings.appearance_mode.as_str() {
            "light" => true,
            "dark" => false,
            _ => ctx.system_theme() == Some(egui::Theme::Light),
        };

        let mut visual = if is_light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };

        let bg = theme::app_bg(ctx);
        visual.panel_fill = bg;
        visual.window_fill = bg;
        visual.widgets.noninteractive.bg_fill = bg;

        if ctx.style().visuals != visual {
            ctx.set_visuals(visual);
        }

        // ── Receive Actions Ring open/close signals from background thread ──
        while let Ok(sig) = self.actions_ring_rx.try_recv() {
            if sig.open {
                // Make the window a fullscreen, borderless, transparent overlay so
                // the ring can centre on the real OS cursor and catch clicks
                // anywhere on screen. Restored when the ring closes.
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
                let center = ctx
                    .input(|i| i.pointer.hover_pos())
                    .unwrap_or_else(|| ctx.screen_rect().center());
                self.actions_ring = Some(crate::views::actions_ring::RingState::from_payload(
                    &sig.layout_json,
                    &sig.folders_json,
                    center,
                ));
            } else {
                self.close_actions_ring(ctx);
            }
        }

        // ── Render ───────────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(bg))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    if self.active_view != ActiveView::Customization {
                        crate::views::top_bar::show(ui, ctx, &mut self.active_view, &self.config.settings.language);
                        ui.add_space(20.0);
                    }
                    match self.active_view {
                        ActiveView::EmptyState => {
                            let mut display_devices = (*self.paired_devices).clone();
                            if display_devices.is_empty() && self.has_active_hidpp_battery.unwrap_or(false) {
                                display_devices.push(("00:00:00:00:00:00".to_string(), "MX Master 3".to_string(), true));
                            }

                            if !display_devices.is_empty() {
                                let height = ui.available_height();
                                // Spacing at the top to vertically center the row
                                let content_height = 360.0;
                                    let top_padding = ((height - content_height) / 2.0 - 20.0).max(0.0);
                                    ui.add_space(top_padding);

                                    egui::ScrollArea::horizontal()
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                        ui.horizontal_top(|ui| {
                                        let gap = 40.0;
                                        let mut total_width = 0.0;
                                        for (_, name, _) in &display_devices {
                                            let layout_key = crate::app::get_layout_key_from_name(name);
                                            let is_kbd = layout_key.contains("keys") || layout_key.contains("mechanical");
                                            let card_w = if is_kbd { 560.0 } else { 220.0 };
                                            total_width += card_w;
                                        }
                                        total_width += gap * (display_devices.len() - 1) as f32;

                                        let viewport_w = ctx.available_rect().width();
                                        let start_space = ((viewport_w - total_width) / 2.0).max(0.0);
                                        ui.add_space(start_space);

                                        let mut action_to_take = None;
                                        let mut textures_loading = false;

                                        for (mac, name, is_connected) in &display_devices {
                                            let layout_key = crate::app::get_layout_key_from_name(name);
                                            if let Some(device_tex) = self.get_or_load_device_texture(ctx, &layout_key) {
                                                let battery_pct = if *is_connected {
                                                    self.battery_pct.clone()
                                                } else {
                                                    "0".to_string()
                                                };

                                                let conn_type = if *is_connected {
                                                    let is_bt = self.paired_devices.iter()
                                                        .any(|(_, n, conn)| n == name && *conn);
                                                    if is_bt {
                                                        "bluetooth"
                                                    } else if self.bolt_receiver_connected.unwrap_or(false) {
                                                        "bolt"
                                                    } else if self.unifying_receiver_connected.unwrap_or(false) {
                                                        "unifying"
                                                    } else {
                                                        "bluetooth"
                                                    }
                                                } else {
                                                    "bluetooth"
                                                };

                                                let action = crate::views::empty_state::show_known_device(
                                                    ui,
                                                    name,
                                                    *is_connected,
                                                    &device_tex,
                                                    conn_type,
                                                    &battery_pct,
                                                    &self.config.settings.language,
                                                );

                                                if action != crate::views::empty_state::DeviceCardAction::None {
                                                    action_to_take = Some((mac.clone(), action));
                                                }
                                            } else {
                                                textures_loading = true;
                                            }
                                            ui.add_space(gap);
                                        }

                                        if textures_loading {
                                            ctx.request_repaint();
                                        }

                                        if let Some((mac, action)) = action_to_take {
                                            match action {
                                                crate::views::empty_state::DeviceCardAction::Unpair => {
                                                    self.engine.unpair_device(&mac);
                                                     let mut devices = (*self.paired_devices).clone();
                                                     devices.retain(|(m, _, _)| m != &mac);
                                                     self.paired_devices = std::sync::Arc::new(devices);
                                                }
                                                crate::views::empty_state::DeviceCardAction::Customize => {
                                                    if let Some((_, name, _)) = display_devices.iter().find(|(m, _, _)| m == &mac) {
                                                        self.customizing_device_name = Some(name.clone());
                                                    }
                                                    self.customization_tab = crate::views::customization::SidebarTab::Buttons;
                                                    self.active_view = ActiveView::Customization;
                                                }
                                                _ => {}
                                            }
                                        }
                                    });
                                });
                            } else {
                                crate::views::empty_state::show(ui, &self.config.settings.language);
                            }
                        }
                        ActiveView::SelectConnectionType => {
                            let unifying = self.unifying_receiver_connected.unwrap_or(false);
                            let bolt = self.bolt_receiver_connected.unwrap_or(false);
                            crate::views::select_connection::show(
                                ui,
                                &mut self.active_view,
                                unifying,
                                bolt,
                                &self.config.settings.language,
                            );
                        }
                        ActiveView::Settings => {
                            crate::views::settings::show(
                                ui,
                                ctx,
                                &mut self.config,
                                &self.engine,
                                &self.updater,
                            );
                        }
                        ActiveView::Customization => {
                            let customizing_device_name = self.customizing_device_name.clone().unwrap_or_else(|| "MX Master 3".to_string());
                            let layout_key = crate::app::get_layout_key_from_name(&customizing_device_name);
                            let is_keyboard = layout_key.contains("keys") || layout_key.contains("mechanical");

                            let device_tex_opt = if is_keyboard {
                                self.get_or_load_device_texture(ctx, &layout_key)
                            } else {
                                self.get_or_load_customization_mouse_texture(ctx)
                                    .or_else(|| self.get_or_load_device_texture(ctx, &layout_key))
                            };

                            if let Some(device_tex) = device_tex_opt {
                                let is_customizing_connected = if self.paired_devices.is_empty() && customizing_device_name == "MX Master 3" {
                                    self.has_active_hidpp_battery.unwrap_or(false)
                                } else {
                                    self.paired_devices.iter()
                                        .find(|(_, name, _)| name == &customizing_device_name)
                                        .map(|(_, _, conn)| *conn)
                                        .unwrap_or(false)
                                };

                                let conn_type = if is_customizing_connected {
                                    let is_bt = self.paired_devices.iter()
                                        .any(|(_, name, conn)| name == &customizing_device_name && *conn);
                                    if is_bt {
                                        "bluetooth"
                                    } else if self.bolt_receiver_connected.unwrap_or(false) {
                                        "bolt"
                                    } else if self.unifying_receiver_connected.unwrap_or(false) {
                                        "unifying"
                                    } else {
                                        "bluetooth"
                                    }
                                } else {
                                    "bluetooth"
                                };

                                let battery_pct = if is_customizing_connected {
                                    self.battery_pct.clone()
                                } else {
                                    "0".to_string()
                                };

                                crate::views::customization::show(
                                    ui,
                                    ctx,
                                    &self.engine,
                                    &mut self.config,
                                    &device_tex,
                                    &mut self.active_view,
                                    &mut self.customizing_button,
                                    &mut self.customization_tab,
                                    conn_type,
                                    &battery_pct,
                                    &self.battery_status,
                                    is_customizing_connected,
                                    &customizing_device_name,
                                );
                            } else {
                                ctx.request_repaint();
                            }
                        }
                    }
                });
            });

        self.draw_toast(ctx);

        // ── Actions Ring overlay (drawn last so it sits above everything) ──
        if let Some(ring) = &mut self.actions_ring {
            match crate::views::actions_ring::show_ring(ctx, ring) {
                Some(action_id) => {
                    self.engine.execute_action(&action_id);
                    self.close_actions_ring(ctx); // engine also broadcasts close
                }
                None => {
                    self.close_actions_ring(ctx);
                }
            }
        }
    }
}
