use mouser_engine::updater::Updater;
use mouser_engine::config::Config;
use mouser_engine::Engine;
use crate::translation::tr;
use crate::theme;
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};

pub fn show(ui: &mut egui::Ui, ctx: &egui::Context, config: &mut Config, engine: &Engine, updater: &Updater) {
    let avail_w = ui.available_width();
    let avail_h = ui.available_height();
    let h_pad = (avail_w * 0.04).max(24.0);
    let v_pad = 20.0;

    // ── Page title ──
    ui.add_space(v_pad);
    ui.horizontal(|ui| {
        ui.add_space(h_pad);
        render_spaced_header(
            ui,
            tr("settings_title", &config.settings.language),
            20.0,
            theme::primary_text(ui.ctx()),
        );
    });
    ui.add_space(16.0);

    // ── 2-column grid layout (no scroll) ──
    // Left col: Updates + Language   |   Right col: Theme + Profiles
    let gap = 16.0;
    let col_w = ((avail_w - h_pad * 2.0 - gap) / 2.0).max(280.0);
    let content_h = avail_h - v_pad - 40.0; // remaining height below title

    ui.horizontal(|ui| {
        ui.add_space(h_pad);

        // ── Left column ──
        ui.vertical(|ui| {
            ui.set_width(col_w);
            ui.set_height(content_h);

            // SECTION 1: SOFTWARE UPDATES
            render_section_updates(ui, config, engine, updater);
            ui.add_space(gap);

            // SECTION 2: LANGUAGE
            render_section_language(ui, config, engine);
            ui.add_space(gap);

            // SECTION 3: THEME
            render_section_theme(ui, ctx, config, engine);
        });

        ui.add_space(gap);

        // ── Right column ──
        ui.vertical(|ui| {
            ui.set_width(col_w);
            ui.set_height(content_h);

            // // SECTION 3: THEME
            // render_section_theme(ui, ctx, config, engine);
            // ui.add_space(gap);

            // SECTION 4: PROFILES
            render_section_profiles(ui, config, engine);
        });
    });
}

/// Draws a section card background: surface-colored rounded rect with a thin border.
/// Returns the inner `InnerResponse` so callers can extract values.
fn section_card<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let bg = theme::surface_color(ui.ctx());
    let border = theme::border_color(ui.ctx());

    let res = egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .rounding(2.0)
        .inner_margin(egui::Margin::symmetric(20.0, 16.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui)
        });

    let is_hovered = ui.rect_contains_pointer(res.response.rect);
    let t = ui
        .ctx()
        .animate_bool(res.response.id.with("tech_corners"), is_hovered);

    let border_color = theme::border_color(ui.ctx());
    let glow_color = theme::accent_color(ui.ctx());
    let color = theme::lerp_color(border_color, glow_color, t);

    theme::draw_tech_corners(ui.painter(), res.response.rect, color, 8.0);

    res
}

