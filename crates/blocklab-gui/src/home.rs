//! Tela inicial: começar um diagrama, explorar exemplos com
//! pré-visualização de T(s), tutoriais e arquivos recentes.
//!
//! As medidas seguem o protótipo (janela de 1440 × 900): barra lateral de
//! 220 px, painel de pré-visualização de 380 px e cartões de 252 px.

use std::path::Path;

use blocklab_core::diagram::{Diagram, examples};
use blocklab_core::format::{Mode, pretty};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Frame, Id, Painter, Pos2, Rect, Sense, Shape,
    Stroke, StrokeKind, TextFormat, Ui, pos2, text::LayoutJob, vec2,
};

use crate::app::App;
use crate::draw::{self, MiniStyle};
use crate::file_actions::Command;
use crate::icons::{clickable, folder, plus};
use crate::layout::Layout;
use crate::theme::{self, Tokens, medium, regular};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    Home,
    #[default]
    Editor,
}

/// Página escolhida na barra lateral.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Start,
    Examples,
    Learn,
}

/// Um exemplo como aparece nos cartões.
pub struct Entry {
    /// Posição em [`examples::all`].
    pub index: usize,
    pub category: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
}

pub const CATALOG: [Entry; 8] = [
    Entry {
        index: 5,
        category: "Regra de Mason",
        title: "Dois caminhos e dois laços",
        desc: "G2 e G3 em paralelo dentro de dois laços de realimentação",
    },
    Entry {
        index: 7,
        category: "Realimentação",
        title: "Laços aninhados",
        desc: "Um laço interno dentro de um externo, reduzido passo a passo",
    },
    Entry {
        index: 2,
        category: "Realimentação",
        title: "Realimentação não unitária",
        desc: "Malha fechada com H(s) no ramo de retorno",
    },
    Entry {
        index: 3,
        category: "Realimentação",
        title: "Realimentação positiva",
        desc: "O sinal de H entra somando: G/(1 − G·H)",
    },
    Entry {
        index: 4,
        category: "Regra de Mason",
        title: "Dois laços que não se tocam",
        desc: "Laços disjuntos: Δ ganha o produto L1·L2",
    },
    Entry {
        index: 6,
        category: "Regra de Mason",
        title: "Dois caminhos, H1 negativo",
        desc: "O mesmo diagrama com H1 entrando com sinal −",
    },
    Entry {
        index: 0,
        category: "Associação",
        title: "Blocos em série",
        desc: "G1 e G2 em cascata: o ganho é o produto",
    },
    Entry {
        index: 1,
        category: "Associação",
        title: "Blocos em paralelo",
        desc: "G1 e G2 somados na saída",
    },
];

/// Quantos exemplos do catálogo aparecem na página inicial.
pub(crate) const FEATURED: usize = 3;

pub(crate) struct Lesson {
    pub title: &'static str,
    pub desc: &'static str,
    pub command: Option<Command>,
}

pub(crate) const LESSONS: [Lesson; 4] = [
    Lesson {
        title: "Montar o primeiro diagrama",
        desc: "Arraste blocos, somadores e ramificações da paleta e ligue as portas",
        command: Some(Command::New),
    },
    Lesson {
        title: "Ler a função de transferência",
        desc: "T(s) aparece à direita, simbólica ou numérica, pronta para copiar",
        command: Some(Command::Example(2)),
    },
    Lesson {
        title: "Regra de Mason passo a passo",
        desc: "Passe o mouse nos caminhos e laços para destacá-los no diagrama",
        command: Some(Command::Example(5)),
    },
    Lesson {
        title: "Atalhos e comandos",
        desc: "Teclado e mouse no editor: zoom, espelhar, desfazer…",
        command: None,
    },
];

/// Medidas do protótipo.
const SIDEBAR_W: f32 = 220.0;
const PREVIEW_W: f32 = 380.0;
const PAD: f32 = 28.0;
const CARD_W: f32 = 252.0;
const CARD_GAP: f32 = 14.0;
const THUMB_H: f32 = 111.0;
const START_CARD_H: f32 = 168.0;
const EXAMPLE_CARD_H: f32 = 203.0;
const LESSON_CARD_H: f32 = 96.0;

pub struct Home {
    pub page: Page,
    pub search: String,
    /// Exemplo na pré-visualização (posição em [`examples::all`]).
    pub selected: usize,
    /// Prévia aberta numa janela por cima (celular e tablet em retrato).
    pub sheet: bool,
    pub(crate) diagrams: Vec<Diagram>,
    /// T(s) de cada exemplo (numerador, denominador), calculada sob demanda.
    formulas: Vec<Option<(String, Option<String>)>>,
}

