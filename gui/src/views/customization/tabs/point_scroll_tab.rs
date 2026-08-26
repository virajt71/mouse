use crate::theme;
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, Stroke};
use mouser_engine::config::Config;
use mouser_engine::client::EngineClient as Engine;

pub fn show_point_scroll_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    mouse_texture: &egui::TextureHandle,
) {
    let rect = ui.max_rect();
    let center = rect.center() + vec2(30.0, -10.0);

    // 1. Draw Clean Mouse Image
    let img_size = vec2(600.0, 600.0);
    let mouse_rect = Rect::from_center_size(center, img_size);
    ui.put(
        mouse_rect,
        egui::Image::new(mouse_texture).fit_to_exact_size(img_size),
    );

    let mut settings_dirty = false;

    // Define card and dot positions
    let scroll_wheel_dot = center + vec2(80.0, -190.0);
    let scroll_wheel_card = center + vec2(270.0, -140.0);

    let thumb_wheel_dot = center + vec2(0.0, -23.0);
    let thumb_wheel_card = center + vec2(-180.0, 30.0);

    let pointer_dot = center + vec2(110.0, 40.0);
    let pointer_card = center + vec2(240.0, 40.0);

    // State IDs for popup open/close (stored in egui ctx temp data)
    let scroll_popup_id = ui.id().with("scroll_popup_open");
    let thumb_popup_id = ui.id().with("thumb_popup_open");
    let ptr_popup_id = ui.id().with("ptr_popup_open");

    let scroll_open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(scroll_popup_id))
        .unwrap_or(false);
    let thumb_open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(thumb_popup_id))
        .unwrap_or(false);
    let ptr_open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(ptr_popup_id))
        .unwrap_or(false);

    // Draw connecting lines and base dot rings
    for (dot_pos, card_pos) in &[
        (scroll_wheel_dot, scroll_wheel_card),
        (thumb_wheel_dot, thumb_wheel_card),
        (pointer_dot, pointer_card),
    ] {
        let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 100));
        ui.painter().line_segment([*dot_pos, *card_pos], stroke);
        ui.painter()
            .circle_stroke(*dot_pos, 8.0, Stroke::new(1.2, Color32::WHITE));
    }

    // Get current settings
    let invert_v = config.settings.invert_vscroll;
    let invert_h = config.settings.invert_hscroll;
    let smart_shift = config.settings.smart_shift_enabled;
    let ss_threshold = config.settings.smart_shift_threshold;
    let dpi = config.settings.dpi;
    let smart_shift_mode = config.settings.smart_shift_mode.clone();

    let pointer_speed_pct = ((dpi.clamp(200, 4000) - 200) as f32 * 100.0 / 3800.0).round() as i32;
    let thumb_speed_pct =
        ((11 - config.settings.hscroll_threshold.clamp(1, 10)) * 10).clamp(10, 100);

    // ── Card 1: Scroll wheel ─────────────────────────────────────────────────
    let scroll_dir_str = if invert_v {
        "Scroll direction: Inverted"
    } else {
        "Scroll direction: Standard"
    };
    let smart_shift_str = if smart_shift {
        "SmartShift: On"
    } else {
        "SmartShift: Off"
    };
    let scroll_rows = &[scroll_dir_str, "Smooth scrolling: Off", smart_shift_str];
    let scroll_card_rect = Rect::from_center_size(scroll_wheel_card, vec2(160.0, 72.0));
    let scroll_res = draw_ps_card(
        ui,
        scroll_wheel_card,
        160.0,
        72.0,
        "Scroll wheel",
        scroll_rows,
        scroll_open,
    );
    let scroll_card_hovered = scroll_res.hovered();
    if scroll_res.clicked() {
        ui.ctx()
            .data_mut(|d| d.insert_temp(scroll_popup_id, !scroll_open));
        ui.ctx().data_mut(|d| d.insert_temp(thumb_popup_id, false));
        ui.ctx().data_mut(|d| d.insert_temp(ptr_popup_id, false));
    }

    // ── Card 2: Thumb wheel ──────────────────────────────────────────────────
    let thumb_dir_str = if invert_h {
        "Scroll direction: Inverted"
    } else {
        "Scroll direction: Default"
    };
    let thumb_speed_str = format!("Speed: {}%", thumb_speed_pct);
    let thumb_rows = &[thumb_speed_str.as_str(), thumb_dir_str];
    let thumb_card_rect = Rect::from_center_size(thumb_wheel_card, vec2(160.0, 58.0));
    let thumb_res = draw_ps_card(
        ui,
        thumb_wheel_card,
        160.0,
        58.0,
        "Thumb wheel",
        thumb_rows,
        thumb_open,
    );
    let thumb_card_hovered = thumb_res.hovered();
    if thumb_res.clicked() {
        ui.ctx()
            .data_mut(|d| d.insert_temp(thumb_popup_id, !thumb_open));
        ui.ctx().data_mut(|d| d.insert_temp(scroll_popup_id, false));
        ui.ctx().data_mut(|d| d.insert_temp(ptr_popup_id, false));
    }

    // ── Card 3: Pointer speed ────────────────────────────────────────────────
    let pointer_speed_str = format!("Speed: {}%", pointer_speed_pct);
    let pointer_rows = &[pointer_speed_str.as_str()];
    let pointer_card_rect = Rect::from_center_size(pointer_card, vec2(145.0, 44.0));
    let pointer_res = draw_ps_card(
        ui,
        pointer_card,
        145.0,
        44.0,
        "Pointer speed",
        pointer_rows,
        ptr_open,
    );
    let pointer_card_hovered = pointer_res.hovered();
    if pointer_res.clicked() {
        ui.ctx()
            .data_mut(|d| d.insert_temp(ptr_popup_id, !ptr_open));
        ui.ctx().data_mut(|d| d.insert_temp(scroll_popup_id, false));
        ui.ctx().data_mut(|d| d.insert_temp(thumb_popup_id, false));
    }

    // ── Animated accent dots ─────────────────────────────────────────────────
    for (dot_pos, is_active, id_str) in &[
        (
            scroll_wheel_dot,
            scroll_card_hovered || scroll_open,
            "scroll_dot",
        ),
        (
            thumb_wheel_dot,
            thumb_card_hovered || thumb_open,
            "thumb_dot",
        ),
        (pointer_dot, pointer_card_hovered || ptr_open, "pointer_dot"),
    ] {
        let t = ui
            .ctx()
            .animate_bool(ui.id().with(id_str).with("dot_glow"), *is_active);
        let dot_color = theme::lerp_color(
            theme::accent_dim_color(ui.ctx()),
            theme::accent_color(ui.ctx()),
            t,
        );
        ui.painter().circle_filled(*dot_pos, 5.0, dot_color);
        if *is_active {
            ui.painter().circle_stroke(
                *dot_pos,
                14.0,
                Stroke::new(2.0, theme::accent_color(ui.ctx())),
            );
        }
    }

    // ── Render Popups ────────────────────────────────────────────────────────

    if scroll_open {
        // Height grows to accommodate SmartShift threshold slider when SS is enabled
        let popup_h = if smart_shift { 400.0_f32 } else { 340.0_f32 };
        let popup_rect = compute_popup_rect(scroll_card_rect, 240.0, popup_h, rect, center);
        let (close, dirty, new_invert_v, new_smart_shift, new_mode, new_ss_thresh) =
            draw_scroll_wheel_popup(
                ui,
                popup_rect,
                scroll_card_rect,
                invert_v,
                smart_shift,
                &smart_shift_mode,
                ss_threshold,
            );
        if dirty {
            config.settings.invert_vscroll = new_invert_v;
            config.settings.smart_shift_enabled = new_smart_shift;
            config.settings.smart_shift_mode = new_mode;
            config.settings.smart_shift_threshold = new_ss_thresh;
            settings_dirty = true;
        }
        if close {
            ui.ctx().data_mut(|d| d.insert_temp(scroll_popup_id, false));
        }
    }

    if thumb_open {
        let popup_rect = compute_popup_rect(thumb_card_rect, 230.0, 200.0, rect, center);
        let (close, dirty, new_speed_pct, new_invert_h) =
            draw_thumb_wheel_popup(ui, popup_rect, thumb_card_rect, thumb_speed_pct, invert_h);
        if dirty {
            if let Some(spct) = new_speed_pct {
                let clamped = spct.clamp(10.0, 100.0);
                let rounded = ((clamped / 10.0).round() * 10.0) as i32;
                let threshold = (11 - (rounded / 10)).clamp(1, 10);
                config.settings.hscroll_threshold = threshold;
            }
            if let Some(inv) = new_invert_h {
                config.settings.invert_hscroll = inv;
            }
            settings_dirty = true;
        }
        if close {
            ui.ctx().data_mut(|d| d.insert_temp(thumb_popup_id, false));
        }
    }

    if ptr_open {
        let popup_rect = compute_popup_rect(pointer_card_rect, 230.0, 100.0, rect, center);
        let (close, dirty, new_dpi) =
            draw_pointer_speed_popup(ui, popup_rect, pointer_card_rect, dpi, pointer_speed_pct);
        if dirty {
            config.settings.dpi = new_dpi;
            settings_dirty = true;
        }
        if close {
            ui.ctx().data_mut(|d| d.insert_temp(ptr_popup_id, false));
        }
    }

    if settings_dirty {
        engine.update_global_settings(
            config.settings.dpi as u32,
            config.settings.smart_shift_mode.clone(),
            config.settings.smart_shift_enabled,
            config.settings.smart_shift_threshold as u8,
            config.settings.invert_hscroll,
            config.settings.invert_vscroll,
            config.settings.gesture_threshold,
            config.settings.gesture_deadzone,
            config.settings.accent_color.clone(),
            config.settings.hscroll_threshold,
        );
    }
}

