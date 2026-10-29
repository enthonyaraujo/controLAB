//! ControLAB palette and typography, preserved from the established design.
//! Sizes are logical pixels, which egui treats as points.
use eframe::egui::{self, Color32, FontFamily, FontId};

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}
const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied_const(r, g, b, (a * 255.0) as u8)
}

/// Foreground, background and border of a semantic badge.
#[derive(Clone, Copy)]
pub struct Tone {
    pub fg: Color32,
    pub bg: Color32,
    pub border: Color32,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub light: bool,
    pub bg: Color32,
    pub surface: Color32,
    pub header: Color32,
    pub inset: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub faint: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_contrast: Color32,
    /// Border of primary buttons and the mini segmented control.
    pub accent_border: Color32,
    /// Primary buttons (Calcular, `.sci-btn-mini`) stay teal in both themes.
    pub primary: Color32,
    pub primary_hover: Color32,
    pub shell: Color32,
    pub on_shell: Color32,
    pub on_shell_muted: Color32,
    pub shell_hover: Color32,
    pub valid: Color32,
    pub error: Color32,
    pub emerald: Color32,
    pub purple: Color32,
    pub amber: Color32,
    /// `--success` fallback used by the Studio run icon and comments.
    pub success: Color32,
    pub string: Color32,
    pub stable: Tone,
    pub marginal: Tone,
    pub unstable: Tone,
    pub dependent: Tone,
    /// Highlighted LaTeX card and step header tints.
    pub accent_tint: Color32,
    pub step_header: Color32,
    pub latex_card: Color32,
    pub table_head_fg: Color32,
    pub table_head_bg: Color32,
    pub table_cell_bg: Color32,
    pub table_border: Color32,
    pub soft_border: Color32,
    pub pole_tint: Color32,
    pub pole_border: Color32,
    pub toast: Color32,
}

pub const LIGHT: Palette = Palette {
    light: true,
    bg: rgb(244, 246, 248),
    surface: rgb(255, 255, 255),
    header: rgb(235, 238, 242),
    inset: rgb(248, 250, 252),
    border: rgb(203, 211, 220),
    border_strong: rgb(180, 191, 203),
    text: rgb(14, 20, 27),
    muted: rgb(82, 96, 113),
    faint: rgb(120, 135, 150),
    accent: rgb(14, 106, 153),
    accent_hover: rgb(12, 88, 107),
    accent_contrast: rgb(255, 255, 255),
    accent_border: rgb(22, 122, 148),
    primary: rgb(14, 106, 153),
    primary_hover: rgb(12, 88, 107),
    shell: rgb(14, 106, 153),
    on_shell: rgb(255, 255, 255),
    on_shell_muted: rgba(255, 255, 255, 0.85),
    shell_hover: rgba(255, 255, 255, 0.16),
    valid: rgb(5, 150, 105),
    error: rgb(220, 38, 38),
    emerald: rgb(5, 150, 105),
    purple: rgb(109, 40, 217),
    amber: rgb(180, 83, 9),
    success: rgb(24, 132, 75),
    string: rgb(175, 105, 16),
    stable: Tone {
        fg: rgb(4, 120, 87),
        bg: rgb(236, 253, 245),
        border: rgb(167, 243, 208),
    },
    marginal: Tone {
        fg: rgb(180, 83, 9),
        bg: rgb(255, 251, 235),
        border: rgb(253, 230, 138),
    },
    unstable: Tone {
        fg: rgb(185, 28, 28),
        bg: rgb(254, 242, 242),
        border: rgb(254, 202, 202),
    },
    dependent: Tone {
        fg: rgb(29, 78, 216),
        bg: rgb(239, 246, 255),
        border: rgb(191, 219, 254),
    },
    accent_tint: rgb(234, 242, 245),
    step_header: rgb(240, 242, 245),
    latex_card: rgba(0, 0, 0, 0.02),
    table_head_fg: rgb(51, 65, 85),
    table_head_bg: rgb(241, 245, 249),
    table_cell_bg: rgb(248, 250, 252),
    table_border: rgb(226, 232, 240),
    soft_border: rgb(226, 232, 240),
    pole_tint: rgb(229, 239, 243),
    pole_border: rgb(128, 171, 186),
    toast: rgb(235, 235, 235),
};

