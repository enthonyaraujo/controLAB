//! Scientific module pages (`#page-scientific`, `#page-lgr`, `#page-routh`).
use crate::icons::{self, Icon};
use crate::kit::{self, fill, galley, hline};
use crate::model::{Field, Module};
use crate::results::{self, KATEX_SCALE};
use crate::theme::{Palette, Weight, sans};
use crate::{Controlab, MEASURED_REM, Mode, PreviewState, ToastKind};
use eframe::egui::{self, Color32, Rect, Sense, Stroke, Ui, pos2, vec2};

/// `MODULE_METAS[*].solverMeta` and the LGR/Routh footers.
fn solver_meta(id: &str) -> &'static str {
    match id {
        "time" => "Módulo: Resposta no Tempo • Solver: scipy",
        "frequency" => "Módulo: Resposta em Frequência • Solver: scipy / python-control",
        "controllers" => "Módulo: Projeto de Controladores • Solver: LGR / sympy / python-control",
        "state-space" => "Módulo: Espaço de Estados • Solver: scipy / sympy",
        "lgr" => "Módulo: Lugar Geométrico das Raízes • Solver: python-control",
        "routh" => "Módulo: Estabilidade • Solver: sympy",
        _ => "Pronto",
    }
}

fn loading_text(id: &str) -> &'static str {
    match id {
        "time" => "Calculando Resposta no Tempo...",
        "frequency" => "Calculando Resposta em Frequência (Bode)...",
        "controllers" => "Projetando controlador em série via LGR...",
        "state-space" => "Calculando Espaço de Estados...",
        "lgr" => "Calculando Lugar Geométrico das Raízes...",
        _ => "Calculando estabilidade...",
    }
}

const CONTROLLER_FORMULAS: [(&str, &str); 6] = [
    ("lead", r"G_c(s) = \frac{K(s+z)}{s+p}, \quad z < p"),
    ("lag", r"G_c(s) = \frac{K(s+z)}{s+p}, \quad p < z"),
    ("P", r"G_c(s) = K_p, \quad u(t) = K_p e(t)"),
    (
        "PD",
        r"G_c(s) = K_p(1 + \tau_d s), \quad z = \frac{1}{\tau_d}",
    ),
    (
        "PI",
        r"G_c(s) = K_p\frac{s + 1/\tau_i}{s} = \frac{K_p(s+z)}{s}, \quad z = \frac{1}{\tau_i}",
    ),
    (
        "PID",
        r"G_c(s) = K_p + \frac{K_i}{s} + K_d s = \frac{K(s+z_i)(s+z_d)}{s}",
    ),
];

/// `formatExpressionToLatex`: instant preview while the engine validates.
pub fn instant_tex(raw: &str, prefix: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return format!("{prefix} = 0");
    }
    let tex = raw
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('*', " ");
    let tex = superscripts(&tex);
    match tex.split_once('/') {
        Some((num, den)) => {
            let unwrap = |s: &str| -> String {
                let s = s.trim();
                if s.starts_with('(') && s.ends_with(')') {
                    s[1..s.len() - 1].trim().to_owned()
                } else {
                    s.to_owned()
                }
            };
            format!("{prefix} = \\frac{{{}}}{{{}}}", unwrap(num), unwrap(den))
        }
        None => format!("{prefix} = {tex}"),
    }
}

