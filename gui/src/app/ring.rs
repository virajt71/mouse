use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, Stroke};
use mouser_engine::config::{Profile, RingBubble, RingLayout};

// ── State ──

#[derive(Clone, Debug)]
pub struct RingState {
    pub open: bool,
    pub position: egui::Pos2,
    pub layout: Option<RingLayout>,
    pub hovered_index: Option<usize>,
    pub held_index: Option<usize>,
    pub folder_open_index: Option<usize>,
    pub adjustment_value: f32,
    pub adjustment_held: bool,
    pub opacity: f32,
    pub scale: f32,
    pub last_active_time: f32,
    pub closing: bool,
}

impl Default for RingState {
    fn default() -> Self {
        Self {
            open: false,
            position: pos2(0.0, 0.0),
            layout: None,
            hovered_index: None,
            held_index: None,
            folder_open_index: None,
            adjustment_value: 0.5,
            adjustment_held: false,
            opacity: 0.0,
            scale: 0.0,
            last_active_time: 0.0,
            closing: false,
        }
    }
}

/// Resets ring state to closed defaults.
impl RingState {
    pub fn reset(&mut self) {
        self.open = false;
        self.position = pos2(0.0, 0.0);
        self.layout = None;
        self.adjustment_value = 0.5;
        self.opacity = 0.0;
        self.scale = 0.0;
        self.hovered_index = None;
        self.held_index = None;
        self.folder_open_index = None;
        self.adjustment_held = false;
        self.last_active_time = 0.0;
        self.closing = false;
    }

    /// Begins a fade-out; draw_ring animates opacity to 0 then resets.
    pub fn begin_close(&mut self) {
        self.closing = true;
    }
}

// ── Constants ──

pub const RING_RADIUS: f32 = 90.0;
pub const BUBBLE_RADIUS: f32 = 32.0;
pub const BUBBLE_FONT_SIZE: f32 = 11.0;
pub const RING_BACKGROUND_RADIUS: f32 = RING_RADIUS + BUBBLE_RADIUS + 12.0;
const CLOSE_SIZE: f32 = 20.0;
const CLOSE_OFFSET_X: f32 = 0.72;
const CLOSE_OFFSET_Y: f32 = 0.72;
const TIMEOUT_SECONDS: f32 = 2.0;

// ── Action sender trait ──
// Allows ring.rs to fire actions back to the engine.
pub trait ActionSender {
    fn send(&mut self, action_id: &str);
}


/// Bridge from ring overlay to engine action execution.
pub struct EngineActionSender<'a> {
    pub engine: &'a mouser_engine::client::EngineClient,
}

impl<'a> ActionSender for EngineActionSender<'a> {
    fn send(&mut self, action_id: &str) {
        if action_id == "close_ring" {
            return;
        }
        let _ = self.engine.execute_engine_action(action_id);
    }
}

// ── Helpers: layout & hit-test ──

pub fn get_ring_layout(profile: &Profile) -> Option<&RingLayout> {
    profile.ring_layout.as_ref()
}

pub fn get_primary_bubbles(layout: Option<&RingLayout>) -> Vec<RingBubble> {
    if let Some(layout) = layout {
        if !layout.primary.is_empty() {
            return layout.primary.clone();
        }
    }
    default_bubbles()
}

pub fn get_folder_bubbles(layout: Option<&RingLayout>, folder_index: usize) -> Vec<RingBubble> {
    if let Some(layout) = layout {
        if folder_index < layout.folders.len() {
            let folder = &layout.folders[folder_index];
            if !folder.bubbles.is_empty() {
                return folder.bubbles.clone();
            }
        }
    }
    default_folder_bubbles()
}

pub fn compute_bubble_positions(count: usize, center: egui::Pos2, radius: f32) -> Vec<egui::Pos2> {
    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let angle = (2.0 * std::f32::consts::PI * i as f32 / count as f32) - std::f32::consts::PI / 2.0;
        let pos = pos2(center.x + angle.cos() * radius, center.y + angle.sin() * radius);
        positions.push(pos);
    }
    positions
}

