//! Estado da aplicação e o quadro principal: menus, atalhos e painéis.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use blocklab_core::diagram::{Diagram, NodeId, NodeKind, Pos, Sign, WireId, examples};
use blocklab_core::format::Mode;
use eframe::egui::{
    self, Frame, Key, KeyboardShortcut, Margin, Modifiers, Rect, ThemePreference, vec2,
};

use crate::doc::Document;
use crate::file_actions::{Command, Pending};
use crate::geometry::{self, Handle};
use crate::home::{Home, Screen};
use crate::layout::Layout;
use crate::prefs::Prefs;
use crate::result_panel::ResultState;
use crate::safe_area::SafeArea;
use crate::solver::Solver;
use crate::theme;
use crate::view::View;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Selection {
    pub nodes: BTreeSet<NodeId>,
    pub wires: BTreeSet<WireId>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.wires.is_empty()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.wires.clear();
    }

    pub fn only_node(id: NodeId) -> Self {
        Selection {
            nodes: BTreeSet::from([id]),
            wires: BTreeSet::new(),
        }
    }
}

/// O que o mouse está fazendo no canvas.
#[derive(Clone, Debug, Default)]
pub enum Interaction {
    #[default]
    Idle,
    /// Arrastando os nós selecionados (posições de antes, para encaixe).
    Moving {
        grab: egui::Pos2,
        originals: Vec<(NodeId, Pos)>,
    },
    /// Puxando um fio a partir de uma porta.
    Connecting {
        from: Handle,
    },
    /// Retângulo de seleção (coordenadas de tela).
    BoxSelect {
        start: egui::Pos2,
        additive: bool,
    },
    Panning,
}

/// Texto sendo editado num nó (expressão do bloco ou nome da entrada/saída).
#[derive(Clone, Debug)]
pub struct Editing {
    pub node: NodeId,
    pub text: String,
    pub focus_requested: bool,
}

/// Captura de tela automática (`--screenshot arquivo.png`), útil para
/// conferir a interface sem interagir.
pub struct ScreenshotPlan {
    pub path: PathBuf,
    pub requested: bool,
}

pub struct App {
    pub doc: Document,
    pub view: View,
    pub sel: Selection,
    pub interaction: Interaction,
    pub editing: Option<Editing>,
    pub solver: Solver,
    pub result: ResultState,
    pub mode: Mode,
    pub snap: bool,
    pub show_grid: bool,
    /// Nós a destacar (passar o mouse sobre um caminho/laço na lista).
    pub highlight: Vec<NodeId>,
    /// Alvo do menu de contexto aberto (clique direito).
    pub context: Option<crate::canvas::Hit>,
    pub context_world: egui::Pos2,
    /// Enquadrar o diagrama no próximo quadro (ao abrir um exemplo).
    pub pending_fit: bool,
    pub canvas_rect: Rect,
    pub show_shortcuts: bool,
    /// O usuário cancelou o cálculo em andamento.
    pub cancelled: bool,
    pub screenshot: Option<ScreenshotPlan>,
    /// Arquivo do documento atual (`None` = ainda não salvo).
    pub path: Option<PathBuf>,
    /// Diagrama como estava ao abrir/salvar (para saber se há mudanças).
    pub saved: Diagram,
    /// Ação esperando a resposta de "salvar alterações?".
    pub pending: Option<Pending>,
    /// Mensagem de erro a mostrar (janela modal).
    pub error: Option<String>,
    /// Aviso passageiro na barra de status (texto, instante).
    pub notice: Option<(String, f64)>,
    /// O usuário já confirmou que pode fechar.
    pub allow_close: bool,
    /// Título atual da janela (só reenvia quando muda).
    pub title: String,
    /// Comandos de menu/atalho a executar no fim do quadro.
    pub commands: Vec<Command>,
    /// Tela inicial ou editor.
    pub screen: Screen,
    pub home: Home,
    /// Já houve um documento aberto nesta sessão (novo, arquivo ou exemplo).
    pub opened: bool,
    pub prefs: Prefs,
    /// Onde gravar as preferências (`None`: não gravar, como nos testes).
    pub prefs_path: Option<PathBuf>,
    /// Arranjo deste quadro (computador, tablet ou celular).
    pub layout: Layout,
    /// Área da janela neste quadro.
    pub viewport: Rect,
    /// Alvos maiores no canvas (arranjo de toque ou tela sensível ao toque).
    pub touch: bool,
    /// Modo de seleção múltipla (toque): tocar soma à seleção e arrastar
    /// no vazio desenha o retângulo.
    pub multi_select: bool,
    /// Pinça em curso: o canvas ignora o dedo até todos saírem.
    pub touch_lock: bool,
    /// Painel de resultado do tablet aberto ou fechado pelo usuário
    /// (`None`: depende da largura).
    pub result_panel: Option<bool>,
    /// Folha de resultado aberta (celular).
    pub result_sheet: bool,
    /// Caminho ou laço do Mason fixado com um toque (rótulo, ex.: "L1 = …").
    pub pinned: Option<String>,
    /// Lista "Abrir…" aberta, com os diagramas da pasta do app (Android,
    /// onde não há diálogo nativo).
    pub browse: Option<Vec<PathBuf>>,
    /// Bordas cobertas pelo sistema (barras, recorte, teclado), em pontos.
    /// Zero no computador.
    pub safe_area: SafeArea,
    /// Véu escuro atrás das barras do sistema (Android abaixo da API 30 no
    /// tema claro, onde os ícones das barras não podem ficar escuros).
    pub bar_scrim: bool,
    /// Recuos simulados no computador (`--safe-area`, para capturas).
    pub simulated_safe_area: Option<SafeArea>,
    frame: u64,
}

