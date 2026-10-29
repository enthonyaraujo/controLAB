//! Widgets reproducing the previous renderer's components with their computed metrics.
use crate::icons::{self, Icon};
use crate::theme::{Palette, Weight, mono, sans};
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2,
};

/// Letter subscripts/superscripts absent from the bundled fonts, drawn as small letters.
fn script_letter(c: char) -> Option<(char, bool)> {
    let subscripts = "ₐₑₒₓₕₖₗₘₙₚₛₜᵢⱼᵣᵤᵥ";
    let letters = "aeoxhklmnpstijruv";
    if let Some(i) = subscripts.chars().position(|s| s == c) {
        return letters.chars().nth(i).map(|l| (l, false));
    }
    match c {
        'ⁿ' => Some(('n', true)),
        'ⁱ' => Some(('i', true)),
        _ => None,
    }
}

pub fn job(text: &str, font: FontId, color: Color32, spacing: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let format = egui::TextFormat {
        font_id: font.clone(),
        color,
        extra_letter_spacing: spacing,
        ..Default::default()
    };
    if !text.chars().any(|c| script_letter(c).is_some()) {
        job.append(text, 0.0, format);
        return job;
    }
    let mut plain = String::new();
    for c in text.chars() {
        match script_letter(c) {
            Some((letter, sup)) => {
                job.append(&std::mem::take(&mut plain), 0.0, format.clone());
                let small = FontId::new(font.size * 0.7, font.family.clone());
                let valign = if sup {
                    egui::Align::TOP
                } else {
                    egui::Align::BOTTOM
                };
                job.append(
                    &letter.to_string(),
                    0.0,
                    egui::TextFormat {
                        font_id: small,
                        valign,
                        ..format.clone()
                    },
                );
            }
            None => plain.push(c),
        }
    }
    job.append(&plain, 0.0, format);
    job
}

pub fn galley(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    spacing: f32,
) -> std::sync::Arc<egui::Galley> {
    ui.fonts_mut(|f| f.layout_job(job(text, font, color, spacing)))
}

/// Single line truncated with `…`, as `text-overflow: ellipsis`.
pub fn ellipsized(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    spacing: f32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = job(text, font, color, spacing);
    job.wrap = egui::text::TextWrapping {
        max_width: width,
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    ui.fonts_mut(|f| f.layout_job(job))
}

pub fn wrapped(
    ui: &Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = job(text, font, color, 0.0);
    job.wrap.max_width = width;
    ui.fonts_mut(|f| f.layout_job(job))
}

/// A non-interactive label with exact font, color and letter spacing.
pub fn text(ui: &mut Ui, text: &str, font: FontId, color: Color32) -> Response {
    spaced(ui, text, font, color, 0.0)
}

pub fn spaced(ui: &mut Ui, text: &str, font: FontId, color: Color32, spacing: f32) -> Response {
    let g = galley(ui, text, font, color, spacing);
    let (rect, response) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    response
}

/// Synthetic italic label (egui slants the regular face).
pub fn italic(ui: &mut Ui, text: &str, font: FontId, color: Color32) -> Response {
    let mut job = job(text, font, color, 0.0);
    job.sections[0].format.italics = true;
    let g = ui.fonts_mut(|f| f.layout_job(job));
    let (rect, response) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    response
}

/// A wrapping paragraph that fills the available width.
pub fn paragraph(ui: &mut Ui, text: &str, font: FontId, color: Color32) -> Response {
    let g = wrapped(ui, text, font, color, ui.available_width());
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), g.size().y), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    response
}

pub fn fill(ui: &Ui, rect: Rect, radius: impl Into<CornerRadius>, fill: Color32, stroke: Stroke) {
    ui.painter()
        .rect(rect, radius, fill, stroke, egui::StrokeKind::Inside);
}

pub fn hline(ui: &Ui, x: impl Into<egui::Rangef>, y: f32, color: Color32) {
    ui.painter().hline(x, y + 0.5, Stroke::new(1.0, color));
}

pub fn vline(ui: &Ui, x: f32, y: impl Into<egui::Rangef>, color: Color32) {
    ui.painter().vline(x + 0.5, y, Stroke::new(1.0, color));
}

/// Frame with the `.routh-card` / card geometry.
pub fn frame(
    fill: Color32,
    border: Color32,
    radius: u8,
    margin: impl Into<egui::Margin>,
) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(radius)
        .inner_margin(margin)
}

// ---------------------------------------------------------------------------
// Shell

pub const TABS: [(&str, &str); 7] = [
    ("home", "INÍCIO"),
    ("time", "RESPOSTA NO TEMPO"),
    ("routh", "ESTABILIDADE"),
    ("lgr", "LUGAR GEOMÉTRICO DAS RAÍZES"),
    ("frequency", "RESPOSTA EM FREQUÊNCIA"),
    ("controllers", "CONTROLADORES"),
    ("state-space", "ESPAÇO DE ESTADOS"),
];

