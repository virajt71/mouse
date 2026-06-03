pub mod theme;
pub mod top_bar;
pub mod empty_state;
pub mod select_connection;
pub mod settings;
pub mod mouse_ui;
pub mod translation;

use mouser_engine::updater::Updater;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    EmptyState,
    SelectConnectionType,
    Settings,
    Customization,
}

pub struct MouserApp {
    tray_icon: Option<tray_icon::TrayIcon>,
    current_tray_icon_type: String,
    active_view: ActiveView,
    unifying_receiver_connected: Option<bool>,
    bolt_receiver_connected: Option<bool>,
    paired_devices: Vec<(String, String, bool)>, // (mac, name, is_connected)
    mouse_texture: Option<egui::TextureHandle>,
    customization_mouse_texture: Option<egui::TextureHandle>,
    bluetooth_available: bool,
    window_initialized: bool,
    pub config: mouser_engine::config::Config,
    pub engine: mouser_engine::Engine,
    pub updater: Updater,
    pub customizing_button: Option<self::mouse_ui::CustomizingButton>,
    pub customization_tab: self::mouse_ui::SidebarTab,
    last_config_generation: u64,

    // Hardware polling channel
    rx: std::sync::mpsc::Receiver<mouser_engine::worker::DeviceStateUpdate>,
    tx: std::sync::mpsc::Sender<mouser_engine::worker::BackgroundTxCmd>,
    battery_pct: String,
    has_active_hidpp_battery: Option<bool>,

    // Background image decode channel — None once both images have been received
    img_rx: Option<std::sync::mpsc::Receiver<(egui::ColorImage, egui::ColorImage)>>,
    // CPU-side pixel buffers; Some until the first GPU upload, then None to free memory
    preloaded_mouse_image: Option<egui::ColorImage>,
    preloaded_customization_mouse_image: Option<egui::ColorImage>,
}

impl MouserApp {
    pub fn new(ctx: egui::Context, tray_icon: Option<tray_icon::TrayIcon>, engine: mouser_engine::Engine) -> Self {
        let cached = mouser_engine::cache::load_device_cache();
        let config = engine.get_config();
        let config_gen = engine.config_generation();
        let updater = Updater::new();

        // Decode both PNG files in a background thread so the main thread is
        // never blocked. include_bytes! embeds the data at compile-time, so no
        // disk I/O occurs — only CPU work for PNG decompression.
        let (img_tx, img_rx) = std::sync::mpsc::channel::<(egui::ColorImage, egui::ColorImage)>();
        std::thread::spawn(move || {
            let decode = |bytes: &[u8]| -> Option<egui::ColorImage> {
                let img = image::load_from_memory(bytes).ok()?;
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                Some(egui::ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    &rgba.into_raw(),
                ))
            };

            let main_bytes =
                include_bytes!("../../assets/images/logitech-mice/mx_master_3/mouse.png");
            let cust_bytes =
                include_bytes!("../../assets/images/logitech-mice/mx_master_3/mx_master.png");

            if let (Some(main), Some(cust)) = (decode(main_bytes), decode(cust_bytes)) {
                let _ = img_tx.send((main, cust));
            }
        });

        let (tx, rx) = mouser_engine::worker::spawn_background_worker(ctx);

