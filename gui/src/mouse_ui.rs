use mouser_engine::Engine;
use mouser_engine::config::Config;
use crate::ActiveView;
use crate::theme;
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Buttons,
    PointAndScroll,
    Flow,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomizingButton {
    Middle,
    Top,
    Forward,
    Back,
    Thumbwheel,
    Thumb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    MiddleClick,
    ModeShift,
    Forward,
    Back,
    HorizontalScroll,
    Gestures,
    Keystroke,
    Disabled,
}

impl ButtonAction {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::MiddleClick => "Middle click",
            Self::ModeShift => "Shift wheel mode",
            Self::Forward => "Forward click",
            Self::Back => "Back click",
            Self::HorizontalScroll => "Horizontal scroll",
            Self::Gestures => "Gestures",
            Self::Keystroke => "Keyboard shortcut",
            Self::Disabled => "Disabled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingTarget {
    Button(CustomizingButton),
    Gesture(CustomizingButton, String), // (button, swipe direction: "up", "down", "left", "right")
}

pub fn get_button_keys(btn: CustomizingButton) -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
    match btn {
        CustomizingButton::Middle => (
            "middle",
            "middle_gesture_enabled",
            "middle_gesture_up",
            "middle_gesture_down",
            "middle_gesture_left",
            "middle_gesture_right",
        ),
        CustomizingButton::Top => (
            "mode_shift",
            "top_gesture_enabled",
            "top_gesture_up",
            "top_gesture_down",
            "top_gesture_left",
            "top_gesture_right",
        ),
        CustomizingButton::Forward => (
            "xbutton2",
            "xbutton2_gesture_enabled",
            "xbutton2_gesture_up",
            "xbutton2_gesture_down",
            "xbutton2_gesture_left",
            "xbutton2_gesture_right",
        ),
        CustomizingButton::Back => (
            "xbutton1",
            "xbutton1_gesture_enabled",
            "xbutton1_gesture_up",
            "xbutton1_gesture_down",
            "xbutton1_gesture_left",
            "xbutton1_gesture_right",
        ),
        CustomizingButton::Thumb => (
            "gesture",
            "gesture_enabled",
            "gesture_up",
            "gesture_down",
            "gesture_left",
            "gesture_right",
        ),
        CustomizingButton::Thumbwheel => (
            "hscroll",
            "hscroll_gesture_enabled",
            "hscroll_gesture_up",
            "hscroll_gesture_down",
            "hscroll_gesture_left",
            "hscroll_gesture_right",
        ),
    }
}

pub fn mapping_to_action(btn: CustomizingButton, mappings: &std::collections::HashMap<String, String>) -> ButtonAction {
    let (base_key, gesture_enabled_key, _, _, _, _) = get_button_keys(btn);
    let val = mappings.get(base_key).cloned().unwrap_or_else(|| "none".to_string());
    let gesture_enabled = mappings.get(gesture_enabled_key).map(|s| s == "true").unwrap_or(false);

    // Check gesture mode via explicit flag, legacy placeholder, or any configured direction.
    let is_gesture = gesture_enabled
        || val == "gestures"
        || {
            let (_, _, up_k, down_k, left_k, right_k) = get_button_keys(btn);
            [up_k, down_k, left_k, right_k].iter().any(|k| {
                mappings.get(*k).map(|v| v != "none").unwrap_or(false)
            })
        };

    if is_gesture {
        ButtonAction::Gestures
    } else if val == "none" {
        ButtonAction::Disabled
    } else if val.starts_with("custom:") {
        ButtonAction::Keystroke
    } else {
        match val.as_str() {
            "mouse_middle_click" => ButtonAction::MiddleClick,
            "switch_scroll_mode" => ButtonAction::ModeShift,
            "mouse_forward_click" => ButtonAction::Forward,
            "mouse_back_click" => ButtonAction::Back,
            "hscroll" => ButtonAction::HorizontalScroll,
            _ => ButtonAction::Disabled,
        }
    }
}

fn egui_key_to_string(key: egui::Key) -> String {
    match key {
        egui::Key::A => "a".to_string(),
        egui::Key::B => "b".to_string(),
        egui::Key::C => "c".to_string(),
        egui::Key::D => "d".to_string(),
        egui::Key::E => "e".to_string(),
        egui::Key::F => "f".to_string(),
        egui::Key::G => "g".to_string(),
        egui::Key::H => "h".to_string(),
        egui::Key::I => "i".to_string(),
        egui::Key::J => "j".to_string(),
        egui::Key::K => "k".to_string(),
        egui::Key::L => "l".to_string(),
        egui::Key::M => "m".to_string(),
        egui::Key::N => "n".to_string(),
        egui::Key::O => "o".to_string(),
        egui::Key::P => "p".to_string(),
        egui::Key::Q => "q".to_string(),
        egui::Key::R => "r".to_string(),
        egui::Key::S => "s".to_string(),
        egui::Key::T => "t".to_string(),
        egui::Key::U => "u".to_string(),
        egui::Key::V => "v".to_string(),
        egui::Key::W => "w".to_string(),
        egui::Key::X => "x".to_string(),
        egui::Key::Y => "y".to_string(),
        egui::Key::Z => "z".to_string(),
        egui::Key::Num0 => "0".to_string(),
        egui::Key::Num1 => "1".to_string(),
        egui::Key::Num2 => "2".to_string(),
        egui::Key::Num3 => "3".to_string(),
        egui::Key::Num4 => "4".to_string(),
        egui::Key::Num5 => "5".to_string(),
        egui::Key::Num6 => "6".to_string(),
        egui::Key::Num7 => "7".to_string(),
        egui::Key::Num8 => "8".to_string(),
        egui::Key::Num9 => "9".to_string(),
        egui::Key::F1 => "f1".to_string(),
        egui::Key::F2 => "f2".to_string(),
        egui::Key::F3 => "f3".to_string(),
        egui::Key::F4 => "f4".to_string(),
        egui::Key::F5 => "f5".to_string(),
        egui::Key::F6 => "f6".to_string(),
        egui::Key::F7 => "f7".to_string(),
        egui::Key::F8 => "f8".to_string(),
        egui::Key::F9 => "f9".to_string(),
        egui::Key::F10 => "f10".to_string(),
        egui::Key::F11 => "f11".to_string(),
        egui::Key::F12 => "f12".to_string(),
        egui::Key::ArrowLeft => "left".to_string(),
        egui::Key::ArrowRight => "right".to_string(),
        egui::Key::ArrowUp => "up".to_string(),
        egui::Key::ArrowDown => "down".to_string(),
        egui::Key::Home => "home".to_string(),
        egui::Key::End => "end".to_string(),
        egui::Key::PageUp => "pageup".to_string(),
        egui::Key::PageDown => "pagedown".to_string(),
        egui::Key::Backspace => "backspace".to_string(),
        egui::Key::Delete => "delete".to_string(),
        egui::Key::Tab => "tab".to_string(),
        egui::Key::Space => "space".to_string(),
        egui::Key::Enter => "enter".to_string(),
        egui::Key::Escape => "escape".to_string(),
        _ => "".to_string(),
    }
}

fn is_valid_combo(combo: &str) -> bool {
    if combo.is_empty() {
        return false;
    }
    let parts: Vec<String> = combo.split('+').map(|s| s.trim().to_lowercase()).collect();
    for part in parts {
        if part != "ctrl" && part != "shift" && part != "alt" && part != "meta" && !part.is_empty() {
            return true;
        }
    }
    false
}