/// `^2` → `^{2}` and `s2` → `s^{2}`, as the regex replacements of the previous preview.
fn superscripts(tex: &str) -> String {
    let chars: Vec<char> = tex.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if (c == '^' || c == 's') && chars.get(i + 1).is_some_and(char::is_ascii_digit) {
            let end = (i + 1..chars.len())
                .find(|&j| !chars[j].is_ascii_digit())
                .unwrap_or(chars.len());
            let digits: String = chars[i + 1..end].iter().collect();
            if c == 's' {
                out.push('s');
            }
            out.push_str(&format!("^{{{digits}}}"));
            i = end;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Routh `updatePreview`: polynomial or rational function, typeset locally.
fn routh_tex(raw: &str) -> String {
    let tex = superscripts(&raw.replace('*', " "));
    match tex.split_once('/') {
        Some((num, den)) => format!("G(s) = \\frac{{{}}}{{{}}}", num.trim(), den.trim()),
        None => format!("P(s) = {tex} = 0"),
    }
}

/// `recomputeControllerTransients`: keeps Mp↔ξ and ts↔ωn consistent.
pub fn recompute_controller(
    values: &mut std::collections::BTreeMap<String, String>,
    changed: &str,
) {
    let num = |v: &std::collections::BTreeMap<String, String>, k: &str| {
        v.get(k)
            .and_then(|s| s.trim().replace(',', ".").parse::<f64>().ok())
            .filter(|n| n.is_finite())
    };
    let fmt = |n: f64, digits: i32| {
        let scale = 10f64.powi(digits);
        let r = (n * scale).round() / scale;
        format!("{r}").replace('.', ",")
    };
    let crit = if values.get("settling_criterion").map(String::as_str) == Some("5") {
        3.0
    } else {
        4.0
    };
    match changed {
        "mp" => {
            values.insert("primary_damping_spec".into(), "mp".into());
            if let Some(mp) = num(values, "mp") {
                if mp > 0.0 && mp < 100.0 {
                    let ln = (mp / 100.0).ln();
                    values.insert(
                        "xi".into(),
                        fmt(-ln / (std::f64::consts::PI.powi(2) + ln * ln).sqrt(), 4),
                    );
                } else if mp == 0.0 {
                    values.insert("xi".into(), "1".into());
                }
            }
        }
        "xi" => {
            values.insert("primary_damping_spec".into(), "xi".into());
            if let Some(xi) = num(values, "xi") {
                if xi > 0.0 && xi < 1.0 {
                    values.insert(
                        "mp".into(),
                        fmt(
                            (-xi * std::f64::consts::PI / (1.0 - xi * xi).sqrt()).exp() * 100.0,
                            3,
                        ),
                    );
                } else if xi >= 1.0 {
                    values.insert("mp".into(), "0".into());
                }
            }
        }
        _ => {}
    }
    let xi = num(values, "xi").unwrap_or(0.0);
    match changed {
        "ts" => {
            values.insert("primary_freq_spec".into(), "ts".into());
            if let Some(ts) = num(values, "ts").filter(|t| *t > 0.0)
                && xi > 0.0
            {
                values.insert("wn".into(), fmt(crit / (xi * ts), 4));
            }
        }
        "wn" => {
            values.insert("primary_freq_spec".into(), "wn".into());
            if let Some(wn) = num(values, "wn").filter(|w| *w > 0.0)
                && xi > 0.0
            {
                values.insert("ts".into(), fmt(crit / (xi * wn), 4));
            }
        }
        "mp" | "xi" | "settling_criterion" => {
            if values.get("primary_freq_spec").map(String::as_str) == Some("wn") {
                if let Some(wn) = num(values, "wn").filter(|w| *w > 0.0)
                    && xi > 0.0
                {
                    values.insert("ts".into(), fmt(crit / (xi * wn), 4));
                }
            } else if let Some(ts) = num(values, "ts").filter(|t| *t > 0.0)
                && xi > 0.0
            {
                values.insert("wn".into(), fmt(crit / (xi * ts), 4));
            }
        }
        _ => {}
    }
}

enum Action {
    Calculate,
    Clear,
    ZoomIn,
    ZoomOut,
    Fit,
    Grid,
    Export(&'static str),
    Code,
    Copy,
}

impl Controlab {
    pub(crate) fn module_page(&mut self, ui: &mut Ui) {
        let p = self.palette();
        let id = self.page.clone();
        let module = self.schema[&id].clone();
        let mode = self.layout.mode;
        let rem = MEASURED_REM;
        let mut action = None;
        let frame = egui::Frame::NONE.fill(p.surface);
        let tab = self.analyses[&id].mobile_tab;
        let mut chosen = None;
        if mode == Mode::Phone {
            // `.sci-mobile-tabs` on top, `.sci-mobile-calc-bar` above the bottom navigation.
            let pending = self.analyses[&id].pending;
            if !self.layout.keyboard_open {
                egui::Panel::bottom("sci-mobile-calc")
                    .exact_size(4.0 * rem + 1.0)
                    .frame(egui::Frame::NONE)
                    .show(ui, |ui| {
                        let icon = id != "lgr" && id != "routh";
                        if kit::mobile_calc(ui, &p, icon, !pending).clicked() {
                            action = Some(Action::Calculate);
                        }
                    });
            }
            egui::Panel::top("sci-mobile-tabs")
                .exact_size(2.75 * rem)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    let labels = ["Parâmetros", "Gráfico", "Resultados"];
                    chosen = kit::segment_tabs(ui, &p, &labels, tab, 0.8125, p.surface);
                });
            egui::CentralPanel::default()
                .frame(frame)
                .show(ui, |ui| match tab {
                    0 => self.parameters(ui, &p, &id, &module),
                    1 => self.plot_panel(ui, &p, &id, false),
                    _ => {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        kit::panel_header(ui, &p, "Resultados", false, |_| {});
                        self.results_area(ui, &p, &id, &module, false);
                    }
                });
        } else {
            let tablet = mode == Mode::Tablet;
            egui::Panel::top("sci-ribbon")
                .exact_size(if tablet { 3.47 * rem } else { 61.0 })
                .frame(egui::Frame::NONE.fill(p.header))
                .show(ui, |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt("sci-ribbon-scroll")
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            ui.allocate_ui_with_layout(
                                vec2(
                                    ui.available_width().max(650.0),
                                    if tablet { 3.47 * rem } else { 61.0 },
                                ),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| action = self.ribbon(ui, &p, &id, tablet),
                            );
                        });
                });
            egui::Panel::bottom("sci-status")
                .exact_size(27.0)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    let left = match (id.as_str(), self.analyses[&id].pending) {
                        ("routh", true) => "Calculando...",
                        ("routh", false) => "Pronto",
                        _ => "",
                    };
                    kit::status_bar(ui, &p, left, solver_meta(&id));
                });
            if tablet {
                egui::Panel::left("sci-rail")
                    .exact_size(3.625 * rem)
                    .frame(egui::Frame::NONE)
                    .show(ui, |ui| {
                        if let Some(page) = kit::rail(ui, &p, &id, 1.0) {
                            self.page = page.into();
                        }
                    });
            }
            let (default, range) = if tablet {
                (16.5 * rem, 13.5 * rem..=18.5 * rem)
            } else {
                (342.0, 260.0..=520.0)
            };
            egui::Panel::left(if tablet {
                "sci-params-tablet"
            } else {
                "sci-params"
            })
            .default_size(default)
            .size_range(range)
            .resizable(!tablet)
            .frame(frame)
            .show(ui, |ui| self.parameters(ui, &p, &id, &module));
            if tablet {
                // `.sci-tablet-switcher`: the results live in a tab of the figure panel.
                egui::CentralPanel::default().frame(frame).show(ui, |ui| {
                    let (bar, _) = ui.allocate_exact_size(
                        vec2(ui.available_width(), 2.75 * rem),
                        Sense::hover(),
                    );
                    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(bar));
                    let active = usize::from(tab == 2);
                    if let Some(i) = kit::segment_tabs(
                        &mut child,
                        &p,
                        &["Figura", "Resultados"],
                        active,
                        0.75,
                        p.header,
                    ) {
                        chosen = Some(i + 1);
                    }
                    self.plot_panel(ui, &p, &id, tab == 2);
                });
            } else {
                egui::Panel::right("sci-results")
                    .default_size(480.0)
                    .size_range(280.0..=760.0)
                    .resizable(true)
                    .frame(frame)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        kit::panel_header(ui, &p, "Resultados", false, |_| {});
                        self.results_area(ui, &p, &id, &module, false);
                    });
                egui::CentralPanel::default()
                    .frame(frame)
                    .show(ui, |ui| self.plot_panel(ui, &p, &id, false));
            }
        }
        if let Some(i) = chosen {
            self.analyses.get_mut(&id).unwrap().mobile_tab = i;
        }
        if let Some(action) = action {
            self.ribbon_action(ui.ctx(), &id, action);
        }
    }

    /// Scrollable `.sci-panel-body` of the results; `tablet` stacks the formulas first.
    fn results_area(&mut self, ui: &mut Ui, p: &Palette, id: &str, module: &Module, tablet: bool) {
        egui::ScrollArea::vertical()
            .id_salt(("results", id))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 8,
                        top: 6,
                        bottom: 10,
                    })
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if tablet && (id == "lgr" || id == "routh") {
                            kit::panel_header(ui, p, "Resultados", false, |_| {});
                            ui.add_space(10.0);
                        }
                        let view = results::View {
                            id,
                            module,
                            result: &self.analyses[id].result,
                            p: *p,
                            tablet,
                        };
                        if tablet && (id == "lgr" || id == "routh") {
                            egui::Frame::new()
                                .inner_margin(egui::Margin::symmetric(10, 0))
                                .show(ui, |ui| {
                                    results::results_panel(ui, &view, &mut self.formulas)
                                });
                        } else {
                            results::results_panel(ui, &view, &mut self.formulas);
                        }
                    });
            });
    }

    fn ribbon(&mut self, ui: &mut Ui, p: &Palette, id: &str, compact: bool) -> Option<Action> {
        let rect = ui.max_rect();
        hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border_strong);
        // Tablet: `padding: .125rem .25rem`.
        let pad = if compact {
            vec2(0.25, 0.125) * MEASURED_REM
        } else {
            vec2(8.3, 4.2)
        };
        let inner = Rect::from_min_max(rect.min + pad, rect.max - vec2(pad.x, pad.y + 1.0));
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let ui = &mut child;
        ui.spacing_mut().item_spacing.x = 2.1;
        let a = &self.analyses[id];
        let zoomed = a.zoom > 1.0;
        let pending = a.pending;
        let has_result = !a.result.is_null();
        let mut action = None;
        let mut button =
            |ui: &mut Ui, icon: Icon, label: &str, primary: bool, enabled: bool, act: Action| {
                if kit::sci_button(ui, p, icon, label, primary, enabled).clicked() {
                    action = Some(act);
                }
            };
        let thin = id == "lgr" || id == "routh";
        button(
            ui,
            icons::CALC,
            "Calcular",
            true,
            !pending,
            Action::Calculate,
        );
        button(
            ui,
            if thin { icons::X_THIN } else { icons::ERASER },
            "Limpar",
            false,
            true,
            Action::Clear,
        );
        kit::ribbon_separator(ui, p);
        button(
            ui,
            if thin {
                icons::ZOOM_IN_THIN
            } else {
                icons::ZOOM_IN
            },
            "Zoom +",
            false,
            true,
            Action::ZoomIn,
        );
        button(
            ui,
            if thin {
                icons::ZOOM_OUT_THIN
            } else {
                icons::ZOOM_OUT
            },
            "Zoom −",
            false,
            zoomed || !thin,
            Action::ZoomOut,
        );
        button(
            ui,
            if thin { icons::RESET } else { icons::FIT },
            "Ajustar",
            false,
            true,
            Action::Fit,
        );
        kit::ribbon_separator(ui, p);
        match id {
            "routh" => {
                button(
                    ui,
                    icons::DOWNLOAD_THIN,
                    "PNG",
                    false,
                    true,
                    Action::Export("png"),
                );
                button(
                    ui,
                    icons::CODE_THIN,
                    "SVG",
                    false,
                    true,
                    Action::Export("svg"),
                );
                button(ui, icons::COPY, "Copiar", false, true, Action::Copy);
            }
            "lgr" => {
                button(
                    ui,
                    icons::CODE_THIN,
                    "SVG",
                    false,
                    true,
                    Action::Export("svg"),
                );
                button(
                    ui,
                    icons::DOWNLOAD_THIN,
                    "PNG",
                    false,
                    true,
                    Action::Export("png"),
                );
                button(ui, icons::COPY, "Copiar", false, true, Action::Copy);
            }
            _ => {
                button(ui, icons::GRID, "Grade", false, true, Action::Grid);
                button(
                    ui,
                    icons::EXPORT,
                    "Exportar",
                    false,
                    true,
                    Action::Export("png"),
                );
                button(ui, icons::CODE, "Código", false, true, Action::Code);
            }
        }
        let _ = has_result;
        action
    }

    fn ribbon_action(&mut self, ctx: &egui::Context, id: &str, action: Action) {
        let field = self.expression_field(id);
        match action {
            Action::Calculate => {
                let expr = self.analyses[id]
                    .values
                    .get(field)
                    .map(|v| v.trim().is_empty())
                    .unwrap_or(false);
                if id == "routh" && expr {
                    self.toast(
                        "Digite uma função de transferência ou equação característica.",
                        ToastKind::Info,
                    );
                } else {
                    let width = self.plot_width;
                    self.calculate(id, width);
                }
            }
            Action::Clear => {
                let a = self.analyses.get_mut(id).unwrap();
                a.values.insert(field.into(), String::new());
                a.preview.state = PreviewState::Waiting;
                a.preview.tex.clear();
                a.preview.due = None;
                a.zoom = 1.0;
                a.pan = egui::Vec2::ZERO;
            }
            Action::ZoomIn => {
                let a = self.analyses.get_mut(id).unwrap();
                a.zoom = (a.zoom * 1.25).min(5.0);
            }
            Action::ZoomOut => {
                let a = self.analyses.get_mut(id).unwrap();
                a.zoom = (a.zoom / 1.25).max(1.0);
                if a.zoom <= 1.0 {
                    a.pan = egui::Vec2::ZERO;
                }
            }
            Action::Fit => {
                let a = self.analyses.get_mut(id).unwrap();
                a.zoom = 1.0;
                a.pan = egui::Vec2::ZERO;
            }
            Action::Grid => self.toast("Grade ativada na figura.", ToastKind::Info),
            Action::Export(format) => self.export_figure(id, format),
            Action::Copy => self.copy_figure(ctx, id),
            Action::Code => {
                let expr = self.analyses[id]
                    .values
                    .get(field)
                    .map(|v| v.trim().to_owned())
                    .filter(|v| !v.is_empty())
                    .unwrap_or_else(|| "4 / (s^2 + 2*s + 4)".into());
                let code = format!(
                    concat!(
                        "# ControLAB v",
                        env!("CARGO_PKG_VERSION"),
                        " - Script de Simulação\nimport control as ct\nimport matplotlib.pyplot as plt\n\ns = ct.tf('s')\nG = {}\nT = ct.feedback(G, 1)\nt, y = ct.step_response(T, T=10)\n\nplt.figure(figsize=(8, 4.5), dpi=120)\nplt.plot(t, y, label='Degrau unitário')\nplt.grid(True)\nplt.title('Resposta ao Degrau - ControLAB')\nplt.xlabel('Tempo (s)')\nplt.ylabel('Amplitude')\nplt.legend()\nplt.show()\n"
                    ),
                    expr.replace('^', "**")
                );
                ctx.copy_text(code);
                self.toast(
                    "Código Python copiado para a Área de Transferência!",
                    ToastKind::Success,
                );
            }
        }
    }

    fn parameters(&mut self, ui: &mut Ui, p: &Palette, id: &str, module: &Module) {
        ui.spacing_mut().item_spacing.y = 0.0;
        kit::panel_header(ui, p, "Parâmetros", false, |_| {});
        if self.layout.mode == Mode::Phone && self.layout.keyboard_open {
            egui::Panel::bottom(egui::Id::new(("pinned-math", id)))
                .exact_size(3.5 * MEASURED_REM)
                .frame(egui::Frame::NONE.fill(p.surface))
                .show(ui, |ui| self.expression_keys(ui, p, id));
        }
        egui::ScrollArea::vertical()
            .id_salt(("params", id))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new().inner_margin(10.4).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    match id {
                        "lgr" => self.lgr_form(ui, p, module),
                        "routh" => self.routh_form(ui, p, module),
                        _ => self.schema_form(ui, p, id, module),
                    }
                    if self.layout.mode == Mode::Phone
                        && !self.layout.keyboard_open
                        && id != "lgr"
                        && id != "routh"
                    {
                        self.expression_keys(ui, p, id);
                    }
                    ui.add_space(if self.layout.mode == Mode::Desktop {
                        0.0
                    } else {
                        20.0
                    });
                });
            });
    }

    fn expression_keys(&mut self, ui: &mut Ui, p: &Palette, id: &str) {
        if self.layout.mode != Mode::Phone {
            return;
        }
        let keys = if id == "routh" {
            &["^", "*", "/", "(", ")", "+", "-", "s", "K"][..]
        } else {
            &["^", "*", "/", "(", ")", "+", "-", "s"][..]
        };
        if let Some(key) = kit::virtual_keys(ui, p, keys) {
            let field = self.expression_field(id);
            let value = self
                .analyses
                .get_mut(id)
                .unwrap()
                .values
                .entry(field.into())
                .or_default();
            kit::insert_at_cursor(ui.ctx(), egui::Id::new(("expression", id)), value, key);
            self.expression_changed(id, 0.3);
        }
    }

    fn expression_changed(&mut self, id: &str, delay: f64) {
        let field = self.expression_field(id);
        let now = self.now;
        let a = self.analyses.get_mut(id).unwrap();
        let raw = a
            .values
            .get(field)
            .map(|v| v.trim().to_owned())
            .unwrap_or_default();
        if id != "lgr" {
            a.preview.tex = instant_tex(&raw, "G(s)");
        }
        a.preview.error.clear();
        a.preview.state = if raw.is_empty() {
            PreviewState::Waiting
        } else if raw == a.preview.valid_for {
            PreviewState::Valid
        } else {
            PreviewState::Validating
        };
        a.preview.due = (!raw.is_empty()).then_some(now + delay);
    }

    fn schema_form(&mut self, ui: &mut Ui, p: &Palette, id: &str, module: &Module) {
        let mut changed: Vec<String> = Vec::new();
        for field in &module.fields {
            let values = &self.analyses[id].values;
            if field.kind == "hidden" || !field.visible(values) {
                continue;
            }
            ui.push_id(&field.name, |ui| {
                kit::form_label(ui, p, &field.label, &field.help);
                ui.add_space(2.0);
                if self.field_control(ui, p, id, field) {
                    changed.push(field.name.clone());
                }
            });
            if field.name == "expr" || field.name == "plant_expr" {
                ui.add_space(16.0);
                self.preview_box(ui, p, id, "Prévia", true);
            }
            if field.name == "controller_type" {
                ui.add_space(6.0);
                let kind = self.analyses[id]
                    .values
                    .get("controller_type")
                    .cloned()
                    .unwrap_or_default();
                let tex = CONTROLLER_FORMULAS
                    .iter()
                    .find(|(k, _)| *k == kind)
                    .map_or(CONTROLLER_FORMULAS[0].1, |(_, t)| *t);
                formula_card(ui, p, &mut self.formulas, tex);
            }
            if field.name == "wn" {
                ui.add_space(6.0);
                let values = self.analyses[id].values.clone();
                dominant_pole(ui, p, &mut self.formulas, &values);
            }
            ui.add_space(16.0);
        }
        for name in changed {
            if name == "expr" || name == "plant_expr" {
                self.expression_changed(id, 0.3);
            }
            if id == "controllers"
                && ["mp", "xi", "ts", "wn", "settling_criterion"].contains(&name.as_str())
            {
                recompute_controller(&mut self.analyses.get_mut(id).unwrap().values, &name);
            }
        }
    }

    /// Input, textarea, select or segmented control; returns true when edited.
    fn field_control(&mut self, ui: &mut Ui, p: &Palette, id: &str, field: &Field) -> bool {
        let value = self
            .analyses
            .get_mut(id)
            .unwrap()
            .values
            .entry(field.name.clone())
            .or_default();
        match field.kind.as_str() {
            "select" => {
                let short = field.options.len() <= 3
                    && field.options.iter().all(|o| o[1].chars().count() <= 15);
                if short {
                    kit::segmented(ui, p, &field.options, value)
                } else {
                    kit::select(ui, p, ui.id().with("select"), &field.options, value)
                }
            }
            "textarea" => kit::textarea(ui, p, value, 1).changed(),
            _ if field.name == "expr" || field.name == "plant_expr" => kit::input_id(
                ui,
                p,
                value,
                &field.placeholder,
                true,
                Some(egui::Id::new(("expression", id))),
            )
            .changed(),
            _ => kit::input(ui, p, value, &field.placeholder, true).changed(),
        }
    }

    fn preview_box(&mut self, ui: &mut Ui, p: &Palette, id: &str, title: &str, uppercase: bool) {
        let preview = &self.analyses[id].preview;
        let (status, color) = match (id, preview.state) {
            (_, PreviewState::Waiting) => ("Aguardando entrada", p.muted),
            (_, PreviewState::Validating) => ("Validando…", p.muted),
            (_, PreviewState::Invalid) => ("Revise a entrada", p.error),
            ("lgr" | "routh", PreviewState::Valid) => ("Expressão válida", p.valid),
            (_, PreviewState::Valid) => ("válida", p.valid),
        };
        let tex = if preview.tex.is_empty() {
            if id == "routh" {
                "P(s) = 0".to_owned()
            } else {
                "G(s)".to_owned()
            }
        } else {
            preview.tex.clone()
        };
        let error = preview.error.clone();
        let frame = kit::frame(p.inset, p.border_strong, 2, egui::Margin::symmetric(8, 6));
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(19.0);
            ui.horizontal(|ui| {
                ui.set_min_height(19.0);
                let (font, spacing, label) = if uppercase {
                    (sans(12.7, Weight::Bold), 0.5, title.to_uppercase())
                } else {
                    (sans(12.7, Weight::SemiBold), 0.0, title.to_owned())
                };
                kit::spaced(ui, &label, font, p.muted, spacing);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    kit::text(ui, status, sans(12.7, Weight::SemiBold), color);
                });
            });
            // `.preview-math`: 10 px padding plus the KaTeX display margins.
            ui.add_space(16.0);
            self.formulas
                .math_centered(ui, &tex, 16.7 * KATEX_SCALE, p.text);
            ui.add_space(14.0);
            if !error.is_empty() {
                kit::paragraph(ui, &error, sans(12.5, Weight::Regular), p.error);
            }
        });
    }

    fn lgr_form(&mut self, ui: &mut Ui, p: &Palette, module: &Module) {
        let expr = &module.fields[1];
        kit::text(ui, &expr.label, sans(14.6, Weight::SemiBold), p.text);
        ui.add_space(6.0);
        let changed = {
            let value = self
                .analyses
                .get_mut("lgr")
                .unwrap()
                .values
                .entry("expr".into())
                .or_default();
            kit::input_id(
                ui,
                p,
                value,
                &expr.placeholder,
                true,
                Some(egui::Id::new(("expression", "lgr"))),
            )
            .changed()
        };
        if changed {
            self.expression_changed("lgr", 0.45);
        }
        ui.add_space(8.0);
        crate::results::rich(ui, &mut self.formulas, &expr.help, 13.6, p.muted, p.text);
        ui.add_space(16.0);
        if !self.layout.keyboard_open {
            self.expression_keys(ui, p, "lgr");
        }
        self.preview_box(ui, p, "lgr", "Prévia matemática", false);
        ui.add_space(16.0);
        let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        hline(ui, line.x_range(), line.top(), p.border);
        ui.add_space(6.0);
        let id = ui.id().with("lgr-options");
        let open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
        let (row, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
        let arrow = if open { "▼" } else { "▶" };
        let g = galley(
            ui,
            &format!("{arrow}  Opções do gráfico"),
            sans(13.6, Weight::Regular),
            p.muted,
            0.0,
        );
        ui.painter().galley(
            pos2(row.left(), row.center().y - g.size().y / 2.0),
            g,
            p.muted,
        );
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            ui.data_mut(|d| d.insert_temp(id, !open));
        }
        if open {
            ui.add_space(4.0);
            let value = self
                .analyses
                .get_mut("lgr")
                .unwrap()
                .values
                .entry("title".into())
                .or_default();
            kit::input(ui, p, value, "", false);
        }
    }

    fn routh_form(&mut self, ui: &mut Ui, p: &Palette, module: &Module) {
        let expr = &module.fields[0];
        kit::text(ui, &expr.label, sans(14.6, Weight::SemiBold), p.text);
        ui.add_space(6.0);
        let value = self
            .analyses
            .get_mut("routh")
            .unwrap()
            .values
            .entry("expr".into())
            .or_default();
        kit::input_id(
            ui,
            p,
            value,
            &expr.placeholder,
            true,
            Some(egui::Id::new(("expression", "routh"))),
        );
        let raw = value.trim().to_owned();
        let a = self.analyses.get_mut("routh").unwrap();
        a.preview.tex = if raw.is_empty() {
            "P(s) = 0".into()
        } else {
            routh_tex(&raw)
        };
        a.preview.state = if raw.is_empty() {
            PreviewState::Waiting
        } else {
            PreviewState::Valid
        };
        a.preview.due = None;
        ui.add_space(8.0);
        crate::results::rich(ui, &mut self.formulas, &expr.help, 13.6, p.muted, p.text);
        ui.add_space(16.0);
        if !self.layout.keyboard_open {
            self.expression_keys(ui, p, "routh");
        }
        self.preview_box(ui, p, "routh", "Prévia matemática", true);
    }

    fn plot_panel(&mut self, ui: &mut Ui, p: &Palette, id: &str, results_tab: bool) {
        if results_tab {
            if id != "lgr" && id != "routh" {
                kit::panel_header(ui, p, "Resultados", false, |_| {});
            }
            let module = self.schema[id].clone();
            self.results_area(ui, p, id, &module, true);
            return;
        }
        ui.spacing_mut().item_spacing.y = 0.0;
        self.plot_width = ui.available_width() - 8.4;
        let stacked = self.analyses[id].figures.len() > 1;
        let title = if stacked {
            "Figuras — Resposta no tempo"
        } else {
            results::figure_title(id)
        };
        let mut response_click = None;
        let response = self.analyses[id].response.clone();
        kit::panel_header(ui, p, title, false, |ui| {
            if id == "time" {
                response_click = kit::mini_segmented(
                    ui,
                    p,
                    &[
                        ("impulse", "Impulso"),
                        ("ramp", "Rampa"),
                        ("step", "Degrau"),
                    ]
                    .iter()
                    .rev()
                    .copied()
                    .collect::<Vec<_>>(),
                    &response,
                );
            }
        });
        if let Some(r) = response_click {
            let a = self.analyses.get_mut(id).unwrap();
            a.response = r.clone();
            ui.data_mut(|d| d.insert_temp(egui::Id::new("scroll-to-figure"), r));
        }
        if id == "lgr" {
            self.lgr_tabs(ui, p);
            let view = self.analyses["lgr"].view;
            if view != 0 {
                let rect = ui.available_rect_before_wrap();
                fill(ui, rect, 0, p.surface, Stroke::NONE);
                egui::ScrollArea::vertical()
                    .id_salt(("lgr-view", view))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::new().inner_margin(8).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            if view == 1 {
                                let module = self.schema["lgr"].clone();
                                let view = results::View {
                                    id: "lgr",
                                    module: &module,
                                    result: &self.analyses["lgr"].result,
                                    p: *p,
                                    tablet: false,
                                };
                                if results::lgr_memorial(ui, &view, &mut self.formulas) {
                                    self.analyses.get_mut("lgr").unwrap().deductions = true;
                                }
                            } else {
                                results::lgr_theory(ui, p);
                            }
                        });
                    });
                return;
            }
        }
        if results_tab {
            let module = self.schema[id].clone();
            self.results_area(ui, p, id, &module, true);
            return;
        }
        let viewport = ui.available_rect_before_wrap();
        ui.allocate_rect(viewport, Sense::hover());
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(viewport.shrink(4.2))
                .id_salt(("viewport", id)),
        );
        self.viewport(&mut child, p, id);
    }

    fn lgr_tabs(&mut self, ui: &mut Ui, p: &Palette) {
        egui::ScrollArea::horizontal()
            .id_salt("lgr-view-tabs")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| self.lgr_tabs_row(ui, p));
    }
    fn lgr_tabs_row(&mut self, ui: &mut Ui, p: &Palette) {
        let (bar, _) =
            ui.allocate_exact_size(vec2(ui.available_width().max(445.0), 49.0), Sense::hover());
        hline(ui, bar.x_range(), bar.bottom() - 1.0, p.border);
        let mut x = bar.left() + 8.0;
        let current = self.analyses["lgr"].view;
        for (i, label) in ["Gráfico", "Passo a passo (7 Passos)", "Regras Clássicas"]
            .iter()
            .enumerate()
        {
            let active = current == i;
            let g = galley(
                ui,
                label,
                sans(13.6, Weight::SemiBold),
                if active { p.text } else { p.muted },
                0.0,
            );
            let tab = Rect::from_min_size(
                pos2(x, bar.top() + 3.0),
                vec2(g.size().x + 16.0, bar.height() - 3.0),
            );
            let response = ui.interact(tab, ui.id().with(("view", i)), Sense::click());
            if active {
                let radius = egui::CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                };
                ui.painter().rect(
                    tab,
                    radius,
                    p.bg,
                    Stroke::new(1.0, p.border),
                    egui::StrokeKind::Inside,
                );
                fill(
                    ui,
                    Rect::from_min_max(
                        pos2(tab.left() + 1.0, tab.bottom() - 2.0),
                        pos2(tab.right() - 1.0, tab.bottom() + 1.0),
                    ),
                    0,
                    p.bg,
                    Stroke::NONE,
                );
                fill(
                    ui,
                    Rect::from_min_max(
                        tab.min + vec2(1.0, 0.0),
                        pos2(tab.right() - 1.0, tab.top() + 2.0),
                    ),
                    egui::CornerRadius {
                        nw: 6,
                        ne: 6,
                        sw: 0,
                        se: 0,
                    },
                    p.accent,
                    Stroke::NONE,
                );
            }
            let color = if active || response.hovered() {
                p.text
            } else {
                p.muted
            };
            ui.painter().galley(
                pos2(tab.left() + 8.0, tab.center().y - g.size().y / 2.0),
                g,
                color,
            );
            if response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                self.analyses.get_mut("lgr").unwrap().view = i;
            }
            x = tab.right() + 2.0;
        }
    }

    fn viewport(&mut self, ui: &mut Ui, p: &Palette, id: &str) {
        let rect = ui.max_rect();
        let a = &self.analyses[id];
        if a.pending {
            loading_card(ui, p, loading_text(id));
            return;
        }
        if !a.error.is_empty() && a.result.is_null() {
            let g = kit::wrapped(
                ui,
                &a.error,
                sans(13.6, Weight::Regular),
                p.error,
                rect.width() * 0.6,
            );
            ui.painter()
                .galley(rect.center() - g.size() / 2.0, g, p.error);
            return;
        }
        if a.figures.is_empty() {
            kit::empty_box(
                ui,
                p,
                results::empty_message(id),
                !(id == "routh" || id == "lgr"),
                if id == "routh" || id == "lgr" {
                    440.0
                } else {
                    600.0
                },
            );
            return;
        }
        if a.figures.len() > 1 {
            self.stacked_figures(ui, p, id);
            return;
        }
        let texture = a.figures[0].2.clone();
        let (zoom, pan) = (a.zoom, a.pan);
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let size = texture.size_vec2() / ui.ctx().pixels_per_point().max(1.0);
        // `.plot-img { max-width: 98%; max-height: 98%; object-fit: contain }`: never upscaled.
        let fit = (rect.width() * 0.98 / size.x)
            .min(rect.height() * 0.98 / size.y)
            .clamp(0.05, 1.0);
        let base = Rect::from_center_size(rect.center(), size * fit);
        let shown = Rect::from_center_size(base.center() + pan, base.size() * zoom);
        ui.painter().with_clip_rect(rect).image(
            texture.id(),
            shown,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        let a = self.analyses.get_mut(id).unwrap();
        if let Some(gesture) = ui.input(|i| i.multi_touch())
            && rect.contains(gesture.center_pos)
        {
            a.zoom = (a.zoom * gesture.zoom_delta).clamp(1.0, 6.0);
            if a.zoom > 1.0 {
                a.pan += gesture.translation_delta;
            } else {
                a.pan = egui::Vec2::ZERO;
            }
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                a.zoom = (a.zoom * if scroll > 0.0 { 1.15 } else { 0.85 }).clamp(1.0, 6.0);
                if a.zoom <= 1.0 {
                    a.pan = egui::Vec2::ZERO;
                }
            }
        }
        if a.zoom > 1.0 && response.dragged() {
            a.pan += response.drag_delta();
        }
        if response.double_clicked() {
            a.zoom = 1.0;
            a.pan = egui::Vec2::ZERO;
        }
        if a.zoom > 1.0 {
            response.on_hover_cursor(egui::CursorIcon::Grab);
        }
    }

    fn stacked_figures(&mut self, ui: &mut Ui, p: &Palette, id: &str) {
        let figures = self.analyses[id].figures.clone();
        let target: Option<String> =
            ui.data_mut(|d| d.remove_temp(egui::Id::new("scroll-to-figure")));
        egui::ScrollArea::vertical()
            .id_salt("stacked")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new().inner_margin(10).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    for (key, caption, texture) in &figures {
                        let response = kit::frame(
                            p.surface,
                            p.border_strong,
                            4,
                            egui::Margin::symmetric(20, 16),
                        )
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 12.0;
                            ui.set_width(ui.available_width());
                            let (bar, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 31.0),
                                Sense::hover(),
                            );
                            fill(ui, bar, 0, p.header, Stroke::NONE);
                            hline(ui, bar.x_range(), bar.bottom() - 1.0, p.border_strong);
                            let g = galley(ui, caption, sans(13.0, Weight::SemiBold), p.text, 0.0);
                            ui.painter().galley(
                                pos2(bar.left() + 10.0, bar.center().y - g.size().y / 2.0),
                                g,
                                p.text,
                            );
                            let size = texture.size_vec2() / ui.ctx().pixels_per_point().max(1.0);
                            let width = ui.available_width().min(size.x);
                            ui.vertical_centered(|ui| {
                                ui.add(
                                    egui::Image::new(texture)
                                        .fit_to_exact_size(vec2(width, width * size.y / size.x)),
                                );
                            });
                        });
                        if target.as_deref() == Some(key.as_str()) {
                            response.response.scroll_to_me(Some(egui::Align::TOP));
                        }
                    }
                });
            });
    }
}