fn render_section_updates(ui: &mut egui::Ui, config: &mut Config, engine: &Engine, updater: &Updater) {
    section_card(ui, |ui| {
        // ── Section header ──
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            draw_refresh_icon(ui, icon_rect, theme::accent_color(ui.ctx()));
            ui.add_space(6.0);
            render_spaced_header(
                ui,
                tr("updates_label", &config.settings.language),
                14.0,
                theme::primary_text(ui.ctx()),
            );
        });

        ui.add_space(4.0);
        ui.add(egui::Label::new(
            RichText::new(tr("updates_desc", &config.settings.language))
                .color(theme::muted_text(ui.ctx()))
                .size(12.0),
        ));

        ui.add_space(12.0);

        // ── Toggle row ──
        ui.horizontal(|ui| {
            ui.add(egui::Label::new(
                RichText::new(tr("auto_update_toggle", &config.settings.language))
                    .color(theme::secondary_text(ui.ctx()))
                    .size(13.5),
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let toggle_w = 44.0;
                let toggle_h = 24.0;
                let (rect, response) =
                    ui.allocate_exact_size(vec2(toggle_w, toggle_h), egui::Sense::click());

                let was_auto = config.settings.install_updates;
                if response.clicked() {
                    config.settings.install_updates = !config.settings.install_updates;
                    let _ = config.save();
                    engine.reload_config();

                    if config.settings.install_updates && !was_auto {
                        let status = updater.status.lock().unwrap().clone();
                        if let mouser_engine::updater::UpdateStatus::Available {
                            version,
                            bin_url,
                            sha_url,
                            bin_name,
                        } = status
                        {
                            updater.start_download_and_install(
                                ui.ctx().clone(),
                                version,
                                bin_url,
                                sha_url,
                                bin_name,
                            );
                        } else if let mouser_engine::updater::UpdateStatus::Idle
                        | mouser_engine::updater::UpdateStatus::Failed(_) = status
                        {
                            updater.check_for_updates(ui.ctx().clone(), config.settings.install_updates);
                        }
                    }
                }

                if response.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }

                let t = ui.ctx().animate_bool(response.id, config.settings.install_updates);

                let inactive_fill = if ui.visuals().dark_mode {
                    Color32::from_rgb(0x2a, 0x2a, 0x2a)
                } else {
                    Color32::from_rgb(0xdc, 0xdc, 0xd8)
                };

                let fill_color = theme::lerp_color(inactive_fill, theme::accent_color(ui.ctx()), t);
                ui.painter().rect_filled(rect, 2.0, fill_color);

                let knob_size = toggle_h - 4.0;
                let knob_x =
                    rect.left() + knob_size / 2.0 + 2.0 + t * (rect.width() - knob_size - 4.0);
                let knob_center = pos2(knob_x, rect.center().y);
                let knob_rect = Rect::from_center_size(knob_center, vec2(knob_size, knob_size));
                ui.painter().rect_filled(knob_rect, 1.0, Color32::WHITE);
            });
        });

        // ── Manual check button (when auto-update is OFF) ──
        if !config.settings.install_updates {
            ui.add_space(10.0);
            let status = updater.status.lock().unwrap().clone();
            let is_busy = matches!(
                status,
                mouser_engine::updater::UpdateStatus::Checking
                    | mouser_engine::updater::UpdateStatus::Downloading { .. }
                    | mouser_engine::updater::UpdateStatus::Verifying
                    | mouser_engine::updater::UpdateStatus::Installing
            );
            let btn = ui.add_enabled(
                !is_busy,
                egui::Button::new(
                    RichText::new(if is_busy {
                        "Checking..."
                    } else {
                        "Check for Updates"
                    })
                    .size(12.5),
                )
                .fill(theme::elevated_color(ui.ctx()))
                .stroke(Stroke::new(1.0, theme::border_color(ui.ctx())))
                .rounding(2.0),
            );
            if btn.clicked() {
                updater.check_for_updates(ui.ctx().clone(), false);
            }
            if btn.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
        }

        // ── Divider ──
        let status = updater.status.lock().unwrap().clone();
        if status != mouser_engine::updater::UpdateStatus::Idle {
            ui.add_space(12.0);
            let divider_rect = ui.allocate_space(vec2(ui.available_width(), 1.0)).1;
            ui.painter()
                .rect_filled(divider_rect, 0.0, theme::border_color(ui.ctx()));
            ui.add_space(10.0);

            // ── Status row ──
            match status {
                mouser_engine::updater::UpdateStatus::Checking => {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("●")
                                .color(theme::accent_color(ui.ctx()))
                                .size(10.0),
                        ));
                        ui.add(egui::Label::new(
                            RichText::new("Checking for updates...")
                                .color(theme::muted_text(ui.ctx()))
                                .size(12.5),
                        ));
                    });
                }
                mouser_engine::updater::UpdateStatus::UpToDate => {
                    let current_version = env!("CARGO_PKG_VERSION");
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("✓")
                                .color(theme::accent_color(ui.ctx()))
                                .size(13.0)
                                .strong(),
                        ));
                        ui.add(egui::Label::new(
                            RichText::new(format!("Up to date — v{}", current_version))
                                .color(theme::muted_text(ui.ctx()))
                                .size(12.5),
                        ));
                    });
                }
                mouser_engine::updater::UpdateStatus::Available {
                    version,
                    bin_url,
                    sha_url,
                    bin_name,
                } => {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("⬆")
                                .color(theme::accent_color(ui.ctx()))
                                .size(13.0),
                        ));
                        ui.add(egui::Label::new(
                            RichText::new(format!("Version {} available", version))
                                .color(theme::primary_text(ui.ctx()))
                                .size(12.5)
                                .strong(),
                        ));
                    });
                    if !config.settings.install_updates {
                        ui.add_space(6.0);
                        let btn = ui.add(
                            egui::Button::new(
                                RichText::new("Download & Install").strong().size(12.5),
                            )
                            .fill(theme::accent_color(ui.ctx()))
                            .stroke(Stroke::NONE)
                            .rounding(2.0),
                        );
                        if btn.clicked() {
                            updater.start_download_and_install(
                                ui.ctx().clone(),
                                version,
                                bin_url,
                                sha_url,
                                bin_name,
                            );
                        }
                    } else {
                        ui.add(egui::Label::new(
                            RichText::new("Downloading automatically...")
                                .color(theme::muted_text(ui.ctx()))
                                .size(12.0),
                        ));
                    }
                }
                mouser_engine::updater::UpdateStatus::Downloading { progress } => {
                    let pct = (progress * 100.0) as u32;
                    ui.add(egui::Label::new(
                        RichText::new(format!("Downloading — {}%", pct))
                            .color(theme::secondary_text(ui.ctx()))
                            .size(12.5),
                    ));
                    ui.add_space(4.0);
                    let pb_w = ui.available_width();
                    let pb_h = 4.0;
                    let (pb_rect, _) =
                        ui.allocate_exact_size(vec2(pb_w, pb_h), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(pb_rect, 2.0, theme::elevated_color(ui.ctx()));
                    let fill_rect = Rect::from_min_max(
                        pb_rect.min,
                        pos2(pb_rect.left() + pb_w * progress, pb_rect.max.y),
                    );
                    ui.painter()
                        .rect_filled(fill_rect, 2.0, theme::accent_color(ui.ctx()));
                }
                mouser_engine::updater::UpdateStatus::Verifying => {
                    ui.add(egui::Label::new(
                        RichText::new("Verifying checksum...")
                            .color(theme::secondary_text(ui.ctx()))
                            .size(12.5),
                    ));
                }
                mouser_engine::updater::UpdateStatus::Installing => {
                    ui.add(egui::Label::new(
                        RichText::new("Installing update...")
                            .color(theme::secondary_text(ui.ctx()))
                            .size(12.5),
                    ));
                }
                mouser_engine::updater::UpdateStatus::RestartRequired => {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("✓")
                                .color(theme::accent_color(ui.ctx()))
                                .size(14.0)
                                .strong(),
                        ));
                        ui.add(egui::Label::new(
                            RichText::new("Update installed!")
                                .color(theme::accent_color(ui.ctx()))
                                .size(13.0)
                                .strong(),
                        ));
                    });
                    ui.add_space(6.0);
                    let btn = ui.add(
                        egui::Button::new(RichText::new("Restart App").strong().size(12.5))
                            .fill(theme::accent_color(ui.ctx()))
                            .stroke(Stroke::NONE)
                            .rounding(2.0),
                    );
                    if btn.clicked() {
                        let _ = mouser_engine::updater::Updater::restart_and_apply();
                    }
                }
                mouser_engine::updater::UpdateStatus::Failed(err) => {
                    ui.add(egui::Label::new(
                        RichText::new(format!("Failed: {}", err))
                            .color(theme::danger_color(ui.ctx()))
                            .size(12.5),
                    ));
                    ui.add_space(4.0);
                    let btn = ui.add(
                        egui::Button::new(RichText::new("Retry").size(12.5))
                            .fill(theme::elevated_color(ui.ctx()))
                            .stroke(Stroke::new(1.0, theme::border_color(ui.ctx())))
                            .rounding(2.0),
                    );
                    if btn.clicked() {
                        updater.check_for_updates(ui.ctx().clone(), config.settings.install_updates);
                    }
                }
                _ => {}
            }
        }
    });
}

