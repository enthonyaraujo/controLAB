//! Novo / abrir / salvar / exportar / sair, com a pergunta "salvar
//! alterações?" quando há mudanças não salvas.
//!
//! Os menus e atalhos só *enfileiram* um [`Command`]; ele roda no fim do
//! quadro (ver [`App::run_commands`]). Assim um diálogo nativo (que
//! bloqueia até o usuário escolher) nunca abre com um menu desenhado por
//! cima.

use std::path::PathBuf;

use blocklab_core::diagram::{Diagram, examples};
use eframe::egui::{self, Id, RichText, ViewportCommand};

use crate::app::{App, Interaction};
use crate::doc::Document;
use crate::home::Screen;
use crate::{export, files};

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    New,
    Open,
    /// Abrir um arquivo conhecido (lista de recentes).
    OpenPath(PathBuf),
    Save,
    SaveAs,
    ExportSvg,
    ExportPng,
    Quit,
    Example(usize),
}

/// Ação que troca ou fecha o documento atual (pode perder alterações).
#[derive(Clone, Debug, PartialEq)]
pub enum Pending {
    Close,
    New,
    /// `None`: perguntar o arquivo no diálogo.
    Open(Option<PathBuf>),
    Example(usize),
}

impl App {
    /// O diagrama difere do que foi aberto/salvo por último?
    pub fn is_dirty(&self) -> bool {
        self.doc.diagram != self.saved
    }

    /// Nome para o título e para sugerir arquivos.
    pub fn doc_name(&self) -> String {
        self.path.as_ref().and_then(|p| p.file_stem()).map_or_else(
            || "sem título".to_string(),
            |s| s.to_string_lossy().into_owned(),
        )
    }

    pub fn window_title(&self) -> String {
        if self.screen == Screen::Home && !self.opened {
            return "controLAB — Diagrama de Blocos".to_string();
        }
        let dirty = if self.is_dirty() { " •" } else { "" };
        format!(
            "{}{dirty} — controLAB — Diagrama de Blocos",
            self.doc_name()
        )
    }

    pub fn notice(&mut self, ctx: &egui::Context, text: String) {
        let now = ctx.input(|i| i.time);
        self.notice = Some((text, now));
    }

    /// Troca o documento inteiro (histórico novo).
    pub fn reset(&mut self, d: Diagram, path: Option<PathBuf>) {
        self.saved = d.clone();
        self.doc = Document::new(d);
        self.path = path;
        self.sel.clear();
        self.editing = None;
        self.interaction = Interaction::Idle;
        self.pending_fit = true;
        self.cancelled = false;
        self.screen = Screen::Editor;
        self.opened = true;
    }

    /// Pede uma ação que pode perder alterações: se houver mudanças não
    /// salvas, pergunta antes.
    pub fn request(&mut self, ctx: &egui::Context, action: Pending) {
        if self.is_dirty() {
            self.pending = Some(action);
        } else {
            self.perform(ctx, action);
        }
    }

