pub mod section_language;
pub mod section_profiles;
pub mod section_theme;
pub mod section_updates;

use crate::theme;
use crate::translation::tr;
use crate::updater::Updater;
use eframe::egui;
use egui::{Color32, Stroke};
use mouser_engine::config::Config;
use mouser_engine::Engine;

pub use section_language::render_section_language;
pub use section_profiles::render_section_profiles;
pub use section_theme::render_section_theme;
pub use section_updates::render_section_updates;

pub fn show(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    config: &mut Config,
    engine: &Engine,
    updater: &Updater,
) {
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

            // SECTION 4: PROFILES
            render_section_profiles(ui, config, engine);
        });
    });
}

/// Draws a section card background: surface-colored rounded rect with a thin border.
pub fn section_card<R>(
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

pub fn render_spaced_header(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
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
        let next_idx = chars
            .peek()
            .map(|(n_idx, _)| *n_idx)
            .unwrap_or(text_upper.len());
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
