use eframe::egui;
use egui::{pos2, vec2, Color32, Pos2, Rect, Stroke, TextureHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

thread_local! {
    static ICON_CACHE: RefCell<HashMap<String, Option<TextureHandle>>> = RefCell::new(HashMap::new());
}

fn icon_theme_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut bases = vec![
        PathBuf::from("/usr/share/icons"),
        PathBuf::from("/usr/local/share/icons"),
    ];
    if let Some(home) = dirs::home_dir() {
        bases.push(home.join(".local/share/icons"));
        bases.push(home.join(".icons"));
    }
    for base in bases {
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    roots.push(entry.path());
                }
            }
        }
    }
    // hicolor is the spec's fallback theme — check it last so a concrete
    // active-theme icon wins when both exist.
    roots.sort_by_key(|p| p.file_name().is_some_and(|n| n == "hicolor"));
    roots
}

pub fn resolve_icon_path(icon: &str) -> Option<PathBuf> {
    if icon.is_empty() {
        return None;
    }

    // If it's already an absolute path, check if it exists
    let path = Path::new(icon);
    if path.is_absolute() && path.exists() {
        return Some(path.to_path_buf());
    }

    let themes = icon_theme_roots();
    let sizes = [
        "256x256", "256x256@2x",
        "128x128", "128x128@2x",
        "96x96", "96x96@2x",
        "64x64", "64x64@2x",
        "48x48", "48x48@2x",
        "32x32", "32x32@2x",
        "24x24", "24x24@2x",
        "16x16", "16x16@2x"
    ];
    let categories = ["apps", "devices", "categories", "mimetypes", "places", "status"];

    for theme_dir in &themes {
        // 1. Search size-bucket directories
        for size in &sizes {
            for category in &categories {
                let base = theme_dir.join(size).join(category);
                let p1 = base.join(icon);
                if p1.exists() {
                    return Some(p1);
                }
                let p2 = base.join(format!("{}.png", icon));
                if p2.exists() {
                    return Some(p2);
                }
            }
        }

        // 2. Search flat layouts under the theme (e.g. {theme}/apps/{icon}.png)
        for category in &categories {
            let base = theme_dir.join(category);
            let p1 = base.join(icon);
            if p1.exists() {
                return Some(p1);
            }
            let p2 = base.join(format!("{}.png", icon));
            if p2.exists() {
                return Some(p2);
            }
        }
    }

    // Fall back to /usr/share/pixmaps
    let base_pix = PathBuf::from("/usr/share/pixmaps");
    let p1 = base_pix.join(icon);
    if p1.exists() {
        return Some(p1);
    }
    let p2 = base_pix.join(format!("{}.png", icon));
    if p2.exists() {
        return Some(p2);
    }

    log::debug!(
        "[icon_loader] could not resolve a PNG for icon '{}' in any installed theme (it may only ship as SVG)",
        icon
    );

    None
}

pub fn get_app_icon_texture(ctx: &egui::Context, icon: &str) -> Option<TextureHandle> {
    if icon.is_empty() {
        return None;
    }

    // Check cache
    let cached = ICON_CACHE.with(|cache| cache.borrow().get(icon).cloned());
    if let Some(tex_opt) = cached {
        return tex_opt;
    }

    // Resolve path
    let tex_opt = if let Some(path) = resolve_icon_path(icon) {
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let color_img = egui::ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    &rgba.into_raw(),
                );
                let tex = ctx.load_texture(
                    format!("app_icon_{}", icon),
                    color_img,
                    egui::TextureOptions::default(),
                );
                Some(tex)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Cache result
    ICON_CACHE.with(|cache| {
        cache.borrow_mut().insert(icon.to_string(), tex_opt.clone());
    });

    tex_opt
}

pub fn draw_generic_app_icon(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    fallback_color: Color32,
) {
    let size = radius * 1.5;
    let rect = Rect::from_center_size(center, vec2(size, size));

    // Draw background (elevated surface look)
    painter.rect_filled(rect, 2.0, Color32::from_rgb(0x22, 0x22, 0x22));

    // Draw border
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, Color32::from_rgb(0x44, 0x44, 0x44)),
    );

    // Draw generic app header line (title bar)
    let header_y = rect.min.y + size * 0.25;
    painter.line_segment(
        [pos2(rect.min.x, header_y), pos2(rect.max.x, header_y)],
        Stroke::new(1.0, Color32::from_rgb(0x44, 0x44, 0x44)),
    );

    // Draw three window controls (minimize/maximize/close representation as small dots)
    let dot_radius = size * 0.05;
    let dot_spacing = size * 0.12;
    let start_x = rect.min.x + size * 0.15;
    let control_y = rect.min.y + size * 0.12;

    painter.circle_filled(
        pos2(start_x, control_y),
        dot_radius,
        Color32::from_rgb(0xef, 0x44, 0x44),
    ); // red
    painter.circle_filled(
        pos2(start_x + dot_spacing, control_y),
        dot_radius,
        Color32::from_rgb(0xea, 0x58, 0x0c),
    ); // orange
    painter.circle_filled(
        pos2(start_x + dot_spacing * 2.0, control_y),
        dot_radius,
        Color32::from_rgb(0x22, 0xc5, 0x5e),
    ); // green

    // Draw small inner window accent using fallback_color
    let inner_rect = Rect::from_min_max(
        pos2(rect.min.x + size * 0.2, header_y + size * 0.2),
        pos2(rect.max.x - size * 0.2, rect.max.y - size * 0.2),
    );
    painter.rect_stroke(inner_rect, 1.0, Stroke::new(0.8, fallback_color));
}

pub fn draw_app_icon(
    ui: &mut egui::Ui,
    icon: &str,
    center: Pos2,
    radius: f32,
    fallback_color: Color32,
) {
    if let Some(tex) = get_app_icon_texture(ui.ctx(), icon) {
        let rect = Rect::from_center_size(center, vec2(radius * 2.0, radius * 2.0));
        // Draw real texture
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        // Draw generic app icon
        draw_generic_app_icon(ui.painter(), center, radius, fallback_color);
    }
}
