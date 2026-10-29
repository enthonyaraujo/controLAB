//! Result views of the previous renderer: striped tables, LaTeX cards, step panels, the
//! Routh cards and the LGR memorial. TeX is typeset on demand by the engine's native
//! composer with the KaTeX fonts, in white, and tinted here.
use crate::kit::{self, fill, galley, hline};
use crate::model::Module;
use crate::texfmt::{display_number, fixed, list, raw, tex_to_text, text};
use crate::theme::{Palette, Tone, Weight, mono, sans};
use eframe::egui::{self, Color32, Rect, Response, Sense, Stroke, Ui, pos2, vec2};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap, HashSet};

/// Matches the engine's formula batch limit (formulas::render takes at most 200).
const MAX_BATCH: usize = 200;
/// Em size, in pixels, of the typeset images. MathText draws KaTeX glyphs about 13% smaller
/// than the browser at the same nominal size (measured on identical formulas), so the
/// 48 px images (native_presentation.FORMULA_EM_PX) are treated as a 42.5 px em.
const SOURCE_EM: f32 = 42.5;
/// KaTeX renders at 1.21 em of the surrounding text.
pub const KATEX_SCALE: f32 = 1.21;

/// TeX sources displayed by the views, typeset in batches.
pub struct Formulas {
    ctx: egui::Context,
    images: HashMap<String, Option<egui::TextureHandle>>,
    wanted: BTreeSet<String>,
    requested: HashSet<String>,
    in_flight: Option<Vec<String>>,
}

impl Formulas {
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            ctx,
            images: HashMap::new(),
            wanted: BTreeSet::new(),
            requested: HashSet::new(),
            in_flight: None,
        }
    }
    #[cfg(test)]
    pub fn typeset(&self) -> usize {
        self.images.values().filter(|image| image.is_some()).count()
    }
    pub fn busy(&self) -> bool {
        self.in_flight.is_some() || !self.wanted.is_empty()
    }
    /// The next batch of displayed-but-missing sources, one request at a time.
    pub fn next_request(&mut self) -> Option<Value> {
        if self.in_flight.is_some() || self.wanted.is_empty() {
            return None;
        }
        let batch: Vec<String> = self.wanted.iter().take(MAX_BATCH).cloned().collect();
        for source in &batch {
            self.wanted.remove(source);
        }
        self.requested = batch.iter().cloned().collect();
        let payload = json!({"action":"render_formulas","sources":batch});
        self.in_flight = Some(batch);
        Some(payload)
    }
    /// Stores a reply. Transport failures and unsupported TeX keep the text fallback.
    pub fn receive(&mut self, reply: &Value) {
        let Some(batch) = self.in_flight.take() else {
            return;
        };
        self.requested.clear();
        let images = reply["images"].as_array();
        for (i, source) in batch.into_iter().enumerate() {
            let image = images
                .and_then(|a| a.get(i))
                .and_then(Value::as_str)
                .and_then(|png| texture(&self.ctx, &source, png));
            self.images.insert(source, image);
        }
    }
    fn lookup(&mut self, key: &str) -> Option<egui::TextureHandle> {
        match self.images.get(key) {
            Some(found) => found.clone(),
            None => {
                if !self.requested.contains(key) {
                    self.wanted.insert(key.to_owned());
                }
                None
            }
        }
    }
    /// Size of `key` (display source, or `\textstyle`-prefixed inline source) at `em` points.
    pub fn size(&mut self, key: &str, em: f32) -> Option<egui::Vec2> {
        let t = self.lookup(key.trim())?;
        Some(vec2(t.size()[0] as f32, t.size()[1] as f32) * (em / SOURCE_EM))
    }
    /// Display formula centered in the available width (`.katex-display`).
    pub fn math_centered(&mut self, ui: &mut Ui, tex: &str, em: f32, color: Color32) -> Response {
        let tex = tex.trim();
        let width = ui.available_width();
        match self.size(tex, em) {
            Some(size) if size.x <= width => {
                let (rect, response) = ui.allocate_exact_size(vec2(width, size.y), Sense::hover());
                if let Some(t) = self.lookup(tex) {
                    let inner = Rect::from_center_size(rect.center(), size);
                    ui.painter().image(
                        t.id(),
                        inner,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        color,
                    );
                }
                response
            }
            // `overflow-x: auto`: wide formulas start at the left and scroll.
            Some(_) => {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt(("math-scroll", tex))
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show(ui, |ui| self.draw(ui, tex, em, color, false))
                        .inner
                })
                .inner
            }
            None => {
                ui.vertical_centered(|ui| self.draw(ui, tex, em, color, false))
                    .inner
            }
        }
    }
    /// Inline formula inside running text (KaTeX inline mode).
    pub fn inline(&mut self, ui: &mut Ui, tex: &str, em: f32, color: Color32) -> Response {
        self.draw(ui, &format!("\\textstyle {}", tex.trim()), em, color, true)
    }
    fn draw(&mut self, ui: &mut Ui, key: &str, em: f32, color: Color32, inline: bool) -> Response {
        if key.is_empty() || key == "\\textstyle " {
            return ui.allocate_response(egui::Vec2::ZERO, Sense::hover());
        }
        match self.lookup(key) {
            Some(t) => {
                let size = vec2(t.size()[0] as f32, t.size()[1] as f32) * (em / SOURCE_EM);
                let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
                ui.painter().image(
                    t.id(),
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    color,
                );
                response
            }
            None => {
                // Until typeset, wrap the readable text so it never widens a panel.
                let source = key.strip_prefix("\\textstyle ").unwrap_or(key);
                let font = sans(
                    em / KATEX_SCALE * if inline { 1.0 } else { 1.05 },
                    Weight::Regular,
                );
                let g = kit::wrapped(
                    ui,
                    &tex_to_text(source),
                    font,
                    color,
                    ui.available_width().max(40.0),
                );
                let (rect, response) = ui.allocate_exact_size(g.size(), Sense::hover());
                ui.painter().galley(rect.min, g, color);
                response
            }
        }
    }
}

/// Decodes a base64 PNG, with or without a data URI prefix.
pub fn texture(ctx: &egui::Context, name: &str, png: &str) -> Option<egui::TextureHandle> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(png.strip_prefix("data:image/png;base64,").unwrap_or(png))
        .ok()?;
    let rgba = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    // Formulas are drawn ~3× smaller than rasterized; mipmaps keep 1 px strokes (minus signs).
    let options = egui::TextureOptions {
        mipmap_mode: Some(egui::TextureFilter::Linear),
        ..egui::TextureOptions::LINEAR
    };
    Some(ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()),
        options,
    ))
}

/// Inline rich text: `**bold**`, `$math$` and `` `code` `` within wrapping prose.
pub fn rich(ui: &mut Ui, f: &mut Formulas, text: &str, size: f32, color: Color32, strong: Color32) {
    #[derive(PartialEq)]
    enum Kind {
        Plain,
        Bold,
        Math,
        Code,
        Pill,
    }
    let mut segments: Vec<(Kind, String)> = vec![(Kind::Plain, String::new())];
    let mut bold = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let current = &segments.last().unwrap().0;
        if *current == Kind::Pill {
            if c == ']' && chars.peek() == Some(&']') {
                chars.next();
                segments.push((if bold { Kind::Bold } else { Kind::Plain }, String::new()));
            } else {
                segments.last_mut().unwrap().1.push(c);
            }
            continue;
        }
        if *current == Kind::Math || *current == Kind::Code {
            let close = if *current == Kind::Math { '$' } else { '`' };
            if c == close {
                if close == '$' && chars.peek() == Some(&'$') {
                    chars.next();
                }
                segments.push((if bold { Kind::Bold } else { Kind::Plain }, String::new()));
            } else {
                segments.last_mut().unwrap().1.push(c);
            }
            continue;
        }
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                bold = !bold;
                segments.push((if bold { Kind::Bold } else { Kind::Plain }, String::new()));
            }
            '$' => {
                if chars.peek() == Some(&'$') {
                    chars.next();
                }
                segments.push((Kind::Math, String::new()));
            }
            '`' => segments.push((Kind::Code, String::new())),
            '[' if chars.peek() == Some(&'[') => {
                chars.next();
                segments.push((Kind::Pill, String::new()));
            }
            _ => segments.last_mut().unwrap().1.push(c),
        }
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
        for (kind, content) in segments {
            match kind {
                Kind::Math => {
                    // KaTeX inherits the emphasized text color inside running text.
                    if !content.trim().is_empty() {
                        f.inline(ui, &content, size * KATEX_SCALE, strong);
                    }
                }
                Kind::Code => {
                    kit::text(ui, &content, mono(size, Weight::Medium), strong);
                }
                // `.math-pill`: inline formula in a bordered chip.
                Kind::Pill => {
                    ui.add_space(4.0);
                    let p = crate::theme::palette(!ui.visuals().dark_mode);
                    kit::frame(p.inset, p.border, 6, egui::Margin::symmetric(8, 2)).show(
                        ui,
                        |ui| {
                            f.inline(ui, &content, size * KATEX_SCALE, color);
                        },
                    );
                    ui.add_space(4.0);
                }
                Kind::Plain | Kind::Bold => {
                    let font = sans(
                        size,
                        if kind == Kind::Bold {
                            Weight::SemiBold
                        } else {
                            Weight::Regular
                        },
                    );
                    let c = if kind == Kind::Bold { strong } else { color };
                    for word in content.split_inclusive(' ') {
                        kit::text(ui, word, font.clone(), c);
                    }
                }
            }
        }
    });
}

