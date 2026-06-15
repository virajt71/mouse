use crate::theme;
use eframe::egui;
use egui::{Color32, RichText};
use mouser_engine::config::Config;
use mouser_engine::Engine;

pub fn show_profiles_settings_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    is_keyboard: bool,
) {
    if is_keyboard {
        ui.horizontal(|ui| {
            ui.add_space(40.0);
            ui.vertical(|ui| {
                ui.add_space(20.0);
                ui.add(egui::Label::new(
                    RichText::new("KEYBOARD LAYOUT")
                        .color(Color32::WHITE)
                        .size(18.0)
                        .strong(),
                ));
                ui.add_space(4.0);
                ui.label(RichText::new("Select your physical keyboard layout to ensure custom hotkey shortcuts are simulated correctly:")
                    .size(10.0)
                    .color(theme::muted_text(ui.ctx())));
                ui.add_space(15.0);

                let layout = config.settings.device_layout_overrides
                    .get("keyboard_layout")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ANSI (US)")
                    .to_string();

                let layouts = &[
                    "ANSI (US)",
                    "ISO (UK)",
                    "QWERTZ (German)",
                    "AZERTY (French)",
                    "US English",
                    "US International",
                    "UK (British) English",
                    "Arabic",
                    "Armenian",
                    "Azeri/Azerbaijani",
                    "Belgian",
                    "Bengali",
                    "Bosnian",
                    "Bulgarian",
                    "Burmese",
                    "Cherokee",
                    "Chinese",
                    "Colemak English",
                    "Croatian",
                    "Czech",
                    "Danish",
                    "Dutch",
                    "Dvorak English",
                    "Estonian",
                    "Finnish",
                    "French (BÉPO)",
                    "French (Canadian)",
                    "French",
                    "German",
                    "Georgian",
                    "Greek",
                    "Greek Polytonic",
                    "Gujarati",
                    "Hebrew",
                    "Hindi (Devanagari InScript)",
                    "Hungarian",
                    "Icelandic",
                    "Inuktitut (Canadian Aboriginal Syllabary)",
                    "Italian",
                    "Japanese",
                    "Kannada",
                    "Kazakh",
                    "Khmer",
                    "Korean (2-set) Hangul",
                    "Kurdish (Central)",
                    "Latvian",
                    "Lithuanian",
                    "Macedonian",
                    "Malayalam",
                    "Maltese",
                    "Nepali",
                    "Northern Sami",
                    "Norwegian",
                    "Odia/Oriya",
                    "Pashto",
                    "Persian/Farsi",
                    "Polish",
                    "Polish (214)",
                    "Portuguese (Brazilian ABNT)",
                    "Portuguese",
                    "Punjabi (Gurmukhi)",
                    "Romanian",
                    "Russian",
                    "Russian Phonetic/Mnemonic",
                    "Serbian",
                    "Serbian (Latin)",
                    "Sinhala",
                    "Slovak",
                    "Slovene/Slovenian",
                    "Spanish (Latin America)",
                    "Spanish",
                    "Swedish",
                    "Swiss German/French",
                    "Tamil",
                    "Telugu",
                    "Thai (Kedmanee)",
                    "Tibetan",
                    "Turkish F",
                    "Turkish Q",
                    "Ukrainian",
                    "Urdu",
                    "Uyghur",
                    "Uzbek (Arabic)",
                    "Vietnamese",
                ];
                let mut selected = layout.clone();
                let combo = egui::ComboBox::from_id_salt("keyboard_layout_combo_picker")
                    .selected_text(&selected);
                let combo_res = combo.show_ui(ui, |ui| {
                    let mut clicked = None;
                    for &l in layouts {
                        if ui.selectable_value(&mut selected, l.to_string(), l).clicked() {
                            clicked = Some(l.to_string());
                        }
                    }
                    clicked
                });

                if let Some(Some(new_layout)) = combo_res.inner {
                    engine.update_keyboard_layout(&new_layout);
                }
            });
        });
    }
}