pub enum ShellAction {
    Navigate(&'static str),
    Settings,
}

/// `.home-top-bar` / `.sci-top-bar`: 46 px teal bar. Below 1024 px only the brand and the
/// settings button remain (`.sci-top-tabs { display: none }`). `k` scales rem sizes on pages
/// that are not zoomed in rem (the Studio).
pub fn top_bar(ui: &mut Ui, p: &Palette, page: &str, k: f32, tabs: bool) -> Option<ShellAction> {
    let home = page == "home";
    let rect = ui.max_rect();
    fill(ui, rect, 0, p.shell, Stroke::NONE);
    let mut action = None;
    let brand = galley(
        ui,
        "ControLAB",
        sans(16.7 * k, Weight::Bold),
        p.on_shell,
        -0.2,
    );
    let left = if !home {
        25.0
    } else if tabs {
        17.0
    } else if layout(ui).mode == crate::Mode::Phone {
        29.0
    } else {
        17.0
    };
    let mut x = rect.left() + left * k;
    ui.painter().galley(
        pos2(x, rect.center().y - brand.size().y / 2.0),
        brand.clone(),
        p.on_shell,
    );
    x += brand.size().x + if home { 12.0 } else { 20.0 } * k;
    let room = (rect.right() - if home { 50.0 * k } else { 42.0 * k } - x - 8.0).max(1.0);
    let labels_width: f32 = TABS
        .iter()
        .map(|(id, label)| {
            galley(
                ui,
                label,
                sans(
                    if home { 12.5 } else { 12.0 } * k,
                    if *id == page || home {
                        Weight::Bold
                    } else {
                        Weight::SemiBold
                    },
                ),
                p.on_shell,
                0.6,
            )
            .size()
            .x
        })
        .sum();
    let padding = ((room - labels_width - 14.0) / 7.0).clamp(8.0 * k, 29.2 * k);
    let tab_scale = (room / (labels_width + 7.0 * (padding + 2.0))).min(1.0);
    let radius = CornerRadius {
        nw: 4,
        ne: 4,
        sw: 0,
        se: 0,
    };
    for (id, label) in TABS.iter().copied().filter(|_| tabs) {
        let active = page == id;
        let font = sans(
            if home { 12.5 } else { 12.0 } * k * tab_scale,
            if active || home {
                Weight::Bold
            } else {
                Weight::SemiBold
            },
        );
        let g = galley(ui, label, font, p.on_shell, 0.6 * tab_scale);
        let tab = Rect::from_min_size(
            pos2(x, rect.top()),
            vec2(g.size().x + padding * tab_scale, rect.height()),
        );
        let response = ui.interact(tab, ui.id().with(("tab", id)), Sense::click());
        if active {
            fill(ui, tab, radius, p.header, Stroke::NONE);
        } else if response.hovered() {
            fill(ui, tab, radius, p.shell_hover, Stroke::NONE);
        }
        let color = if active {
            p.accent
        } else if response.hovered() {
            p.on_shell
        } else {
            p.on_shell_muted
        };
        ui.painter().galley_with_override_text_color(
            pos2(
                tab.left() + padding * tab_scale / 2.0,
                tab.center().y - g.size().y / 2.0,
            ),
            g,
            color,
        );
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            action = Some(ShellAction::Navigate(id));
        }
        x = tab.right() + 2.0 * tab_scale;
    }
    let gear = Rect::from_min_size(
        pos2(
            rect.right() - if home { 50.0 } else { 42.0 } * k,
            rect.top() + 2.0 * k,
        ),
        vec2(33.0 * k, 42.0 * k),
    );
    let response = ui
        .interact(gear, ui.id().with("settings"), Sense::click())
        .on_hover_text("Configurações");
    if response.hovered() {
        fill(ui, gear, 4, p.shell_hover, Stroke::NONE);
    }
    let alpha = if response.hovered() { 1.0 } else { 0.85 };
    icons::paint(
        ui,
        Rect::from_center_size(gear.center(), vec2(19.0 * k, 19.0 * k)),
        icons::GEAR,
        p.on_shell.gamma_multiply(alpha),
    );
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        action = Some(ShellAction::Settings);
    }
    action
}

// ---------------------------------------------------------------------------
// Ribbons

/// `.sci-ribbon-btn`: icon over label; the primary variant is the filled Calcular button.
pub fn sci_button(
    ui: &mut Ui,
    p: &Palette,
    icon: Icon,
    label: &str,
    primary: bool,
    enabled: bool,
) -> Response {
    let font = sans(
        12.5,
        if primary {
            Weight::SemiBold
        } else {
            Weight::Medium
        },
    );
    let g = galley(ui, label, font, Color32::WHITE, 0.0);
    let size = vec2((g.size().x + 16.6).max(48.0), 52.0);
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    let alpha = if enabled { 1.0 } else { 0.45 };
    let (label_color, icon_color) = if primary {
        let bg = if hovered { p.primary_hover } else { p.primary };
        fill(ui, rect, 3, bg, Stroke::new(1.0, p.accent_border));
        (Color32::WHITE, Color32::WHITE)
    } else {
        if hovered {
            fill(ui, rect, 3, p.inset, Stroke::new(1.0, p.border_strong));
        }
        (p.muted, p.accent)
    };
    let icon_rect =
        Rect::from_center_size(pos2(rect.center().x, rect.top() + 15.2), vec2(20.0, 20.0));
    icons::paint(ui, icon_rect, icon, icon_color.gamma_multiply(alpha));
    let g = galley(
        ui,
        label,
        sans(
            12.5,
            if primary {
                Weight::SemiBold
            } else {
                Weight::Medium
            },
        ),
        label_color,
        0.0,
    );
    ui.painter().galley(
        pos2(rect.center().x - g.size().x / 2.0, rect.top() + 28.0),
        g,
        label_color.gamma_multiply(alpha),
    );
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}

pub fn ribbon_separator(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(13.0, 52.0), Sense::hover());
    fill(
        ui,
        Rect::from_center_size(rect.center(), vec2(1.0, 29.0)),
        0,
        p.border_strong,
        Stroke::NONE,
    );
}

/// `.studio-tool-group` with its caption at the bottom-left and a right divider.
pub fn studio_group(ui: &mut Ui, p: &Palette, caption: &str, add: impl FnOnce(&mut Ui)) {
    let top = ui.cursor().top();
    let inner = ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        add(ui);
        ui.add_space(6.0);
    });
    let rect = Rect::from_min_max(
        pos2(inner.response.rect.left(), top),
        pos2(
            inner.response.rect.right(),
            top + if layout(ui).css_width < 768.0 {
                65.0
            } else {
                75.0
            },
        ),
    );
    let g = galley(ui, caption, sans(10.0, Weight::Regular), p.muted, 0.0);
    ui.painter()
        .galley(pos2(rect.left() + 1.0, rect.bottom() - 15.0), g, p.muted);
    vline(
        ui,
        rect.right(),
        rect.top()..=rect.bottom(),
        p.border_strong,
    );
    ui.add_space(8.0);
}

