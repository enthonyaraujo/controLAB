//! Tela inicial no tablet e no celular (protótipos de 1194 × 834 e
//! 390 × 844).
//!
//! - Tablet: barra superior com a busca, trilho de navegação à esquerda
//!   (botão "Novo" e Início/Exemplos/Aprender), cartões em grade e a
//!   pré-visualização à direita. Em retrato (menos de 1000 px) não cabe o
//!   painel: tocar num exemplo abre a prévia numa janela.
//! - Celular: barra superior, busca, dois botões grandes, exemplos em
//!   lista e a navegação na barra inferior. Tocar num exemplo abre a
//!   prévia numa janela.

use blocklab_core::diagram::Diagram;
use eframe::egui::{
    self, Align2, CornerRadius, Frame, Id, Painter, Pos2, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, Ui, pos2, vec2,
};

use crate::app::App;
use crate::draw;
use crate::file_actions::Command;
use crate::home::{self, CATALOG, FEATURED, LESSONS, Page, entry, file_label, section, text};
use crate::icons::{self, clickable, icon_button};
use crate::layout::{Layout, TOUCH_TARGET};
use crate::mobile::{elided, text_vc, wrapped};
use crate::theme::{self, Tokens, medium, regular};

const TABLET_TOP_H: f32 = 60.0;
const PHONE_TOP_H: f32 = 56.0;
const RAIL_W: f32 = 92.0;
const PREVIEW_W: f32 = 340.0;
/// Largura mínima da janela para o tablet ter o painel de prévia.
const SIDE_PREVIEW_MIN: f32 = 1000.0;
const BOTTOM_NAV_H: f32 = 64.0;
const GAP: f32 = 14.0;
const EXAMPLE_MIN_W: f32 = 200.0;
const EXAMPLE_H: f32 = 195.0;
const EXAMPLE_THUMB_H: f32 = 100.0;

const PAGES: [(Page, &str); 3] = [
    (Page::Start, "Início"),
    (Page::Examples, "Exemplos"),
    (Page::Learn, "Aprender"),
];

fn nav_icon(p: &Painter, page: Page, c: Pos2, s: Stroke) {
    match page {
        Page::Start => icons::home(p, c, s),
        Page::Examples => icons::grid(p, c, s),
        Page::Learn => icons::book(p, c, s),
    }
}

/// Cartão em linha: miniatura à esquerda (se houver) e os textos à
/// direita. Devolve a resposta e o fim (y).
#[allow(clippy::too_many_arguments)]
fn list_card(
    ui: &Ui,
    p: &Painter,
    pos: Pos2,
    width: f32,
    thumb: Option<&Diagram>,
    kicker: &str,
    title: &str,
    desc: &str,
    id: Id,
    t: &Tokens,
) -> (Response, f32) {
    let thumb_w = if thumb.is_some() {
        (width * 0.34).clamp(96.0, 140.0)
    } else {
        0.0
    };
    let tx = pos.x + thumb_w + 14.0;
    let tw = width - thumb_w - 28.0;
    let title_g = elided(p, title, medium(15.0), t.text_strong, tw);
    let desc_g = wrapped(p, desc, regular(12.5), t.text_muted, tw, 18.0, 3);
    let h = (14.0 + 18.0 + 22.0 + desc_g.size().y + 13.0).max(84.0);
    let card = Rect::from_min_size(pos, vec2(width, h));
    let resp = clickable(ui, card, id, title);
    home::card_frame(p, card, t, resp.hovered(), false);
    if let Some(d) = thumb {
        let r = Rect::from_min_size(pos, vec2(thumb_w, h));
        let inner = r.shrink(1.0);
        p.rect_filled(
            Rect::from_min_max(inner.min, pos2(r.right(), inner.bottom())),
            CornerRadius {
                nw: 5,
                sw: 5,
                ne: 0,
                se: 0,
            },
            t.canvas,
        );
        let clip = p.with_clip_rect(inner);
        home::thumb_grid(&clip, r, t.grid);
        draw::miniature(&clip, d, r, &home::mini_style(t));
        p.line_segment(
            [
                pos2(r.right() - 0.5, r.top()),
                pos2(r.right() - 0.5, r.bottom()),
            ],
            Stroke::new(1.0, t.border),
        );
    }
    let mut y = pos.y + 14.0;
    text(p, pos2(tx, y), kicker, regular(12.0), t.link);
    y += 18.0;
    p.galley(pos2(tx, y), title_g, t.text_strong);
    y += 22.0;
    p.galley(pos2(tx, y), desc_g, t.text_muted);
    (resp, card.bottom())
}

