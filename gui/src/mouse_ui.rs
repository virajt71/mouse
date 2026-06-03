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

    if gesture_enabled {
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
    let active_profile_name = config.active_profile.clone();
    let profile_label = format!("MX Master 3 ({})", active_profile_name);
    header_ui.add(
        egui::Label::new(
            RichText::new(&profile_label)
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
    let status_rect = Rect::from_min_max(
        pos2(sidebar_rect.min.x + 20.0, sidebar_rect.max.y - 54.0),
        pos2(sidebar_rect.min.x + 105.0, sidebar_rect.max.y - 20.0),
    );
    ui.painter()
        .rect_filled(status_rect, 2.0, theme::surface_color(ctx));
    ui.painter()
        .rect_stroke(status_rect, 2.0, Stroke::new(1.0, theme::border_color(ctx)));

    let b_center = pos2(status_rect.min.x + 22.0, status_rect.center().y);
    let level = battery_pct.parse::<f32>().unwrap_or(100.0) / 100.0;
    let b_rect = Rect::from_center_size(b_center, vec2(20.0, 10.0));
    crate::empty_state::draw_battery_widget(ui.painter(), b_rect, level);
 
    let conn_center = pos2(status_rect.max.x - 22.0, status_rect.center().y);
    crate::empty_state::draw_connection_icon_mini(ui.painter(), conn_center, conn_type);

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
                let mut current_pressed = RECORDED_KEYS.with(|rk| rk.borrow().clone());
                
                modal_ui.input(|i| {
                    let mods = i.modifiers;
                    let mut parts = Vec::new();
                    if mods.ctrl { parts.push("ctrl".to_string()); }
                    if mods.shift { parts.push("shift".to_string()); }
                    if mods.alt { parts.push("alt".to_string()); }
                    if mods.mac_cmd { parts.push("meta".to_string()); }

                    for event in &i.events {
                        if let egui::Event::Key { key, pressed: true, .. } = event {
                            if *key == egui::Key::Escape {
                                RECORDING_TARGET.with(|r| *r.borrow_mut() = None);
                                RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
                            } else if *key == egui::Key::Enter {
                                let recorded = RECORDED_KEYS.with(|rk| rk.borrow().clone());
                                if !recorded.is_empty() {
                                    // Save the shortcut
                                    let mut mappings = config.profiles.get(&config.active_profile).cloned().unwrap().mappings;
                                    let action_str = format!("custom:{}", recorded);
                                    match &target {
                                        RecordingTarget::Button(b) => {
                                            let (base_key, gesture_enabled_key, _, _, _, _) = get_button_keys(*b);
                                            mappings.insert(base_key.to_string(), action_str);
                                            mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
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
                                    engine.update_profile_mappings(&config.active_profile, mappings);
                                }
                                RECORDING_TARGET.with(|r| *r.borrow_mut() = None);
                                RECORDED_KEYS.with(|rk| rk.borrow_mut().clear());
                            } else {
                                let name = egui_key_to_string(*key);
                                if !name.is_empty() && name != "ctrl" && name != "shift" && name != "alt" && name != "meta" {
                                    parts.push(name);
                                    current_pressed = parts.join("+");
                                }
                            }
                        }
                    }
                });

                if !current_pressed.is_empty() {
                    RECORDED_KEYS.with(|rk| *rk.borrow_mut() = current_pressed.clone());
                }

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
        
        let primary_label = if action == ButtonAction::Keystroke {
            if mapping_str.starts_with("custom:") {
                mapping_str.strip_prefix("custom:").unwrap().to_uppercase()
            } else {
                "Keystroke".to_string()
            }
        } else {
            action.display_name().to_string()
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
    if let Some((btn, card_pos, card_rect, current_action)) = popup_to_draw {
        // Double width/height if Gestures is active to fit grid
        let show_gestures = current_action == ButtonAction::Gestures && matches!(btn, CustomizingButton::Middle | CustomizingButton::Forward | CustomizingButton::Back | CustomizingButton::Thumb);

        let popup_w = if show_gestures { 260.0 } else { 170.0 };
        let popup_h = if show_gestures { 250.0 } else { 190.0 };

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

        let mut selected_action = None;
        let clicked_away = draw_action_popup(
            ui,
            engine,
            config,
            btn,
            popup_rect,
            card_rect,
            current_action,
            &mut selected_action,
        );

        if let Some(action) = selected_action {
            let mut mappings = profile.mappings;
            let (base_key, gesture_enabled_key, _, _, _, _) = get_button_keys(btn);

            match action {
                ButtonAction::Disabled => {
                    mappings.insert(base_key.to_string(), "none".to_string());
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                }
                ButtonAction::MiddleClick => {
                    mappings.insert(base_key.to_string(), "mouse_middle_click".to_string());
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                }
                ButtonAction::ModeShift => {
                    mappings.insert(base_key.to_string(), "switch_scroll_mode".to_string());
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                }
                ButtonAction::Forward => {
                    mappings.insert(base_key.to_string(), "mouse_forward_click".to_string());
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                }
                ButtonAction::Back => {
                    mappings.insert(base_key.to_string(), "mouse_back_click".to_string());
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                }
                ButtonAction::HorizontalScroll => {
                    mappings.insert(base_key.to_string(), "hscroll".to_string());
                    mappings.insert(gesture_enabled_key.to_string(), "false".to_string());
                }
                ButtonAction::Gestures => {
                    mappings.insert(gesture_enabled_key.to_string(), "true".to_string());
                }
                ButtonAction::Keystroke => {
                    // Open modal
                    RECORDING_TARGET.with(|r| *r.borrow_mut() = Some(RecordingTarget::Button(btn)));
                }
            }

            engine.update_profile_mappings(&config.active_profile, mappings);
            *customizing_button = None;
        } else if clicked_away {
            *customizing_button = None;
        }
    }
}

// ── POINT & SCROLL TAB ──────────────────────────────────────────────────────
fn show_point_scroll_tab(ui: &mut egui::Ui, engine: &Engine, config: &mut Config) {
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
            ui.add_space(20.0);

            // SmartShift toggle
            ui.horizontal(|ui| {
                let mut smart_shift_enabled = config.settings.smart_shift_enabled;
                if ui.checkbox(&mut smart_shift_enabled, "SmartShift (Auto-switching Wheel)").changed() {
                    config.settings.smart_shift_enabled = smart_shift_enabled;
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

            let mut invert_h = config.settings.invert_hscroll;
            if ui.checkbox(&mut invert_h, "Invert Horizontal Scroll").changed() {
                config.settings.invert_hscroll = invert_h;
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
        });
    });
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

fn draw_action_popup(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    btn: CustomizingButton,
    rect: Rect,
    card_rect: Rect,
    current_action: ButtonAction,
    selected_action: &mut Option<ButtonAction>,
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

    let supports_gestures = matches!(btn, CustomizingButton::Middle | CustomizingButton::Forward | CustomizingButton::Back | CustomizingButton::Thumb);

    if current_action == ButtonAction::Gestures && supports_gestures {
        // Render 4-way gesture configuration grid directly in the popup
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink(10.0))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );

        child_ui.horizontal(|ui| {
            if ui.button("← Actions").clicked() {
                *selected_action = Some(ButtonAction::Disabled); // Go back to simple actions
            }
            ui.label(RichText::new("GESTURES GRID").strong().color(Color32::WHITE));
        });
        child_ui.add_space(10.0);

        let directions = &["up", "down", "left", "right"];
        let (_, _, up_key, down_key, left_key, right_key) = get_button_keys(btn);

        for &dir in directions {
            let key_str = match dir {
                "up" => up_key,
                "down" => down_key,
                "left" => left_key,
                _ => right_key,
            };

            let cur_val = profile.mappings.get(key_str).cloned().unwrap_or_else(|| "none".to_string());

            child_ui.horizontal(|ui| {
                ui.label(format!("Swipe {}:", dir.to_uppercase()));

                let mut sel_val = cur_val.clone();
                let combo_label = if sel_val.starts_with("custom:") {
                    sel_val.strip_prefix("custom:").unwrap().to_uppercase()
                } else {
                    sel_val.replace('_', " ")
                };

                let combo = egui::ComboBox::from_id_salt(format!("{}_{}", btn as usize, dir))
                    .selected_text(RichText::new(&combo_label).size(10.0));

                let res = combo.show_ui(ui, |ui| {
                    let mut changed = false;
                    let actions_list = &[
                        ("none", "Disabled"),
                        ("alt_tab", "Alt Tab"),
                        ("alt_shift_tab", "Alt Shift Tab"),
                        ("copy", "Copy"),
                        ("paste", "Paste"),
                        ("volume_up", "Volume Up"),
                        ("volume_down", "Volume Down"),
                        ("volume_mute", "Volume Mute"),
                        ("space_left", "Space Left"),
                        ("space_right", "Space Right"),
                        ("page_up", "Page Up"),
                        ("page_down", "Page Down"),
                        ("cycle_dpi", "Cycle DPI"),
                        ("custom", "Keyboard Shortcut"),
                    ];

                    for &(act_id, act_disp) in actions_list {
                        if ui.selectable_label(sel_val == act_id, act_disp).clicked() {
                            if act_id == "custom" {
                                RECORDING_TARGET.with(|r| *r.borrow_mut() = Some(RecordingTarget::Gesture(btn, dir.to_string())));
                            } else {
                                sel_val = act_id.to_string();
                                changed = true;
                            }
                        }
                    }
                    changed
                });

                if let Some(true) = res.inner {
                    let mut mps = profile.mappings.clone();
                    mps.insert(key_str.to_string(), sel_val);
                    engine.update_profile_mappings(&config.active_profile, mps);
                }
            });
            child_ui.add_space(8.0);
        }
    } else {
        // Render normal simple actions
        let item_h = 22.0;
        let margin = 6.0;

        let mut actions = vec![
            ButtonAction::MiddleClick,
            ButtonAction::ModeShift,
            ButtonAction::Forward,
            ButtonAction::Back,
            ButtonAction::HorizontalScroll,
            ButtonAction::Keystroke,
            ButtonAction::Disabled,
        ];

        if supports_gestures {
            actions.insert(5, ButtonAction::Gestures);
        }

        let mut click_occurred = false;

        for (i, &act) in actions.iter().enumerate() {
            let item_rect = Rect::from_min_max(
                pos2(rect.min.x + 1.0, rect.min.y + margin + i as f32 * item_h),
                pos2(
                    rect.max.x - 1.0,
                    rect.min.y + margin + (i + 1) as f32 * item_h,
                ),
            );

            let item_id = ui.id().with("pop_item").with(act);
            let item_res = ui.interact(item_rect, item_id, egui::Sense::click());

            let is_selected = act == current_action;
            let is_hovered = item_res.hovered();

            if is_hovered {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                let hover_bg = theme::hover_color(ui.ctx());
                ui.painter().rect_filled(item_rect, 0.0, hover_bg);
            }

            let text_color = if is_selected {
                theme::accent_color(ui.ctx())
            } else if is_hovered {
                theme::primary_text(ui.ctx())
            } else {
                theme::secondary_text(ui.ctx())
            };

            let label_text = act.display_name();
            let galley = ui.fonts(|f| {
                f.layout_job(egui::text::LayoutJob::simple_singleline(
                    label_text.to_string(),
                    egui::FontId::proportional(11.0),
                    text_color,
                ))
            });
            let text_y = item_rect.center().y - galley.size().y / 2.0;
            ui.painter()
                .galley(pos2(item_rect.min.x + 12.0, text_y), galley, text_color);

            if item_res.clicked() {
                *selected_action = Some(act);
                click_occurred = true;
            }
        }

        return clicked_away && !click_occurred;
    }

    clicked_away
}