    pub fn perform(&mut self, ctx: &egui::Context, action: Pending) {
        match action {
            Pending::Close => {
                self.allow_close = true;
                #[cfg(not(target_os = "android"))]
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Pending::New => self.reset(Diagram::new(), None),
            Pending::Example(i) => {
                if let Some((_, d)) = examples::all().into_iter().nth(i) {
                    self.reset(d, None);
                }
            }
            Pending::Open(path) => {
                #[cfg(target_os = "android")]
                if path.is_none() {
                    self.browse = Some(files::saved_files());
                    return;
                }
                let Some(path) = path.or_else(files::pick_open) else {
                    return;
                };
                match files::load(&path) {
                    Ok(d) => {
                        self.reset(d, Some(path.clone()));
                        self.remember(&path);
                        self.notice(ctx, format!("Aberto: {}", path.display()));
                    }
                    Err(e) => self.error = Some(format!("Não foi possível abrir.\n\n{e}")),
                }
            }
        }
    }

    /// Salva no arquivo atual (ou pergunta onde, se ainda não tem).
    /// Devolve `false` se o usuário desistiu ou deu erro.
    pub fn save(&mut self, ctx: &egui::Context) -> bool {
        match self.path.clone() {
            Some(p) => self.save_to(ctx, p),
            None => self.save_as(ctx),
        }
    }

    pub fn save_as(&mut self, ctx: &egui::Context) -> bool {
        let suggested = self.doc_name().replace("sem título", "diagrama");
        match files::pick_save(
            "Salvar diagrama",
            "Diagrama controLAB — Diagrama de Blocos (JSON)",
            "json",
            &suggested,
        ) {
            Some(p) => self.save_to(ctx, p),
            None => false,
        }
    }

    pub fn save_to(&mut self, ctx: &egui::Context, path: PathBuf) -> bool {
        match files::save(&path, &self.doc.diagram) {
            Ok(()) => {
                self.saved = self.doc.diagram.clone();
                self.notice(ctx, format!("Salvo em {}", path.display()));
                self.remember(&path);
                self.path = Some(path);
                true
            }
            Err(e) => {
                self.error = Some(format!("Não foi possível salvar.\n\n{e}"));
                false
            }
        }
    }

    fn export(&mut self, ctx: &egui::Context, png: bool) {
        let (ext, filter, title) = if png {
            ("png", "Imagem PNG", "Exportar PNG")
        } else {
            ("svg", "Imagem SVG", "Exportar SVG")
        };
        let suggested = self.doc_name().replace("sem título", "diagrama");
        let Some(path) = files::pick_save(title, filter, ext, &suggested) else {
            return;
        };
        let bytes = if png {
            export::to_png(&self.doc.diagram, 2.0)
        } else {
            Ok(export::to_svg(&self.doc.diagram).into_bytes())
        };
        match bytes.and_then(|b| files::write_atomic(&path, &b)) {
            Ok(()) => self.notice(ctx, format!("Exportado: {}", path.display())),
            Err(e) => self.error = Some(format!("Não foi possível exportar.\n\n{e}")),
        }
    }

    /// Executa os comandos enfileirados neste quadro.
    pub fn run_commands(&mut self, ctx: &egui::Context) {
        for cmd in std::mem::take(&mut self.commands) {
            match cmd {
                Command::New => self.request(ctx, Pending::New),
                Command::Open => self.request(ctx, Pending::Open(None)),
                Command::OpenPath(p) => self.request(ctx, Pending::Open(Some(p))),
                Command::Example(i) => self.request(ctx, Pending::Example(i)),
                Command::Quit => self.request(ctx, Pending::Close),
                Command::Save => {
                    self.save(ctx);
                }
                Command::SaveAs => {
                    self.save_as(ctx);
                }
                Command::ExportSvg => self.export(ctx, false),
                Command::ExportPng => self.export(ctx, true),
            }
        }
    }

    /// Fechar a janela, arquivos soltos sobre ela e o título.
    pub fn window_events(&mut self, ctx: &egui::Context) {
        let close = ctx.input(|i| i.viewport().close_requested());
        if close && !self.allow_close && self.is_dirty() && self.screenshot.is_none() {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.pending = Some(Pending::Close);
        }

        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if let Some(p) = dropped.into_iter().find(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        }) {
            self.request(ctx, Pending::Open(Some(p)));
        }

        #[cfg(not(target_os = "android"))]
        {
            let title = self.window_title();
            if title != self.title {
                ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
                self.title = title;
            }
        }
    }