fn render_section_language(ui: &mut egui::Ui, config: &mut Config, engine: &Engine) {
    section_card(ui, |ui| {
        let bg_color = theme::elevated_color(ui.ctx());
        let border_clr = theme::border_color(ui.ctx());
        let hover_bg = theme::hover_color(ui.ctx());
        let is_dark = ui.visuals().dark_mode;
        let accent_color = theme::accent_color(ui.ctx());

        let widgets = &mut ui.style_mut().visuals.widgets;

        // Inactive style (Sharp 2.0 rounding)
        widgets.inactive.bg_fill = bg_color;
        widgets.inactive.bg_stroke = Stroke::new(1.0, border_clr);
        widgets.inactive.rounding = egui::Rounding::same(2.0);

        // Hovered style (Subtle high-contrast transition)
        widgets.hovered.bg_fill = hover_bg;
        widgets.hovered.bg_stroke = Stroke::new(
            1.0,
            if is_dark {
                Color32::from_rgb(0x44, 0x44, 0x44)
            } else {
                Color32::from_rgb(0x88, 0x88, 0x88)
            },
        );
        widgets.hovered.rounding = egui::Rounding::same(2.0);

        // Active style (Border color highlights with accent color)
        widgets.active.bg_fill = bg_color;
        widgets.active.bg_stroke = Stroke::new(1.0, accent_color);
        widgets.active.rounding = egui::Rounding::same(2.0);

        ui.horizontal(|ui| {
            // ── Left side: Icon + Label ──
            let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            draw_globe_icon(ui, icon_rect, theme::accent_color(ui.ctx()));
            ui.add_space(6.0);
            render_spaced_header(
                ui,
                tr("language_label", &config.settings.language),
                14.0,
                theme::primary_text(ui.ctx()),
            );

            // ── Spacing to push combobox to the right ──
            let combo_w = 220.0;
            let spacing = ui.spacing().item_spacing.x;
            let space_to_add = ui.available_width() - combo_w - spacing;
            if space_to_add > 0.0 {
                ui.add_space(space_to_add);
            }

            // ── Right side: Styled Dropdown ──
            ui.spacing_mut().combo_width = combo_w;
            ui.spacing_mut().button_padding = vec2(14.0, 8.0);

            let mut selected_lang = config.settings.language.clone();

            let display_title = if selected_lang == "Use system language" {
                tr("use_system_lang", &config.settings.language).to_string()
            } else {
                selected_lang.clone()
            };

            let combo = egui::ComboBox::from_id_salt("lang_dropdown")
                .selected_text(RichText::new(&display_title).color(theme::primary_text(ui.ctx())));

            let response = combo.show_ui(ui, |ui| {
                let mut changed = false;

                let options = &[
                    (
                        "Use system language",
                        tr("use_system_lang", &config.settings.language),
                    ),
                    ("English", "English"),
                    ("Türkçe", "Türkçe"),
                    ("Deutsch", "Deutsch"),
                    ("Français", "Français"),
                    ("Español", "Español"),
                    ("Italiano", "Italiano"),
                    ("Português", "Português"),
                    ("Nederlands", "Nederlands"),
                    ("Polski", "Polski"),
                    ("Русский", "Русский"),
                    ("Svenska", "Svenska"),
                    ("Dansk", "Dansk"),
                    ("Suomi", "Suomi"),
                    ("Norsk", "Norsk"),
                    ("Ελληνικά", "Ελληνικά"),
                    ("Čeština", "Čeština"),
                    ("Magyar", "Magyar"),
                    ("Română", "Română"),
                    ("Українська", "Українська"),
                ];

                for &(val, display) in options {
                    if ui
                        .selectable_value(&mut selected_lang, val.to_string(), display)
                        .clicked()
                    {
                        changed = true;
                    }
                }
                changed
            });

            if let Some(inner) = response.inner {
                if inner {
                    config.settings.language = selected_lang;
                    let _ = config.save();
                    engine.reload_config();
                }
            }
        });
    });
}