pub const DARK: Palette = Palette {
    light: false,
    bg: rgb(18, 18, 18),
    surface: rgb(26, 26, 26),
    header: rgb(35, 35, 35),
    inset: rgb(20, 20, 20),
    border: rgb(43, 43, 43),
    border_strong: rgb(69, 69, 69),
    text: rgb(218, 218, 218),
    muted: rgb(154, 154, 154),
    faint: rgb(130, 130, 130),
    accent: rgb(95, 180, 224),
    accent_hover: rgb(20, 120, 171),
    accent_contrast: rgb(255, 255, 255),
    accent_border: rgb(14, 106, 153),
    primary: rgb(14, 106, 153),
    primary_hover: rgb(20, 120, 171),
    shell: rgb(26, 26, 26),
    on_shell: rgb(230, 230, 230),
    on_shell_muted: rgba(255, 255, 255, 0.85),
    shell_hover: rgba(255, 255, 255, 0.14),
    valid: rgb(16, 185, 129),
    error: rgb(255, 107, 97),
    emerald: rgb(16, 185, 129),
    purple: rgb(139, 92, 246),
    amber: rgb(224, 160, 64),
    success: rgb(24, 132, 75),
    string: rgb(175, 105, 16),
    stable: Tone {
        fg: rgb(52, 211, 153),
        bg: rgba(16, 185, 129, 0.12),
        border: rgba(16, 185, 129, 0.35),
    },
    marginal: Tone {
        fg: rgb(251, 191, 36),
        bg: rgba(245, 158, 11, 0.12),
        border: rgba(245, 158, 11, 0.35),
    },
    unstable: Tone {
        fg: rgb(248, 113, 113),
        bg: rgba(239, 68, 68, 0.12),
        border: rgba(239, 68, 68, 0.35),
    },
    dependent: Tone {
        fg: rgb(147, 197, 253),
        bg: rgba(37, 99, 235, 0.12),
        border: rgba(37, 99, 235, 0.35),
    },
    accent_tint: rgb(31, 54, 68),
    step_header: rgb(35, 35, 35),
    latex_card: rgba(0, 0, 0, 0.15),
    table_head_fg: rgb(154, 154, 154),
    table_head_bg: rgb(26, 26, 26),
    table_cell_bg: rgb(28, 28, 28),
    table_border: rgb(43, 43, 43),
    soft_border: rgb(43, 43, 43),
    pole_tint: rgb(31, 54, 68),
    pole_border: rgb(22, 74, 102),
    toast: rgb(28, 28, 28),
};

pub fn palette(light: bool) -> Palette {
    if light { LIGHT } else { DARK }
}

/// CSS font weights mapped to the bundled IBM Plex Sans / JetBrains Mono files.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    Regular,
    Medium,
    SemiBold,
    Bold,
}

pub fn sans(size: f32, weight: Weight) -> FontId {
    let name = match weight {
        Weight::Regular => return FontId::proportional(size),
        Weight::Medium => "plex-500",
        Weight::SemiBold => "plex-600",
        Weight::Bold => "plex-700",
    };
    FontId::new(size, FontFamily::Name(name.into()))
}

pub fn mono(size: f32, weight: Weight) -> FontId {
    let name = match weight {
        Weight::Regular => return FontId::monospace(size),
        Weight::Medium => "mono-500",
        Weight::SemiBold | Weight::Bold => "mono-700",
    };
    FontId::new(size, FontFamily::Name(name.into()))
}

macro_rules! font_file {
    ($path:literal) => {
        egui::FontData::from_static(include_bytes!(concat!("../../assets/fonts/", $path)))
    };
}