impl App {
    pub fn new(
        ctx: &egui::Context,
        diagram: Diagram,
        screenshot: Option<PathBuf>,
        mode: Mode,
    ) -> Self {
        let _ = ctx;
        App {
            saved: diagram.clone(),
            path: None,
            pending: None,
            error: None,
            notice: None,
            allow_close: false,
            title: String::new(),
            commands: Vec::new(),
            doc: Document::new(diagram),
            view: View::default(),
            sel: Selection::default(),
            interaction: Interaction::Idle,
            editing: None,
            solver: Solver::default(),
            result: ResultState::default(),
            mode,
            snap: true,
            show_grid: true,
            highlight: Vec::new(),
            context: None,
            context_world: egui::Pos2::ZERO,
            pending_fit: true,
            canvas_rect: Rect::NOTHING,
            show_shortcuts: false,
            cancelled: false,
            screenshot: screenshot.map(|path| ScreenshotPlan {
                path,
                requested: false,
            }),
            screen: Screen::Editor,
            home: Home::default(),
            opened: true,
            prefs: Prefs::default(),
            prefs_path: None,
            layout: Layout::Desktop,
            viewport: Rect::NOTHING,
            touch: false,
            multi_select: false,
            touch_lock: false,
            result_panel: None,
            result_sheet: false,
            pinned: None,
            browse: None,
            safe_area: SafeArea::default(),
            bar_scrim: false,
            simulated_safe_area: None,
            frame: 0,
        }
    }

    /// Abre o programa na tela inicial, sem documento.
    pub fn show_home(&mut self) {
        self.screen = Screen::Home;
        self.opened = false;
    }

    /// Lê as preferências (tema e recentes) e passa a gravá-las em `path`.
    pub fn load_prefs(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.prefs = Prefs::load(&path);
        self.prefs_path = Some(path);
        ctx.set_theme(self.prefs.theme);
    }

    pub fn set_theme(&mut self, ctx: &egui::Context, theme: ThemePreference) {
        ctx.set_theme(theme);
        self.prefs.theme = theme;
        self.save_prefs();
    }