fn render_section_theme(ui: &mut egui::Ui, _ctx: &egui::Context, config: &mut Config, engine: &Engine) {
    section_card(ui, |ui| {
        // ── Section header ──
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            draw_rgb_palette_icon(ui, icon_rect);
            ui.add_space(6.0);
            render_spaced_header(
                ui,
                tr("theme_label", &config.settings.language),
                14.0,
                theme::primary_text(ui.ctx()),
            );
        });

        ui.add_space(14.0);

        let total_w = ui.available_width();
        let gap = 12.0;
        let card_w = ((total_w - 2.0 * gap) / 3.0).max(140.0);
        let card_h = 170.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, 0.0);

            // Card 1: Follow OS Theme
            let label1 = tr("follow_os", &config.settings.language);
            if draw_theme_card(
                ui,
                card_w,
                card_h,
                label1,
                config.settings.appearance_mode == "system",
                |painter, rect| {
                    // Split background: left periwinkle, right navy
                    let mid_x = rect.center().x;
                    let left_bg = Rect::from_min_max(rect.min, pos2(mid_x, rect.max.y));
                    let right_bg = Rect::from_min_max(pos2(mid_x, rect.min.y), rect.max);
                    painter.rect_filled(
                        left_bg,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 0.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0xb5, 0xc4, 0xf5),
                    );
                    painter.rect_filled(
                        right_bg,
                        egui::Rounding {
                            nw: 0.0,
                            ne: 2.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0x14, 0x19, 0x23),
                    );

                    // Draw mock panel in the middle
                    let panel_w = 110.0;
                    let panel_h = 80.0;
                    let panel_rect = Rect::from_center_size(
                        pos2(rect.center().x, rect.bottom() - 10.0),
                        vec2(panel_w, panel_h),
                    );

                    // Left half of panel (White, rounded top-left)
                    let panel_mid_x = panel_rect.center().x;
                    let left_panel =
                        Rect::from_min_max(panel_rect.min, pos2(panel_mid_x, panel_rect.max.y));
                    painter.rect_filled(
                        left_panel,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 0.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::WHITE,
                    );

                    // Right half of panel (Dark charcoal, rounded top-right)
                    let right_panel =
                        Rect::from_min_max(pos2(panel_mid_x, panel_rect.min.y), panel_rect.max);
                    painter.rect_filled(
                        right_panel,
                        egui::Rounding {
                            nw: 0.0,
                            ne: 2.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::from_rgb(0x1a, 0x1a, 0x1a),
                    );

                    // Draw window controls dots (Red, Yellow, Green)
                    let dot_y = left_panel.top() + 8.0;
                    painter.circle_filled(
                        pos2(left_panel.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0x5f, 0x56),
                    );
                    painter.circle_filled(
                        pos2(left_panel.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0xbd, 0x2e),
                    );
                    painter.circle_filled(
                        pos2(left_panel.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x27, 0xc9, 0x3f),
                    );

                    painter.circle_filled(
                        pos2(right_panel.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x8f, 0x3a, 0x35),
                    );
                    painter.circle_filled(
                        pos2(right_panel.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x8f, 0x6e, 0x1d),
                    );
                    painter.circle_filled(
                        pos2(right_panel.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x1a, 0x75, 0x27),
                    );

                    // Draw mock UI lines
                    let line_y1 = panel_rect.top() + 18.0;
                    let line_y2 = panel_rect.top() + 32.0;
                    let line_y3 = panel_rect.top() + 46.0;

                    // Draw mock UI lines (Gray bars + orange accent pill + muted pill)
                    let stroke_gray = Stroke::new(2.5, Color32::from_rgb(0xdd, 0xdd, 0xdd));
                    painter.line_segment(
                        [
                            pos2(left_panel.left() + 8.0, line_y1),
                            pos2(left_panel.right() - 8.0, line_y1),
                        ],
                        stroke_gray,
                    );

                    // Preview accent pill
                    let preview_accent = Color32::from_rgb(0x00, 0xC8, 0xB0);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(left_panel.left() + 8.0, line_y2 - 2.5),
                            pos2(left_panel.right() - 12.0, line_y2 + 2.5),
                        ),
                        1.0,
                        preview_accent,
                    );

                    // Muted pill
                    let muted_color = Color32::from_rgb(0x60, 0x60, 0x60);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(left_panel.left() + 8.0, line_y3 - 2.5),
                            pos2(left_panel.right() - 16.0, line_y3 + 2.5),
                        ),
                        1.0,
                        muted_color,
                    );

                    // Right side details (Gray bars)
                    let stroke_dark_gray = Stroke::new(2.5, Color32::from_rgb(0x44, 0x44, 0x44));
                    painter.line_segment(
                        [
                            pos2(right_panel.left() + 8.0, line_y1),
                            pos2(right_panel.right() - 8.0, line_y1),
                        ],
                        stroke_dark_gray,
                    );
                    painter.line_segment(
                        [
                            pos2(right_panel.left() + 8.0, line_y2),
                            pos2(right_panel.right() - 12.0, line_y2),
                        ],
                        stroke_dark_gray,
                    );
                    painter.line_segment(
                        [
                            pos2(right_panel.left() + 8.0, line_y3),
                            pos2(right_panel.right() - 20.0, line_y3),
                        ],
                        stroke_dark_gray,
                    );
                },
            ) {
                config.settings.appearance_mode = "system".to_string();
                let _ = config.save();
                engine.reload_config();
            }

            // Card 2: Light Theme
            let label2 = tr("light_theme", &config.settings.language);
            if draw_theme_card(
                ui,
                card_w,
                card_h,
                label2,
                config.settings.appearance_mode == "light",
                |painter, rect| {
                    // Background: soft periwinkle blue
                    painter.rect_filled(
                        rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0xb5, 0xc4, 0xf5),
                    );

                    // Draw mock panel
                    let panel_w = 110.0;
                    let panel_h = 80.0;
                    let panel_rect = Rect::from_center_size(
                        pos2(rect.center().x, rect.bottom() - 10.0),
                        vec2(panel_w, panel_h),
                    );

                    // Panel (White, rounded top)
                    painter.rect_filled(
                        panel_rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::WHITE,
                    );

                    // Draw window controls dots (Red, Yellow, Green)
                    let dot_y = panel_rect.top() + 8.0;
                    painter.circle_filled(
                        pos2(panel_rect.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0x5f, 0x56),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0xbd, 0x2e),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x27, 0xc9, 0x3f),
                    );

                    // Mock UI lines
                    let line_y1 = panel_rect.top() + 18.0;
                    let line_y2 = panel_rect.top() + 32.0;
                    let line_y3 = panel_rect.top() + 46.0;

                    let stroke_gray = Stroke::new(2.5, Color32::from_rgb(0xdd, 0xdd, 0xdd));
                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y1),
                            pos2(panel_rect.right() - 8.0, line_y1),
                        ],
                        stroke_gray,
                    );

                    // Preview accent pill
                    let preview_accent = Color32::from_rgb(0x00, 0x89, 0x7B);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(panel_rect.left() + 8.0, line_y2 - 2.5),
                            pos2(panel_rect.right() - 20.0, line_y2 + 2.5),
                        ),
                        1.0,
                        preview_accent,
                    );

                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y3),
                            pos2(panel_rect.right() - 16.0, line_y3),
                        ],
                        stroke_gray,
                    );
                },
            ) {
                config.settings.appearance_mode = "light".to_string();
                let _ = config.save();
                engine.reload_config();
            }

            // Card 3: Dark Theme
            let label3 = tr("dark_theme", &config.settings.language);
            if draw_theme_card(
                ui,
                card_w,
                card_h,
                label3,
                config.settings.appearance_mode == "dark",
                |painter, rect| {
                    // Background: dark navy
                    painter.rect_filled(
                        rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            sw: 0.0,
                            se: 0.0,
                        },
                        Color32::from_rgb(0x14, 0x19, 0x23),
                    );

                    // Draw mock panel
                    let panel_w = 110.0;
                    let panel_h = 80.0;
                    let panel_rect = Rect::from_center_size(
                        pos2(rect.center().x, rect.bottom() - 10.0),
                        vec2(panel_w, panel_h),
                    );

                    // Panel (Dark charcoal, rounded top)
                    painter.rect_filled(
                        panel_rect,
                        egui::Rounding {
                            nw: 2.0,
                            ne: 2.0,
                            se: 0.0,
                            sw: 0.0,
                        },
                        Color32::from_rgb(0x1a, 0x1a, 0x1a),
                    );

                    // Draw window controls dots (Red, Yellow, Green)
                    let dot_y = panel_rect.top() + 8.0;
                    painter.circle_filled(
                        pos2(panel_rect.left() + 8.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0x5f, 0x56),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 13.0, dot_y),
                        1.6,
                        Color32::from_rgb(0xff, 0xbd, 0x2e),
                    );
                    painter.circle_filled(
                        pos2(panel_rect.left() + 18.0, dot_y),
                        1.6,
                        Color32::from_rgb(0x27, 0xc9, 0x3f),
                    );

                    // Mock UI lines
                    let line_y1 = panel_rect.top() + 18.0;
                    let line_y2 = panel_rect.top() + 32.0;
                    let line_y3 = panel_rect.top() + 46.0;

                    let stroke_dark_gray = Stroke::new(2.5, Color32::from_rgb(0x44, 0x44, 0x44));
                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y1),
                            pos2(panel_rect.right() - 8.0, line_y1),
                        ],
                        stroke_dark_gray,
                    );

                    // Preview accent pill
                    let preview_accent = Color32::from_rgb(0x00, 0xC8, 0xB0);
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(panel_rect.left() + 8.0, line_y2 - 2.5),
                            pos2(panel_rect.right() - 20.0, line_y2 + 2.5),
                        ),
                        1.0,
                        preview_accent,
                    );

                    painter.line_segment(
                        [
                            pos2(panel_rect.left() + 8.0, line_y3),
                            pos2(panel_rect.right() - 16.0, line_y3),
                        ],
                        stroke_dark_gray,
                    );
                },
            ) {
                config.settings.appearance_mode = "dark".to_string();
                let _ = config.save();
                engine.reload_config();
            }
        });
    }); // section_card
}