impl Default for Home {
    fn default() -> Self {
        let diagrams: Vec<Diagram> = examples::all().into_iter().map(|(_, d)| d).collect();
        Home {
            page: Page::Start,
            search: String::new(),
            selected: CATALOG[0].index,
            sheet: false,
            formulas: vec![None; diagrams.len()],
            diagrams,
        }
    }
}

impl Home {
    pub(crate) fn formula(&mut self, i: usize) -> Option<(String, Option<String>)> {
        if self.formulas[i].is_none() {
            let r = blocklab_core::solve(&self.diagrams[i]).ok()?;
            let pf = pretty(&r.transfer, Mode::Symbolic);
            self.formulas[i] = Some((pf.num.text(), pf.den.as_ref().map(|d| d.text())));
        }
        self.formulas[i].clone()
    }
}

pub(crate) fn entry(index: usize) -> &'static Entry {
    CATALOG
        .iter()
        .find(|e| e.index == index)
        .unwrap_or(&CATALOG[0])
}

/// Busca sem diferenciar maiúsculas nem acentos.
pub(crate) fn matches(query: &str, fields: &[&str]) -> bool {
    let fold = |s: &str| -> String {
        s.chars()
            .flat_map(char::to_lowercase)
            .map(|c| match c {
                'á' | 'à' | 'â' | 'ã' => 'a',
                'é' | 'ê' => 'e',
                'í' => 'i',
                'ó' | 'ô' | 'õ' => 'o',
                'ú' => 'u',
                'ç' => 'c',
                c => c,
            })
            .collect()
    };
    let hay = fold(&fields.join(" "));
    fold(query).split_whitespace().all(|w| hay.contains(w))
}

// --- Desenho -------------------------------------------------------------

/// Texto com o canto superior esquerdo em `pos`; devolve o retângulo.
pub(crate) fn text(p: &Painter, pos: Pos2, s: &str, font: FontId, color: Color32) -> Rect {
    let g = p.layout_no_wrap(s.to_owned(), font, color);
    let r = Rect::from_min_size(pos, g.size());
    p.galley(pos, g, color);
    r
}

/// Texto alinhado à direita em `right`.
pub(crate) fn text_right(p: &Painter, right: f32, top: f32, s: &str, font: FontId, color: Color32) {
    let g = p.layout_no_wrap(s.to_owned(), font, color);
    p.galley(pos2(right - g.size().x, top), g, color);
}

/// Texto quebrado em `width`, com altura de linha fixa; devolve a altura.
pub(crate) fn paragraph(
    p: &Painter,
    pos: Pos2,
    s: &str,
    font: FontId,
    color: Color32,
    width: f32,
    line_height: f32,
) -> f32 {
    let mut job = LayoutJob::single_section(
        s.to_owned(),
        TextFormat {
            font_id: font,
            color,
            line_height: Some(line_height),
            ..Default::default()
        },
    );
    job.wrap.max_width = width;
    let g = p.layout_job(job);
    let h = g.size().y;
    p.galley(pos, g, color);
    h
}

/// Rótulo de seção ("COMEÇAR"): maiúsculas espaçadas.
pub(crate) fn section(p: &Painter, pos: Pos2, s: &str, t: &Tokens) -> Rect {
    let pos = pos + vec2(0.0, 1.0);
    let job = LayoutJob::single_section(
        s.to_owned(),
        TextFormat {
            font_id: medium(12.0),
            color: t.text_section,
            extra_letter_spacing: 0.6,
            ..Default::default()
        },
    );
    let g = p.layout_job(job);
    let r = Rect::from_min_size(pos, g.size());
    p.galley(pos, g, t.text_section);
    r
}

/// Grade de fundo das miniaturas (linhas a cada 16 px).
pub(crate) fn thumb_grid(p: &Painter, r: Rect, color: Color32) {
    let s = Stroke::new(1.0, color);
    let mut x = r.left() + 1.5;
    while x < r.right() {
        p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], s);
        x += 16.0;
    }
    let mut y = r.top() + 1.5;
    while y < r.bottom() {
        p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], s);
        y += 16.0;
    }
}