pub fn hit_test_bubbles(positions: &[egui::Pos2], cursor: egui::Pos2, bubble_radius: f32) -> Option<usize> {
    for (i, &pos) in positions.iter().enumerate() {
        if cursor.distance(pos) <= bubble_radius {
            return Some(i);
        }
    }
    None
}

pub fn clamp_ring_to_screen(center: egui::Pos2, screen_rect: Rect, margin: f32) -> egui::Pos2 {
    pos2(
        center.x.clamp(screen_rect.min.x + margin, screen_rect.max.x - margin),
        center.y.clamp(screen_rect.min.y + margin, screen_rect.max.y - margin),
    )
}

// ── Bubble data ──

fn default_bubbles() -> Vec<RingBubble> {
    vec![
        RingBubble { label: "Copy".into(), action_id: "copy".into(), icon_name: "edit_copy".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "copy".into() }, adjustment_range: None },
        RingBubble { label: "Paste".into(), action_id: "paste".into(), icon_name: "edit_paste".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "paste".into() }, adjustment_range: None },
        RingBubble { label: "Undo".into(), action_id: "undo".into(), icon_name: "undo".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "undo".into() }, adjustment_range: None },
        RingBubble { label: "Redo".into(), action_id: "redo".into(), icon_name: "redo".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "redo".into() }, adjustment_range: None },
        RingBubble { label: "Volume ↑".into(), action_id: "volume_up".into(), icon_name: "volume_up".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "volume_up".into() }, adjustment_range: Some(mouser_engine::config::RingAdjustmentRange { min: 0, max: 100, step: 1, default: Some(50) }) },
        RingBubble { label: "Volume ↓".into(), action_id: "volume_down".into(), icon_name: "volume_down".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "volume_down".into() }, adjustment_range: Some(mouser_engine::config::RingAdjustmentRange { min: 0, max: 100, step: 1, default: Some(50) }) },
        RingBubble { label: "Zoom In".into(), action_id: "zoom_in".into(), icon_name: "zoom_in".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "zoom_in".into() }, adjustment_range: None },
        RingBubble { label: "Zoom Out".into(), action_id: "zoom_out".into(), icon_name: "zoom_out".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "zoom_out".into() }, adjustment_range: None },
    ]
}

fn default_folder_bubbles() -> Vec<RingBubble> {
    vec![
        RingBubble { label: "Cut".into(), action_id: "cut".into(), icon_name: "edit_cut".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "cut".into() }, adjustment_range: None },
        RingBubble { label: "Select All".into(), action_id: "select_all".into(), icon_name: "select_all".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "select_all".into() }, adjustment_range: None },
        RingBubble { label: "Find".into(), action_id: "find".into(), icon_name: "find".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "find".into() }, adjustment_range: None },
        RingBubble { label: "Search".into(), action_id: "search".into(), icon_name: "search".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "search".into() }, adjustment_range: None },
        RingBubble { label: "Save".into(), action_id: "save".into(), icon_name: "save".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "save".into() }, adjustment_range: None },
        RingBubble { label: "Print".into(), action_id: "print".into(), icon_name: "print".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "print".into() }, adjustment_range: None },
        RingBubble { label: "Settings".into(), action_id: "settings".into(), icon_name: "settings".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "settings".into() }, adjustment_range: None },
        RingBubble { label: "Help".into(), action_id: "help".into(), icon_name: "help".into(), kind: mouser_engine::config::RingBubbleKind::Action { action_id: "help".into() }, adjustment_range: None },
    ]
}

// ── Bubble display helpers ──

struct DisplayBubbles {
    bubbles: Vec<RingBubble>,
    count: usize,
}

