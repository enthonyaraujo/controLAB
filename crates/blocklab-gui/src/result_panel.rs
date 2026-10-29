//! Painel de resultado: T(s) desenhada como fração, modo simbólico ou
//! numérico, parâmetros, botões de copiar e a lista didática do Mason.

use std::collections::{BTreeMap, HashSet};

use blocklab_core::diagram::{Diagram, NodeId, NodeKind};
use blocklab_core::format::{Mode, pretty, to_text};
use blocklab_core::{MasonReport, SolveError};
use eframe::egui::{self, FontId, RichText, Sense, Stroke, vec2};

use crate::app::{App, Selection};
use crate::solver::Outcome;

#[derive(Default)]
pub struct ResultState {
    outcome: Option<Outcome>,
    /// Muda a cada resultado novo (invalida o cache de texto).
    version: u64,
    cache: Option<Cache>,
}

struct Cache {
    version: u64,
    mode: Mode,
    params: BTreeMap<String, String>,
    view: Rendered,
}

/// Tudo já formatado em texto (formatar a cada quadro seria desperdício).
struct Rendered {
    num: String,
    den: Option<String>,
    text: String,
    ascii: String,
    latex: String,
    note: Option<String>,
    paths: Vec<Item>,
    loops: Vec<Item>,
    nontouching: Vec<String>,
    delta: String,
    symbols: Vec<String>,
}

struct Item {
    label: String,
    route: String,
    cofactor: Option<String>,
    sfg_nodes: Vec<usize>,
}

impl ResultState {
    pub fn set(&mut self, outcome: Outcome) {
        self.outcome = Some(outcome);
        self.version += 1;
    }

    pub fn has_outcome(&self) -> bool {
        self.outcome.is_some()
    }

    /// Último resultado recebido do cálculo.
    #[cfg(test)]
    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    /// O estado do resultado numa linha (faixa recolhida do celular) e se
    /// é um erro.
    pub fn summary(&mut self, mode: Mode, d: &Diagram) -> (String, bool) {
        match &self.outcome {
            None => ("Aguardando o primeiro cálculo…".into(), false),
            Some(Err(SolveError::Invalid(_))) => {
                ("O diagrama ainda não pode ser resolvido".into(), true)
            }
            Some(Err(e)) => (e.to_string(), true),
            Some(Ok(_)) => match self.rendered(mode, d) {
                Some(v) => (format!("T(s) = {}", v.text), false),
                None => (String::new(), false),
            },
        }
    }

    /// Nós com erro de validação (contorno vermelho no canvas).
    pub fn error_nodes(&self) -> HashSet<NodeId> {
        match &self.outcome {
            Some(Err(SolveError::Invalid(errs))) => errs.iter().flat_map(|e| e.nodes()).collect(),
            _ => HashSet::new(),
        }
    }

    fn rendered(&mut self, mode: Mode, d: &Diagram) -> Option<&Rendered> {
        let Some(Ok(report)) = &self.outcome else {
            return None;
        };
        let fresh = self
            .cache
            .as_ref()
            .is_some_and(|c| c.version == self.version && c.mode == mode && c.params == d.params);
        if !fresh {
            self.cache = Some(Cache {
                version: self.version,
                mode,
                params: d.params.clone(),
                view: render(report, mode, d),
            });
        }
        self.cache.as_ref().map(|c| &c.view)
    }
}

