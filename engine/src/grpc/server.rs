/// gRPC server implementation for the Mouser daemon.
///
/// The generated tonic stubs are included from OUT_DIR. This module wraps
/// the `Engine` API and exposes every method as a gRPC RPC, plus two
/// server-streaming RPCs (`WatchConfig`, `WatchDeviceState`) that push
/// updates to connected GUI clients.
use std::{collections::HashMap, path::Path, pin::Pin, sync::mpsc::Sender};

use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt as _};
use tonic::{transport::Server, Request, Response, Status};

use crate::engine::Engine;
use crate::worker::BackgroundTxCmd;

// Include the protobuf-generated types and service traits.
pub mod proto {
    tonic::include_proto!("mouser");
}

use proto::{
    mouser_daemon_server::{MouserDaemon, MouserDaemonServer},
    ActionsRingState, AppBindingsRequest, ConfigResponse, DeviceInfoResponse, DeviceStateResponse,
    DeviceProfileRequest, Empty, FlowStatusResponse, ProfileIconRequest, ProfileMappingsRequest,
    SettingsJson,
    StatusResponse, StringValue,
};

/// Extract the device key carried in a request, falling back to the daemon's
/// currently-selected device when the request omits one (legacy/compat path).
fn device_key_from_proto(d: &Option<proto::DeviceKey>) -> crate::config::DeviceKey {
    match d {
        Some(k) => crate::config::DeviceKey {
            serial: k.serial.clone(),
            layout: k.layout.clone(),
        },
        None => crate::config::DeviceKey::default(),
    }
}

fn device_key_from_req(
    req: &Request<DeviceProfileRequest>,
) -> crate::config::DeviceKey {
    device_key_from_proto(&req.get_ref().device)
}

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

/// Shared sender for Actions Ring open/close signals; receivers stream
/// `ActionsRingState` to every connected GUI client.
#[derive(Clone)]
pub struct ActionsRingBroadcast(broadcast::Sender<ActionsRingState>);

// ─── Service implementation ────────────────────────────────────────────────────

pub struct MouserDaemonService {
    engine: Engine,
    config_tx: ConfigBroadcast,
    device_state_tx: DeviceStateBroadcast,
    actions_ring_tx: ActionsRingBroadcast,
    worker_tx: Sender<BackgroundTxCmd>,
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
    }
}

#[tonic::async_trait]
impl MouserDaemon for MouserDaemonService {
    // ── Config ────────────────────────────────────────────────────────────────

    async fn get_config(&self, _req: Request<Empty>) -> Result<Response<ConfigResponse>, Status> {
        Ok(Response::new(config_to_proto(&self.engine)))
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
        req: Request<DeviceProfileRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let device = device_key_from_req(&req);
        self.engine.select_profile(&device, &req.into_inner().name);
        Ok(Response::new(Self::ok()))
    }

    async fn add_profile(
        &self,
        req: Request<DeviceProfileRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let device = device_key_from_req(&req);
        self.engine.add_profile(&device, &req.into_inner().name);
        Ok(Response::new(Self::ok()))
    }

    async fn delete_profile(
        &self,
        req: Request<DeviceProfileRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let device = device_key_from_req(&req);
        self.engine.delete_profile(&device, &req.into_inner().name);
        Ok(Response::new(Self::ok()))
    }

    async fn update_profile_mappings(
        &self,
        req: Request<ProfileMappingsRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let inner = req.into_inner();
        let device = device_key_from_proto(&inner.device);
        let mappings: HashMap<String, String> = match serde_json::from_str(&inner.mappings_json) {
            Ok(m) => m,
            Err(e) => return Ok(Response::new(Self::err(e))),
        };
        self.engine
            .update_profile_mappings(&device, &inner.profile_name, mappings);
        Ok(Response::new(Self::ok()))
    }