fn compute_display_bubbles(layout: Option<&RingLayout>, folder_open_index: Option<usize>) -> DisplayBubbles {
    let bubbles = match folder_open_index {
        Some(idx) => get_folder_bubbles(layout, idx),
        None => get_primary_bubbles(layout),
    };
    DisplayBubbles {
        count: bubbles.len().min(8),
        bubbles,
    }
}

pub fn is_numeric_action_id(action_id: &str) -> bool {
    matches!(
        action_id,
        "volume_up" | "volume_down" | "zoom_in" | "zoom_out"
    )
}

pub fn open_ring(state: &mut RingState, position: egui::Pos2, layout: Option<RingLayout>) {
    state.position = position;
    state.layout = layout;
    state.open = true;
    state.opacity = 0.0;
    state.scale = 0.0;
    state.hovered_index = None;
    state.held_index = None;
    state.folder_open_index = None;
    state.adjustment_value = 0.5;
    state.adjustment_held = false;
    state.last_active_time = 0.0;
    state.closing = false;
}

/// Updates ring state from pointer input.
pub fn handle_ring_input(
    state: &mut RingState,
    ctx: &egui::Context,
    sender: &mut dyn ActionSender,
) {
    if !state.open {
        return;
    }

    let now = ctx.input(|i| i.time) as f32;

    // Timeout: close ring after inactivity. last_active_time == 0.0 is the
    // "just opened" sentinel set by open_ring(); skip it so frame 1 doesn't
    // instantly reset (prevents the ring from never rendering).
    if state.last_active_time > 0.0 && now - state.last_active_time > TIMEOUT_SECONDS {
        state.begin_close();
        return;
    }

    let pointer_pos = ctx.input(|i| i.pointer.hover_pos());

    let Some(pos) = pointer_pos else {
        return;
    };

    let screen_rect = ctx.screen_rect();
    let scale = state.scale.max(0.001); // avoid div-by-zero in clamped calc
    let bg_radius = RING_BACKGROUND_RADIUS * scale;
    let clamped_center = clamp_ring_to_screen(state.position, screen_rect, bg_radius);

    // Close button hit
    let close_rect = Rect::from_center_size(
        clamped_center + vec2(
            bg_radius * CLOSE_OFFSET_X,
            -(bg_radius * CLOSE_OFFSET_Y),
        ),
        vec2(CLOSE_SIZE * scale, CLOSE_SIZE * scale),
    );
    if close_rect.contains(pos) {
        if ctx.input(|i| i.pointer.any_click()) {
            state.begin_close();
            let _ = sender.send("close_ring");
            return;
        }
    }

    // Bubble hit test
    let DisplayBubbles { bubbles, count } =
        compute_display_bubbles(state.layout.as_ref(), state.folder_open_index);
    let positions = compute_bubble_positions(count, clamped_center, RING_RADIUS * scale);

    if let Some(idx) = hit_test_bubbles(&positions, pos, BUBBLE_RADIUS * scale) {
        state.hovered_index = Some(idx);
        let is_folder = matches!(bubbles[idx].kind, mouser_engine::config::RingBubbleKind::Folder { .. });

        // Click to fire action or open folder
        if ctx.input(|i| i.pointer.primary_clicked()) {
            if is_folder {
                // Find the folder's index in the layout
                if let Some(layout) = &state.layout {
                    if let Some(folder_idx) = layout.folders.iter().position(|f| {
                        // Match by first bubble's action_id as heuristic
                        f.bubbles.first().map_or(false, |b| b.action_id == bubbles[idx].action_id)
                    }) {
                        state.folder_open_index = Some(folder_idx);
                        state.hovered_index = None;
                        state.held_index = None;
                        state.adjustment_value = 0.5;
                        state.adjustment_held = false;
                        state.last_active_time = now;
                        return;
                    }
                }
            } else {
                // Fire the action
                sender.send(&bubbles[idx].action_id);
                state.begin_close();
                return;
            }
        }

        // Click-and-hold for adjustment
        if ctx.input(|i| i.pointer.primary_down()) && !is_folder {
            state.held_index = Some(idx);
            state.adjustment_held = true;
            state.adjustment_value = 0.5;
        }
    } else {
        // If not hovering a bubble and primary is down, close held state
        if !ctx.input(|i| i.pointer.primary_down()) {
            state.held_index = None;
            state.adjustment_held = false;
        }
    }

    // Scroll adjustment while holding
    if state.adjustment_held {
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            state.adjustment_value = (state.adjustment_value + scroll_delta * 0.001).clamp(0.0, 1.0);
            state.last_active_time = now;
        }
    }

    state.last_active_time = ctx.input(|i| i.time) as f32;
}