// Global UI states for text edits (thread local to avoid unsafe static mut)
thread_local! {
    pub static NEW_PROFILE_NAME: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static APP_BINDINGS_BUFFER: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static SELECTED_EDIT_PROFILE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    // Keystroke recording state (thread local to avoid multi-thread unsafety)
    pub static RECORDING_TARGET: std::cell::RefCell<Option<RecordingTarget>> = const { std::cell::RefCell::new(None) };
    pub static RECORDED_KEYS: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    engine: &Engine,
    config: &mut Config,
    mouse_texture: &egui::TextureHandle,
    active_view: &mut ActiveView,
    customizing_button: &mut Option<CustomizingButton>,
    customization_tab: &mut SidebarTab,
    conn_type: &str,
    battery_pct: &str,
    is_connected: bool,
) {
    let rect = ui.max_rect();
    let bg = theme::app_bg(ctx);

    // Fill background
    ui.painter().rect_filled(rect, 0.0, bg);

    // ── 1. Header Row ────────────────────────────────────────────────────────
    let header_height = 45.0;
    let header_rect = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + header_height));

    let top_bar_bg = if ui.visuals().dark_mode {
        theme::COLOR_TOP_BAR
    } else {
        egui::Color32::from_rgb(0xe5, 0xe7, 0xeb)
    };
    ui.painter().rect_filled(header_rect, 0.0, top_bar_bg);

    let border_y = header_rect.max.y;
    ui.painter().line_segment(
        [
            pos2(header_rect.min.x, border_y),
            pos2(header_rect.max.x, border_y),
        ],
        Stroke::new(1.0, theme::border_color(ctx)),
    );

    let mut header_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    header_ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

    let mut back_clicked = false;
    let mut header_buttons_hovered = false;

    header_ui.add_space(20.0);

    let (back_rect, back_res) =
        header_ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::click());
    let back_hover_color = if back_res.hovered() {
        header_buttons_hovered = true;
        header_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        if ui.visuals().dark_mode {
            Color32::from_rgba_unmultiplied(255, 255, 255, 20)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 15)
        }
    } else {
        Color32::TRANSPARENT
    };
    header_ui
        .painter()
        .rect_filled(back_rect, 2.0, back_hover_color);

    let arrow_stroke = Stroke::new(1.5, theme::primary_text(ctx));
    let cx = back_rect.center().x;
    let cy = back_rect.center().y;
    header_ui
        .painter()
        .line_segment([pos2(cx - 7.0, cy), pos2(cx + 7.0, cy)], arrow_stroke);
    header_ui
        .painter()
        .line_segment([pos2(cx - 7.0, cy), pos2(cx - 2.0, cy - 5.0)], arrow_stroke);
    header_ui
        .painter()
        .line_segment([pos2(cx - 7.0, cy), pos2(cx - 2.0, cy + 5.0)], arrow_stroke);

    if back_res.clicked() {
        back_clicked = true;
    }

    header_ui.add_space(8.0);

    // Profile pill in header
    let profile_label = "MX Master 3";
    header_ui.add(
        egui::Label::new(
            RichText::new(profile_label)
                .color(theme::primary_text(ctx))
                .size(15.0)
                .strong(),
        )
        .selectable(false),
    );

    header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        ui.add_space(16.0);

        let (close_rect, close_res) =
            ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::click());
        let close_hover = if close_res.hovered() {
            header_buttons_hovered = true;
            if ui.visuals().dark_mode {
                Color32::from_rgba_unmultiplied(255, 0, 0, 40)
            } else {
                Color32::from_rgba_unmultiplied(255, 0, 0, 30)
            }
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(close_rect, 2.0, close_hover);

        let cr_stroke = Stroke::new(1.5, theme::primary_text(ctx));
        let ccx = close_rect.center().x;
        let ccy = close_rect.center().y;
        ui.painter().line_segment(
            [pos2(ccx - 5.0, ccy - 5.0), pos2(ccx + 5.0, ccy + 5.0)],
            cr_stroke,
        );
        ui.painter().line_segment(
            [pos2(ccx - 5.0, ccy + 5.0), pos2(ccx + 5.0, ccy - 5.0)],
            cr_stroke,
        );
        if close_res.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        ui.add_space(6.0);

        let (min_rect, min_res) = ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::click());
        let min_hover = if min_res.hovered() {
            header_buttons_hovered = true;
            if ui.visuals().dark_mode {
                Color32::from_rgba_unmultiplied(255, 255, 255, 20)
            } else {
                Color32::from_rgba_unmultiplied(0, 0, 0, 15)
            }
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(min_rect, 2.0, min_hover);

        let mcx = min_rect.center().x;
        let mcy = min_rect.center().y;
        ui.painter()
            .line_segment([pos2(mcx - 6.0, mcy), pos2(mcx + 6.0, mcy)], cr_stroke);
        if min_res.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    });

    if back_clicked {
        *active_view = ActiveView::EmptyState;
        *customizing_button = None;
        return;
    }

    if !header_buttons_hovered
        && ui.rect_contains_pointer(header_rect)
        && ui.input(|i| i.pointer.primary_pressed())
    {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    // ── 3. Sidebar (Vertical Navigation) ─────────────────────────────────────
    let sidebar_rect = Rect::from_min_max(
        pos2(rect.min.x, header_rect.max.y),
        pos2(rect.min.x + 240.0, rect.max.y),
    );
    let mut sidebar_ui = ui.new_child(egui::UiBuilder::new().max_rect(sidebar_rect));

    sidebar_ui.vertical(|ui| {
        ui.add_space(40.0);

        let tab_width = 170.0;
        let tab_height = 36.0;

        let tabs = &[
            (SidebarTab::Buttons, "BUTTONS"),
            (SidebarTab::PointAndScroll, "POINT AND SCROLL"),
            (SidebarTab::Flow, "FLOW"),
            (SidebarTab::Settings, "PROFILES"),
        ];

        for &(tab, label) in tabs {
            let is_active = tab == *customization_tab;

            ui.horizontal(|ui| {
                ui.add_space(20.0);

                let (tab_rect, tab_res) =
                    ui.allocate_exact_size(vec2(tab_width, tab_height), egui::Sense::click());
                if tab_res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }

                let bg_color = if is_active {
                    theme::accent_color(ui.ctx())
                } else if tab_res.hovered() {
                    theme::hover_color(ui.ctx())
                } else {
                    Color32::TRANSPARENT
                };

                ui.painter().rect_filled(tab_rect, 2.0, bg_color);

                let tab_hov_anim = ui
                    .ctx()
                    .animate_bool(tab_res.id.with("tab_hov"), tab_res.hovered());
                let tab_act_anim = ui.ctx().animate_bool(tab_res.id.with("tab_act"), is_active);
                let tab_t = tab_hov_anim.max(tab_act_anim);
                let tab_corner_color =
                    theme::lerp_color(Color32::TRANSPARENT, theme::accent_color(ui.ctx()), tab_t);
                theme::draw_tech_corners(ui.painter(), tab_rect, tab_corner_color, 4.0);

                let text_color = if is_active {
                    Color32::BLACK
                } else {
                    theme::primary_text(ui.ctx())
                };

                let icon_center = pos2(tab_rect.min.x + 18.0, tab_rect.center().y);
                match tab {
                    SidebarTab::Buttons => draw_equalizer_icon(ui, icon_center, text_color),
                    SidebarTab::PointAndScroll => {
                        draw_mouse_outline_icon(ui, icon_center, text_color)
                    }
                    SidebarTab::Flow => draw_flow_icon(ui, icon_center, text_color),
                    SidebarTab::Settings => draw_hamburger_icon(ui, icon_center, text_color),
                }

                let text_pos = pos2(tab_rect.min.x + 36.0, tab_rect.center().y);
                let text_galley = ui.fonts(|f| {
                    f.layout_job(egui::text::LayoutJob::simple_singleline(
                        label.to_string(),
                        egui::FontId::proportional(11.0),
                        text_color,
                    ))
                });
                ui.painter().galley(
                    pos2(text_pos.x, text_pos.y - text_galley.size().y / 2.0),
                    text_galley,
                    text_color,
                );

                if tab_res.clicked() {
                    *customization_tab = tab;
                    *customizing_button = None;
                }
            });
            ui.add_space(12.0);
        }
    });

    // ── 4. Bottom-Left Status Pill ───────────────────────────────────────────
    let status_w = if is_connected { 85.0 } else { 110.0 };
    let status_rect = Rect::from_min_max(
        pos2(sidebar_rect.min.x + 20.0, sidebar_rect.max.y - 54.0),
        pos2(sidebar_rect.min.x + 20.0 + status_w, sidebar_rect.max.y - 20.0),
    );
    let status_res = ui.allocate_rect(status_rect, egui::Sense::hover());
    let status_res = if is_connected {
        status_res.on_hover_text(format!("{}%", battery_pct))
    } else {
        status_res
    };
    let t = ui.ctx().animate_bool(status_res.id, status_res.hovered());

    let bg = theme::lerp_color(
        theme::surface_color(ctx),
        if ui.visuals().dark_mode {
            egui::Color32::from_rgb(0x1c, 0x1c, 0x1c)
        } else {
            egui::Color32::from_rgb(0xf3, 0xf4, 0xf6)
        },
        t,
    );
    let border = theme::lerp_color(
        theme::border_color(ctx),
        if is_connected {
            theme::COLOR_ACCENT_DIM
        } else {
            theme::border_color(ctx)
        },
        t,
    );

    // Subtle elevation shadow
    let shadow_rect = status_rect
        .expand2(egui::vec2(1.5, 2.0))
        .translate(egui::vec2(0.0, 1.5));
    let shadow_color = if ui.visuals().dark_mode {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 12)
    };
    ui.painter().rect_filled(shadow_rect, 3.0, shadow_color);

    ui.painter().rect_filled(status_rect, 2.0, bg);
    ui.painter().rect_stroke(status_rect, 2.0, Stroke::new(1.0 + 0.5 * t, border));

    let cx = status_rect.center().x;
    let cy = status_rect.center().y;

    if is_connected {
        // Battery widget
        let b_center = pos2(cx - 19.0, cy);
        let level = battery_pct.parse::<f32>().unwrap_or(100.0) / 100.0;
        let b_rect = Rect::from_center_size(b_center, vec2(20.0, 10.0));
        crate::empty_state::draw_battery_widget(ui.painter(), b_rect, level);

        // Thin vertical divider
        let div_x = cx;
        ui.painter().line_segment(
            [pos2(div_x, cy - 9.0), pos2(div_x, cy + 9.0)],
            Stroke::new(1.0, theme::divider_color(ctx)),
        );

        // Connection icon
        let conn_center = pos2(cx + 19.0, cy);
        crate::empty_state::draw_connection_icon_mini(ui.painter(), conn_center, conn_type);
    } else {
        // Grey status dot
        let dot_cx = status_rect.min.x + 14.0;
        let dot_cy = cy;
        ui.painter().circle_filled(
            pos2(dot_cx, dot_cy),
            3.0,
            theme::elevated_color(ctx),
        );
        ui.painter().circle_stroke(
            pos2(dot_cx, dot_cy),
            3.0,
            Stroke::new(1.0, egui::Color32::from_rgb(0x44, 0x44, 0x44)),
        );

        // "Disconnected" label
        let label_font = egui::FontId::proportional(10.0);
        let text_start = pos2(dot_cx + 8.0, dot_cy);
        let disc_text = crate::translation::tr("disconnected", &config.settings.language);
        ui.painter().text(
            text_start,
            egui::Align2::LEFT_CENTER,
            disc_text,
            label_font,
            theme::COLOR_INACTIVE_TEXT,
        );
    }

    // ── 5. Main Content Canvas ───────────────────────────────────────────────
    let canvas_rect = Rect::from_min_max(pos2(sidebar_rect.max.x, header_rect.max.y), rect.max);
    let mut canvas_ui = ui.new_child(egui::UiBuilder::new().max_rect(canvas_rect));

    match *customization_tab {
        SidebarTab::Buttons => {
            show_buttons_tab(&mut canvas_ui, engine, config, mouse_texture, customizing_button);
        }
        SidebarTab::PointAndScroll => {
            show_point_scroll_tab(&mut canvas_ui, engine, config);
        }
        SidebarTab::Flow => {
            show_flow_tab(&mut canvas_ui);
        }
        SidebarTab::Settings => {
            show_profiles_settings_tab(&mut canvas_ui, engine, config);
        }
    }

    // ── 6. Keystroke Recording Overlay Modal ─────────────────────────────────
    let rec_opt = RECORDING_TARGET.with(|r| r.borrow().clone());
    if let Some(target) = rec_opt {
        egui::Area::new(egui::Id::new("recording_keystroke_modal"))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let screen_r = ctx.screen_rect();
                // Dark overlay
                ui.painter().rect_filled(screen_r, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 200));

                let card_w = 420.0;
                let card_h = 180.0;
                let card_rect = Rect::from_center_size(screen_r.center(), vec2(card_w, card_h));

                ui.painter().rect_filled(card_rect, 2.0, theme::surface_color(ctx));
                ui.painter().rect_stroke(card_rect, 2.0, Stroke::new(1.0, theme::border_color(ctx)));
                theme::draw_tech_corners(ui.painter(), card_rect, theme::accent_color(ctx), 8.0);

                let mut modal_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(card_rect.shrink(20.0))
                        .layout(egui::Layout::top_down(egui::Align::Center)),
                );

                modal_ui.add_space(8.0);
                modal_ui.add(egui::Label::new(
                    RichText::new("RECORD KEYBOARD SHORTCUT")
                        .color(Color32::WHITE)
                        .size(14.0)
                        .strong(),
                ));
                modal_ui.add_space(10.0);
                modal_ui.add(egui::Label::new(
                    RichText::new("Press any key combination on your keyboard.\nPress Enter to save, or Escape to cancel.")
                        .color(theme::secondary_text(ctx))
                        .size(11.5),
                ));

                // Listen to keyboard keys
                let mut escape_pressed = false;
                let mut enter_pressed = false;
                let mut new_keys_recorded = None;
                
                modal_ui.input(|i| {
                    let mods = i.modifiers;
                    let has_modifiers = mods.ctrl || mods.shift || mods.alt || mods.mac_cmd;
                    let mut parts = Vec::new();
                    if mods.ctrl { parts.push("ctrl".to_string()); }
                    if mods.shift { parts.push("shift".to_string()); }
                    if mods.alt { parts.push("alt".to_string()); }
                    if mods.mac_cmd { parts.push("meta".to_string()); }

                    for event in &i.events {
                        let detected_key = match event {
                            egui::Event::Key { key, pressed: true, .. } => Some(*key),
                            egui::Event::Copy => Some(egui::Key::C),
                            egui::Event::Cut => Some(egui::Key::X),
                            egui::Event::Paste(_) => Some(egui::Key::V),
                            _ => None,
                        };

                        if let Some(key) = detected_key {
                            if key == egui::Key::Escape && !has_modifiers {
                                escape_pressed = true;
                            } else if key == egui::Key::Enter && !has_modifiers {
                                enter_pressed = true;
                            } else {
                                let name = egui_key_to_string(key);
                                if !name.is_empty() && name != "ctrl" && name != "shift" && name != "alt" && name != "meta" && name != "tab" {
                                    parts.push(name);
                                    new_keys_recorded = Some(parts.join("+"));
                                }
                            }
                        }
                    }
                });

                if let Some(keys) = new_keys_recorded {
                    RECORDED_KEYS.with(|rk| *rk.borrow_mut() = keys);
                }

                if escape_pressed {
                    RECORDING_TARGET.with(|r| *r.borrow_mut() = None);
                    RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
                } else if enter_pressed {
                    let recorded = RECORDED_KEYS.with(|rk| rk.borrow().clone());
                    if is_valid_combo(&recorded) {
                        // Save the shortcut safely
                        let profile_name = &config.active_profile;
                        if let Some(profile) = config.profiles.get(profile_name).cloned().or_else(|| config.profiles.get("default").cloned()) {
                            let mut mappings = profile.mappings;
                            let action_str = format!("custom:{}", recorded);
                            match &target {
                                RecordingTarget::Button(b) => {
                                    let (base_key, gesture_enabled_key, up_k, down_k, left_k, right_k) = get_button_keys(*b);
                                    mappings.insert(base_key.to_string(), action_str);
                                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                                    for dir_key in [up_k, down_k, left_k, right_k] {
                                        mappings.insert(dir_key.to_string(), "none".to_string());
                                    }
                                }
                                RecordingTarget::Gesture(b, dir) => {
                                    let (_, _, up_key, down_key, left_key, right_key) = get_button_keys(*b);
                                    let key_to_update = match dir.as_str() {
                                        "up" => up_key,
                                        "down" => down_key,
                                        "left" => left_key,
                                        _ => right_key,
                                    };
                                    mappings.insert(key_to_update.to_string(), action_str);
                                }
                            }
                            let engine_bg = engine.clone();
                            let profile_name_bg = profile_name.clone();
                            std::thread::spawn(move || {
                                engine_bg.update_profile_mappings(&profile_name_bg, mappings);
                            });
                        }
                    }
                    RECORDING_TARGET.with(|r| *r.borrow_mut() = None);
                    RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
                }

                let current_pressed = RECORDED_KEYS.with(|rk| rk.borrow().clone());

                modal_ui.add_space(20.0);

                let display_combo = if current_pressed.is_empty() {
                    "Press keys...".to_string()
                } else {
                    current_pressed.to_uppercase()
                };

                let text_color = if current_pressed.is_empty() {
                    theme::muted_text(ctx)
                } else {
                    theme::accent_color(ctx)
                };

                // Crisp box for combo preview
                let preview_rect = Rect::from_center_size(card_rect.center() + vec2(0.0, 48.0), vec2(280.0, 36.0));
                ui.painter().rect_filled(preview_rect, 1.0, theme::elevated_color(ctx));
                ui.painter().rect_stroke(preview_rect, 1.0, Stroke::new(1.0, theme::border_color(ctx)));
                ui.painter().text(
                    preview_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &display_combo,
                    egui::FontId::monospace(13.0),
                    text_color,
                );
            });
    }
}

