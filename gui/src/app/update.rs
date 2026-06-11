use super::MouserApp;
use eframe::egui;
use crate::theme;
use crate::views::ActiveView;

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
            self.has_active_hidpp_battery = Some(update.has_active_hidpp_battery);

            self.last_known_profile = update.active_profile.clone();
        }

        // Evaluate and lock/freeze connection type on device connect
        let is_device_connected = if !self.paired_devices.is_empty() {
            self.paired_devices[0].2
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
        let active_conn = if let Some(ref conn) = self.current_connection_type {
            if conn == "bluetooth" {
                "bluetooth"
            } else {
                "mouse"
            }
        } else {
            "mouse"
        };

        if active_conn != self.current_tray_icon_type {
            if let Some(ref tray) = self.tray_icon {
                let new_icon = match active_conn {
                    "bluetooth" => crate::theme::create_bluetooth_tray_icon(),
                    _ => crate::theme::create_mouse_tray_icon(),
                };
                let _ = tray.set_icon(Some(new_icon));
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
                            if !self.paired_devices.is_empty() {
                                let (mac, name, is_connected) = self.paired_devices[0].clone();

                                // Texture may not be ready yet — skip drawing the
                                // device card on the first frame(s) while decoding.
                                if let Some(mouse_tex) = self.get_or_load_mouse_texture(ctx) {
                                    let battery_pct = if is_connected {
                                        self.battery_pct.clone()
                                    } else {
                                        "0".to_string()
                                    };

                                    let conn_type = self.current_connection_type.as_deref().unwrap_or("bluetooth");

                                    match crate::views::empty_state::show_known_device(
                                        ui,
                                        &name,
                                        is_connected,
                                        &mouse_tex,
                                        conn_type,
                                        &battery_pct,
                                        &self.config.settings.language,
                                    ) {
                                        crate::views::empty_state::DeviceCardAction::Unpair => {
                                            let _ = self.tx.send(
                                                mouser_engine::worker::BackgroundTxCmd::Unpair(
                                                    mac.clone(),
                                                ),
                                            );
                                            self.paired_devices
                                                .retain(|(m, _, _)| m != &mac);
                                        }
                                        crate::views::empty_state::DeviceCardAction::Customize => {
                                            self.active_view = ActiveView::Customization;
                                        }
                                        _ => {}
                                    }
                                } else {
                                    // Images still loading — request repaint so we
                                    // retry on the next frame.
                                    ctx.request_repaint();
                                }
                            } else if self.has_active_hidpp_battery.unwrap_or(false) {
                                if let Some(mouse_tex) = self.get_or_load_mouse_texture(ctx) {
                                    let conn_type = self.current_connection_type.as_deref().unwrap_or("bluetooth");

                                    if crate::views::empty_state::show_known_device(
                                        ui,
                                        "MX Master 3",
                                        true,
                                        &mouse_tex,
                                        conn_type,
                                        &self.battery_pct,
                                        &self.config.settings.language,
                                    ) == crate::views::empty_state::DeviceCardAction::Customize
                                    {
                                        self.active_view = ActiveView::Customization;
                                    }
                                } else {
                                    ctx.request_repaint();
                                }
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
                            if let Some(mouse_tex) =
                                self.get_or_load_customization_mouse_texture(ctx)
                            {
                                let conn_type = self.current_connection_type.as_deref().unwrap_or("bluetooth");

                                crate::views::customization::show(
                                    ui,
                                    ctx,
                                    &self.engine,
                                    &mut self.config,
                                    &mouse_tex,
                                    &mut self.active_view,
                                    &mut self.customizing_button,
                                    &mut self.customization_tab,
                                    conn_type,
                                    &self.battery_pct,
                                    is_device_connected,
                                );
                            } else {
                                ctx.request_repaint();
                            }
                        }
                    }
                });
            });

        self.draw_toast(ctx);
    }
}
