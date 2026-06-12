pub mod mappings;
pub mod popups;
pub mod sidebar;
pub mod tabs;

use crate::widgets::draw_status_pill;
use crate::{theme, ActiveView};
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::config::Config;
use mouser_engine::Engine;

use mappings::{egui_key_to_string, get_button_keys, is_valid_combo, CustomizingButton};
pub use popups::{
    draw_add_app_modal, RecordingTarget, APP_SEARCH_QUERY, FOCUS_REQUESTED, RECORDED_KEYS,
    RECORDING_TARGET, SCANNED_APPS, SHOW_ADD_APP_MODAL,
};
use sidebar::draw_sidebar;
pub use sidebar::SidebarTab;
use tabs::{show_buttons_tab, show_flow_tab, show_point_scroll_tab, show_profiles_settings_tab};

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    engine: &Engine,
    config: &mut Config,
    device_texture: &egui::TextureHandle,
    active_view: &mut ActiveView,
    customizing_button: &mut Option<CustomizingButton>,
    customization_tab: &mut SidebarTab,
    conn_type: &str,
    battery_pct: &str,
    is_connected: bool,
    customizing_device_name: &str,
) {
    let rect = ui.max_rect();
    let bg = theme::app_bg(ctx);

    let layout_key = crate::app::get_layout_key_from_name(customizing_device_name);
    let is_keyboard = layout_key.contains("keys") || layout_key.contains("mechanical");

    // Fill background
    ui.painter().rect_filled(rect, 0.0, bg);

    // ── 1. Thin Window Title Bar (decorations) ───────────────────────────────
    let title_bar_height = 24.0;
    let title_bar_rect =
        Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + title_bar_height));

    let title_bar_bg = Color32::from_rgb(0x11, 0x11, 0x11);
    ui.painter().rect_filled(title_bar_rect, 0.0, title_bar_bg);

    let mut title_bar_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    title_bar_ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
    title_bar_ui.add_space(8.0);

    // Close button in thin title bar
    let (close_rect, close_res) =
        title_bar_ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
    let close_hover = if close_res.hovered() {
        Color32::from_rgba_unmultiplied(255, 0, 0, 40)
    } else {
        Color32::TRANSPARENT
    };
    title_bar_ui
        .painter()
        .rect_filled(close_rect, 2.0, close_hover);
    let cr_stroke = Stroke::new(1.0, Color32::from_rgb(0xaa, 0xaa, 0xaa));
    let ccx = close_rect.center().x;
    let ccy = close_rect.center().y;
    title_bar_ui.painter().line_segment(
        [pos2(ccx - 4.0, ccy - 4.0), pos2(ccx + 4.0, ccy + 4.0)],
        cr_stroke,
    );
    title_bar_ui.painter().line_segment(
        [pos2(ccx - 4.0, ccy + 4.0), pos2(ccx + 4.0, ccy - 4.0)],
        cr_stroke,
    );
    if close_res.clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    title_bar_ui.add_space(4.0);

    // Minimize button in thin title bar
    let (min_rect, min_res) =
        title_bar_ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
    let min_hover = if min_res.hovered() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 20)
    } else {
        Color32::TRANSPARENT
    };
    title_bar_ui.painter().rect_filled(min_rect, 2.0, min_hover);
    let mcx = min_rect.center().x;
    let mcy = min_rect.center().y;
    title_bar_ui
        .painter()
        .line_segment([pos2(mcx - 5.0, mcy), pos2(mcx + 5.0, mcy)], cr_stroke);
    if min_res.clicked() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }

    // Drag behavior for title bar
    let is_title_button_hovered = close_res.hovered() || min_res.hovered();
    if !is_title_button_hovered
        && ui.rect_contains_pointer(title_bar_rect)
        && ui.input(|i| i.pointer.primary_pressed())
    {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    // ── 2. Content Header Row ────────────────────────────────────────────────
    let header_height = 45.0;
    let header_rect = Rect::from_min_max(
        pos2(rect.min.x, title_bar_rect.max.y),
        pos2(rect.max.x, title_bar_rect.max.y + header_height),
    );

    let header_bg = Color32::from_rgb(0x11, 0x11, 0x11);
    ui.painter().rect_filled(header_rect, 0.0, header_bg);

    let mut header_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(header_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    header_ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

    let mut back_clicked = false;
    header_ui.add_space(20.0);

    let (back_rect, back_res) =
        header_ui.allocate_exact_size(vec2(32.0, 32.0), egui::Sense::click());
    let back_hover_color = if back_res.hovered() {
        header_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        Color32::from_rgba_unmultiplied(255, 255, 255, 20)
    } else {
        Color32::TRANSPARENT
    };
    header_ui
        .painter()
        .rect_filled(back_rect, 2.0, back_hover_color);

    let arrow_stroke = Stroke::new(1.5, Color32::WHITE);
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

    // Profile title in header
    let profile_label = customizing_device_name;
    header_ui.add(
        egui::Label::new(
            RichText::new(profile_label)
                .color(Color32::WHITE)
                .size(17.0)
                .strong(),
        )
        .selectable(false),
    );

    let show_switcher = if is_keyboard {
        *customization_tab == SidebarTab::Buttons
    } else {
        *customization_tab == SidebarTab::Buttons
            || *customization_tab == SidebarTab::PointAndScroll
    };

    // Auto-close modal if we switch tabs away from Buttons or PointAndScroll
    if !show_switcher {
        SHOW_ADD_APP_MODAL.with(|s| *s.borrow_mut() = false);
    }

    if show_switcher {
        header_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            ui.add_space(20.0);

            // 3. Plus Icon
            let (plus_rect, plus_res) =
                ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());
            if plus_res.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
            let plus_color = if plus_res.hovered() {
                Color32::from_rgb(0, 212, 200)
            } else {
                Color32::WHITE
            };
            let pc = plus_rect.center();
            ui.painter().line_segment(
                [pos2(pc.x - 7.0, pc.y), pos2(pc.x + 7.0, pc.y)],
                Stroke::new(2.0, plus_color),
            );
            ui.painter().line_segment(
                [pos2(pc.x, pc.y - 7.0), pos2(pc.x, pc.y + 7.0)],
                Stroke::new(2.0, plus_color),
            );

            if plus_res.clicked() {
                SHOW_ADD_APP_MODAL.with(|s| *s.borrow_mut() = true);
                APP_SEARCH_QUERY.with(|q| q.borrow_mut().clear());
                FOCUS_REQUESTED.with(|f| *f.borrow_mut() = false);
                SCANNED_APPS.with(|apps| {
                    *apps.borrow_mut() = Some(crate::desktop_apps::scan_all_applications());
                });
            }

            // Gap ~14px
            ui.add_space(14.0);

            // Get custom profiles sorted alphabetically from active group
            let mut custom_profiles: Vec<String> = Vec::new();
            if let Some(group) = config.profile_groups.get(&config.active_group) {
                custom_profiles = group
                    .profiles
                    .keys()
                    .filter(|k| *k != "global")
                    .cloned()
                    .collect();
            }
            custom_profiles.sort_by_key(|a| a.to_lowercase());

            // In right-to-left layout, drawing in reverse order preserves left-to-right alphabetical sequence
            for p_name in custom_profiles.iter().rev() {
                let is_active = config.active_app_profile == *p_name;
                let is_brave = p_name.to_lowercase().contains("brave");

                let (p_rect, p_res) =
                    ui.allocate_exact_size(vec2(23.0, 23.0), egui::Sense::click());
                if p_res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }

                let mut delete_clicked = false;

                if is_active {
                    // Delete/Close button geometry
                    let delete_rect = Rect::from_center_size(
                        pos2(p_rect.max.x - 3.0, p_rect.min.y + 3.0),
                        vec2(12.0, 12.0),
                    );
                    let delete_id = ui.make_persistent_id(format!("del_prof_{}", p_name));
                    let delete_res = ui.interact(delete_rect, delete_id, egui::Sense::click());

                    if delete_res.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }

                    if delete_res.clicked() {
                        engine.delete_profile(p_name);
                        delete_clicked = true;
                    }

                    let pc = p_rect.center();

                    if is_brave {
                        let sc = pc;
                        let shield_pts = vec![
                            pos2(sc.x - 10.0, sc.y - 10.0),
                            pos2(sc.x + 10.0, sc.y - 10.0),
                            pos2(sc.x + 10.0, sc.y + 1.0),
                            pos2(sc.x + 5.0, sc.y + 8.0),
                            pos2(sc.x, sc.y + 11.5),
                            pos2(sc.x - 5.0, sc.y + 8.0),
                            pos2(sc.x - 10.0, sc.y + 1.0),
                        ];

                        let shield_fill = if p_res.hovered() {
                            Color32::from_rgb(251, 146, 60)
                        } else {
                            Color32::from_rgb(249, 115, 22)
                        };
                        ui.painter().add(egui::Shape::convex_polygon(
                            shield_pts,
                            shield_fill,
                            Stroke::NONE,
                        ));

                        let lion_stroke = Stroke::new(1.2, Color32::WHITE);
                        ui.painter().line_segment(
                            [pos2(sc.x, sc.y - 2.0), pos2(sc.x, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 2.0, sc.y + 3.0), pos2(sc.x + 2.0, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 4.0, sc.y - 4.0), pos2(sc.x, sc.y - 2.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x + 4.0, sc.y - 4.0), pos2(sc.x, sc.y - 2.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 5.0, sc.y - 1.0), pos2(sc.x - 3.0, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x + 5.0, sc.y - 1.0), pos2(sc.x + 3.0, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 4.0, sc.y - 7.0), pos2(sc.x - 2.0, sc.y - 5.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x + 4.0, sc.y - 7.0), pos2(sc.x + 2.0, sc.y - 5.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x, sc.y - 7.0), pos2(sc.x, sc.y - 4.0)],
                            lion_stroke,
                        );
                    } else {
                        let initial = p_name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();
                        let circle_color = if is_active {
                            Color32::from_rgb(0x1a, 0x1a, 0x1a)
                        } else if p_res.hovered() {
                            Color32::from_rgb(0x2a, 0x2a, 0x2a)
                        } else {
                            Color32::from_rgb(0x22, 0x22, 0x22)
                        };
                        let border_color = if is_active {
                            Color32::from_rgb(0, 212, 200)
                        } else if p_res.hovered() {
                            Color32::from_rgb(0x88, 0x88, 0x88)
                        } else {
                            Color32::from_rgb(0x44, 0x44, 0x44)
                        };

                        ui.painter()
                            .circle(pc, 11.5, circle_color, Stroke::new(1.0, border_color));

                        let text_color = if is_active {
                            Color32::from_rgb(0, 212, 200)
                        } else {
                            Color32::WHITE
                        };
                        ui.painter().text(
                            pos2(pc.x, pc.y - 0.5),
                            egui::Align2::CENTER_CENTER,
                            initial,
                            egui::FontId::proportional(11.0),
                            text_color,
                        );
                    }

                    // Draw close/delete button overlay (only on active profile hover/interact)
                    let is_profile_hovered = p_res.hovered() || delete_res.hovered();
                    let del_circle_color = if delete_res.hovered() {
                        Color32::from_rgb(239, 68, 68) // Bright red
                    } else if is_profile_hovered {
                        Color32::from_rgb(185, 28, 28) // Muted red
                    } else {
                        Color32::from_rgb(63, 63, 70) // Gray
                    };

                    ui.painter()
                        .circle_filled(delete_rect.center(), 5.0, del_circle_color);
                    let cross_stroke = Stroke::new(1.0, Color32::WHITE);
                    let dc = delete_rect.center();
                    ui.painter().line_segment(
                        [pos2(dc.x - 2.0, dc.y - 2.0), pos2(dc.x + 2.0, dc.y + 2.0)],
                        cross_stroke,
                    );
                    ui.painter().line_segment(
                        [pos2(dc.x - 2.0, dc.y + 2.0), pos2(dc.x + 2.0, dc.y - 2.0)],
                        cross_stroke,
                    );

                    let bar_y = pc.y + 11.5 + 4.0;
                    let bar_left = pc.x - 11.5;
                    let bar_right = pc.x + 11.5;
                    ui.painter().line_segment(
                        [pos2(bar_left, bar_y), pos2(bar_right, bar_y)],
                        Stroke::new(2.0, Color32::from_rgb(0, 212, 200)),
                    );
                } else {
                    let pc = p_rect.center();

                    if is_brave {
                        let sc = pc;
                        let shield_pts = vec![
                            pos2(sc.x - 10.0, sc.y - 10.0),
                            pos2(sc.x + 10.0, sc.y - 10.0),
                            pos2(sc.x + 10.0, sc.y + 1.0),
                            pos2(sc.x + 5.0, sc.y + 8.0),
                            pos2(sc.x, sc.y + 11.5),
                            pos2(sc.x - 5.0, sc.y + 8.0),
                            pos2(sc.x - 10.0, sc.y + 1.0),
                        ];

                        let shield_fill = if p_res.hovered() {
                            Color32::from_rgb(251, 146, 60)
                        } else {
                            Color32::from_rgb(249, 115, 22)
                        };
                        ui.painter().add(egui::Shape::convex_polygon(
                            shield_pts,
                            shield_fill,
                            Stroke::NONE,
                        ));

                        let lion_stroke = Stroke::new(1.2, Color32::WHITE);
                        ui.painter().line_segment(
                            [pos2(sc.x, sc.y - 2.0), pos2(sc.x, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 2.0, sc.y + 3.0), pos2(sc.x + 2.0, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 4.0, sc.y - 4.0), pos2(sc.x, sc.y - 2.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x + 4.0, sc.y - 4.0), pos2(sc.x, sc.y - 2.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 5.0, sc.y - 1.0), pos2(sc.x - 3.0, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x + 5.0, sc.y - 1.0), pos2(sc.x + 3.0, sc.y + 3.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x - 4.0, sc.y - 7.0), pos2(sc.x - 2.0, sc.y - 5.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x + 4.0, sc.y - 7.0), pos2(sc.x + 2.0, sc.y - 5.0)],
                            lion_stroke,
                        );
                        ui.painter().line_segment(
                            [pos2(sc.x, sc.y - 7.0), pos2(sc.x, sc.y - 4.0)],
                            lion_stroke,
                        );
                    } else {
                        let initial = p_name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string();
                        let circle_color = if p_res.hovered() {
                            Color32::from_rgb(0x2a, 0x2a, 0x2a)
                        } else {
                            Color32::from_rgb(0x22, 0x22, 0x22)
                        };
                        let border_color = if p_res.hovered() {
                            Color32::from_rgb(0x88, 0x88, 0x88)
                        } else {
                            Color32::from_rgb(0x44, 0x44, 0x44)
                        };

                        ui.painter()
                            .circle(pc, 11.5, circle_color, Stroke::new(1.0, border_color));

                        ui.painter().text(
                            pos2(pc.x, pc.y - 0.5),
                            egui::Align2::CENTER_CENTER,
                            initial,
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                    }
                }

                if p_res.clicked() && !delete_clicked {
                    engine.select_profile(p_name);
                    config.active_app_profile = p_name.clone();
                }

                ui.add_space(14.0);
            }

            // 1. 2x2 Grid Icon
            let is_default_active = config.active_app_profile == "global";
            let (grid_rect, grid_res) =
                ui.allocate_exact_size(vec2(23.0, 23.0), egui::Sense::click());
            if grid_res.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }

            if grid_res.clicked() {
                engine.select_profile("global");
                config.active_app_profile = "global".to_string();
            }

            let gc = grid_rect.center();
            let grid_color = if is_default_active {
                Color32::from_rgb(0, 212, 200)
            } else if grid_res.hovered() {
                Color32::from_rgb(200, 200, 200)
            } else {
                Color32::WHITE
            };

            let sq_size = 8.0;
            let sq_half = sq_size / 2.0;
            let gap = 3.0;
            let r = 1.5;

            let draw_sq = |painter: &egui::Painter, center_pos: egui::Pos2| {
                let rect = Rect::from_center_size(center_pos, vec2(sq_size, sq_size));
                painter.rect_filled(rect, r, grid_color);
            };

            draw_sq(
                ui.painter(),
                pos2(gc.x - sq_half - gap / 2.0, gc.y - sq_half - gap / 2.0),
            );
            draw_sq(
                ui.painter(),
                pos2(gc.x + sq_half + gap / 2.0, gc.y - sq_half - gap / 2.0),
            );
            draw_sq(
                ui.painter(),
                pos2(gc.x - sq_half - gap / 2.0, gc.y + sq_half + gap / 2.0),
            );
            draw_sq(
                ui.painter(),
                pos2(gc.x + sq_half + gap / 2.0, gc.y + sq_half + gap / 2.0),
            );

            // Active State: Underline
            if is_default_active {
                let icon_w = 2.0 * sq_size + gap;
                let bar_y = gc.y + sq_size + gap / 2.0 + 4.0;
                let bar_left = gc.x - icon_w / 2.0;
                let bar_right = gc.x + icon_w / 2.0;
                ui.painter().line_segment(
                    [pos2(bar_left, bar_y), pos2(bar_right, bar_y)],
                    Stroke::new(2.0, Color32::from_rgb(0, 212, 200)),
                );
            }
        });
    }

    if back_clicked {
        *active_view = ActiveView::EmptyState;
        *customizing_button = None;
        return;
    }

    // ── 3. Sidebar (Vertical Navigation) ─────────────────────────────────────
    let sidebar_rect = Rect::from_min_max(
        pos2(rect.min.x, header_rect.max.y),
        pos2(rect.min.x + 240.0, rect.max.y),
    );



    draw_sidebar(
        ui,
        sidebar_rect,
        customization_tab,
        customizing_button,
        is_keyboard,
    );

    // ── 4. Bottom-Left Status Pill ───────────────────────────────────────────
    let status_w = if is_connected { 85.0 } else { 110.0 };
    let status_rect = Rect::from_min_max(
        pos2(sidebar_rect.min.x + 20.0, sidebar_rect.max.y - 54.0),
        pos2(
            sidebar_rect.min.x + 20.0 + status_w,
            sidebar_rect.max.y - 20.0,
        ),
    );
    draw_status_pill(
        ui,
        status_rect,
        is_connected,
        battery_pct,
        conn_type,
        &config.settings.language,
        true,
    );

    // ── 5. Main Content Canvas ───────────────────────────────────────────────
    let canvas_rect = Rect::from_min_max(pos2(sidebar_rect.max.x, header_rect.max.y), rect.max);
    let mut canvas_ui = ui.new_child(egui::UiBuilder::new().max_rect(canvas_rect));

    if is_keyboard {
        match *customization_tab {
            SidebarTab::Buttons => {
                tabs::show_keyboard_keys_tab(
                    &mut canvas_ui,
                    engine,
                    config,
                    device_texture,
                    customizing_button,
                );
            }
            SidebarTab::PointAndScroll => {
                tabs::show_keyboard_backlighting_tab(&mut canvas_ui, engine, config, device_texture);
            }
            SidebarTab::Flow => {
                tabs::show_keyboard_easy_switch_tab(&mut canvas_ui, engine, config);
            }
            SidebarTab::Settings => {
                show_profiles_settings_tab(&mut canvas_ui, engine, config, true);
            }
        }
    } else {
        match *customization_tab {
            SidebarTab::Buttons => {
                show_buttons_tab(
                    &mut canvas_ui,
                    engine,
                    config,
                    device_texture,
                    customizing_button,
                );
            }
            SidebarTab::PointAndScroll => {
                show_point_scroll_tab(&mut canvas_ui, engine, config, device_texture);
            }
            SidebarTab::Flow => {
                show_flow_tab(&mut canvas_ui, engine, config);
            }
            SidebarTab::Settings => {
                show_profiles_settings_tab(&mut canvas_ui, engine, config, false);
            }
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
                        let mut event_parts = parts.clone();
                        let detected_key = match event {
                            egui::Event::Key { key, pressed: true, .. } => Some(*key),
                            egui::Event::Copy => {
                                if !event_parts.contains(&"ctrl".to_string()) && !event_parts.contains(&"meta".to_string()) {
                                    if cfg!(target_os = "macos") || mods.mac_cmd {
                                        event_parts.push("meta".to_string());
                                    } else {
                                        event_parts.push("ctrl".to_string());
                                    }
                                }
                                Some(egui::Key::C)
                            }
                            egui::Event::Cut => {
                                if !event_parts.contains(&"ctrl".to_string()) && !event_parts.contains(&"meta".to_string()) {
                                    if cfg!(target_os = "macos") || mods.mac_cmd {
                                        event_parts.push("meta".to_string());
                                    } else {
                                        event_parts.push("ctrl".to_string());
                                    }
                                }
                                Some(egui::Key::X)
                            }
                            egui::Event::Paste(_) => {
                                if !event_parts.contains(&"ctrl".to_string()) && !event_parts.contains(&"meta".to_string()) {
                                    if cfg!(target_os = "macos") || mods.mac_cmd {
                                        event_parts.push("meta".to_string());
                                    } else {
                                        event_parts.push("ctrl".to_string());
                                    }
                                }
                                Some(egui::Key::V)
                            }
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
                                    event_parts.push(name);
                                    new_keys_recorded = Some(event_parts.join("+"));
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
                        let profile_name = &config.active_app_profile;
                        if let Some(profile) = config.get_profile(profile_name).or_else(|| config.get_profile("global")) {
                            let mut mappings = profile.mappings.clone();
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

    // ── 7. Linux Application Selector Modal ──────────────────────────────────
    draw_add_app_modal(ctx, engine, config);
}