fn draw_theme_card<F>(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    label: &str,
    is_selected: bool,
    draw_preview: F,
) -> bool
where
    F: FnOnce(&egui::Painter, Rect),
{
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), egui::Sense::click());

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    // Smooth border and background color hover transitions
    let is_sel_anim = ui.ctx().animate_bool(response.id, is_selected);
    let is_hov_anim = ui
        .ctx()
        .animate_bool(response.id.with("hover"), response.hovered());

    let base_border = theme::border_color(ui.ctx());
    let hover_border = if ui.visuals().dark_mode {
        Color32::from_rgb(0x55, 0x55, 0x55)
    } else {
        Color32::from_rgb(0xaa, 0xaa, 0xaa)
    };
    let selected_border = theme::accent_color(ui.ctx());

    // Calculate animated border color and thickness
    let border_color = if is_selected {
        selected_border
    } else {
        theme::lerp_color(base_border, hover_border, is_hov_anim)
    };
    let border_thickness = if is_selected {
        1.5
    } else {
        1.0 + 0.5 * is_hov_anim
    };

    // Calculate animated background color using surface and hover colors from active theme
    let bg_fill = theme::lerp_color(
        theme::card_bg(ui.ctx()),
        theme::hover_color(ui.ctx()),
        is_hov_anim,
    );
    ui.painter().rect_filled(rect, 2.0, bg_fill);

    // Split layout for preview (top 110px) and label (bottom 70px)
    let preview_rect = Rect::from_min_max(
        rect.min + vec2(1.0, 1.0),
        pos2(rect.max.x - 1.0, rect.min.y + 110.0),
    );

    // Draw preview
    let painter = ui.painter();
    draw_preview(&painter.with_clip_rect(preview_rect), preview_rect);

    // Bottom area
    let label_rect = Rect::from_min_max(pos2(rect.min.x, rect.min.y + 110.0), rect.max);

    // Minimalist Square Checkbox (14x14 px)
    let check_size = 14.0;
    let check_rect = Rect::from_center_size(
        pos2(label_rect.left() + 24.0, label_rect.center().y),
        vec2(check_size, check_size),
    );

    let inactive_border = theme::border_color(ui.ctx());
    let active_border = theme::accent_color(ui.ctx());
    let check_border_color = theme::lerp_color(inactive_border, active_border, is_sel_anim);

    // Draw outer checkbox border (2px corner radius)
    ui.painter()
        .rect_stroke(check_rect, 1.0, Stroke::new(1.0, check_border_color));

    // Draw filled Signal Orange inner square (8x8 px) when selected
    let inner_size = 8.0 * is_sel_anim;
    if inner_size > 0.0 {
        let inner_rect = Rect::from_center_size(check_rect.center(), vec2(inner_size, inner_size));
        ui.painter()
            .rect_filled(inner_rect, 0.0, theme::accent_color(ui.ctx()));
    }

    // Label text
    let text_pos = pos2(check_rect.center().x + 20.0, label_rect.center().y);
    let text_color = theme::primary_text(ui.ctx());

    // Draw text with support for multi-line (using splitting on newline if present)
    let lines: Vec<&str> = label.split('\n').collect();
    if lines.len() == 2 {
        let galley1 = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                lines[0].to_string(),
                egui::FontId::proportional(13.0),
                text_color,
            ))
        });
        let galley2 = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                lines[1].to_string(),
                egui::FontId::proportional(13.0),
                text_color,
            ))
        });
        ui.painter()
            .galley(text_pos - vec2(0.0, 14.0), galley1, text_color);
        ui.painter()
            .galley(text_pos - vec2(0.0, -2.0), galley2, text_color);
    } else {
        let galley = ui.fonts(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                label.to_string(),
                egui::FontId::proportional(13.0),
                text_color,
            ))
        });
        let text_y = text_pos.y - galley.size().y / 2.0;
        ui.painter()
            .galley(pos2(text_pos.x, text_y), galley, text_color);
    }

    // Draw border around the entire card
    ui.painter()
        .rect_stroke(rect, 2.0, Stroke::new(border_thickness, border_color));

    // Tech corner brackets (which glow in orange on hover or selection)
    let corner_color = theme::lerp_color(
        theme::border_color(ui.ctx()),
        theme::accent_color(ui.ctx()),
        is_sel_anim.max(is_hov_anim),
    );
    theme::draw_tech_corners(ui.painter(), rect, corner_color, 6.0);

    response.clicked()
}

