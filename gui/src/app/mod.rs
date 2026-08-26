use crate::updater::Updater;
use crate::views::ActiveView;
use eframe::egui;

pub mod texture;
pub mod toast;
pub mod update;

#[derive(Clone)]
pub struct MouserTray {
    #[cfg(target_os = "linux")]
    pub(crate) sender: gtk::glib::Sender<tray_icon::Icon>,
    #[cfg(not(target_os = "linux"))]
    pub(crate) icon: std::sync::Arc<tray_icon::TrayIcon>,
}

impl MouserTray {
    #[cfg(target_os = "linux")]
    pub fn new(sender: gtk::glib::Sender<tray_icon::Icon>) -> Self {
        Self { sender }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new(icon: tray_icon::TrayIcon) -> Self {
        Self {
            icon: std::sync::Arc::new(icon),
        }
    }

    pub fn set_icon(&self, icon: tray_icon::Icon) {
        #[cfg(target_os = "linux")]
        {
            let _ = self.sender.send(icon);
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = self.icon.set_icon(Some(icon));
        }
    }
}

pub struct MouserApp {
    pub(crate) tray_icon: Option<MouserTray>,
    pub(crate) current_tray_icon_type: String,
    pub(crate) active_view: ActiveView,
    pub(crate) unifying_receiver_connected: Option<bool>,
    pub(crate) bolt_receiver_connected: Option<bool>,
    pub(crate) paired_devices: std::sync::Arc<Vec<(String, String, bool)>>, // (mac, name, is_connected)
    pub(crate) mouse_texture: Option<egui::TextureHandle>,
    pub(crate) customization_mouse_texture: Option<egui::TextureHandle>,
    pub(crate) device_textures: std::collections::HashMap<String, egui::TextureHandle>,
    pub(crate) bluetooth_available: bool,
    pub(crate) window_initialized: bool,
    pub config: mouser_engine::config::Config,
    /// gRPC client — the GUI's sole connection to the daemon.
    pub engine: mouser_engine::client::EngineClient,
    pub updater: Updater,
    pub customizing_button: Option<crate::views::customization::mappings::CustomizingButton>,
    pub customization_tab: crate::views::customization::SidebarTab,
    pub customizing_device_name: Option<String>,
    pub(crate) last_config_generation: u64,
    pub(crate) config_changed_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,

    // Config stream channel
    pub(crate) config_rx: std::sync::mpsc::Receiver<mouser_engine::config::Config>,
    // Hardware polling channel — filled by the gRPC WatchDeviceState stream
    pub(crate) rx: std::sync::mpsc::Receiver<mouser_engine::worker::DeviceStateUpdate>,
    pub(crate) battery_pct: String,
    pub(crate) battery_status: String,
    pub(crate) has_active_hidpp_battery: Option<bool>,

    // Background image decode channel — None once both images have been received
    pub(crate) img_rx: Option<std::sync::mpsc::Receiver<(egui::ColorImage, egui::ColorImage)>>,
    // CPU-side pixel buffers; Some until the first GPU upload, then None to free memory
    pub(crate) preloaded_mouse_image: Option<egui::ColorImage>,
    pub(crate) preloaded_customization_mouse_image: Option<egui::ColorImage>,
    pub(crate) current_connection_type: Option<String>,
    pub(crate) toast_message: Option<String>,
    pub(crate) toast_shown_at: Option<std::time::Instant>,
}

impl MouserApp {
    pub fn new(
        ctx: egui::Context,
        tray_icon: Option<MouserTray>,
        engine: mouser_engine::client::EngineClient,
    ) -> Self {
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
                include_bytes!("../../../assets/images/logitech-mice/mx_master_3/mouse.png");
            let cust_bytes =
                include_bytes!("../../../assets/images/logitech-mice/mx_master_3/mx_master.png");

            if let (Some(main), Some(cust)) = (decode(main_bytes), decode(cust_bytes)) {
                let _ = img_tx.send((main, cust));
            }
        });

        // Subscribe to config changes via the gRPC WatchConfig stream.
        // Updates are pushed into config_changed_flag + config_rx + repaint trigger.
        let config_changed_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let (config_tx, config_rx) = std::sync::mpsc::channel::<mouser_engine::config::Config>();
        let flag_clone = config_changed_flag.clone();
        let repaint_ctx_config = ctx.clone();
        std::sync::Arc::new(engine.clone()).subscribe_config(config_tx, move || {
            flag_clone.store(true, std::sync::atomic::Ordering::Relaxed);
            repaint_ctx_config.request_repaint();
        });

        // Subscribe to device-state updates via the gRPC WatchDeviceState stream.
        let (device_tx, rx) = std::sync::mpsc::channel::<mouser_engine::worker::DeviceStateUpdate>();
        let repaint_ctx_device = ctx.clone();
        std::sync::Arc::new(engine.clone()).subscribe_device_state(device_tx, move || {
            repaint_ctx_device.request_repaint();
        });

        Self {
            tray_icon,
            current_tray_icon_type: "mouse".to_string(),
            active_view: ActiveView::EmptyState,
            unifying_receiver_connected: None,
            bolt_receiver_connected: None,
            paired_devices: std::sync::Arc::new(cached),
            mouse_texture: None,
            customization_mouse_texture: None,
            device_textures: std::collections::HashMap::new(),
            bluetooth_available: true,
            window_initialized: false,
            config,
            engine,
            updater,
            customizing_button: None,
            customization_tab: crate::views::customization::SidebarTab::Buttons,
            customizing_device_name: None,
            last_config_generation: config_gen,
            config_changed_flag,
            config_rx,
            rx,
            battery_pct: "0".to_string(),
            battery_status: String::new(),
            has_active_hidpp_battery: None,
            img_rx: Some(img_rx),
            preloaded_mouse_image: None,
            preloaded_customization_mouse_image: None,
            current_connection_type: None,
            toast_message: None,
            toast_shown_at: None,
        }
    }

    pub fn reload_config(&mut self) {
        let mut got_update = false;
        while let Ok(fresh) = self.config_rx.try_recv() {
            self.config = fresh;
            got_update = true;
        }

        if got_update {
            self.last_config_generation = self.engine.config_generation();
            self.config_changed_flag.store(false, std::sync::atomic::Ordering::Relaxed);
        } else if self
            .config_changed_flag
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            if let Some(fresh) = self
                .engine
                .get_config_if_changed(self.last_config_generation)
            {
                self.config = fresh;
                self.last_config_generation = self.engine.config_generation();
            }
        }
    }
}

pub fn get_layout_key_from_name(name: &str) -> String {
    mouser_engine::hidpp::device::get_layout_key_from_name(name)
}