// ── Main draw ──

/// Draws the ring overlay if open. Must be called each frame while open.
pub fn draw_ring(
    state: &mut RingState,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
) {
    if !state.open {
        return;
    }

    let _ = ctx.input(|i| i.time) as f32;

    // Fade-in / fade-out
    if state.closing {
        state.opacity = (state.opacity - 0.12).max(0.0);
        if state.opacity <= 0.0 {
            state.reset();
            return;
        }
    } else {
        state.opacity = (state.opacity + 0.05).min(1.0);
        state.scale = (state.scale + 0.08).min(1.0);
    }
    state.last_active_time = ctx.input(|i| i.time) as f32;

    let screen_rect = ctx.screen_rect();
    let center = pos2(state.position.x, state.position.y);
    let bg_rect = Rect::from_center_size(center, vec2(state.scale * 200.0, state.scale * 200.0));

    // cull off-screen
    if !bg_rect.intersects(screen_rect) {
        return;
    }

    let painter = ui.painter();
    let opacity = state.opacity;
    let scale = state.scale;

    // --- High-contrast canvas background ---
    draw_background_canvas(painter, center, RING_BACKGROUND_RADIUS * scale, opacity);

    // --- Main ring track ---
    draw_ring_track(painter, center, RING_BACKGROUND_RADIUS * scale, opacity);

    // --- Bubble ring content ---
    let DisplayBubbles { bubbles, count } =
        compute_display_bubbles(state.layout.as_ref(), state.folder_open_index);
    let positions = compute_bubble_positions(count, center, RING_RADIUS * scale);
    let bubbles_len = bubbles.len();

    for (i, bubble) in bubbles.iter().take(count).enumerate() {
        let pos = positions[i];
        let is_hovered = state.hovered_index == Some(i);
        let is_held = state.held_index == Some(i);

        draw_bubble(
            painter,
            pos,
            BUBBLE_RADIUS * scale,
            scale,
            is_hovered,
            is_held,
            bubble,
            opacity,
        );
    }

    // --- Close button ---
    draw_close_button(painter, center, RING_BACKGROUND_RADIUS * scale, scale, opacity);

    // --- Adjustment bar (if held) ---
    if let Some(held_idx) = state.held_index {
        if held_idx < bubbles_len {
            let bub = &bubbles[held_idx];
            if bub.adjustment_range.is_some() || is_numeric_action_id(&bub.action_id) {
                draw_adjustment_bar(
                    painter,
                    center,
                    state.adjustment_value,
                    opacity,
                    scale,
                );
            }
        }
    }
}

// ── Drawing helpers ──

fn draw_background_canvas(painter: &egui::Painter, center: egui::Pos2, radius: f32, opacity: f32) {
    let color = Color32::from_rgba_unmultiplied(30, 30, 35, (120.0 * opacity) as u8);
    painter.circle_filled(center, radius, color);
}

fn draw_ring_track(painter: &egui::Painter, center: egui::Pos2, radius: f32, opacity: f32) {
    let color = Color32::from_rgba_unmultiplied(200, 200, 210, (40.0 * opacity) as u8);
    painter.circle_stroke(center, radius * 0.8, Stroke::new(2.0, color));
}

