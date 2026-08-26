use crate::theme;
use crate::ActiveView;
use eframe::egui;

pub fn show(
    ui: &mut egui::Ui,
    _active_view: &mut ActiveView,
    unifying_connected: bool,
    bolt_connected: bool,
    lang: &str,
) {
    ui.vertical_centered(|ui| {
        // Build dynamic list of active rows
        let mut rows = Vec::new();
        if unifying_connected {
            rows.push("unifying");
        }
        if bolt_connected {
            rows.push("bolt");
        }
        rows.push("bluetooth");

        let num_rows = rows.len();

        // Vertical spacing for optical centering
        let available_height = ui.available_height();
        let content_height = 110.0 + num_rows as f32 * 70.0; // Approximate height of header + spacing + card
        let top_space = (available_height - content_height) / 2.0 - 40.0;

        if top_space > 0.0 {
            ui.add_space(top_space);
        }

        // 1. Header Title
        ui.add(
            egui::Label::new(
                egui::RichText::new(crate::translation::tr("select_conn", lang))
                    .color(theme::primary_text(ui.ctx()))
                    .size(22.0)
                    .strong(),
            )
            .selectable(false),
        );

        ui.add_space(35.0);

        // 2. Selection Card dimensions
        let card_width = 460.0;
        let row_height = 70.0;
        let card_height = num_rows as f32 * row_height;

        // Allocate a rect for the entire card
        let (card_rect, _) =
            ui.allocate_exact_size(egui::vec2(card_width, card_height), egui::Sense::hover());

        // Setup interaction/responses BEFORE drawing anything
        let mut row_responses = Vec::new();
        for (i, row_name) in rows.iter().enumerate() {
            let rect_row = egui::Rect::from_min_max(
                egui::pos2(card_rect.min.x, card_rect.min.y + i as f32 * row_height),
                egui::pos2(
                    card_rect.max.x,
                    card_rect.min.y + (i + 1) as f32 * row_height,
                ),
            );

            let id_str = format!("row_{}", row_name);
            let response = ui.interact(rect_row, egui::Id::new(id_str), egui::Sense::click());
            row_responses.push(response);
        }

        let painter = ui.painter();

        // Draw Card Background
        painter.rect_filled(card_rect, 2.0, theme::card_bg(ui.ctx()));

        // Draw hover highlights and click pulse transitions
        for (i, response) in row_responses.iter().enumerate() {
            let rect_row = egui::Rect::from_min_max(
                egui::pos2(card_rect.min.x, card_rect.min.y + i as f32 * row_height),
                egui::pos2(
                    card_rect.max.x,
                    card_rect.min.y + (i + 1) as f32 * row_height,
                ),
            );

            let rounding = if num_rows == 1 {
                egui::Rounding::same(2.0)
            } else if i == 0 {
                egui::Rounding {
                    nw: 2.0,
                    ne: 2.0,
                    sw: 0.0,
                    se: 0.0,
                }
            } else if i == num_rows - 1 {
                egui::Rounding {
                    nw: 0.0,
                    ne: 0.0,
                    sw: 2.0,
                    se: 2.0,
                }
            } else {
                egui::Rounding::ZERO
            };

            // Draw hover highlight
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);

                let hover_bg = if ui.visuals().dark_mode {
                    egui::Color32::from_rgb(0x18, 0x18, 0x18)
                } else {
                    egui::Color32::from_rgb(0xf3, 0xf4, 0xf6)
                };
                painter.rect_filled(rect_row, rounding, hover_bg);
            }

            // Click pulse transition (Item 11) - Option A (Trigger instantly)
            let click_t = ui
                .ctx()
                .animate_bool(response.id.with("click"), response.clicked());
            if click_t > 0.0 {
                let click_overlay = egui::Color32::from_rgba_unmultiplied(
                    theme::accent_color(ui.ctx()).r(),
                    theme::accent_color(ui.ctx()).g(),
                    theme::accent_color(ui.ctx()).b(),
                    (20.0 * click_t) as u8,
                );
                painter.rect_filled(rect_row, rounding, click_overlay);
            }
        }

        // Draw Outer Border
        painter.rect_stroke(
            card_rect,
            2.0,
            egui::Stroke::new(1.0, theme::card_border(ui.ctx())),
        );

        // Tech corner brackets glowing on hover
        let is_any_hovered = ui.rect_contains_pointer(card_rect);
        let t = ui.ctx().animate_bool(
            egui::Id::new("select_connection_tech_corners"),
            is_any_hovered,
        );
        let corner_color = theme::lerp_color(
            theme::card_border(ui.ctx()),
            theme::accent_color(ui.ctx()),
            t,
        );
        theme::draw_tech_corners(painter, card_rect, corner_color, 8.0);

        // Draw Separator Lines between rows
        if num_rows > 1 {
            for i in 1..num_rows {
                let separator_y = card_rect.min.y + i as f32 * row_height;
                painter.line_segment(
                    [
                        egui::pos2(card_rect.min.x, separator_y),
                        egui::pos2(card_rect.max.x, separator_y),
                    ],
                    egui::Stroke::new(1.0, theme::card_border(ui.ctx())),
                );
            }
        }

        // Click Actions
        for (i, row_type) in rows.iter().enumerate() {
            if row_responses[i].clicked() {
                match *row_type {
                    "unifying" => {
                        // Unifying receiver action placeholder
                    }
                    "bolt" => {
                        // Bolt receiver action placeholder
                    }
                    "bluetooth" => {
                        open_bluetooth_settings();
                    }
                    _ => {}
                }
            }
        }

        // Render contents of each row
        for (i, row_type) in rows.iter().enumerate() {
            let rect_row = egui::Rect::from_min_max(
                egui::pos2(card_rect.min.x, card_rect.min.y + i as f32 * row_height),
                egui::pos2(
                    card_rect.max.x,
                    card_rect.min.y + (i + 1) as f32 * row_height,
                ),
            );

            // 1. Render Icon (vertically centered, 24px left margin)
            let icon_center = egui::pos2(rect_row.min.x + 24.0 + 16.0, rect_row.center().y);
            match *row_type {
                "unifying" => {
                    draw_unifying_icon(ui, icon_center);
                }
                "bolt" => {
                    draw_bolt_icon(ui, icon_center);
                }
                "bluetooth" => {
                    draw_bluetooth_icon(ui, icon_center);
                }
                _ => {}
            }

            // 2. Render Text (mathematically centered vertically in rect_row)
            let has_subtitle = matches!(*row_type, "unifying" | "bolt");

            let text_left = rect_row.min.x + 24.0 + 32.0 + 16.0; // 72.0px from left
            let text_right = rect_row.max.x - 24.0 - 12.0 - 16.0; // Keep space for chevron
            let text_height = if has_subtitle { 34.0 } else { 18.0 };
            let text_rect = egui::Rect::from_min_max(
                egui::pos2(text_left, rect_row.center().y - text_height / 2.0),
                egui::pos2(text_right, rect_row.center().y + text_height / 2.0),
            );

            let mut text_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(text_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            text_ui.spacing_mut().item_spacing.y = 2.0;

            match *row_type {
                "unifying" => {
                    text_ui.add(
                        egui::Label::new(
                            egui::RichText::new(crate::translation::tr("unifying", lang))
                                .color(theme::primary_text(ui.ctx()))
                                .size(15.0)
                                .strong(),
                        )
                        .selectable(false),
                    );
                    text_ui.add(
                        egui::Label::new(
                            egui::RichText::new("MX Master 3, MX Keys")
                                .color(theme::muted_text(ui.ctx()))
                                .size(11.0),
                        )
                        .selectable(false),
                    );
                }
                "bolt" => {
                    text_ui.add(
                        egui::Label::new(
                            egui::RichText::new(crate::translation::tr("bolt", lang))
                                .color(theme::primary_text(ui.ctx()))
                                .size(15.0)
                                .strong(),
                        )
                        .selectable(false),
                    );
                    text_ui.add(
                        egui::Label::new(
                            egui::RichText::new("MX Master 3S, MX Keys Mini")
                                .color(theme::muted_text(ui.ctx()))
                                .size(11.0),
                        )
                        .selectable(false),
                    );
                }
                "bluetooth" => {
                    text_ui.add(
                        egui::Label::new(
                            egui::RichText::new(crate::translation::tr("bluetooth", lang))
                                .color(theme::primary_text(ui.ctx()))
                                .size(15.0)
                                .strong(),
                        )
                        .selectable(false),
                    );
                }
                _ => {}
            }

            // 3. Render Chevron (vertically centered, 24px right margin)
            let chevron_rect = egui::Rect::from_center_size(
                egui::pos2(rect_row.max.x - 24.0 - 6.0, rect_row.center().y),
                egui::vec2(12.0, 24.0),
            );
            let mut chevron_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(chevron_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            chevron_ui.add(
                egui::Label::new(
                    egui::RichText::new("›")
                        .color(theme::muted_text(ui.ctx()))
                        .size(22.0),
                )
                .selectable(false),
            );
        }

        // 3. Footer text pushed to the bottom
        let footer_space = (ui.available_height() - 40.0).max(20.0);
        ui.add_space(footer_space);
        ui.add(
            egui::Label::new(
                egui::RichText::new(crate::translation::tr("footer_desc", lang))
                    .color(theme::COLOR_FOOTER_ORANGE)
                    .size(12.0),
            )
            .selectable(false),
        );
    });
}

