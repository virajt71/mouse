//! Actions Ring overlay — a cursor-centred radial menu rendered in the GUI.
//!
//! The engine daemon opens the ring by broadcasting an [`ActionsRingState`]
//! (with the resolved per-app layout + any folders) over the gRPC
//! `WatchActionsRing` stream. This view draws the eight slots, highlights the
//! one under the pointer, and returns the action id to execute when the user
//! clicks a slot. Clicking outside any slot, pressing Escape, or the display
//! timeout all dismiss it. A slot whose action id is `folder:<name>` opens that
//! named sub-ring (nested navigation); Escape from a sub-ring pops back.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use egui::{
    pos2, vec2, Align2, Color32, FontId, LayerId, Order, Painter, Pos2, Rect, Sense, Stroke,
};

use mouser_engine::config::actions_ring::ActionRingSlot;

/// Mirror of OpenLogi's overlay geometry.
const RADIUS: f32 = 122.0;
const SLOT_SIZE: f32 = 54.0;
/// Ring lifetime before auto-dismiss (mirrors OpenLogi's DISPLAY_LIFETIME).
const DISPLAY_LIFETIME: Duration = Duration::from_secs(6);

/// Live overlay state, owned by [`crate::app::MouserApp`].
pub struct RingState {
    /// Resolved slot → action id for the current level, in slot order.
    pub slots: Vec<(ActionRingSlot, String)>,
    /// Folder navigation stack (each entry is a sub-ring's slots).
    pub stack: Vec<Vec<(ActionRingSlot, String)>>,
    /// Folders available to this ring session, keyed by name.
    pub folders: HashMap<String, Vec<(ActionRingSlot, String)>>,
    /// Where the ring is centred (screen space).
    pub center: Pos2,
    pub opened_at: Instant,
}

/// Parse a serialised `ActionRingLayout` (`{"slots": {"Top": "copy", ...}}`)
/// into an ordered slot list. Unknown slot names are skipped.
fn parse_layout(json: &str) -> Vec<(ActionRingSlot, String)> {
    #[derive(serde::Deserialize)]
    struct Wire {
        slots: HashMap<String, String>,
    }
    let Ok(wire) = serde_json::from_str::<Wire>(json) else {
        return Vec::new();
    };
    ActionRingSlot::ALL
        .iter()
        .filter_map(|s| wire.slots.get(s.as_str()).map(|a| (*s, a.clone())))
        .collect()
}

/// Parse a serialised `HashMap<String, ActionRingLayout>` into ordered folders.
fn parse_folders(json: &str) -> HashMap<String, Vec<(ActionRingSlot, String)>> {
    #[derive(serde::Deserialize)]
    struct FolderWire {
        slots: HashMap<String, String>,
    }
    let Ok(wire) = serde_json::from_str::<HashMap<String, FolderWire>>(json) else {
        return HashMap::new();
    };
    wire.into_iter()
        .map(|(name, fw)| {
            let slots = ActionRingSlot::ALL
                .iter()
                .filter_map(|s| fw.slots.get(s.as_str()).map(|a| (*s, a.clone())))
                .collect();
            (name, slots)
        })
        .collect()
}

impl RingState {
    pub fn from_payload(layout_json: &str, folders_json: &str, center: Pos2) -> Self {
        Self {
            slots: parse_layout(layout_json),
            stack: Vec::new(),
            folders: parse_folders(folders_json),
            center,
            opened_at: Instant::now(),
        }
    }

    /// Open a named folder as the current level, pushing the previous level.
    fn open_folder(&mut self, name: &str) {
        if let Some(folder) = self.folders.get(name).cloned() {
            let prev = std::mem::take(&mut self.slots);
            self.stack.push(prev);
            self.slots = folder;
            self.opened_at = Instant::now(); // reset the dismiss timer per level
        }
    }

    /// Pop back to the parent level; returns false if already at the root.
    fn pop(&mut self) -> bool {
        if let Some(prev) = self.stack.pop() {
            self.slots = prev;
            self.opened_at = Instant::now();
            true
        } else {
            false
        }
    }
}