// ── BUTTONS TAB IMPLEMENTATION ───────────────────────────────────────────────
fn show_buttons_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    mouse_texture: &egui::TextureHandle,
    customizing_button: &mut Option<CustomizingButton>,
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

    // 2. Fetch active mappings
    let profile = config.profiles.get(&config.active_profile).cloned().unwrap_or_else(|| {
        config.profiles.get("default").cloned().unwrap()
    });

    let middle_val = mapping_to_action(CustomizingButton::Middle, &profile.mappings);
    let top_val = mapping_to_action(CustomizingButton::Top, &profile.mappings);
    let forward_val = mapping_to_action(CustomizingButton::Forward, &profile.mappings);
    let back_val = mapping_to_action(CustomizingButton::Back, &profile.mappings);
    let thumbwheel_val = mapping_to_action(CustomizingButton::Thumbwheel, &profile.mappings);
    let thumb_val = mapping_to_action(CustomizingButton::Thumb, &profile.mappings);

    let items = &[
        (
            CustomizingButton::Middle,
            center + vec2(80.0, -190.0),
            center + vec2(300.0, -160.0),
            "Middle button",
            middle_val,
        ),
        (
            CustomizingButton::Top,
            center + vec2(112.5, -77.0),
            center + vec2(320.0, 40.0),
            "Top button",
            top_val,
        ),
        (
            CustomizingButton::Thumbwheel,
            center + vec2(0.0, -23.0),
            center + vec2(320.0, 150.0),
            "Thumb wheel",
            thumbwheel_val,
        ),
        (
            CustomizingButton::Forward,
            center + vec2(-50.0, -40.0),
            center + vec2(-320.0, -10.0),
            "Forward button",
            forward_val,
        ),
        (
            CustomizingButton::Back,
            center + vec2(-25.0, 40.0),
            center + vec2(-300.0, 200.0),
            "Back button",
            back_val,
        ),
        (
            CustomizingButton::Thumb,
            center + vec2(-160.0, 60.0),
            center + vec2(-320.0, 130.0),
            "Thumb button",
            thumb_val,
        ),
    ];

    let mut hovered_button = None;

    // Draw connecting lines and dots
    for &(_btn, dot_pos, card_pos, _title, _action) in items {
        let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 100));
        ui.painter().line_segment([dot_pos, card_pos], stroke);

        let card_rect = Rect::from_center_size(card_pos, vec2(145.0, 44.0));
        let is_card_hovered = ui.rect_contains_pointer(card_rect);
        let is_customizing = *customizing_button == Some(_btn);

        let t = ui.ctx().animate_bool(
            ui.id().with(_btn).with("dot_glow"),
            is_card_hovered || is_customizing,
        );
        let dot_color = theme::lerp_color(
            theme::accent_dim_color(ui.ctx()),
            theme::accent_color(ui.ctx()),
            t,
        );

        ui.painter()
            .circle_stroke(dot_pos, 8.0, Stroke::new(1.2, Color32::WHITE));
        ui.painter().circle_filled(dot_pos, 5.0, dot_color);
    }

    // Draw floating tooltip cards
    let mut popup_to_draw = None;

    for &(btn, _dot_pos, card_pos, _title, action) in items {
        let is_customizing = *customizing_button == Some(btn);
        let mut is_hovered = false;

        let subtitle = match btn {
            CustomizingButton::Middle => "Wheel button",
            CustomizingButton::Top => "Top button",
            CustomizingButton::Forward => "Forward button",
            CustomizingButton::Back => "Back button",
            CustomizingButton::Thumbwheel => "Thumb wheel",
            CustomizingButton::Thumb => "Thumb button",
        };

        // Determine specific label
        let (base_key, _, _, _, _, _) = get_button_keys(btn);
        let mapping_str = profile.mappings.get(base_key).cloned().unwrap_or_else(|| "none".to_string());
        
        let primary_label = if btn == CustomizingButton::Thumbwheel {
            let opt = get_thumbwheel_option(&profile.mappings);
            if opt == ThumbwheelOption::KeyboardShortcut {
                let left_val = profile.mappings.get("hscroll_left").cloned().unwrap_or_else(|| "none".to_string());
                let right_val = profile.mappings.get("hscroll_right").cloned().unwrap_or_else(|| "none".to_string());
                let left_label = if left_val.starts_with("custom:") {
                    left_val.strip_prefix("custom:").unwrap().to_uppercase()
                } else {
                    "NONE".to_string()
                };
                let right_label = if right_val.starts_with("custom:") {
                    right_val.strip_prefix("custom:").unwrap().to_uppercase()
                } else {
                    "NONE".to_string()
                };
                format!("{} / {}", left_label, right_label)
            } else {
                opt.display_name().to_string()
            }
        } else {
            let opt = get_button_option(btn, &profile.mappings);
            if opt == UniversalButtonOption::KeyboardShortcut {
                if mapping_str.starts_with("custom:") {
                    mapping_str.strip_prefix("custom:").unwrap().to_uppercase()
                } else {
                    "NONE".to_string()
                }
            } else {
                opt.display_name(btn).to_string()
            }
        };

        let card_res = draw_tooltip_card(
            ui,
            card_pos,
            &primary_label,
            subtitle,
            &mut is_hovered,
            is_customizing,
        );

        if is_hovered {
            hovered_button = Some(btn);
        }

        if card_res.clicked() {
            *customizing_button = Some(btn);
        }

        if is_customizing {
            popup_to_draw = Some((btn, card_pos, card_res.rect, action));
        }
    }

    if let Some(hovered) = hovered_button {
        if let Some(&(_, dot_pos, _, _, _)) = items.iter().find(|(b, _, _, _, _)| *b == hovered) {
            ui.painter().circle_stroke(
                dot_pos,
                14.0,
                Stroke::new(2.0, theme::accent_color(ui.ctx())),
            );
        }
    }

    // Draw Selection Popup Card if active
    if let Some((btn, card_pos, card_rect, _)) = popup_to_draw {
        let show_thumbwheel = btn == CustomizingButton::Thumbwheel;
        let show_thumb = btn == CustomizingButton::Thumb;
        let show_forward = btn == CustomizingButton::Forward;
        let show_back = btn == CustomizingButton::Back;
        let show_top = btn == CustomizingButton::Top;
        let show_wheel = btn == CustomizingButton::Middle;

        let is_gesture_active = if btn == CustomizingButton::Thumbwheel {
            false
        } else {
            get_button_option(btn, &profile.mappings) == UniversalButtonOption::Gesture
        };

        let view_state_id = ui.id().with(format!("popup_view_for_{:?}", btn));
        let current_view = ui.ctx().data(|d| d.get_temp::<PopupView>(view_state_id)).unwrap_or_else(|| {
            if is_gesture_active {
                PopupView::GesturesConfig
            } else {
                PopupView::ActionList
            }
        });

        let popup_w = match &current_view {
            PopupView::GesturesConfig => 240.0,
            PopupView::RecordShortcut { .. } => 200.0,
            PopupView::ActionList => {
                if show_thumbwheel || show_thumb || show_forward || show_back || show_top || show_wheel {
                    200.0
                } else {
                    170.0
                }
            }
        };

        let popup_h = match &current_view {
            PopupView::GesturesConfig => 370.0,
            PopupView::RecordShortcut { .. } => 300.0,
            PopupView::ActionList => {
                if show_thumbwheel || show_thumb || show_forward || show_back || show_top || show_wheel {
                    300.0
                } else {
                    190.0
                }
            }
        };

        let mut popup_rect = if card_pos.x > center.x {
            Rect::from_min_size(
                pos2(card_rect.max.x + 10.0, card_rect.min.y - 40.0),
                vec2(popup_w, popup_h),
            )
        } else {
            Rect::from_min_size(
                pos2(card_rect.min.x - popup_w - 10.0, card_rect.min.y - 40.0),
                vec2(popup_w, popup_h),
            )
        };

        // Clamp inside boundary
        let canvas_min_x = rect.min.x;
        let canvas_max_x = rect.max.x;
        let canvas_min_y = rect.min.y;
        let canvas_max_y = rect.max.y;

        if popup_rect.min.x < canvas_min_x + 10.0 {
            popup_rect = popup_rect.translate(vec2(canvas_min_x + 10.0 - popup_rect.min.x, 0.0));
        }
        if popup_rect.max.x > canvas_max_x - 10.0 {
            popup_rect = popup_rect.translate(vec2((canvas_max_x - 10.0) - popup_rect.max.x, 0.0));
        }
        if popup_rect.min.y < canvas_min_y + 10.0 {
            popup_rect = popup_rect.translate(vec2(0.0, canvas_min_y + 10.0 - popup_rect.min.y));
        }
        if popup_rect.max.y > canvas_max_y - 10.0 {
            popup_rect = popup_rect.translate(vec2(0.0, (canvas_max_y - 10.0) - popup_rect.max.y));
        }

        let clicked_away = match &current_view {
            PopupView::GesturesConfig => {
                draw_gesture_config_ui(
                    ui,
                    engine,
                    config,
                    btn,
                    popup_rect,
                    card_rect,
                    customizing_button,
                )
            }
            PopupView::RecordShortcut { target_key, display_label } => {
                draw_record_shortcut_ui(
                    ui,
                    engine,
                    config,
                    btn,
                    target_key.clone(),
                    display_label.clone(),
                    popup_rect,
                    card_rect,
                    customizing_button,
                )
            }
            PopupView::ActionList => {
                if show_thumbwheel {
                    let mut selected_opt = None;
                    let res = draw_thumbwheel_action_popup(
                        ui,
                        engine,
                        config,
                        popup_rect,
                        card_rect,
                        customizing_button,
                        &mut selected_opt,
                        view_state_id,
                    );

                    if let Some(opt) = selected_opt {
                        let mut mappings = profile.mappings.clone();
                        if opt == ThumbwheelOption::KeyboardShortcut {
                            // Leave it in ActionList view; inline buttons handle transitions.
                            save_thumbwheel_option(opt, &mut mappings);
                            let engine_bg = engine.clone();
                            let profile_bg = config.active_profile.clone();
                            std::thread::spawn(move || { engine_bg.update_profile_mappings(&profile_bg, mappings); });
                        } else {
                            save_thumbwheel_option(opt, &mut mappings);
                            let engine_bg = engine.clone();
                            let profile_bg = config.active_profile.clone();
                            std::thread::spawn(move || { engine_bg.update_profile_mappings(&profile_bg, mappings); });
                            *customizing_button = None;
                        }
                    }
                    res
                } else {
                    let mut selected_opt = None;
                    let res = draw_button_action_popup(
                        ui,
                        btn,
                        engine,
                        config,
                        popup_rect,
                        card_rect,
                        customizing_button,
                        &mut selected_opt,
                        view_state_id,
                    );

                    if let Some(opt) = selected_opt {
                        let mut mappings = profile.mappings.clone();
                        if opt == UniversalButtonOption::Gesture {
                            // Immediately persist gesture_enabled = "true" so the button
                            // card and engine both recognise gestures as active.
                            save_button_option(btn, opt, &mut mappings);
                            let engine_bg = engine.clone();
                            let profile_bg = config.active_profile.clone();
                            std::thread::spawn(move || { engine_bg.update_profile_mappings(&profile_bg, mappings); });
                            // Then show the gesture configuration panel.
                            ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::GesturesConfig));
                        } else if opt == UniversalButtonOption::KeyboardShortcut {
                            let (click_key, _, _, _, _, _) = get_button_keys(btn);
                            let display_label = match btn {
                                CustomizingButton::Thumb => "Thumb Button",
                                CustomizingButton::Forward => "Forward Button",
                                CustomizingButton::Back => "Back Button",
                                CustomizingButton::Top => "Top Button",
                                CustomizingButton::Middle => "Wheel Button",
                                _ => "",
                            };
                            ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::RecordShortcut {
                                target_key: click_key.to_string(),
                                display_label: display_label.to_string(),
                            }));
                        } else {
                            save_button_option(btn, opt, &mut mappings);
                            let engine_bg = engine.clone();
                            let profile_bg = config.active_profile.clone();
                            std::thread::spawn(move || { engine_bg.update_profile_mappings(&profile_bg, mappings); });
                            *customizing_button = None;
                        }
                    }
                    res
                }
            }
        };

        if clicked_away {
            *customizing_button = None;
        }
    }
}

// ── POINT & SCROLL TAB ──────────────────────────────────────────────────────
fn show_point_scroll_tab(ui: &mut egui::Ui, engine: &Engine, config: &mut Config) {
    let mut settings_dirty = false;
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        ui.vertical(|ui| {
            ui.add_space(40.0);
            ui.add(egui::Label::new(
                RichText::new("POINT AND SCROLL")
                    .color(Color32::WHITE)
                    .size(18.0)
                    .strong(),
            ));
            ui.add_space(24.0);

            // Pointer DPI Slider
            ui.add(egui::Label::new(
                RichText::new(format!("Pointer speed (DPI: {})", config.settings.dpi))
                    .color(Color32::WHITE)
                    .size(13.0),
            ));
            ui.add_space(6.0);
            let mut dpi = config.settings.dpi;
            if ui.add(egui::Slider::new(&mut dpi, 200..=4000).step_by(50.0).show_value(false)).changed() {
                config.settings.dpi = dpi;
                settings_dirty = true;
            }
            ui.add_space(20.0);

            // SmartShift toggle
            ui.horizontal(|ui| {
                let mut smart_shift_enabled = config.settings.smart_shift_enabled;
                if ui.checkbox(&mut smart_shift_enabled, "SmartShift (Auto-switching Wheel)").changed() {
                    config.settings.smart_shift_enabled = smart_shift_enabled;
                    settings_dirty = true;
                }
            });
            ui.add_space(10.0);

            if config.settings.smart_shift_enabled {
                // SmartShift Mode dropdown
                ui.horizontal(|ui| {
                    ui.label("Wheel Mode:");
                    let mut mode = config.settings.smart_shift_mode.clone();
                    let combo = egui::ComboBox::from_id_salt("ss_mode")
                        .selected_text(if mode == "ratchet" { "Ratchet (Tactile)" } else { "Freespin (Smooth)" });
                    let res = combo.show_ui(ui, |ui| {
                        let mut changed = false;
                        if ui.selectable_value(&mut mode, "ratchet".to_string(), "Ratchet (Tactile)").clicked() {
                            changed = true;
                        }
                        if ui.selectable_value(&mut mode, "freespin".to_string(), "Freespin (Smooth)").clicked() {
                            changed = true;
                        }
                        changed
                    });

                    if let Some(true) = res.inner {
                        config.settings.smart_shift_mode = mode;
                        settings_dirty = true;
                    }
                });
                ui.add_space(10.0);

                // SmartShift sensitivity threshold slider
                ui.add(egui::Label::new(
                    RichText::new(format!("SmartShift Sensitivity (Threshold: {})", config.settings.smart_shift_threshold))
                        .color(Color32::WHITE)
                        .size(12.0),
                ));
                let mut ss_thresh = config.settings.smart_shift_threshold;
                if ui.add(egui::Slider::new(&mut ss_thresh, 1..=50).show_value(false)).changed() {
                    config.settings.smart_shift_threshold = ss_thresh;
                    settings_dirty = true;
                }
                ui.add_space(20.0);
            }

            // Scroll Direction
            ui.add(egui::Label::new(
                RichText::new("Scroll Direction Settings").color(Color32::WHITE).size(13.0).strong(),
            ));
            ui.add_space(6.0);

            let mut invert_v = config.settings.invert_vscroll;
            if ui.checkbox(&mut invert_v, "Invert Vertical Scroll").changed() {
                config.settings.invert_vscroll = invert_v;
                settings_dirty = true;
            }

            let mut invert_h = config.settings.invert_hscroll;
            if ui.checkbox(&mut invert_h, "Invert Horizontal Scroll").changed() {
                config.settings.invert_hscroll = invert_h;
                settings_dirty = true;
            }
        });
    });

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
        );
    }
}