/// Result figures as (key, caption, texture). Time responses stack every input in order.
pub fn plots(
    ctx: &egui::Context,
    id: &str,
    result: &Value,
) -> Vec<(String, String, egui::TextureHandle)> {
    let rank = |key: &str| ["step", "ramp", "impulse"].iter().position(|k| *k == key);
    let mut entries: Vec<(&String, &Value)> =
        result["plots"].as_object().into_iter().flatten().collect();
    entries.sort_by_key(|(key, _)| rank(key).unwrap_or(usize::MAX));
    let stacked: Vec<(String, String, &str)> = entries
        .into_iter()
        .filter_map(|(key, plot)| {
            let caption = match plot["title"].as_str() {
                Some(title) => format!("Figura — {title}"),
                None => match key.as_str() {
                    "step" => "Figura 1 — Resposta ao degrau".into(),
                    "ramp" => "Figura 2 — Resposta à rampa".into(),
                    "impulse" => "Figura 3 — Resposta ao impulso".into(),
                    _ => format!("Figura — {key}"),
                },
            };
            Some((key.clone(), caption, plot["image"].as_str()?))
        })
        .collect();
    let figures = if stacked.len() > 1 {
        stacked
    } else {
        ["image", "plot_image"]
            .iter()
            .filter_map(|key| result[*key].as_str())
            .map(|png| ("main".to_owned(), figure_title(id).to_owned(), png))
            .take(1)
            .collect()
    };
    figures
        .into_iter()
        .filter_map(|(key, caption, png)| {
            let tex = texture(ctx, &format!("{id}:{caption}"), png)?;
            Some((key, caption, tex))
        })
        .collect()
}

/// `MODULE_METAS[*].figureTitle` and the LGR/Routh panel captions.
pub fn figure_title(id: &str) -> &'static str {
    match id {
        "time" => "Figura 1 — Resposta temporal",
        "frequency" => "Figura 1 — Diagrama de Bode",
        "controllers" => "Figura 1 — Lugar Geométrico das Raízes (LGR) e Resposta Compensada",
        "state-space" => "Figura 1 — Plano complexo e polos de estados",
        "lgr" => "Figura 1 — Lugar Geométrico das Raízes",
        "routh" => "Figura 1 — Raízes no plano complexo s",
        _ => "Figura 1",
    }
}

pub struct View<'a> {
    pub id: &'a str,
    pub module: &'a Module,
    pub result: &'a Value,
    pub p: Palette,
    /// Tablet `.sci-tablet-switcher` results: formulas stacked before the metrics.
    pub tablet: bool,
}

/// Content of the Resultados panel.
pub fn results_panel(ui: &mut Ui, view: &View, f: &mut Formulas) {
    match view.id {
        "routh" => routh(ui, view, f),
        "lgr" => lgr_table(ui, view),
        _ => scientific(ui, view, f),
    }
}

// ---------------------------------------------------------------------------
// Time, frequency, controllers and state space

fn scientific(ui: &mut Ui, view: &View, f: &mut Formulas) {
    let p = &view.p;
    let r = view.result;
    let latex = list(&r["latex"]);
    ui.spacing_mut().item_spacing.y = 0.0;
    if view.id == "time" {
        let step = &r["details"]["step"];
        let cell = |key: &str, digits: usize, unit: &str| -> String {
            match step[key].as_f64().filter(|n| n.is_finite()) {
                Some(n) if unit.is_empty() => crate::texfmt::pt_br(n, digits),
                Some(n) => format!("{} {unit}", crate::texfmt::pt_br(n, digits)),
                None => "—".into(),
            }
        };
        let has = !r.is_null();
        let rows = [
            (
                "Sobressinal",
                if has && step["overshoot_percent"].is_null() {
                    "0,0 %".into()
                } else {
                    cell("overshoot_percent", 1, "%")
                },
            ),
            ("Tempo de pico", cell("peak_time", 2, "s")),
            ("Tempo de subida", cell("rise_time", 2, "s")),
            ("Tempo de acomodação", cell("settling_time", 1, "s")),
            ("Valor final", cell("final_value", 3, "")),
            ("Razão de amortecimento (ζ)", cell("damping_ratio", 3, "")),
            (
                "Frequência natural (ωn)",
                cell("natural_frequency", 3, "rad/s"),
            ),
            (
                "Estabilidade",
                if !has {
                    "—".into()
                } else if r["details"]["stable"].as_bool().unwrap_or(true) {
                    "Estável".into()
                } else {
                    "Instável".into()
                },
            ),
        ];
        if view.tablet {
            latex_stack(ui, view, f, latex);
        }
        for (i, (label, value)) in rows.iter().enumerate() {
            kit::striped_row(ui, p, i, label, value);
        }
        if !view.tablet {
            latex_grid(ui, view, f, latex);
        }
    } else if r.is_null() {
        for (i, key) in view.module.primary_keys.iter().enumerate() {
            kit::striped_row(ui, p, i, key, "—");
        }
    } else {
        if view.tablet {
            latex_stack(ui, view, f, latex);
        } else {
            latex_grid(ui, view, f, latex);
        }
        for (i, metric) in list(&r["metrics"]).iter().enumerate() {
            if let Some(header) = metric["header"].as_str() {
                kit::group_header(ui, p, header);
            } else if let Some(label) = metric["label"].as_str() {
                let unit = metric["unit"]
                    .as_str()
                    .map(|u| format!(" {u}"))
                    .unwrap_or_default();
                kit::striped_row(
                    ui,
                    p,
                    i,
                    label,
                    &format!("{}{unit}", display_number(&metric["value"])),
                );
            }
        }
    }
    steps_panel(ui, p, f, list(&r["details"]["steps"]));
}

/// `.scientific-latex-grid`: two columns of cards; the last odd card spans both.
fn latex_grid(ui: &mut Ui, view: &View, f: &mut Formulas, items: &[Value]) {
    if items.is_empty() {
        return;
    }
    let p = &view.p;
    ui.add_space(2.0);
    let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 7.0), Sense::hover());
    hline(ui, line.x_range(), line.top(), p.border);
    let width = ui.available_width();
    let em = 13.6 * KATEX_SCALE;
    let mut i = 0;
    while i < items.len() {
        let pair = if i + 1 < items.len() { 2 } else { 1 };
        let col = if pair == 2 {
            (width - 6.0) / 2.0
        } else {
            width
        };
        let height = (0..pair)
            .map(|j| {
                10.0 + 2.0
                    + 21.0
                    + f.size(text(&items[i + j]["value"]), em)
                        .map_or(22.0, |s| s.y)
                    + 10.0
            })
            .fold(0.0, f32::max);
        let (row, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        for j in 0..pair {
            let item = &items[i + j];
            let rect = Rect::from_min_size(
                pos2(row.left() + j as f32 * (col + 6.0), row.top()),
                vec2(col, height),
            );
            let highlight = view.id == "controllers" && i + j < 2;
            fill(
                ui,
                rect,
                6,
                if highlight {
                    p.accent_tint
                } else {
                    p.latex_card
                },
                Stroke::new(1.0, p.border),
            );
            if highlight {
                let bar = Rect::from_min_max(rect.min, pos2(rect.left() + 3.0, rect.bottom()));
                fill(
                    ui,
                    bar,
                    egui::CornerRadius {
                        nw: 6,
                        sw: 6,
                        ne: 0,
                        se: 0,
                    },
                    p.accent,
                    Stroke::NONE,
                );
            }
            let inner = rect.shrink2(vec2(8.0 + if highlight { 2.0 } else { 0.0 }, 5.0));
            let label = item["label"].as_str().unwrap_or("Equação").to_uppercase();
            let g = kit::ellipsized(
                ui,
                &label,
                sans(11.5, Weight::Bold),
                p.muted,
                0.5,
                inner.width(),
            );
            ui.painter().galley(inner.min, g, p.muted);
            let math_rect = Rect::from_min_max(pos2(inner.left(), inner.top() + 21.0), inner.max);
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(math_rect)
                    .id_salt(("latex", i + j))
                    .layout(egui::Layout::top_down(egui::Align::Center)),
            );
            child.set_clip_rect(math_rect);
            f.math_centered(&mut child, text(&item["value"]), em, p.text);
        }
        ui.add_space(6.0);
        i += pair;
    }
}

/// Tablet results tab: one roomy card per formula, above the metrics table.
fn latex_stack(ui: &mut Ui, view: &View, f: &mut Formulas, items: &[Value]) {
    let p = &view.p;
    let em = 17.0 * KATEX_SCALE;
    ui.add_space(17.0);
    for (i, item) in items.iter().enumerate() {
        let width = ui.available_width() - 40.0;
        let math = f.size(text(&item["value"]), em).map_or(24.0, |s| s.y);
        let (row, _) = ui.allocate_exact_size(
            vec2(ui.available_width(), 18.0 + 16.0 + 14.0 + math + 20.0),
            Sense::hover(),
        );
        let rect = Rect::from_min_size(
            pos2(row.left() + 20.0, row.top()),
            vec2(width, row.height()),
        );
        let highlight = view.id == "controllers" && i < 2;
        fill(
            ui,
            rect,
            6,
            if highlight {
                p.accent_tint
            } else {
                p.latex_card
            },
            Stroke::new(1.0, p.border),
        );
        if highlight {
            let bar = Rect::from_min_max(rect.min, pos2(rect.left() + 3.0, rect.bottom()));
            fill(
                ui,
                bar,
                egui::CornerRadius {
                    nw: 6,
                    sw: 6,
                    ne: 0,
                    se: 0,
                },
                p.accent,
                Stroke::NONE,
            );
        }
        let inner = rect.shrink2(vec2(18.0, 18.0));
        let label = item["label"].as_str().unwrap_or("Equação").to_uppercase();
        let g = kit::ellipsized(
            ui,
            &label,
            sans(13.4, Weight::Bold),
            p.muted,
            0.5,
            inner.width(),
        );
        ui.painter().galley(inner.min, g, p.muted);
        let math_rect = Rect::from_min_max(
            pos2(inner.left(), inner.top() + 30.0),
            pos2(inner.right(), rect.bottom() - 16.0),
        );
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(math_rect)
                .id_salt(("latex-stack", i))
                .layout(egui::Layout::top_down(egui::Align::Center)),
        );
        child.set_clip_rect(math_rect);
        f.math_centered(&mut child, text(&item["value"]), em, p.text);
        ui.add_space(13.0);
    }
    ui.add_space(8.0);
}