    /// Janelas modais: "salvar alterações?" e mensagens de erro.
    pub fn file_dialogs_ui(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.pending.clone() {
            let what = match action {
                Pending::Close => "fechar",
                Pending::New => "criar um diagrama novo",
                Pending::Open(_) => "abrir outro arquivo",
                Pending::Example(_) => "abrir o exemplo",
            };
            let name = self.doc_name();
            let mut choice = None;
            let resp = egui::Modal::new(Id::new("salvar-alteracoes")).show(ctx, |ui| {
                ui.set_max_width(380.0);
                ui.heading("Salvar alterações?");
                ui.add_space(6.0);
                ui.label(format!(
                    "O diagrama \u{201c}{name}\u{201d} tem alterações não salvas. Salvar antes de {what}?"
                ));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Salvar").strong()).clicked() {
                        choice = Some(0);
                    }
                    if ui.button("Não salvar").clicked() {
                        choice = Some(1);
                    }
                    if ui.button("Cancelar").clicked() {
                        choice = Some(2);
                    }
                });
            });
            if resp.should_close() && choice.is_none() {
                choice = Some(2);
            }
            match choice {
                Some(0) => {
                    // Se o usuário desistir no diálogo de salvar, nada acontece.
                    if self.save(ctx) {
                        self.pending = None;
                        self.perform(ctx, action);
                    }
                }
                Some(1) => {
                    self.pending = None;
                    self.perform(ctx, action);
                }
                Some(_) => self.pending = None,
                None => {}
            }
        }

        if let Some(list) = self.browse.clone() {
            let mut chosen = None;
            let mut close = false;
            let resp = egui::Modal::new(Id::new("abrir-lista")).show(ctx, |ui| {
                ui.set_width((ctx.content_rect().width() - 64.0).clamp(220.0, 420.0));
                crate::mobile::touch_spacing(ui);
                ui.heading("Abrir diagrama");
                ui.add_space(6.0);
                if list.is_empty() {
                    ui.label(
                        "Nenhum diagrama salvo ainda. Use ⋮ → Salvar no editor; os \
                         arquivos ficam na pasta do app.",
                    );
                }
                egui::ScrollArea::vertical()
                    .max_height(ctx.content_rect().height() * 0.6)
                    .show(ui, |ui| {
                        for p in &list {
                            let name = p
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            let button = egui::Button::new(name)
                                .min_size(egui::vec2(ui.available_width(), 0.0));
                            if ui.add(button).clicked() {
                                chosen = Some(p.clone());
                            }
                        }
                    });
                ui.add_space(8.0);
                close = ui.button("Cancelar").clicked();
            });
            if let Some(p) = chosen {
                // O "salvar alterações?" já foi respondido antes da lista.
                self.browse = None;
                self.perform(ctx, Pending::Open(Some(p)));
            } else if close || resp.should_close() {
                self.browse = None;
            }
        }

        if let Some(msg) = self.error.clone() {
            let resp = egui::Modal::new(Id::new("erro-arquivo")).show(ctx, |ui| {
                ui.set_max_width(460.0);
                ui.heading("Erro");
                ui.add_space(6.0);
                ui.label(msg);
                ui.add_space(10.0);
                ui.button("OK").clicked()
            });
            if resp.inner || resp.should_close() {
                self.error = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::{Pos, Sign};
    use blocklab_core::format::Mode;

    fn app() -> (egui::Context, App) {
        let ctx = egui::Context::default();
        let app = App::new(&ctx, examples::series(), None, Mode::Symbolic);
        (ctx, app)
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("blocklab-app-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn sem_alteracoes_troca_direto() {
        let (ctx, mut app) = app();
        assert!(!app.is_dirty());
        app.request(&ctx, Pending::Example(5));
        assert!(app.pending.is_none());
        assert_eq!(app.doc.diagram, examples::reference(Sign::Plus));
        // Histórico novo: não dá para "desfazer" a troca de documento.
        assert!(!app.doc.can_undo());
        assert!(!app.is_dirty());
    }

    #[test]
    fn com_alteracoes_pergunta_antes() {
        let (ctx, mut app) = app();
        app.doc.edit(|d| {
            d.add_block("G9", Pos::new(0.0, 0.0));
        });
        assert!(app.is_dirty());
        let before = app.doc.diagram.clone();
        app.request(&ctx, Pending::New);
        assert_eq!(app.pending, Some(Pending::New));
        assert_eq!(
            app.doc.diagram, before,
            "não pode trocar antes de responder"
        );
        // "Não salvar":
        app.pending = None;
        app.perform(&ctx, Pending::New);
        assert!(app.doc.diagram.nodes.is_empty());
    }

    #[test]
    fn salvar_limpa_a_marca_e_abrir_recupera() {
        let (ctx, mut app) = app();
        app.doc.edit(|d| d.nodes[0].pos.x += 40.0);
        assert!(app.window_title().contains('•'));
        let p = temp("serie.json");
        assert!(app.save_to(&ctx, p.clone()));
        assert!(!app.is_dirty());
        assert_eq!(app.window_title(), "serie — controLAB — Diagrama de Blocos");
        let saved = app.doc.diagram.clone();

        app.perform(&ctx, Pending::New);
        app.perform(&ctx, Pending::Open(Some(p.clone())));
        assert_eq!(app.doc.diagram, saved);
        assert_eq!(app.path.as_deref(), Some(p.as_path()));
        assert!(app.error.is_none());
    }

    #[test]
    fn erro_ao_abrir_vira_mensagem() {
        let (ctx, mut app) = app();
        let before = app.doc.diagram.clone();
        app.perform(&ctx, Pending::Open(Some(temp("nao-existe.json"))));
        assert!(app.error.as_ref().unwrap().contains("nao-existe.json"));
        assert_eq!(app.doc.diagram, before);
    }
}