        Self {
            tray_icon,
            current_tray_icon_type: "mouse".to_string(),
            active_view: ActiveView::EmptyState,
            unifying_receiver_connected: None,
            bolt_receiver_connected: None,
            paired_devices: cached,
            mouse_texture: None,
            customization_mouse_texture: None,
            bluetooth_available: true,
            window_initialized: false,
            config,
            engine,
            updater,
            customizing_button: None,
            customization_tab: self::mouse_ui::SidebarTab::Buttons,
            last_config_generation: config_gen,
            rx,
            tx,
            battery_pct: "0".to_string(),
            has_active_hidpp_battery: None,
            img_rx: Some(img_rx),
            preloaded_mouse_image: None,
            preloaded_customization_mouse_image: None,
        }
    }

    /// Upload the pre-decoded main mouse image to the GPU on first call; returns
    /// None if the background thread hasn't finished decoding yet.
    fn get_or_load_mouse_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.mouse_texture.is_none() {
            if let Some(image) = self.preloaded_mouse_image.take() {
                self.mouse_texture = Some(ctx.load_texture(
                    "mx_master_mouse_main",
                    image,
                    Default::default(),
                ));
            }
        }
        self.mouse_texture.clone()
    }

    /// Upload the pre-decoded customization mouse image to the GPU on first call;
    /// returns None if the background thread hasn't finished decoding yet.
    fn get_or_load_customization_mouse_texture(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<egui::TextureHandle> {
        if self.customization_mouse_texture.is_none() {
            if let Some(image) = self.preloaded_customization_mouse_image.take() {
                self.customization_mouse_texture = Some(ctx.load_texture(
                    "mx_master_mouse_customization",
                    image,
                    Default::default(),
                ));
            }
        }
        self.customization_mouse_texture.clone()
    }

    pub fn reload_config(&mut self) {
        let current_gen = self.engine.config_generation();
        if current_gen != self.last_config_generation {
            self.config = self.engine.get_config();
            self.last_config_generation = current_gen;
        }
    }
}

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
        }

        // Update system tray icon based on connection type transition
        let active_conn = {
            let any_bt_connected = self.paired_devices.iter().any(|(_, _, is_conn)| *is_conn);
            if any_bt_connected {
                "bluetooth"
            } else if self.has_active_hidpp_battery.unwrap_or(false) {
                if self.bolt_receiver_connected.unwrap_or(false) {
                    "mouse"
                } else if self.unifying_receiver_connected.unwrap_or(false) {
                    "mouse"
                } else {
                    "bluetooth"
                }
            } else {
                "mouse"
            }
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
                        top_bar::show(ui, ctx, &mut self.active_view, &self.config.settings.language);
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

                                    match empty_state::show_known_device(
                                        ui,
                                        &name,
                                        is_connected,
                                        &mouse_tex,
                                        "bluetooth",
                                        &battery_pct,
                                        &self.config.settings.language,
                                    ) {
                                        empty_state::DeviceCardAction::Unpair => {
                                            let _ = self.tx.send(
                                                mouser_engine::worker::BackgroundTxCmd::Unpair(
                                                    mac.clone(),
                                                ),
                                            );
                                            self.paired_devices
                                                .retain(|(m, _, _)| m != &mac);
                                        }
                                        empty_state::DeviceCardAction::Customize => {
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
                                    let conn_type =
                                        if self.bolt_receiver_connected.unwrap_or(false) {
                                            "bolt"
                                        } else if self
                                            .unifying_receiver_connected
                                            .unwrap_or(false)
                                        {
                                            "unifying"
                                        } else {
                                            "bluetooth"
                                        };

                                    if empty_state::show_known_device(
                                        ui,
                                        "MX Master 3",
                                        true,
                                        &mouse_tex,
                                        conn_type,
                                        &self.battery_pct,
                                        &self.config.settings.language,
                                    ) == empty_state::DeviceCardAction::Customize
                                    {
                                        self.active_view = ActiveView::Customization;
                                    }
                                } else {
                                    ctx.request_repaint();
                                }
                            } else {
                                empty_state::show(ui, &self.config.settings.language);
                            }
                        }
                        ActiveView::SelectConnectionType => {
                            let unifying = self.unifying_receiver_connected.unwrap_or(false);
                            let bolt = self.bolt_receiver_connected.unwrap_or(false);
                            select_connection::show(
                                ui,
                                &mut self.active_view,
                                unifying,
                                bolt,
                                &self.config.settings.language,
                            );
                        }
                        ActiveView::Settings => {
                            self::settings::show(
                                ui,
                                ctx,
                                &mut self.config,
                                &self.engine,
                                &self.updater,
                            );
                        }
                        ActiveView::Customization => {
                            // If mouse is disconnected, redirect back to EmptyState
                            let is_mouse_connected = if !self.paired_devices.is_empty() {
                                self.paired_devices[0].2
                            } else {
                                self.has_active_hidpp_battery.unwrap_or(false)
                            };

                            if !is_mouse_connected {
                                self.active_view = ActiveView::EmptyState;
                                ctx.request_repaint();
                            } else if let Some(mouse_tex) =
                                self.get_or_load_customization_mouse_texture(ctx)
                            {
                                let conn_type =
                                    if self.bolt_receiver_connected.unwrap_or(false) {
                                        "bolt"
                                    } else if self
                                        .unifying_receiver_connected
                                        .unwrap_or(false)
                                    {
                                        "unifying"
                                    } else {
                                        "bluetooth"
                                    };

                                self::mouse_ui::show(
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
                                );
                            } else {
                                // Texture not ready yet — go back and retry next frame
                                self.active_view = ActiveView::EmptyState;
                                ctx.request_repaint();
                            }
                        }
                    }
                });
            });
    }
}