/// `.ctrl-steps-panel`: open `<details>` with the controller design memory.
fn steps_panel(ui: &mut Ui, p: &Palette, f: &mut Formulas, steps: &[Value]) {
    if steps.is_empty() {
        return;
    }
    ui.add_space(10.0);
    let id = ui.id().with("steps-open");
    let open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(true);
    kit::frame(p.surface, p.border_strong, 4, 0).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let (bar, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 37.0), Sense::click());
        fill(
            ui,
            bar.shrink(1.0),
            egui::CornerRadius {
                nw: 3,
                ne: 3,
                sw: 0,
                se: 0,
            },
            p.header,
            Stroke::NONE,
        );
        if open {
            hline(ui, bar.x_range(), bar.bottom() - 1.0, p.border);
        }
        let g = galley(
            ui,
            "Memória de Cálculo Passo a Passo (LGR)",
            sans(13.6, Weight::Bold),
            p.text,
            0.0,
        );
        ui.painter().galley(
            pos2(bar.left() + 11.0, bar.center().y - g.size().y / 2.0),
            g,
            p.text,
        );
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            ui.data_mut(|d| d.insert_temp(id, !open));
        }
        if !open {
            return;
        }
        egui::Frame::new().inner_margin(8).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            for (i, step) in steps.iter().enumerate() {
                ui.push_id(i, |ui| step_card(ui, p, f, step));
            }
        });
    });
}