pub fn fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for (name, data) in [
        (
            "plex-400",
            font_file!("ibm-plex-sans/IBMPlexSans-Regular.ttf"),
        ),
        (
            "plex-500",
            font_file!("ibm-plex-sans/IBMPlexSans-Medium.ttf"),
        ),
        (
            "plex-600",
            font_file!("ibm-plex-sans/IBMPlexSans-SemiBold.ttf"),
        ),
        ("plex-700", font_file!("ibm-plex-sans/IBMPlexSans-Bold.ttf")),
        (
            "mono-400",
            font_file!("jetbrains-mono/JetBrainsMono-Regular.ttf"),
        ),
        (
            "mono-500",
            font_file!("jetbrains-mono/JetBrainsMono-Medium.ttf"),
        ),
        (
            "mono-700",
            font_file!("jetbrains-mono/JetBrainsMono-Bold.ttf"),
        ),
    ] {
        fonts.font_data.insert(name.into(), data.into());
    }
    // Math symbols in result text (⇔, ∠, ℝ, ●) exist only in the monospace fonts.
    let fallbacks = [
        "mono-400",
        "Hack",
        "Ubuntu-Light",
        "NotoEmoji-Regular",
        "emoji-icon-font",
    ];
    let chain = |first: &str| -> Vec<String> {
        std::iter::once(first)
            .chain(fallbacks.iter().copied().filter(|f| *f != first))
            .map(str::to_owned)
            .collect()
    };
    fonts
        .families
        .insert(FontFamily::Proportional, chain("plex-400"));
    fonts
        .families
        .insert(FontFamily::Monospace, chain("mono-400"));
    for name in ["plex-500", "plex-600", "plex-700", "mono-500", "mono-700"] {
        fonts
            .families
            .insert(FontFamily::Name(name.into()), chain(name));
    }
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context, light: bool) {
    let p = palette(light);
    let mut v = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    v.panel_fill = p.surface;
    v.window_fill = p.surface;
    v.window_stroke = egui::Stroke::new(1.0, p.border_strong);
    v.window_corner_radius = egui::CornerRadius::same(6);
    v.faint_bg_color = p.header;
    v.extreme_bg_color = p.inset;
    v.code_bg_color = p.inset;
    v.override_text_color = None;
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = p.accent.gamma_multiply(0.25);
    v.selection.stroke = egui::Stroke::new(1.0, p.accent);
    v.text_cursor.stroke = egui::Stroke::new(1.5, p.text);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, p.text);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, p.border);
    v.widgets.noninteractive.bg_fill = p.surface;
    for (w, fill) in [
        (&mut v.widgets.inactive, p.header),
        (&mut v.widgets.hovered, p.inset),
        (&mut v.widgets.active, p.inset),
        (&mut v.widgets.open, p.inset),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = egui::Stroke::new(1.0, p.border);
        w.fg_stroke = egui::Stroke::new(1.0, p.text);
        w.corner_radius = egui::CornerRadius::same(3);
        w.expansion = 0.0;
    }
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, p.border_strong);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, p.accent);
    v.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(if light { 50 } else { 100 }),
    };
    v.window_shadow = egui::epaint::Shadow {
        offset: [0, 20],
        blur: 45,
        spread: 0,
        color: Color32::from_black_alpha(if light { 40 } else { 140 }),
    };
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 6.0);
        s.spacing.button_padding = egui::vec2(8.0, 4.0);
        // `::-webkit-scrollbar { width: 6px }`: solid bars that take layout width.
        s.spacing.scroll = egui::style::ScrollStyle {
            bar_width: 6.0,
            bar_inner_margin: 2.0,
            bar_outer_margin: 0.0,
            ..egui::style::ScrollStyle::solid()
        };
        s.interaction.selectable_labels = false;
        s.text_styles
            .insert(egui::TextStyle::Body, sans(13.6, Weight::Regular));
        s.text_styles
            .insert(egui::TextStyle::Button, sans(13.0, Weight::Regular));
        s.text_styles
            .insert(egui::TextStyle::Small, sans(11.0, Weight::Regular));
        s.text_styles
            .insert(egui::TextStyle::Monospace, mono(13.0, Weight::Regular));
        s.text_styles
            .insert(egui::TextStyle::Heading, sans(20.0, Weight::SemiBold));
    });
}

/// MATLAB highlighting with the previous editor's classes: comments, strings and numbers.
pub fn highlight(source: &str, light: bool) -> egui::text::LayoutJob {
    let p = palette(light);
    let font = mono(13.0, Weight::Regular);
    let mut job = egui::text::LayoutJob::default();
    let format = |color| egui::TextFormat {
        font_id: font.clone(),
        color,
        line_height: Some(21.0),
        ..Default::default()
    };
    for line in source.split_inclusive('\n') {
        let chars: Vec<(usize, char)> = line.char_indices().collect();
        let mut i = 0;
        let mut plain_start = 0;
        let flush = |job: &mut egui::text::LayoutJob, from: usize, to: usize| {
            if to > from {
                job.append(&line[from..to], 0.0, format(p.text));
            }
        };
        while i < chars.len() {
            let (at, c) = chars[i];
            let previous = if i > 0 { Some(chars[i - 1].1) } else { None };
            if c == '%' {
                flush(&mut job, plain_start, at);
                job.append(&line[at..], 0.0, format(p.success));
                plain_start = line.len();
                break;
            }
            // A quote after an identifier or bracket is the transpose operator.
            let transpose = previous.is_some_and(|q| q.is_alphanumeric() || ")]}'_.".contains(q));
            if c == '"' || (c == '\'' && !transpose) {
                let end = chars[i + 1..]
                    .iter()
                    .find(|(_, q)| *q == c || *q == '\n')
                    .map_or(line.len(), |(e, q)| if *q == c { e + 1 } else { *e });
                flush(&mut job, plain_start, at);
                job.append(&line[at..end], 0.0, format(p.string));
                plain_start = end;
                while i < chars.len() && chars[i].0 < end {
                    i += 1;
                }
                continue;
            }
            let boundary = previous.is_none_or(|q| !(q.is_alphanumeric() || q == '_'));
            if c.is_ascii_digit() && boundary {
                let end = chars[i..]
                    .iter()
                    .find(|(_, q)| !(q.is_ascii_digit() || *q == '.'))
                    .map_or(line.len(), |(e, _)| *e);
                flush(&mut job, plain_start, at);
                job.append(&line[at..end], 0.0, format(p.accent));
                plain_start = end;
                while i < chars.len() && chars[i].0 < end {
                    i += 1;
                }
                continue;
            }
            i += 1;
        }
        flush(&mut job, plain_start, line.len());
    }
    job
}
