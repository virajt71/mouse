use super::{render_spaced_header, section_card};
use crate::theme;
use crate::translation::tr;
use crate::widgets::draw_globe_icon;
use eframe::egui;
use egui::{vec2, Color32, RichText, Stroke};
use mouser_engine::client::EngineClient as Engine;
use mouser_engine::config::Config;

pub fn render_section_language(ui: &mut egui::Ui, config: &mut Config, engine: &Engine) {
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