fn loading_card(ui: &mut Ui, p: &Palette, text: &str) {
    let rect = ui.max_rect();
    let g = galley(ui, text, sans(13.6, Weight::Regular), p.muted, 0.0);
    let card = Rect::from_center_size(rect.center(), vec2(g.size().x + 80.0, 56.0));
    fill(ui, card, 4, p.header, Stroke::new(1.0, p.border));
    let center = pos2(card.left() + 30.0, card.center().y);
    let t = ui.input(|i| i.time) as f32;
    ui.painter()
        .circle_stroke(center, 9.0, Stroke::new(2.5, p.border_strong));
    let start = t * 6.0;
    let points: Vec<egui::Pos2> = (0..=16)
        .map(|i| {
            let a = start + i as f32 / 16.0 * 1.6;
            center + vec2(a.cos(), a.sin()) * 9.0
        })
        .collect();
    ui.painter()
        .add(egui::Shape::line(points, Stroke::new(2.5, p.accent)));
    ui.painter().galley(
        pos2(card.left() + 52.0, card.center().y - g.size().y / 2.0),
        g,
        p.muted,
    );
    ui.ctx().request_repaint();
}

fn formula_card(ui: &mut Ui, p: &Palette, f: &mut crate::results::Formulas, tex: &str) {
    let inner = egui::Frame::new()
        .fill(p.inset)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(3)
        .inner_margin(egui::Margin {
            left: 11,
            right: 8,
            top: 6,
            bottom: 6,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 2.0;
            kit::spaced(
                ui,
                "LEI DO CONTROLADOR EM SÉRIE:",
                sans(11.7, Weight::SemiBold),
                p.muted,
                0.4,
            );
            ui.add_space(16.0);
            f.math_centered(ui, tex, 14.2 * KATEX_SCALE, p.text);
            ui.add_space(16.0);
        });
    let r = inner.response.rect;
    fill(
        ui,
        Rect::from_min_max(r.min, pos2(r.left() + 3.0, r.bottom())),
        egui::CornerRadius {
            nw: 3,
            sw: 3,
            ne: 0,
            se: 0,
        },
        p.accent,
        Stroke::NONE,
    );
}

