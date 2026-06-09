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
    pub(crate) bluetooth_available: bool,
    pub(crate) window_initialized: bool,
    pub config: mouser_engine::config::Config,
    pub engine: mouser_engine::Engine,
    pub updater: Updater,
    pub customizing_button: Option<crate::views::customization::mappings::CustomizingButton>,
    pub customization_tab: crate::views::customization::SidebarTab,
    pub(crate) last_config_generation: u64,

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
    pub(crate) toast_message: Option<String>,
    pub(crate) toast_shown_at: Option<std::time::Instant>,
    pub(crate) last_known_profile: String,
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
            bluetooth_available: true,
            window_initialized: false,
            config,
            engine,
            updater,
            customizing_button: None,
            customization_tab: crate::views::customization::SidebarTab::Buttons,
            last_config_generation: config_gen,
            rx,
            tx,
            battery_pct: "0".to_string(),
            has_active_hidpp_battery: None,
            img_rx: Some(img_rx),
            preloaded_mouse_image: None,
            preloaded_customization_mouse_image: None,
            current_connection_type: None,
            toast_message: None,
            toast_shown_at: None,
            last_known_profile: String::new(),
        }
    }

    pub fn reload_config(&mut self) {
        let current_gen = self.engine.config_generation();
        if current_gen != self.last_config_generation {
            self.config = self.engine.get_config();
            self.last_config_generation = current_gen;
        }
    }
}
