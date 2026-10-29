//! The previous renderer's SVG icons, rasterized with resvg at the display density.
//! `currentColor` is replaced by the requested color; textures are cached per variant.
use eframe::egui::{self, Color32};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

const fn stroke(width: &'static str, body: &'static str) -> (&'static str, &'static str) {
    (width, body)
}

/// Lucide-style stroke icons: (stroke width, inner SVG).
pub type Icon = (&'static str, &'static str);

pub const GEAR: Icon = stroke(
    "2",
    r#"<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>"#,
);

// Studio ribbon (`.studio-tool-group svg`: stroke 1.5).
pub const NEW_SCRIPT: Icon = stroke(
    "1.5",
    r#"<path d="M6 3h9l4 4v14H6zM14 3v5h5M9 12h7M9 16h7"/>"#,
);
pub const OPEN: Icon = stroke(
    "1.5",
    r#"<path d="M3 7h7l2 3h9l-2 10H3zM3 7V4h7l2 3h8v3"/>"#,
);
pub const SAVE: Icon = stroke(
    "1.5",
    r#"<path d="M4 3h14l3 3v15H3V3zM7 3v6h10V3M7 21v-8h10v8"/>"#,
);
pub const VARIABLES: Icon = stroke(
    "1.5",
    r#"<path d="M3 3h18v18H3zM3 9h18M3 15h18M9 3v18M15 3v18"/>"#,
);
pub const CLEAR: Icon = stroke("1.5", r#"<path d="m6 6 12 12M18 6 6 18"/>"#);
pub const LAYOUT: Icon = stroke("1.5", r#"<path d="M3 3h18v18H3zM8 3v18M16 3v18M8 16h8"/>"#);
pub const HELP: Icon = stroke(
    "1.5",
    r#"<circle cx="12" cy="12" r="9"/><path d="M9 8a3 3 0 0 1 6 1c0 2-3 2-3 4M12 16v1"/>"#,
);
/// `.studio-run svg` is filled with the success color.
pub const RUN_FILLED: Icon = stroke("1.5", r#"<path d="m7 3 15 9-15 9z" fill="currentColor"/>"#);

// Welcome actions (stroke 1.6).
pub const WELCOME_NEW: Icon = stroke(
    "1.6",
    r#"<path d="M6 3h9l4 4v14H6zM14 3v5h5M12 11v6M9 14h6"/>"#,
);
pub const WELCOME_OPEN: Icon = stroke(
    "1.6",
    r#"<path d="M3 7h7l2 3h9l-2 10H3zM3 7V4h7l2 3h8v3"/>"#,
);

// Module symbols (`#s-*`, stroke 1.75).
pub const S_HOME: Icon = stroke(
    "1.75",
    r#"<path d="M3 10.5L12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1v-9.5z"/>"#,
);
pub const S_TIME: Icon = stroke(
    "1.75",
    r#"<path d="M3 3v18h18"/><path d="M5 18C6.5 18 7 7.5 10 7.5c2 0 2.5 5 4.5 5 1.6 0 2.2-1.5 5.5-1.5"/><path d="M5 11h15" stroke-dasharray="1.5 2.5" stroke-width="1.25"/>"#,
);
pub const S_STAB: Icon = stroke(
    "1.75",
    r#"<path d="M3 12h18"/><path d="M13 3v18"/><path d="M5.5 6.5l3 3m0-3l-3 3"/><path d="M5.5 14.5l3 3m0-3l-3 3"/>"#,
);
pub const S_ROOT: Icon = stroke(
    "1.75",
    r#"<path d="M3 12h2"/><path d="M7 12h4"/><path d="M11 12c2 0 2.5-5 5-8"/><path d="M11 12c2 0 2.5 5 5 8"/><path d="M18.5 4v16" stroke-dasharray="1.5 2.5" stroke-width="1.25"/><path d="M5 10.5l2 3m0-3l-2 3"/>"#,
);
pub const S_FREQ: Icon = stroke(
    "1.75",
    r#"<path d="M3 3v18h18"/><path d="M6 8h5c2 0 3 1 4 3l3 6"/><path d="M6 17h4" stroke-dasharray="1.5 2.5" stroke-width="1.25"/>"#,
);
pub const S_CTRL: Icon = stroke(
    "1.75",
    r#"<circle cx="6" cy="10" r="2"/><path d="M2 10h2"/><path d="M8 10h3"/><rect x="11" y="6.5" width="8" height="7" rx="1.5"/><path d="M19 10h3"/><path d="M21 10v8H6v-6"/><path d="M4.8 13.2L6 12l1.2 1.2"/>"#,
);
pub const S_STATE: Icon = stroke(
    "1.75",
    r#"<path d="M8 4H5v16h3"/><path d="M16 4h3v16h-3"/><circle cx="10" cy="9.5" r="0.9" fill="currentColor"/><circle cx="14" cy="9.5" r="0.9" fill="currentColor"/><circle cx="10" cy="14.5" r="0.9" fill="currentColor"/><circle cx="14" cy="14.5" r="0.9" fill="currentColor"/>"#,
);

// Scientific ribbon (stroke 2).
pub const CALC: Icon = stroke(
    "0",
    r#"<polygon points="5 3 19 12 5 21 5 3" fill="currentColor"/>"#,
);
pub const ERASER: Icon = stroke(
    "2",
    r#"<path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21"/><path d="M22 21H7"/><path d="m5 11 9 9"/>"#,
);
pub const ZOOM_IN: Icon = stroke(
    "2",
    r#"<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/>"#,
);
pub const ZOOM_OUT: Icon = stroke(
    "2",
    r#"<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="8" y1="11" x2="14" y2="11"/>"#,
);
pub const FIT: Icon = stroke(
    "2",
    r#"<polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><polyline points="21 15 21 21 15 21"/><polyline points="3 9 3 3 9 3"/>"#,
);
pub const GRID: Icon = stroke(
    "2",
    r#"<circle cx="5" cy="5" r="1.5"/><circle cx="12" cy="5" r="1.5"/><circle cx="19" cy="5" r="1.5"/><circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/><circle cx="5" cy="19" r="1.5"/><circle cx="12" cy="19" r="1.5"/><circle cx="19" cy="19" r="1.5"/>"#,
);
pub const EXPORT: Icon = stroke(
    "2",
    r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>"#,
);
pub const CODE: Icon = stroke(
    "2",
    r#"<polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>"#,
);

// Routh/LGR ribbon (stroke 1.75).
pub const X_THIN: Icon = stroke("1.75", r#"<path d="M18 6L6 18M6 6l12 12"/>"#);
pub const ZOOM_IN_THIN: Icon = stroke("1.75", ZOOM_IN.1);
pub const ZOOM_OUT_THIN: Icon = stroke("1.75", ZOOM_OUT.1);
pub const RESET: Icon = stroke(
    "1.75",
    r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>"#,
);
pub const DOWNLOAD_THIN: Icon = stroke("1.75", EXPORT.1);
pub const CODE_THIN: Icon = stroke("1.75", CODE.1);
pub const COPY: Icon = stroke(
    "1.75",
    r#"<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>"#,
);

pub const REFRESH: Icon = stroke(
    "2",
    r#"<polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>"#,
);
pub const EXTERNAL: Icon = stroke(
    "2",
    r#"<line x1="7" y1="17" x2="17" y2="7"/><polyline points="7 7 17 7 17 17"/>"#,
);
pub const GITHUB: Icon = stroke(
    "0",
    r#"<path fill="currentColor" d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.020-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>"#,
);
pub const INFO: Icon = stroke(
    "1.5",
    r#"<circle cx="12" cy="12" r="9"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>"#,
);
pub const CHEVRON: Icon = stroke("2", r#"<polyline points="6 9 12 15 18 9"/>"#);
pub const CHECK: Icon = stroke("2.5", r#"<polyline points="20 6 9 17 4 12"/>"#);
pub const PIN: Icon = stroke(
    "2",
    r#"<path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/>"#,
);
pub const BOLT: Icon = stroke("2", r#"<path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/>"#);
pub const ARROW_UP: Icon = stroke(
    "2",
    r#"<circle cx="12" cy="12" r="10"/><path d="m16 12-4-4-4 4"/><path d="M12 16V8"/>"#,
);
pub const TARGET: Icon = stroke(
    "2",
    r#"<circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/>"#,
);

/// Folder navigation icons of the Studio, with their two-tone fills.
pub fn folder_icon(kind: &str, p: &crate::theme::Palette) -> String {
    let accent = hex(p.accent);
    let fill = format!(
        "rgba({},{},{},0.15)",
        p.accent.r(),
        p.accent.g(),
        p.accent.b()
    );
    let folder = format!(
        r#"<path d="M3 7h7l2 3h9l-2 10H3zM3 7V4h7l2 3h8v3" stroke="{}" fill="rgba(180,139,49,0.2)"/>"#,
        hex(p.muted)
    );
    let body = match kind {
        "back" => r#"<path d="M14 5 5 12l9 7v-5h6v-4h-6z"/>"#.to_owned(),
        "forward" => r#"<path d="m10 5 9 7-9 7v-5H4v-4h6z"/>"#.to_owned(),
        "up" => format!(r#"{folder}<path d="M12 18v-6m-3 3 3-3 3 3"/>"#),
        _ => format!(
            r#"{folder}<path d="m10 15 2 2 5-5" stroke="{}" fill="none" stroke-width="2"/>"#,
            hex(p.success)
        ),
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="{fill}" stroke="{accent}" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">{body}</svg>"#
    )
}

fn hex(c: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

pub fn svg(icon: Icon) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
        icon.0, icon.1
    )
}

type Cache = Arc<Mutex<HashMap<u64, Option<egui::TextureHandle>>>>;

/// Texture for `source` at `size` points, rasterized at the current pixel density.
pub fn texture(
    ctx: &egui::Context,
    source: &str,
    color: Color32,
    size: f32,
) -> Option<egui::TextureHandle> {
    let pixels = (size * ctx.pixels_per_point()).round().max(1.0) as u32;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (source, color.to_array(), pixels).hash(&mut hasher);
    let key = hasher.finish();
    let cache: Cache = ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Cache>(egui::Id::new("icons"))
            .clone()
    });
    if let Some(found) = cache.lock().unwrap().get(&key) {
        return found.clone();
    }
    let alpha = color.a() as f32 / 255.0;
    let colored = source.replace(
        "currentColor",
        &format!("rgba({},{},{},{alpha})", color.r(), color.g(), color.b()),
    );
    let texture = rasterize(&colored, pixels).map(|image| {
        ctx.load_texture(format!("icon-{key:x}"), image, egui::TextureOptions::LINEAR)
    });
    cache.lock().unwrap().insert(key, texture.clone());
    texture
}

fn rasterize(source: &str, pixels: u32) -> Option<egui::ColorImage> {
    use resvg::{tiny_skia, usvg};
    let tree = usvg::Tree::from_str(source, &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(pixels, pixels)?;
    let scale = pixels as f32 / tree.size().width().max(tree.size().height());
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Some(egui::ColorImage::from_rgba_premultiplied(
        [pixels as usize, pixels as usize],
        pixmap.data(),
    ))
}

/// Paints `icon` centered in `rect`.
pub fn paint(ui: &egui::Ui, rect: egui::Rect, icon: Icon, color: Color32) {
    paint_svg(ui, rect, &svg(icon), color);
}

pub fn paint_svg(ui: &egui::Ui, rect: egui::Rect, source: &str, color: Color32) {
    let size = rect.width().min(rect.height());
    if let Some(texture) = texture(ui.ctx(), source, color, size) {
        let square = egui::Rect::from_center_size(rect.center(), egui::vec2(size, size));
        ui.painter().image(
            texture.id(),
            square,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_icon_rasterizes_with_visible_pixels() {
        let p = crate::theme::LIGHT;
        let mut sources: Vec<String> = [
            GEAR,
            NEW_SCRIPT,
            OPEN,
            SAVE,
            VARIABLES,
            CLEAR,
            LAYOUT,
            HELP,
            RUN_FILLED,
            WELCOME_NEW,
            WELCOME_OPEN,
            S_TIME,
            S_STAB,
            S_ROOT,
            S_FREQ,
            S_CTRL,
            S_STATE,
            CALC,
            ERASER,
            ZOOM_IN,
            ZOOM_OUT,
            FIT,
            GRID,
            EXPORT,
            CODE,
            X_THIN,
            RESET,
            COPY,
            INFO,
            CHEVRON,
            CHECK,
            PIN,
            BOLT,
            ARROW_UP,
            TARGET,
            REFRESH,
            EXTERNAL,
            GITHUB,
        ]
        .into_iter()
        .map(svg)
        .collect();
        sources.extend(["back", "forward", "up", "select"].map(|k| folder_icon(k, &p)));
        for source in sources {
            let image = rasterize(&source.replace("currentColor", "#0f6e86"), 24)
                .unwrap_or_else(|| panic!("invalid SVG: {source}"));
            assert!(
                image.pixels.iter().any(|px| px.a() > 0),
                "blank icon: {source}"
            );
        }
    }
}