/// `updateDominantPolePreview`.
fn dominant_pole(
    ui: &mut Ui,
    p: &Palette,
    f: &mut crate::results::Formulas,
    values: &std::collections::BTreeMap<String, String>,
) {
    let num = |k: &str| {
        values
            .get(k)
            .and_then(|s| s.trim().replace(',', ".").parse::<f64>().ok())
    };
    let desired = values.get("desired_pole").map(|s| s.trim()).unwrap_or("");
    let inner = egui::Frame::new()
        .fill(p.pole_tint)
        .corner_radius(3)
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 2.0;
            kit::spaced(
                ui,
                "POLOS DOMINANTES DESEJADOS:",
                sans(11.7, Weight::SemiBold),
                p.accent,
                0.4,
            );
            if !desired.is_empty() {
                kit::text(
                    ui,
                    &format!("s = {desired} (polo informado diretamente)"),
                    sans(13.6, Weight::Regular),
                    p.text,
                );
            } else if let (Some(xi), Some(wn)) = (num("xi"), num("wn"))
                && xi > 0.0
                && xi < 1.0
                && wn > 0.0
            {
                let fmt =
                    |v: f64| format!("{}", (v * 10000.0).round() / 10000.0).replace('.', "{,}");
                let re = xi * wn;
                let im = wn * (1.0 - xi * xi).max(0.0).sqrt();
                f.inline(
                    ui,
                    &format!(
                        r"s = -\text{{Re}} \pm \text{{Im}}j = -{} \pm {}j",
                        fmt(re),
                        fmt(im)
                    ),
                    13.6 * KATEX_SCALE,
                    p.text,
                );
            } else {
                kit::text(
                    ui,
                    "s = -Re ± Im·j (preencha ξ, Mp, ts ou ωn)",
                    sans(13.6, Weight::Regular),
                    p.text,
                );
            }
        });
    // Dashed border, as `.ctrl-dominant-pole-badge`.
    let r = inner.response.rect;
    let stroke = Stroke::new(1.0, p.pole_border);
    for (a, b) in [
        (r.left_top(), r.right_top()),
        (r.right_top(), r.right_bottom()),
        (r.right_bottom(), r.left_bottom()),
        (r.left_bottom(), r.left_top()),
    ] {
        ui.painter()
            .extend(egui::Shape::dashed_line(&[a, b], stroke, 4.0, 3.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_preview_matches_the_previous_formatter() {
        assert_eq!(
            instant_tex("4 / (s^2 + 2*s + 4)", "G(s)"),
            r"G(s) = \frac{4}{s^{2} + 2 s + 4}"
        );
        assert_eq!(instant_tex("", "G(s)"), "G(s) = 0");
        assert_eq!(
            routh_tex("s^3 + 3*s^2 + 2*s + K"),
            "P(s) = s^{3} + 3 s^{2} + 2 s + K = 0"
        );
    }

    #[test]
    fn controller_specifications_stay_consistent() {
        let mut v: std::collections::BTreeMap<String, String> = [
            ("mp", "16"),
            ("xi", "0.5"),
            ("ts", "2"),
            ("wn", "4"),
            ("settling_criterion", "2"),
            ("primary_freq_spec", "ts"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        v.insert("mp".into(), "25".into());
        recompute_controller(&mut v, "mp");
        assert_eq!(v["xi"], "0,4037");
        assert_eq!(v["primary_damping_spec"], "mp");
        assert_eq!(v["wn"], "4,9542");
        v.insert("wn".into(), "5".into());
        recompute_controller(&mut v, "wn");
        assert_eq!(v["ts"], "1,9817");
        assert_eq!(v["primary_freq_spec"], "wn");
    }
}