fn step_card(ui: &mut Ui, p: &Palette, f: &mut Formulas, step: &Value) {
    kit::frame(p.inset, p.border, 4, 0).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.set_width(ui.available_width());
        let badge = format!("PASSO {}", raw(&step["number"]));
        let title_g = kit::wrapped(
            ui,
            text(&step["title"]),
            sans(13.3, Weight::SemiBold),
            p.text,
            ui.available_width() - 90.0,
        );
        let (bar, _) = ui.allocate_exact_size(
            vec2(ui.available_width(), title_g.size().y.max(21.0) + 12.0),
            Sense::hover(),
        );
        fill(
            ui,
            bar.shrink(1.0),
            egui::CornerRadius {
                nw: 3,
                ne: 3,
                sw: 0,
                se: 0,
            },
            p.step_header,
            Stroke::NONE,
        );
        hline(ui, bar.x_range(), bar.bottom() - 1.0, p.border);
        let bg = galley(ui, &badge, sans(11.5, Weight::Bold), p.accent, 0.0);
        let badge_rect = Rect::from_min_size(
            pos2(bar.left() + 8.0, bar.center().y - 10.5),
            bg.size() + vec2(12.0, 4.0),
        );
        fill(
            ui,
            badge_rect,
            3,
            p.accent.gamma_multiply(0.18),
            Stroke::NONE,
        );
        ui.painter()
            .galley(badge_rect.min + vec2(6.0, 2.0), bg, p.accent);
        ui.painter().galley(
            pos2(
                badge_rect.right() + 8.0,
                bar.center().y - title_g.size().y / 2.0,
            ),
            title_g,
            p.text,
        );
        egui::Frame::new().inner_margin(8).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            for (j, eq) in list(&step["equations"]).iter().enumerate() {
                ui.push_id(j, |ui| {
                    kit::frame(p.surface, p.border, 3, egui::Margin::symmetric(6, 4)).show(
                        ui,
                        |ui| {
                            ui.set_width(ui.available_width());
                            f.math_centered(ui, text(eq), 13.7 * KATEX_SCALE, p.text);
                        },
                    );
                });
            }
            for note in list(&step["notes"]) {
                ui.horizontal_top(|ui| {
                    ui.add_space(4.0);
                    kit::text(ui, "•  ", sans(13.0, Weight::Regular), p.muted);
                    ui.vertical(|ui| rich(ui, f, text(note), 13.0, p.muted, p.text));
                });
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Root locus

fn lgr_table(ui: &mut Ui, view: &View) {
    let p = &view.p;
    let d = &view.result["detalhes"];
    ui.spacing_mut().item_spacing.y = 0.0;
    let rows: Vec<(&str, String)> = if d.is_null() {
        [
            "Polos em malha aberta",
            "Zeros em malha aberta",
            "Número de ramos",
            "Centroide (σₐ)",
            "Assíntotas",
            "Pontos de ruptura",
            "Cruzamento jω",
        ]
        .into_iter()
        .map(|l| (l, "—".to_owned()))
        .collect()
    } else {
        let pn = d["P"].as_u64().unwrap_or(0);
        let zn = d["Z"].as_u64().unwrap_or(0);
        let strs = |key: &str| {
            list(&d[key])
                .iter()
                .map(|v| text(&v["str"]).to_owned())
                .collect::<Vec<_>>()
                .join("; ")
        };
        let asym = if pn > zn {
            let angles: Vec<String> = list(&d["angulos_assintotas"])
                .iter()
                .filter_map(|a| a["graus"].as_f64())
                .map(|g| format!("{}°", g.round()))
                .collect();
            if angles.is_empty() {
                format!("{} assíntotas", pn - zn)
            } else {
                format!("{} assíntotas ({})", pn - zn, angles.join(", "))
            }
        } else {
            "Nenhuma assíntota".into()
        };
        let breaks = list(&d["break_points"]);
        let jw = list(&d["jw_cruzamentos"]);
        vec![
            (
                "Polos em malha aberta",
                if pn > 0 {
                    format!("{pn} ({})", strs("polos"))
                } else {
                    "—".into()
                },
            ),
            (
                "Zeros em malha aberta",
                if zn > 0 {
                    format!("{zn} ({})", strs("zeros"))
                } else {
                    "0 (Nenhum)".into()
                },
            ),
            (
                "Número de ramos",
                d["ramos"].as_u64().unwrap_or(pn).to_string(),
            ),
            (
                "Centroide (σₐ)",
                d["centroide"]
                    .as_f64()
                    .map_or("N/A".into(), |c| fixed(c, 3)),
            ),
            ("Assíntotas", asym),
            (
                "Pontos de ruptura",
                if breaks.is_empty() {
                    "Nenhum no eixo real".into()
                } else {
                    breaks
                        .iter()
                        .map(|b| match (b["s"].as_f64(), b["K"].as_f64()) {
                            (Some(s), Some(k)) => format!("{} (K={})", fixed(s, 3), fixed(k, 2)),
                            _ => raw(&b["s"]),
                        })
                        .collect::<Vec<_>>()
                        .join("; ")
                },
            ),
            (
                "Cruzamento jω",
                if jw.is_empty() {
                    "Sem cruzamento".into()
                } else {
                    jw.iter()
                        .map(|c| {
                            format!(
                                "±j{:.2} (K={:.1})",
                                c["w"].as_f64().unwrap_or(0.0),
                                c["K"].as_f64().unwrap_or(0.0)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ")
                },
            ),
        ]
    };
    for (i, (label, value)) in rows.iter().enumerate() {
        kit::striped_row(ui, p, i, label, value);
    }
}

/// jω crossings come in ±ω pairs with the same gain; the memorial lists each once.
pub fn unique_crossings(crossings: &[Value]) -> Vec<(f64, f64)> {
    let mut seen = HashSet::new();
    crossings
        .iter()
        .filter_map(|c| Some((c["w"].as_f64()?.abs(), c["K"].as_f64()?)))
        .filter(|(w, k)| seen.insert((format!("{w:.2}"), format!("{k:.2}"))))
        .collect()
}

/// `formatComplexToLatex`: `-2.00 +2.00j` becomes `-2.00 + j2.00`.
fn complex_tex(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        let mut j = i;
        if chars[j] == '+' || chars[j] == '-' {
            j += 1;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
        }
        let digits = j;
        while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
            j += 1;
        }
        if j > digits && j < chars.len() && chars[j] == 'j' {
            let sign = match chars[start] {
                '-' => "- ",
                '+' => "+ ",
                _ => "",
            };
            let number: String = chars[digits..j].iter().collect();
            out.push_str(&format!("{sign}j{number}"));
            i = j + 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out.trim().to_owned()
}

/// `#view-memorial`: the seven classic Evans steps. Returns true to open the deductions.
pub fn lgr_memorial(ui: &mut Ui, view: &View, f: &mut Formulas) -> bool {
    let p = &view.p;
    let mut open_deductions = false;
    kit::frame(p.surface, p.border, 6, egui::Margin::symmetric(18, 14)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let g = galley(
                ui,
                "7 PASSOS CLÁSSICOS DE EVANS",
                mono(13.0, Weight::Medium),
                p.accent,
                1.0,
            );
            let (rect, _) = ui.allocate_exact_size(g.size() + vec2(20.0, 12.0), Sense::hover());
            fill(
                ui,
                rect,
                3,
                Color32::TRANSPARENT,
                Stroke::new(1.0, p.accent.gamma_multiply(0.7)),
            );
            ui.painter().galley(rect.min + vec2(10.0, 6.0), g, p.accent);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if kit::mini_button(ui, p, "Deduções detalhadas →").clicked() {
                    open_deductions = true;
                }
            });
        });
    });
    ui.add_space(8.0);
    let d = &view.result["detalhes"];
    if d.is_null() {
        kit::empty_box(
            ui,
            p,
            &[
                (
                    "Nenhum memorial calculado ainda. Insira a função de transferência e clique em ",
                    false,
                ),
                ("Calcular", true),
                (".", false),
            ],
            false,
            440.0,
        );
        return open_deductions;
    }
    let pn = d["P"].as_u64().unwrap_or(0);
    let zn = d["Z"].as_u64().unwrap_or(0);
    let join = |key: &str| {
        list(&d[key])
            .iter()
            .map(|v| format!("${}$", complex_tex(text(&v["str"]))))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let body = 13.6;
    let (t, m) = (p.text, p.muted);
    memorial_card(
        ui,
        p,
        "Passos 1, 2, 3:",
        "Ramos, Polos, Zeros e Eixo Real",
        |ui| {
            item_box(ui, p.accent, p.inset, |ui| {
                rich(
                    ui,
                    f,
                    &format!("**Número de Polos ($P$):** {pn}  |  **Número de Zeros ($Z$):** {zn}"),
                    body,
                    m,
                    t,
                )
            });
            let poles = if pn > 0 {
                join("polos")
            } else {
                "Nenhum polo finito".into()
            };
            let zeros = if zn > 0 {
                join("zeros")
            } else {
                "Nenhum zero finito".into()
            };
            rich(
                ui,
                f,
                &format!("• **Polos de Malha Aberta ($K=0$):** {poles}"),
                body,
                m,
                t,
            );
            rich(
                ui,
                f,
                &format!("• **Zeros de Malha Aberta ($K\\to\\infty$):** {zeros}"),
                body,
                m,
                t,
            );
            rich(
                ui,
                f,
                &format!(
                    "• **Total de Ramos do LGR ($n = \\max(P, Z)$):** {}",
                    raw(&d["ramos"])
                ),
                body,
                m,
                t,
            );
            rich(
                ui,
                f,
                "• **Simetria:** O LGR é perfeitamente simétrico em relação ao Eixo Real ($\\sigma$).",
                body,
                m,
                t,
            );
        },
    );
    memorial_card(ui, p, "Passo 4:", "Assíntotas e Centroide (σₐ)", |ui| {
        if pn > zn {
            item_box(ui, p.purple, p.purple.gamma_multiply(0.08), |ui| {
                rich(
                    ui,
                    f,
                    &format!(
                        "**Ramos que tendem ao infinito ($P - Z$):** {} ramo(s)",
                        pn - zn
                    ),
                    body,
                    m,
                    t,
                )
            });
            rich(
                ui,
                f,
                &format!(
                    "• **Centroide das Assíntotas ($\\sigma_a$):** $\\sigma_a = {:.3}$",
                    d["centroide"].as_f64().unwrap_or(0.0)
                ),
                body,
                m,
                t,
            );
            let pills: Vec<String> = list(&d["angulos_assintotas"])
                .iter()
                .map(|a| {
                    format!(
                        "[[\\theta_{{{}}} = {:.1}^\\circ]]",
                        raw(&a["k"]),
                        a["graus"].as_f64().unwrap_or(0.0)
                    )
                })
                .collect();
            rich(
                ui,
                f,
                &format!(
                    "• **Ângulos das Assíntotas ($\\theta_k = \\frac{{(2k+1)180^\\circ}}{{P-Z}}$):** {}",
                    pills.join(" ")
                ),
                body,
                m,
                t,
            );
        } else {
            item_box(ui, p.accent, p.inset, |ui| {
                rich(
                    ui,
                    f,
                    "Como $P \\le Z$, todos os ramos terminam nos zeros finitos. Não há assíntotas direcionadas ao infinito.",
                    body,
                    m,
                    t,
                )
            });
        }
    });
    memorial_card(
        ui,
        p,
        "Passo 5:",
        "Pontos de Ruptura (Break-in/Breakaway)",
        |ui| {
            let breaks = list(&d["break_points"]);
            if breaks.is_empty() {
                item_box(ui, p.accent, p.inset, |ui| {
                    rich(
                        ui,
                        f,
                        "Nenhum ponto de partida ou retorno no eixo real com ganho $K > 0$ válido.",
                        body,
                        m,
                        t,
                    )
                });
            } else {
                rich(
                    ui,
                    f,
                    "Pontos onde $\\frac{dK}{ds} = 0 \\iff N'(s)D(s) - N(s)D'(s) = 0$:",
                    body,
                    m,
                    t,
                );
                for b in breaks {
                    item_box(ui, p.emerald, p.emerald.gamma_multiply(0.08), |ui| {
                        ui.horizontal(|ui| {
                        badge_icon(ui, crate::icons::PIN, p.emerald);
                        rich(ui, f, &format!("Ponto no Eixo Real: $s = {:.3}$  |  Ganho Crítico: $K = {:.2}$", b["s"].as_f64().unwrap_or(0.0), b["K"].as_f64().unwrap_or(0.0)), body, m, t);
                    });
                    });
                }
            }
        },
    );
    memorial_card(
        ui,
        p,
        "Passo 6:",
        "Cruzamento do Eixo Imaginário (jω)",
        |ui| {
            let crossings = unique_crossings(list(&d["jw_cruzamentos"]));
            if crossings.is_empty() {
                item_box(ui, p.accent, p.inset, |ui| {
                    rich(
                        ui,
                        f,
                        "Nenhum cruzamento com o eixo $j\\omega$ detectado na faixa de ganho analisada (o sistema permanece no semiplano atual).",
                        body,
                        m,
                        t,
                    )
                });
            } else {
                rich(
                    ui,
                    f,
                    "Cruzamentos com o eixo imaginário ($j\\omega$):",
                    body,
                    m,
                    t,
                );
                for (w, k) in crossings {
                    item_box(ui, p.amber, p.amber.gamma_multiply(0.08), |ui| {
                        ui.horizontal(|ui| {
                        badge_icon(ui, crate::icons::BOLT, p.amber);
                        rich(ui, f, &format!("Cruzamento detectado em: $s = \\pm j{w:.2}$  |  Ganho Limite de Estabilidade: $K_{{crit}} = {k:.2}$"), body, m, t);
                    });
                    });
                }
            }
        },
    );
    memorial_card(ui, p, "Passo 7:", "Ângulos de Partida e Chegada", |ui| {
        let departures = list(&d["angulos_partida"]);
        let arrivals = list(&d["angulos_chegada"]);
        if departures.is_empty() && arrivals.is_empty() {
            item_box(ui, p.accent, p.inset, |ui| {
                rich(
                    ui,
                    f,
                    "Não há polos ou zeros complexos conjugados nesta função de transferência (todas as singularidades são puramente reais).",
                    body,
                    m,
                    t,
                )
            });
        }
        if !departures.is_empty() {
            rich(
                ui,
                f,
                "• **Ângulo de Partida em Polos Complexos ($\\theta_d$):**",
                body,
                m,
                t,
            );
            rich(
                ui,
                f,
                "Condição angular: $\\theta_d = 180^\\circ + \\sum \\angle(p - z) - \\sum \\angle(p - p_{outros})$",
                13.0,
                m,
                t,
            );
            for a in departures {
                item_box(ui, p.purple, p.purple.gamma_multiply(0.08), |ui| {
                    ui.horizontal(|ui| {
                        badge_icon(ui, crate::icons::ARROW_UP, p.purple);
                        rich(ui, f, &format!("Para o polo complexo $p = {}$: Ângulo de partida $\\theta_d = {:.1}^\\circ$", complex_tex(text(&a["polo_str"])), a["angulo"].as_f64().unwrap_or(0.0)), body, m, t);
                    });
                });
            }
        }
        if !arrivals.is_empty() {
            rich(
                ui,
                f,
                "• **Ângulo de Chegada em Zeros Complexos ($\\theta_a$):**",
                body,
                m,
                t,
            );
            rich(
                ui,
                f,
                "Condição angular: $\\theta_a = 180^\\circ + \\sum \\angle(z - p) - \\sum \\angle(z - z_{outros})$",
                13.0,
                m,
                t,
            );
            for a in arrivals {
                item_box(ui, p.emerald, p.emerald.gamma_multiply(0.08), |ui| {
                    ui.horizontal(|ui| {
                        badge_icon(ui, crate::icons::TARGET, p.emerald);
                        rich(ui, f, &format!("Para o zero complexo $z = {}$: Ângulo de chegada $\\theta_a = {:.1}^\\circ$", complex_tex(text(&a["zero_str"])), a["angulo"].as_f64().unwrap_or(0.0)), body, m, t);
                    });
                });
            }
        }
    });
    open_deductions
}

fn memorial_card(ui: &mut Ui, p: &Palette, bold: &str, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.push_id(bold, |ui| {
        kit::frame(p.surface, p.border, 6, 0).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(ui.available_width());
            let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::hover());
            fill(
                ui,
                bar.shrink(1.0),
                egui::CornerRadius {
                    nw: 5,
                    ne: 5,
                    sw: 0,
                    se: 0,
                },
                p.inset,
                Stroke::NONE,
            );
            hline(ui, bar.x_range(), bar.bottom() - 1.0, p.border);
            let a = galley(ui, bold, sans(16.0, Weight::SemiBold), p.text, 0.0);
            let x = bar.left() + 18.0;
            let y = bar.center().y - a.size().y / 2.0;
            let w = a.size().x;
            ui.painter().galley(pos2(x, y), a, p.text);
            let b = galley(ui, title, sans(16.0, Weight::Regular), p.text, 0.0);
            ui.painter().galley(pos2(x + w + 14.0, y), b, p.text);
            egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: 18,
                    right: 18,
                    top: 16,
                    bottom: 16,
                })
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    ui.set_width(ui.available_width());
                    add(ui);
                });
        });
    });
    ui.add_space(8.0);
}