// ── FLOW TAB ─────────────────────────────────────────────────────────────────
fn show_flow_tab(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        ui.vertical(|ui| {
            ui.add_space(40.0);
            ui.add(egui::Label::new(
                RichText::new("FLOW")
                    .color(Color32::WHITE)
                    .size(18.0)
                    .strong(),
            ));
            ui.add_space(20.0);
            ui.add(egui::Label::new(
                RichText::new("Control multiple computers seamlessly with a single mouse.")
                    .color(theme::muted_text(ui.ctx()))
                    .size(12.5),
            ));

            ui.add_space(30.0);

            let cx = rect.center().x - 100.0;
            let cy = rect.center().y;

            let s_w = 120.0;
            let s_h = 80.0;

            let screen1 = Rect::from_center_size(pos2(cx - 70.0, cy), vec2(s_w, s_h));
            let screen2 = Rect::from_center_size(pos2(cx + 70.0, cy), vec2(s_w, s_h));

            // Screen 1: active
            ui.painter()
                .rect_filled(screen1, 2.0, theme::app_bg(ui.ctx()));
            ui.painter().rect_stroke(
                screen1,
                2.0,
                Stroke::new(1.5, theme::accent_color(ui.ctx())),
            );
            ui.painter().text(
                screen1.center(),
                egui::Align2::CENTER_CENTER,
                "Computer 1",
                egui::FontId::proportional(11.0),
                theme::primary_text(ui.ctx()),
            );

            // Screen 2: inactive
            ui.painter()
                .rect_filled(screen2, 2.0, theme::app_bg(ui.ctx()));
            ui.painter().rect_stroke(
                screen2,
                2.0,
                Stroke::new(1.0, theme::border_color(ui.ctx())),
            );
            ui.painter().text(
                screen2.center(),
                egui::Align2::CENTER_CENTER,
                "Computer 2",
                egui::FontId::proportional(11.0),
                theme::muted_text(ui.ctx()),
            );
        });
    });
}

// ── PROFILES MANAGEMENT TAB ──────────────────────────────────────────────────
fn show_profiles_settings_tab(ui: &mut egui::Ui, engine: &Engine, config: &mut Config) {
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        ui.vertical(|ui| {
            ui.add_space(40.0);
            ui.add(egui::Label::new(
                RichText::new("PROFILES MANAGEMENT")
                    .color(Color32::WHITE)
                    .size(18.0)
                    .strong(),
            ));
            ui.add_space(20.0);

            // Left Sidebar of Profile names, Right pane for Selected Profile setup
            ui.horizontal(|ui| {
                // Left list: Profiles
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(RichText::new("Existing Profiles").strong().size(12.0)));
                    ui.add_space(6.0);

                    let mut sorted_keys: Vec<String> = config.profiles.keys().cloned().collect();
                    sorted_keys.sort();

                    let scroll_h = 180.0;
                    egui::ScrollArea::vertical().max_height(scroll_h).show(ui, |ui| {
                        for p_name in sorted_keys {
                            let is_active = p_name == config.active_profile;
                            let is_editing = SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone()) == p_name;

                            ui.horizontal(|ui| {
                                let label = if is_active {
                                    RichText::new(format!("★ {}", p_name)).strong().color(theme::accent_color(ui.ctx()))
                                } else {
                                    RichText::new(&p_name).color(theme::secondary_text(ui.ctx()))
                                };

                                let select_res = ui.selectable_label(is_editing, label);
                                if select_res.clicked() {
                                    SELECTED_EDIT_PROFILE.with(|p| *p.borrow_mut() = p_name.clone());
                                    // Initialize edit buffer
                                    if let Some(prof) = config.profiles.get(&p_name) {
                                        APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = prof.apps.join(", "));
                                    }
                                }

                                if is_active {
                                    ui.label(RichText::new("(Active)").size(10.0).color(theme::muted_text(ui.ctx())));
                                } else {
                                    // Button to select it as active
                                    if ui.button("Activate").clicked() {
                                        engine.select_profile(&p_name);
                                    }
                                }
                            });
                            ui.add_space(4.0);
                        }
                    });

                    // Add Profile Form
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label("Add New Profile:");
                    ui.horizontal(|ui| {
                        NEW_PROFILE_NAME.with(|name_cell| {
                            let mut name_ref = name_cell.borrow_mut();
                            ui.text_edit_singleline(&mut *name_ref);
                            if ui.button("Add").clicked() {
                                let clean = name_ref.trim();
                                if !clean.is_empty() {
                                    engine.add_profile(clean);
                                    name_ref.clear();
                                }
                            }
                        });
                    });
                });

                ui.add_space(40.0);
                ui.separator();
                ui.add_space(40.0);

                // Right Pane: Edit profile app mappings & deletion
                ui.vertical(|ui| {
                    let sel_profile = SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());
                    if sel_profile.is_empty() {
                        ui.label("Select a profile from the list to manage its settings.");
                    } else {
                        ui.label(RichText::new(format!("Managing Profile: {}", sel_profile)).strong().size(13.0));
                        ui.add_space(14.0);

                        // Deletion (Disabled for default)
                        if sel_profile == "default" {
                            ui.add_enabled(false, egui::Button::new("Delete Profile"));
                            ui.label(RichText::new("The 'default' profile cannot be deleted.").size(10.0).color(theme::muted_text(ui.ctx())));
                        } else {
                            if ui.button(RichText::new("Delete Profile").color(theme::COLOR_DOT_RED)).clicked() {
                                engine.delete_profile(&sel_profile);
                                SELECTED_EDIT_PROFILE.with(|p| p.borrow_mut().clear());
                            }
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(20.0);

                        // App Process Rules
                        ui.label("Target Application Process Names:");
                        ui.label(RichText::new("Comma-separated list of execution processes (e.g. chrome, code, slack)")
                            .size(10.5)
                            .color(theme::muted_text(ui.ctx())));
                        ui.add_space(6.0);

                        APP_BINDINGS_BUFFER.with(|buff_cell| {
                            let mut buff_ref = buff_cell.borrow_mut();
                            ui.text_edit_singleline(&mut *buff_ref);

                            ui.add_space(8.0);
                            if ui.button("Save App Triggers").clicked() {
                                engine.update_app_bindings(&sel_profile, &*buff_ref);
                            }
                        });
                    }
                });
            });
        });
    });
}

// ── CUSTOM INTERFACE ELEMENTS ───────────────────────────────────────────────
fn draw_equalizer_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    for i in 0..3 {
        let x = center.x - 5.0 + i as f32 * 5.0;
        painter.line_segment([pos2(x, center.y - 5.5), pos2(x, center.y + 5.5)], stroke);
        let knob_y = match i {
            0 => center.y - 2.0,
            1 => center.y + 2.5,
            _ => center.y - 3.5,
        };
        painter.circle_filled(pos2(x, knob_y), 1.8, color);
    }
}

fn draw_mouse_outline_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    let r = Rect::from_center_size(center, vec2(10.0, 14.0));
    painter.rect_stroke(r, 4.0, stroke);
    painter.line_segment(
        [pos2(center.x, r.min.y), pos2(center.x, center.y - 1.0)],
        stroke,
    );
}

fn draw_flow_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    let r1 = Rect::from_center_size(center - vec2(2.5, 2.5), vec2(8.0, 6.0));
    painter.rect_stroke(r1, 1.0, stroke);

    let r2 = Rect::from_center_size(center + vec2(2.5, 2.5), vec2(8.0, 6.0));
    painter.rect_filled(r2, 1.0, Color32::from_rgb(0x11, 0x11, 0x11));
    painter.rect_stroke(r2, 1.0, stroke);
}

fn draw_hamburger_icon(ui: &egui::Ui, center: egui::Pos2, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    painter.line_segment(
        [
            pos2(center.x - 6.0, center.y - 3.5),
            pos2(center.x + 6.0, center.y - 3.5),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x - 6.0, center.y),
            pos2(center.x + 6.0, center.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            pos2(center.x - 6.0, center.y + 3.5),
            pos2(center.x + 6.0, center.y + 3.5),
        ],
        stroke,
    );
}

fn draw_tooltip_card(
    ui: &mut egui::Ui,
    pos: egui::Pos2,
    title: &str,
    subtitle: &str,
    is_hovered: &mut bool,
    is_active: bool,
) -> egui::Response {
    let card_w = 145.0;
    let card_h = 44.0;
    let rect = Rect::from_center_size(pos, vec2(card_w, card_h));
    let response = ui.allocate_rect(rect, egui::Sense::click());

    *is_hovered = response.hovered();
    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    let bg_fill = if is_active || response.hovered() {
        Color32::from_rgb(0x24, 0x24, 0x24)
    } else {
        Color32::from_rgb(0x16, 0x16, 0x16)
    };

    let border_color = if is_active {
        theme::accent_color(ui.ctx())
    } else if response.hovered() {
        Color32::from_rgb(0x3c, 0x3c, 0x3c)
    } else {
        Color32::from_rgb(0x26, 0x26, 0x26)
    };

    ui.painter().rect_filled(rect, 2.0, bg_fill);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border_color));

    let is_sel_anim = ui.ctx().animate_bool(response.id, is_active);
    let is_hov_anim = ui
        .ctx()
        .animate_bool(response.id.with("hov"), response.hovered());
    let corner_color = theme::lerp_color(
        Color32::TRANSPARENT,
        theme::accent_color(ui.ctx()),
        is_sel_anim.max(is_hov_anim),
    );
    theme::draw_tech_corners(ui.painter(), rect, corner_color, 4.0);

    let text_color = if is_active {
        theme::accent_color(ui.ctx())
    } else {
        theme::primary_text(ui.ctx())
    };

    let title_pos = pos2(rect.left() + 10.0, rect.top() + 7.0);
    let title_galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            title.to_string(),
            egui::FontId::proportional(10.5),
            text_color,
        ))
    });
    ui.painter().galley(title_pos, title_galley, text_color);

    let sub_pos = pos2(rect.left() + 10.0, rect.top() + 21.0);
    let sub_color = theme::muted_text(ui.ctx());
    let sub_galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            subtitle.to_string(),
            egui::FontId::proportional(9.0),
            sub_color,
        ))
    });
    ui.painter()
        .galley(sub_pos, sub_galley, theme::muted_text(ui.ctx()));

    response
}


// ── THUMBWHEEL OPTIONS ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThumbwheelOption {
    HorizontalScroll,
    Zoom,
    Volume,
    NavTabs,
    KeyboardShortcut,
    Brightness,
    DoNothing,
    ForwardBack,
    NavApps,
    NextPrev,
    SwitchDesktops,
}

impl ThumbwheelOption {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::HorizontalScroll => "Horizontal scroll",
            Self::Zoom => "Zoom in/out",
            Self::Volume => "Volume up/down",
            Self::NavTabs => "Navigate between tabs",
            Self::KeyboardShortcut => "Keyboard shortcut",
            Self::Brightness => "Brightness up/down",
            Self::DoNothing => "Do nothing",
            Self::ForwardBack => "Forward/Back",
            Self::NavApps => "Navigate between apps",
            Self::NextPrev => "Next/Previous",
            Self::SwitchDesktops => "Switch between desktops",
        }
    }
}

pub fn get_thumbwheel_option(mappings: &std::collections::HashMap<String, String>) -> ThumbwheelOption {
    if let Some(hscroll_val) = mappings.get("hscroll") {
        match hscroll_val.as_str() {
            "zoom" => return ThumbwheelOption::Zoom,
            "volume" => return ThumbwheelOption::Volume,
            "nav_tabs" => return ThumbwheelOption::NavTabs,
            "custom" => return ThumbwheelOption::KeyboardShortcut,
            "brightness" => return ThumbwheelOption::Brightness,
            "none" => return ThumbwheelOption::DoNothing,
            "forward_back" => return ThumbwheelOption::ForwardBack,
            "nav_apps" => return ThumbwheelOption::NavApps,
            "next_prev" => return ThumbwheelOption::NextPrev,
            "switch_desktops" => return ThumbwheelOption::SwitchDesktops,
            _ => {}
        }
    }
    ThumbwheelOption::HorizontalScroll
}