/// Moldura de cartão: fundo, borda e, se selecionado, contorno de destaque.
pub(crate) fn card_frame(p: &Painter, r: Rect, t: &Tokens, hovered: bool, selected: bool) {
    let border = if hovered && !selected {
        t.control_border_hover
    } else {
        t.card_border
    };
    p.rect(
        r,
        CornerRadius::same(6),
        t.surface,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    if selected {
        p.rect_stroke(
            r.shrink(1.0),
            CornerRadius::same(5),
            Stroke::new(2.0, t.accent),
            StrokeKind::Inside,
        );
    }
}

/// Topo do cartão com a grade (cantos de cima arredondados) e a divisória.
fn card_thumb(p: &Painter, card: Rect, t: &Tokens) -> Rect {
    card_thumb_h(p, card, THUMB_H, t)
}

pub(crate) fn card_thumb_h(p: &Painter, card: Rect, h: f32, t: &Tokens) -> Rect {
    let thumb = Rect::from_min_size(card.min, vec2(card.width(), h));
    let inner = thumb.shrink(1.0);
    p.rect_filled(
        Rect::from_min_max(inner.min, pos2(inner.right(), thumb.bottom())),
        CornerRadius {
            nw: 5,
            ne: 5,
            sw: 0,
            se: 0,
        },
        t.canvas,
    );
    thumb_grid(&p.with_clip_rect(inner), thumb, t.grid);
    p.line_segment(
        [
            pos2(card.left(), thumb.bottom() + 0.5),
            pos2(card.right(), thumb.bottom() + 0.5),
        ],
        Stroke::new(1.0, t.border),
    );
    thumb
}

pub(crate) fn mini_style(t: &Tokens) -> MiniStyle {
    MiniStyle {
        line: t.mini_line,
        text: t.mini_text,
        fill: t.mini_fill,
        margin: 9.0,
        max_zoom: 0.42,
        stretch: 1.0,
        node_scale: 1.45,
        font_min: 7.0,
        font_max: 10.0,
        sum_max: 7.5,
        dot_r: 2.5,
    }
}

pub(crate) fn preview_style(t: &Tokens) -> MiniStyle {
    MiniStyle {
        line: t.preview_line,
        text: t.preview_text,
        fill: t.preview_fill,
        margin: 4.0,
        max_zoom: 0.6,
        stretch: 1.3,
        node_scale: 1.3,
        font_min: 10.0,
        font_max: 13.0,
        sum_max: 11.0,
        dot_r: 3.5,
    }
}

impl App {
    pub fn home_ui(&mut self, ui: &mut Ui) {
        match self.layout {
            Layout::Phone => return self.home_phone_ui(ui),
            Layout::Tablet => return self.home_tablet_ui(ui),
            Layout::Desktop => {}
        }
        let t = theme::tokens(ui.visuals());
        egui::Panel::left("inicio-lateral")
            .resizable(false)
            .exact_size(SIDEBAR_W)
            .frame(Frame::new().fill(t.surface))
            .show(ui, |ui| self.sidebar_ui(ui));
        egui::Panel::right("inicio-previa")
            .resizable(false)
            .exact_size(PREVIEW_W)
            .frame(Frame::new().fill(t.surface))
            .show(ui, |ui| self.preview_ui(ui));
        egui::CentralPanel::default()
            .frame(Frame::new().fill(t.app_bg))
            .show(ui, |ui| self.home_center_ui(ui));
    }

    fn sidebar_ui(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let r = ui.max_rect();
        let o = r.min;
        let p = ui.painter().clone();
        let at = |x: f32, y: f32| o + vec2(x, y);

        text(
            &p,
            at(15.0, 14.0),
            "Diagrama de Blocos",
            medium(14.0),
            t.text_strong,
        );
        text(
            &p,
            at(15.0, 33.0),
            "monte e simplifique",
            regular(11.0),
            t.text_muted,
        );

        // Novo diagrama (botão principal).
        let new = Rect::from_min_size(at(10.0, 65.0), vec2(199.0, 44.0));
        let resp = clickable(ui, new, Id::new("inicio-novo"), "Novo diagrama");
        let fill = if resp.hovered() {
            t.accent_hover
        } else {
            t.accent
        };
        p.rect_filled(new, CornerRadius::same(6), fill);
        plus(
            &p,
            pos2(new.left() + 21.0, new.center().y),
            5.0,
            Stroke::new(1.6, t.on_accent),
        );
        text(
            &p,
            at(51.0, 80.0),
            "Novo diagrama",
            medium(13.0),
            t.on_accent,
        );
        text_right(
            &p,
            new.right() - 13.0,
            o.y + 81.0,
            "Ctrl+N",
            regular(10.5),
            t.on_accent_weak,
        );
        if resp.clicked() {
            self.commands.push(Command::New);
        }

        // Abrir… (secundário).
        let open = Rect::from_min_size(at(10.0, 117.0), vec2(199.0, 44.0));
        let resp = clickable(ui, open, Id::new("inicio-abrir"), "Abrir…");
        let (fill, border) = if resp.hovered() {
            (t.hover, t.control_border_hover)
        } else {
            (t.subtle, t.control_border)
        };
        p.rect(
            open,
            CornerRadius::same(6),
            fill,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
        folder(
            &p,
            Rect::from_min_max(at(24.5, 137.5), at(38.5, 144.5)),
            Stroke::new(1.4, t.text),
        );
        text(&p, at(51.0, 131.0), "Abrir…", regular(13.0), t.text);
        text_right(
            &p,
            open.right() - 13.0,
            o.y + 133.0,
            "Ctrl+O",
            regular(10.5),
            t.text_muted,
        );
        if resp.clicked() {
            self.commands.push(Command::Open);
        }

        p.line_segment(
            [at(10.0, 179.5), at(209.0, 179.5)],
            Stroke::new(1.0, t.border),
        );

        // Navegação.
        for (k, (page, label)) in [
            (Page::Start, "Início"),
            (Page::Examples, "Exemplos"),
            (Page::Learn, "Aprender"),
        ]
        .into_iter()
        .enumerate()
        {
            let item = Rect::from_min_size(at(10.0, 198.0 + 42.0 * k as f32), vec2(199.0, 40.0));
            let resp = clickable(ui, item, Id::new(("inicio-nav", k)), label);
            let current = self.home.page == page && self.home.search.trim().is_empty();
            if current {
                p.rect_filled(item, CornerRadius::same(6), t.accent);
                p.rect_filled(
                    Rect::from_min_max(item.min + vec2(3.0, 0.0), item.max),
                    CornerRadius {
                        nw: 4,
                        sw: 4,
                        ne: 6,
                        se: 6,
                    },
                    t.nav_selected,
                );
                text(
                    &p,
                    item.min + vec2(13.0, 13.0),
                    label,
                    medium(13.0),
                    t.text_strong,
                );
            } else {
                if resp.hovered() {
                    p.rect_filled(item, CornerRadius::same(6), t.subtle);
                }
                text(
                    &p,
                    item.min + vec2(12.0, 13.0),
                    label,
                    regular(13.0),
                    t.text_nav,
                );
            }
            if resp.clicked() {
                self.home.page = page;
                self.home.search.clear();
            }
        }

        let tip = "Dica: Arquivo › Tela inicial reabre esta tela a qualquer momento.";
        paragraph(
            &p,
            pos2(o.x + 14.0, r.bottom() - 47.0),
            tip,
            regular(11.0),
            t.text_muted,
            190.0,
            17.0,
        );
    }

    fn preview_ui(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let r = ui.max_rect();
        let p = ui.painter().clone();
        let x0 = r.left() + 16.0;
        let w = r.width() - 33.0;
        let at = |x: f32, y: f32| pos2(x0 + x, r.top() + y);
        let i = self.home.selected;
        let e = entry(i);

        text(
            &p,
            at(0.0, 14.0),
            "Pré-visualização",
            regular(14.5),
            t.text_heading,
        );
        text_right(
            &p,
            x0 + w,
            r.top() + 16.0,
            e.category,
            regular(11.0),
            t.text_muted,
        );

        let frame = Rect::from_min_size(at(0.0, 45.0), vec2(w, 172.0));
        p.rect(
            frame,
            CornerRadius::same(6),
            t.preview_bg,
            Stroke::new(1.0, t.border),
            StrokeKind::Inside,
        );
        draw::miniature(
            &p.with_clip_rect(frame.shrink(1.0)),
            &self.home.diagrams[i],
            frame,
            &preview_style(t),
        );

        text(&p, at(1.0, 230.0), e.title, medium(15.0), t.text_strong);
        let h = paragraph(
            &p,
            at(0.0, 252.0),
            e.desc,
            regular(12.0),
            t.text_muted,
            w,
            16.0,
        );

        let top = 284.0 + (h - 16.0).max(0.0);
        let boxr = Rect::from_min_size(at(0.0, top), vec2(w, 92.0));
        p.rect(
            boxr,
            CornerRadius::same(6),
            t.formula_bg,
            Stroke::new(1.0, t.border),
            StrokeKind::Inside,
        );
        text(
            &p,
            boxr.min + vec2(13.0, 12.0),
            "Y(s) / X(s)",
            regular(12.0),
            t.text_section,
        );
        if let Some((num, den)) = self.home.formula(i) {
            formula(&p, boxr, &num, den.as_deref(), t);
        }

        let button = Rect::from_min_size(at(0.0, top + 106.0), vec2(w, 40.0));
        let resp = clickable(
            ui,
            button,
            Id::new("inicio-abrir-exemplo"),
            "Abrir exemplo no editor",
        );
        let fill = if resp.hovered() {
            t.accent_hover
        } else {
            t.accent
        };
        p.rect_filled(button, CornerRadius::same(6), fill);
        p.text(
            button.center() + vec2(0.0, -0.5),
            Align2::CENTER_CENTER,
            "Abrir exemplo no editor",
            medium(13.0),
            t.on_accent,
        );
        if resp.clicked() {
            self.commands.push(Command::Example(i));
        }
    }

    fn home_center_ui(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let o = ui.cursor().min;
                let width = (ui.available_width() - 2.0 * PAD).max(CARD_W);
                let x0 = o.x + PAD;
                let p = ui.painter().clone();

                // Busca.
                let search =
                    Rect::from_min_size(pos2(x0, o.y + 22.0), vec2(width.min(560.0), 38.0));
                p.rect(
                    search,
                    CornerRadius::same(6),
                    t.surface,
                    Stroke::new(1.0, t.control_border),
                    StrokeKind::Inside,
                );
                let c = search.left_center() + vec2(20.5, -0.5);
                p.circle_stroke(c, 4.6, Stroke::new(1.4, t.text_muted));
                p.line_segment(
                    [c + vec2(3.4, 3.4), c + vec2(6.6, 6.6)],
                    Stroke::new(1.4, t.text_muted),
                );
                let field = Rect::from_min_max(
                    pos2(search.left() + 42.0, search.top() + 1.0),
                    pos2(search.right() - 10.0, search.bottom() - 1.0),
                );
                ui.put(
                    field,
                    egui::TextEdit::singleline(&mut self.home.search)
                        .frame(Frame::NONE)
                        .margin(vec2(0.0, 0.0))
                        .horizontal_align(egui::Align::LEFT)
                        .font(regular(13.0))
                        .text_color(t.text)
                        .hint_text(
                            egui::RichText::new("Buscar exemplos e tutoriais").color(t.placeholder),
                        )
                        .desired_width(field.width()),
                );

                let query = self.home.search.trim().to_string();
                let bottom = if !query.is_empty() {
                    self.search_results(ui, &p, x0, o.y + 81.0, width, &query, t)
                } else {
                    match self.home.page {
                        Page::Start => self.start_page(ui, &p, x0, o.y, width, t),
                        Page::Examples => {
                            section(&p, pos2(x0, o.y + 81.0), "EXEMPLOS", t);
                            let all: Vec<usize> = CATALOG.iter().map(|e| e.index).collect();
                            self.example_cards(ui, &p, x0, o.y + 107.0, width, &all, t)
                        }
                        Page::Learn => {
                            section(&p, pos2(x0, o.y + 81.0), "APRENDER", t);
                            let all: Vec<usize> = (0..LESSONS.len()).collect();
                            self.lesson_cards(ui, &p, x0, o.y + 107.0, width, &all, t)
                        }
                    }
                };
                ui.allocate_rect(
                    Rect::from_min_max(o, pos2(o.x + width + 2.0 * PAD, bottom + PAD)),
                    Sense::hover(),
                );
            });
    }

    fn start_page(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        t: &Tokens,
    ) -> f32 {
        section(p, pos2(x0, y0 + 81.0), "COMEÇAR", t);
        let cols = columns(width);
        let mut bottom = y0 + 107.0;
        for k in 0..2 {
            let (col, row) = (k % cols, k / cols);
            let card = Rect::from_min_size(
                pos2(
                    x0 + col as f32 * (CARD_W + CARD_GAP),
                    y0 + 107.0 + row as f32 * (START_CARD_H + CARD_GAP),
                ),
                vec2(CARD_W, START_CARD_H),
            );
            bottom = card.bottom();
            let (title, desc) = if k == 0 {
                (
                    "Diagrama em branco",
                    "Canvas vazio com a paleta de elementos",
                )
            } else {
                ("Abrir diagrama", "Escolher um arquivo salvo no computador")
            };
            let resp = clickable(ui, card, Id::new(("inicio-comecar", k)), title);
            card_frame(p, card, t, resp.hovered(), false);
            let thumb = card_thumb(p, card, t);
            let c = thumb.center() + vec2(0.0, 0.5);
            let s = Stroke::new(2.0, t.icon);
            if k == 0 {
                let sq = Rect::from_center_size(c, vec2(34.0, 34.0));
                dashed_round_rect(p, sq, 4.0, s);
                plus(p, c, 6.0, s);
            } else {
                folder(
                    p,
                    Rect::from_center_size(c + vec2(0.0, 2.0), vec2(34.0, 22.0)),
                    s,
                );
                let a = c + vec2(0.0, 8.0);
                let b = c + vec2(0.0, -2.0);
                p.line_segment([a, b], s);
                p.line_segment([b, b + vec2(-4.0, 4.0)], s);
                p.line_segment([b, b + vec2(4.0, 4.0)], s);
            }
            text(p, card.min + vec2(14.0, 123.0), title, medium(13.0), t.text);
            text(
                p,
                card.min + vec2(13.0, 141.0),
                desc,
                regular(11.0),
                t.text_muted,
            );
            if resp.clicked() {
                self.commands
                    .push(if k == 0 { Command::New } else { Command::Open });
            }
        }

        let y = bottom + 31.0;
        let label = section(p, pos2(x0 + 1.0, y - 3.0), "EXEMPLOS", t);
        let link = text(
            p,
            pos2(label.right() + 17.0, y - 2.0),
            "Ver todos",
            regular(12.0),
            t.link,
        );
        let resp = clickable(
            ui,
            link.expand(4.0),
            Id::new("inicio-ver-todos"),
            "Ver todos",
        );
        if resp.hovered() {
            p.line_segment(
                [
                    link.left_bottom() - vec2(0.0, 1.0),
                    link.right_bottom() - vec2(0.0, 1.0),
                ],
                Stroke::new(1.0, t.link),
            );
        }
        if resp.clicked() {
            self.home.page = Page::Examples;
        }
        let featured: Vec<usize> = CATALOG.iter().take(FEATURED).map(|e| e.index).collect();
        let bottom = self.example_cards(ui, p, x0, y + 29.0, width, &featured, t);

        let y = bottom + 25.0;
        section(p, pos2(x0 + 1.0, y - 3.0), "RECENTES", t);
        self.recent_list(
            ui,
            p,
            x0,
            y + 23.0,
            width.clamp(CARD_W, 3.0 * CARD_W + 2.0 * CARD_GAP),
            t,
        )
    }

    /// Cartões de exemplo em grade; devolve o fim (y) da grade.
    #[allow(clippy::too_many_arguments)]
    fn example_cards(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        indices: &[usize],
        t: &Tokens,
    ) -> f32 {
        let cols = columns(width);
        let mut bottom = y0;
        let style = mini_style(t);
        for (k, &i) in indices.iter().enumerate() {
            let (col, row) = (k % cols, k / cols);
            let card = Rect::from_min_size(
                pos2(
                    x0 + col as f32 * (CARD_W + CARD_GAP),
                    y0 + row as f32 * (EXAMPLE_CARD_H + CARD_GAP),
                ),
                vec2(CARD_W, EXAMPLE_CARD_H),
            );
            bottom = card.bottom();
            let e = entry(i);
            let resp = clickable(ui, card, Id::new(("inicio-exemplo", i)), e.title);
            let selected = self.home.selected == i;
            card_frame(p, card, t, resp.hovered(), selected);
            let thumb = card_thumb(p, card, t);
            draw::miniature(
                &p.with_clip_rect(thumb.shrink(1.0)),
                &self.home.diagrams[i],
                thumb,
                &style,
            );
            if selected {
                // A grade e a miniatura pintaram por cima do contorno.
                card_frame_outline(p, card, t);
            }
            text(
                p,
                card.min + vec2(14.0, 122.0),
                e.category,
                regular(11.0),
                t.link,
            );
            text(
                p,
                card.min + vec2(14.0, 139.0),
                e.title,
                medium(13.0),
                t.text,
            );
            paragraph(
                p,
                card.min + vec2(13.0, 158.0),
                e.desc,
                regular(11.0),
                t.text_muted,
                CARD_W - 26.0,
                16.0,
            );
            if resp.clicked() {
                self.home.selected = i;
            }
            if resp.double_clicked() {
                self.commands.push(Command::Example(i));
            }
        }
        bottom
    }

    #[allow(clippy::too_many_arguments)]
    fn lesson_cards(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        indices: &[usize],
        t: &Tokens,
    ) -> f32 {
        let cols = columns(width);
        let mut bottom = y0;
        for (k, &i) in indices.iter().enumerate() {
            let (col, row) = (k % cols, k / cols);
            let card = Rect::from_min_size(
                pos2(
                    x0 + col as f32 * (CARD_W + CARD_GAP),
                    y0 + row as f32 * (LESSON_CARD_H + CARD_GAP),
                ),
                vec2(CARD_W, LESSON_CARD_H),
            );
            bottom = card.bottom();
            let l = &LESSONS[i];
            let resp = clickable(ui, card, Id::new(("inicio-licao", i)), l.title);
            card_frame(p, card, t, resp.hovered(), false);
            text(
                p,
                card.min + vec2(14.0, 11.0),
                "Tutorial",
                regular(11.0),
                t.link,
            );
            text(
                p,
                card.min + vec2(14.0, 27.0),
                l.title,
                medium(13.0),
                t.text,
            );
            paragraph(
                p,
                card.min + vec2(13.0, 46.0),
                l.desc,
                regular(11.0),
                t.text_muted,
                CARD_W - 26.0,
                16.0,
            );
            if resp.clicked() {
                match &l.command {
                    Some(cmd) => self.commands.push(cmd.clone()),
                    None => self.show_shortcuts = true,
                }
            }
        }
        bottom
    }

    #[allow(clippy::too_many_arguments)]
    fn search_results(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        query: &str,
        t: &Tokens,
    ) -> f32 {
        let found: Vec<usize> = CATALOG
            .iter()
            .filter(|e| matches(query, &[e.title, e.category, e.desc]))
            .map(|e| e.index)
            .collect();
        let lessons: Vec<usize> = (0..LESSONS.len())
            .filter(|&i| matches(query, &[LESSONS[i].title, LESSONS[i].desc, "tutorial"]))
            .collect();
        let mut y = y0;
        if found.is_empty() && lessons.is_empty() {
            section(p, pos2(x0, y), "RESULTADOS", t);
            text(
                p,
                pos2(x0, y + 26.0),
                &format!("Nada encontrado para \u{201c}{query}\u{201d}."),
                regular(12.0),
                t.text_muted,
            );
            return y + 44.0;
        }
        if !found.is_empty() {
            section(p, pos2(x0, y), "EXEMPLOS", t);
            y = self.example_cards(ui, p, x0, y + 26.0, width, &found, t) + 25.0;
        }
        if !lessons.is_empty() {
            section(p, pos2(x0, y), "APRENDER", t);
            y = self.lesson_cards(ui, p, x0, y + 26.0, width, &lessons, t) + 25.0;
        }
        y - 25.0
    }

    fn recent_list(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        t: &Tokens,
    ) -> f32 {
        let recent = self.prefs.recent.clone();
        if recent.is_empty() {
            let r = Rect::from_min_size(pos2(x0, y0), vec2(width, 49.0));
            p.rect_filled(r, CornerRadius::same(6), t.recent_bg);
            dashed_round_rect(p, r.shrink(0.5), 6.0, Stroke::new(1.0, t.dashed));
            text(
                p,
                r.min + vec2(18.0, 15.0),
                "Nenhum diagrama recente. Os arquivos que você abrir ou salvar aparecem aqui.",
                regular(12.0),
                t.text_muted,
            );
            return r.bottom();
        }
        let row_h = 40.0;
        let r = Rect::from_min_size(pos2(x0, y0), vec2(width, row_h * recent.len() as f32));
        p.rect(
            r,
            CornerRadius::same(6),
            t.surface,
            Stroke::new(1.0, t.card_border),
            StrokeKind::Inside,
        );
        for (k, path) in recent.iter().enumerate() {
            let row = Rect::from_min_size(r.min + vec2(0.0, k as f32 * row_h), vec2(width, row_h));
            let name = file_label(path);
            let resp = clickable(ui, row, Id::new(("inicio-recente", k)), &name);
            if resp.hovered() {
                p.rect_filled(row.shrink(1.0), CornerRadius::same(5), t.subtle);
            }
            if k > 0 {
                p.line_segment(
                    [
                        row.left_top() + vec2(1.0, 0.5),
                        row.right_top() + vec2(-1.0, 0.5),
                    ],
                    Stroke::new(1.0, t.border),
                );
            }
            let name_r = text(p, row.min + vec2(18.0, 12.0), &name, medium(13.0), t.text);
            let dir = path
                .parent()
                .map(|d| d.display().to_string())
                .unwrap_or_default();
            let clip = Rect::from_min_max(row.min, row.max - vec2(14.0, 0.0));
            text(
                &p.with_clip_rect(clip.intersect(p.clip_rect())),
                pos2(name_r.right() + 12.0, row.top() + 14.0),
                &dir,
                regular(11.0),
                t.text_muted,
            );
            if resp.clicked() {
                self.commands.push(Command::OpenPath(path.clone()));
            }
        }
        r.bottom()
    }

    /// Barra de status da tela inicial.
    pub fn home_status(&self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let r = ui.max_rect();
        let p = ui.painter();
        let first = if self.opened {
            format!("Diagrama aberto: {}", self.doc_name())
        } else {
            "Nenhum diagrama aberto".to_string()
        };
        let a = text(
            p,
            pos2(r.left() + 11.0, r.top() + 5.0),
            &first,
            regular(11.0),
            t.text_muted,
        );
        let dot = pos2(a.right() + 13.0, r.top() + 5.0);
        text(p, dot, "·", regular(11.0), t.text_muted);
        text(
            p,
            pos2(dot.x + 16.0, r.top() + 5.0),
            "Ctrl+N novo · Ctrl+O abrir · F1 ajuda",
            regular(11.0),
            t.text_muted,
        );
    }
}

