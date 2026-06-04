use std::sync::atomic::{AtomicBool, Ordering};
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn handle_sig(_sig: std::os::raw::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

pub fn setup_signal_handlers() {
    unsafe {
        let sa = SigAction::new(SigHandler::Handler(handle_sig), SaFlags::empty(), SigSet::empty());
        let _ = sigaction(Signal::SIGINT, &sa);
        let _ = sigaction(Signal::SIGTERM, &sa);
        let _ = sigaction(Signal::SIGHUP, &sa);
    }
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

#[allow(dead_code)]
pub fn stop() {
    RUNNING.store(false, Ordering::SeqCst);
}