// ── Popup rect helper ────────────────────────────────────────────────────────

fn compute_popup_rect(
    card_rect: Rect,
    popup_w: f32,
    popup_h: f32,
    canvas: Rect,
    center: egui::Pos2,
) -> Rect {
    let mut r = if card_rect.center().x > center.x {
        // card is on right → popup to the left
        Rect::from_min_size(
            pos2(card_rect.min.x - popup_w - 10.0, card_rect.min.y - 20.0),
            vec2(popup_w, popup_h),
        )
    } else {
        // card is on left → popup to the right
        Rect::from_min_size(
            pos2(card_rect.max.x + 10.0, card_rect.min.y - 20.0),
            vec2(popup_w, popup_h),
        )
    };
    // Clamp inside canvas
    if r.min.x < canvas.min.x + 10.0 {
        r = r.translate(vec2(canvas.min.x + 10.0 - r.min.x, 0.0));
    }
    if r.max.x > canvas.max.x - 10.0 {
        r = r.translate(vec2((canvas.max.x - 10.0) - r.max.x, 0.0));
    }
    if r.min.y < canvas.min.y + 10.0 {
        r = r.translate(vec2(0.0, canvas.min.y + 10.0 - r.min.y));
    }
    if r.max.y > canvas.max.y - 10.0 {
        r = r.translate(vec2(0.0, (canvas.max.y - 10.0) - r.max.y));
    }
    r
}

