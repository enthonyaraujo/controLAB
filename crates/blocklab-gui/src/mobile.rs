//! Editor no tablet e no celular: barra superior com desfazer/refazer e
//! menu, paleta grande (trilho à esquerda no tablet, faixa embaixo no
//! celular), controles de zoom sobre o canvas, barra de ações da seleção
//! (o que no computador fica no teclado e no clique direito) e o resultado
//! num painel (tablet) ou numa folha que sobe de baixo (celular).

use std::sync::Arc;

use blocklab_core::diagram::{NodeKind, examples};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Frame, Galley, Id, Margin, Painter, Rect,
    Response, Sense, Stroke, StrokeKind, TextFormat, TextStyle, ThemePreference, Ui, pos2,
    text::LayoutJob, vec2,
};

use crate::app::{App, Interaction};
use crate::file_actions::Command;
use crate::home::Screen;
use crate::icons::{self, clickable, icon_button};
use crate::layout::{Layout, TOUCH_TARGET};
use crate::theme::{self, medium, regular};

pub const TABLET_PALETTE_W: f32 = 96.0;
const PHONE_PALETTE_H: f32 = 72.0;
const SUMMARY_H: f32 = 52.0;
/// No tablet, o painel de resultado começa aberto a partir desta largura
/// (no retrato ele tomaria espaço demais do canvas).
const RESULT_OPEN_MIN: f32 = 1000.0;

// --- Texto ---------------------------------------------------------------

/// Texto centrado na vertical em `cy`, começando em `x`.
pub fn text_vc(p: &Painter, x: f32, cy: f32, s: &str, font: FontId, color: Color32) -> Rect {
    let g = p.layout_no_wrap(s.to_owned(), font, color);
    let r = Rect::from_min_size(pos2(x, cy - g.size().y / 2.0), g.size());
    p.galley(r.min, g, color);
    r
}

/// Texto quebrado em `width` (ainda não desenhado), no máximo `rows` linhas.
pub fn wrapped(
    p: &Painter,
    s: &str,
    font: FontId,
    color: Color32,
    width: f32,
    line_height: f32,
    rows: usize,
) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(
        s.to_owned(),
        TextFormat {
            font_id: font,
            color,
            line_height: Some(line_height),
            ..Default::default()
        },
    );
    job.wrap.max_width = width.max(1.0);
    job.wrap.max_rows = rows;
    p.layout_job(job)
}

