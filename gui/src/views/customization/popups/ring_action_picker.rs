use crate::theme;
use eframe::egui;
use egui::{pos2, vec2, Color32, RichText};
use mouser_engine::config::{RingBubble, RingBubbleKind};

/// System actions available for ring bubbles (matching UniversalButtonOption)
pub fn ring_system_actions() -> &'static [(&'static str, &'static str)] {
    &[
        ("none", "— None —"),
        ("copy", "Copy"),
        ("paste", "Paste"),
        ("cut", "Cut"),
        ("undo", "Undo"),
        ("redo", "Redo"),
        ("select_all", "Select All"),
        ("save", "Save"),
        ("close_tab", "Close Tab"),
        ("new_tab", "New Tab"),
        ("find", "Find"),
        ("zoom_in", "Zoom In"),
        ("zoom_out", "Zoom Out"),
        ("volume_up", "Volume Up"),
        ("volume_down", "Volume Down"),
        ("volume_mute", "Mute/Unmute"),
        ("play_pause", "Play/Pause"),
        ("next_track", "Next Track"),
        ("prev_track", "Previous Track"),
        ("brightness_up", "Brightness Up"),
        ("brightness_down", "Brightness Down"),
        ("tab_prev", "Previous Tab"),
        ("tab_next", "Next Tab"),
        ("page_up", "Page Up"),
        ("page_down", "Page Down"),
        ("home", "Home"),
        ("end", "End"),
        ("browser_back", "Browser Back"),
        ("browser_forward", "Browser Forward"),
        ("space_left", "Switch Workspace Left"),
        ("space_right", "Switch Workspace Right"),
        ("app_prev", "Previous App"),
        ("app_next", "Next App"),
        ("task_view", "Task View"),
        ("win_d", "Show Desktop"),
        ("print_screen", "Print Screen"),
        ("screen_snip", "Screen Snip"),
        ("lock", "Lock Screen"),
        ("emoji", "Emoji Picker"),
        ("dictation", "Dictation"),
        ("calculator", "Calculator"),
    ]
}

/// Action picker popup for ring bubbles. Returns true if an action was selected.
pub fn draw_ring_action_picker_popup(
    ui: &mut egui::Ui,
    bubble: &mut RingBubble,
    bubble_idx: usize,
    ctx: &egui::Context,
    _view_state_id: egui::Id,
) -> bool {
    let mut selected = false;
    let actions = ring_system_actions();

    egui::ScrollArea::vertical()
        .id_salt("ring_action_list")
        .show(ui, |ui| {
            ui.add_space(4.0);

            let mut filter = String::new();
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(RichText::new("Filter:").size(10.0).color(theme::secondary_text(ctx)));
                ui.add_space(4.0);
                ui.add(egui::TextEdit::singleline(&mut filter).font(egui::FontId::proportional(11.0)).desired_width(140.0));
                ui.add_space(4.0);
            });

            ui.add_space(8.0);

            let filter_lower = filter.to_lowercase();
            let filtered: Vec<_> = actions
                .iter()
                .filter(|(id, name)| {
                    filter_lower.is_empty() || name.to_lowercase().contains(&filter_lower) || id.to_lowercase().contains(&filter_lower)
                })
                .collect();

            for &(action_id, name) in &filtered {
                let item_h = 26.0;
                let (rect, response) = ui.allocate_exact_size(
                    vec2(ui.available_width(), item_h),
                    egui::Sense::click(),
                );

                let current_action = match &bubble.kind {
                    RingBubbleKind::Action { action_id } => action_id.as_str(),
                    RingBubbleKind::Folder { .. } => "", // folders don't match any action
                };
                let is_selected = current_action == *action_id;
                let is_hovered = response.hovered();

                if is_hovered {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(rect, 0.0, theme::hover_color(ctx));
                }

                let bullet_x = rect.min.x + 14.0;
                let bullet_y = rect.center().y;
                if is_selected {
                    let accent = theme::accent_color(ctx);
                    ui.painter().circle_filled(pos2(bullet_x, bullet_y), 6.0, accent);
                    ui.painter().circle_filled(pos2(bullet_x, bullet_y), 2.0, Color32::WHITE);
                } else {
                    ui.painter().circle_stroke(pos2(bullet_x, bullet_y), 6.0, egui::Stroke::new(1.0, Color32::from_gray(120)));
                }

                let text_color = if is_selected {
                    theme::accent_color(ctx)
                } else if is_hovered {
                    theme::primary_text(ctx)
                } else {
                    theme::secondary_text(ctx)
                };
                let galley = ui.fonts(|f| {
                    f.layout_job(egui::text::LayoutJob::simple_singleline(
                        name.to_string(),
                        egui::FontId::proportional(11.0),
                        text_color,
                    ))
                });
                let text_y = rect.center().y - galley.size().y / 2.0;
                ui.painter().galley(pos2(rect.min.x + 28.0, text_y), galley, text_color);

                if response.clicked() {
                    bubble.kind = RingBubbleKind::Action { action_id: action_id.to_string() };
                    if *action_id == "none" {
                        bubble.label = format!("Bubble {}", bubble_idx + 1);
                    } else {
                        bubble.label = name.to_string();
                    }
                    selected = true;
                }
            }

            ui.add_space(4.0);
        });

    selected
}