// ── Popup shell ──────────────────────────────────────────────────────────────

/// Draws the popup chrome (shadow, bg, border, corners).
/// Returns true if the user clicked outside (= should close).
fn draw_popup_shell(ui: &mut egui::Ui, rect: Rect, card_rect: Rect) -> bool {
    let _ = ui.allocate_rect(rect, egui::Sense::click());

    // Drop shadow
    ui.painter().rect_filled(
        rect.expand2(vec2(2.0, 3.0)).translate(vec2(0.0, 2.0)),
        3.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );
    ui.painter()
        .rect_filled(rect, 2.0, theme::surface_color(ui.ctx()));
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, theme::border_color(ui.ctx())));
    theme::draw_tech_corners(ui.painter(), rect, theme::accent_color(ui.ctx()), 6.0);

    ui.input(|i| i.pointer.any_click())
        && !ui.rect_contains_pointer(rect)
        && !ui.rect_contains_pointer(card_rect)
}

// ── Widget helpers ───────────────────────────────────────────────────────────

fn draw_section_header(ui: &mut egui::Ui, label: &str) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(12.0))
                .strong()
                .color(theme::primary_text(ui.ctx())),
        );
    });
    ui.add_space(6.0);
}

fn draw_sub_header(ui: &mut egui::Ui, label: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(9.5))
                .strong()
                .color(theme::secondary_text(ui.ctx())),
        );
    });
    ui.add_space(4.0);
}

