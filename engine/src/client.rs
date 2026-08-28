/// Synchronous gRPC client wrapper for the Mouser daemon.
///
/// `EngineClient` mirrors the public API of `Engine` exactly so the GUI can
/// swap between the two types with minimal changes.  Internally it holds a
/// `tonic` generated stub and a `tokio::runtime::Handle`; every method uses
/// `block_on` to make the async call synchronous from the egui render thread.
use std::{collections::HashMap, path::Path, sync::mpsc::Sender, sync::Arc};

use tonic::transport::{Channel, Endpoint, Uri};

use crate::{config::Config, worker::DeviceStateUpdate};

// The generated tonic client stub lives in the proto submodule.
use crate::grpc::server::proto::mouser_daemon_client::MouserDaemonClient;
use crate::grpc::server::proto::{
    AppBindingsRequest, Empty, ProfileIconRequest, ProfileMappingsRequest, SettingsJson,
    StringValue,
};

// ─── EngineClient ─────────────────────────────────────────────────────────────

/// A synchronous, thread-safe wrapper around the tonic gRPC stub.
///
/// All method signatures mirror `Engine` so the GUI can treat the two types
/// identically (aside from the type name in the field declaration).
#[derive(Clone)]
pub struct EngineClient {
    stub: MouserDaemonClient<Channel>,
    rt: Arc<tokio::runtime::Runtime>,
    // Local generation counter so `get_config_if_changed` keeps working.
    last_config: Arc<std::sync::Mutex<(u64, Config)>>,
}

impl EngineClient {
    /// Connect to the daemon's UNIX socket.  Retries are the caller's
    /// responsibility (see `main.rs`).
    pub fn connect(socket_path: &Path) -> anyhow::Result<Self> {
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("mouser-rpc-client")
                .build()?,
        );