    /// Põe o arquivo no topo dos recentes.
    pub fn remember(&mut self, path: &Path) {
        let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        self.prefs.touch(&path);
        self.save_prefs();
    }

    fn save_prefs(&self) {
        if let Some(p) = &self.prefs_path
            && let Err(e) = self.prefs.save(p)
        {
            eprintln!(
                "não foi possível gravar as preferências em {}: {e}",
                p.display()
            );
        }
    }

    pub fn fit_view(&mut self) {
        if let Some(bounds) = self
            .doc
            .diagram
            .nodes
            .iter()
            .map(geometry::rect)
            .reduce(|a, b| a.union(b))
            && self.canvas_rect.is_positive()
        {
            self.view.fit(bounds, self.canvas_rect);
        }
    }

    pub fn delete_selection(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        let sel = std::mem::take(&mut self.sel);
        self.doc.edit(|d| {
            for id in &sel.nodes {
                d.remove_node(*id);
            }
            for id in &sel.wires {
                d.remove_wire(*id);
            }
        });
    }

    pub fn flip_selection(&mut self) {
        let ids = self.sel.nodes.clone();
        self.doc.edit(|d| {
            for n in d.nodes.iter_mut().filter(|n| ids.contains(&n.id)) {
                n.flipped = !n.flipped;
            }
        });
    }

    pub fn select_all(&mut self) {
        self.sel.nodes = self.doc.diagram.nodes.iter().map(|n| n.id).collect();
        self.sel.wires = self.doc.diagram.wires.iter().map(|w| w.id).collect();
    }

    pub fn undo(&mut self) {
        self.doc.undo();
        self.after_history_jump();
    }

    pub fn redo(&mut self) {
        self.doc.redo();
        self.after_history_jump();
    }

    fn after_history_jump(&mut self) {
        let d = &self.doc.diagram;
        self.sel.nodes.retain(|id| d.node(*id).is_some());
        self.sel
            .wires
            .retain(|id| d.wires.iter().any(|w| w.id == *id));
        self.editing = None;
    }