pub fn save_thumbwheel_option(opt: ThumbwheelOption, mappings: &mut std::collections::HashMap<String, String>) {
    match opt {
        ThumbwheelOption::HorizontalScroll => {
            mappings.insert("hscroll".to_string(), "hscroll".to_string());
            mappings.insert("hscroll_left".to_string(), "none".to_string());
            mappings.insert("hscroll_right".to_string(), "none".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::Zoom => {
            mappings.insert("hscroll".to_string(), "zoom".to_string());
            mappings.insert("hscroll_left".to_string(), "zoom_out".to_string());
            mappings.insert("hscroll_right".to_string(), "zoom_in".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::Volume => {
            mappings.insert("hscroll".to_string(), "volume".to_string());
            mappings.insert("hscroll_left".to_string(), "volume_down".to_string());
            mappings.insert("hscroll_right".to_string(), "volume_up".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::NavTabs => {
            mappings.insert("hscroll".to_string(), "nav_tabs".to_string());
            mappings.insert("hscroll_left".to_string(), "tab_prev".to_string());
            mappings.insert("hscroll_right".to_string(), "tab_next".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::KeyboardShortcut => {
            mappings.insert("hscroll".to_string(), "custom".to_string());
            if !mappings.contains_key("hscroll_left") {
                mappings.insert("hscroll_left".to_string(), "none".to_string());
            }
            if !mappings.contains_key("hscroll_right") {
                mappings.insert("hscroll_right".to_string(), "none".to_string());
            }
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::Brightness => {
            mappings.insert("hscroll".to_string(), "brightness".to_string());
            mappings.insert("hscroll_left".to_string(), "brightness_down".to_string());
            mappings.insert("hscroll_right".to_string(), "brightness_up".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::DoNothing => {
            mappings.insert("hscroll".to_string(), "none".to_string());
            mappings.insert("hscroll_left".to_string(), "none".to_string());
            mappings.insert("hscroll_right".to_string(), "none".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::ForwardBack => {
            mappings.insert("hscroll".to_string(), "forward_back".to_string());
            mappings.insert("hscroll_left".to_string(), "browser_back".to_string());
            mappings.insert("hscroll_right".to_string(), "browser_forward".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::NavApps => {
            mappings.insert("hscroll".to_string(), "nav_apps".to_string());
            mappings.insert("hscroll_left".to_string(), "app_prev".to_string());
            mappings.insert("hscroll_right".to_string(), "app_next".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::NextPrev => {
            mappings.insert("hscroll".to_string(), "next_prev".to_string());
            mappings.insert("hscroll_left".to_string(), "prev_track".to_string());
            mappings.insert("hscroll_right".to_string(), "next_track".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
        ThumbwheelOption::SwitchDesktops => {
            mappings.insert("hscroll".to_string(), "switch_desktops".to_string());
            mappings.insert("hscroll_left".to_string(), "space_left".to_string());
            mappings.insert("hscroll_right".to_string(), "space_right".to_string());
            mappings.insert("hscroll_gesture_enabled".to_string(), "false".to_string());
        }
    }
}

fn draw_thumbwheel_action_popup(
    ui: &mut egui::Ui,
    _engine: &Engine,
    config: &mut Config,
    rect: Rect,
    card_rect: Rect,
    customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<ThumbwheelOption>,
    view_state_id: egui::Id,
) -> bool {
    let _response = ui.allocate_rect(rect, egui::Sense::click());

    let clicked_away = ui.input(|i| i.pointer.any_click())
        && !ui.rect_contains_pointer(rect)
        && !ui.rect_contains_pointer(card_rect);

    let bg = theme::surface_color(ui.ctx());
    let border = theme::border_color(ui.ctx());

    // Draw drop shadow
    let shadow_rect = rect.expand2(vec2(2.0, 3.0)).translate(vec2(0.0, 2.0));
    ui.painter().rect_filled(
        shadow_rect,
        3.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );

    // Outer popup container
    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border));
    theme::draw_tech_corners(ui.painter(), rect, theme::accent_color(ui.ctx()), 6.0);

    let profile = config.profiles.get(&config.active_profile).cloned().unwrap();
    let current_opt = get_thumbwheel_option(&profile.mappings);

    let mut click_occurred = false;

    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::ScrollArea::vertical()
        .id_salt("thumbwheel_scroll")
        .show(&mut child_ui, |ui| {
            ui.add_space(4.0);
            
            // RECOMMENDED header
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("RECOMMENDED")
                        .font(egui::FontId::proportional(9.0))
                        .strong()
                        .color(theme::secondary_text(ui.ctx())),
                );
            });
            ui.add_space(4.0);

            let recommended = &[
                ThumbwheelOption::HorizontalScroll,
                ThumbwheelOption::Zoom,
                ThumbwheelOption::Volume,
                ThumbwheelOption::NavTabs,
                ThumbwheelOption::KeyboardShortcut,
            ];

            for &opt in recommended {
                if draw_thumbwheel_item(ui, opt, current_opt, &profile, customizing_button, selected_option, view_state_id) {
                    click_occurred = true;
                }
            }

            ui.add_space(8.0);

            // OTHER ACTIONS header
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("OTHER ACTIONS")
                        .font(egui::FontId::proportional(9.0))
                        .strong()
                        .color(theme::secondary_text(ui.ctx())),
                );
            });
            ui.add_space(4.0);

            let other = &[
                ThumbwheelOption::Brightness,
                ThumbwheelOption::DoNothing,
                ThumbwheelOption::ForwardBack,
                ThumbwheelOption::NavApps,
                ThumbwheelOption::NextPrev,
                ThumbwheelOption::SwitchDesktops,
            ];

            for &opt in other {
                if draw_thumbwheel_item(ui, opt, current_opt, &profile, customizing_button, selected_option, view_state_id) {
                    click_occurred = true;
                }
            }

            ui.add_space(4.0);
        });

    clicked_away && !click_occurred
}

fn draw_thumbwheel_item(
    ui: &mut egui::Ui,
    opt: ThumbwheelOption,
    current_opt: ThumbwheelOption,
    profile: &mouser_engine::config::Profile,
    _customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<ThumbwheelOption>,
    view_state_id: egui::Id,
) -> bool {
    let is_selected = opt == current_opt;
    let item_h = 24.0;
    
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), item_h), egui::Sense::click());
    let is_hovered = response.hovered();

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(rect, 0.0, theme::hover_color(ui.ctx()));
    }

    let bullet_center = pos2(rect.min.x + 14.0, rect.center().y);
    if is_selected {
        let accent = theme::accent_color(ui.ctx());
        ui.painter().circle_filled(bullet_center, 6.0, accent);
        ui.painter().circle_filled(bullet_center, 2.0, theme::surface_color(ui.ctx()));
    } else {
        let bullet_color = Color32::from_gray(60);
        ui.painter().circle_filled(bullet_center, 6.0, bullet_color);
    }

    let text_color = if is_selected {
        theme::accent_color(ui.ctx())
    } else if is_hovered {
        theme::primary_text(ui.ctx())
    } else {
        theme::secondary_text(ui.ctx())
    };

    let label_text = opt.display_name();
    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            label_text.to_string(),
            egui::FontId::proportional(11.0),
            text_color,
        ))
    });
    let text_y = rect.center().y - galley.size().y / 2.0;
    ui.painter().galley(pos2(rect.min.x + 28.0, text_y), galley, text_color);

    let mut clicked = false;
    if response.clicked() {
        *selected_option = Some(opt);
        clicked = true;
    }

    if opt == ThumbwheelOption::KeyboardShortcut && is_selected {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(28.0);
            
            let left_val = profile.mappings.get("hscroll_left").cloned().unwrap_or_else(|| "none".to_string());
            let left_text = if left_val.starts_with("custom:") {
                left_val.strip_prefix("custom:").unwrap().to_uppercase()
            } else {
                "Record Left".to_string()
            };
            
            let btn_left = ui.add(egui::Button::new(
                RichText::new(format!("Left: {}", left_text)).size(10.0)
            ));
            if btn_left.clicked() {
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
                ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::RecordShortcut {
                    target_key: "hscroll_left".to_string(),
                    display_label: "Scroll Left".to_string(),
                }));
                clicked = true;
            }

            let right_val = profile.mappings.get("hscroll_right").cloned().unwrap_or_else(|| "none".to_string());
            let right_text = if right_val.starts_with("custom:") {
                right_val.strip_prefix("custom:").unwrap().to_uppercase()
            } else {
                "Record Right".to_string()
            };
            
            let btn_right = ui.add(egui::Button::new(
                RichText::new(format!("Right: {}", right_text)).size(10.0)
            ));
            if btn_right.clicked() {
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
                ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::RecordShortcut {
                    target_key: "hscroll_right".to_string(),
                    display_label: "Scroll Right".to_string(),
                }));
                clicked = true;
            }
        });
        ui.add_space(6.0);
    }

    clicked
}


// ── UNIFIED BUTTON OPTIONS ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniversalButtonOption {
    Gesture,
    TaskView,
    ShowHideDesktop,
    ScreenCapture,
    PrintScreen,
    SwitchApplication,
    KeyboardShortcut,
    ActionCenter,
    AdvancedClick,
    Back,
    BrightnessDown,
    BrightnessUp,
    Calculator,
    ChangePointerSpeed,
    CloseWindow,
    Copy,
    Cut,
    DesktopLeft,
    DesktopRight,
    Dictation,
    DoNothing,
    Emoji,
    EmojisMenu,
    Forward,
    InputLanguage,
    Lock,
    MaximizeWindow,
    MiddleButton,
    MinimizeWindow,
    MuteUnmuteSpeaker,
    NewBrowserTab,
    Next,
    OpenApplication,
    OpenFile,
    OpenFolder,
    Paste,
    PlayPause,
    Previous,
    Redo,
    RightCtrl,
    ScreenSnip,
    ShiftWheelMode,
    ThisPC,
    Undo,
    VolumeDown,
    VolumeUp,
    ZoomIn,
    ZoomOut,
}

impl UniversalButtonOption {
    pub fn display_name(self, btn: CustomizingButton) -> &'static str {
        match self {
            Self::Gesture => {
                if btn == CustomizingButton::Thumb || btn == CustomizingButton::Top || btn == CustomizingButton::Middle {
                    "Gestures"
                } else {
                    "Gesture"
                }
            }
            Self::TaskView => "Task view",
            Self::ShowHideDesktop => "Show/hide desktop",
            Self::ScreenCapture => "Screen capture",
            Self::PrintScreen => "Print screen",
            Self::SwitchApplication => "Switch application",
            Self::KeyboardShortcut => "Keyboard shortcut",
            Self::ActionCenter => "Action center",
            Self::AdvancedClick => "Advanced click",
            Self::Back => "Back",
            Self::BrightnessDown => "Brightness down",
            Self::BrightnessUp => "Brightness up",
            Self::Calculator => "Calculator",
            Self::ChangePointerSpeed => "Change pointer speed",
            Self::CloseWindow => "Close window",
            Self::Copy => "Copy",
            Self::Cut => "Cut",
            Self::DesktopLeft => "Desktop left",
            Self::DesktopRight => "Desktop right",
            Self::Dictation => "Dictation",
            Self::DoNothing => "Do nothing",
            Self::Emoji => "Emoji",
            Self::EmojisMenu => "Emoji's menu",
            Self::Forward => "Forward",
            Self::InputLanguage => "Input Language",
            Self::Lock => "Lock",
            Self::MaximizeWindow => "Maximize window",
            Self::MiddleButton => "Middle button",
            Self::MinimizeWindow => "Minimize window",
            Self::MuteUnmuteSpeaker => "Mute/Unmute speaker",
            Self::NewBrowserTab => "New browser tab",
            Self::Next => "Next",
            Self::OpenApplication => "Open application",
            Self::OpenFile => "Open file",
            Self::OpenFolder => "Open folder",
            Self::Paste => "Paste",
            Self::PlayPause => "Play/Pause",
            Self::Previous => "Previous",
            Self::Redo => "Redo",
            Self::RightCtrl => "Right Ctrl",
            Self::ScreenSnip => "Screen snip",
            Self::ShiftWheelMode => "Shift wheel mode",
            Self::ThisPC => "This PC",
            Self::Undo => "Undo",
            Self::VolumeDown => "Volume down",
            Self::VolumeUp => "Volume up",
            Self::ZoomIn => "Zoom in",
            Self::ZoomOut => "Zoom out",
        }
    }
}