pub fn studio_button(
    ui: &mut Ui,
    p: &Palette,
    source: &str,
    color: Color32,
    label: &str,
    enabled: bool,
) -> Response {
    let narrow = layout(ui).css_width < 768.0;
    let g = galley(
        ui,
        label,
        sans(if narrow { 10.0 } else { 11.0 }, Weight::Regular),
        p.text,
        0.0,
    );
    let size = vec2(
        (g.size().x + if narrow { 10.0 } else { 18.0 }).max(if narrow { 53.0 } else { 62.0 }),
        if narrow { 51.0 } else { 59.0 },
    );
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if enabled && response.hovered() {
        fill(ui, rect, 2, p.inset, Stroke::new(1.0, p.border_strong));
    }
    let alpha = if enabled { 1.0 } else { 0.5 };
    icons::paint_svg(
        ui,
        Rect::from_center_size(
            pos2(rect.center().x, rect.top() + 19.5),
            vec2(
                if narrow { 20.0 } else { 25.0 },
                if narrow { 20.0 } else { 25.0 },
            ),
        ),
        source,
        color.gamma_multiply(alpha),
    );
    ui.painter().galley(
        pos2(rect.center().x - g.size().x / 2.0, rect.top() + 38.0),
        g,
        p.text.gamma_multiply(alpha),
    );
    response
}

// ---------------------------------------------------------------------------
// Panels