    /// Começa a editar o texto de um nó (bloco, entrada ou saída).
    pub fn start_editing(&mut self, id: NodeId) {
        let Some(n) = self.doc.diagram.node(id) else {
            return;
        };
        let text = match &n.kind {
            NodeKind::Block { expr } => expr.clone(),
            NodeKind::Input { name } | NodeKind::Output { name } => name.clone(),
            _ => return,
        };
        self.editing = Some(Editing {
            node: id,
            text,
            focus_requested: false,
        });
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let cmd = |key| KeyboardShortcut::new(Modifiers::COMMAND, key);
        let cmd_shift = |key| KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, key);
        let mut redo = false;
        let mut undo = false;
        let mut fit = false;
        let mut zoom = 1.0;
        let mut delete = false;
        let mut flip = false;
        let mut select_all = false;
        let mut escape = false;
        let mut edit = false;
        let mut help = false;
        let mut commands = Vec::new();
        let editor = self.screen == Screen::Editor;
        ctx.input_mut(|i| {
            // Ctrl+Shift+S antes de Ctrl+S (o segundo também casaria).
            for (shortcut, cmd) in [
                (cmd_shift(Key::S), Command::SaveAs),
                (cmd(Key::S), Command::Save),
                (cmd(Key::O), Command::Open),
                (cmd(Key::N), Command::New),
                (cmd(Key::Q), Command::Quit),
            ] {
                let file_cmd = matches!(cmd, Command::Save | Command::SaveAs);
                if (editor || !file_cmd) && i.consume_shortcut(&shortcut) {
                    commands.push(cmd);
                }
            }
            help = i.consume_key(Modifiers::NONE, Key::F1);
            if !editor {
                return;
            }
            redo = i.consume_shortcut(&cmd_shift(Key::Z)) || i.consume_shortcut(&cmd(Key::Y));
            undo = i.consume_shortcut(&cmd(Key::Z));
            fit = i.consume_shortcut(&cmd(Key::Num0));
            if i.consume_shortcut(&cmd(Key::Equals)) || i.consume_shortcut(&cmd(Key::Plus)) {
                zoom = 1.25;
            }
            if i.consume_shortcut(&cmd(Key::Minus)) {
                zoom = 0.8;
            }
            select_all = i.consume_shortcut(&cmd(Key::A));
            delete = i.consume_key(Modifiers::NONE, Key::Delete)
                || i.consume_key(Modifiers::NONE, Key::Backspace);
            flip = i.consume_key(Modifiers::NONE, Key::F);
            edit = i.consume_key(Modifiers::NONE, Key::F2);
            escape = i.consume_key(Modifiers::NONE, Key::Escape);
        });
        self.commands.extend(commands);
        if help {
            self.show_shortcuts = true;
        }
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
        if fit {
            self.fit_view();
        }
        if zoom != 1.0 {
            let c = self.canvas_rect.center();
            self.view.zoom_at(c, zoom);
        }
        if select_all {
            self.select_all();
        }
        if delete {
            self.delete_selection();
        }
        if flip {
            self.flip_selection();
        }
        if edit && let Some(&id) = self.sel.nodes.iter().next() {
            self.start_editing(id);
        }
        if escape {
            if self.doc.in_gesture() {
                self.doc.end_gesture();
            }
            self.interaction = Interaction::Idle;
            self.sel.clear();
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let editor = self.screen == Screen::Editor;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.spacing_mut().button_padding = vec2(6.0, 4.0);
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.menu_button("Arquivo", |ui| {
                if ui.button("Tela inicial").clicked() {
                    self.screen = Screen::Home;
                }
                ui.separator();
                for (label, shortcut, cmd) in [
                    ("Novo", "Ctrl+N", Command::New),
                    ("Abrir…", "Ctrl+O", Command::Open),
                    ("Salvar", "Ctrl+S", Command::Save),
                    ("Salvar como…", "Ctrl+Shift+S", Command::SaveAs),
                ] {
                    let enabled = editor || !matches!(cmd, Command::Save | Command::SaveAs);
                    if ui
                        .add_enabled(enabled, egui::Button::new(label).shortcut_text(shortcut))
                        .clicked()
                    {
                        self.commands.push(cmd);
                    }
                }
                ui.separator();
                if ui
                    .add_enabled(editor, egui::Button::new("Exportar SVG…"))
                    .clicked()
                {
                    self.commands.push(Command::ExportSvg);
                }
                if ui
                    .add_enabled(editor, egui::Button::new("Exportar PNG…"))
                    .clicked()
                {
                    self.commands.push(Command::ExportPng);
                }
                ui.separator();
                if ui
                    .add(egui::Button::new("Sair").shortcut_text("Ctrl+Q"))
                    .clicked()
                {
                    self.commands.push(Command::Quit);
                }
            });
            ui.menu_button("Editar", |ui| {
                ui.add_enabled_ui(editor, |ui| self.edit_menu(ui));
            });
            ui.menu_button("Exibir", |ui| {
                ui.add_enabled_ui(editor, |ui| self.view_menu(ui));
            });
            ui.menu_button("Exemplos", |ui| {
                for (i, (name, _)) in examples::all().into_iter().enumerate() {
                    if ui.button(name).clicked() {
                        self.commands.push(Command::Example(i));
                    }
                }
            });
            ui.menu_button("Visualização", |ui| {
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
            ui.menu_button("Ajuda", |ui| {
                if ui
                    .add(egui::Button::new("Atalhos e comandos").shortcut_text("F1"))
                    .clicked()
                {
                    self.show_shortcuts = true;
                }
            });
        });
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_enabled(
                self.doc.can_undo(),
                egui::Button::new("Desfazer").shortcut_text("Ctrl+Z"),
            )
            .clicked()
        {
            self.undo();
        }
        if ui
            .add_enabled(
                self.doc.can_redo(),
                egui::Button::new("Refazer").shortcut_text("Ctrl+Shift+Z"),
            )
            .clicked()
        {
            self.redo();
        }
        ui.separator();
        let has_sel = !self.sel.is_empty();
        if ui
            .add_enabled(has_sel, egui::Button::new("Apagar").shortcut_text("Del"))
            .clicked()
        {
            self.delete_selection();
        }
        if ui
            .add_enabled(
                !self.sel.nodes.is_empty(),
                egui::Button::new("Espelhar").shortcut_text("F"),
            )
            .clicked()
        {
            self.flip_selection();
        }
        if ui
            .add(egui::Button::new("Selecionar tudo").shortcut_text("Ctrl+A"))
            .clicked()
        {
            self.select_all();
        }
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(egui::Button::new("Ajustar à tela").shortcut_text("Ctrl+0"))
            .clicked()
        {
            self.fit_view();
        }
        if ui
            .add(egui::Button::new("Aumentar zoom").shortcut_text("Ctrl+="))
            .clicked()
        {
            let c = self.canvas_rect.center();
            self.view.zoom_at(c, 1.25);
        }
        if ui
            .add(egui::Button::new("Diminuir zoom").shortcut_text("Ctrl+-"))
            .clicked()
        {
            let c = self.canvas_rect.center();
            self.view.zoom_at(c, 0.8);
        }
        ui.separator();
        ui.checkbox(&mut self.show_grid, "Mostrar grade");
        ui.checkbox(&mut self.snap, "Encaixar na grade (Alt solta)");
    }

