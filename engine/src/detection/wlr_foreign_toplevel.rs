//! Frontmost backend using the wlroots `zwlr_foreign_toplevel_management_v1` protocol.
//! Ported and adapted from OpenLogi's implementation.

use std::collections::HashMap;
use std::os::unix::io::AsRawFd;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use wayland_client::backend::ObjectId;
use wayland_client::protocol::wl_callback;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle, event_created_child};
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::{
    self, ZwlrForeignToplevelHandleV1,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::{
    self, ZwlrForeignToplevelManagerV1,
};

/// Highest protocol version this backend understands.
const MANAGER_MAX_VERSION: u32 = 3;

/// Deadline for the two `wl_display.sync` round-trips in `Session::open`.
const INIT_TIMEOUT: Duration = Duration::from_secs(3);

/// Maximum time the poll-path drain will wait for new Wayland events.
const POLL_CAP_MS: u64 = 25;

/// Accumulated per-toplevel data.
#[derive(Default)]
struct Toplevel {
    app_id: Option<String>,
    activated: bool,
    pending_app_id: Option<String>,
    pending_activated: bool,
}

/// Dispatch state: the bound manager plus the toplevels seen so far.
#[derive(Default)]
struct State {
    manager: Option<ZwlrForeignToplevelManagerV1>,
    toplevels: HashMap<ObjectId, Toplevel>,
    finished: bool,
    sync_done: bool,
}

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == ZwlrForeignToplevelManagerV1::interface().name {
                let version = version.min(MANAGER_MAX_VERSION);
                let manager =
                    registry.bind::<ZwlrForeignToplevelManagerV1, (), Self>(name, version, qh, ());
                state.manager = Some(manager);
            }
        }
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                state.toplevels.insert(toplevel.id(), Toplevel::default());
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                state.finished = true;
                state.manager = None;
            }
            _ => {}
        }
    }

    event_created_child!(State, ZwlrForeignToplevelManagerV1, [
        zwlr_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (ZwlrForeignToplevelHandleV1, ()),
    ]);
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for State {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zwlr_foreign_toplevel_handle_v1::Event;

        let id = handle.id();
        match event {
            Event::AppId { app_id } => {
                if let Some(toplevel) = state.toplevels.get_mut(&id) {
                    toplevel.pending_app_id = Some(app_id);
                }
            }
            Event::State { state: states } => {
                let activated = is_activated(&states);
                if let Some(toplevel) = state.toplevels.get_mut(&id) {
                    toplevel.pending_activated = activated;
                }
            }
            Event::Done => {
                if let Some(toplevel) = state.toplevels.get_mut(&id) {
                    if toplevel.pending_app_id.is_some() {
                        toplevel.app_id = toplevel.pending_app_id.clone();
                    }
                    toplevel.activated = toplevel.pending_activated;
                }
            }
            Event::Closed => {
                state.toplevels.remove(&id);
                handle.destroy();
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.sync_done = true;
        }
    }
}

fn is_activated(states: &[u8]) -> bool {
    use zwlr_foreign_toplevel_handle_v1::State;

    let (values, _partial) = states.as_chunks::<4>();
    values.iter().any(|value| {
        State::try_from(u32::from_ne_bytes(*value)).is_ok_and(|s| s == State::Activated)
    })
}

fn millis_until(deadline: Instant) -> i32 {
    i32::try_from(
        deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(i32::MAX as u128),
    )
    .unwrap_or(i32::MAX)
}

fn poll_fd(fd: libc::c_int, deadline: Instant) -> bool {
    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN | libc::POLLERR,
        revents: 0,
    };
    loop {
        let timeout_ms = millis_until(deadline);
        if timeout_ms == 0 {
            return false;
        }
        // SAFETY: `pfd` is owned by this thread for the duration of the call and has length 1.
        let r = unsafe { libc::poll(&mut pfd, 1, timeout_ms) };
        if r > 0 {
            return true;
        }
        if r == 0 {
            return false;
        }
        let e = io_error_raw();
        if e != libc::EINTR {
            return false;
        }
    }
}

fn io_error_raw() -> i32 {
    // SAFETY: __errno_location is thread-safe.
    unsafe { *libc::__errno_location() }
}

fn timed_roundtrip(
    conn: &Connection,
    queue: &mut EventQueue<State>,
    state: &mut State,
    deadline: Instant,
) -> bool {
    state.sync_done = false;
    let qh = queue.handle();
    let _ = conn.display().sync(&qh, ());

    loop {
        if queue.flush().is_err() {
            return false;
        }
        if queue.dispatch_pending(state).is_err() {
            return false;
        }
        if state.sync_done {
            return true;
        }
        if millis_until(deadline) == 0 {
            return false;
        }

        match queue.prepare_read() {
            None => {}
            Some(guard) => {
                let fd = guard.connection_fd().as_raw_fd();
                if !poll_fd(fd, deadline) {
                    return false;
                }
                if guard.read().is_err() {
                    return false;
                }
            }
        }
    }
}

fn drain_events(queue: &mut EventQueue<State>, state: &mut State) {
    if queue.flush().is_err() || queue.dispatch_pending(state).is_err() {
        state.finished = true;
        return;
    }

    let deadline = Instant::now() + Duration::from_millis(POLL_CAP_MS);
    match queue.prepare_read() {
        None => {}
        Some(guard) => {
            let fd = guard.connection_fd().as_raw_fd();
            if poll_fd(fd, deadline)
                && (guard.read().is_err() || queue.dispatch_pending(state).is_err())
            {
                state.finished = true;
            }
        }
    }
}

struct Session {
    _conn: Connection,
    queue: EventQueue<State>,
    state: State,
}

impl Session {
    fn open() -> Option<Self> {
        let conn = Connection::connect_to_env().ok()?;
        let mut queue = conn.new_event_queue();
        let qh = queue.handle();

        let _registry = conn.display().get_registry(&qh, ());
        let mut state = State::default();
        let deadline = Instant::now() + INIT_TIMEOUT;

        if !timed_roundtrip(&conn, &mut queue, &mut state, deadline) {
            return None;
        }
        if state.manager.is_none() {
            return None;
        }

        if !timed_roundtrip(&conn, &mut queue, &mut state, deadline) {
            return None;
        }

        Some(Self {
            _conn: conn,
            queue,
            state,
        })
    }
}

pub struct WlrForeignToplevelTracker {
    session: Mutex<Option<Session>>,
}

impl WlrForeignToplevelTracker {
    pub fn connect() -> Option<Self> {
        Session::open().map(|s| Self {
            session: Mutex::new(Some(s)),
        })
    }

    pub fn get_foreground_app_id(&self) -> Option<String> {
        let mut guard = self.session.lock().ok()?;

        let needs_reconnect = guard.as_ref().is_none_or(|s| s.state.finished);
        if needs_reconnect {
            *guard = Session::open();
            if guard.is_none() {
                return None;
            }
        }

        let Session { queue, state, .. } = guard.as_mut()?;
        drain_events(queue, state);
        if state.finished {
            return None;
        }

        state
            .toplevels
            .values()
            .find(|toplevel| toplevel.activated)
            .and_then(|toplevel| toplevel.app_id.clone())
    }
}