        let path_str = socket_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("socket path is not valid UTF-8"))?
            .to_string();

        let channel = rt.block_on(async {
            let path = path_str.clone();
            Endpoint::try_from("http://[::]:50051")?
                .connect_with_connector(tower::service_fn(move |_: Uri| {
                    let path = path.clone();
                    async move {
                        let stream = tokio::net::UnixStream::connect(&path).await?;
                        Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(stream))
                    }
                }))
                .await
        })?;

        let mut stub = MouserDaemonClient::new(channel);

        // Fetch initial config to seed our local cache.
        let init_resp = rt.block_on(stub.get_config(Empty {}))?;
        let init_cfg: Config = serde_json::from_str(&init_resp.into_inner().config_json)
            .unwrap_or_else(|_| Config::load());

        Ok(Self {
            stub,
            rt,
            last_config: Arc::new(std::sync::Mutex::new((0, init_cfg))),
        })
    }

    // ─── Helpers ─────────────────────────────────────────────────────────────

    fn call<F, Fut, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(MouserDaemonClient<Channel>) -> Fut,
        Fut: std::future::Future<Output = Result<tonic::Response<T>, tonic::Status>>,
    {
        let stub = self.stub.clone();
        match self.rt.block_on(f(stub)) {
            Ok(resp) => Some(resp.into_inner()),
            Err(e) => {
                log::warn!("[EngineClient] gRPC call failed: {e}");
                None
            }
        }
    }

    fn call_ok<F, Fut>(&self, f: F)
    where
        F: FnOnce(MouserDaemonClient<Channel>) -> Fut,
        Fut: std::future::Future<
            Output = Result<
                tonic::Response<crate::grpc::server::proto::StatusResponse>,
                tonic::Status,
            >,
        >,
    {
        if let Some(resp) = self.call(f) {
            if !resp.ok {
                log::warn!(
                    "[EngineClient] daemon returned error: {}",
                    resp.error_message
                );
            }
        }
    }

    // ─── Config API (mirrors Engine) ─────────────────────────────────────────

    pub fn get_config(&self) -> Config {
        if let Some(resp) = self.call(|mut s| async move { s.get_config(Empty {}).await }) {
            let cfg: Config =
                serde_json::from_str(&resp.config_json).unwrap_or_else(|_| Config::load());
            // Update local cache.
            if let Ok(mut guard) = self.last_config.lock() {
                guard.0 = resp.generation;
                guard.1 = cfg.clone();
            }
            cfg
        } else {
            self.last_config
                .lock()
                .map(|g| g.1.clone())
                .unwrap_or_else(|_| Config::load())
        }
    }

    pub fn config_generation(&self) -> u64 {
        self.last_config.lock().map(|g| g.0).unwrap_or(0)
    }

    pub fn get_config_if_changed(&self, last_gen: u64) -> Option<Config> {
        let current = self.config_generation();
        if current > last_gen {
            Some(
                self.last_config
                    .lock()
                    .map(|g| g.1.clone())
                    .unwrap_or_else(|_| Config::load()),
            )
        } else {
            None
        }
    }

    pub fn reload_config(&self) {
        self.call_ok(|mut s| async move { s.reload_config(Empty {}).await });
    }

    pub fn update_keyboard_layout(&self, layout: &str) {
        let v = layout.to_string();
        self.call_ok(
            |mut s| async move { s.update_keyboard_layout(StringValue { value: v }).await },
        );
    }

    // ─── Profile API (mirrors Engine) ────────────────────────────────────────

    pub fn select_profile(&self, name: &str) {
        let v = name.to_string();
        self.call_ok(|mut s| async move { s.select_profile(StringValue { value: v }).await });
    }

    pub fn add_profile(&self, name: &str) {
        let v = name.to_string();
        self.call_ok(|mut s| async move { s.add_profile(StringValue { value: v }).await });
    }

    pub fn delete_profile(&self, name: &str) {
        let v = name.to_string();
        self.call_ok(|mut s| async move { s.delete_profile(StringValue { value: v }).await });
    }

    pub fn update_profile_mappings(&self, profile_name: &str, mappings: HashMap<String, String>) {
        let profile_name = profile_name.to_string();
        let mappings_json = serde_json::to_string(&mappings).unwrap_or_default();
        self.call_ok(|mut s| async move {
            s.update_profile_mappings(ProfileMappingsRequest {
                profile_name,
                mappings_json,
            })
            .await
        });
    }

    pub fn update_app_bindings(&self, profile_name: &str, exec: &str) {
        let profile_name = profile_name.to_string();
        let exec = exec.to_string();
        self.call_ok(|mut s| async move {
            s.update_app_bindings(AppBindingsRequest { profile_name, exec })
                .await
        });
    }

    pub fn set_profile_icon(&self, profile_name: &str, icon_data: &str) {
        let profile_name = profile_name.to_string();
        let icon_data = icon_data.to_string();
        self.call_ok(|mut s| async move {
            s.set_profile_icon(ProfileIconRequest {
                profile_name,
                icon_data,
            })
            .await
        });
    }

    // ─── Profile Group API (mirrors Engine) ──────────────────────────────────

    pub fn select_profile_group(&self, group_name: &str) {
        let v = group_name.to_string();
        self.call_ok(|mut s| async move { s.select_profile_group(StringValue { value: v }).await });
    }

    pub fn add_profile_group(&self, group_name: &str) {
        let v = group_name.to_string();
        self.call_ok(|mut s| async move { s.add_profile_group(StringValue { value: v }).await });
    }

    pub fn delete_profile_group(&self, group_name: &str) {
        let v = group_name.to_string();
        self.call_ok(|mut s| async move { s.delete_profile_group(StringValue { value: v }).await });
    }

    // ─── Settings API (mirrors Engine::update_global_settings) ───────────────

    pub fn update_global_settings(
        &self,
        dpi: u32,
        smart_shift_mode: String,
        smart_shift_enabled: bool,
        smart_shift_threshold: u8,
        invert_hscroll: bool,
        invert_vscroll: bool,
        gesture_threshold: i32,
        gesture_deadzone: i32,
        accent_color: String,
        hscroll_threshold: i32,
    ) {
        let json = serde_json::json!({
            "dpi": dpi,
            "smart_shift_mode": smart_shift_mode,
            "smart_shift_enabled": smart_shift_enabled,
            "smart_shift_threshold": smart_shift_threshold,
            "invert_hscroll": invert_hscroll,
            "invert_vscroll": invert_vscroll,
            "gesture_threshold": gesture_threshold,
            "gesture_deadzone": gesture_deadzone,
            "accent_color": accent_color,
            "hscroll_threshold": hscroll_threshold,
        })
        .to_string();
        self.call_ok(|mut s| async move {
            s.update_settings(SettingsJson {
                settings_json: json,
            })
            .await
        });
    }

    // ─── Device Info (mirrors Engine) ────────────────────────────────────────

    pub fn device_connected(&self) -> bool {
        let resp = self.call(|mut s| async move { s.get_device_info(Empty {}).await });
        resp.as_ref().map(|r| r.device_connected).unwrap_or(false)
    }

    pub fn selected_device_name(&self) -> String {
        let resp = self.call(|mut s| async move { s.get_device_info(Empty {}).await });
        resp.map(|r| r.device_name)
            .unwrap_or_else(|| "MX Master 3".to_string())
    }

    pub fn active_host_channel(&self) -> Option<u8> {
        let resp = self.call(|mut s| async move { s.get_device_info(Empty {}).await })?;
        if resp.active_host_channel == 0 {
            None
        } else {
            Some(resp.active_host_channel as u8)
        }
    }

    // ─── Pairing ─────────────────────────────────────────────────────────────

    pub fn unpair_device(&self, mac: &str) {
        let v = mac.to_string();
        self.call_ok(|mut s| async move { s.unpair_device(StringValue { value: v }).await });
    }

    // ─── Background streaming subscriptions ──────────────────────────────────

    /// Spawn a background thread that streams config updates from the daemon
    /// and pushes new `Config` values into `tx`.  The thread exits when `tx`
    /// is dropped (send error).
    pub fn subscribe_config(
        self: Arc<Self>,
        tx: Sender<Config>,
        repaint: impl Fn() + Send + 'static,
    ) {
        let client = self.clone();
        std::thread::spawn(move || {
            let mut stub = client.stub.clone();
            let result = client.rt.block_on(async {
                let mut stream = stub.watch_config(Empty {}).await?.into_inner();
                while let Some(item) = stream.message().await? {
                    let cfg: Config = match serde_json::from_str(&item.config_json) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    // Update local generation cache.
                    if let Ok(mut guard) = client.last_config.lock() {
                        guard.0 = item.generation;
                        guard.1 = cfg.clone();
                    }
                    if tx.send(cfg).is_err() {
                        break; // GUI dropped the receiver — exit cleanly.
                    }
                    repaint();
                }
                Ok::<_, tonic::Status>(())
            });
            if let Err(e) = result {
                log::warn!("[EngineClient] config stream ended: {e}");
            }
        });
    }

    /// Spawn a background thread that streams device-state updates and pushes
    /// them into `tx`.
    pub fn subscribe_device_state(
        self: Arc<Self>,
        tx: Sender<DeviceStateUpdate>,
        repaint: impl Fn() + Send + 'static,
    ) {
        let client = self.clone();
        std::thread::spawn(move || {
            let mut stub = client.stub.clone();
            let result = client.rt.block_on(async {
                let mut stream = stub.watch_device_state(Empty {}).await?.into_inner();
                while let Some(item) = stream.message().await? {
                    let paired: Vec<(String, String, bool)> =
                        serde_json::from_str(&item.paired_devices_json).unwrap_or_default();
                    let update = DeviceStateUpdate {
                        unifying_receiver_connected: item.unifying_receiver_connected,
                        bolt_receiver_connected: item.bolt_receiver_connected,
                        bluetooth_available: item.bluetooth_available,
                        paired_devices: std::sync::Arc::new(paired),
                        battery_pct: item.battery_pct,
                        battery_status: item.battery_status,
                        has_active_hidpp_battery: item.has_active_hidpp_battery,
                        active_profile: String::new(), // filled by daemon stream context
                    };
                    if tx.send(update).is_err() {
                        break;
                    }
                    repaint();
                }
                Ok::<_, tonic::Status>(())
            });
            if let Err(e) = result {
                log::warn!("[EngineClient] device-state stream ended: {e}");
            }
        });
    }
}
