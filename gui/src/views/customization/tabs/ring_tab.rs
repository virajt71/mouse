use crate::theme;
use crate::views::customization::mappings::CustomizingButton;
use eframe::egui;
use egui::{RichText};
use mouser_engine::client::EngineClient as Engine;
use mouser_engine::config::{Config, RingBubble, RingBubbleKind, RingFolder, RingLayout};

pub fn show_ring_tab(
    ui: &mut egui::Ui,
    engine: &Engine,
    config: &mut Config,
    customizing_button: &mut Option<CustomizingButton>,
) {
    let _ = engine;
    let _ = customizing_button;

    ui.painter().rect_filled(ui.max_rect(), 0.0, theme::app_bg(ui.ctx()));
    ui.add_space(16.0);

    let active_app_profile = config.active_app_profile.clone();
    let profile = config.get_profile(&active_app_profile)
        .or_else(|| config.get_profile("global"))
        .cloned()
        .unwrap_or_default();

    let has_layout = profile.ring_layout.is_some();
    let layout = profile.ring_layout.as_ref();

    ui.horizontal(|ui| {
        ui.label(RichText::new("Actions Ring").heading());
        if !has_layout {
            ui.label(RichText::new("(No layout for this profile)").color(theme::secondary_text(ui.ctx())));
        } else if let Some(l) = layout {
            ui.label(RichText::new(format!("{} primary bubbles", l.primary.len())).color(theme::secondary_text(ui.ctx())));
        }
    });
    ui.add_space(12.0);

    if !has_layout {
        if ui.button("Initialize Ring Layout (8 bubbles)").clicked() {
            if let Some(p) = config.get_profile_mut(&active_app_profile) {
                if p.ring_layout.is_none() {
                    let mut layout = RingLayout::default();
                    layout.primary = (0..8).map(|i| RingBubble {
                        action_id: String::new(),
                        icon_name: String::new(),
                        label: format!("Bubble {}", i + 1),
                        kind: RingBubbleKind::Action { action_id: String::new() },
                        adjustment_range: None,
                    }).collect();
                    p.ring_layout = Some(layout);
                }
            }
            let _ = config.save();
        }
        return;
    }

    let layout = profile.ring_layout.as_ref().unwrap();
    let mut edited = false;

    // Primary bubbles editing
    ui.label(RichText::new("Primary Bubbles (8 max):"));
    ui.add_space(8.0);

    let mut new_bubbles: Vec<RingBubble> = layout.primary.clone();
    for (i, bubble) in new_bubbles.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("B{}", i + 1)).size(11.0).monospace());

            let mut label = bubble.label.clone();
            ui.label("Label:");
            let resp = ui.text_edit_singleline(&mut label);
            if resp.changed() {
                bubble.label = label;
                edited = true;
            }

            let mut icon = bubble.icon_name.clone();
            ui.label("Icon:");
            ui.text_edit_singleline(&mut icon);
            if icon != bubble.icon_name {
                bubble.icon_name = icon;
                edited = true;
            }
        });

        // Action field
        ui.horizontal(|ui| {
            ui.label("  Action ID:");
            let mut action_str = get_bubble_action_id(&bubble.kind);
            let resp = ui.text_edit_singleline(&mut action_str);
            if resp.changed() {
                bubble.kind = RingBubbleKind::Action { action_id: action_str };
                edited = true;
            }
        });

        // Toggle folder
        let is_folder = matches!(&bubble.kind, RingBubbleKind::Folder { .. });
        if ui.button(if is_folder { "Unlink Folder" } else { "Link to Folder" }).clicked() {
            if is_folder {
                bubble.kind = RingBubbleKind::Action { action_id: String::new() };
            } else {
                bubble.kind = RingBubbleKind::Folder { folder_id: String::new() };
            }
            edited = true;
        }
    }

    if edited {
        if let Some(p) = config.get_profile_mut(&active_app_profile) {
            if let Some(l) = &mut p.ring_layout {
                l.primary = new_bubbles;
            }
        }
        let _ = config.save();
    }

    ui.add_space(12.0);
    ui.add(egui::Separator::default().horizontal());
    ui.add_space(12.0);

    // Folders section
    ui.label(RichText::new("Folders:").heading());
    ui.add_space(8.0);

    let mut new_folders = layout.folders.clone();
    let mut remove_folder: Vec<usize> = vec![];
    for (i, folder) in new_folders.iter_mut().enumerate() {
        let mut name = folder.name.clone();
        let mut remove = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Folder {}", i + 1)).size(11.0));
            ui.text_edit_singleline(&mut name);
            if ui.button("Remove").clicked() {
                remove = true;
            }
        });
        if name != folder.name {
            folder.name = name;
            edited = true;
        }
        if remove {
            remove_folder.push(i);
            edited = true;
        }
    }
    for i in remove_folder.into_iter().rev() {
        new_folders.remove(i);
    }

    if ui.button("Add Folder").clicked() {
        new_folders.push(RingFolder {
            id: format!("folder_{}", new_folders.len() + 1),
            name: format!("Folder {}", new_folders.len() + 1),
            bubbles: vec![],
        });
        edited = true;
    }

    if edited {
        if let Some(p) = config.get_profile_mut(&active_app_profile) {
            if let Some(l) = &mut p.ring_layout {
                l.folders = new_folders;
            }
        }
        let _ = config.save();
    }
}

fn get_bubble_action_id(kind: &RingBubbleKind) -> String {
    match kind {
        RingBubbleKind::Action { action_id } => action_id.clone(),
        RingBubbleKind::Folder { folder_id } => format!("folder:{}", folder_id),
    }
}