fn draw_divider(ui: &mut egui::Ui, x_min: f32, x_max: f32) {
    ui.add_space(10.0);
    let y = ui.cursor().top();
    ui.painter().line_segment(
        [pos2(x_min + 10.0, y), pos2(x_max - 10.0, y)],
        Stroke::new(1.0, theme::border_color(ui.ctx())),
    );
    ui.add_space(2.0);
}

fn draw_description(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(text)
                .font(egui::FontId::proportional(10.0))
                .color(theme::secondary_text(ui.ctx())),
        );
    });
    ui.add_space(4.0);
}

/// Radio bullet item — returns true if clicked.
fn draw_ps_radio_item(ui: &mut egui::Ui, label: &str, is_selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 26.0), egui::Sense::click());
    let is_hovered = response.hovered();

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        ui.painter()
            .rect_filled(rect, 0.0, theme::hover_color(ui.ctx()));
    }

    let bullet = pos2(rect.min.x + 14.0, rect.center().y);
    if is_selected {
        ui.painter()
            .circle_filled(bullet, 6.0, theme::accent_color(ui.ctx()));
        ui.painter()
            .circle_filled(bullet, 2.0, theme::surface_color(ui.ctx()));
    } else {
        ui.painter()
            .circle_filled(bullet, 6.0, Color32::from_gray(60));
    }

    let text_color = if is_selected {
        theme::accent_color(ui.ctx())
    } else if is_hovered {
        theme::primary_text(ui.ctx())
    } else {
        theme::secondary_text(ui.ctx())
    };

    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            label.to_string(),
            egui::FontId::proportional(11.0),
            text_color,
        ))
    });
    ui.painter().galley(
        pos2(rect.min.x + 28.0, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );

    response.clicked()
}

/// Toggle row (label + animated pill) — returns true if clicked.
fn draw_ps_toggle_row(ui: &mut egui::Ui, label: &str, is_on: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 28.0), egui::Sense::click());
    let is_hovered = response.hovered();

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        ui.painter()
            .rect_filled(rect, 0.0, theme::hover_color(ui.ctx()));
    }

    let text_color = theme::primary_text(ui.ctx());
    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            label.to_string(),
            egui::FontId::proportional(12.0),
            text_color,
        ))
    });
    ui.painter().galley(
        pos2(rect.min.x + 12.0, rect.center().y - galley.size().y / 2.0),
        galley,
        text_color,
    );

    // Animated pill
    let pill_w = 32.0;
    let pill_h = 16.0;
    let pill_rect = Rect::from_min_size(
        pos2(rect.max.x - pill_w - 12.0, rect.center().y - pill_h / 2.0),
        vec2(pill_w, pill_h),
    );
    let t = ui.ctx().animate_bool(response.id.with("toggle"), is_on);
    ui.painter().rect_filled(
        pill_rect,
        pill_h / 2.0,
        theme::lerp_color(Color32::from_gray(50), theme::accent_color(ui.ctx()), t),
    );
    let knob_x = pill_rect.min.x + pill_h / 2.0 + t * (pill_w - pill_h);
    ui.painter().circle_filled(
        pos2(knob_x, pill_rect.center().y),
        (pill_h / 2.0) - 2.0,
        Color32::WHITE,
    );

    response.clicked()
}

