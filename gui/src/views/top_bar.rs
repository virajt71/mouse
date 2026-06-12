use crate::theme;
use crate::ActiveView;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, ctx: &egui::Context, active_view: &mut ActiveView, lang: &str) {
    let top_bar_height = 45.0;

    let (rect, _response) = ui.allocate_at_least(
        egui::vec2(ui.available_width(), top_bar_height),
        egui::Sense::hover(),
    );

    // Background
    let top_bar_bg = if ui.visuals().dark_mode {
        theme::COLOR_TOP_BAR
    } else {
        egui::Color32::from_rgb(0xe5, 0xe7, 0xeb)
    };
    ui.painter().rect_filled(rect, 0.0, top_bar_bg);

    // Draw 1px bottom border on top bar
    let border_y = rect.max.y;
    ui.painter().line_segment(
        [
            egui::pos2(rect.min.x, border_y),
            egui::pos2(rect.max.x, border_y),
        ],
        egui::Stroke::new(1.0, theme::border_color(ctx)),
    );

    // Create a horizontal layout for the content directly covering the top bar rect
    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    content_ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

    // Left padding
    content_ui.add_space(20.0);

    let mut back_clicked = false;
    let mut back_hovered = false;

    // App Title or Back button depending on active view
    if *active_view == ActiveView::SelectConnectionType || *active_view == ActiveView::Settings {
        // Back button (Custom drawn crisp '←' to avoid font fallback issues)
        let (back_rect, back_res) =
            content_ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
        let back_hover_color = if back_res.hovered() {
            back_hovered = true;
            content_ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            if content_ui.visuals().dark_mode {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15)
            }
        } else {
            egui::Color32::TRANSPARENT
        };
        content_ui
            .painter()
            .rect_filled(back_rect, 4.0, back_hover_color);

        let stroke = egui::Stroke::new(1.5, theme::primary_text(ctx));
        let cx = back_rect.center().x;
        let cy = back_rect.center().y;

        // Draw crisp back arrow
        content_ui
            .painter()
            .line_segment([egui::pos2(cx - 7.0, cy), egui::pos2(cx + 7.0, cy)], stroke);
        content_ui.painter().line_segment(
            [egui::pos2(cx - 7.0, cy), egui::pos2(cx - 2.0, cy - 5.0)],
            stroke,
        );
        content_ui.painter().line_segment(
            [egui::pos2(cx - 7.0, cy), egui::pos2(cx - 2.0, cy + 5.0)],
            stroke,
        );

        if back_res.clicked() {
            back_clicked = true;
        }
    } else {
        let title_text = "MOUSER-RS";
        let mut job = egui::text::LayoutJob::default();
        let mut chars = title_text.char_indices().peekable();
        while let Some((idx, _)) = chars.next() {
            let next_idx = chars
                .peek()
                .map(|(n_idx, _)| *n_idx)
                .unwrap_or(title_text.len());
            job.append(
                &title_text[idx..next_idx],
                1.5, // spaced caps effect
                egui::TextFormat {
                    font_id: egui::FontId::proportional(15.0),
                    color: theme::secondary_text(ctx),
                    ..Default::default()
                },
            );
        }
        content_ui.add(egui::Label::new(job).selectable(false));
    }

    // Right-aligned elements container
    content_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // Zero out item spacing to manage gaps explicitly
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        // Right padding
        ui.add_space(16.0);

        // Close button (Custom drawn crisp 'X' to avoid font fallback box issues)
        let (close_rect, close_res) =
            ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
        let close_hover = if close_res.hovered() {
            if ui.visuals().dark_mode {
                egui::Color32::from_rgba_unmultiplied(255, 0, 0, 40)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 0, 0, 30)
            }
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect_filled(close_rect, 4.0, close_hover);

        let stroke = egui::Stroke::new(1.5, theme::primary_text(ctx));
        let cx = close_rect.center().x;
        let cy = close_rect.center().y;
        let d = 5.0;
        ui.painter().line_segment(
            [egui::pos2(cx - d, cy - d), egui::pos2(cx + d, cy + d)],
            stroke,
        );
        ui.painter().line_segment(
            [egui::pos2(cx - d, cy + d), egui::pos2(cx + d, cy - d)],
            stroke,
        );

        if close_res.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // Gap
        ui.add_space(6.0);

        // Minimize button (Custom drawn centered horizontal dash)
        let (min_rect, min_res) =
            ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
        let min_hover = if min_res.hovered() {
            if ui.visuals().dark_mode {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
            } else {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15)
            }
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect_filled(min_rect, 4.0, min_hover);

        let center_y = min_rect.center().y;
        let line_left = min_rect.center().x - 6.0;
        let line_right = min_rect.center().x + 6.0;
        ui.painter().line_segment(
            [
                egui::pos2(line_left, center_y),
                egui::pos2(line_right, center_y),
            ],
            stroke,
        );

        if min_res.clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        let mut settings_hovered = false;
        let mut add_hovered = false;
        let mut add_clicked = false;

        // Render remaining buttons only in EmptyState view
        if *active_view == ActiveView::EmptyState {
            // Gap
            ui.add_space(6.0);

            // Settings button (Custom vector gear, no emojis)
            let (settings_rect, settings_res) =
                ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
            let hover_color = if settings_res.hovered() {
                settings_hovered = true;
                if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
                } else {
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 15)
                }
            } else {
                egui::Color32::TRANSPARENT
            };
            ui.painter().rect_filled(settings_rect, 4.0, hover_color);

            let gear_color = theme::primary_text(ui.ctx());
            draw_settings_gear_icon(ui, settings_rect, gear_color);

            if settings_res.clicked() {
                *active_view = ActiveView::Settings;
            }

            // Gap
            ui.add_space(16.0);

            // Divider
            let (d_rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 16.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(d_rect, 0.0, theme::divider_color(ui.ctx()));

            // Gap
            ui.add_space(16.0);

            // ADD DEVICE button (Increased affordance & height)
            let (add_rect, add_res) =
                ui.allocate_exact_size(egui::vec2(110.0, 34.0), egui::Sense::click());

            if add_res.hovered() {
                add_hovered = true;
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }

            let add_bg = if add_res.hovered() {
                let a = theme::accent_dim_color(ui.ctx());
                egui::Color32::from_rgba_unmultiplied(a.r(), a.g(), a.b(), 76) // 30% alpha of accent dim
            } else {
                if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, 4)
                } else {
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 2)
                }
            };
            let add_stroke = if add_res.hovered() {
                egui::Stroke::new(1.5, theme::accent_color(ui.ctx()))
            } else {
                egui::Stroke::new(1.5, theme::divider_color(ui.ctx()))
            };
            ui.painter().rect_filled(add_rect, 2.0, add_bg);
            ui.painter().rect_stroke(add_rect, 2.0, add_stroke);

            // Draw "+ ADD DEVICE" text using a LayoutJob to prevent text selection and ensure perfect clickability
            let mut job = egui::text::LayoutJob::default();
            job.append(
                "+  ",
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(14.0),
                    color: theme::accent_color(ui.ctx()),
                    ..Default::default()
                },
            );
            let add_device_text = crate::translation::tr("add_device", lang);
            job.append(
                add_device_text,
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(11.0),
                    color: theme::primary_text(ui.ctx()),
                    ..Default::default()
                },
            );
            let galley = ui.fonts(|f| f.layout_job(job));
            let text_pos = add_rect.center() - galley.size() / 2.0 - egui::vec2(0.0, 1.5);
            ui.painter()
                .galley(text_pos, galley, theme::primary_text(ui.ctx()));

            if add_res.clicked() {
                add_clicked = true;
            }
        }

        // Apply state changes outside of response blocks
        if add_clicked {
            *active_view = ActiveView::SelectConnectionType;
        }

        // Check for window drag when primary button is pressed on the top bar, but NOT on any of the buttons
        let screen_rect = ctx.screen_rect();
        let top_bar_rect = egui::Rect::from_min_max(
            screen_rect.min,
            egui::pos2(screen_rect.max.x, screen_rect.min.y + top_bar_height),
        );
        if ui.rect_contains_pointer(top_bar_rect) {
            let is_button_hovered = close_res.hovered()
                || min_res.hovered()
                || (*active_view == ActiveView::EmptyState && (settings_hovered || add_hovered))
                || ((*active_view == ActiveView::SelectConnectionType
                    || *active_view == ActiveView::Settings)
                    && back_hovered);

            if !is_button_hovered && ui.input(|i| i.pointer.primary_pressed()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
        }
    });

    if back_clicked {
        *active_view = ActiveView::EmptyState;
    }
}

fn draw_settings_gear_icon(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let center = rect.center();
    let r_out = 6.5; // Outer radius of the gear body
    let r_in = 4.0; // Inner radius of the gear body
    let r_hole = 2.0; // Center hole radius
    let num_teeth = 8;

    // Draw outer body outline & filled area
    painter.circle_stroke(center, r_out, egui::Stroke::new(1.5, color));
    painter.circle_filled(center, r_out, color);

    // Draw center hole to cut out the inner area
    let bg_color = theme::app_bg(ui.ctx());
    painter.circle_filled(center, r_hole, bg_color);

    // Draw teeth/spokes
    let stroke = egui::Stroke::new(2.2, color);
    for i in 0..num_teeth {
        let angle = (i as f32) * std::f32::consts::TAU / (num_teeth as f32);
        let direction = egui::vec2(angle.cos(), angle.sin());
        let p_start = center + direction * r_in;
        let p_end = center + direction * (r_out + 2.0);
        painter.line_segment([p_start, p_end], stroke);
    }

    // Re-draw center hole to keep it clean (in case spokes overlap it)
    painter.circle_filled(center, r_hole, bg_color);
}