fn render(r: &MasonReport, mode: Mode, d: &Diagram) -> Rendered {
    let mut note = None;
    let mut transfer = r.transfer.clone();
    let symbols: Vec<String> = r
        .sfg
        .edges
        .iter()
        .flat_map(|e| e.gain.symbols())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    if mode == Mode::Numeric {
        match d.numeric_params() {
            Ok(p) if p.is_empty() && !symbols.is_empty() => {
                note = Some(
                    "Preencha os parâmetros abaixo para ver os coeficientes numéricos.".into(),
                );
            }
            Ok(p) => match transfer.substitute(&p) {
                Ok(t) => transfer = t,
                Err(_) => note = Some("Com esses valores o denominador zera.".into()),
            },
            Err(e) => note = Some(e.to_string()),
        }
    }
    let pf = pretty(&transfer, mode);
    let item =
        |p: &blocklab_core::sfg::GainPath, name: String, closed: bool, dk: Option<String>| Item {
            label: format!("{name} = {}", to_text(&p.gain, Mode::Symbolic)),
            route: p.describe(&r.sfg, closed),
            cofactor: dk,
            sfg_nodes: p.nodes.clone(),
        };
    Rendered {
        num: pf.num.text(),
        den: pf.den.as_ref().map(|d| d.text()),
        text: pf.text(),
        ascii: pf.ascii(),
        latex: pf.latex(),
        note,
        paths: r
            .paths
            .iter()
            .enumerate()
            .map(|(k, p)| {
                let dk = format!("Δ{} = {}", k + 1, to_text(&r.cofactors[k], Mode::Symbolic));
                item(p, format!("P{}", k + 1), false, Some(dk))
            })
            .collect(),
        loops: r
            .loops
            .iter()
            .enumerate()
            .map(|(i, l)| item(l, format!("L{}", i + 1), true, None))
            .collect(),
        nontouching: r
            .nontouching
            .iter()
            .map(|c| {
                c.iter()
                    .map(|i| format!("L{}", i + 1))
                    .collect::<Vec<_>>()
                    .join("·")
            })
            .collect(),
        delta: to_text(&r.delta, Mode::Symbolic),
        symbols,
    }
}

/// Como entregar o LaTeX copiado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LatexWrap {
    /// `\[ T(s) = … \]`: documento LaTeX (Overleaf, .tex).
    Display,
    /// `$$ T(s) = … $$`: Markdown, Notion, Obsidian, Jupyter.
    Markdown,
    /// Só a fórmula, para quem já está em modo matemático.
    Bare,
}

impl LatexWrap {
    const ALL: [LatexWrap; 3] = [LatexWrap::Display, LatexWrap::Markdown, LatexWrap::Bare];

    fn label(self) -> &'static str {
        match self {
            LatexWrap::Display => "Para documento LaTeX/Overleaf   \\[ … \\]",
            LatexWrap::Markdown => "Para Markdown/Notion/Obsidian   $$ … $$",
            LatexWrap::Bare => "Só a fórmula (Word, editor de equação)",
        }
    }

    fn apply(self, latex: &str) -> String {
        match self {
            LatexWrap::Display => format!("\\[ T(s) = {latex} \\]"),
            LatexWrap::Markdown => format!("$$ T(s) = {latex} $$"),
            LatexWrap::Bare => latex.to_string(),
        }
    }
}

/// Fração desenhada: numerador, traço e denominador, centralizados.
fn fraction(ui: &mut egui::Ui, num: &str, den: Option<&str>, size: f32) {
    let font = FontId::proportional(size);
    let color = ui.visuals().strong_text_color();
    let gn = ui
        .painter()
        .layout_no_wrap(num.to_string(), font.clone(), color);
    let Some(den) = den else {
        let (rect, _) = ui.allocate_exact_size(gn.size(), Sense::hover());
        ui.painter().galley(rect.min, gn, color);
        return;
    };
    let gd = ui.painter().layout_no_wrap(den.to_string(), font, color);
    let w = gn.size().x.max(gd.size().x) + 12.0;
    let gap = 6.0;
    let h = gn.size().y + gd.size().y + 2.0 * gap;
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let mid = rect.top() + gn.size().y + gap;
    let p = ui.painter();
    p.galley(
        egui::pos2(rect.center().x - gn.size().x / 2.0, rect.top()),
        gn,
        color,
    );
    p.line_segment(
        [egui::pos2(rect.left(), mid), egui::pos2(rect.right(), mid)],
        Stroke::new(1.5, color),
    );
    p.galley(
        egui::pos2(rect.center().x - gd.size().x / 2.0, mid + gap),
        gd,
        color,
    );
}