/// `.sci-panel-header` (29 px, 13 px semibold) or `.home-panel-header` (25 px, 12 px).
pub fn panel_header(
    ui: &mut Ui,
    p: &Palette,
    title: &str,
    studio: bool,
    right: impl FnOnce(&mut Ui),
) {
    let height = if studio { 25.0 } else { 29.0 };
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    fill(ui, rect, 0, p.header, Stroke::NONE);
    hline(
        ui,
        rect.x_range(),
        rect.bottom() - 1.0,
        if studio { p.border } else { p.border_strong },
    );
    let font = sans(if studio { 12.0 } else { 13.0 }, Weight::SemiBold);
    let pad = if studio { 7.0 } else { 10.4 };
    let right_rect = Rect::from_min_max(
        pos2(rect.left() + pad, rect.top()),
        pos2(rect.right() - pad, rect.bottom() - 1.0),
    );
    // A child does not reserve space in the parent, so the next row starts below the header.
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(right_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    right(&mut child);
    let available = (child.min_rect().left() - rect.left() - 2.0 * pad).max(0.0);
    let g = ellipsized(ui, title, font, p.text, 0.0, available);
    ui.painter().galley(
        pos2(rect.left() + pad, rect.center().y - g.size().y / 2.0),
        g,
        p.text,
    );
}

/// Small text button used as `Limpar` in Studio headers.
pub fn link_button(ui: &mut Ui, p: &Palette, label: &str) -> Response {
    let g = galley(ui, label, sans(12.0, Weight::Medium), p.muted, 0.0);
    let (rect, response) = ui.allocate_exact_size(g.size() + vec2(12.6, 4.2), Sense::click());
    if response.hovered() {
        fill(ui, rect, 3, p.inset, Stroke::NONE);
    }
    let color = if response.hovered() { p.text } else { p.muted };
    ui.painter().galley(rect.min + vec2(6.3, 2.1), g, color);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// `.studio-folder-bar button`: bordered 24 px button.
pub fn bar_button(ui: &mut Ui, p: &Palette, label: &str) -> Response {
    let g = galley(ui, label, sans(11.0, Weight::Regular), p.text, 0.0);
    let (rect, response) = ui.allocate_exact_size(vec2(g.size().x + 16.0, 24.0), Sense::click());
    fill(
        ui,
        rect,
        2,
        if response.hovered() {
            p.inset
        } else {
            p.header
        },
        Stroke::new(
            1.0,
            if response.hovered() {
                p.border_strong
            } else {
                p.border
            },
        ),
    );
    ui.painter().galley(
        pos2(rect.left() + 8.0, rect.center().y - g.size().y / 2.0),
        g,
        p.text,
    );
    response
}

pub fn icon_button(
    ui: &mut Ui,
    p: &Palette,
    source: &str,
    size: Vec2,
    icon: f32,
    enabled: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if enabled && response.hovered() {
        fill(ui, rect, 2, p.header, Stroke::new(1.0, p.border));
    }
    let tint = if enabled {
        Color32::WHITE
    } else {
        Color32::WHITE.gamma_multiply(0.35)
    };
    icons::paint_svg(
        ui,
        Rect::from_center_size(rect.center(), vec2(icon, icon)),
        source,
        tint,
    );
    response
}

/// `.studio-welcome-primary` / `-secondary`.
pub fn welcome_button(
    ui: &mut Ui,
    p: &Palette,
    icon: Icon,
    label: &str,
    primary: bool,
) -> Response {
    let font = sans(13.0, Weight::Medium);
    let g = galley(ui, label, font, p.text, 0.0);
    let (rect, response) = ui.allocate_exact_size(
        vec2(
            ui.data(|d| d.get_temp::<f32>(egui::Id::new("welcome-button-width")))
                .unwrap_or(g.size().x + 51.0),
            42.0,
        ),
        Sense::click(),
    );
    let hovered = response.hovered();
    let (bg, border, fg) = if primary {
        let c = if hovered { p.accent_hover } else { p.accent };
        (c, c, p.accent_contrast)
    } else {
        (
            p.header,
            if hovered { p.accent } else { p.border_strong },
            p.text,
        )
    };
    fill(ui, rect, 4, bg, Stroke::new(1.0, border));
    icons::paint(
        ui,
        Rect::from_min_size(
            pos2(
                rect.center().x - (g.size().x + 23.0) / 2.0,
                rect.center().y - 8.0,
            ),
            vec2(16.0, 16.0),
        ),
        icon,
        fg,
    );
    ui.painter().galley_with_override_text_color(
        pos2(
            rect.center().x - (g.size().x + 23.0) / 2.0 + 23.0,
            rect.center().y - g.size().y / 2.0,
        ),
        g,
        fg,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// `.sci-btn-mini`: compact teal button.
pub fn mini_button(ui: &mut Ui, p: &Palette, label: &str) -> Response {
    let g = galley(ui, label, sans(13.0, Weight::SemiBold), Color32::WHITE, 0.0);
    let (rect, response) = ui.allocate_exact_size(vec2(g.size().x + 24.0, 30.0), Sense::click());
    fill(
        ui,
        rect,
        4,
        if response.hovered() {
            p.primary_hover
        } else {
            p.primary
        },
        Stroke::new(1.0, p.accent_border),
    );
    ui.painter().galley(
        pos2(rect.left() + 12.0, rect.center().y - g.size().y / 2.0),
        g,
        Color32::WHITE,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

// ---------------------------------------------------------------------------
// Forms

/// `.form-label-row`: 14.6 px semibold label with the round `?` tooltip.
pub fn form_label(ui: &mut Ui, p: &Palette, label: &str, help: &str) {
    let width = ui.available_width();
    let help_width = if help.is_empty() { 0.0 } else { 20.0 };
    let g = wrapped(
        ui,
        label,
        sans(14.6, Weight::SemiBold),
        p.text,
        (width - help_width).max(40.0),
    );
    let (row, _) =
        ui.allocate_exact_size(vec2(width, (g.size().y + 6.0).max(28.0)), Sense::hover());
    let gx = g.size().x;
    ui.painter().galley(row.min, g, p.text);
    if !help.is_empty() {
        let rect = Rect::from_min_size(
            pos2(
                (row.left() + gx + 4.0).min(row.right() - 14.0),
                row.top() + 3.0,
            ),
            vec2(14.0, 14.0),
        );
        let response = ui.interact(rect, ui.id().with(("help", label)), Sense::click());
        ui.painter().circle(
            rect.center(),
            6.5,
            p.header,
            Stroke::new(1.0, p.border_strong),
        );
        let g = galley(ui, "?", sans(10.0, Weight::Regular), p.muted, 0.0);
        ui.painter()
            .galley(rect.center() - g.size() / 2.0, g, p.muted);
        response.on_hover_text(help);
    }
}

/// `.form-input`: inset background, 3 px radius, JetBrains Mono; height follows the layout.
pub fn input(
    ui: &mut Ui,
    p: &Palette,
    value: &mut String,
    hint: &str,
    monospace: bool,
) -> Response {
    input_id(ui, p, value, hint, monospace, None)
}

/// Height and font size of `.form-input` in the parameters panel for the current layout.
pub fn field_metrics(ui: &Ui) -> (f32, f32) {
    let rem = crate::MEASURED_REM;
    match layout(ui).mode {
        crate::Mode::Phone => (2.75 * rem, rem),
        crate::Mode::Tablet => (2.5 * rem, 0.8438 * rem),
        crate::Mode::Desktop => (33.0, 13.6),
    }
}

pub fn input_id(
    ui: &mut Ui,
    p: &Palette,
    value: &mut String,
    hint: &str,
    monospace: bool,
    id: Option<egui::Id>,
) -> Response {
    let (height, size) = field_metrics(ui);
    let font = if monospace {
        mono(size, Weight::Regular)
    } else {
        sans(size, Weight::Regular)
    };
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    fill(ui, rect, 3, p.inset, Stroke::NONE);
    let mut edit = egui::TextEdit::singleline(value)
        .font(font.clone())
        .text_color(p.text)
        .hint_text(egui::RichText::new(hint).color(p.faint).font(font))
        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(
            if height > 34.0 { 14 } else { 10 },
            0,
        )))
        .vertical_align(egui::Align::Center)
        .desired_width(width);
    if let Some(id) = id {
        edit = edit.id(id);
    }
    let response = ui.put(rect, edit);
    if response.has_focus() && (content_resized(ui, response.id) || response.gained_focus()) {
        response.scroll_to_me(Some(egui::Align::Center));
    }
    let border = if response.has_focus() {
        p.accent
    } else {
        p.border
    };
    ui.painter().rect(
        rect,
        3,
        Color32::TRANSPARENT,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    response
}

/// Whether the content area changed height since the last frame `id` had focus: the soft
/// keyboard opened or closed, or the window rotated. Call it every frame the field has focus.
pub fn content_resized(ui: &Ui, id: egui::Id) -> bool {
    let height = ui.ctx().content_rect().height();
    let key = id.with("viewport-height");
    let previous = ui.data_mut(|d| d.get_temp::<f32>(key));
    ui.data_mut(|d| d.insert_temp(key, height));
    previous.is_some_and(|h| (height - h).abs() > 1.0)
}

pub fn textarea(ui: &mut Ui, p: &Palette, value: &mut String, rows: usize) -> Response {
    let height = 12.0 + rows as f32 * 19.0;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    fill(ui, rect, 3, p.inset, Stroke::NONE);
    let edit = egui::TextEdit::multiline(value)
        .font(mono(13.6, Weight::Regular))
        .text_color(p.text)
        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(10, 6)))
        .desired_rows(rows)
        .desired_width(rect.width());
    let response = ui.put(rect, edit);
    if response.has_focus() && content_resized(ui, response.id) {
        response.scroll_to_me(Some(egui::Align::Center));
    }
    let border = if response.has_focus() {
        p.accent
    } else {
        p.border
    };
    ui.painter().rect(
        rect,
        3,
        Color32::TRANSPARENT,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    response
}

/// `.tabs-segmented`: pill buttons inside a tinted track; returns true when changed.
pub fn segmented(ui: &mut Ui, p: &Palette, options: &[[String; 2]], value: &mut String) -> bool {
    let width = ui.available_width();
    let fonts: Vec<_> = options
        .iter()
        .map(|o| {
            let active = *value == o[0];
            let font = sans(
                13.0,
                if active {
                    Weight::SemiBold
                } else {
                    Weight::Medium
                },
            );
            let lines: Vec<String> = o[1].split(' ').map(str::to_owned).collect();
            (active, font, lines)
        })
        .collect();
    // Labels wrap at spaces when three options share the track, as `Degrau\n1/s` did.
    let natural: f32 = fonts
        .iter()
        .map(|(_, f, l)| galley(ui, &l.join(" "), f.clone(), p.text, 0.0).size().x + 28.0)
        .sum();
    // Browser text runs a little wider than egui's; the tolerance matches its wrapping point.
    let wrap = natural + 20.0 > width;
    let height = if wrap { 51.0 + 8.0 } else { 40.0 };
    let (track, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    fill(
        ui,
        track,
        3,
        Color32::from_black_alpha(10),
        Stroke::new(1.0, p.border),
    );
    let inner = track.shrink(4.0);
    let mut x = inner.left();
    let mut changed = false;
    let slot = if wrap {
        (inner.width() - 2.0 * (options.len() as f32 - 1.0)) / options.len() as f32
    } else {
        0.0
    };
    for (i, (active, font, lines)) in fonts.iter().enumerate() {
        let text = if wrap {
            lines.join("\n")
        } else {
            lines.join(" ")
        };
        let g = ui.fonts_mut(|f| {
            let mut job = job(&text, font.clone(), p.text, 0.0);
            job.halign = egui::Align::Center;
            f.layout_job(job)
        });
        let w = if wrap { slot } else { g.size().x + 28.0 };
        let rect = Rect::from_min_size(pos2(x, inner.top()), vec2(w, inner.height()));
        let response = ui.interact(rect, ui.id().with(("seg", i)), Sense::click());
        if *active {
            ui.painter().rect_filled(
                rect.translate(vec2(0.0, 1.0)),
                7,
                Color32::from_black_alpha(if p.light { 20 } else { 50 }),
            );
            fill(
                ui,
                rect,
                7,
                if p.light { p.surface } else { p.inset },
                Stroke::NONE,
            );
        }
        let color = if *active || response.hovered() {
            p.text
        } else {
            p.muted
        };
        let pos = pos2(rect.center().x, rect.center().y - g.size().y / 2.0);
        ui.painter().galley_with_override_text_color(pos, g, color);
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && !*active
        {
            *value = options[i][0].clone();
            changed = true;
        }
        x += w + 2.0;
    }
    changed
}

/// `.sci-segmented-mini` in figure headers (Degrau / Rampa / Impulso).
pub fn mini_segmented(
    ui: &mut Ui,
    p: &Palette,
    options: &[(&str, &str)],
    value: &str,
) -> Option<String> {
    let font = sans(12.0, Weight::SemiBold);
    let widths: Vec<f32> = options
        .iter()
        .map(|(_, l)| galley(ui, l, font.clone(), p.text, 0.0).size().x + 16.0)
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 2.0;
    let (rect, _) = ui.allocate_exact_size(vec2(total, 20.0), Sense::hover());
    fill(ui, rect, 3, p.surface, Stroke::new(1.0, p.accent_border));
    let mut x = rect.left() + 1.0;
    let mut clicked = None;
    for (i, (id, label)) in options.iter().enumerate() {
        let seg = Rect::from_min_size(pos2(x, rect.top() + 1.0), vec2(widths[i], 18.0));
        let active = *id == value;
        let response = ui.interact(seg, ui.id().with(("mini", i)), Sense::click());
        if active {
            fill(ui, seg, 0, p.accent, Stroke::NONE);
        }
        if i + 1 < options.len() {
            vline(ui, seg.right() - 1.0, seg.y_range(), p.accent_border);
        }
        let weight = if active {
            Weight::SemiBold
        } else {
            Weight::Medium
        };
        let color = if active { p.accent_contrast } else { p.muted };
        let g = galley(ui, label, sans(12.0, weight), color, 0.0);
        ui.painter().galley(seg.center() - g.size() / 2.0, g, color);
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            clicked = Some((*id).to_owned());
        }
        x += widths[i];
    }
    clicked
}

/// Select styled as `.form-input` with a chevron; options open in a popup list.
pub fn select(
    ui: &mut Ui,
    p: &Palette,
    id: egui::Id,
    options: &[[String; 2]],
    value: &mut String,
) -> bool {
    let (height, size) = field_metrics(ui);
    select_with(
        ui,
        p,
        id,
        options,
        value,
        mono(size, Weight::Regular),
        height,
    )
}

pub fn select_with(
    ui: &mut Ui,
    p: &Palette,
    id: egui::Id,
    options: &[[String; 2]],
    value: &mut String,
    font: FontId,
    height: f32,
) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    let open = egui::Popup::is_id_open(ui.ctx(), id);
    fill(
        ui,
        rect,
        3,
        p.inset,
        Stroke::new(
            1.0,
            if open || response.hovered() {
                p.accent
            } else {
                p.border
            },
        ),
    );
    let label = options
        .iter()
        .find(|o| o[0] == *value)
        .map_or(value.clone(), |o| o[1].clone());
    let label = label.as_str();
    let painter = ui.painter().with_clip_rect(
        rect.shrink2(vec2(10.0, 0.0))
            .with_max_x(rect.right() - 30.0),
    );
    let g = galley(ui, label, font, p.text, 0.0);
    painter.galley(
        pos2(rect.left() + 10.4, rect.center().y - g.size().y / 2.0),
        g,
        p.text,
    );
    icons::paint(
        ui,
        Rect::from_center_size(pos2(rect.right() - 22.0, rect.center().y), vec2(14.0, 14.0)),
        icons::CHEVRON,
        p.muted,
    );
    if response.clicked() {
        if open {
            egui::Popup::close_id(ui.ctx(), id);
        } else {
            egui::Popup::open_id(ui.ctx(), id);
        }
    }
    let mut changed = false;
    egui::Popup::new(id, ui.ctx().clone(), rect, ui.layer_id())
        .open_memory(None)
        .width(rect.width())
        .frame(frame(p.surface, p.border_strong, 4, 4))
        .show(|ui| {
            ui.set_min_width(rect.width() - 10.0);
            for o in options {
                let selected = o[0] == *value;
                let g = wrapped(
                    ui,
                    &o[1],
                    sans(13.6, Weight::Regular),
                    p.text,
                    rect.width() - 30.0,
                );
                let (row, r) = ui.allocate_exact_size(
                    vec2(rect.width() - 10.0, g.size().y + 12.0),
                    Sense::click(),
                );
                if selected || r.hovered() {
                    fill(
                        ui,
                        row,
                        3,
                        if selected {
                            p.accent.gamma_multiply(0.15)
                        } else {
                            p.header
                        },
                        Stroke::NONE,
                    );
                }
                ui.painter()
                    .galley(pos2(row.left() + 8.0, row.top() + 6.0), g, p.text);
                if r.clicked() {
                    *value = o[0].clone();
                    changed = true;
                    egui::Popup::close_id(ui.ctx(), id);
                }
            }
        });
    response.on_hover_text(label);
    changed
}

// ---------------------------------------------------------------------------
// Results

/// `.results-table` rows: striped, muted label left, monospace semibold value right.
pub fn striped_row(ui: &mut Ui, p: &Palette, index: usize, label: &str, value: &str) {
    let width = ui.available_width();
    let label_g = wrapped(
        ui,
        label,
        sans(13.0, Weight::Regular),
        p.muted,
        width * 0.55,
    );
    let value_g = wrapped(
        ui,
        value,
        mono(13.6, Weight::SemiBold),
        p.text,
        width - label_g.size().x - 30.0,
    );
    let height = label_g.size().y.max(value_g.size().y) + 12.6;
    let (rect, response) = ui.allocate_exact_size(vec2(width, height.max(31.0)), Sense::hover());
    fill(
        ui,
        rect,
        0,
        if index.is_multiple_of(2) {
            p.header
        } else {
            p.surface
        },
        Stroke::NONE,
    );
    hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border);
    ui.painter().galley(
        pos2(rect.left() + 10.4, rect.center().y - label_g.size().y / 2.0),
        label_g,
        p.muted,
    );
    let vx = rect.right() - 10.4 - value_g.size().x;
    ui.painter().galley(
        pos2(vx, rect.center().y - value_g.size().y / 2.0),
        value_g,
        p.text,
    );
    response.on_hover_text(value);
}

/// `.results-group-header`: uppercase accent caption between bordered rules.
pub fn group_header(ui: &mut Ui, p: &Palette, label: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 27.0), Sense::hover());
    hline(ui, rect.x_range(), rect.top(), p.border);
    hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border);
    let g = galley(
        ui,
        &label.to_uppercase(),
        sans(11.7, Weight::Bold),
        p.accent,
        0.6,
    );
    ui.painter().galley(
        pos2(rect.left() + 6.0, rect.center().y - g.size().y / 2.0),
        g,
        p.accent,
    );
}