    async fn update_app_bindings(
        &self,
        req: Request<AppBindingsRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let inner = req.into_inner();
        let device = device_key_from_proto(&inner.device);
        self.engine
            .update_app_bindings(&device, &inner.profile_name, &inner.exec);
        Ok(Response::new(Self::ok()))
    }

    async fn set_profile_icon(
        &self,
        req: Request<ProfileIconRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let inner = req.into_inner();
        let device = device_key_from_proto(&inner.device);
        self.engine
            .set_profile_icon(&device, &inner.profile_name, &inner.icon_data);
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
        // Route through the background worker: it removes the device from the OS
        // (bluetoothctl remove), drops it from the on-disk cache, and re-polls so
        // the GUI receives a fresh device list. Sending the command (rather than
        // calling bluetooth::unpair_device directly) is the single source of truth.
        let _ = self.worker_tx.send(BackgroundTxCmd::Unpair(mac.clone()));
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

    type WatchActionsRingStream =
        Pin<Box<dyn Stream<Item = Result<ActionsRingState, Status>> + Send + 'static>>;

    async fn watch_actions_ring(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<Self::WatchActionsRingStream>, Status> {
        let rx = self.actions_ring_tx.0.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|r| r.ok()).map(Ok);
        Ok(Response::new(Box::pin(stream)))
    }

    async fn execute_action(
        &self,
        req: Request<StringValue>,
    ) -> Result<Response<StatusResponse>, Status> {
        let action_id = req.into_inner().value;
        self.engine.execute_engine_action(&action_id);
        // Closing the ring after the selected action runs is the GUI's job,
        // but broadcasting a close here also keeps the engine authoritative.
        crate::grpc::broadcast_actions_ring(false, 0, "", "", &self.actions_ring_tx);
        Ok(Response::new(Self::ok()))
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Start the gRPC server bound to a UNIX domain socket and return handles to
/// the broadcast channels so the daemon loop can push config/device-state
/// updates to all connected GUI clients.
pub fn start_grpc_server(
    engine: Engine,
    socket_path: &Path,
    worker_tx: Sender<BackgroundTxCmd>,
) -> anyhow::Result<(ConfigBroadcast, DeviceStateBroadcast, ActionsRingBroadcast)> {
    // Remove stale socket if present.
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }

    let (config_tx, _) = broadcast::channel::<ConfigResponse>(64);
    let (device_state_tx, _) = broadcast::channel::<DeviceStateResponse>(64);
    let (actions_ring_tx, _) = broadcast::channel::<ActionsRingState>(64);

    let config_broadcast = ConfigBroadcast(config_tx.clone());
    let device_state_broadcast = DeviceStateBroadcast(device_state_tx.clone());
    let actions_ring_broadcast = ActionsRingBroadcast(actions_ring_tx.clone());

    let svc = MouserDaemonService {
        engine,
        config_tx: ConfigBroadcast(config_tx),
        device_state_tx: DeviceStateBroadcast(device_state_tx),
        actions_ring_tx: ActionsRingBroadcast(actions_ring_tx),
        worker_tx,
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

    Ok((
        config_broadcast,
        device_state_broadcast,
        actions_ring_broadcast,
    ))
}

/// Push a fresh config snapshot to all watching GUI clients.
pub fn broadcast_config(cfg: &crate::config::Config, generation: u64, tx: &ConfigBroadcast) {
    let json = serde_json::to_string(cfg).unwrap_or_default();
    let msg = ConfigResponse {
        config_json: json,
        generation,
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

/// Push an Actions Ring open/close signal to all watching GUI clients.
/// `layout_json` / `folders_json` are ignored when `open == false`.
pub fn broadcast_actions_ring(
    open: bool,
    session_id: u64,
    layout_json: &str,
    folders_json: &str,
    tx: &ActionsRingBroadcast,
) {
    let msg = proto::ActionsRingState {
        open,
        session_id,
        layout_json: layout_json.to_string(),
        folders_json: folders_json.to_string(),
    };
    let _ = tx.0.send(msg);
}