/// Um caminho ou laço da lista do Mason. Devolve se o mouse está sobre ele
/// (em qualquer ponto da faixa, inclusive sobre os textos, que são
/// selecionáveis e por isso "cobririam" um teste de hover comum) e se foi
/// clicado/tocado; a faixa ganha fundo destacado se estiver sob o mouse ou
/// `pinned`.
fn mason_item(ui: &mut egui::Ui, pinned: bool, add: impl FnOnce(&mut egui::Ui)) -> (bool, bool) {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner = ui.vertical(|ui| {
        ui.add_space(3.0);
        add(ui);
        ui.add_space(3.0);
    });
    let r = inner.response.rect;
    let row = egui::Rect::from_min_max(
        egui::pos2(ui.max_rect().left() - 4.0, r.top()),
        egui::pos2(ui.max_rect().right(), r.bottom()),
    );
    let hovered = ui.rect_contains_pointer(row);
    let clicked = hovered && ui.input(|i| i.pointer.primary_clicked());
    if hovered || pinned {
        let t = crate::theme::tokens(ui.visuals());
        let fill = t.highlight_bg;
        ui.painter()
            .set(bg, egui::Shape::rect_filled(row, 4.0, fill));
    }
    (hovered, clicked)
}

/// Nós do diagrama a destacar para um caminho/laço do SFG: os que geram
/// cada sinal e as ramificações entre eles.
fn diagram_nodes(r: &MasonReport, d: &Diagram, sfg_nodes: &[usize]) -> Vec<NodeId> {
    let mut set: HashSet<NodeId> = sfg_nodes
        .iter()
        .filter_map(|&i| r.sfg.nodes[i].origin)
        .collect();
    // Ramificações: entram se o sinal vem de um nó destacado e segue para
    // outro. Duas passadas cobrem cadeias de ramificações.
    for _ in 0..2 {
        for n in &d.nodes {
            if !matches!(n.kind, NodeKind::Branch { .. }) || set.contains(&n.id) {
                continue;
            }
            let fed = d
                .wires
                .iter()
                .any(|w| w.to.node == n.id && set.contains(&w.from.node));
            let feeds = d
                .wires
                .iter()
                .any(|w| w.from.node == n.id && set.contains(&w.to.node));
            if fed && feeds {
                set.insert(n.id);
            }
        }
    }
    set.into_iter().collect()
}