/// `.empty-discrete-box` / `.routh-empty-card`, centered in the remaining space.
pub fn empty_box(ui: &mut Ui, p: &Palette, parts: &[(&str, bool)], icon: bool, max_width: f32) {
    let area = ui.available_rect_before_wrap();
    let width = max_width.min(area.width() - 20.0);
    let mut job = egui::text::LayoutJob::default();
    for (text, bold) in parts {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: sans(13.6, if *bold { Weight::Bold } else { Weight::Regular }),
                color: p.muted,
                ..Default::default()
            },
        );
    }
    job.wrap.max_width = width - 37.6 - if icon { 34.4 } else { 0.0 };
    if !icon {
        job.halign = egui::Align::Center;
    }
    let g = ui.fonts_mut(|f| f.layout_job(job));
    let content_w = if icon {
        g.size().x + 34.4
    } else {
        width - 37.6
    };
    let size = vec2(content_w + 37.6, g.size().y.max(24.0) + 25.0);
    let rect = Rect::from_center_size(area.center(), size);
    fill(ui, rect, 4, p.header, Stroke::new(1.0, p.border));
    if icon {
        icons::paint(
            ui,
            Rect::from_min_size(
                pos2(rect.left() + 18.8, rect.center().y - 12.0),
                vec2(24.0, 24.0),
            ),
            icons::INFO,
            p.faint,
        );
        ui.painter().galley(
            pos2(
                rect.left() + 18.8 + 34.4,
                rect.center().y - g.size().y / 2.0,
            ),
            g,
            p.muted,
        );
    } else {
        ui.painter().galley(
            pos2(rect.center().x, rect.center().y - g.size().y / 2.0),
            g,
            p.muted,
        );
    }
}

