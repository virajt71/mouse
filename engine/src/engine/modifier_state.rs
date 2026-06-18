use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared modifier key state, updated by KeyboardHook threads.
/// All keyboard hooks share the same ModifierState (OR semantics: any
/// keyboard holding the key counts as the modifier being active).
pub struct ModifierState {
    pub ctrl: AtomicBool,
    pub alt: AtomicBool,
    pub shift: AtomicBool,
    pub meta: AtomicBool,
}

impl ModifierState {
    pub fn new() -> Arc<Self> {
        Arc::new(ModifierState {
            ctrl: AtomicBool::new(false),
            alt: AtomicBool::new(false),
            shift: AtomicBool::new(false),
            meta: AtomicBool::new(false),
        })
    }

    /// Returns true if the configured hold-key constraint is satisfied.
    ///
    /// # Hold-key strings
    /// - `"none"` — always satisfied
    /// - `"ctrl"` — Ctrl must be held
    /// - `"alt"` — Alt must be held
    /// - `"shift"` — Shift must be held
    /// - `"meta"` / `"super"` — Meta/Super must be held
    pub fn is_satisfied(&self, hold_key: &str) -> bool {
        match hold_key {
            "none" | "" => true,
            "ctrl" => self.ctrl.load(Ordering::SeqCst),
            "alt" => self.alt.load(Ordering::SeqCst),
            "shift" => self.shift.load(Ordering::SeqCst),
            "meta" | "super" => self.meta.load(Ordering::SeqCst),
            _ => {
                log::warn!("[ModifierState] Unknown hold_key value: '{}' — treating as satisfied", hold_key);
                true
            }
        }
    }
}