impl App {
    pub fn result_ui(&mut self, ui: &mut egui::Ui) {
        // Texto um pouco maior que no resto da interface: é o que se lê.
        {
            use crate::theme::{medium, regular};
            let styles = &mut ui.style_mut().text_styles;
            styles.insert(egui::TextStyle::Body, regular(14.5));
            styles.insert(egui::TextStyle::Button, regular(14.0));
            styles.insert(egui::TextStyle::Small, regular(12.5));
            styles.insert(egui::TextStyle::Heading, medium(17.0));
            styles.insert(egui::TextStyle::Monospace, FontId::monospace(13.0));
        }
        if self.layout.is_touch() {
            // Arrastar o dedo sobre o texto rola o painel em vez de
            // selecionar.
            ui.style_mut().interaction.selectable_labels = false;
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("Y(s) / X(s)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.selectable_value(&mut self.mode, Mode::Numeric, "Numérico");
                ui.selectable_value(&mut self.mode, Mode::Symbolic, "Simbólico");
            });
        });
        ui.separator();

        if self.solver.is_running() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("calculando…");
                if ui.button("Cancelar").clicked() {
                    self.solver.cancel();
                    self.cancelled = true;
                }
            });
        } else if self.cancelled {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Cálculo cancelado.").color(ui.visuals().warn_fg_color));
                if ui.button("Recalcular").clicked() {
                    self.cancelled = false;
                    self.solver.invalidate();
                }
            });
        }

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| match &self.result.outcome {
                None => {
                    ui.label(RichText::new("Aguardando o primeiro cálculo…").weak());
                }
                Some(Err(SolveError::Invalid(errs))) => {
                    let errs = errs.clone();
                    self.errors_ui(ui, &errs);
                }
                Some(Err(e)) => {
                    ui.label(RichText::new(e.to_string()).color(ui.visuals().error_fg_color));
                }
                Some(Ok(_)) => self.report_ui(ui),
            });
    }

    /// Destaque de um item do Mason: enquanto o mouse está em cima, ou
    /// fixado por um clique/toque (outro toque solta). Sem mouse, o toque
    /// é o único jeito de destacar.
    fn mason_hover(
        &mut self,
        report: &MasonReport,
        label: &str,
        nodes: &[usize],
        hovered: bool,
        clicked: bool,
        pinned: bool,
    ) {
        if clicked {
            self.pinned = if pinned {
                None
            } else {
                Some(label.to_string())
            };
        }
        let pinned = self.pinned.as_deref() == Some(label);
        if hovered || pinned {
            self.highlight = diagram_nodes(report, &self.doc.diagram, nodes);
        }
    }

    fn errors_ui(&mut self, ui: &mut egui::Ui, errs: &[blocklab_core::diagram::DiagramError]) {
        ui.label(RichText::new("O diagrama ainda não pode ser resolvido:").strong());
        ui.add_space(4.0);
        for e in errs {
            let text = e.describe(&self.doc.diagram);
            let nodes = e.nodes();
            let color = ui.visuals().error_fg_color;
            if nodes.is_empty() {
                ui.label(RichText::new(format!("• {text}")).color(color));
            } else if ui
                .link(RichText::new(format!("• {text}")).color(color))
                .on_hover_text("Clique para selecionar no diagrama")
                .clicked()
            {
                self.sel = Selection {
                    nodes: nodes.into_iter().collect(),
                    wires: Default::default(),
                };
            }
        }
    }

    fn report_ui(&mut self, ui: &mut egui::Ui) {
        let mode = self.mode;
        let Some(Ok(report)) = self.result.outcome.clone() else {
            return;
        };
        let Some(view) = self.result.rendered(mode, &self.doc.diagram) else {
            return;
        };
        // Copiamos o que precisamos para soltar o empréstimo de `self`.
        let (num, den, text, ascii, latex, note) = (
            view.num.clone(),
            view.den.clone(),
            view.text.clone(),
            view.ascii.clone(),
            view.latex.clone(),
            view.note.clone(),
        );
        let paths: Vec<(String, String, Option<String>, Vec<usize>)> = view
            .paths
            .iter()
            .map(|i| {
                (
                    i.label.clone(),
                    i.route.clone(),
                    i.cofactor.clone(),
                    i.sfg_nodes.clone(),
                )
            })
            .collect();
        let loops: Vec<(String, String, Vec<usize>)> = view
            .loops
            .iter()
            .map(|i| (i.label.clone(), i.route.clone(), i.sfg_nodes.clone()))
            .collect();
        let (nontouching, delta, symbols) = (
            view.nontouching.clone(),
            view.delta.clone(),
            view.symbols.clone(),
        );

        ui.add_space(8.0);
        egui::ScrollArea::horizontal()
            .id_salt("fracao")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("T(s) =").size(20.0));
                    fraction(ui, &num, den.as_deref(), 20.0);
                });
            });
        if let Some(n) = note {
            ui.label(RichText::new(n).color(ui.visuals().warn_fg_color));
        }
        ui.add_space(8.0);
        let mut copied = None;
        ui.horizontal_wrapped(|ui| {
            ui.menu_button("Copiar LaTeX ▾", |ui| {
                for wrap in LatexWrap::ALL {
                    if ui.button(wrap.label()).clicked() {
                        ui.ctx().copy_text(wrap.apply(&latex));
                        copied = Some("LaTeX");
                    }
                }
            });
            if ui.button("Copiar texto").clicked() {
                ui.ctx().copy_text(text.clone());
                copied = Some("texto");
            }
            if ui
                .button("Copiar ASCII")
                .on_hover_text("Com * e -, para MATLAB/Python")
                .clicked()
            {
                ui.ctx().copy_text(ascii.clone());
                copied = Some("ASCII");
            }
        });
        if let Some(what) = copied {
            self.notice(ui.ctx(), format!("T(s) copiada em {what}"));
        }
        ui.add_space(4.0);
        // Código já com os delimitadores: \frac só funciona em modo
        // matemático, e colado "cru" num documento dá "Missing $ inserted".
        // Quebra só nos espaços, nunca no meio de um comando como `\left(`.
        egui::Frame::new()
            .fill(ui.visuals().code_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(4)
            .inner_margin(8)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.add(
                    egui::Label::new(RichText::new(LatexWrap::Display.apply(&latex)).monospace())
                        .wrap()
                        .selectable(true),
                );
            });
        ui.label(
            RichText::new(
                "Para o Word ou outro editor de equação, use Copiar LaTeX ▾ → só a fórmula.",
            )
            .weak()
            .small(),
        );

        ui.add_space(8.0);
        egui::CollapsingHeader::new(format!("Parâmetros ({})", symbols.len()))
            .default_open(true)
            .show(ui, |ui| {
                if symbols.is_empty() {
                    ui.label(RichText::new("Nenhum símbolo: o resultado já é numérico.").weak());
                    return;
                }
                ui.label(
                    RichText::new("Valores para o modo numérico (vazio = continua simbólico). Aceita 10, 0.5, 1/3.")
                        .weak()
                        .small(),
                );
                egui::Grid::new("params").num_columns(2).show(ui, |ui| {
                    for s in &symbols {
                        let label = ui.label(RichText::new(s).strong());
                        // Edita uma cópia e só grava se o usuário mudar algo:
                        // mostrar o campo não pode "sujar" o documento.
                        let params = &mut self.doc.diagram.params;
                        let mut value = params.get(s).cloned().unwrap_or_default();
                        let changed = ui
                            .add(egui::TextEdit::singleline(&mut value).desired_width(120.0))
                            .labelled_by(label.id)
                            .changed();
                        if changed {
                            if value.trim().is_empty() {
                                params.remove(s);
                            } else {
                                params.insert(s.clone(), value);
                            }
                        }
                        ui.end_row();
                    }
                });
                if let Err(e) = self.doc.diagram.numeric_params() {
                    ui.label(RichText::new(e.to_string()).color(ui.visuals().error_fg_color));
                }
            });

        egui::CollapsingHeader::new("Caminhos e laços (Mason)")
            .default_open(true)
            .show(ui, |ui| {
                let hint = if self.layout.is_touch() {
                    "T = Σ Pₖ·Δₖ / Δ   — toque num item para destacá-lo no diagrama"
                } else {
                    "T = Σ Pₖ·Δₖ / Δ   — passe o mouse para destacar (clique fixa)"
                };
                ui.label(RichText::new(hint).weak().small());
                ui.add_space(4.0);
                ui.label(RichText::new("Caminhos diretos").strong());
                for (label, route, dk, nodes) in &paths {
                    let pinned = self.pinned.as_ref() == Some(label);
                    let (hovered, clicked) = mason_item(ui, pinned, |ui| {
                        ui.label(RichText::new(label).strong());
                        ui.label(RichText::new(route).weak().small());
                        if let Some(dk) = dk {
                            ui.label(RichText::new(dk).small());
                        }
                    });
                    self.mason_hover(&report, label, nodes, hovered, clicked, pinned);
                }
                ui.add_space(4.0);
                ui.label(RichText::new(format!("Laços ({})", loops.len())).strong());
                if loops.is_empty() {
                    ui.label(RichText::new("nenhum").weak());
                }
                for (label, route, nodes) in &loops {
                    let pinned = self.pinned.as_ref() == Some(label);
                    let (hovered, clicked) = mason_item(ui, pinned, |ui| {
                        ui.label(RichText::new(label).strong());
                        ui.label(RichText::new(route).weak().small());
                    });
                    self.mason_hover(&report, label, nodes, hovered, clicked, pinned);
                }
                if !nontouching.is_empty() {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Laços que não se tocam").strong());
                    ui.label(RichText::new(nontouching.join(",  ")).strong());
                }
                ui.add_space(4.0);
                ui.label(RichText::new(format!("Δ = {delta}")).strong());
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::{Sign, examples};
    use blocklab_core::solve;

    #[test]
    fn modo_numerico_usa_os_parametros() {
        let mut d = examples::feedback_with("K/(s(s+1))", "1", Sign::Minus);
        let r = solve(&d).unwrap();
        let sym = render(&r, Mode::Symbolic, &d);
        assert_eq!(sym.text, "K / (s^2 + s + K)");

        d.params.insert("K".into(), "10".into());
        let num = render(&r, Mode::Numeric, &d);
        assert_eq!(
            (num.num.as_str(), num.den.as_deref()),
            ("10", Some("s^2 + s + 10"))
        );
        assert!(num.note.is_none());

        d.params.insert("K".into(), "s".into());
        assert!(
            render(&r, Mode::Numeric, &d)
                .note
                .unwrap()
                .contains("precisa ser um número")
        );
    }

    #[test]
    fn lista_didatica_e_destaque() {
        let d = examples::reference(Sign::Plus);
        let r = solve(&d).unwrap();
        let v = render(&r, Mode::Symbolic, &d);
        assert_eq!(v.paths.len(), 2);
        assert_eq!(v.loops.len(), 3);
        assert_eq!(v.paths[0].label, "P1 = G1·G2·G4");
        assert_eq!(v.paths[0].cofactor.as_deref(), Some("Δ1 = 1"));
        // Destacar P1 inclui as ramificações A e Y do caminho.
        let nodes = diagram_nodes(&r, &d, &v.paths[0].sfg_nodes);
        let branches = d
            .nodes
            .iter()
            .filter(|n| matches!(n.kind, NodeKind::Branch { .. }))
            .filter(|n| nodes.contains(&n.id))
            .count();
        assert_eq!(branches, 2);
    }

    #[test]
    fn latex_copiado_vem_com_delimitadores() {
        let d = examples::reference(Sign::Plus);
        let v = render(&solve(&d).unwrap(), Mode::Symbolic, &d);
        let f = v.latex.as_str();
        assert_eq!(LatexWrap::Display.apply(f), format!("\\[ T(s) = {f} \\]"));
        assert_eq!(LatexWrap::Markdown.apply(f), format!("$$ T(s) = {f} $$"));
        assert_eq!(LatexWrap::Bare.apply(f), f);
    }

    #[test]
    fn destaque_dos_lacos() {
        let d = examples::reference(Sign::Plus);
        let r = solve(&d).unwrap();
        let v = render(&r, Mode::Symbolic, &d);
        for l in &v.loops {
            let nodes = diagram_nodes(&r, &d, &l.sfg_nodes);
            let names: Vec<String> = nodes
                .iter()
                .filter_map(|id| d.node(*id))
                .filter_map(|n| match &n.kind {
                    NodeKind::Block { expr } => Some(expr.clone()),
                    _ => None,
                })
                .collect();
            assert!(names.contains(&"G1".to_string()), "{}: {names:?}", l.label);
            assert!(names.contains(&"G4".to_string()), "{}: {names:?}", l.label);
        }
        // L3 = G1·G4·H1: os três blocos, Σ2 e a ramificação A.
        let l3 = diagram_nodes(&r, &d, &v.loops[2].sfg_nodes);
        assert_eq!(l3.len(), 5);
    }
}