/// Status bar (`.sci-status-bar`): 27 px, left state and right module/solver text.
pub fn status_bar(ui: &mut Ui, p: &Palette, left: &str, right: &str) {
    let rect = ui.max_rect();
    fill(ui, rect, 0, p.surface, Stroke::NONE);
    hline(ui, rect.x_range(), rect.top(), p.border_strong);
    let font = sans(12.5, Weight::Regular);
    let g = galley(ui, left, font.clone(), p.muted, 0.0);
    ui.painter().galley(
        pos2(rect.left() + 12.5, rect.center().y - g.size().y / 2.0),
        g,
        p.muted,
    );
    let g = galley(ui, right, font, p.muted, 0.0);
    ui.painter().galley(
        pos2(
            rect.right() - 12.5 - g.size().x,
            rect.center().y - g.size().y / 2.0,
        ),
        g,
        p.muted,
    );
}

// ---------------------------------------------------------------------------
// Responsive shell (`styles.css` breakpoints)

pub fn set_layout(ctx: &egui::Context, layout: crate::Layout) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("controlab-layout"), layout));
}

pub fn layout(ui: &Ui) -> crate::Layout {
    ui.data(|d| d.get_temp(egui::Id::new("controlab-layout")))
        .unwrap_or_default()
}

/// Module entries of the rail and of the bottom navigation, with the home symbol.
pub const NAV: [(&str, Icon, &str, &str); 7] = [
    ("home", icons::S_HOME, "Início", "Início"),
    ("time", icons::S_TIME, "Tempo", "Tempo"),
    ("routh", icons::S_STAB, "Estab.", "Estabil."),
    ("lgr", icons::S_ROOT, "LGR", "LGR"),
    ("frequency", icons::S_FREQ, "Freq.", "Freq."),
    ("controllers", icons::S_CTRL, "Control.", "Control."),
    ("state-space", icons::S_STATE, "Estados", "Estados"),
];