fn draw_globe_icon(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let center = rect.center();
    let r = 6.0;
    let stroke = egui::Stroke::new(1.2, color);
    painter.circle_stroke(center, r, stroke);

    // Horizontal equator
    painter.line_segment(
        [center - egui::vec2(r, 0.0), center + egui::vec2(r, 0.0)],
        stroke,
    );
    // Vertical meridian
    painter.line_segment(
        [center - egui::vec2(0.0, r), center + egui::vec2(0.0, r)],
        stroke,
    );

    // Oval vertical meridian
    let num_pts = 16;
    for i in 0..num_pts {
        let t1 = i as f32 / num_pts as f32;
        let t2 = (i + 1) as f32 / num_pts as f32;
        let ang1 = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * t1;
        let ang2 = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * t2;

        let p1 = center + egui::vec2(ang1.cos() * r * 0.5, ang1.sin() * r);
        let p2 = center + egui::vec2(ang2.cos() * r * 0.5, ang2.sin() * r);
        painter.line_segment([p1, p2], stroke);
    }
}

fn draw_rgb_palette_icon(ui: &mut egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let center = rect.center();
    let r = 4.0;
    let offset = 2.5;

    // Three overlapping circles (Red, Green, Blue)
    let c_red = center + egui::vec2(0.0, -offset);
    let c_green = center + egui::vec2(-offset * 0.866, offset * 0.5);
    let c_blue = center + egui::vec2(offset * 0.866, offset * 0.5);

    let alpha = 150;
    painter.circle_filled(
        c_red,
        r,
        egui::Color32::from_rgba_unmultiplied(255, 0, 0, alpha),
    );
    painter.circle_filled(
        c_green,
        r,
        egui::Color32::from_rgba_unmultiplied(0, 255, 0, alpha),
    );
    painter.circle_filled(
        c_blue,
        r,
        egui::Color32::from_rgba_unmultiplied(0, 0, 255, alpha),
    );

    // Add thin outlines
    let stroke = egui::Stroke::new(0.8, egui::Color32::WHITE);
    painter.circle_stroke(c_red, r, stroke);
    painter.circle_stroke(c_green, r, stroke);
    painter.circle_stroke(c_blue, r, stroke);
}