fn draw_unifying_icon(ui: &egui::Ui, center: egui::Pos2) {
    let painter = ui.painter();

    // USB connector (silver metal part at top)
    let metal_rect =
        egui::Rect::from_center_size(center - egui::vec2(0.0, 4.0), egui::vec2(10.0, 8.0));
    painter.rect_filled(metal_rect, 1.0, egui::Color32::from_rgb(0xb0, 0xb0, 0xb0));

    // Two small slots in metal connector
    let hole1 = egui::Rect::from_center_size(center - egui::vec2(2.5, 5.5), egui::vec2(2.0, 2.5));
    let hole2 = egui::Rect::from_center_size(center - egui::vec2(-2.5, 5.5), egui::vec2(2.0, 2.5));
    painter.rect_filled(hole1, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));
    painter.rect_filled(hole2, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));

    // USB plastic collar (bottom base)
    let base_rect =
        egui::Rect::from_center_size(center + egui::vec2(0.0, 3.5), egui::vec2(15.0, 7.0));
    painter.rect_filled(base_rect, 1.5, egui::Color32::from_rgb(0x38, 0x38, 0x38));

    // Unifying orange dot logo
    painter.circle_filled(
        center + egui::vec2(0.0, 3.5),
        1.8,
        egui::Color32::from_rgb(0xff, 0x98, 0x00),
    );
}

