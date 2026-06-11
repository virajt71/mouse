use crate::updater::Updater;
use crate::views::ActiveView;
use eframe::egui;

pub mod texture;
pub mod toast;
pub mod update;

pub struct MouserApp {
    pub(crate) tray_icon: Option<tray_icon::TrayIcon>,
    pub(crate) current_tray_icon_type: String,
    pub(crate) active_view: ActiveView,
    pub(crate) unifying_receiver_connected: Option<bool>,
    pub(crate) bolt_receiver_connected: Option<bool>,
    pub(crate) paired_devices: Vec<(String, String, bool)>, // (mac, name, is_connected)
    pub(crate) mouse_texture: Option<egui::TextureHandle>,
    pub(crate) customization_mouse_texture: Option<egui::TextureHandle>,
    pub(crate) device_textures: std::collections::HashMap<String, egui::TextureHandle>,
    pub(crate) bluetooth_available: bool,
    pub(crate) window_initialized: bool,
    pub config: mouser_engine::config::Config,
    pub engine: mouser_engine::Engine,
    pub updater: Updater,
    pub customizing_button: Option<crate::views::customization::mappings::CustomizingButton>,
    pub customization_tab: crate::views::customization::SidebarTab,
    pub customizing_device_name: Option<String>,
    pub(crate) last_config_generation: u64,
    pub gui_active_profile: String,

    // Hardware polling channel
    pub(crate) rx: std::sync::mpsc::Receiver<mouser_engine::worker::DeviceStateUpdate>,
    pub(crate) tx: std::sync::mpsc::Sender<mouser_engine::worker::BackgroundTxCmd>,
    pub(crate) battery_pct: String,
    pub(crate) has_active_hidpp_battery: Option<bool>,

    // Background image decode channel — None once both images have been received
    pub(crate) img_rx: Option<std::sync::mpsc::Receiver<(egui::ColorImage, egui::ColorImage)>>,
    // CPU-side pixel buffers; Some until the first GPU upload, then None to free memory
    pub(crate) preloaded_mouse_image: Option<egui::ColorImage>,
    pub(crate) preloaded_customization_mouse_image: Option<egui::ColorImage>,
    pub(crate) current_connection_type: Option<String>,
    pub(crate) device_batteries: std::collections::HashMap<String, String>,
    pub(crate) device_conn_types: std::collections::HashMap<String, String>,
    pub(crate) toast_message: Option<String>,
    pub(crate) toast_shown_at: Option<std::time::Instant>,
    pub(crate) last_known_profile: String,
}

impl MouserApp {
    pub fn new(ctx: egui::Context, tray_icon: Option<tray_icon::TrayIcon>, engine: mouser_engine::Engine) -> Self {
        let cached = mouser_engine::cache::load_device_cache();
        let config = engine.get_config();
        let gui_active_profile = config.active_app_profile.clone();
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

        let repaint_ctx = ctx.clone();
        let (tx, rx) = mouser_engine::worker::spawn_background_worker(
            move || {
                repaint_ctx.request_repaint();
            },
            engine.active_profile_shared(),
        );

        Self {
            tray_icon,
            current_tray_icon_type: "mouse".to_string(),
            active_view: ActiveView::EmptyState,
            unifying_receiver_connected: None,
            bolt_receiver_connected: None,
            paired_devices: cached,
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
            rx,
            tx,
            battery_pct: "0".to_string(),
            has_active_hidpp_battery: None,
            img_rx: Some(img_rx),
            preloaded_mouse_image: None,
            preloaded_customization_mouse_image: None,
            current_connection_type: None,
            device_batteries: std::collections::HashMap::new(),
            device_conn_types: std::collections::HashMap::new(),
            toast_message: None,
            toast_shown_at: None,
            last_known_profile: String::new(),
            gui_active_profile,
        }
    }

    pub fn reload_config(&mut self) {
        let current_gen = self.engine.config_generation();
        if current_gen != self.last_config_generation {
            let fresh = self.engine.get_config();
            
            self.config.settings = fresh.settings;
            self.config.active_group = fresh.active_group;
            self.config.active_app_profile = fresh.active_app_profile;
            self.config.profile_groups = fresh.profile_groups;
            self.config.version = fresh.version;
            
            self.last_config_generation = current_gen;
        }
    }
}

pub fn get_layout_key_from_name(name: &str) -> String {
    let name = name.to_lowercase();
    if name.contains("mechanical") || name.contains("mchncl") {
        return "mx_mechanical".to_string();
    }
    if name.contains("master 3s") {
        "mx_master_3s".to_string()
    } else if name.contains("master 3") || name.contains("master 4") {
        "mx_master_3".to_string()
    } else if name.contains("master 2") || name.contains("master 2s") {
        "mx_master_2s".to_string()
    } else if name.contains("master") {
        "mx_master".to_string()
    } else if name.contains("anywhere 3s") {
        "mx_anywhere_3s".to_string()
    } else if name.contains("anywhere 3") {
        "mx_anywhere_3".to_string()
    } else if name.contains("anywhere") {
        "mx_anywhere".to_string()
    } else if name.contains("vertical") {
        "mx_vertical".to_string()
    } else if name.contains("ergo") {
        "mx_ergo".to_string()
    } else if name.contains("mx keys mini") {
        "mx_keys_mini".to_string()
    } else if name.contains("mx keys s") {
        "mx_keys_s".to_string()
    } else if name.contains("mx keys") {
        "mx_keys".to_string()
    } else if name.contains("mx mechanical mini") {
        "mx_mechanical_mini".to_string()
    } else if name.contains("mx mechanical") {
        "mx_mechanical".to_string()
    } else {
        "generic".to_string()
    }
}