/// `.step-item-box`: tinted row with a 3 px left rule.
fn item_box(ui: &mut Ui, rule: Color32, bg: Color32, add: impl FnOnce(&mut Ui)) {
    let inner = egui::Frame::new()
        .fill(bg)
        .corner_radius(egui::CornerRadius {
            nw: 0,
            sw: 0,
            ne: 8,
            se: 8,
        })
        .inner_margin(egui::Margin {
            left: 17,
            right: 14,
            top: 10,
            bottom: 10,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
    let r = inner.response.rect;
    fill(
        ui,
        Rect::from_min_max(r.min, pos2(r.left() + 3.0, r.bottom())),
        0,
        rule,
        Stroke::NONE,
    );
}

fn badge_icon(ui: &mut Ui, icon: crate::icons::Icon, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
    fill(ui, rect, 6, color.gamma_multiply(0.2), Stroke::NONE);
    crate::icons::paint(ui, rect.shrink(5.0), icon, color);
    ui.add_space(4.0);
}

/// `#view-theory`.
pub fn lgr_theory(ui: &mut Ui, p: &Palette) {
    kit::frame(p.surface, p.border, 6, 12).show(ui, |ui| {
        ui.set_width(ui.available_width());
        kit::text(ui, "Os 7 Passos Clássicos do Lugar Geométrico das Raízes (LGR)", sans(14.0, Weight::SemiBold), p.text);
        ui.add_space(4.0);
        for (n, rule) in [
            ("Passo 1:", "Ramos = max(P, Z). Início nos polos (K=0) e término nos zeros ou infinito."),
            ("Passo 2:", "Simetria em relação ao eixo real."),
            ("Passo 3:", "Pertencem ao eixo real os pontos à esquerda de um número ímpar de polos e zeros reais."),
            ("Passo 4:", "P - Z assíntotas com centroide σₐ = (Σp - Σz)/(P - Z) e ângulos θ_k = (2k+1)180°/(P - Z)."),
            ("Passo 5:", "Pontos de dispersão/confluência onde dK/ds = 0."),
            ("Passo 6:", "Cruzamento do eixo imaginário jω com ganho crítico K_crit."),
            ("Passo 7:", "Ângulos de partida e chegada para polos/zeros complexos."),
        ] {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                kit::text(ui, n, sans(13.0, Weight::SemiBold), p.accent);
                kit::text(ui, " ", sans(13.0, Weight::Regular), p.text);
                for word in rule.split_inclusive(' ') {
                    kit::text(ui, word, sans(13.0, Weight::Regular), p.text);
                }
            });
        }
    });
}