/// Nome do arquivo para a lista de recentes.
pub(crate) fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn columns(width: f32) -> usize {
    (((width + CARD_GAP) / (CARD_W + CARD_GAP)).floor() as usize).max(1)
}

/// Contorno de destaque redesenhado por cima da miniatura.
pub(crate) fn card_frame_outline(p: &Painter, r: Rect, t: &Tokens) {
    p.rect_stroke(
        r,
        CornerRadius::same(6),
        Stroke::new(1.0, t.card_border),
        StrokeKind::Inside,
    );
    p.rect_stroke(
        r.shrink(1.0),
        CornerRadius::same(5),
        Stroke::new(2.0, t.accent),
        StrokeKind::Inside,
    );
}

/// Retângulo arredondado tracejado.
pub(crate) fn dashed_round_rect(p: &Painter, r: Rect, radius: f32, s: Stroke) {
    let mut pts = Vec::new();
    let arc = |c: Pos2, a0: f32, pts: &mut Vec<Pos2>| {
        for k in 0..=6 {
            let a = a0 + std::f32::consts::FRAC_PI_2 * k as f32 / 6.0;
            pts.push(c + radius * vec2(a.cos(), a.sin()));
        }
    };
    use std::f32::consts::PI;
    arc(r.left_top() + vec2(radius, radius), PI, &mut pts);
    arc(r.right_top() + vec2(-radius, radius), 1.5 * PI, &mut pts);
    arc(r.right_bottom() + vec2(-radius, -radius), 0.0, &mut pts);
    arc(r.left_bottom() + vec2(radius, -radius), 0.5 * PI, &mut pts);
    pts.push(pts[0]);
    p.extend(Shape::dashed_line(&pts, s, 4.0, 3.0));
}