/// `.sci-bottom-nav` background and top border.
pub fn bottom_nav_colors(p: &Palette) -> (Color32, Color32) {
    if p.light {
        (Color32::WHITE, Color32::from_rgb(226, 232, 240))
    } else {
        (Color32::from_rgb(30, 36, 44), Color32::from_rgb(45, 55, 72))
    }
}

/// `.sci-bottom-nav`: scrollable icon bar of the phone layout; keeps the active item visible.
pub fn bottom_nav(ui: &mut Ui, p: &Palette, page: &str, k: f32) -> Option<&'static str> {
    let rect = ui.max_rect();
    let (bg, border) = bottom_nav_colors(p);
    fill(ui, rect, 0, bg, Stroke::NONE);
    hline(ui, rect.x_range(), rect.top(), border);
    let rem = crate::MEASURED_REM * k;
    let last = ui.data(|d| d.get_temp::<String>(egui::Id::new("bottom-nav-page")));
    let changed = last.as_deref() != Some(page);
    ui.data_mut(|d| d.insert_temp(egui::Id::new("bottom-nav-page"), page.to_owned()));
    let mut chosen = None;
    let mut child =
        ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(vec2(0.5 * rem, 0.0))));
    egui::ScrollArea::horizontal()
        .id_salt("bottom-nav")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut child, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (id, icon, _, label) in NAV {
                    let active = page == id;
                    let font = sans(
                        0.72 * rem,
                        if active { Weight::Bold } else { Weight::Medium },
                    );
                    let g = galley(ui, label, font, p.muted, 0.0);
                    let width = (g.size().x + rem).max(4.875 * rem);
                    let (item, response) =
                        ui.allocate_exact_size(vec2(width, rect.height()), Sense::click());
                    let color = if active {
                        p.accent
                    } else if response.hovered() {
                        p.text
                    } else {
                        p.muted
                    };
                    if active {
                        fill(
                            ui,
                            Rect::from_min_max(
                                pos2(item.left() + 0.875 * rem, item.top()),
                                pos2(item.right() - 0.875 * rem, item.top() + 3.0),
                            ),
                            CornerRadius {
                                nw: 0,
                                ne: 0,
                                sw: 2,
                                se: 2,
                            },
                            p.accent,
                            Stroke::NONE,
                        );
                        if changed {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }
                    }
                    let size = 1.375 * rem;
                    let gap = 0.1875 * rem;
                    let top = item.center().y - (size + gap + g.size().y) / 2.0;
                    icons::paint(
                        ui,
                        Rect::from_min_size(
                            pos2(item.center().x - size / 2.0, top),
                            vec2(size, size),
                        ),
                        icon,
                        color,
                    );
                    ui.painter().galley_with_override_text_color(
                        pos2(item.center().x - g.size().x / 2.0, top + size + gap),
                        g,
                        color,
                    );
                    if response.clicked() && !active {
                        chosen = Some(id);
                    }
                }
            });
        });
    chosen
}

/// `.sci-rail`: vertical module rail of the tablet layout.
pub fn rail(ui: &mut Ui, p: &Palette, page: &str, k: f32) -> Option<&'static str> {
    let rect = ui.max_rect();
    fill(ui, rect, 0, p.surface, Stroke::NONE);
    vline(ui, rect.right() - 1.0, rect.y_range(), p.border_strong);
    let rem = crate::MEASURED_REM * k;
    let item_h = 3.625 * rem;
    let mut chosen = None;
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    egui::ScrollArea::vertical()
        .id_salt("rail-scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (id, icon, label, _) in NAV.iter() {
                let (item, response) =
                    ui.allocate_exact_size(vec2(rect.width() - 1.0, item_h), Sense::click());
                let active = page == *id;
                if active {
                    fill(ui, item, 0, p.bg, Stroke::NONE);
                    fill(
                        ui,
                        Rect::from_min_size(item.min, vec2(2.0, item_h)),
                        0,
                        p.accent,
                        Stroke::NONE,
                    );
                } else if response.hovered() {
                    fill(ui, item, 0, p.header, Stroke::NONE);
                }
                let color = if active {
                    p.accent
                } else if response.hovered() {
                    p.text
                } else {
                    p.muted
                };
                let g = galley(
                    ui,
                    label,
                    sans(
                        0.75 * rem,
                        if active {
                            Weight::SemiBold
                        } else {
                            Weight::Medium
                        },
                    ),
                    color,
                    0.0,
                );
                let size = 1.375 * rem;
                let top = item.center().y - (size + 0.125 * rem + g.size().y) / 2.0;
                icons::paint(
                    ui,
                    Rect::from_min_size(pos2(item.center().x - size / 2.0, top), vec2(size, size)),
                    *icon,
                    color,
                );
                ui.painter().galley(
                    pos2(item.center().x - g.size().x / 2.0, top + size + 0.125 * rem),
                    g,
                    color,
                );
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    && !active
                {
                    chosen = Some(*id);
                }
            }
        });
    chosen
}

