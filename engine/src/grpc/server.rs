/// gRPC server implementation for the Mouser daemon.
///
/// The generated tonic stubs are included from OUT_DIR. This module wraps
/// the `Engine` API and exposes every method as a gRPC RPC, plus two
/// server-streaming RPCs (`WatchConfig`, `WatchDeviceState`) that push
/// updates to connected GUI clients.
use std::{collections::HashMap, path::Path, pin::Pin};

use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt as _};
use tonic::{transport::Server, Request, Response, Status};

use crate::engine::Engine;

// Include the protobuf-generated types and service traits.
pub mod proto {
    tonic::include_proto!("mouser");
}

use proto::{
    mouser_daemon_server::{MouserDaemon, MouserDaemonServer},
    AppBindingsRequest, ConfigResponse, DeviceInfoResponse, DeviceStateResponse, Empty,
    FlowStatusResponse, ProfileIconRequest, ProfileMappingsRequest, SettingsJson, StatusResponse,
    StringValue,
};

// ─── UDS Connected newtype ─────────────────────────────────────────────────────

/// Newtype wrapping `tokio::net::UnixStream` that implements
/// `tonic::transport::server::Connected` so it can be used with
/// `Server::serve_with_incoming`.
struct UdsStream(tokio::net::UnixStream);

impl tonic::transport::server::Connected for UdsStream {
    type ConnectInfo = ();
    fn connect_info(&self) -> Self::ConnectInfo {}
}

impl tokio::io::AsyncRead for UdsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for UdsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

// ─── Internal broadcast channels ──────────────────────────────────────────────

/// Shared sender for config-change events; receivers stream `ConfigResponse` to
/// every connected GUI client.
#[derive(Clone)]
pub struct ConfigBroadcast(broadcast::Sender<ConfigResponse>);

/// Shared sender for device-state updates from the background worker; receivers
/// stream `DeviceStateResponse` to every connected GUI client.
#[derive(Clone)]
pub struct DeviceStateBroadcast(broadcast::Sender<DeviceStateResponse>);

// ─── Service implementation ────────────────────────────────────────────────────

pub struct MouserDaemonService {
    engine: Engine,
    config_tx: ConfigBroadcast,
    device_state_tx: DeviceStateBroadcast,
}

impl MouserDaemonService {
    fn ok() -> StatusResponse {
        StatusResponse {
            ok: true,
            error_message: String::new(),
        }
    }

    fn err(msg: impl ToString) -> StatusResponse {
        StatusResponse {
            ok: false,
            error_message: msg.to_string(),
        }
    }
}

// Helper: serialize Config → ConfigResponse
fn config_to_proto(engine: &Engine) -> ConfigResponse {
    let cfg = engine.get_config();
    let json = serde_json::to_string(&cfg).unwrap_or_default();
    ConfigResponse {
        config_json: json,
        generation: engine.config_generation(),
        ring_trigger_count: engine.get_ring_trigger_count(),
    }
}

#[tonic::async_trait]
impl MouserDaemon for MouserDaemonService {
    // ── Config ────────────────────────────────────────────────────────────────

    async fn get_config(&self, _req: Request<Empty>) -> Result<Response<ConfigResponse>, Status> {
        let cfg = self.engine.get_config();
        let json = serde_json::to_string(&cfg).unwrap_or_default();
        let gen = self.engine.config_generation();
        Ok(Response::new(ConfigResponse { config_json: json, generation: gen, ring_trigger_count: self.engine.get_ring_trigger_count() }))
    }

    async fn reload_config(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.reload_config();
        Ok(Response::new(Self::ok()))
    }

    async fn update_settings(
        &self,
        req: Request<SettingsJson>,
    ) -> Result<Response<StatusResponse>, Status> {
        // Deserialize the partial settings JSON and apply field-by-field.
        let json = req.into_inner().settings_json;
        let patch: serde_json::Value = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(e) => return Ok(Response::new(Self::err(e))),
        };

        let p = &patch;
        let dpi = p["dpi"].as_u64().unwrap_or(0) as u32;
        let ss_mode = p["smart_shift_mode"].as_str().unwrap_or("").to_string();
        let ss_enabled = p["smart_shift_enabled"].as_bool().unwrap_or(false);
        let ss_threshold = p["smart_shift_threshold"].as_u64().unwrap_or(0) as u8;
        let invert_hscroll = p["invert_hscroll"].as_bool().unwrap_or(false);
        let invert_vscroll = p["invert_vscroll"].as_bool().unwrap_or(false);
        let gesture_threshold = p["gesture_threshold"].as_i64().unwrap_or(0) as i32;
        let gesture_deadzone = p["gesture_deadzone"].as_i64().unwrap_or(0) as i32;
        let accent_color = p["accent_color"].as_str().unwrap_or("").to_string();
        let hscroll_threshold = p["hscroll_threshold"].as_i64().unwrap_or(0) as i32;