/// `.memorial-sidebar` content: formal deductions of every step (`filter` 0 = all).
pub fn lgr_deductions(ui: &mut Ui, p: &Palette, f: &mut Formulas, result: &Value, filter: usize) {
    let pap = &result["detalhes"]["passo_a_passo"];
    if pap.as_object().is_none_or(|o| o.is_empty()) {
        kit::text(
            ui,
            "Trace o LGR para visualizar as deduções matemáticas detalhadas.",
            sans(13.6, Weight::Regular),
            p.muted,
        );
        return;
    }
    let (t, m) = (p.text, p.muted);
    let body = 13.6;
    let em = 13.6 * KATEX_SCALE;
    let show = |n: usize| filter == 0 || filter == n;
    let p1 = &pap["passo_1_2_3"];
    if show(1) {
        deduction(ui, p, "1, 2, 3", "Polos, Zeros, Ramos e Eixo Real", |ui| {
            deduction_item(
                ui,
                p,
                "1. Equação Característica em Malha Fechada",
                |ui| {
                    rich(
                        ui,
                        f,
                        "A condição fundamental de malha fechada $1 + K G(s) = 0$ resulta na equação característica polinomial:",
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p1["eq_caracteristica"]), em);
                },
            );
            deduction_item(
                ui,
                p,
                "2. Fatoração e Singularidades de Malha Aberta",
                |ui| {
                    rich(
                        ui,
                        f,
                        "Forma fatorada da função de malha aberta $G(s)$:",
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p1["eq_fatorada"]), em);
                    rich(
                        ui,
                        f,
                        &format!(
                            "• Polos ($K = 0$): $P = {}$ raiz(es) de $D(s) = 0$",
                            raw(&p1["P"])
                        ),
                        body,
                        t,
                        t,
                    );
                    rich(
                        ui,
                        f,
                        &format!(
                            "• Zeros ($K \\to \\infty$): $Z = {}$ raiz(es) de $N(s) = 0$",
                            raw(&p1["Z"])
                        ),
                        body,
                        t,
                        t,
                    );
                    rich(
                        ui,
                        f,
                        &format!(
                            "• Total de Ramos do LGR: $n = \\max(P, Z) = {}$ ramo(s)",
                            raw(&p1["ramos"])
                        ),
                        body,
                        t,
                        t,
                    );
                },
            );
            deduction_item(
                ui,
                p,
                "3. Segmentos no Eixo Real (Regra da Contagem Ímpar)",
                |ui| {
                    rich(
                        ui,
                        f,
                        "Um ponto sobre o eixo real pertence ao LGR se o total de polos e zeros reais à sua direita for **ímpar**:",
                        body,
                        t,
                        t,
                    );
                    let rows: Vec<[Cell; 3]> = list(&p1["segmentos_eixo_real"])
                        .iter()
                        .map(|s| {
                            let ok = s["pertence"].as_bool() == Some(true);
                            [
                                Cell::Math(text(&s["intervalo"]).to_owned()),
                                Cell::Text(raw(&s["contagem_direita"])),
                                Cell::Badge(
                                    ok,
                                    if ok { "Pertence ao LGR" } else { "Fora do LGR" }.into(),
                                    String::new(),
                                ),
                            ]
                        })
                        .collect();
                    deduction_table(
                        ui,
                        p,
                        f,
                        ["Intervalo Real", "À Direita", "Status no LGR"],
                        &rows,
                        "Nenhuma singularidade real para delimitar intervalos.",
                    );
                },
            );
            deduction_item(ui, p, "4. Simetria Reflexiva do LGR", |ui| {
                rich(ui, f, text(&p1["simetria_justificativa"]), body, t, t)
            });
        });
    }
    let p4 = &pap["passo_4"];
    if show(4) {
        deduction(ui, p, "Passo 4", "Assíntotas e Centroide", |ui| {
            if p4["tem_assintotas"].as_bool() == Some(true) {
                deduction_item(ui, p, "1. Centroide das Assíntotas (σa)", |ui| {
                    rich(
                        ui,
                        f,
                        "Fórmula analítica fundamental baseada na conservação da soma das raízes:",
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p4["formula_centroide"]), em);
                    rich(
                        ui,
                        f,
                        "Substituição numérica com soma dos polos e soma dos zeros:",
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p4["substituicao_centroide"]), em);
                });
                deduction_item(ui, p, "2. Ângulos das Assíntotas (θk)", |ui| {
                    let last = p4["n_assintotas"].as_i64().unwrap_or(1) - 1;
                    rich(
                        ui,
                        f,
                        &format!("Condição angular para $k = 0, 1, \\dots, {last}$:"),
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p4["formula_angulos"]), em);
                    rich(
                        ui,
                        f,
                        "Desenvolvimento analítico de cada ramo assintótico que tende ao infinito:",
                        body,
                        t,
                        t,
                    );
                    for (i, a) in list(&p4["angulos_deduzidos"]).iter().enumerate() {
                        ui.push_id(i, |ui| math_box(ui, p, f, text(&a["formula"]), em));
                    }
                });
            } else {
                rich(
                    ui,
                    f,
                    "Como $P \\le Z$, todos os ramos terminam nos zeros finitos. Não há assíntotas direcionadas ao infinito.",
                    body,
                    m,
                    t,
                );
            }
        });
    }
    let p5 = &pap["passo_5"];
    if show(5) {
        deduction(
            ui,
            p,
            "Passo 5",
            "Break-in / Breakaway (Pontos de Quebra)",
            |ui| {
                deduction_item(ui, p, "1. Condição de Raiz Múltipla (dK/ds = 0)", |ui| {
                    rich(
                        ui,
                        f,
                        "Expressão do ganho $K$ em função de $s$ na malha fechada:",
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p5["formula_derivada"]), em);
                    rich(
                        ui,
                        f,
                        "Anulando o numerador da derivada, obtém-se a condição polinomial:",
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p5["condicao"]), em);
                });
                deduction_item(ui, p, "2. Derivação dos Polinômios", |ui| {
                    rich(
                        ui,
                        f,
                        "Derivadas analíticas de $N(s)$ e $D(s)$:",
                        body,
                        t,
                        t,
                    );
                    math_box(
                        ui,
                        p,
                        f,
                        &format!(
                            r"N'(s) = \frac{{d}}{{ds}} N(s) = {}",
                            text(&p5["derivada_N"])
                        ),
                        em,
                    );
                    math_box(
                        ui,
                        p,
                        f,
                        &format!(
                            r"D'(s) = \frac{{d}}{{ds}} D(s) = {}",
                            text(&p5["derivada_D"])
                        ),
                        em,
                    );
                    rich(
                        ui,
                        f,
                        "Polinômio de quebra expandido $P_{break}(s) = 0$:",
                        body,
                        t,
                        t,
                    );
                    math_box(ui, p, f, text(&p5["polinomio_break"]), em);
                });
                deduction_item(
                    ui,
                    p,
                    "3. Resolução das Raízes e Verificação de Validade",
                    |ui| {
                        rich(
                            ui,
                            f,
                            "Critérios de aceitação: $s \\in \\mathbb{R}$, ganho positivo $K(s) > 0$ e pertencimento aos ramos reais do LGR:",
                            body,
                            t,
                            t,
                        );
                        let rows: Vec<[Cell; 3]> = list(&p5["raizes_analisadas"])
                            .iter()
                            .map(|r| {
                                let ok = r["valido"].as_bool() == Some(true);
                                let k = r["K"]
                                    .as_f64()
                                    .map_or("N/A".to_owned(), |k| format!("K = {k:.2}"));
                                [
                                    Cell::Math(format!("s = {}", complex_tex(text(&r["s_str"])))),
                                    Cell::Math(k),
                                    Cell::Badge(
                                        ok,
                                        if ok { "Válido" } else { "Descartado" }.into(),
                                        text(&r["motivo"]).to_owned(),
                                    ),
                                ]
                            })
                            .collect();
                        deduction_table(
                            ui,
                            p,
                            f,
                            ["Raiz Candidata", "Ganho K(s)", "Validação e Justificativa"],
                            &rows,
                            "Nenhuma raiz candidata encontrada.",
                        );
                    },
                );
            },
        );
    }
    let p6 = &pap["passo_6"];
    if show(6) {
        deduction(
            ui,
            p,
            "Passo 6",
            "Cruzamento com o Eixo Imaginário (jω)",
            |ui| {
                deduction_item(
                    ui,
                    p,
                    "1. Substituição de Frequência Limite (s = jω)",
                    |ui| {
                        rich(
                            ui,
                            f,
                            "Equação característica em malha fechada:",
                            body,
                            t,
                            t,
                        );
                        math_box(ui, p, f, text(&p6["eq_caracteristica"]), em);
                        rich(
                            ui,
                            f,
                            "Substituindo $s = j\\omega$ com potências de $j$ ($j^2 = -1$, $j^3 = -j$, $j^4 = 1$):",
                            body,
                            t,
                            t,
                        );
                        math_box(ui, p, f, text(&p6["substituicao_jw"]), em);
                    },
                );
                deduction_item(
                    ui,
                    p,
                    "2. Decomposição Algébrica (Parte Real e Imaginária)",
                    |ui| {
                        rich(
                            ui,
                            f,
                            "Equações desacopladas que devem anular-se simultaneamente:",
                            body,
                            t,
                            t,
                        );
                        math_box(
                            ui,
                            p,
                            f,
                            &format!(
                                r"\text{{Re}}(\omega, K): \quad {}",
                                text(&p6["parte_real_eq"])
                            ),
                            em,
                        );
                        math_box(
                            ui,
                            p,
                            f,
                            &format!(
                                r"\text{{Im}}(\omega, K): \quad {}",
                                text(&p6["parte_imaginaria_eq"])
                            ),
                            em,
                        );
                    },
                );
                deduction_item(
                    ui,
                    p,
                    "3. Soluções Reais de Cruzamento (Kcrit > 0, ω > 0)",
                    |ui| {
                        if p6["tem_cruzamento"].as_bool() == Some(true) {
                            rich(
                                ui,
                                f,
                                "Resolvendo o sistema algébrico para frequências críticas no limiar de estabilidade:",
                                body,
                                t,
                                t,
                            );
                            for c in list(&p6["cruzamentos"]) {
                                item_box(ui, p.amber, p.amber.gamma_multiply(0.08), |ui| {
                                    rich(
                                        ui,
                                        f,
                                        &format!(
                                            "Ponto crítico detectado em: ${}$ com ganho limite ${}$",
                                            text(&c["s_str"]),
                                            text(&c["K_str"])
                                        ),
                                        body,
                                        t,
                                        t,
                                    )
                                });
                            }
                        } else {
                            rich(
                                ui,
                                f,
                                "Não há soluções reais com $\\omega > 0$ e $K > 0$ satisfazendo simultaneamente a parte real e imaginária. O sistema não cruza o eixo imaginário na faixa analisada.",
                                body,
                                t,
                                t,
                            );
                        }
                    },
                );
            },
        );
    }
    let p7 = &pap["passo_7"];
    if show(7) {
        deduction(
            ui,
            p,
            "Passo 7",
            "Ângulos de Partida (θd) e Chegada (θa)",
            |ui| {
                let departures = list(&p7["deducoes_partida"]);
                let arrivals = list(&p7["deducoes_chegada"]);
                if departures.is_empty() && arrivals.is_empty() {
                    rich(
                        ui,
                        f,
                        "Não há singularidades complexas conjugadas nesta função. Todas as singularidades são puramente reais, não se aplicando ângulos tangenciais de partida ou chegada.",
                        body,
                        m,
                        t,
                    );
                }
                let angles = |terms: &[Value], origin: &str, key: &str| -> String {
                    terms
                        .iter()
                        .map(|tz| {
                            format!(
                                "$\\angle({origin} - ({})) = {:.1}^\\circ$",
                                complex_tex(text(&tz[key])),
                                tz["angulo"].as_f64().unwrap_or(0.0)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                for (i, dp) in departures.iter().enumerate() {
                    ui.push_id(("dep", i), |ui| {
                        deduction_item(
                            ui,
                            p,
                            &format!(
                                "• Polo Complexo: p = {}",
                                complex_tex(text(&dp["polo_str"]))
                            ),
                            |ui| {
                                rich(ui, f, "Condição angular de partida:", body, t, t);
                                math_box(ui, p, f, text(&dp["formula_aplicada"]), em);
                                rich(
                                    ui,
                                    f,
                                    "Vetores angulares para cada zero $\\phi_z$:",
                                    body,
                                    t,
                                    t,
                                );
                                let zeros = angles(list(&dp["termos_zeros"]), "p", "zero_str");
                                rich(
                                    ui,
                                    f,
                                    if zeros.is_empty() {
                                        "Nenhum zero finito"
                                    } else {
                                        &zeros
                                    },
                                    12.8,
                                    t,
                                    t,
                                );
                                rich(
                                    ui,
                                    f,
                                    "Vetores angulares para os outros polos $\\theta_p$:",
                                    body,
                                    t,
                                    t,
                                );
                                rich(
                                    ui,
                                    f,
                                    &angles(list(&dp["termos_polos"]), "p", "polo_str"),
                                    12.8,
                                    t,
                                    t,
                                );
                                rich(ui, f, "Substituição vetorial:", body, t, t);
                                math_box(ui, p, f, text(&dp["calculo_substituicao"]), em);
                            },
                        );
                    });
                }
                for (i, dc) in arrivals.iter().enumerate() {
                    ui.push_id(("arr", i), |ui| {
                        deduction_item(
                            ui,
                            p,
                            &format!(
                                "• Zero Complexo: z = {}",
                                complex_tex(text(&dc["zero_str"]))
                            ),
                            |ui| {
                                rich(ui, f, "Condição angular de chegada:", body, t, t);
                                math_box(ui, p, f, text(&dc["formula_aplicada"]), em);
                                rich(
                                    ui,
                                    f,
                                    "Vetores angulares para cada polo $\\phi_p$:",
                                    body,
                                    t,
                                    t,
                                );
                                rich(
                                    ui,
                                    f,
                                    &angles(list(&dc["termos_polos"]), "z", "polo_str"),
                                    12.8,
                                    t,
                                    t,
                                );
                                rich(ui, f, "Substituição vetorial:", body, t, t);
                                math_box(ui, p, f, text(&dc["calculo_substituicao"]), em);
                            },
                        );
                    });
                }
            },
        );
    }
}

fn deduction(ui: &mut Ui, p: &Palette, badge: &str, title: &str, add: impl FnOnce(&mut Ui)) {
    ui.push_id(badge, |ui| {
        kit::frame(p.surface, p.border, 6, 0).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.set_width(ui.available_width());
            let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::hover());
            fill(
                ui,
                bar.shrink(1.0),
                egui::CornerRadius {
                    nw: 5,
                    ne: 5,
                    sw: 0,
                    se: 0,
                },
                p.inset,
                Stroke::NONE,
            );
            hline(ui, bar.x_range(), bar.bottom() - 1.0, p.border);
            let g = galley(ui, badge, mono(13.0, Weight::Medium), p.accent, 1.0);
            let rect = Rect::from_min_size(
                pos2(bar.left() + 16.0, bar.center().y - 12.0),
                vec2(g.size().x + 18.0, 24.0),
            );
            fill(
                ui,
                rect,
                3,
                Color32::TRANSPARENT,
                Stroke::new(1.0, p.accent.gamma_multiply(0.7)),
            );
            ui.painter().galley(
                pos2(rect.left() + 9.0, rect.center().y - g.size().y / 2.0),
                g,
                p.accent,
            );
            let tg = galley(ui, title, sans(14.6, Weight::SemiBold), p.text, 0.0);
            ui.painter().galley(
                pos2(rect.right() + 10.0, bar.center().y - tg.size().y / 2.0),
                tg,
                p.text,
            );
            egui::Frame::new().inner_margin(16).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 12.0;
                ui.set_width(ui.available_width());
                add(ui);
            });
        });
    });
    ui.add_space(14.0);
}

fn deduction_item(ui: &mut Ui, p: &Palette, title: &str, add: impl FnOnce(&mut Ui)) {
    kit::frame(p.surface, p.border, 6, 14).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.set_width(ui.available_width());
        kit::paragraph(ui, title, sans(13.0, Weight::Bold), p.text);
        add(ui);
    });
}

fn math_box(ui: &mut Ui, p: &Palette, f: &mut Formulas, tex: &str, em: f32) {
    kit::frame(p.surface, p.border, 4, egui::Margin::symmetric(4, 16)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        f.math_centered(ui, tex, em, p.text);
    });
}

enum Cell {
    Math(String),
    Text(String),
    Badge(bool, String, String),
}