/// "T(s) = num / den" dentro da caixa da fórmula, reduzindo a fonte se não
/// couber na largura.
pub(crate) fn formula(p: &Painter, boxr: Rect, num: &str, den: Option<&str>, t: &Tokens) {
    let label = p.layout_no_wrap("T(s) =".into(), regular(13.0), t.text_section);
    let avail = boxr.width() - 26.0 - label.size().x - 12.0;
    let measure = |size: f32| {
        let w = |s: &str| {
            p.layout_no_wrap(s.into(), regular(size), t.text_strong)
                .size()
                .x
        };
        w(num).max(den.map_or(0.0, w)) + 9.0
    };
    let size = (14.0 * avail / measure(14.0)).clamp(8.0, 14.0);
    let gn = p.layout_no_wrap(num.into(), regular(size), t.text_strong);
    let lx = boxr.left() + 13.0;
    let fx = lx + label.size().x + 12.0;
    match den {
        Some(den) => {
            p.galley(pos2(lx, boxr.top() + 49.0), label, t.text_section);
            let gd = p.layout_no_wrap(den.into(), regular(size), t.text_strong);
            let w = gn.size().x.max(gd.size().x) + 9.0;
            let mid = boxr.top() + 58.5;
            let cx = fx + w / 2.0;
            p.galley(
                pos2(cx - gn.size().x / 2.0, mid - 22.5 + (14.0 - size)),
                gn,
                t.text_strong,
            );
            p.line_segment(
                [pos2(fx, mid), pos2(fx + w, mid)],
                Stroke::new(1.0, t.text_heading),
            );
            p.galley(pos2(cx - gd.size().x / 2.0, mid + 3.5), gd, t.text_strong);
        }
        None => {
            p.galley(pos2(lx, boxr.top() + 49.0), label, t.text_section);
            p.galley(pos2(fx, boxr.top() + 47.0), gn, t.text_strong);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_cobre_todos_os_exemplos() {
        let mut idx: Vec<usize> = CATALOG.iter().map(|e| e.index).collect();
        idx.sort_unstable();
        assert_eq!(idx, (0..examples::all().len()).collect::<Vec<_>>());
    }

    #[test]
    fn busca_ignora_acentos_e_caixa() {
        assert!(matches("lacos", &["Laços aninhados"]));
        assert!(matches("MASON dois", &["Regra de Mason", "Dois caminhos"]));
        assert!(!matches("série", &["Laços aninhados"]));
    }

    #[test]
    fn formula_da_previa() {
        let mut h = Home::default();
        let (num, den) = h.formula(CATALOG[0].index).unwrap();
        assert_eq!(num, "G1·G4·(G2+G3)");
        assert_eq!(den.as_deref(), Some("1 − G1·G4·H1 + G1·G4·(G2+G3)·H2"));
    }
}