fn draw_bolt_icon(ui: &egui::Ui, center: egui::Pos2) {
    let painter = ui.painter();

    // USB connector (silver metal part at top)
    let metal_rect =
        egui::Rect::from_center_size(center - egui::vec2(0.0, 4.0), egui::vec2(10.0, 8.0));
    painter.rect_filled(metal_rect, 1.0, egui::Color32::from_rgb(0xb0, 0xb0, 0xb0));

    // Two small slots in metal connector
    let hole1 = egui::Rect::from_center_size(center - egui::vec2(2.5, 5.5), egui::vec2(2.0, 2.5));
    let hole2 = egui::Rect::from_center_size(center - egui::vec2(-2.5, 5.5), egui::vec2(2.0, 2.5));
    painter.rect_filled(hole1, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));
    painter.rect_filled(hole2, 0.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));

    // USB plastic collar (bottom base)
    let base_rect =
        egui::Rect::from_center_size(center + egui::vec2(0.0, 3.5), egui::vec2(15.0, 7.0));
    painter.rect_filled(base_rect, 1.5, egui::Color32::from_rgb(0x38, 0x38, 0x38));

    // Bolt neon yellow-green lightning logo
    let bolt_center = center + egui::vec2(0.0, 3.5);
    let bolt_color = egui::Color32::from_rgb(0xcc, 0xff, 0x00);

    let pts = vec![
        bolt_center + egui::vec2(0.5, -2.5),
        bolt_center + egui::vec2(1.5, -0.5),
        bolt_center + egui::vec2(0.2, -0.5),
        bolt_center + egui::vec2(0.8, 2.5),
        bolt_center + egui::vec2(-0.8, 0.5),
        bolt_center + egui::vec2(0.2, 0.5),
    ];

    painter.add(egui::Shape::convex_polygon(
        pts,
        bolt_color,
        egui::Stroke::NONE,
    ));
}

fn draw_bluetooth_icon(ui: &egui::Ui, center: egui::Pos2) {
    let painter = ui.painter();

    // Blue background circle + white bluetooth rune
    painter.circle_filled(center, 12.0, egui::Color32::from_rgb(0x00, 0x7a, 0xff));
    crate::widgets::draw_bluetooth_rune(painter, center, 12.0, egui::Color32::WHITE);
}

fn open_bluetooth_settings() {
    // 1. Try specific Linux Bluetooth settings/pairing commands
    let bluetooth_commands = [
        ("gnome-control-center", &["bluetooth"] as &[&str]),
        ("systemsettings", &["kcm_bluetooth"]),
        ("kcmshell5", &["kcm_bluetooth"]),
        ("kcmshell6", &["kcm_bluetooth"]),
        ("blueman-manager", &[]),
        ("blueman-assistant", &[]),
        ("bluetooth-wizard", &[]),
    ];

    for (cmd, args) in bluetooth_commands.iter() {
        if std::process::Command::new(cmd).args(*args).spawn().is_ok() {
            return;
        }
    }

    // 2. Fallback to generic settings managers if no bluetooth-specific tool exists
    let fallback_settings = [
        ("xfce4-settings-manager", &[] as &[&str]),
        ("gnome-control-center", &[]),
        ("systemsettings", &[]),
        ("mate-control-center", &[]),
        ("cinnamon-settings", &[]),
    ];

    for (cmd, args) in fallback_settings.iter() {
        if std::process::Command::new(cmd).args(*args).spawn().is_ok() {
            return;
        }
    }
}