/// Custom styled slider. Returns new value in [range] if changed.
fn draw_ps_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
) -> Option<f32> {
    let accent = theme::accent_color(ui.ctx());
    let slider_id = ui.id().with(label).with("ps_slider");
    let stored_id = slider_id.with("live_val");
    let live_val: Option<f32> = ui.ctx().data(|d| d.get_temp(stored_id));
    let display_value = live_val.unwrap_or(value);

    // Header row: label + current value (accent)
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new(label)
                .font(egui::FontId::proportional(12.0))
                .strong()
                .color(theme::primary_text(ui.ctx())),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!("{:.0}%", display_value))
                    .font(egui::FontId::proportional(12.0))
                    .strong()
                    .color(accent),
            );
        });
    });
    ui.add_space(6.0);

    let slider_h = 4.0;
    let thumb_r = 8.0;
    let margin = 12.0 + thumb_r;

    // Allocate the full slider row as clickable+draggable immediately
    let (track_rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), thumb_r * 2.0 + 4.0),
        egui::Sense::click_and_drag(),
    );

    let full_range = *range.end() - *range.start();
    let track_start_x = track_rect.min.x + margin;
    let track_end_x = track_rect.max.x - margin;
    let track_w = track_end_x - track_start_x;
    let cy = track_rect.center().y;

    let t = ((display_value - *range.start()) / full_range).clamp(0.0, 1.0);
    let thumb_x = track_start_x + t * track_w;

    // Background track
    ui.painter().rect_filled(
        Rect::from_min_max(
            pos2(track_start_x, cy - slider_h / 2.0),
            pos2(track_end_x, cy + slider_h / 2.0),
        ),
        slider_h / 2.0,
        Color32::from_gray(50),
    );
    // Filled portion
    ui.painter().rect_filled(
        Rect::from_min_max(
            pos2(track_start_x, cy - slider_h / 2.0),
            pos2(thumb_x, cy + slider_h / 2.0),
        ),
        slider_h / 2.0,
        accent,
    );
    // Thumb
    ui.painter()
        .circle_filled(pos2(thumb_x, cy), thumb_r, accent);
    ui.painter()
        .circle_filled(pos2(thumb_x, cy), thumb_r - 3.0, Color32::from_gray(15));

    // Show cursor when hovering
    if response.hovered() || response.dragged() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
    }

    // Compute new value from pointer position
    if response.dragged() || response.clicked() {
        if let Some(ptr) = ui.input(|i| i.pointer.interact_pos()) {
            let new_t = ((ptr.x - track_start_x) / track_w).clamp(0.0, 1.0);
            let new_val = *range.start() + new_t * full_range;
            // Store continuous value in ctx to avoid snapping
            ui.ctx().data_mut(|d| d.insert_temp(stored_id, new_val));
            return Some(new_val);
        }
    }

    // Return the live value while dragging (prevents snapping back)
    if response.dragged() {
        return live_val;
    }
    // Clear stored val when released
    if response.drag_stopped() {
        ui.ctx().data_mut(|d| d.remove::<f32>(stored_id));
    }

    None
}

// ── Scroll Wheel Popup ───────────────────────────────────────────────────────