/// Uma linha, cortada com "…" se passar de `width`.
pub fn elided(p: &Painter, s: &str, font: FontId, color: Color32, width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(
        s.to_owned(),
        TextFormat {
            font_id: font,
            color,
            ..Default::default()
        },
    );
    job.wrap.max_width = width.max(1.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    p.layout_job(job)
}

/// Botões e linhas de menu com altura de dedo.
pub fn touch_spacing(ui: &mut Ui) {
    let s = ui.spacing_mut();
    s.button_padding = vec2(14.0, 10.0);
    s.interact_size.y = TOUCH_TARGET - 4.0;
    s.item_spacing = vec2(10.0, 4.0);
    s.icon_width = 20.0;
    s.icon_width_inner = 10.0;
    let styles = &mut ui.style_mut().text_styles;
    styles.insert(TextStyle::Button, regular(15.0));
    styles.insert(TextStyle::Body, regular(15.0));
}

impl App {
    /// Menu "⋮" da barra superior (tela inicial e editor): tudo o que no
    /// computador fica na barra de menus.
    pub fn app_menu(&mut self, button: &Response, editor: bool) {
        egui::Popup::menu(button).width(240.0).show(|ui| {
            touch_spacing(ui);
            if editor {
                for (label, cmd) in [
                    ("Novo diagrama", Command::New),
                    ("Abrir…", Command::Open),
                    ("Salvar", Command::Save),
                    ("Salvar como…", Command::SaveAs),
                    ("Exportar SVG…", Command::ExportSvg),
                    ("Exportar PNG…", Command::ExportPng),
                ] {
                    if ui.button(label).clicked() {
                        self.commands.push(cmd);
                    }
                }
                ui.separator();
                if ui.button("Selecionar tudo").clicked() {
                    self.select_all();
                }
                ui.checkbox(&mut self.show_grid, "Mostrar grade");
                ui.checkbox(&mut self.snap, "Encaixar na grade");
                ui.separator();
                ui.menu_button("Exemplos", |ui| {
                    touch_spacing(ui);
                    for (i, (name, _)) in examples::all().into_iter().enumerate() {
                        if ui.button(name).clicked() {
                            self.commands.push(Command::Example(i));
                        }
                    }
                });
            } else {
                if ui.button("Novo diagrama").clicked() {
                    self.commands.push(Command::New);
                }
                if ui.button("Abrir…").clicked() {
                    self.commands.push(Command::Open);
                }
                if self.opened && ui.button("Voltar ao diagrama").clicked() {
                    self.screen = Screen::Editor;
                }
                ui.separator();
            }
            ui.menu_button("Tema", |ui| {
                touch_spacing(ui);
                for (label, theme) in [
                    ("Modo claro", ThemePreference::Light),
                    ("Modo escuro", ThemePreference::Dark),
                    ("Seguir o sistema", ThemePreference::System),
                ] {
                    if ui.radio(self.prefs.theme == theme, label).clicked() {
                        self.set_theme(ui.ctx(), theme);
                    }
                }
            });
            if ui.button("Gestos e atalhos").clicked() {
                self.show_shortcuts = true;
            }
            if editor && ui.button("Tela inicial").clicked() {
                self.screen = Screen::Home;
            }
            ui.separator();
            if ui.button("Sair").clicked() {
                self.commands.push(Command::Quit);
            }
        });
    }

    fn result_open(&self) -> bool {
        self.result_panel
            .unwrap_or(self.viewport.width() >= RESULT_OPEN_MIN)
    }

    pub fn editor_touch_ui(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let phone = self.layout == Layout::Phone;
        egui::Panel::top("editor-topo")
            .exact_size(if phone { 52.0 } else { 56.0 })
            .frame(Frame::new().fill(t.surface))
            .show(ui, |ui| self.editor_top_bar(ui));
        self.highlight.clear();
        if phone {
            egui::Panel::bottom("editor-paleta")
                .exact_size(PHONE_PALETTE_H)
                .frame(Frame::new().fill(t.surface))
                .show(ui, |ui| self.palette_strip(ui));
            if self.result_sheet {
                let h = (self.viewport.height() * 0.5).max(240.0);
                egui::Panel::bottom("editor-resultado")
                    .exact_size(h)
                    .frame(Frame::new().fill(t.surface).inner_margin(Margin {
                        left: 14,
                        right: 14,
                        top: 0,
                        bottom: 4,
                    }))
                    .show(ui, |ui| {
                        self.sheet_header(ui);
                        self.result_ui(ui);
                    });
            } else {
                egui::Panel::bottom("editor-resumo")
                    .exact_size(SUMMARY_H)
                    .frame(Frame::new().fill(t.surface))
                    .show(ui, |ui| self.result_summary(ui));
            }
        } else {
            egui::Panel::left("paleta")
                .resizable(false)
                .exact_size(TABLET_PALETTE_W)
                .show(ui, |ui| self.palette_ui(ui));
            if self.result_open() {
                egui::Panel::right("resultado")
                    .resizable(true)
                    .default_size(360.0)
                    .min_size(280.0)
                    .max_size((self.viewport.width() * 0.5).max(280.0))
                    .show(ui, |ui| self.result_ui(ui));
            }
        }
        egui::CentralPanel::default()
            .frame(Frame::new())
            .show(ui, |ui| {
                self.canvas_ui(ui);
                self.canvas_controls(ui);
                self.selection_bar(ui);
                self.toast(ui);
            });
    }

    fn editor_top_bar(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let phone = self.layout == Layout::Phone;
        let r = ui.max_rect();
        let p = ui.painter().clone();
        let cy = r.center().y;
        let slot = |x: f32| Rect::from_center_size(pos2(x, cy), vec2(TOUCH_TARGET, TOUCH_TARGET));

        if icon_button(ui, slot(r.left() + 28.0), "Tela inicial", true, icons::back).clicked() {
            self.screen = Screen::Home;
        }

        // Da direita para a esquerda.
        let mut x = r.right() - 28.0;
        let more = icon_button(ui, slot(x), "Mais opções", true, |p, c, s| {
            icons::kebab(p, c, s.color)
        });
        x -= 48.0;
        if !phone {
            let open = self.result_open();
            let rect = Rect::from_center_size(pos2(x - 8.0, cy), vec2(60.0, 36.0));
            let resp = clickable(ui, rect, ui.id().with("resultado"), "Resultado");
            if open {
                p.rect_filled(rect, CornerRadius::same(10), t.nav_pill);
            } else if resp.hovered() {
                p.rect_filled(rect, CornerRadius::same(10), t.hover);
            }
            let color = if open { t.text_strong } else { t.text };
            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "T(s)",
                medium(14.0),
                color,
            );
            if resp.clicked() {
                self.result_panel = Some(!open);
            }
            x = rect.left() - 30.0;
            if icon_button(ui, slot(x), "Ajustar à tela", true, icons::fit).clicked() {
                self.fit_view();
            }
            x -= 60.0;
        }
        let can_redo = self.doc.can_redo();
        if icon_button(ui, slot(x), "Refazer", can_redo, |p, c, s| {
            icons::undo(p, c, s, true)
        })
        .clicked()
        {
            self.redo();
        }
        x -= 48.0;
        let can_undo = self.doc.can_undo();
        if icon_button(ui, slot(x), "Desfazer", can_undo, |p, c, s| {
            icons::undo(p, c, s, false)
        })
        .clicked()
        {
            self.undo();
        }

        let left = r.left() + 56.0;
        let right = x - 26.0;
        let dirty = if self.is_dirty() { " •" } else { "" };
        let name = format!("{}{dirty}", self.doc_name());
        let g = elided(&p, &name, medium(16.0), t.text_strong, right - left);
        p.galley(pos2(left, cy - g.size().y / 2.0), g, t.text_strong);

        self.app_menu(&more, true);
    }

    /// Faixa recolhida do celular: T(s) numa linha; tocar abre a folha.
    fn result_summary(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let r = ui.max_rect();
        let resp = clickable(ui, r, Id::new("editor-resumo"), "Ver resultado");
        let p = ui.painter();
        if resp.hovered() {
            p.rect_filled(r, 0.0, t.hover);
        }
        let (text, error) = if self.solver.is_running() && !self.result.has_outcome() {
            ("calculando…".to_string(), false)
        } else {
            self.result.summary(self.mode, &self.doc.diagram)
        };
        let color = if error { t.error } else { t.text_strong };
        icons::chevron(
            p,
            pos2(r.right() - 28.0, r.center().y),
            Stroke::new(1.6, t.text_muted),
            true,
        );
        let g = elided(p, &text, regular(15.0), color, r.width() - 72.0);
        p.galley(
            pos2(r.left() + 16.0, r.center().y - g.size().y / 2.0),
            g,
            color,
        );
        if resp.clicked() {
            self.result_sheet = true;
        }
    }

    /// Alça da folha de resultado; tocar recolhe.
    fn sheet_header(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
        let resp = clickable(ui, rect, Id::new("editor-folha"), "Recolher resultado");
        let p = ui.painter();
        p.rect_filled(
            Rect::from_center_size(pos2(rect.center().x, rect.top() + 9.0), vec2(36.0, 4.0)),
            CornerRadius::same(2),
            t.dashed,
        );
        icons::chevron(
            p,
            pos2(rect.right() - 14.0, rect.center().y + 3.0),
            Stroke::new(1.6, t.text_muted),
            false,
        );
        if resp.clicked() {
            self.result_sheet = false;
        }
    }

    /// Zoom, enquadrar e seleção múltipla, empilhados no canto do canvas.
    fn canvas_controls(&mut self, ui: &mut Ui) {
        let t = theme::tokens(ui.visuals());
        let r = self.canvas_rect;
        if !r.is_positive() {
            return;
        }
        let phone = self.layout == Layout::Phone;
        let mut items: Vec<&str> = vec!["Aumentar zoom", "Diminuir zoom"];
        if phone {
            // No tablet o enquadrar fica na barra superior.
            items.push("Ajustar à tela");
        }
        items.push("Seleção múltipla");
        let step = TOUCH_TARGET + 2.0;
        let column = Rect::from_min_size(
            pos2(
                r.right() - 12.0 - TOUCH_TARGET - 4.0,
                r.bottom() - 12.0 - step * items.len() as f32 - 6.0,
            ),
            vec2(TOUCH_TARGET + 8.0, step * items.len() as f32 + 6.0),
        );
        if column.top() < r.top() + 8.0 {
            return;
        }
        let p = ui.painter();
        p.rect(
            column,
            CornerRadius::same(14),
            t.surface,
            Stroke::new(1.0, t.control_border),
            StrokeKind::Inside,
        );
        for (k, label) in items.into_iter().enumerate() {
            let slot = Rect::from_min_size(
                column.min + vec2(4.0, 3.0 + step * k as f32),
                vec2(TOUCH_TARGET, TOUCH_TARGET),
            );
            if label == "Seleção múltipla" && self.multi_select {
                p.rect_filled(slot.shrink(2.0), CornerRadius::same(10), t.nav_pill);
            }
            let resp = icon_button(ui, slot, label, true, |p, c, s| match label {
                "Aumentar zoom" => icons::plus(p, c, 6.5, s),
                "Diminuir zoom" => icons::minus(p, c, 6.5, s),
                "Ajustar à tela" => icons::fit(p, c, s),
                _ => icons::select_area(p, c, s),
            });
            if resp.clicked() {
                let c = r.center();
                match label {
                    "Aumentar zoom" => self.view.zoom_at(c, 1.25),
                    "Diminuir zoom" => self.view.zoom_at(c, 0.8),
                    "Ajustar à tela" => self.fit_view(),
                    _ => self.multi_select = !self.multi_select,
                }
            }
        }
    }

    /// Ações sobre o que está selecionado, no alto do canvas.
    fn selection_bar(&mut self, ui: &mut Ui) {
        if self.sel.is_empty()
            || self.editing.is_some()
            || !matches!(self.interaction, Interaction::Idle)
        {
            return;
        }
        let t = theme::tokens(ui.visuals());
        let d = &self.doc.diagram;
        let single = if self.sel.nodes.len() == 1 && self.sel.wires.is_empty() {
            self.sel.nodes.first().and_then(|id| d.node(*id)).cloned()
        } else {
            None
        };
        let flippable = self.sel.nodes.iter().any(|id| {
            d.node(*id)
                .is_some_and(|n| !matches!(n.kind, NodeKind::Branch { .. }))
        });

        let mut actions: Vec<&str> = Vec::new();
        match single.as_ref().map(|n| &n.kind) {
            Some(NodeKind::Block { .. }) => actions.push("Editar"),
            Some(NodeKind::Input { .. } | NodeKind::Output { .. }) => actions.push("Renomear"),
            Some(NodeKind::Sum { .. }) => actions.push("Entradas"),
            _ => {}
        }
        if flippable {
            actions.push("Espelhar");
        }
        actions.push("Apagar");

        let w = 68.0;
        let r = self.canvas_rect;
        let bar = Rect::from_center_size(
            pos2(r.center().x, r.top() + 12.0 + 28.0),
            vec2(w * actions.len() as f32 + 8.0, 56.0),
        );
        let p = ui.painter().clone();
        p.rect(
            bar,
            CornerRadius::same(16),
            t.surface,
            Stroke::new(1.0, t.control_border),
            StrokeKind::Inside,
        );
        for (k, label) in actions.into_iter().enumerate() {
            let cell = Rect::from_min_size(
                bar.min + vec2(4.0 + w * k as f32, 4.0),
                vec2(w, bar.height() - 8.0),
            );
            let resp = clickable(ui, cell, ui.id().with(("acao", label)), label);
            if resp.hovered() || resp.is_pointer_button_down_on() {
                p.rect_filled(cell, CornerRadius::same(12), t.hover);
            }
            let color = if label == "Apagar" { t.error } else { t.text };
            let s = Stroke::new(1.6, color);
            let c = cell.center() - vec2(0.0, 7.0);
            match label {
                "Editar" | "Renomear" => icons::pencil(&p, c, s),
                "Entradas" => icons::dots(&p, c, color),
                "Espelhar" => icons::flip(&p, c, s),
                _ => icons::trash(&p, c, s),
            }
            p.text(
                cell.center() + vec2(0.0, 13.0),
                Align2::CENTER_CENTER,
                label,
                regular(11.5),
                color,
            );
            match label {
                "Entradas" => {
                    if let Some(n) = &single {
                        let id = n.id;
                        egui::Popup::menu(&resp).width(260.0).show(|ui| {
                            touch_spacing(ui);
                            self.node_menu(ui, id);
                        });
                    }
                }
                _ if !resp.clicked() => {}
                "Editar" | "Renomear" => {
                    if let Some(n) = &single {
                        self.start_editing(n.id);
                    }
                }
                "Espelhar" => self.flip_selection(),
                _ => self.delete_selection(),
            }
        }
    }

    /// Aviso passageiro (salvo, copiado...), no lugar da barra de status.
    fn toast(&mut self, ui: &mut Ui) {
        let Some((text, at)) = &self.notice else {
            return;
        };
        let now = ui.input(|i| i.time);
        if now - at >= 4.0 {
            return;
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));
        let t = theme::tokens(ui.visuals());
        let r = self.canvas_rect;
        let p = ui.painter();
        let g = wrapped(
            p,
            text,
            regular(13.5),
            t.text_strong,
            (r.width() - 64.0).min(420.0),
            19.0,
            3,
        );
        let top = if self.sel.is_empty() { 12.0 } else { 80.0 };
        let rect = Rect::from_min_size(
            pos2(r.center().x - g.size().x / 2.0 - 14.0, r.top() + top),
            g.size() + vec2(28.0, 18.0),
        );
        p.rect(
            rect,
            CornerRadius::same(10),
            t.surface,
            Stroke::new(1.0, t.control_border),
            StrokeKind::Inside,
        );
        p.galley(rect.min + vec2(14.0, 9.0), g, t.text_strong);
    }
}