impl App {
    pub fn home_tablet_ui(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let side_preview = self.viewport.width() >= SIDE_PREVIEW_MIN;
        egui::Panel::top("inicio-topo")
            .exact_size(TABLET_TOP_H)
            .frame(Frame::new().fill(t.surface))
            .show(ui, |ui| self.home_top_bar(ui));
        egui::Panel::left("inicio-trilho")
            .resizable(false)
            .exact_size(RAIL_W)
            .frame(Frame::new().fill(t.surface))
            .show(ui, |ui| self.home_rail(ui));
        if side_preview {
            self.home.sheet = false;
            egui::Panel::right("inicio-previa-tablet")
                .resizable(false)
                .exact_size(PREVIEW_W)
                .frame(Frame::new().fill(t.surface))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            let o = ui.cursor().min;
                            let w = ui.available_width() - 34.0;
                            let p = ui.painter().clone();
                            let bottom = self.preview_block(
                                ui,
                                &p,
                                o + vec2(17.0, 25.0),
                                w,
                                f32::INFINITY,
                                t,
                            );
                            ui.allocate_rect(
                                Rect::from_min_max(o, pos2(o.x + 1.0, bottom + 24.0)),
                                Sense::hover(),
                            );
                        });
                });
        }
        egui::CentralPanel::default()
            .frame(Frame::new().fill(t.app_bg))
            .show(ui, |ui| self.touch_home_center(ui));
        self.preview_sheet_ui(ui);
    }

    pub fn home_phone_ui(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        egui::Panel::top("inicio-topo")
            .exact_size(PHONE_TOP_H)
            .frame(Frame::new().fill(t.surface))
            .show(ui, |ui| self.home_top_bar(ui));
        egui::Panel::bottom("inicio-nav-inferior")
            .exact_size(BOTTOM_NAV_H)
            .frame(Frame::new().fill(t.surface))
            .show(ui, |ui| {
                let r = ui.max_rect();
                let p = ui.painter().clone();
                let w = r.width() / PAGES.len() as f32;
                for (k, (page, label)) in PAGES.into_iter().enumerate() {
                    let item = Rect::from_min_size(
                        pos2(r.left() + w * k as f32, r.top() + 4.0),
                        vec2(w, r.height() - 8.0),
                    );
                    let resp = clickable(ui, item, Id::new(("inicio-nav", k)), label);
                    self.nav_item(&p, item, page, label, resp.hovered(), t);
                    if resp.clicked() {
                        self.go_to(page);
                    }
                }
            });
        egui::CentralPanel::default()
            .frame(Frame::new().fill(t.app_bg))
            .show(ui, |ui| self.touch_home_center(ui));
        self.preview_sheet_ui(ui);
    }

    fn go_to(&mut self, page: Page) {
        self.home.page = page;
        self.home.search.clear();
    }

    /// Ícone numa pílula e o rótulo embaixo (trilho e barra inferior).
    fn nav_item(
        &self,
        p: &Painter,
        item: Rect,
        page: Page,
        label: &str,
        hovered: bool,
        t: &Tokens,
    ) {
        let current = self.home.page == page && self.home.search.trim().is_empty();
        let pill =
            Rect::from_center_size(pos2(item.center().x, item.top() + 18.0), vec2(56.0, 30.0));
        if current {
            p.rect_filled(pill, CornerRadius::same(15), t.nav_pill);
        } else if hovered {
            p.rect_filled(pill, CornerRadius::same(15), t.hover);
        }
        let color = if current { t.text_strong } else { t.text_muted };
        nav_icon(p, page, pill.center(), Stroke::new(1.5, color));
        let font = if current { medium(12.0) } else { regular(12.0) };
        p.text(
            pos2(item.center().x, pill.bottom() + 12.0),
            Align2::CENTER_CENTER,
            label,
            font,
            color,
        );
    }

    fn home_top_bar(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let phone = self.layout == Layout::Phone;
        let r = ui.max_rect();
        let p = ui.painter().clone();
        let cy = r.center().y;
        let title = text_vc(
            &p,
            r.left() + if phone { 16.0 } else { 20.0 },
            cy,
            "Diagrama de Blocos",
            medium(if phone { 18.0 } else { 17.0 }),
            t.text_strong,
        );
        let (edge, step) = if phone { (28.0, 52.0) } else { (34.0, 60.0) };
        let slot = |k: f32| {
            Rect::from_center_size(
                pos2(r.right() - edge - k * step, cy),
                vec2(TOUCH_TARGET, TOUCH_TARGET),
            )
        };
        let more = icon_button(ui, slot(0.0), "Mais opções", true, |p, c, s| {
            icons::kebab(p, c, s.color)
        });
        if phone {
            let open = icon_button(ui, slot(1.0), "Abrir diagrama", true, |p, c, s| {
                icons::folder(
                    p,
                    Rect::from_center_size(c + vec2(0.0, 1.5), vec2(17.0, 12.0)),
                    s,
                )
            });
            if open.clicked() {
                self.commands.push(Command::Open);
            }
        } else {
            if icon_button(ui, slot(1.0), "Gestos e atalhos", true, icons::help).clicked() {
                self.show_shortcuts = true;
            }
            let left = (r.left() + 236.0).max(title.right() + 24.0);
            let right = (r.left() + 756.0).min(slot(1.0).left() - 24.0);
            if right - left > 160.0 {
                let rect = Rect::from_min_max(pos2(left, cy - 22.0), pos2(right, cy + 22.0));
                self.search_box(ui, &p, rect, 22.0, t.app_bg, t);
            }
        }
        self.app_menu(&more, false);
    }

    /// Campo de busca com a lupa e, havendo texto, o botão de limpar.
    fn search_box(
        &mut self,
        ui: &mut Ui,
        p: &Painter,
        rect: Rect,
        radius: f32,
        fill: egui::Color32,
        t: &Tokens,
    ) {
        p.rect(
            rect,
            CornerRadius::same(radius as u8),
            fill,
            Stroke::new(1.0, t.control_border),
            StrokeKind::Inside,
        );
        icons::search(
            p,
            pos2(rect.left() + 24.0, rect.center().y),
            Stroke::new(1.5, t.text_muted),
        );
        let mut right = rect.right() - 14.0;
        if !self.home.search.is_empty() {
            let clear = Rect::from_center_size(
                pos2(rect.right() - 24.0, rect.center().y),
                vec2(40.0, 40.0),
            );
            if icon_button(ui, clear, "Limpar busca", true, icons::close).clicked() {
                self.home.search.clear();
            }
            right = clear.left();
        }
        let field = Rect::from_min_max(
            pos2(rect.left() + 46.0, rect.top() + 1.0),
            pos2(right, rect.bottom() - 1.0),
        );
        ui.put(
            field,
            egui::TextEdit::singleline(&mut self.home.search)
                .frame(Frame::NONE)
                .margin(vec2(0.0, 0.0))
                .horizontal_align(egui::Align::LEFT)
                .font(regular(14.5))
                .text_color(t.text)
                .hint_text(RichText::new("Buscar exemplos e tutoriais").color(t.placeholder))
                .desired_width(field.width()),
        );
    }

    fn home_rail(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let r = ui.max_rect();
        let p = ui.painter().clone();
        let cx = r.center().x;

        let fab = Rect::from_center_size(pos2(cx, r.top() + 45.0), vec2(56.0, 56.0));
        let resp = clickable(ui, fab, Id::new("inicio-novo-trilho"), "Novo diagrama");
        let fill = if resp.hovered() {
            t.accent_hover
        } else {
            t.accent
        };
        p.rect_filled(fab, CornerRadius::same(14), fill);
        icons::plus(&p, fab.center(), 9.0, Stroke::new(2.0, t.on_accent));
        p.text(
            pos2(cx, fab.bottom() + 13.0),
            Align2::CENTER_CENTER,
            "Novo",
            regular(12.0),
            t.text_muted,
        );
        if resp.clicked() {
            self.commands.push(Command::New);
        }

        for (k, (page, label)) in PAGES.into_iter().enumerate() {
            let item = Rect::from_min_size(
                pos2(r.left() + 4.0, r.top() + 116.0 + 70.0 * k as f32),
                vec2(r.width() - 8.0, 64.0),
            );
            let resp = clickable(ui, item, Id::new(("inicio-nav", k)), label);
            self.nav_item(&p, item, page, label, resp.hovered(), t);
            if resp.clicked() {
                self.go_to(page);
            }
        }
    }

    fn touch_home_center(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let phone = self.layout == Layout::Phone;
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                let o = ui.cursor().min;
                let pad = if phone { 16.0 } else { 24.0 };
                let width = (ui.available_width() - 2.0 * pad).max(200.0);
                let x0 = o.x + pad;
                let p = ui.painter().clone();
                let mut y = o.y + if phone { 16.0 } else { 24.0 };
                if phone {
                    let rect = Rect::from_min_size(pos2(x0, y), vec2(width, 48.0));
                    self.search_box(ui, &p, rect, 12.0, t.surface, t);
                    y = rect.bottom() + 17.0;
                }
                let query = self.home.search.trim().to_string();
                let bottom = if !query.is_empty() {
                    self.touch_search_results(ui, &p, x0, y, width, &query, t)
                } else {
                    match self.home.page {
                        Page::Start => self.touch_start_page(ui, &p, x0, y, width, t),
                        Page::Examples => {
                            let label = section(&p, pos2(x0, y), "EXEMPLOS", t);
                            let all: Vec<usize> = CATALOG.iter().map(|e| e.index).collect();
                            self.touch_examples(ui, &p, x0, label.bottom() + 16.0, width, &all, t)
                        }
                        Page::Learn => {
                            let label = section(&p, pos2(x0, y), "APRENDER", t);
                            let all: Vec<usize> = (0..LESSONS.len()).collect();
                            self.touch_lessons(ui, &p, x0, label.bottom() + 16.0, width, &all, t)
                        }
                    }
                };
                ui.allocate_rect(
                    Rect::from_min_max(o, pos2(o.x + width + 2.0 * pad, bottom + pad)),
                    Sense::hover(),
                );
            });
    }

    fn touch_start_page(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        t: &Tokens,
    ) -> f32 {
        let phone = self.layout == Layout::Phone;
        let mut y = y0;
        if !phone {
            y = section(p, pos2(x0, y), "COMEÇAR", t).bottom() + 11.0;
        }

        // Novo / abrir.
        let gap = if phone { 12.0 } else { GAP };
        let side_by_side = width >= 260.0;
        let tile_w = if side_by_side {
            (width - gap) / 2.0
        } else {
            width
        };
        let tile_h = if phone { 96.0 } else { 76.0 };
        let mut tiles_bottom = y;
        for k in 0..2 {
            let (col, row) = if side_by_side { (k, 0) } else { (0, k) };
            let r = Rect::from_min_size(
                pos2(
                    x0 + col as f32 * (tile_w + gap),
                    y + row as f32 * (tile_h + gap),
                ),
                vec2(tile_w, tile_h),
            );
            tiles_bottom = r.bottom();
            let (title, sub, cmd) = match (k, phone) {
                (0, true) => ("Novo diagrama", "", Command::New),
                (0, false) => (
                    "Diagrama em branco",
                    "Canvas vazio com a paleta",
                    Command::New,
                ),
                (_, true) => ("Abrir diagrama", "", Command::Open),
                (_, false) => ("Abrir diagrama", "Arquivo salvo no aparelho", Command::Open),
            };
            let resp = clickable(ui, r, Id::new(("inicio-comecar", k)), title);
            let (fg, sub_color) = if k == 0 {
                let fill = if resp.hovered() {
                    t.accent_hover
                } else {
                    t.accent
                };
                p.rect_filled(r, CornerRadius::same(8), fill);
                (t.on_accent, t.on_accent_weak)
            } else {
                let (fill, border) = if resp.hovered() {
                    (t.hover, t.control_border_hover)
                } else {
                    (t.surface, t.card_border)
                };
                p.rect(
                    r,
                    CornerRadius::same(8),
                    fill,
                    Stroke::new(1.0, border),
                    StrokeKind::Inside,
                );
                (t.text_strong, t.text_muted)
            };
            let s = Stroke::new(1.8, fg);
            let icon_at = if phone {
                r.min + vec2(24.0, 26.0)
            } else {
                pos2(r.left() + 30.0, r.center().y)
            };
            if k == 0 {
                icons::plus(p, icon_at, 7.5, s);
            } else {
                icons::folder(
                    p,
                    Rect::from_center_size(icon_at + vec2(0.0, 1.5), vec2(19.0, 13.0)),
                    s,
                );
            }
            if phone {
                let g = elided(p, title, medium(15.0), fg, r.width() - 30.0);
                p.galley(pos2(r.left() + 15.0, r.top() + 48.0), g, fg);
            } else {
                let w = r.width() - 72.0;
                let g = elided(p, title, medium(15.0), fg, w);
                p.galley(pos2(r.left() + 58.0, r.center().y - 19.0), g, fg);
                let g = elided(p, sub, regular(12.0), sub_color, w);
                p.galley(pos2(r.left() + 58.0, r.center().y + 2.0), g, sub_color);
            }
            if resp.clicked() {
                self.commands.push(cmd);
            }
        }

        // Exemplos em destaque.
        let y = tiles_bottom + 37.0;
        let label = section(p, pos2(x0, y), "EXEMPLOS", t);
        let g = p.layout_no_wrap("Ver todos".into(), regular(13.0), t.link);
        let link_at = if phone {
            pos2(x0 + width - g.size().x, y - 1.0)
        } else {
            pos2(label.right() + 16.0, y - 1.0)
        };
        let link = Rect::from_min_size(link_at, g.size());
        p.galley(link_at, g, t.link);
        let resp = clickable(
            ui,
            link.expand2(vec2(10.0, 14.0)),
            Id::new("inicio-ver-todos"),
            "Ver todos",
        );
        if resp.clicked() {
            self.go_to(Page::Examples);
        }
        let featured: Vec<usize> = CATALOG.iter().take(FEATURED).map(|e| e.index).collect();
        let bottom = self.touch_examples(ui, p, x0, label.bottom() + 24.0, width, &featured, t);

        let label = section(p, pos2(x0, bottom + 26.0), "RECENTES", t);
        self.touch_recent(ui, p, x0, label.bottom() + 12.0, width, t)
    }

    /// Exemplos: lista no celular, grade no tablet. Devolve o fim (y).
    #[allow(clippy::too_many_arguments)]
    fn touch_examples(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        indices: &[usize],
        t: &Tokens,
    ) -> f32 {
        if self.layout == Layout::Phone {
            let mut y = y0;
            for &i in indices {
                let e = entry(i);
                let (resp, bottom) = list_card(
                    ui,
                    p,
                    pos2(x0, y),
                    width,
                    Some(&self.home.diagrams[i]),
                    e.category,
                    e.title,
                    e.desc,
                    Id::new(("inicio-exemplo", i)),
                    t,
                );
                if resp.clicked() {
                    self.home.selected = i;
                    self.home.sheet = true;
                }
                y = bottom + 10.0;
            }
            return y - 10.0;
        }

        let side_preview = self.viewport.width() >= SIDE_PREVIEW_MIN;
        let cols = (((width + GAP) / (EXAMPLE_MIN_W + GAP)).floor() as usize).clamp(1, 3);
        let w = (width - GAP * (cols - 1) as f32) / cols as f32;
        let style = home::mini_style(t);
        let mut bottom = y0;
        for (k, &i) in indices.iter().enumerate() {
            let (col, row) = (k % cols, k / cols);
            let card = Rect::from_min_size(
                pos2(
                    x0 + col as f32 * (w + GAP),
                    y0 + row as f32 * (EXAMPLE_H + GAP),
                ),
                vec2(w, EXAMPLE_H),
            );
            bottom = card.bottom();
            let e = entry(i);
            let resp = clickable(ui, card, Id::new(("inicio-exemplo", i)), e.title);
            let selected = side_preview && self.home.selected == i;
            home::card_frame(p, card, t, resp.hovered(), selected);
            let thumb = home::card_thumb_h(p, card, EXAMPLE_THUMB_H, t);
            draw::miniature(
                &p.with_clip_rect(thumb.shrink(1.0)),
                &self.home.diagrams[i],
                thumb,
                &style,
            );
            if selected {
                home::card_frame_outline(p, card, t);
            }
            let tx = card.left() + 12.0;
            let tw = w - 24.0;
            text(
                p,
                pos2(tx, thumb.bottom() + 11.0),
                e.category,
                regular(11.5),
                t.link,
            );
            let g = elided(p, e.title, medium(14.0), t.text_strong, tw);
            p.galley(pos2(tx, thumb.bottom() + 28.0), g, t.text_strong);
            let g = wrapped(p, e.desc, regular(12.0), t.text_muted, tw, 17.0, 2);
            p.galley(pos2(tx, thumb.bottom() + 49.0), g, t.text_muted);
            if resp.clicked() {
                self.home.selected = i;
                self.home.sheet = !side_preview;
            }
            if side_preview && resp.double_clicked() {
                self.commands.push(Command::Example(i));
            }
        }
        bottom
    }

    #[allow(clippy::too_many_arguments)]
    fn touch_lessons(
        &mut self,
        ui: &Ui,
        p: &Painter,
        x0: f32,
        y0: f32,
        width: f32,
        indices: &[usize],
        t: &Tokens,
    ) -> f32 {
        let cols = if width >= 560.0 { 2 } else { 1 };
        let w = (width - GAP * (cols - 1) as f32) / cols as f32;
        let mut y = y0;
        let mut bottom = y0;
        for (k, &i) in indices.iter().enumerate() {
            let col = k % cols;
            if col == 0 && k > 0 {
                y = bottom + 10.0;
            }
            let l = &LESSONS[i];
            // Sem teclado, a lição de atalhos vira a de gestos.
            let (title, desc) = match l.command {
                None => (
                    "Gestos e atalhos",
                    "Toque, pinça e toque longo no editor; atalhos se houver teclado",
                ),
                Some(_) => (l.title, l.desc),
            };
            let (resp, card_bottom) = list_card(
                ui,
                p,
                pos2(x0 + col as f32 * (w + GAP), y),
                w,
                None,
                "Tutorial",
                title,
                desc,
                Id::new(("inicio-licao", i)),
                t,
            );
            bottom = bottom.max(card_bottom);
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
    fn touch_search_results(
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
            .filter(|e| home::matches(query, &[e.title, e.category, e.desc]))
            .map(|e| e.index)
            .collect();
        let lessons: Vec<usize> = (0..LESSONS.len())
            .filter(|&i| home::matches(query, &[LESSONS[i].title, LESSONS[i].desc, "tutorial"]))
            .collect();
        let mut y = y0;
        if found.is_empty() && lessons.is_empty() {
            let label = section(p, pos2(x0, y), "RESULTADOS", t);
            let g = wrapped(
                p,
                &format!("Nada encontrado para \u{201c}{query}\u{201d}."),
                regular(13.0),
                t.text_muted,
                width,
                19.0,
                3,
            );
            let h = g.size().y;
            p.galley(pos2(x0, label.bottom() + 12.0), g, t.text_muted);
            return label.bottom() + 12.0 + h;
        }
        if !found.is_empty() {
            let label = section(p, pos2(x0, y), "EXEMPLOS", t);
            y = self.touch_examples(ui, p, x0, label.bottom() + 16.0, width, &found, t) + 26.0;
        }
        if !lessons.is_empty() {
            let label = section(p, pos2(x0, y), "APRENDER", t);
            y = self.touch_lessons(ui, p, x0, label.bottom() + 16.0, width, &lessons, t) + 26.0;
        }
        y - 26.0
    }

    fn touch_recent(
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
            let g = wrapped(
                p,
                "Nenhum diagrama recente. Os arquivos que você abrir ou salvar aparecem aqui.",
                regular(13.0),
                t.text_muted,
                width - 32.0,
                20.0,
                4,
            );
            let r = Rect::from_min_size(pos2(x0, y0), vec2(width, g.size().y + 26.0));
            p.rect_filled(r, CornerRadius::same(8), t.recent_bg);
            home::dashed_round_rect(p, r.shrink(0.5), 8.0, Stroke::new(1.0, t.dashed));
            p.galley(r.min + vec2(16.0, 13.0), g, t.text_muted);
            return r.bottom();
        }
        let row_h = 58.0;
        let r = Rect::from_min_size(pos2(x0, y0), vec2(width, row_h * recent.len() as f32));
        p.rect(
            r,
            CornerRadius::same(8),
            t.surface,
            Stroke::new(1.0, t.card_border),
            StrokeKind::Inside,
        );
        for (k, path) in recent.iter().enumerate() {
            let row = Rect::from_min_size(r.min + vec2(0.0, k as f32 * row_h), vec2(width, row_h));
            let name = file_label(path);
            let resp = clickable(ui, row, Id::new(("inicio-recente", k)), &name);
            if resp.hovered() {
                p.rect_filled(row.shrink(1.0), CornerRadius::same(7), t.subtle);
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
            let g = elided(p, &name, medium(14.0), t.text, width - 32.0);
            p.galley(row.min + vec2(16.0, 10.0), g, t.text);
            let dir = path
                .parent()
                .map(|d| d.display().to_string())
                .unwrap_or_default();
            let g = elided(p, &dir, regular(12.0), t.text_muted, width - 32.0);
            p.galley(row.min + vec2(16.0, 31.0), g, t.text_muted);
            if resp.clicked() {
                self.commands.push(Command::OpenPath(path.clone()));
            }
        }
        r.bottom()
    }

    /// Prévia do exemplo escolhido (painel do tablet e janela do celular):
    /// miniatura, T(s) e o botão para abrir. Com pouca altura (celular
    /// deitado) a miniatura encolhe para o botão continuar à vista.
    /// Devolve o fim (y).
    fn preview_block(
        &mut self,
        ui: &Ui,
        p: &Painter,
        at: Pos2,
        w: f32,
        max_h: f32,
        t: &Tokens,
    ) -> f32 {
        let i = self.home.selected;
        let e = entry(i);
        let heading = text(p, at, "Pré-visualização", regular(15.0), t.text_heading);
        // Tudo menos a miniatura ocupa cerca de 300 px.
        let thumb_h = (w * 0.53).clamp(130.0, 200.0).min(max_h - 300.0).max(60.0);
        let frame = Rect::from_min_size(pos2(at.x, heading.bottom() + 13.0), vec2(w, thumb_h));
        p.rect(
            frame,
            CornerRadius::same(8),
            t.preview_bg,
            Stroke::new(1.0, t.border),
            StrokeKind::Inside,
        );
        draw::miniature(
            &p.with_clip_rect(frame.shrink(1.0)),
            &self.home.diagrams[i],
            frame,
            &home::preview_style(t),
        );
        let mut y = frame.bottom() + 17.0;
        text(p, pos2(at.x, y), e.category, regular(12.0), t.link);
        y += 18.0;
        let g = wrapped(p, e.title, medium(17.0), t.text_strong, w, 22.0, 2);
        let title_h = g.size().y;
        p.galley(pos2(at.x, y), g, t.text_strong);
        y += title_h + 4.0;
        let h = home::paragraph(
            p,
            pos2(at.x, y),
            e.desc,
            regular(13.0),
            t.text_muted,
            w,
            19.0,
        );
        y += h + 14.0;
        let boxr = Rect::from_min_size(pos2(at.x, y), vec2(w, 92.0));
        p.rect(
            boxr,
            CornerRadius::same(8),
            t.formula_bg,
            Stroke::new(1.0, t.border),
            StrokeKind::Inside,
        );
        text(
            p,
            boxr.min + vec2(13.0, 12.0),
            "Y(s) / X(s)",
            regular(12.0),
            t.text_section,
        );
        if let Some((num, den)) = self.home.formula(i) {
            home::formula(p, boxr, &num, den.as_deref(), t);
        }
        let button = Rect::from_min_size(pos2(at.x, boxr.bottom() + 14.0), vec2(w, 48.0));
        let resp = clickable(
            ui,
            button,
            Id::new("inicio-abrir-no-editor"),
            "Abrir no editor",
        );
        let fill = if resp.hovered() {
            t.accent_hover
        } else {
            t.accent
        };
        p.rect_filled(button, CornerRadius::same(8), fill);
        p.text(
            button.center(),
            Align2::CENTER_CENTER,
            "Abrir no editor",
            medium(14.0),
            t.on_accent,
        );
        if resp.clicked() {
            self.commands.push(Command::Example(i));
            self.home.sheet = false;
        }
        button.bottom()
    }

    /// Prévia numa janela por cima (celular e tablet em retrato).
    fn preview_sheet_ui(&mut self, ui: &Ui) {
        if !self.home.sheet {
            return;
        }
        let t = theme::tokens(ui.visuals());
        let width = (self.viewport.width() - 32.0).min(440.0);
        let max_h = self.viewport.height() - 48.0;
        let resp = egui::Modal::new(Id::new("inicio-previa-janela"))
            .frame(
                Frame::new()
                    .fill(t.surface)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(12)
                    .inner_margin(16),
            )
            .show(ui.ctx(), |ui| {
                ui.set_width(width - 32.0);
                let o = ui.cursor().min;
                let w = ui.available_width();
                let p = ui.painter().clone();
                let bottom = self.preview_block(ui, &p, o, w, max_h - 32.0, t);
                let close = Rect::from_center_size(
                    pos2(o.x + w - 18.0, o.y + 9.0),
                    vec2(TOUCH_TARGET, TOUCH_TARGET),
                );
                let closed = icon_button(ui, close, "Fechar", true, icons::close).clicked();
                ui.allocate_rect(Rect::from_min_max(o, pos2(o.x + w, bottom)), Sense::hover());
                closed
            });
        if resp.inner || resp.should_close() {
            self.home.sheet = false;
        }
    }
}