/// Returns: (should_close, dirty, new_invert_v, new_smart_shift, new_ss_mode, new_ss_threshold)
fn draw_scroll_wheel_popup(
    ui: &mut egui::Ui,
    popup_rect: Rect,
    card_rect: Rect,
    invert_v: bool,
    smart_shift: bool,
    smart_shift_mode: &str,
    ss_threshold: i32,
) -> (bool, bool, bool, bool, String, i32) {
    let should_close = draw_popup_shell(ui, popup_rect, card_rect);

    let mut new_invert_v = invert_v;
    let mut new_smart_shift = smart_shift;
    let mut new_mode = smart_shift_mode.to_string();
    let mut new_ss_thresh = ss_threshold;
    let mut dirty = false;

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(popup_rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    // No visible scrollbar — content fits within the fixed popup height
    egui::ScrollArea::vertical()
        .id_salt("scroll_wheel_popup_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut child, |ui| {

            // — Scroll direction —
            draw_section_header(ui, "Scroll direction");
            if draw_ps_radio_item(ui, "Inverted", invert_v) {
                new_invert_v = true;
                dirty = true;
            }
            if draw_ps_radio_item(ui, "Standard", !invert_v) {
                new_invert_v = false;
                dirty = true;
            }

            draw_divider(ui, popup_rect.min.x, popup_rect.max.x);

            // — Smooth scrolling (display-only; not supported by engine) —
            ui.add_space(4.0);
            let _ = draw_ps_toggle_row(ui, "Smooth scrolling", false);
            draw_description(ui, "With smooth scrolling, web pages glide across your screen smoothly making it easy to read and navigate them.");

            draw_divider(ui, popup_rect.min.x, popup_rect.max.x);

            // — SmartShift —
            ui.add_space(4.0);
            if draw_ps_toggle_row(ui, "SmartShift", smart_shift) {
                new_smart_shift = !smart_shift;
                dirty = true;
            }
            draw_description(ui, "Automatically switches the scroll wheel from line-by-line scrolling to hyper-fast scrolling when you scroll faster.");

            if smart_shift {
                // — SmartShift sensitivity slider —
                ui.add_space(6.0);
                let thresh_pct = ((ss_threshold.clamp(1, 50) - 1) as f32 / 49.0 * 100.0).round() as f32;
                if let Some(new_pct) = draw_ps_slider(ui, "Sensitivity", thresh_pct, 0.0..=100.0) {
                    new_ss_thresh = (new_pct / 100.0 * 49.0 + 1.0).round() as i32;
                    dirty = true;
                }

                // — Scroll mode —
                draw_sub_header(ui, "SCROLL MODE");
                let is_freespin = smart_shift_mode == "freespin";
                if draw_ps_radio_item(ui, "Free spin", is_freespin) {
                    new_mode = "freespin".to_string();
                    dirty = true;
                }
                if draw_ps_radio_item(ui, "Ratchet", !is_freespin) {
                    new_mode = "ratchet".to_string();
                    dirty = true;
                }
            }

            ui.add_space(8.0);
        });

    (
        should_close,
        dirty,
        new_invert_v,
        new_smart_shift,
        new_mode,
        new_ss_thresh,
    )
}

// ── Thumb Wheel Popup ────────────────────────────────────────────────────────

/// Returns: (should_close, dirty, new_speed_pct, new_invert_h)
fn draw_thumb_wheel_popup(
    ui: &mut egui::Ui,
    popup_rect: Rect,
    card_rect: Rect,
    speed_pct: i32,
    invert_h: bool,
) -> (bool, bool, Option<f32>, Option<bool>) {
    let should_close = draw_popup_shell(ui, popup_rect, card_rect);

    let mut new_speed: Option<f32> = None;
    let mut new_invert: Option<bool> = None;
    let mut dirty = false;

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(popup_rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::ScrollArea::vertical()
        .id_salt("thumb_wheel_popup_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut child, |ui| {
            ui.add_space(4.0);

            // — Speed slider —
            if let Some(val) =
                draw_ps_slider(ui, "Thumb wheel speed", speed_pct as f32, 10.0..=100.0)
            {
                new_speed = Some(val);
                dirty = true;
            }

            draw_divider(ui, popup_rect.min.x, popup_rect.max.x);

            // — Direction —
            draw_section_header(ui, "Thumb wheel direction");
            if draw_ps_radio_item(ui, "Default", !invert_h) {
                new_invert = Some(false);
                dirty = true;
            }
            if draw_ps_radio_item(ui, "Inverted", invert_h) {
                new_invert = Some(true);
                dirty = true;
            }

            ui.add_space(8.0);
        });

    (should_close, dirty, new_speed, new_invert)
}