    fn shortcuts_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_shortcuts;
        if self.layout.is_touch() {
            // Janela estreita e rolável: lista em vez de tabela, para caber
            // no celular.
            egui::Window::new("Gestos e atalhos")
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .max_width((self.viewport.width() - 32.0).min(460.0))
                .max_height(self.viewport.height() - 120.0)
                .vscroll(true)
                .show(ctx, |ui| {
                    for (k, v) in GESTURES {
                        ui.label(egui::RichText::new(k).strong());
                        ui.label(egui::RichText::new(v).weak());
                        ui.add_space(6.0);
                    }
                });
            self.show_shortcuts = open;
            return;
        }
        egui::Window::new("Atalhos e comandos")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Grid::new("atalhos").striped(true).show(ui, |ui| {
                    for (k, v) in [
                        ("Arrastar da paleta", "cria o elemento no canvas"),
                        (
                            "Arrastar de uma porta",
                            "liga um fio (solte em outra porta)",
                        ),
                        ("Arrastar uma entrada já ligada", "reconecta o fio"),
                        ("Clique no sinal do somador", "alterna + / −"),
                        ("Duplo clique / F2", "edita a expressão ou o nome"),
                        (
                            "Clique direito",
                            "menu do elemento (entradas do somador, etc.)",
                        ),
                        ("Arrastar no vazio", "seleção por retângulo (Shift soma)"),
                        ("Botão do meio / Espaço + arrastar", "move a vista"),
                        ("Rolagem", "move a vista"),
                        ("Ctrl + rolagem / pinça", "zoom no cursor"),
                        ("Del / Backspace", "apaga a seleção"),
                        ("F", "espelha (realimentação da direita p/ esquerda)"),
                        ("Ctrl+Z / Ctrl+Shift+Z", "desfaz / refaz"),
                        ("Ctrl+A", "seleciona tudo"),
                        ("Ctrl+0", "ajusta à tela"),
                        ("Alt (arrastando)", "solta da grade"),
                        ("Esc", "cancela / limpa a seleção"),
                        ("Ctrl+N / Ctrl+O", "novo / abrir"),
                        ("Ctrl+S / Ctrl+Shift+S", "salvar / salvar como"),
                        ("Soltar um .json na janela", "abre o arquivo"),
                        ("Ctrl+Q", "sair"),
                        ("F1", "esta lista"),
                    ] {
                        ui.strong(k);
                        ui.label(v);
                        ui.end_row();
                    }
                });
            });
        self.show_shortcuts = open;
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        if self.screen == Screen::Home {
            self.home_status(ui);
            return;
        }
        ui.horizontal_centered(|ui| {
            ui.add_space(4.0);
            let d = &self.doc.diagram;
            ui.label(format!("{} nós · {} fios", d.nodes.len(), d.wires.len()));
            ui.separator();
            ui.label(format!("zoom {:.0}%", self.view.zoom * 100.0));
            ui.separator();
            let now = ui.input(|i| i.time);
            if let Some((text, at)) = &self.notice
                && now - at < 5.0 {
                    ui.label(egui::RichText::new(text).color(ui.visuals().warn_fg_color));
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
                    return;
                }
            let hint = match &self.interaction {
                Interaction::Connecting { .. } => "Solte sobre uma porta compatível para ligar (Esc cancela)",
                Interaction::Moving { .. } => "Movendo — segure Alt para soltar da grade",
                Interaction::BoxSelect { .. } => "Selecionando…",
                Interaction::Panning => "Movendo a vista",
                Interaction::Idle => "Arraste da paleta · ligue arrastando de uma porta · clique direito para opções · Ajuda → Atalhos",
            };
            ui.label(egui::RichText::new(hint).weak());
        });
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event
                && let Some(plan) = &self.screenshot
            {
                if let Err(error) = crate::save_png(&plan.path, &image) {
                    self.error = Some(error.to_string());
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        let Some(plan) = &mut self.screenshot else {
            return;
        };
        // Espera alguns quadros e o resultado do cálculo antes de capturar.
        let ready = self.frame > 8 && (self.result.has_outcome() || self.frame > 300);
        if ready && !plan.requested {
            plan.requested = true;
            let path = plan.path.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            let _ = path;
        }
        ctx.request_repaint();
    }
}