fn deduction_table(
    ui: &mut Ui,
    p: &Palette,
    f: &mut Formulas,
    heads: [&str; 3],
    rows: &[[Cell; 3]],
    empty: &str,
) {
    let width = ui.available_width();
    let cols = [width * 0.35, width * 0.24, width * 0.41];
    let row_frame = |ui: &mut Ui, height: f32, head: bool| -> Rect {
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        if head {
            fill(ui, rect, 0, p.surface, Stroke::NONE);
        }
        hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border);
        let mut x = rect.left();
        for w in cols {
            kit::vline(ui, x, rect.y_range(), p.border);
            x += w;
        }
        kit::vline(ui, rect.right() - 1.0, rect.y_range(), p.border);
        rect
    };
    let cell_ui = |ui: &mut Ui, rect: Rect, i: usize, salt: (usize, usize)| -> Ui {
        let x = rect.left() + cols[..i].iter().sum::<f32>();
        let cell = Rect::from_min_size(pos2(x, rect.top()), vec2(cols[i], rect.height()));
        ui.new_child(
            egui::UiBuilder::new()
                .max_rect(cell.shrink2(vec2(12.0, 0.0)))
                .id_salt(salt)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        )
    };
    ui.spacing_mut().item_spacing.y = 0.0;
    let (top, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    hline(ui, top.x_range(), top.top(), p.border);
    let rect = row_frame(ui, 40.0, true);
    for (i, head) in heads.iter().enumerate() {
        let mut child = cell_ui(ui, rect, i, (usize::MAX, i));
        kit::text(&mut child, head, sans(13.0, Weight::SemiBold), p.text);
    }
    if rows.is_empty() {
        let (rect, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
        ui.painter().text(
            rect.left_center() + vec2(12.0, 0.0),
            egui::Align2::LEFT_CENTER,
            empty,
            sans(13.0, Weight::Regular),
            p.muted,
        );
        hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border);
    }
    for (r, row) in rows.iter().enumerate() {
        let tall = matches!(&row[2], Cell::Badge(_, _, why) if !why.is_empty());
        let rect = row_frame(ui, if tall { 64.0 } else { 43.0 }, false);
        for (i, cell) in row.iter().enumerate() {
            let mut child = cell_ui(ui, rect, i, (r, i));
            match cell {
                Cell::Math(tex) => {
                    f.inline(&mut child, tex, 13.0 * KATEX_SCALE, p.text);
                }
                Cell::Text(s) => {
                    child.add_space(((cols[i] - 24.0) / 2.0 - 4.0).max(0.0));
                    kit::text(&mut child, s, sans(13.0, Weight::Regular), p.text);
                }
                Cell::Badge(ok, label, why) => {
                    child.vertical(|ui| {
                        let tone = if *ok { p.stable } else { p.unstable };
                        tone_badge(ui, &tone, label, 12.0);
                        if !why.is_empty() {
                            kit::paragraph(ui, why, sans(11.8, Weight::Regular), p.text);
                        }
                    });
                }
            }
        }
    }
}

fn tone_badge(ui: &mut Ui, tone: &Tone, label: &str, size: f32) -> Response {
    let g = galley(ui, label, sans(size, Weight::SemiBold), tone.fg, 0.0);
    let (rect, response) = ui.allocate_exact_size(g.size() + vec2(14.0, 6.0), Sense::hover());
    fill(ui, rect, 3, tone.bg, Stroke::new(1.0, tone.border));
    ui.painter().galley(rect.min + vec2(7.0, 3.0), g, tone.fg);
    response
}

// ---------------------------------------------------------------------------
// Routh-Hurwitz

fn routh_card(ui: &mut Ui, p: &Palette, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.push_id(label, |ui| {
        kit::frame(p.surface, p.border, 4, 10).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            if !label.is_empty() {
                kit::spaced(ui, label, sans(13.6, Weight::Medium), p.muted, -0.1);
            }
            add(ui);
        });
    });
    ui.add_space(20.0);
}

fn routh(ui: &mut Ui, view: &View, f: &mut Formulas) {
    let p = &view.p;
    let r = view.result;
    if r.is_null() {
        return;
    }
    let verdict = text(&r["verdict"]);
    let tone = match verdict {
        "Estável" => p.stable,
        "Marginalmente Estável" => p.marginal,
        "Instável" => p.unstable,
        _ => p.dependent,
    };
    routh_card(ui, p, "", |ui| {
        kit::spaced(
            ui,
            "RESULTADO DA ANÁLISE",
            sans(12.5, Weight::Bold),
            p.muted,
            0.5,
        );
        ui.add_space(2.0);
        let g = galley(ui, verdict, sans(19.2, Weight::Bold), tone.fg, -0.2);
        let (rect, _) = ui.allocate_exact_size(g.size() + vec2(28.0, 14.0), Sense::hover());
        fill(ui, rect, 8, tone.bg, Stroke::new(1.0, tone.border));
        ui.painter().galley(rect.min + vec2(14.0, 7.0), g, tone.fg);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for badge in [
                format!("Grau {}", raw(&r["degree"])),
                format!("{} Trocas de Sinal", raw(&r["sign_changes"])),
                format!("{} Raízes no SPD", raw(&r["rhp_poles"])),
            ] {
                let g = galley(ui, &badge, mono(13.6, Weight::Medium), p.muted, 0.0);
                let (rect, _) = ui.allocate_exact_size(g.size() + vec2(12.0, 4.0), Sense::hover());
                fill(ui, rect, 2, p.header, Stroke::new(1.0, p.border_strong));
                ui.painter().galley(rect.min + vec2(6.0, 2.0), g, p.muted);
            }
        });
        ui.add_space(8.0);
        let summary = if p.light {
            Color32::from_rgb(51, 65, 85)
        } else {
            p.text
        };
        kit::paragraph(
            ui,
            text(&r["summary"]),
            sans(13.6, Weight::Regular),
            summary,
        );
    });
    routh_card(ui, p, "Equação Característica P(s) = 0:", |ui| {
        kit::frame(p.inset, p.border, 4, 6).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add_space(4.0);
            f.math_centered(ui, text(&r["char_poly_latex"]), 17.5 * KATEX_SCALE, p.text);
            ui.add_space(4.0);
        });
    });
    let k = &r["k_range"];
    if k["has_k"].as_bool() == Some(true) {
        routh_card(ui, p, "Regimes de Estabilidade por Ganho K:", |ui| {
            ui.add_space(2.0);
            let stable_latex = k["stable_latex"]
                .as_str()
                .unwrap_or(r"\text{Sem faixa estável}");
            let unstable_latex = k["unstable_latex"]
                .as_str()
                .unwrap_or(r"\text{Sem faixa instável}");
            let omegas: Vec<String> = list(&k["marginal_cases"])
                .iter()
                .map(|m| format!("${}$", text(&m["omega_latex"])))
                .collect();
            let regimes = [
                (
                    "Estável",
                    p.stable,
                    Color32::from_rgb(16, 185, 129),
                    stable_latex,
                    if stable_latex.contains("Sem") {
                        "Nenhum valor de K garante todos os polos estritamente no semiplano esquerdo.".to_owned()
                    } else {
                        "Todos os polos de malha fechada situam-se no SPE com amortecimento."
                            .to_owned()
                    },
                ),
                (
                    "Marginalmente Estável",
                    p.marginal,
                    Color32::from_rgb(245, 158, 11),
                    k["marginal_latex"]
                        .as_str()
                        .unwrap_or(r"\text{Nenhum ponto marginal}"),
                    if omegas.is_empty() {
                        "Não há cruzamento com o eixo jω para ganhos positivos.".to_owned()
                    } else {
                        format!(
                            "Oscilações sustentadas no eixo imaginário com {}",
                            omegas.join(", ")
                        )
                    },
                ),
                (
                    "Instável",
                    p.unstable,
                    Color32::from_rgb(239, 68, 68),
                    unstable_latex,
                    if unstable_latex.contains("Sem") {
                        "O sistema permanece estável para todos os valores admissíveis de K."
                            .to_owned()
                    } else {
                        "Polos no semiplano direito causam crescimento exponencial e instabilidade."
                            .to_owned()
                    },
                ),
            ];
            for (title, tone, rule, latex, desc) in regimes {
                ui.push_id(title, |ui| {
                    let bg = if p.light {
                        tone.bg
                    } else {
                        rule.gamma_multiply(0.06)
                    };
                    let inner = egui::Frame::new()
                        .fill(bg)
                        .stroke(Stroke::new(1.0, tone.border))
                        .corner_radius(8)
                        .inner_margin(egui::Margin {
                            left: 10,
                            right: 16,
                            top: 14,
                            bottom: 14,
                        })
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let g = galley(
                                ui,
                                &title.to_uppercase(),
                                sans(12.5, Weight::SemiBold),
                                tone.fg,
                                0.4,
                            );
                            let (rect, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), g.size().y + 6.0),
                                Sense::hover(),
                            );
                            fill(
                                ui,
                                rect,
                                6,
                                if p.light {
                                    tone.bg
                                } else {
                                    rule.gamma_multiply(0.18)
                                },
                                Stroke::new(1.0, tone.border),
                            );
                            ui.painter().galley(rect.min + vec2(8.0, 3.0), g, tone.fg);
                            ui.add_space(12.0);
                            f.math_centered(ui, latex, 16.7 * KATEX_SCALE, p.text);
                            ui.add_space(12.0);
                            rich(ui, f, &desc, 13.6, p.muted, p.text);
                        });
                    let rr = inner.response.rect;
                    fill(
                        ui,
                        Rect::from_min_max(rr.min, pos2(rr.left() + 3.0, rr.bottom())),
                        egui::CornerRadius {
                            nw: 8,
                            sw: 8,
                            ne: 0,
                            se: 0,
                        },
                        rule,
                        Stroke::NONE,
                    );
                });
                ui.add_space(6.0);
            }
        });
    }
    let rows = list(&r["routh_table"]);
    if !rows.is_empty() {
        routh_card(ui, p, "Tabela de Routh:", |ui| routh_table(ui, p, f, rows));
    }
    for (i, case) in list(&r["special_cases"]).iter().enumerate() {
        ui.push_id(("case", i), |ui| {
            let (bg, border, tag) = if p.light {
                (
                    Color32::from_rgb(255, 251, 235),
                    Color32::from_rgb(253, 230, 138),
                    Tone {
                        fg: Color32::from_rgb(180, 83, 9),
                        bg: Color32::from_rgb(254, 243, 199),
                        border: Color32::TRANSPARENT,
                    },
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(245, 158, 11, 20),
                    Color32::from_rgba_unmultiplied(245, 158, 11, 76),
                    Tone {
                        fg: Color32::from_rgb(251, 191, 36),
                        bg: Color32::from_rgba_unmultiplied(245, 158, 11, 46),
                        border: Color32::TRANSPARENT,
                    },
                )
            };
            kit::frame(bg, border, 8, egui::Margin::symmetric(16, 12)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                let g = galley(ui, "CASO ESPECIAL", sans(11.5, Weight::Bold), tag.fg, 0.4);
                let (rect, _) = ui.allocate_exact_size(g.size() + vec2(12.0, 4.0), Sense::hover());
                fill(ui, rect, 4, tag.bg, Stroke::NONE);
                ui.painter().galley(rect.min + vec2(6.0, 2.0), g, tag.fg);
                rich(ui, f, text(&case["desc"]), 14.0, p.text, p.text);
            });
        });
        ui.add_space(8.0);
    }
    let steps = list(&r["steps"]);
    if !steps.is_empty() {
        routh_card(ui, p, "Passo a Passo das Linhas:", |ui| {
            kit::frame(
                Color32::TRANSPARENT,
                p.border,
                6,
                egui::Margin::symmetric(12, 8),
            )
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 10.0;
                for (i, step) in steps.iter().enumerate() {
                    ui.push_id(i, |ui| {
                        kit::frame(p.inset, p.soft_border, 8, egui::Margin::symmetric(16, 12))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal_top(|ui| {
                                    f.inline(ui, text(&step["power"]), 13.6 * KATEX_SCALE, p.text);
                                    ui.add_space(10.0);
                                    ui.vertical(|ui| {
                                        rich(ui, f, text(&step["desc"]), 13.6, p.text, p.text)
                                    });
                                });
                                ui.add_space(8.0);
                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                                    for (j, v) in list(&step["values"]).iter().enumerate() {
                                        ui.push_id(j, |ui| {
                                            kit::frame(
                                                p.surface,
                                                p.soft_border,
                                                6,
                                                egui::Margin::symmetric(10, 4),
                                            )
                                            .show(
                                                ui,
                                                |ui| {
                                                    f.inline(
                                                        ui,
                                                        text(v),
                                                        15.8 * KATEX_SCALE,
                                                        p.text,
                                                    );
                                                },
                                            );
                                        });
                                    }
                                });
                            });
                    });
                }
            });
        });
    }
    let roots = list(&r["exact_roots"]);
    if !roots.is_empty() {
        routh_card(ui, p, "Polos do Polinômio para Conferência:", |ui| {
            for (i, root) in roots.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    f.inline(
                        ui,
                        &format!("s_{{{}}}:", i + 1),
                        13.6 * KATEX_SCALE,
                        p.faint,
                    );
                    ui.add_space(4.0);
                    f.inline(ui, text(&root["str"]), 13.6 * KATEX_SCALE, p.text);
                    ui.add_space(4.0);
                    let re = root["real"].as_f64().unwrap_or(0.0);
                    let (tone, label) = if re > 0.0 {
                        (p.unstable, "SPD")
                    } else if re < 0.0 {
                        (p.stable, "SPE")
                    } else {
                        (p.marginal, "jω")
                    };
                    tone_badge(ui, &tone, label, 11.5);
                });
            }
        });
    }
}