// ── Pointer Speed Popup ──────────────────────────────────────────────────────

/// Returns: (should_close, dirty, new_dpi)
fn draw_pointer_speed_popup(
    ui: &mut egui::Ui,
    popup_rect: Rect,
    card_rect: Rect,
    current_dpi: i32,
    speed_pct: i32,
) -> (bool, bool, i32) {
    let should_close = draw_popup_shell(ui, popup_rect, card_rect);

    let mut new_dpi = current_dpi;
    let mut dirty = false;

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(popup_rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::ScrollArea::vertical()
        .id_salt("pointer_speed_popup_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut child, |ui| {
            ui.add_space(4.0);

            if let Some(val) = draw_ps_slider(ui, "Pointer speed", speed_pct as f32, 0.0..=100.0) {
                // pct → DPI:  dpi = pct / 100 * 3800 + 200
                new_dpi = ((val / 100.0) * 3800.0 + 200.0).round() as i32;
                new_dpi = new_dpi.clamp(200, 4000);
                dirty = true;
            }

            ui.add_space(8.0);
        });

    (should_close, dirty, new_dpi)
}

// ── Card renderer ────────────────────────────────────────────────────────────

fn draw_ps_card(
    ui: &mut egui::Ui,
    pos: egui::Pos2,
    width: f32,
    height: f32,
    title: &str,
    rows: &[&str],
    is_active: bool,
) -> egui::Response {
    let rect = Rect::from_center_size(pos, vec2(width, height));
    let response = ui.allocate_rect(rect, egui::Sense::click());
    let is_hovered = response.hovered();

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    let bg = if is_active || is_hovered {
        Color32::from_rgb(0x24, 0x24, 0x24)
    } else {
        Color32::from_rgb(0x16, 0x16, 0x16)
    };
    let border = if is_active {
        theme::accent_color(ui.ctx())
    } else if is_hovered {
        Color32::from_rgb(0x3c, 0x3c, 0x3c)
    } else {
        Color32::from_rgb(0x26, 0x26, 0x26)
    };

    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border));

    let sel_t = ui.ctx().animate_bool(response.id, is_active);
    let hov_t = ui.ctx().animate_bool(response.id.with("hov"), is_hovered);
    theme::draw_tech_corners(
        ui.painter(),
        rect,
        theme::lerp_color(
            Color32::TRANSPARENT,
            theme::accent_color(ui.ctx()),
            sel_t.max(hov_t),
        ),
        4.0,
    );

    let left_x = rect.left() + 10.0;
    let mut y = rect.top() + 7.0;

    let title_color = if is_active {
        theme::accent_color(ui.ctx())
    } else {
        theme::primary_text(ui.ctx())
    };
    let title_galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            title.to_string(),
            egui::FontId::proportional(10.5),
            title_color,
        ))
    });
    ui.painter()
        .galley(pos2(left_x, y), title_galley, title_color);
    y += 14.0;

    for &row in rows {
        let sub_color = theme::muted_text(ui.ctx());
        let galley = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                row.to_string(),
                egui::FontId::proportional(9.0),
                sub_color,
            ))
        });
        ui.painter()
            .galley(pos2(left_x, y + 1.0), galley, sub_color);
        y += 14.0;
    }

    response
}