impl CustomizingButton {
    pub fn config_keys(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Thumb => ("gesture", "gesture_enabled", "none"),
            Self::Forward => ("xbutton2", "xbutton2_gesture_enabled", "mouse_forward_click"),
            Self::Back => ("xbutton1", "xbutton1_gesture_enabled", "mouse_back_click"),
            Self::Top => ("mode_shift", "top_gesture_enabled", "switch_scroll_mode"),
            Self::Middle => ("middle", "middle_gesture_enabled", "mouse_middle_click"),
            _ => ("", "", ""),
        }
    }

    pub fn recommended_options(self) -> &'static [UniversalButtonOption] {
        match self {
            Self::Thumb => &[
                UniversalButtonOption::Gesture,
                UniversalButtonOption::TaskView,
                UniversalButtonOption::ShowHideDesktop,
                UniversalButtonOption::ScreenCapture,
                UniversalButtonOption::PrintScreen,
                UniversalButtonOption::SwitchApplication,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Forward => &[
                UniversalButtonOption::Forward,
                UniversalButtonOption::Paste,
                UniversalButtonOption::VolumeUp,
                UniversalButtonOption::Redo,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Back => &[
                UniversalButtonOption::Back,
                UniversalButtonOption::Copy,
                UniversalButtonOption::VolumeDown,
                UniversalButtonOption::Undo,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Top => &[
                UniversalButtonOption::ShiftWheelMode,
                UniversalButtonOption::TaskView,
                UniversalButtonOption::Gesture,
                UniversalButtonOption::ScreenCapture,
                UniversalButtonOption::PrintScreen,
                UniversalButtonOption::KeyboardShortcut,
            ],
            Self::Middle => &[
                UniversalButtonOption::MiddleButton,
                UniversalButtonOption::ShiftWheelMode,
                UniversalButtonOption::TaskView,
                UniversalButtonOption::ShowHideDesktop,
                UniversalButtonOption::Gesture,
                UniversalButtonOption::KeyboardShortcut,
            ],
            _ => &[],
        }
    }

    pub fn other_options(self) -> Vec<UniversalButtonOption> {
        let recommended = self.recommended_options();
        let all_options = &[
            UniversalButtonOption::Gesture,
            UniversalButtonOption::ActionCenter,
            UniversalButtonOption::AdvancedClick,
            UniversalButtonOption::Back,
            UniversalButtonOption::BrightnessDown,
            UniversalButtonOption::BrightnessUp,
            UniversalButtonOption::Calculator,
            UniversalButtonOption::ChangePointerSpeed,
            UniversalButtonOption::CloseWindow,
            UniversalButtonOption::Copy,
            UniversalButtonOption::Cut,
            UniversalButtonOption::DesktopLeft,
            UniversalButtonOption::DesktopRight,
            UniversalButtonOption::Dictation,
            UniversalButtonOption::DoNothing,
            UniversalButtonOption::Emoji,
            UniversalButtonOption::EmojisMenu,
            UniversalButtonOption::Forward,
            UniversalButtonOption::InputLanguage,
            UniversalButtonOption::Lock,
            UniversalButtonOption::MaximizeWindow,
            UniversalButtonOption::MiddleButton,
            UniversalButtonOption::MinimizeWindow,
            UniversalButtonOption::MuteUnmuteSpeaker,
            UniversalButtonOption::NewBrowserTab,
            UniversalButtonOption::Next,
            UniversalButtonOption::OpenApplication,
            UniversalButtonOption::OpenFile,
            UniversalButtonOption::OpenFolder,
            UniversalButtonOption::Paste,
            UniversalButtonOption::PlayPause,
            UniversalButtonOption::Previous,
            UniversalButtonOption::PrintScreen,
            UniversalButtonOption::Redo,
            UniversalButtonOption::RightCtrl,
            UniversalButtonOption::ScreenCapture,
            UniversalButtonOption::ScreenSnip,
            UniversalButtonOption::ShiftWheelMode,
            UniversalButtonOption::ShowHideDesktop,
            UniversalButtonOption::SwitchApplication,
            UniversalButtonOption::TaskView,
            UniversalButtonOption::ThisPC,
            UniversalButtonOption::Undo,
            UniversalButtonOption::VolumeDown,
            UniversalButtonOption::VolumeUp,
            UniversalButtonOption::ZoomIn,
            UniversalButtonOption::ZoomOut,
        ];

        all_options.iter()
            .copied()
            .filter(|opt| !recommended.contains(opt))
            .collect()
    }
}

pub fn get_button_option(
    btn: CustomizingButton,
    mappings: &std::collections::HashMap<String, String>,
) -> UniversalButtonOption {
    let (base_key, gesture_enabled_key, _) = btn.config_keys();

    // Primary check: explicit gesture_enabled flag.
    let gesture_enabled = mappings.get(gesture_enabled_key).map(|s| s == "true").unwrap_or(false);
    if gesture_enabled {
        return UniversalButtonOption::Gesture;
    }

    // Legacy migration: if the base key is "gestures" (old placeholder), treat as gesture mode.
    if mappings.get(base_key).map(|s| s == "gestures").unwrap_or(false) {
        return UniversalButtonOption::Gesture;
    }

    // Auto-detect: if any gesture direction is non-"none", the button is in gesture mode
    // even if gesture_enabled was never explicitly saved as "true" (e.g. old configs).
    {
        let (_, _, up_k, down_k, left_k, right_k) = get_button_keys(btn);
        let has_gesture = [up_k, down_k, left_k, right_k].iter().any(|k| {
            mappings.get(*k).map(|v| v != "none").unwrap_or(false)
        });
        if has_gesture {
            return UniversalButtonOption::Gesture;
        }
    }

    if let Some(val) = mappings.get(base_key) {
        match val.as_str() {
            "task_view" => return UniversalButtonOption::TaskView,
            "win_d" => return UniversalButtonOption::ShowHideDesktop,
            "screen_capture" => return UniversalButtonOption::ScreenCapture,
            "print_screen" => return UniversalButtonOption::PrintScreen,
            "alt_tab" => return UniversalButtonOption::SwitchApplication,
            "action_center" => return UniversalButtonOption::ActionCenter,
            "advanced_click" => return UniversalButtonOption::AdvancedClick,
            "browser_back" | "mouse_back_click" => return UniversalButtonOption::Back,
            "brightness_down" => return UniversalButtonOption::BrightnessDown,
            "brightness_up" => return UniversalButtonOption::BrightnessUp,
            "calculator" => return UniversalButtonOption::Calculator,
            "cycle_dpi" => return UniversalButtonOption::ChangePointerSpeed,
            "close_window" => return UniversalButtonOption::CloseWindow,
            "copy" => return UniversalButtonOption::Copy,
            "cut" => return UniversalButtonOption::Cut,
            "space_left" => return UniversalButtonOption::DesktopLeft,
            "space_right" => return UniversalButtonOption::DesktopRight,
            "dictation" => return UniversalButtonOption::Dictation,
            "none" => return UniversalButtonOption::DoNothing,
            "emoji" => return UniversalButtonOption::Emoji,
            "emojis_menu" => return UniversalButtonOption::EmojisMenu,
            "browser_forward" | "mouse_forward_click" => return UniversalButtonOption::Forward,
            "input_language" => return UniversalButtonOption::InputLanguage,
            "lock" => return UniversalButtonOption::Lock,
            "maximize_window" => return UniversalButtonOption::MaximizeWindow,
            "mouse_middle_click" => return UniversalButtonOption::MiddleButton,
            "minimize_window" => return UniversalButtonOption::MinimizeWindow,
            "volume_mute" => return UniversalButtonOption::MuteUnmuteSpeaker,
            "new_tab" => return UniversalButtonOption::NewBrowserTab,
            "next_track" => return UniversalButtonOption::Next,
            "open_application" => return UniversalButtonOption::OpenApplication,
            "open_file" => return UniversalButtonOption::OpenFile,
            "open_folder" => return UniversalButtonOption::OpenFolder,
            "paste" => return UniversalButtonOption::Paste,
            "play_pause" => return UniversalButtonOption::PlayPause,
            "prev_track" => return UniversalButtonOption::Previous,
            "redo" => return UniversalButtonOption::Redo,
            "right_ctrl" => return UniversalButtonOption::RightCtrl,
            "screen_snip" => return UniversalButtonOption::ScreenSnip,
            "switch_scroll_mode" => return UniversalButtonOption::ShiftWheelMode,
            "this_pc" => return UniversalButtonOption::ThisPC,
            "undo" => return UniversalButtonOption::Undo,
            "volume_down" => return UniversalButtonOption::VolumeDown,
            "volume_up" => return UniversalButtonOption::VolumeUp,
            "zoom_in" => return UniversalButtonOption::ZoomIn,
            "zoom_out" => return UniversalButtonOption::ZoomOut,
            s if s.starts_with("custom:") => return UniversalButtonOption::KeyboardShortcut,
            _ => {}
        }
    }

    match btn {
        CustomizingButton::Thumb => UniversalButtonOption::DoNothing,
        CustomizingButton::Forward => UniversalButtonOption::Forward,
        CustomizingButton::Back => UniversalButtonOption::Back,
        CustomizingButton::Top => UniversalButtonOption::ShiftWheelMode,
        CustomizingButton::Middle => UniversalButtonOption::MiddleButton,
        _ => UniversalButtonOption::DoNothing,
    }
}

pub fn save_button_option(
    btn: CustomizingButton,
    opt: UniversalButtonOption,
    mappings: &mut std::collections::HashMap<String, String>,
) {
    let (base_key, gesture_enabled_key, _) = btn.config_keys();
    if opt == UniversalButtonOption::Gesture {
        mappings.insert(gesture_enabled_key.to_string(), "true".to_string());
        // Preserve the existing click-fallback action (the base key doubles as the
        // "CLICK" slot inside gesture config). Only reset if it holds the legacy
        // "gestures" placeholder or is completely absent — never overwrite a valid action.
        let existing = mappings.get(base_key).map(|s| s.as_str()).unwrap_or("none");
        if existing == "gestures" || existing.is_empty() {
            mappings.insert(base_key.to_string(), "none".to_string());
        }
        return;
    }

    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());

    // Clear gesture direction slots so auto-detect won't re-enable gesture mode.
    let (_, _, up_k, down_k, left_k, right_k) = get_button_keys(btn);
    for dir_key in [up_k, down_k, left_k, right_k] {
        mappings.insert(dir_key.to_string(), "none".to_string());
    }

    let val = match opt {
        UniversalButtonOption::Gesture => "gestures",
        UniversalButtonOption::TaskView => "task_view",
        UniversalButtonOption::ShowHideDesktop => "win_d",
        UniversalButtonOption::ScreenCapture => "screen_capture",
        UniversalButtonOption::PrintScreen => "print_screen",
        UniversalButtonOption::SwitchApplication => "alt_tab",
        UniversalButtonOption::KeyboardShortcut => {
            if !mappings.get(base_key).map(|s| s.starts_with("custom:")).unwrap_or(false) {
                mappings.insert(base_key.to_string(), "none".to_string());
            }
            return;
        }
        UniversalButtonOption::ActionCenter => "action_center",
        UniversalButtonOption::AdvancedClick => "advanced_click",
        UniversalButtonOption::Back => {
            if btn == CustomizingButton::Back { "mouse_back_click" } else { "browser_back" }
        }
        UniversalButtonOption::BrightnessDown => "brightness_down",
        UniversalButtonOption::BrightnessUp => "brightness_up",
        UniversalButtonOption::Calculator => "calculator",
        UniversalButtonOption::ChangePointerSpeed => "cycle_dpi",
        UniversalButtonOption::CloseWindow => "close_window",
        UniversalButtonOption::Copy => "copy",
        UniversalButtonOption::Cut => "cut",
        UniversalButtonOption::DesktopLeft => "space_left",
        UniversalButtonOption::DesktopRight => "space_right",
        UniversalButtonOption::Dictation => "dictation",
        UniversalButtonOption::DoNothing => "none",
        UniversalButtonOption::Emoji => "emoji",
        UniversalButtonOption::EmojisMenu => "emojis_menu",
        UniversalButtonOption::Forward => {
            if btn == CustomizingButton::Forward { "mouse_forward_click" } else { "browser_forward" }
        }
        UniversalButtonOption::InputLanguage => "input_language",
        UniversalButtonOption::Lock => "lock",
        UniversalButtonOption::MaximizeWindow => "maximize_window",
        UniversalButtonOption::MiddleButton => "mouse_middle_click",
        UniversalButtonOption::MinimizeWindow => "minimize_window",
        UniversalButtonOption::MuteUnmuteSpeaker => "volume_mute",
        UniversalButtonOption::NewBrowserTab => "new_tab",
        UniversalButtonOption::Next => "next_track",
        UniversalButtonOption::OpenApplication => "open_application",
        UniversalButtonOption::OpenFile => "open_file",
        UniversalButtonOption::OpenFolder => "open_folder",
        UniversalButtonOption::Paste => "paste",
        UniversalButtonOption::PlayPause => "play_pause",
        UniversalButtonOption::Previous => "prev_track",
        UniversalButtonOption::Redo => "redo",
        UniversalButtonOption::RightCtrl => "right_ctrl",
        UniversalButtonOption::ScreenSnip => "screen_snip",
        UniversalButtonOption::ShiftWheelMode => "switch_scroll_mode",
        UniversalButtonOption::ThisPC => "this_pc",
        UniversalButtonOption::Undo => "undo",
        UniversalButtonOption::VolumeDown => "volume_down",
        UniversalButtonOption::VolumeUp => "volume_up",
        UniversalButtonOption::ZoomIn => "zoom_in",
        UniversalButtonOption::ZoomOut => "zoom_out",
    };

    mappings.insert(base_key.to_string(), val.to_string());
}

fn draw_button_item(
    ui: &mut egui::Ui,
    btn: CustomizingButton,
    opt: UniversalButtonOption,
    current_opt: UniversalButtonOption,
    profile: &mouser_engine::config::Profile,
    _customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<UniversalButtonOption>,
    view_state_id: egui::Id,
) -> bool {
    let is_selected = opt == current_opt;
    let item_h = 24.0;

    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), item_h), egui::Sense::click());
    let is_hovered = response.hovered();

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        ui.painter().rect_filled(rect, 0.0, theme::hover_color(ui.ctx()));
    }

    let bullet_center = pos2(rect.min.x + 14.0, rect.center().y);
    if is_selected {
        let accent = theme::accent_color(ui.ctx());
        ui.painter().circle_filled(bullet_center, 6.0, accent);
        ui.painter().circle_filled(bullet_center, 2.0, theme::surface_color(ui.ctx()));
    } else {
        let bullet_color = Color32::from_gray(60);
        ui.painter().circle_filled(bullet_center, 6.0, bullet_color);
    }

    let text_color = if is_selected {
        theme::accent_color(ui.ctx())
    } else if is_hovered {
        theme::primary_text(ui.ctx())
    } else {
        theme::secondary_text(ui.ctx())
    };

    let label_text = opt.display_name(btn);
    let galley = ui.fonts(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            label_text.to_string(),
            egui::FontId::proportional(11.0),
            text_color,
        ))
    });
    let text_y = rect.center().y - galley.size().y / 2.0;
    ui.painter().galley(pos2(rect.min.x + 28.0, text_y), galley, text_color);

    let mut clicked = false;
    if response.clicked() {
        *selected_option = Some(opt);
        clicked = true;
    }

    if opt == UniversalButtonOption::KeyboardShortcut && is_selected {
        let (base_key, _, _) = btn.config_keys();
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(28.0);

            let val = profile.mappings.get(base_key).cloned().unwrap_or_else(|| "none".to_string());
            let keys_text = if val.starts_with("custom:") {
                val.strip_prefix("custom:").unwrap().to_uppercase()
            } else {
                "Record Keystroke".to_string()
            };

            let btn_rec = ui.add(egui::Button::new(
                RichText::new(keys_text).size(10.0)
            ));
            if btn_rec.clicked() {
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
                let display_label = match btn {
                    CustomizingButton::Thumb => "Thumb Button",
                    CustomizingButton::Forward => "Forward Button",
                    CustomizingButton::Back => "Back Button",
                    CustomizingButton::Top => "Top Button",
                    CustomizingButton::Middle => "Wheel Button",
                    _ => "",
                };
                ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::RecordShortcut {
                    target_key: base_key.to_string(),
                    display_label: display_label.to_string(),
                }));
                clicked = true;
            }
        });
        ui.add_space(6.0);
    }

    clicked
}