fn draw_refresh_icon(ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter();
    let center = rect.center();
    let r = 5.5;
    let stroke = egui::Stroke::new(1.2, color);

    // Draw arc for circular arrow (about 270 degrees)
    let start_ang = -45.0_f32.to_radians();
    let end_ang = 225.0_f32.to_radians();
    let num_segments = 16;
    let mut points = Vec::with_capacity(num_segments + 1);
    for j in 0..=num_segments {
        let t = j as f32 / num_segments as f32;
        let angle = start_ang + (end_ang - start_ang) * t;
        points.push(center + egui::vec2(angle.cos() * r, angle.sin() * r));
    }
    for j in 0..num_segments {
        painter.line_segment([points[j], points[j + 1]], stroke);
    }

    // Draw arrow head at start_ang
    let arrow_pos = points[0];
    let arrow_stroke = egui::Stroke::new(1.2, color);
    painter.line_segment(
        [arrow_pos, arrow_pos + egui::vec2(-2.0, -3.0)],
        arrow_stroke,
    );
    painter.line_segment([arrow_pos, arrow_pos + egui::vec2(-3.0, 2.0)], arrow_stroke);
}

fn render_spaced_header(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
    let mut job = egui::text::LayoutJob::default();
    let text_upper = text.to_uppercase();

    // Add technical HUD prefix "// "
    job.append(
        "// ",
        0.0,
        egui::text::TextFormat {
            font_id: egui::FontId::monospace(size),
            color: color.linear_multiply(0.6),
            ..Default::default()
        },
    );

    let mut chars = text_upper.char_indices().peekable();
    let mut is_first = true;
    while let Some((idx, _)) = chars.next() {
        let next_idx = chars.peek().map(|(n_idx, _)| *n_idx).unwrap_or(text_upper.len());
        let space = if is_first {
            is_first = false;
            0.0
        } else {
            size * 0.15
        };
        job.append(
            &text_upper[idx..next_idx],
            space,
            egui::text::TextFormat {
                font_id: egui::FontId::monospace(size),
                color,
                ..Default::default()
            },
        );
    }
    ui.add(egui::Label::new(job));
}