/// `.sci-mobile-tabs` (phone) and `.sci-tablet-switcher` (tablet): equal tabs with an
/// accent underline. Returns the clicked index.
pub fn segment_tabs(
    ui: &mut Ui,
    p: &Palette,
    labels: &[&str],
    active: usize,
    font_rem: f32,
    background: Color32,
) -> Option<usize> {
    let rect = ui.max_rect();
    fill(ui, rect, 0, background, Stroke::NONE);
    hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border_strong);
    let w = rect.width() / labels.len() as f32;
    let mut chosen = None;
    for (i, label) in labels.iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(rect.left() + i as f32 * w, rect.top()),
            vec2(w, rect.height()),
        );
        let response = ui.interact(r, ui.id().with(("segment-tab", i)), Sense::click());
        let on = i == active;
        let color = if on {
            p.accent
        } else if response.hovered() {
            p.text
        } else {
            p.muted
        };
        let g = galley(
            ui,
            label,
            sans(
                font_rem * crate::MEASURED_REM,
                if on { Weight::SemiBold } else { Weight::Medium },
            ),
            color,
            0.0,
        );
        ui.painter().galley(r.center() - g.size() / 2.0, g, color);
        if on {
            fill(
                ui,
                Rect::from_min_max(pos2(r.left(), r.bottom() - 2.0), r.max),
                0,
                p.accent,
                Stroke::NONE,
            );
        }
        if response.clicked() {
            chosen = Some(i);
        }
    }
    chosen
}

/// `.sci-btn-mobile-calc`: full-width Calcular button of the phone footer.
pub fn mobile_calc(ui: &mut Ui, p: &Palette, icon: bool, enabled: bool) -> Response {
    let rem = crate::MEASURED_REM;
    let rect = ui.max_rect();
    fill(ui, rect, 0, p.surface, Stroke::NONE);
    hline(ui, rect.x_range(), rect.top(), p.border_strong);
    let button = rect.shrink2(vec2(0.75 * rem, 0.5 * rem));
    let response = ui.interact(
        button,
        ui.id().with("mobile-calc"),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let bg = if response.hovered() {
        Color32::from_rgb(18, 127, 155)
    } else {
        Color32::from_rgb(15, 110, 134)
    };
    fill(
        ui,
        button,
        4,
        bg.gamma_multiply(if enabled { 1.0 } else { 0.7 }),
        Stroke::new(1.0, Color32::from_rgb(22, 122, 148)),
    );
    let g = galley(
        ui,
        "Calcular",
        sans(0.9375 * rem, Weight::SemiBold),
        Color32::WHITE,
        0.0,
    );
    let icon_w = if icon { 18.0 + 0.375 * rem } else { 0.0 };
    let x = button.center().x - (g.size().x + icon_w) / 2.0;
    if icon {
        icons::paint(
            ui,
            Rect::from_min_size(pos2(x, button.center().y - 9.0), vec2(18.0, 18.0)),
            icons::CALC,
            Color32::WHITE,
        );
    }
    ui.painter().galley(
        pos2(x + icon_w, button.center().y - g.size().y / 2.0),
        g,
        Color32::WHITE,
    );
    response
}

/// `.sci-virtual-math`: symbol keys inserted into the expression on phones.
pub fn virtual_keys(ui: &mut Ui, p: &Palette, keys: &[&'static str]) -> Option<&'static str> {
    let rem = crate::MEASURED_REM;
    ui.add_space(0.25 * rem);
    let width = ui.available_width();
    let gap = 0.25 * rem;
    let w = (width - gap * (keys.len() - 1) as f32) / keys.len() as f32;
    let (row, _) = ui.allocate_exact_size(vec2(width, 2.8 * rem), Sense::hover());
    let mut chosen = None;
    for (i, key) in keys.iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(row.left() + i as f32 * (w + gap), row.top()),
            vec2(w, row.height()),
        );
        let response = ui.interact(r, ui.id().with(("vkey", i)), Sense::click());
        let pressed = response.is_pointer_button_down_on();
        fill(
            ui,
            r,
            4,
            if pressed { p.accent } else { p.header },
            Stroke::new(1.0, p.border_strong),
        );
        let label = if *key == "-" { "−" } else { key };
        let g = galley(
            ui,
            label,
            mono(0.9375 * rem, Weight::Regular),
            if pressed { p.accent_contrast } else { p.text },
            0.0,
        );
        ui.painter().galley(
            r.center() - g.size() / 2.0,
            g,
            if pressed { p.accent_contrast } else { p.text },
        );
        if response.clicked() {
            chosen = Some(*key);
        }
    }
    ui.add_space(0.375 * rem);
    chosen
}

/// Inserts `text` at the caret of the single-line editor `id`, as the virtual keys did.
pub fn insert_at_cursor(ctx: &egui::Context, id: egui::Id, value: &mut String, text: &str) {
    let mut state = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    let count = value.chars().count();
    let (start, end) = state
        .cursor
        .char_range()
        .map(|r| {
            let range = r.as_sorted_char_range();
            (range.start.0.min(count), range.end.0.min(count))
        })
        .unwrap_or((count, count));
    let byte = |index: usize| {
        value
            .char_indices()
            .nth(index)
            .map_or(value.len(), |(b, _)| b)
    };
    let (from, to) = (byte(start), byte(end));
    value.replace_range(from..to, text);
    let caret = egui::text::CCursor::new(start + text.chars().count());
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(caret)));
    state.store(ctx, id);
    ctx.memory_mut(|m| m.request_focus(id));
}
