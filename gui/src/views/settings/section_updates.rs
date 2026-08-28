use super::{render_spaced_header, section_card};
use crate::theme;
use crate::translation::tr;
use crate::updater::{UpdateStatus, Updater};
use crate::widgets::draw_refresh_icon;
use eframe::egui;
use egui::{pos2, vec2, Color32, Rect, RichText, Stroke};
use mouser_engine::client::EngineClient as Engine;
use mouser_engine::config::Config;
use mouser_engine::lock_ext::MutexExt;

pub fn render_section_updates(
    ui: &mut egui::Ui,
    config: &mut Config,
    engine: &Engine,
    updater: &Updater,
) {
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
                        let status = updater.status.lock_safe().clone();
                        if let UpdateStatus::Available {
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
                        } else if let UpdateStatus::Idle | UpdateStatus::Failed(_) = status {
                            updater.check_for_updates(
                                ui.ctx().clone(),
                                config.settings.install_updates,
                            );
                        }
                    }
                }

                if response.hovered() {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                }

                let t = ui
                    .ctx()
                    .animate_bool(response.id, config.settings.install_updates);

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
            let status = updater.status.lock_safe().clone();
            let is_busy = matches!(
                status,
                UpdateStatus::Checking
                    | UpdateStatus::Downloading { .. }
                    | UpdateStatus::Verifying
                    | UpdateStatus::Installing
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
        let status = updater.status.lock_safe().clone();
        if status != UpdateStatus::Idle {
            ui.add_space(12.0);
            let divider_rect = ui.allocate_space(vec2(ui.available_width(), 1.0)).1;
            ui.painter()
                .rect_filled(divider_rect, 0.0, theme::border_color(ui.ctx()));
            ui.add_space(10.0);

            // ── Status row ──
            match status {
                UpdateStatus::Checking => {
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
                UpdateStatus::UpToDate => {
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
                UpdateStatus::Available {
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
                UpdateStatus::Downloading { progress } => {
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
                UpdateStatus::Verifying => {
                    ui.add(egui::Label::new(
                        RichText::new("Verifying checksum...")
                            .color(theme::secondary_text(ui.ctx()))
                            .size(12.5),
                    ));
                }
                UpdateStatus::Installing => {
                    ui.add(egui::Label::new(
                        RichText::new("Installing update...")
                            .color(theme::secondary_text(ui.ctx()))
                            .size(12.5),
                    ));
                }
                UpdateStatus::RestartRequired => {
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
                        let _ = Updater::restart_and_apply();
                    }
                }
                UpdateStatus::Failed(err) => {
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
                        updater
                            .check_for_updates(ui.ctx().clone(), config.settings.install_updates);
                    }
                }
                _ => {}
            }
        }
    });
}