fn draw_profiles_icon_settings(ui: &egui::Ui, rect: egui::Rect, color: Color32) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.2, color);
    let center = rect.center();
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

fn render_section_profiles(ui: &mut egui::Ui, config: &mut Config, engine: &Engine) {
    section_card(ui, |ui| {
        // ── Section header ──
        ui.horizontal(|ui| {
            let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            draw_profiles_icon_settings(ui, icon_rect, theme::accent_color(ui.ctx()));
            ui.add_space(6.0);
            render_spaced_header(
                ui,
                "PROFILES",
                14.0,
                theme::primary_text(ui.ctx()),
            );
        });

        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.label("Selected Profile:");
            let mut sorted_keys: Vec<String> = config.profiles.keys().cloned().collect();
            sorted_keys.sort();

            let mut editing_profile = crate::mouse_ui::SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());
            if editing_profile.is_empty() {
                editing_profile = config.active_profile.clone();
                crate::mouse_ui::SELECTED_EDIT_PROFILE.with(|p| *p.borrow_mut() = editing_profile.clone());
                if let Some(prof) = config.profiles.get(&editing_profile) {
                    crate::mouse_ui::APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = prof.apps.join(", "));
                }
            }

            let combo = egui::ComboBox::from_id_salt("settings_editing_profile_combo")
                .selected_text(RichText::new(&editing_profile).color(theme::primary_text(ui.ctx())));
            let res = combo.show_ui(ui, |ui| {
                let mut changed = false;
                for p_name in &sorted_keys {
                    let label = if *p_name == config.active_profile {
                        format!("★ {}", p_name)
                    } else {
                        p_name.clone()
                    };
                    if ui.selectable_value(&mut editing_profile, p_name.clone(), label).clicked() {
                        changed = true;
                    }
                }
                changed
            });

            if let Some(true) = res.inner {
                crate::mouse_ui::SELECTED_EDIT_PROFILE.with(|p| *p.borrow_mut() = editing_profile.clone());
                if let Some(prof) = config.profiles.get(&editing_profile) {
                    crate::mouse_ui::APP_BINDINGS_BUFFER.with(|b| *b.borrow_mut() = prof.apps.join(", "));
                }
            }

            // Button to activate if not active
            if editing_profile != config.active_profile {
                if ui.button("Activate").clicked() {
                    engine.select_profile(&editing_profile);
                }
            } else {
                ui.label(RichText::new("Active").size(11.0).color(theme::accent_color(ui.ctx())));
            }
        });

        ui.add_space(14.0);

        let editing_profile = crate::mouse_ui::SELECTED_EDIT_PROFILE.with(|p| p.borrow().clone());
        if !editing_profile.is_empty() {
            // Delete profile
            if editing_profile == "default" {
                ui.add_enabled(false, egui::Button::new("Delete Profile"));
                ui.label(RichText::new("The 'default' profile cannot be deleted.").size(10.0).color(theme::muted_text(ui.ctx())));
            } else {
                if ui.button(RichText::new("Delete Profile").color(theme::danger_color(ui.ctx()))).clicked() {
                    engine.delete_profile(&editing_profile);
                    crate::mouse_ui::SELECTED_EDIT_PROFILE.with(|p| p.borrow_mut().clear());
                }
            }


        }

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(14.0);

        // Add profile form
        ui.label("Create New Profile:");
        ui.horizontal(|ui| {
            crate::mouse_ui::NEW_PROFILE_NAME.with(|name_cell| {
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
}

