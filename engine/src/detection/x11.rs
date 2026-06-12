use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};

pub fn get_pid_via_active_window(
    conn: &impl Connection,
    root: u32,
) -> Result<Option<u32>, ReplyError> {
    let active_window_atom = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
        .reply()?
        .atom;
    let pid_atom = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?.atom;

    let resp = conn
        .get_property(false, root, active_window_atom, AtomEnum::WINDOW, 0, 1)?
        .reply()?;

    if resp.value_len == 0 {
        return Ok(None);
    }

    let window_id = resp.value32().and_then(|mut iter| iter.next());
    let Some(window_id) = window_id else {
        return Ok(None);
    };
    if window_id == 0 {
        return Ok(None);
    }

    let pid_resp = conn
        .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)?
        .reply()?;

    if pid_resp.value_len == 0 {
        return Ok(None);
    }

    let pid = pid_resp.value32().and_then(|mut iter| iter.next());
    Ok(pid)
}

pub fn get_pid_via_stacking_list(
    conn: &impl Connection,
    root: u32,
) -> Result<Option<u32>, ReplyError> {
    let stacking_atom = conn
        .intern_atom(false, b"_NET_CLIENT_LIST_STACKING")?
        .reply()?
        .atom;
    let pid_atom = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?.atom;

    let resp = conn
        .get_property(false, root, stacking_atom, AtomEnum::WINDOW, 0, 1024)?
        .reply()?;

    if resp.value_len == 0 {
        return Ok(None);
    }

    let window_id = resp.value32().and_then(|iter| iter.last());
    let Some(window_id) = window_id else {
        return Ok(None);
    };
    if window_id == 0 {
        return Ok(None);
    }

    let pid_resp = conn
        .get_property(false, window_id, pid_atom, AtomEnum::CARDINAL, 0, 1)?
        .reply()?;

    if pid_resp.value_len == 0 {
        return Ok(None);
    }

    let pid = pid_resp.value32().and_then(|mut iter| iter.next());
    Ok(pid)
}

pub fn get_active_app_pid_x11_persistent(
    conn: &impl Connection,
    screen_num: usize,
) -> Result<Option<u32>, ReplyError> {
    let setup = conn.setup();
    if screen_num >= setup.roots.len() {
        return Ok(None);
    }
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    if let Some(pid) = get_pid_via_active_window(conn, root)? {
        return Ok(Some(pid));
    }

    get_pid_via_stacking_list(conn, root)
}