/// Gestos do editor no tablet e no celular.
const GESTURES: [(&str, &str); 11] = [
    (
        "Tocar num elemento da paleta",
        "cria o elemento no meio da vista",
    ),
    ("Arrastar da paleta", "cria o elemento onde soltar"),
    (
        "Arrastar de uma porta",
        "liga um fio (solte sobre outra porta)",
    ),
    ("Tocar no sinal do somador", "alterna + / −"),
    ("Toque duplo", "edita a expressão do bloco ou o nome"),
    (
        "Toque longo",
        "menu do elemento (entradas do somador, etc.)",
    ),
    ("Arrastar no vazio", "move a vista"),
    (
        "Pinça com dois dedos",
        "zoom; arrastar com dois dedos move a vista",
    ),
    (
        "Seleção múltipla (canto do canvas)",
        "tocar soma à seleção; arrastar no vazio seleciona por área",
    ),
    (
        "Barra de ações (no alto)",
        "editar, espelhar, entradas do somador e apagar o que está selecionado",
    ),
    (
        "Tocar num caminho ou laço do Mason",
        "destaca no diagrama (toque de novo para soltar)",
    ),
];

impl eframe::App for App {
    /// Entrega ao egui a área segura do Android: daí em diante a `Ui` raiz,
    /// os painéis, menus e diálogos ficam dentro de `ctx.content_rect()`.
    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        if let Some(simulated) = self.simulated_safe_area {
            self.safe_area = simulated;
            _raw_input.safe_area_insets = Some(simulated.to_egui());
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // A `Ui` raiz cobre a tela toda; a interface fica na área segura
        // (`ctx.content_rect()`), e só a cor de fundo vai até as bordas. No
        // computador as duas coincidem.
        let safe = ui.ctx().content_rect();
        // Fundo da tela inteira na cor das barras (superior e inferior):
        // é o que aparece atrás das barras do sistema.
        let full = ui.ctx().viewport_rect();
        ui.painter()
            .rect_filled(full, 0.0, theme::tokens(ui.visuals()).surface);
        if safe == ui.max_rect() {
            self.frame_ui(ui);
        } else {
            ui.scope_builder(egui::UiBuilder::new().max_rect(safe), |ui| {
                self.frame_ui(ui)
            });
        }
    }
}