        self.engine.update_global_settings(
            dpi,
            ss_mode,
            ss_enabled,
            ss_threshold,
            invert_hscroll,
            invert_vscroll,
            gesture_threshold,
            gesture_deadzone,
            accent_color,
            hscroll_threshold,
        );
        Ok(Response::new(Self::ok()))
    }

    async fn update_keyboard_layout(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.update_keyboard_layout(&req.into_inner().value);
        Ok(Response::new(Self::ok()))
    }

    // ── Profiles ──────────────────────────────────────────────────────────────

    async fn select_profile(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.select_profile(&req.into_inner().value);
        Ok(Response::new(Self::ok()))
    }

    async fn add_profile(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.add_profile(&req.into_inner().value);
        Ok(Response::new(Self::ok()))
    }

    async fn delete_profile(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.delete_profile(&req.into_inner().value);
        Ok(Response::new(Self::ok()))
    }

    async fn update_profile_mappings(
        &self,
        req: Request<ProfileMappingsRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let inner = req.into_inner();
        let mappings: HashMap<String, String> = match serde_json::from_str(&inner.mappings_json) {
            Ok(m) => m,
            Err(e) => return Ok(Response::new(Self::err(e))),
        };
        self.engine
            .update_profile_mappings(&inner.profile_name, mappings);
        Ok(Response::new(Self::ok()))
    }

    async fn update_app_bindings(
        &self,
        req: Request<AppBindingsRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let inner = req.into_inner();
        self.engine
            .update_app_bindings(&inner.profile_name, &inner.exec);
        Ok(Response::new(Self::ok()))
    }

    async fn set_profile_icon(
        &self,
        req: Request<ProfileIconRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let inner = req.into_inner();
        self.engine
            .set_profile_icon(&inner.profile_name, &inner.icon_data);
        Ok(Response::new(Self::ok()))
    }

    // ── Profile Groups ────────────────────────────────────────────────────────

    async fn select_profile_group(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.select_profile_group(&req.into_inner().value);
        Ok(Response::new(Self::ok()))
    }

    async fn add_profile_group(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.add_profile_group(&req.into_inner().value);
        Ok(Response::new(Self::ok()))
    }

    async fn delete_profile_group(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.delete_profile_group(&req.into_inner().value);
        Ok(Response::new(Self::ok()))
    }

    // ── Device Info ───────────────────────────────────────────────────────────

    async fn execute_engine_action(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        self.engine.execute_engine_action(&req.into_inner().value);
        // The ring trigger (actions_ring_open) bumps the engine counter but is
        // otherwise never broadcast; push a fresh snapshot so the GUI's poll sees it.
        let _ = self.config_tx.0.send(config_to_proto(&self.engine));
        Ok(Response::new(Self::ok()))
    }

    async fn get_device_info(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<DeviceInfoResponse>, Status> {
        let connected = self.engine.device_connected();
        let device_name = self.engine.selected_device_name();
        let channel = self.engine.active_host_channel().unwrap_or(0) as u32;
        Ok(Response::new(DeviceInfoResponse {
            device_connected: connected,
            device_name,
            active_host_channel: channel,
        }))
    }

    async fn get_devices(&self, _req: Request<Empty>) -> Result<Response<proto::DeviceListResponse>, Status> {
        let devices = self.engine.device_list().into_iter()
            .map(|(serial, layout, active_app_profile, profile_count)| proto::DeviceInfo {
                serial, layout, active_app_profile, profile_count,
            })
            .collect();
        Ok(Response::new(proto::DeviceListResponse { devices }))
    }

    async fn select_device(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        let serial = req.into_inner().value;
        if self.engine.select_device(&serial) {
            Ok(Response::new(Self::ok()))
        } else {
            Ok(Response::new(Self::err(format!("device not found: {serial}"))))
        }
    }

    async fn get_flow_status(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<FlowStatusResponse>, Status> {
        Ok(Response::new(FlowStatusResponse {
            is_searching: false,
        }))
    }

    // ── Pairing ───────────────────────────────────────────────────────────────

    async fn unpair_device(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        let mac = req.into_inner().value;
        crate::bluetooth::unpair_device(&mac);
        Ok(Response::new(Self::ok()))
    }

    // ── Streaming ─────────────────────────────────────────────────────────────

    type WatchConfigStream =
        Pin<Box<dyn Stream<Item = Result<ConfigResponse, Status>> + Send + 'static>>;

    async fn watch_config(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<Self::WatchConfigStream>, Status> {
        let rx = self.config_tx.0.subscribe();
        // Send the current config immediately as the first item.
        let initial = config_to_proto(&self.engine);
        let stream = BroadcastStream::new(rx).filter_map(|r| r.ok()).map(Ok);
        // Prepend the initial snapshot.
        let stream = tokio_stream::once(Ok(initial)).chain(stream);
        Ok(Response::new(Box::pin(stream)))
    }

    type WatchDeviceStateStream =
        Pin<Box<dyn Stream<Item = Result<DeviceStateResponse, Status>> + Send + 'static>>;

    async fn watch_device_state(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<Self::WatchDeviceStateStream>, Status> {
        let rx = self.device_state_tx.0.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|r| r.ok()).map(Ok);
        Ok(Response::new(Box::pin(stream)))
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Start the gRPC server bound to a UNIX domain socket and return handles to
/// the broadcast channels so the daemon loop can push config/device-state
/// updates to all connected GUI clients.
pub fn start_grpc_server(
    engine: Engine,
    socket_path: &Path,
) -> anyhow::Result<(ConfigBroadcast, DeviceStateBroadcast)> {
    // Remove stale socket if present.
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }

    let (config_tx, _) = broadcast::channel::<ConfigResponse>(64);
    let (device_state_tx, _) = broadcast::channel::<DeviceStateResponse>(64);

    let config_broadcast = ConfigBroadcast(config_tx.clone());
    let device_state_broadcast = DeviceStateBroadcast(device_state_tx.clone());

    let svc = MouserDaemonService {
        engine,
        config_tx: ConfigBroadcast(config_tx),
        device_state_tx: DeviceStateBroadcast(device_state_tx),
    };

    let uds_path = socket_path.to_path_buf();

    // Spawn a dedicated tokio runtime on a background thread so the gRPC server
    // doesn't block the daemon's main thread (which handles signals).
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async move {
            use tokio::net::UnixListener;

            let listener = UnixListener::bind(&uds_path).expect("bind gRPC UNIX socket");

            log::info!("[gRPC] Server listening on {:?}", uds_path);

            // Build a stream of UdsStream-wrapped connections.
            // UdsStream implements tonic::Connected + AsyncRead + AsyncWrite.
            // We use tokio_stream::wrappers::ReceiverStream to avoid type inference
            // issues with async_stream::try_stream! and tonic's error bounds.
            let (tx_conn, rx_conn) =
                tokio::sync::mpsc::channel::<Result<UdsStream, std::io::Error>>(32);
            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            if tx_conn.send(Ok(UdsStream(stream))).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            log::warn!("[gRPC] accept error: {e}");
                        }
                    }
                }
            });

            let incoming = tokio_stream::wrappers::ReceiverStream::new(rx_conn).map(|r| {
                r.map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
            });

            use tokio_stream::StreamExt as _;
            Server::builder()
                .add_service(MouserDaemonServer::new(svc))
                .serve_with_incoming(incoming)
                .await
                .expect("gRPC server error");
        });
    });

    Ok((config_broadcast, device_state_broadcast))
}

/// Push a fresh config snapshot to all watching GUI clients.
pub fn broadcast_config(cfg: &crate::config::Config, generation: u64, tx: &ConfigBroadcast) {
    let json = serde_json::to_string(cfg).unwrap_or_default();
    let msg = ConfigResponse {
        config_json: json,
        generation,
        ring_trigger_count: 0,
    };
    let _ = tx.0.send(msg);
}

/// Push a device-state snapshot to all watching GUI clients.
pub fn broadcast_device_state(
    update: &crate::worker::DeviceStateUpdate,
    tx: &DeviceStateBroadcast,
) {
    let paired_json = serde_json::to_string(&*update.paired_devices).unwrap_or_default();
    let msg = DeviceStateResponse {
        unifying_receiver_connected: update.unifying_receiver_connected,
        bolt_receiver_connected: update.bolt_receiver_connected,
        bluetooth_available: update.bluetooth_available,
        battery_pct: update.battery_pct.clone(),
        has_active_hidpp_battery: update.has_active_hidpp_battery,
        paired_devices_json: paired_json,
        battery_status: update.battery_status.clone(),
    };
    let _ = tx.0.send(msg);
}