fn draw_button_action_popup(
    ui: &mut egui::Ui,
    btn: CustomizingButton,
    _engine: &Engine,
    config: &mut Config,
    rect: Rect,
    card_rect: Rect,
    customizing_button: &mut Option<CustomizingButton>,
    selected_option: &mut Option<UniversalButtonOption>,
    view_state_id: egui::Id,
) -> bool {
    let _response = ui.allocate_rect(rect, egui::Sense::click());

    let clicked_away = ui.input(|i| i.pointer.any_click())
        && !ui.rect_contains_pointer(rect)
        && !ui.rect_contains_pointer(card_rect);

    let bg = theme::surface_color(ui.ctx());
    let border = theme::border_color(ui.ctx());

    // Draw drop shadow
    let shadow_rect = rect.expand2(vec2(2.0, 3.0)).translate(vec2(0.0, 2.0));
    ui.painter().rect_filled(
        shadow_rect,
        3.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );

    // Outer popup container
    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border));
    theme::draw_tech_corners(ui.painter(), rect, theme::accent_color(ui.ctx()), 6.0);

    let profile = config.profiles.get(&config.active_profile).cloned().unwrap();
    let current_opt = get_button_option(btn, &profile.mappings);

    let mut click_occurred = false;

    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    let salt = match btn {
        CustomizingButton::Thumb => "thumb_button_scroll",
        CustomizingButton::Forward => "forward_button_scroll",
        CustomizingButton::Back => "back_button_scroll",
        CustomizingButton::Top => "top_button_scroll",
        CustomizingButton::Middle => "wheel_button_scroll",
        _ => "button_scroll",
    };

    egui::ScrollArea::vertical()
        .id_salt(salt)
        .show(&mut child_ui, |ui| {
            ui.add_space(4.0);

            // RECOMMENDED header
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("RECOMMENDED")
                        .font(egui::FontId::proportional(9.0))
                        .strong()
                        .color(theme::secondary_text(ui.ctx())),
                );
            });
            ui.add_space(4.0);

            let recommended = btn.recommended_options();

            for &opt in recommended {
                if draw_button_item(ui, btn, opt, current_opt, &profile, customizing_button, selected_option, view_state_id) {
                    click_occurred = true;
                }
            }

            ui.add_space(8.0);

            // OTHER ACTIONS header
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("OTHER ACTIONS")
                        .font(egui::FontId::proportional(9.0))
                        .strong()
                        .color(theme::secondary_text(ui.ctx())),
                );
            });
            ui.add_space(4.0);

            let other = btn.other_options();

            for opt in other {
                if draw_button_item(ui, btn, opt, current_opt, &profile, customizing_button, selected_option, view_state_id) {
                    click_occurred = true;
                }
            }

            ui.add_space(4.0);
        });

    clicked_away && !click_occurred
}
// ── GESTURES POPUP UI ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupView {
    ActionList,
    GesturesConfig,
    RecordShortcut { target_key: String, display_label: String },
}

fn draw_record_shortcut_ui(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    btn: CustomizingButton,
    target_key: String,
    display_label: String,
    rect: Rect,
    card_rect: Rect,
    _customizing_button: &mut Option<CustomizingButton>,
) -> bool {
    let _response = ui.allocate_rect(rect, egui::Sense::click());

    // Check click away
    let clicked_away = ui.input(|i| i.pointer.any_click())
        && !ui.rect_contains_pointer(rect)
        && !ui.rect_contains_pointer(card_rect);

    let bg = theme::surface_color(ui.ctx());
    let border = theme::border_color(ui.ctx());

    // Draw drop shadow
    let shadow_rect = rect.expand2(vec2(2.0, 3.0)).translate(vec2(0.0, 2.0));
    ui.painter().rect_filled(
        shadow_rect,
        3.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );

    // Outer popup container
    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border));
    theme::draw_tech_corners(ui.painter(), rect, theme::accent_color(ui.ctx()), 6.0);

    // Teal Header Bar
    let header_h = 36.0;
    let header_rect = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + header_h));
    ui.painter().rect_filled(
        header_rect,
        egui::Rounding { nw: 2.0, ne: 2.0, sw: 0.0, se: 0.0 },
        Color32::from_rgb(0, 245, 198), // Teal `#00f5c6`
    );

    let mut header_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect.shrink(6.0))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    let view_state_id = ui.id().with(format!("popup_view_for_{:?}", btn));

    // Back Button (←)
    let (back_rect, back_res) = header_ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
    let back_hover_color = if back_res.hovered() {
        header_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(0, 0, 0, 20)
    } else {
        Color32::TRANSPARENT
    };
    header_ui.painter().rect_filled(back_rect, 4.0, back_hover_color);

    let stroke = egui::Stroke::new(1.5, Color32::BLACK);
    let cx = back_rect.center().x;
    let cy = back_rect.center().y;
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy - 4.0)], stroke);
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy + 4.0)], stroke);

    let mut click_occurred = false;

    if back_res.clicked() {
        // Return to parent view
        let next_view = if target_key.contains("_gesture_") || target_key.starts_with("gesture_") {
            PopupView::GesturesConfig
        } else {
            PopupView::ActionList
        };
        ui.ctx().data_mut(|d| d.insert_temp(view_state_id, next_view));
        RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
        click_occurred = true;
    }

    header_ui.add_space(8.0);
    header_ui.label(
        RichText::new("Record Shortcut")
            .font(egui::FontId::proportional(12.0))
            .strong()
            .color(Color32::BLACK),
    );

    // Content area
    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(rect.min.x + 8.0, rect.min.y + header_h + 8.0),
                pos2(rect.max.x - 8.0, rect.max.y - 8.0),
            ))
            .layout(egui::Layout::top_down(egui::Align::Center)),
    );

    content_ui.add_space(8.0);
    content_ui.label(
        RichText::new(format!("Target: {}", display_label))
            .font(egui::FontId::proportional(11.0))
            .strong()
            .color(Color32::WHITE),
    );
    content_ui.add_space(8.0);

    content_ui.add(egui::Label::new(
        RichText::new("Press any key combination on your keyboard.\nPress Enter to save, or Escape to cancel.")
            .font(egui::FontId::proportional(10.0))
            .color(theme::secondary_text(ui.ctx())),
    ));

    // Listen to keyboard keys
    let mut escape_pressed = false;
    let mut enter_pressed = false;
    let mut new_keys_recorded = None;
    
    ui.input(|i| {
        let mods = i.modifiers;
        let has_modifiers = mods.ctrl || mods.shift || mods.alt || mods.mac_cmd;
        let mut parts = Vec::new();
        if mods.ctrl { parts.push("ctrl".to_string()); }
        if mods.shift { parts.push("shift".to_string()); }
        if mods.alt { parts.push("alt".to_string()); }
        if mods.mac_cmd { parts.push("meta".to_string()); }

        for event in &i.events {
            let detected_key = match event {
                egui::Event::Key { key, pressed: true, .. } => Some(*key),
                egui::Event::Copy => Some(egui::Key::C),
                egui::Event::Cut => Some(egui::Key::X),
                egui::Event::Paste(_) => Some(egui::Key::V),
                _ => None,
            };

            if let Some(key) = detected_key {
                if key == egui::Key::Escape && !has_modifiers {
                    escape_pressed = true;
                } else if key == egui::Key::Enter && !has_modifiers {
                    enter_pressed = true;
                } else {
                    let name = egui_key_to_string(key);
                    if !name.is_empty() && name != "ctrl" && name != "shift" && name != "alt" && name != "meta" && name != "tab" {
                        parts.push(name);
                        new_keys_recorded = Some(parts.join("+"));
                    }
                }
            }
        }
    });

    if let Some(keys) = new_keys_recorded {
        RECORDED_KEYS.with(|rk| *rk.borrow_mut() = keys);
    }

    if escape_pressed {
        let next_view = if target_key.contains("_gesture_") || target_key.starts_with("gesture_") {
            PopupView::GesturesConfig
        } else {
            PopupView::ActionList
        };
        ui.ctx().data_mut(|d| d.insert_temp(view_state_id, next_view));
        RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
    } else if enter_pressed {
        let recorded = RECORDED_KEYS.with(|rk| rk.borrow().clone());
        if is_valid_combo(&recorded) {
            let profile_name = &config.active_profile;
            if let Some(profile) = config.profiles.get(profile_name).cloned().or_else(|| config.profiles.get("default").cloned()) {
                let mut mappings = profile.mappings;
                let action_str = format!("custom:{}", recorded);
                
                mappings.insert(target_key.clone(), action_str);
                
                if !target_key.contains("_gesture_") && !target_key.starts_with("gesture_") && target_key != "hscroll_left" && target_key != "hscroll_right" {
                    let (_, gesture_enabled_key, up_k, down_k, left_k, right_k) = get_button_keys(btn);
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                    for dir_key in [up_k, down_k, left_k, right_k] {
                        mappings.insert(dir_key.to_string(), "none".to_string());
                    }
                }

                let engine_bg = engine.clone();
                let profile_name_bg = profile_name.clone();
                std::thread::spawn(move || {
                    engine_bg.update_profile_mappings(&profile_name_bg, mappings);
                });
            }
        }
        let next_view = if target_key.contains("_gesture_") || target_key.starts_with("gesture_") {
            PopupView::GesturesConfig
        } else {
            PopupView::ActionList
        };
        ui.ctx().data_mut(|d| d.insert_temp(view_state_id, next_view));
        RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
    }

    let current_pressed = RECORDED_KEYS.with(|rk| rk.borrow().clone());

    content_ui.add_space(20.0);

    let display_combo = if current_pressed.is_empty() {
        "Press keys...".to_string()
    } else {
        current_pressed.to_uppercase()
    };

    let text_color = if current_pressed.is_empty() {
        theme::muted_text(ui.ctx())
    } else {
        theme::accent_color(ui.ctx())
    };

    // Keystroke combo preview box inside popup content
    let preview_w = content_ui.available_width() - 16.0;
    let (preview_rect, _) = content_ui.allocate_exact_size(vec2(preview_w, 36.0), egui::Sense::hover());
    ui.painter().rect_filled(preview_rect, 2.0, theme::elevated_color(ui.ctx()));
    ui.painter().rect_stroke(preview_rect, 2.0, Stroke::new(1.0, theme::border_color(ui.ctx())));
    ui.painter().text(
        preview_rect.center(),
        egui::Align2::CENTER_CENTER,
        &display_combo,
        egui::FontId::monospace(12.0),
        text_color,
    );

    clicked_away && !click_occurred
}

struct GesturePreset {
    name: &'static str,
    left: &'static str,
    right: &'static str,
    up: &'static str,
    down: &'static str,
    click: &'static str,
}

const GESTURE_PRESETS: &[GesturePreset] = &[
    GesturePreset {
        name: "Custom",
        left: "none",
        right: "none",
        up: "none",
        down: "none",
        click: "none",
    },
    GesturePreset {
        name: "Arrange windows",
        left: "snap_left",
        right: "snap_right",
        up: "maximize_window",
        down: "minimize_window",
        click: "alt_tab",
    },
    GesturePreset {
        name: "Pan",
        left: "pan_left",
        right: "pan_right",
        up: "pan_up",
        down: "pan_down",
        click: "mouse_middle_click",
    },
    GesturePreset {
        name: "Zoom/Rotate",
        left: "rotate_left",
        right: "rotate_right",
        up: "zoom_in",
        down: "zoom_out",
        click: "zoom_reset",
    },
    GesturePreset {
        name: "App navigation",
        left: "alt_tab",
        right: "alt_tab",
        up: "start_menu",
        down: "win_d",
        click: "alt_tab",
    },
    GesturePreset {
        name: "Windows management",
        left: "snap_left",
        right: "snap_right",
        up: "maximize_window",
        down: "win_d",
        click: "alt_tab",
    },
    GesturePreset {
        name: "Media controls",
        left: "prev_track",
        right: "next_track",
        up: "volume_up",
        down: "volume_down",
        click: "play_pause",
    },
    GesturePreset {
        name: "Virtual desktops",
        left: "space_left",
        right: "space_right",
        up: "start_menu",
        down: "win_d",
        click: "task_view",
    },
];

fn action_id_to_slot_display_name(action_id: &str) -> String {
    if action_id.starts_with("custom:") {
        return action_id.strip_prefix("custom:").unwrap().to_uppercase();
    }
    match action_id {
        "none" => "Do nothing".to_string(),
        "snap_left" => "Snap left".to_string(),
        "snap_right" => "Snap right".to_string(),
        "maximize_window" => "Maximize window".to_string(),
        "minimize_window" => "Minimize window".to_string(),
        "alt_tab" => "Switch application".to_string(),
        "pan_left" | "pan_right" | "pan_up" | "pan_down" | "pan" => "Pan".to_string(),
        "mouse_middle_click" => "Middle button".to_string(),
        "rotate_left" | "rotate_right" | "rotate" => "Rotate".to_string(),
        "zoom_in" => "Zoom in".to_string(),
        "zoom_out" => "Zoom out".to_string(),
        "zoom_reset" => "Zoom reset".to_string(),
        "start_menu" => "Start menu".to_string(),
        "win_d" => "Show/hide desktop".to_string(),
        "prev_track" => "Previous".to_string(),
        "next_track" => "Next".to_string(),
        "volume_up" => "Volume up".to_string(),
        "volume_down" => "Volume down".to_string(),
        "play_pause" => "Play/Pause".to_string(),
        "space_left" => "Desktop left".to_string(),
        "space_right" => "Desktop right".to_string(),
        "task_view" => "Task view".to_string(),
        _ => action_id.replace('_', " "),
    }
}