impl App {
    fn frame_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.frame += 1;
        // Já descontada a área segura: os pontos de quebra usam a parte
        // visível da tela.
        self.viewport = ui.max_rect();
        self.system_bar_scrims(&ctx);
        self.layout = Layout::for_width(self.viewport.width());
        self.touch = self.layout.is_touch() || ctx.input(|i| i.has_touch_screen());

        // Resultados do cálculo em segundo plano.
        if let Some(outcome) = self.solver.poll() {
            self.result.set(outcome);
        }
        let repaint = ctx.clone();
        self.solver
            .request(&self.doc.diagram, move || repaint.request_repaint());

        self.shortcuts(&ctx);

        if self.layout.is_touch() {
            // Sem barra de menus nem de status: o menu vai para o "⋮" da
            // barra superior e os avisos aparecem sobre o canvas.
            let s = ui.spacing_mut();
            s.interact_size.y = 36.0;
            s.button_padding = vec2(12.0, 8.0);
            s.item_spacing = vec2(10.0, 8.0);
            if self.screen == Screen::Home {
                self.home_ui(ui);
            } else {
                self.editor_touch_ui(ui);
            }
            self.finish_frame(&ctx);
            return;
        }

        let t = theme::tokens(ui.visuals());
        egui::Panel::top("menu")
            .exact_size(31.0)
            .frame(Frame::new().fill(t.surface).inner_margin(Margin {
                left: 8,
                right: 8,
                top: 4,
                bottom: 5,
            }))
            .show(ui, |ui| self.menu_bar(ui));
        egui::Panel::bottom("status")
            .exact_size(25.0)
            .frame(Frame::new().fill(t.surface).inner_margin(Margin::ZERO))
            .show(ui, |ui| self.status_bar(ui));
        if self.screen == Screen::Home {
            self.home_ui(ui);
        } else {
            self.editor_ui(ui);
        }
        self.finish_frame(&ctx);
    }
}

impl App {
    /// Véu atrás dos ícones claros das barras do sistema, quando eles não
    /// podem ficar escuros (ver [`App::bar_scrim`]). Fica só na faixa das
    /// barras, fora da área segura.
    fn system_bar_scrims(&self, ctx: &egui::Context) {
        if !self.bar_scrim {
            return;
        }
        let full = ctx.viewport_rect();
        let content = ctx.content_rect();
        let p = ctx.layer_painter(egui::LayerId::background());
        let veil = egui::Color32::from_black_alpha(110);
        for r in [
            Rect::from_min_max(full.min, egui::pos2(full.max.x, content.min.y)),
            Rect::from_min_max(egui::pos2(full.min.x, content.max.y), full.max),
        ] {
            if r.is_positive() {
                p.rect_filled(r, 0.0, veil);
            }
        }
    }

    /// Janelas, diálogos e comandos enfileirados (comum aos três arranjos).
    fn finish_frame(&mut self, ctx: &egui::Context) {
        self.shortcuts_window(ctx);
        self.file_dialogs_ui(ctx);
        self.window_events(ctx);
        self.run_commands(ctx);
        self.handle_screenshot(ctx);
    }
}

impl App {
    fn editor_ui(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("paleta")
            .resizable(false)
            .exact_size(124.0)
            .show(ui, |ui| self.palette_ui(ui));
        self.highlight.clear();
        egui::Panel::right("resultado")
            .resizable(true)
            .default_size(400.0)
            .min_size(280.0)
            .show(ui, |ui| self.result_ui(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(ui, |ui| self.canvas_ui(ui));
    }
}

/// Primeiro exemplo de referência, usado ao abrir o programa.
pub fn startup_diagram() -> Diagram {
    examples::reference(Sign::Plus)
}