fn routh_table(ui: &mut Ui, p: &Palette, f: &mut Formulas, rows: &[Value]) {
    let columns = list(&rows[0]["cells"]).len();
    let em = 14.7 * KATEX_SCALE;
    let cell_fg = if p.light {
        Color32::from_rgb(29, 78, 216)
    } else {
        p.accent
    };
    let head_font = sans(13.0, Weight::Bold);
    let mut widths = vec![
        galley(ui, "POTÊNCIA", head_font.clone(), p.text, 0.5)
            .size()
            .x
            + 28.0,
    ];
    for j in 0..columns {
        let head = galley(
            ui,
            &format!("COLUNA {}", j + 1),
            head_font.clone(),
            p.text,
            0.5,
        )
        .size()
        .x + 28.0;
        let widest = rows
            .iter()
            .filter_map(|row| list(&row["cells"]).get(j))
            .map(|c| {
                f.size(&format!("\\textstyle {}", text(&c["latex"])), em)
                    .map_or(30.0, |s| s.x)
                    + 28.0
                    + if c["is_first_col"].as_bool() == Some(true) {
                        30.0
                    } else {
                        0.0
                    }
            })
            .fold(head, f32::max);
        widths.push(widest);
    }
    widths[0] = rows
        .iter()
        .map(|row| {
            f.size(&format!("\\textstyle {}", text(&row["power"])), em)
                .map_or(30.0, |s| s.x)
                + 28.0
        })
        .fold(widths[0], f32::max);
    let total: f32 = widths.iter().sum();
    egui::ScrollArea::horizontal()
        .id_salt("routh-table")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(total, 40.0), Sense::hover());
                fill(ui, rect, 0, p.table_head_bg, Stroke::NONE);
                let mut x = rect.left();
                for (j, w) in widths.iter().enumerate() {
                    let label = if j == 0 {
                        "POTÊNCIA".to_owned()
                    } else {
                        format!("COLUNA {j}")
                    };
                    let g = galley(ui, &label, head_font.clone(), p.table_head_fg, 0.5);
                    ui.painter().galley(
                        pos2(x + 14.0, rect.center().y - g.size().y / 2.0),
                        g,
                        p.table_head_fg,
                    );
                    x += w;
                    kit::vline(ui, x - 1.0, rect.y_range(), p.table_border);
                }
                hline(ui, rect.x_range(), rect.bottom() - 1.0, p.table_border);
                for (i, row) in rows.iter().enumerate() {
                    let (rect, _) = ui.allocate_exact_size(vec2(total, 45.0), Sense::hover());
                    fill(ui, rect, 0, p.table_cell_bg, Stroke::NONE);
                    let mut cells: Vec<(String, Option<String>)> =
                        vec![(text(&row["power"]).to_owned(), None)];
                    for c in list(&row["cells"]) {
                        let sign = text(&c["sign"]);
                        let pill = (c["is_first_col"].as_bool() == Some(true)
                            && !sign.is_empty()
                            && sign != "expr")
                            .then(|| sign.to_owned());
                        cells.push((text(&c["latex"]).to_owned(), pill));
                    }
                    let mut x = rect.left();
                    for (j, (tex, pill)) in cells.iter().enumerate() {
                        let w = widths.get(j).copied().unwrap_or(60.0);
                        let cell = Rect::from_min_size(pos2(x, rect.top()), vec2(w, rect.height()));
                        let mut child = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(cell.shrink2(vec2(14.0, 0.0)))
                                .id_salt((i, j))
                                .layout(egui::Layout::left_to_right(egui::Align::Center)),
                        );
                        f.inline(&mut child, tex, em, cell_fg);
                        if let Some(sign) = pill {
                            child.add_space(6.0);
                            let tone = match sign.as_str() {
                                "+" => p.stable,
                                "-" => p.unstable,
                                _ => p.marginal,
                            };
                            let g = galley(&child, sign, mono(12.0, Weight::Bold), tone.fg, 0.0);
                            let (r, _) = child
                                .allocate_exact_size(vec2(g.size().x + 12.0, 24.0), Sense::hover());
                            fill(&child, r, 4, tone.bg, Stroke::new(1.0, tone.border));
                            child.painter().galley(
                                pos2(r.left() + 6.0, r.center().y - g.size().y / 2.0),
                                g,
                                tone.fg,
                            );
                        }
                        x += w;
                        kit::vline(ui, x - 1.0, rect.y_range(), p.border);
                    }
                    hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border);
                }
            });
        });
}

/// Placeholder of the figure area while a module has no result.
pub fn empty_message(id: &str) -> &'static [(&'static str, bool)] {
    match id {
        "routh" => &[
            (
                "Insira uma função de transferência ou polinômio e clique em ",
                false,
            ),
            ("Calcular", true),
            (".", false),
        ],
        "lgr" => &[
            ("Insira a função de transferência e clique em ", false),
            ("Calcular", true),
            (" para visualizar o gráfico e o memorial de cálculo.", false),
        ],
        _ => &[
            ("Aguardando cálculo. Clique em ", false),
            ("Calcular", true),
            (" para gerar a resposta.", false),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossings_list_each_conjugate_pair_once() {
        let crossings = [
            json!({"w":1.9035,"K":10.117}),
            json!({"w":-1.9035,"K":10.117}),
        ];
        assert_eq!(unique_crossings(&crossings).len(), 1);
    }

    #[test]
    fn complex_roots_follow_the_previous_latex_formatting() {
        assert_eq!(complex_tex("-2.00 +2.00j"), "-2.00 + j2.00");
        assert_eq!(complex_tex("-2.00 -2.00j"), "-2.00 - j2.00");
        assert_eq!(complex_tex("0.00"), "0.00");
    }
}
