use crate::theme;
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::client::EngineClient as Engine;

thread_local! {
    pub static SHOW_ADD_APP_MODAL: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
    pub static APP_SEARCH_QUERY: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    pub static SCANNED_APPS: std::cell::RefCell<Option<Vec<crate::desktop_apps::DesktopApp>>> = const { std::cell::RefCell::new(None) };
    pub static FOCUS_REQUESTED: std::cell::RefCell<bool> = const { std::cell::RefCell::new(false) };
}

pub fn draw_add_app_modal(
    ctx: &egui::Context,
    engine: &Engine,
    config: &mut mouser_engine::config::Config,
) {
    let show_app_modal = SHOW_ADD_APP_MODAL.with(|s| *s.borrow());
    if show_app_modal {
        egui::Area::new(egui::Id::new("add_application_modal"))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let screen_r = ctx.screen_rect();
                // Allocate response to make this Area layer active under pointer and block fallthrough
                let _background_response =
                    ui.allocate_rect(screen_r, egui::Sense::click_and_drag());
                // Dark overlay
                ui.painter().rect_filled(
                    screen_r,
                    0.0,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                );

                let card_w = 480.0;
                let card_h = 520.0_f32.min(screen_r.height() - 40.0);
                let card_rect = Rect::from_center_size(screen_r.center(), vec2(card_w, card_h));

                // Draw premium dark container
                ui.painter()
                    .rect_filled(card_rect, 4.0, Color32::from_rgb(0x16, 0x16, 0x16));
                ui.painter().rect_stroke(
                    card_rect,
                    4.0,
                    Stroke::new(1.0, Color32::from_rgb(0x2d, 0x2d, 0x2d)),
                );
                theme::draw_tech_corners(ui.painter(), card_rect, theme::accent_color(ctx), 8.0);

                let mut modal_ui =
                    ui.new_child(egui::UiBuilder::new().max_rect(card_rect.shrink(20.0)));

                modal_ui.vertical(|ui| {
                    // Title and Close Button
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Add Application Profile")
                                .color(Color32::WHITE)
                                .size(13.0)
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (close_rect, close_res) =
                                ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::click());
                            if close_res.hovered() {
                                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                            }
                            let close_color = if close_res.hovered() {
                                Color32::from_rgb(0, 212, 200)
                            } else {
                                Color32::from_rgb(0x88, 0x88, 0x88)
                            };
                            let ccx = close_rect.center().x;
                            let ccy = close_rect.center().y;
                            ui.painter().line_segment(
                                [pos2(ccx - 4.5, ccy - 4.5), pos2(ccx + 4.5, ccy + 4.5)],
                                Stroke::new(1.5, close_color),
                            );
                            ui.painter().line_segment(
                                [pos2(ccx - 4.5, ccy + 4.5), pos2(ccx + 4.5, ccy - 4.5)],
                                Stroke::new(1.5, close_color),
                            );
                            if close_res.clicked() {
                                SHOW_ADD_APP_MODAL.with(|s| *s.borrow_mut() = false);
                            }
                        });
                    });

                    // Search box
                    let mut query = APP_SEARCH_QUERY.with(|q| q.borrow().clone());
                    ui.horizontal(|ui| {
                        ui.add_space(2.0);
                        let search_response = ui.add(
                            egui::TextEdit::singleline(&mut query)
                                .hint_text("Search applications...")
                                .desired_width(ui.available_width() - 4.0)
                                .margin(egui::vec2(8.0, 6.0)),
                        );

                        // Set focus to the search field automatically on popup
                        let focused = FOCUS_REQUESTED.with(|f| *f.borrow());
                        if !focused {
                            search_response.request_focus();
                            FOCUS_REQUESTED.with(|f| *f.borrow_mut() = true);
                        }
                    });
                    APP_SEARCH_QUERY.with(|q| *q.borrow_mut() = query.clone());

                    ui.add_space(12.0);

                    // Get apps and filter
                    let scanned_opt = SCANNED_APPS.with(|apps| apps.borrow().clone());
                    if let Some(scanned_apps) = scanned_opt {
                        let query_lower = query.to_lowercase();
                        let filtered_apps: Vec<_> = scanned_apps
                            .into_iter()
                            .filter(|app| {
                                app.name.to_lowercase().contains(&query_lower)
                                    || app.exec.to_lowercase().contains(&query_lower)
                            })
                            .collect();

                        if filtered_apps.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(
                                    RichText::new("No applications found. Try a different search term.")
                                        .color(theme::muted_text(ctx))
                                        .size(12.0),
                                );
                            });
                        } else {
                            let list_h = ui.available_height();
                            egui::ScrollArea::vertical()
                                .id_salt("add_app_scroll")
                                .max_height(list_h)
                                .auto_shrink([false; 2])
                                .scroll_bar_visibility(
                                    egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                )
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
                                    for app in filtered_apps {
                                        // Check if already assigned
                                        let mut assigned_profile = None;
                                        if let Some(g_data) =
                                            config.profile_groups.get(&config.active_group)
                                        {
                                            for (pname, pdata) in &g_data.profiles {
                                                if pdata.apps.contains(&app.exec) {
                                                    assigned_profile = Some(pname.clone());
                                                    break;
                                                }
                                            }
                                        }

                                        let is_assigned = assigned_profile.is_some();
                                        let item_w = ui.available_width();
                                        let item_h = 42.0;

                                        let (item_rect, item_res) = if is_assigned {
                                            ui.allocate_exact_size(
                                                vec2(item_w, item_h),
                                                egui::Sense::hover(),
                                            )
                                        } else {
                                            ui.allocate_exact_size(
                                                vec2(item_w, item_h),
                                                egui::Sense::click(),
                                            )
                                        };

                                        if item_res.hovered() && !is_assigned {
                                            ui.output_mut(|o| {
                                                o.cursor_icon = egui::CursorIcon::PointingHand
                                            });
                                        }

                                        let is_hovered = item_res.hovered();
                                        let bg_color = if is_assigned {
                                            Color32::from_rgb(0x12, 0x12, 0x12)
                                        } else if is_hovered {
                                            Color32::from_rgb(0x22, 0x22, 0x22)
                                        } else {
                                            Color32::from_rgb(0x1c, 0x1c, 0x1c)
                                        };
                                        ui.painter().rect_filled(item_rect, 2.0, bg_color);
                                        ui.painter().rect_stroke(
                                            item_rect,
                                            2.0,
                                            Stroke::new(1.0, Color32::from_rgb(0x2d, 0x2d, 0x2d)),
                                        );

                                        // Draw app icon or fallback badge
                                        let icon_center =
                                            pos2(item_rect.min.x + 24.0, item_rect.center().y);
                                        let fallback_color = if is_assigned {
                                            Color32::from_rgb(0x66, 0x66, 0x66)
                                        } else {
                                            theme::accent_color(ctx)
                                        };
                                        crate::icon_loader::draw_app_icon(
                                            ui,
                                            &app.icon,
                                            icon_center,
                                            9.0,
                                            fallback_color,
                                        );

                                        // Text details
                                        let name_color = if is_assigned {
                                            Color32::from_rgb(0x88, 0x88, 0x88)
                                        } else {
                                            Color32::WHITE
                                        };
                                        ui.painter().text(
                                            pos2(
                                                item_rect.min.x + 45.0,
                                                item_rect.center().y - 6.0,
                                            ),
                                            egui::Align2::LEFT_CENTER,
                                            &app.name,
                                            egui::FontId::proportional(12.0),
                                            name_color,
                                        );

                                        let (exec_text, text_color) =
                                            if let Some(pname) = &assigned_profile {
                                                (
                                                    format!("Mapped to profile: {}", pname),
                                                    Color32::from_rgb(0xe1, 0x1d, 0x48),
                                                )
                                            } else {
                                                (
                                                    format!("Executable: {}", app.exec),
                                                    theme::muted_text(ctx),
                                                )
                                            };

                                        ui.painter().text(
                                            pos2(
                                                item_rect.min.x + 45.0,
                                                item_rect.center().y + 8.0,
                                            ),
                                            egui::Align2::LEFT_CENTER,
                                            exec_text,
                                            egui::FontId::proportional(9.5),
                                            text_color,
                                        );

                                        if !is_assigned && item_res.clicked() {
                                            // 1. Add Profile to config
                                            engine.add_profile(&app.name);
                                            // 2. Update its app bindings
                                            engine.update_app_bindings(&app.name, &app.exec);
                                            // 3. Set profile icon
                                            engine.set_profile_icon(&app.name, &app.icon);
                                            // 4. Switch to it as the active profile
                                            engine.select_profile(&app.name);
                                            // 5. Instantly refresh local config state from daemon
                                            *config = engine.get_config();
                                            config.active_app_profile = app.name.clone();
                                            // 6. Hide modal
                                            SHOW_ADD_APP_MODAL.with(|s| *s.borrow_mut() = false);
                                        }
                                    }
                                });
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label("Loading applications...");
                        });
                    }
                });
            });
    }
}