/// Build a human-readable label from an action id (e.g. "copy" → "Copy").
fn label_for(action_id: &str) -> String {
    let bare = action_id
        .strip_prefix("macro:")
        .or_else(|| action_id.strip_prefix("folder:"))
        .unwrap_or(action_id);
    let mut out = String::new();
    for (i, word) in bare.split('_').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(&chars.as_str().to_lowercase());
        }
    }
    if action_id.starts_with("folder:") {
        out.push_str(" ›"); // hint that this slot descends into a sub-ring
    }
    out
}

/// Screen-space rect for a slot centered at `center`.
fn slot_rect(slot: ActionRingSlot, center: Pos2) -> Rect {
    let (x, y) = slot.unit_offset();
    let cx = center.x + x * RADIUS;
    let cy = center.y + y * RADIUS;
    Rect::from_center_size(pos2(cx, cy), vec2(SLOT_SIZE, SLOT_SIZE))
}

/// Draw the ring and handle interaction.
///
/// Returns `Some(action_id)` when a concrete (non-folder) slot was activated —
/// the caller should execute it and close the ring. Returns `None` when the
/// ring should be dismissed (escape / backdrop click / timeout). Folder slots
/// are handled internally (they change the displayed level).
pub fn show_ring(ctx: &egui::Context, state: &mut RingState) -> Option<String> {
    if state.opened_at.elapsed() >= DISPLAY_LIFETIME {
        return None; // auto-dismiss
    }

    let center = state.center;
    let painter = ctx.layer_painter(LayerId::new(
        Order::Foreground,
        egui::Id::new("actions_ring"),
    ));

    // Full-screen transparent catcher: click outside a slot dismisses.
    let screen = ctx.screen_rect();
    let clicked_backdrop = egui::Area::new(egui::Id::new("actions_ring_catcher"))
        .order(Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let r = ui.allocate_rect(screen, Sense::click());
            r.clicked()
        })
        .inner;

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        // Escape from a sub-ring pops back; at the root it closes.
        if !state.pop() {
            return None;
        }
    }

    draw_backdrop(&painter, center);

    let pointer = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(center);
    let pointer_down = ctx.input(|i| i.pointer.primary_down());

    let mut hovered: Option<ActionRingSlot> = None;
    let mut activated: Option<String> = None;
    for (slot, action_id) in &state.slots {
        let rect = slot_rect(*slot, center);
        let hov = rect.contains(pointer);
        if hov {
            hovered = Some(*slot);
        }
        draw_slot(&painter, rect, label_for(action_id), hov);

        if hov && pointer_down {
            if action_id.starts_with("folder:") {
                let name = action_id.strip_prefix("folder:").unwrap_or("").to_string();
                state.open_folder(&name);
                ctx.request_repaint();
                return None; // stay open, just changed level
            }
            activated = Some(action_id.clone());
        }
    }

    if hovered.is_none() && clicked_backdrop {
        return None; // click-away
    }

    if let Some(action_id) = activated {
        return Some(action_id); // concrete action → execute + close
    }

    // Keep the overlay alive and animate.
    ctx.request_repaint();
    None
}

fn draw_backdrop(painter: &Painter, center: Pos2) {
    let ring_radius = RADIUS + 27.0;
    painter.circle_filled(center, ring_radius + 14.0, Color32::from_black_alpha(150));
    painter.circle_stroke(
        center,
        ring_radius,
        Stroke::new(2.0, Color32::from_rgb(0, 191, 165)),
    );
    painter.circle_filled(center, 26.0, Color32::from_rgb(0, 191, 165));
}

fn draw_slot(painter: &Painter, rect: Rect, label: String, hovered: bool) {
    let color = if hovered {
        Color32::from_rgb(0, 191, 165)
    } else {
        Color32::from_gray(40)
    };
    painter.circle_filled(rect.center(), rect.width() / 2.0, color);
    painter.circle_stroke(
        rect.center(),
        rect.width() / 2.0,
        Stroke::new(2.0, Color32::from_rgb(0, 191, 165)),
    );
    let text_color = if hovered {
        Color32::WHITE
    } else {
        Color32::LIGHT_GRAY
    };
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(13.0),
        text_color,
    );
}

/// Geometry self-check (used by tests; cheap, no GUI needed).
#[allow(dead_code)]
pub fn slot_unit_offsets() -> [(ActionRingSlot, (f32, f32)); 8] {
    ActionRingSlot::ALL.map(|s| (s, s.unit_offset()))
}