const SLOT_ACTIONS: &[(&str, &str)] = &[
    ("none", "Do nothing"),
    ("snap_left", "Snap left"),
    ("snap_right", "Snap right"),
    ("maximize_window", "Maximize window"),
    ("minimize_window", "Minimize window"),
    ("alt_tab", "Switch application"),
    ("pan", "Pan"),
    ("mouse_middle_click", "Middle button"),
    ("rotate", "Rotate"),
    ("zoom_in", "Zoom in"),
    ("zoom_out", "Zoom out"),
    ("zoom_reset", "Zoom reset"),
    ("start_menu", "Start menu"),
    ("win_d", "Show/hide desktop"),
    ("prev_track", "Previous"),
    ("next_track", "Next"),
    ("volume_up", "Volume up"),
    ("volume_down", "Volume down"),
    ("play_pause", "Play/Pause"),
    ("space_left", "Desktop left"),
    ("space_right", "Desktop right"),
    ("task_view", "Task view"),
    ("custom", "Keyboard Shortcut"),
];

fn resolve_generic_slot_action(action: &str, dir: &str) -> String {
    match action {
        "pan" => match dir {
            "left" => "pan_left".to_string(),
            "right" => "pan_right".to_string(),
            "up" => "pan_up".to_string(),
            _ => "pan_down".to_string(),
        },
        "rotate" => match dir {
            "left" => "rotate_left".to_string(),
            _ => "rotate_right".to_string(),
        },
        _ => action.to_string(),
    }
}

fn get_generic_action_id(action_id: &str) -> &str {
    if action_id.starts_with("pan_") {
        "pan"
    } else if action_id.starts_with("rotate_") {
        "rotate"
    } else {
        action_id
    }
}

fn draw_gesture_config_ui(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    btn: CustomizingButton,
    rect: Rect,
    card_rect: Rect,
    customizing_button: &mut Option<CustomizingButton>,
) -> bool {
    let mut click_occurred = false;

    // Check click away
    let clicked_away = ui.input(|i| i.pointer.any_click())
        && !ui.rect_contains_pointer(rect)
        && !ui.rect_contains_pointer(card_rect);

    let bg = theme::surface_color(ui.ctx());
    let border = theme::border_color(ui.ctx());

    // Draw drop shadow
    let shadow_rect = rect.expand2(vec2(2.0, 3.0)).translate(vec2(0.0, 2.0));
    ui.painter().rect_filled(
        shadow_rect,
        3.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );

    // Outer popup container
    ui.painter().rect_filled(rect, 2.0, bg);
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(1.0, border));
    theme::draw_tech_corners(ui.painter(), rect, theme::accent_color(ui.ctx()), 6.0);

    // Teal Header Bar
    let header_h = 36.0;
    let header_rect = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + header_h));
    ui.painter().rect_filled(
        header_rect,
        egui::Rounding { nw: 2.0, ne: 2.0, sw: 0.0, se: 0.0 },
        Color32::from_rgb(0, 245, 198), // Teal `#00f5c6`
    );

    let profile = config.profiles.get(&config.active_profile).cloned().unwrap();
    let (_, _, up_key, down_key, left_key, right_key) = get_button_keys(btn);
    let click_key = get_button_keys(btn).0;

    let cur_left = profile.mappings.get(left_key).cloned().unwrap_or_else(|| "none".to_string());
    let cur_right = profile.mappings.get(right_key).cloned().unwrap_or_else(|| "none".to_string());
    let cur_up = profile.mappings.get(up_key).cloned().unwrap_or_else(|| "none".to_string());
    let cur_down = profile.mappings.get(down_key).cloned().unwrap_or_else(|| "none".to_string());
    let cur_click = profile.mappings.get(click_key).cloned().unwrap_or_else(|| "none".to_string());

    // Draw elements inside header_rect using child_ui
    let mut header_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect.shrink(6.0))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    // Back Button (←)
    let (back_rect, back_res) = header_ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
    let back_hover_color = if back_res.hovered() {
        header_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(0, 0, 0, 20)
    } else {
        Color32::TRANSPARENT
    };
    header_ui.painter().rect_filled(back_rect, 4.0, back_hover_color);

    let stroke = egui::Stroke::new(1.5, Color32::BLACK);
    let cx = back_rect.center().x;
    let cy = back_rect.center().y;
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy - 4.0)], stroke);
    header_ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy + 4.0)], stroke);

    if back_res.clicked() {
        let view_state_id = ui.id().with(format!("popup_view_for_{:?}", btn));
        ui.ctx().data_mut(|d| d.insert_temp(view_state_id, PopupView::ActionList));
        click_occurred = true;
    }

    header_ui.add_space(4.0);

    // Gestures Icon (circle outline with circle inside)
    let icon_center = header_ui.min_rect().min + vec2(28.0, 12.0);
    ui.painter().circle_filled(icon_center, 6.5, Color32::BLACK);
    ui.painter().circle_filled(icon_center, 2.0, Color32::from_rgb(0, 245, 198));

    header_ui.add_space(12.0);

    // Title: Gestures
    header_ui.label(
        RichText::new("Gestures")
            .font(egui::FontId::proportional(12.0))
            .strong()
            .color(Color32::BLACK),
    );

    // Content container
    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(rect.min.x + 8.0, rect.min.y + header_h + 8.0),
                pos2(rect.max.x - 8.0, rect.max.y - 8.0),
            ))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    // Subheading description text
    content_ui.add(egui::Label::new(
        RichText::new("Choose a preset or select custom to create your own.")
            .font(egui::FontId::proportional(10.0))
            .color(theme::secondary_text(ui.ctx())),
    ));
    content_ui.add_space(8.0);

    // Determine active preset
    let mut active_preset_idx = 0; // Default to Custom
    for (idx, preset) in GESTURE_PRESETS.iter().enumerate().skip(1) {
        if cur_left == preset.left &&
           cur_right == preset.right &&
           cur_up == preset.up &&
           cur_down == preset.down &&
           cur_click == preset.click {
            active_preset_idx = idx;
            break;
        }
    }

    // Preset Selection Dropdown
    let combo_w = content_ui.available_width();
    let combo = egui::ComboBox::from_id_salt(ui.id().with(format!("preset_combo_{:?}", btn)))
        .width(combo_w)
        .selected_text(
            RichText::new(GESTURE_PRESETS[active_preset_idx].name)
                .font(egui::FontId::proportional(11.0))
                .color(Color32::WHITE)
        );

    let res = combo.show_ui(&mut content_ui, |ui| {
        let mut changed = false;
        let mut selected_idx = active_preset_idx;
        for (idx, preset) in GESTURE_PRESETS.iter().enumerate() {
            if ui.selectable_label(idx == active_preset_idx, preset.name).clicked() {
                selected_idx = idx;
                changed = true;
            }
        }
        (changed, selected_idx)
    });

    if let Some((true, new_idx)) = res.inner {
        click_occurred = true;
        let preset = &GESTURE_PRESETS[new_idx];
        let mut mps = profile.mappings.clone();
        mps.insert(left_key.to_string(), preset.left.to_string());
        mps.insert(right_key.to_string(), preset.right.to_string());
        mps.insert(up_key.to_string(), preset.up.to_string());
        mps.insert(down_key.to_string(), preset.down.to_string());
        mps.insert(click_key.to_string(), preset.click.to_string());
        let engine_bg = engine.clone();
        let profile_bg = config.active_profile.clone();
        std::thread::spawn(move || { engine_bg.update_profile_mappings(&profile_bg, mps); });
    }

    content_ui.add_space(8.0);

    // 5 slots card container
    let container_rect = Rect::from_min_size(
        content_ui.cursor().min,
        vec2(content_ui.available_width(), 200.0),
    );

    // Frame background and stroke
    ui.painter().rect_filled(container_rect, 4.0, theme::elevated_color(ui.ctx()));
    ui.painter().rect_stroke(container_rect, 4.0, Stroke::new(1.0, theme::border_color(ui.ctx())));


    let slots = &[
        ("left", "HOLD + MOVE LEFT", left_key, &cur_left),
        ("right", "HOLD + MOVE RIGHT", right_key, &cur_right),
        ("up", "HOLD + MOVE UP", up_key, &cur_up),
        ("down", "HOLD + MOVE DOWN", down_key, &cur_down),
        ("click", "CLICK", click_key, &cur_click),
    ];

    for (i, &(dir, label, key_str, cur_val)) in slots.iter().enumerate() {
        if i > 0 {
            // Draw divider line between slots
            let y = container_rect.min.y + i as f32 * 40.0;
            ui.painter().line_segment(
                [pos2(container_rect.min.x + 4.0, y), pos2(container_rect.max.x - 4.0, y)],
                Stroke::new(0.8, theme::border_color(ui.ctx())),
            );
        }

        let slot_rect = Rect::from_min_max(
            pos2(container_rect.min.x, container_rect.min.y + i as f32 * 40.0),
            pos2(container_rect.max.x, container_rect.min.y + (i + 1) as f32 * 40.0),
        );

        let row_id = ui.id().with(format!("slot_{:?}_{}", btn, dir));
        let response = ui.interact(slot_rect, row_id, egui::Sense::click());
        let is_hovered = response.hovered();

        if is_hovered {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            ui.painter().rect_filled(slot_rect, 0.0, theme::hover_color(ui.ctx()));
        }

        // Left Icon
        let icon_pos = slot_rect.left_center() + vec2(14.0, 0.0);
        let icon_color = theme::primary_text(ui.ctx());
        match dir {
            "left" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
                ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy - 4.0)], stroke);
                ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx - 1.0, cy + 4.0)], stroke);
            }
            "right" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter().line_segment([pos2(cx - 5.0, cy), pos2(cx + 5.0, cy)], stroke);
                ui.painter().line_segment([pos2(cx + 5.0, cy), pos2(cx + 1.0, cy - 4.0)], stroke);
                ui.painter().line_segment([pos2(cx + 5.0, cy), pos2(cx + 1.0, cy + 4.0)], stroke);
            }
            "up" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter().line_segment([pos2(cx, cy - 5.0), pos2(cx, cy + 5.0)], stroke);
                ui.painter().line_segment([pos2(cx, cy - 5.0), pos2(cx - 4.0, cy - 1.0)], stroke);
                ui.painter().line_segment([pos2(cx, cy - 5.0), pos2(cx + 4.0, cy - 1.0)], stroke);
            }
            "down" => {
                let cx = icon_pos.x;
                let cy = icon_pos.y;
                let stroke = Stroke::new(1.5, icon_color);
                ui.painter().line_segment([pos2(cx, cy - 5.0), pos2(cx, cy + 5.0)], stroke);
                ui.painter().line_segment([pos2(cx, cy + 5.0), pos2(cx - 4.0, cy + 1.0)], stroke);
                ui.painter().line_segment([pos2(cx, cy + 5.0), pos2(cx + 4.0, cy + 1.0)], stroke);
            }
            _ => {
                ui.painter().circle_stroke(icon_pos, 4.0, Stroke::new(1.5, icon_color));
            }
        }

        // Text Labels: slot name and current action display name
        let label_pos = slot_rect.left_center() + vec2(28.0, -8.0);
        let action_pos = slot_rect.left_center() + vec2(28.0, 6.0);

        ui.painter().text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(7.5),
            theme::secondary_text(ui.ctx()),
        );

        let disp_name = action_id_to_slot_display_name(cur_val);
        ui.painter().text(
            action_pos,
            egui::Align2::LEFT_CENTER,
            &disp_name,
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );

        // Click handler to open dropdown action selector for this slot
        let generic_action = get_generic_action_id(cur_val);
        let dropdown_id = ui.id().with(format!("dropdown_{:?}_{}", btn, dir));

        // Hidden combobox over the row to handle slot customization
        let mut slot_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(slot_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        // Make the button transparent / frameless
        let widgets = &mut slot_ui.style_mut().visuals.widgets;
        widgets.inactive.bg_fill = Color32::TRANSPARENT;
        widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        widgets.inactive.bg_stroke = Stroke::NONE;
        widgets.hovered.bg_fill = Color32::TRANSPARENT;
        widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
        widgets.hovered.bg_stroke = Stroke::NONE;
        widgets.active.bg_fill = Color32::TRANSPARENT;
        widgets.active.weak_bg_fill = Color32::TRANSPARENT;
        widgets.active.bg_stroke = Stroke::NONE;

        let slot_combo = egui::ComboBox::from_id_salt(dropdown_id)
            .icon(|_, _, _, _, _| {}) // Empty closure to remove default arrow icon
            .width(slot_rect.width());

        let res = slot_combo.show_ui(&mut slot_ui, |ui| {
            let mut changed = false;
            let mut selected_act = generic_action.to_string();
            for &(act_id, act_disp) in SLOT_ACTIONS {
                if ui.selectable_label(generic_action == act_id, act_disp).clicked() {
                    selected_act = act_id.to_string();
                    changed = true;
                }
            }
            (changed, selected_act)
        });

        if let Some((true, act_val)) = res.inner {
            click_occurred = true;
            if act_val == "custom" {
                // Open keyboard recording modal
                ui.ctx().memory_mut(|mem| mem.stop_text_input());
                if dir == "click" {
                    RECORDING_TARGET.with(|r| *r.borrow_mut() = Some(RecordingTarget::Button(btn)));
                } else {
                    RECORDING_TARGET.with(|r| *r.borrow_mut() = Some(RecordingTarget::Gesture(btn, dir.to_string())));
                }
                *customizing_button = None;
            } else {
                let resolved = resolve_generic_slot_action(&act_val, dir);
                let mut mps = profile.mappings.clone();
                mps.insert(key_str.to_string(), resolved);
                let engine_bg = engine.clone();
                let profile_bg = config.active_profile.clone();
                std::thread::spawn(move || { engine_bg.update_profile_mappings(&profile_bg, mps); });
            }
        }
    }

    clicked_away && !click_occurred
}