fn draw_bubble(
    painter: &egui::Painter,
    center: egui::Pos2,
    bubble_r: f32,
    scale: f32,
    is_hovered: bool,
    is_held: bool,
    bubble: &RingBubble,
    opacity: f32,
) {
    let bg_color = if is_held {
        Color32::from_rgba_unmultiplied(0, 210, 190, (220.0 * opacity) as u8)
    } else if is_hovered {
        Color32::from_rgba_unmultiplied(80, 80, 90, (220.0 * opacity) as u8)
    } else {
        Color32::from_rgba_unmultiplied(60, 60, 70, (200.0 * opacity) as u8)
    };

    circle_alpha(painter, center, bubble_r, bg_color);

    // Draw icon (simplified - just use label text)
    let text_color = if is_hovered {
        Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * opacity) as u8)
    } else {
        Color32::from_rgba_unmultiplied(200, 200, 210, (255.0 * opacity) as u8)
    };

    let font = egui::FontId::proportional(BUBBLE_FONT_SIZE * scale);
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        &bubble.label,
        font,
        text_color,
    );
}

fn draw_close_button(painter: &egui::Painter, center: egui::Pos2, bg_radius: f32, scale: f32, opacity: f32) {
    let offset = bg_radius * 0.72;
    let pos = center + vec2(offset, -offset);
    let color = Color32::from_rgba_unmultiplied(200, 50, 50, (180.0 * opacity) as u8);
    circle_alpha(painter, pos, CLOSE_SIZE * 0.5 * scale, color);

    let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * opacity) as u8);
    let font = egui::FontId::proportional(10.0 * scale);
    painter.text(pos, egui::Align2::CENTER_CENTER, "✕", font, text_color);
}

fn circle_alpha(painter: &egui::Painter, center: egui::Pos2, radius: f32, color: Color32) {
    painter.circle_filled(center, radius, color);
}

pub fn draw_adjustment_bar(
    painter: &egui::Painter,
    center: egui::Pos2,
    value: f32,
    opacity: f32,
    scale: f32,
) {
    let bar_width = 160.0 * scale.max(0.5);
    let bar_height = 6.0 * scale.max(0.5);
    let bar_center_y = center.y + RING_RADIUS * scale.max(0.5) + 40.0 * scale.max(0.5);
    let bar_rect = Rect::from_center_size(pos2(center.x, bar_center_y), vec2(bar_width, bar_height));

    // Track
    let track_color = Color32::from_rgba_unmultiplied(60, 60, 70, (150.0 * opacity) as u8);
    painter.rect_filled(bar_rect, bar_height * 0.5, track_color);

    // Filled
    let fill_width = bar_width * value.clamp(0.0, 1.0);
    let fill_rect = Rect::from_min_max(bar_rect.min, pos2(bar_rect.min.x + fill_width, bar_rect.max.y));
    let fill_color = Color32::from_rgba_unmultiplied(0, 210, 190, (200.0 * opacity) as u8);
    painter.rect_filled(fill_rect, bar_height * 0.5, fill_color);

    // Thumb
    let thumb_x = bar_rect.min.x + fill_width;
    let thumb_r = 8.0 * scale.max(0.5);
    let thumb_color = Color32::from_rgba_unmultiplied(0, 220, 200, (255.0 * opacity) as u8);
    circle_alpha(painter, pos2(thumb_x, bar_center_y), thumb_r, thumb_color);

    // Label
    let pct = format!("{:.0}%", value * 100.0);
    let font = egui::FontId::proportional(10.0 * scale.max(0.5));
    let text_color = Color32::from_rgba_unmultiplied(200, 200, 210, (200.0 * opacity) as u8);
    painter.text(
        pos2(center.x, bar_center_y - bar_height - 8.0 * scale.max(0.5)),
        egui::Align2::CENTER_CENTER,
        pct,
        font,
        text_color,
    );
}
