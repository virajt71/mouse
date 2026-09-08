use eframe::egui;
use egui::RichText;
use mouser_engine::config::RingFolder;
use crate::views::customization::popups::ring_action_picker::ring_system_actions;

/// Folder editor popup. Returns true if the popup should close.
pub fn draw_ring_folder_popup(
    ui: &mut egui::Ui,
    folder: &mut RingFolder,
    _folders: &[RingFolder],
    ctx: &egui::Context,
) -> bool {
    let mut close = false;
    let actions = ring_system_actions();

    egui::ScrollArea::vertical()
        .id_salt("ring_folder_editor")
        .show(ui, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(format!("Folder: {}", folder.name)).heading());
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Name:").size(11.0));
                ui.text_edit_singleline(&mut folder.name);
            });

            ui.add_space(12.0);
            ui.label(RichText::new("Sub-Bubble Actions (up to 9):"));
            ui.add_space(8.0);

            // Collect indices of bubbles to remove (can't remove while iterating mutably in closure)
            let mut to_remove: Vec<usize> = vec![];

            let mut edits: Vec<(usize, String, mouser_engine::config::RingBubbleKind, String)> = vec![];
            for (i, bubble) in folder.bubbles.iter().enumerate() {
                let display = bubble.label.clone();
                let kind = bubble.kind.clone();
                let action_id = bubble.action_id.clone();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("B{}", i + 1)).size(11.0).monospace());
                    ui.label(RichText::new(display).size(10.0));

                    for &(a_id, name) in actions.iter().take(8) {
                        if ui.button(name).clicked() {
                            edits.push((i, name.to_string(), 
                                mouser_engine::config::RingBubbleKind::Action { action_id: a_id.to_string() },
                                a_id.to_string()));
                        }
                    }

                    if folder.bubbles.len() > 1 && ui.button("Remove").clicked() {
                        to_remove.push(i);
                    }
                });
                let _ = kind;
                let _ = action_id;
            }
            for (idx, label, kind, action_id) in edits {
                if let Some(b) = folder.bubbles.get_mut(idx) {
                    b.label = label;
                    b.kind = kind;
                    b.action_id = action_id;
                }
            }

            // Remove bubbles after the loop
            for i in to_remove.into_iter().rev() {
                folder.bubbles.remove(i);
            }

            ui.add_space(8.0);

            if folder.bubbles.len() < 9 {
                if ui.button("Add Sub-Bubble").clicked() {
                    folder.bubbles.push(mouser_engine::config::RingBubble::default());
                }
            }

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close = true;
                }
                if ui.button("Done").clicked() {
                    close = true;
                }
            });

            let _ = ctx;
        });

    close
}
