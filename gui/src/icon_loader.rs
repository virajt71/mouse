use eframe::egui;
use egui::{pos2, vec2, Color32, Pos2, Rect, Stroke, TextureHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

thread_local! {
    static ICON_CACHE: RefCell<HashMap<String, Option<TextureHandle>>> = RefCell::new(HashMap::new());
}

use std::sync::OnceLock;

fn icon_theme_roots() -> &'static [PathBuf] {
    static ROOTS: OnceLock<Vec<PathBuf>> = OnceLock::new();
    ROOTS.get_or_init(|| {
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
    })
}

fn load_svg(path: &Path, icon: &str, ctx: &egui::Context) -> Option<TextureHandle> {
    let svg_data = std::fs::read(path).ok()?;
    render_svg_bytes(&svg_data, &format!("app_icon_{}", icon), ctx)
}

/// Rasterize SVG bytes via resvg into an egui texture. Shared by disk-loaded
/// app icons and compiled-in bundled glyphs.
fn render_svg_bytes(
    svg_data: &[u8],
    texture_key: &str,
    ctx: &egui::Context,
) -> Option<TextureHandle> {
    let tree = resvg::usvg::Tree::from_data(svg_data, &resvg::usvg::Options::default()).ok()?;
    let tree_size = tree.size();
    let scale = (256.0 / tree_size.width().max(tree_size.height())).min(1.0);
    let w = (tree_size.width() * scale) as u32;
    let h = (tree_size.height() * scale) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w.max(1), h.max(1))?;
    let transform = resvg::tiny_skia::Transform::from_scale(scale as f32, scale as f32);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let color_img = egui::ColorImage::from_rgba_unmultiplied(
        [pixmap.width() as usize, pixmap.height() as usize],
        pixmap.data(),
    );
    Some(ctx.load_texture(texture_key, color_img, egui::TextureOptions::default()))
}

thread_local! {
    static BUNDLED_ICON_CACHE: RefCell<HashMap<&'static str, Option<TextureHandle>>> =
        RefCell::new(HashMap::new());
}

/// Render a compiled-in SVG asset once and cache it, keyed by name. Mirrors
/// the app-icon texture cache; bundled glyphs stay crisp at any display size
/// instead of being hand-drawn lines.
pub fn get_bundled_icon_texture(
    ctx: &egui::Context,
    key: &'static str,
    svg_bytes: &'static [u8],
) -> Option<TextureHandle> {
    if let Some(cached) = BUNDLED_ICON_CACHE.with(|c| c.borrow().get(key).cloned()) {
        return cached;
    }
    let tex = render_svg_bytes(svg_bytes, key, ctx);
    BUNDLED_ICON_CACHE.with(|c| c.borrow_mut().insert(key, tex.clone()));
    tex
}

pub fn resolve_icon_path(icon: &str) -> Option<PathBuf> {
    if icon.is_empty() {
        return None;
    }

    let path = Path::new(icon);
    if path.is_absolute() && path.exists() {
        return Some(path.to_path_buf());
    }

    let themes = icon_theme_roots();
    let sizes = [
        "512x512",
        "512x512@2x",
        "256x256",
        "256x256@2x",
        "128x128",
        "128x128@2x",
        "96x96",
        "96x96@2x",
        "64x64",
        "64x64@2x",
        "48x48",
        "48x48@2x",
        "32x32",
        "32x32@2x",
        "24x24",
        "24x24@2x",
        "16x16",
        "16x16@2x",
    ];
    let categories = [
        "apps",
        "devices",
        "categories",
        "mimetypes",
        "places",
        "status",
    ];

    for theme_dir in themes {
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

        // Search scalable/ dirs for SVG icons
        for category in &categories {
            let base = theme_dir.join("scalable").join(category);
            let svg = base.join(format!("{}.svg", icon));
            if svg.exists() {
                return Some(svg);
            }
        }

        for category in &categories {
            let base = theme_dir.join(category);
            for ext in &["", ".png", ".svg"] {
                let candidate = base.join(format!("{icon}{ext}"));
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    let base_pix = PathBuf::from("/usr/share/pixmaps");
    for ext in &["", ".png", ".svg"] {
        let candidate = base_pix.join(format!("{icon}{ext}"));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    log::debug!(
        "[icon_loader] could not resolve icon '{}' in any installed theme",
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
        let is_svg = path.extension().is_some_and(|e| e == "svg");
        if is_svg {
            load_svg(&path, icon, ctx)
        } else if let Ok(bytes) = std::fs::read(&path) {
            image::load_from_memory(&bytes).ok().map(|img| {
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
                tex
            })
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
