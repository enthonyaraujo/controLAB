//! Início: the `.m` Studio of the previous renderer (`studio.js`, `styles/studio.css`).
use crate::icons::{self, Icon};
use crate::kit::{self, fill, galley, hline, vline};
use crate::texfmt::list;
use crate::theme::{Palette, Weight, mono, sans};
use crate::{Controlab, Recent};
use eframe::egui::{self, Color32, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::{Value, json};

const MODULE_CARDS: [(&str, Icon, &str, &str); 6] = [
    (
        "time",
        icons::S_TIME,
        "Resposta no tempo",
        "Degrau, impulso e rampa com métricas transitórias e erro estacionário.",
    ),
    (
        "routh",
        icons::S_STAB,
        "Estabilidade",
        "Critério de Routh-Hurwitz com a tabela montada passo a passo.",
    ),
    (
        "lgr",
        icons::S_ROOT,
        "Lugar das raízes",
        "Gráfico do LGR com memorial de assíntotas, rupturas e cruzamentos.",
    ),
    (
        "frequency",
        icons::S_FREQ,
        "Resposta em frequência",
        "Bode com assíntotas, Nyquist, margens e frequências críticas.",
    ),
    (
        "controllers",
        icons::S_CTRL,
        "Controladores",
        "Projeto de P, PD, PI, avanço e atraso de fase a partir de especificações.",
    ),
    (
        "state-space",
        icons::S_STATE,
        "Espaço de estados",
        "Conversão de modelos, controlabilidade, observabilidade e observadores.",
    ),
];

pub const EXAMPLES: [(&str, &str, &str, &str); 4] = [
    (
        "Multiplicação de matrizes",
        "C = A * B",
        "multiplicacao_matrizes.m",
        "% ControLAB · multiplicação matricial\nA = [1 2; 3 4];\nB = [5 6; 7 8];\nC = A * B;\ndisp(C);\n",
    ),
    (
        "Sistema linear",
        "x = A \\ b",
        "sistema_linear.m",
        "% ControLAB · solução de A*x = b\nA = [4 -2; 1 1];\nb = [2; 3];\nx = A \\ b\n",
    ),
    (
        "Gráfico de uma função",
        "plot(x, sin(x))",
        "grafico_seno.m",
        "% ControLAB · gráfico de uma função\nx = 0:0.1:6;\nplot(x, sin(x));\n",
    ),
    (
        "Degrau de 2ª ordem",
        "step(G)",
        "degrau_segunda_ordem.m",
        "% ControLAB · resposta ao degrau de um sistema de 2ª ordem\nwn = 2;\nzeta = 0.3;\nG = tf([wn^2], [1 2*zeta*wn wn^2]);\nstep(G);\n",
    ),
];

fn origin(recent: &Recent) -> &'static str {
    if recent.kind == "script" {
        return "Script .m";
    }
    match recent.nav.as_str() {
        "time" => "Resposta no tempo",
        "routh" => "Estabilidade",
        "lgr" => "Lugar das raízes",
        "frequency" => "Frequência",
        "controllers" => "Controladores",
        "state-space" => "Espaço de estados",
        "console" => "Console",
        _ => "Arquivo",
    }
}

/// `formatRelativeTime`.
pub fn relative_time(timestamp: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64);
    let seconds = now.saturating_sub(timestamp) / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    if seconds < 60 {
        "agora".into()
    } else if minutes < 60 {
        format!("{minutes} min")
    } else if hours < 24 {
        format!("{hours} h")
    } else if days == 1 {
        "ontem".into()
    } else {
        format!("{days} d")
    }
}

impl Controlab {
    pub(crate) fn studio(&mut self, ui: &mut Ui) {
        let p = self.palette();
        egui::Panel::top("studio-ribbon")
            .exact_size(if self.layout.css_width < 768.0 {
                74.0
            } else {
                84.0
            })
            .frame(egui::Frame::NONE.fill(p.header))
            .show(ui, |ui| self.studio_ribbon(ui, &p));
        egui::Panel::top("studio-folder")
            .exact_size(33.0)
            .frame(egui::Frame::NONE.fill(p.surface))
            .show(ui, |ui| self.folder_bar(ui, &p));
        if self.layout.mode == crate::Mode::Tablet {
            egui::Panel::left("studio-rail")
                .exact_size(3.625 * crate::MEASURED_REM * self.layout.rem)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    if let Some(id) = kit::rail(ui, &p, "home", self.layout.rem) {
                        self.page = id.into();
                    }
                });
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(p.bg)
                    .inner_margin(if self.layout.css_width < 768.0 { 0 } else { 6 }),
            )
            .show(ui, |ui| {
                let side_frame = egui::Frame::NONE.fill(p.surface);
                // Below 768 px the previous Studio hid both side columns.
                let narrow = self.layout.css_width < 768.0;
                let compact = self.layout.css_width <= 1000.0;
                if !self.focus_layout && !narrow {
                    egui::Panel::left(if compact {
                        "studio-files-compact"
                    } else {
                        "studio-files"
                    })
                    .default_size(if compact { 170.0 } else { 250.0 })
                    .size_range(if compact {
                        170.0..=170.0
                    } else {
                        150.0..=450.0
                    })
                    .resizable(!compact)
                    .frame(side_frame)
                    .show(ui, |ui| self.files_panel(ui, &p));
                }
                if !self.focus_layout && !self.hide_vars && !narrow {
                    egui::Panel::right(if compact {
                        "studio-right-compact"
                    } else {
                        "studio-right"
                    })
                    .default_size(if compact { 200.0 } else { 313.0 })
                    .size_range(if compact {
                        200.0..=200.0
                    } else {
                        180.0..=520.0
                    })
                    .resizable(!compact)
                    .frame(side_frame)
                    .show(ui, |ui| self.right_column(ui, &p));
                }
                egui::Panel::bottom("studio-console")
                    .default_size(if narrow { 150.0 } else { 167.0 })
                    .size_range(
                        60.0..=(ui.available_height() * if narrow { 0.4 } else { 0.65 })
                            .clamp(60.0, 480.0),
                    )
                    .resizable(true)
                    .frame(side_frame)
                    .show(ui, |ui| self.console_panel(ui, &p));
                egui::CentralPanel::default()
                    .frame(side_frame)
                    .show(ui, |ui| self.center_panel(ui, &p));
                if narrow && (self.show_files || self.show_vars) {
                    self.studio_drawer(ui, &p);
                }
            });
    }

    fn studio_ribbon(&mut self, ui: &mut Ui, p: &Palette) {
        let rect = ui.max_rect();
        hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border);
        let compact = self.layout.css_width < 768.0;
        let inner = rect.shrink2(if compact {
            vec2(6.0, 3.0)
        } else {
            vec2(12.0, 4.0)
        });
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(egui::Layout::left_to_right(egui::Align::Min)),
        );
        egui::ScrollArea::horizontal()
            .id_salt("studio-ribbon-scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(&mut child, |ui| {
                ui.horizontal_top(|ui| {
                    let accent = p.accent;

                    kit::studio_group(ui, p, "Arquivos", |ui| {
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::NEW_SCRIPT),
                            accent,
                            "Novo script",
                            true,
                        )
                        .on_hover_text("Novo script (Ctrl+N)")
                        .clicked()
                        {
                            self.new_document();
                        }
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::OPEN),
                            accent,
                            "Abrir",
                            true,
                        )
                        .on_hover_text("Abrir script .m (Ctrl+O)")
                        .clicked()
                        {
                            self.pick(None, "open");
                        }
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::SAVE),
                            accent,
                            "Salvar",
                            !self.script_pending,
                        )
                        .on_hover_text("Salvar script (Ctrl+S)")
                        .clicked()
                        {
                            self.save_document();
                        }
                    });
                    kit::studio_group(ui, p, "Workspace", |ui| {
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::VARIABLES),
                            accent,
                            "Variáveis",
                            true,
                        )
                        .clicked()
                        {
                            self.focus_layout = false;
                            if compact {
                                self.show_files = false;
                                self.show_vars = !self.show_vars;
                            } else {
                                self.hide_vars = !self.hide_vars;
                            }
                        }
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::CLEAR),
                            accent,
                            "Limpar",
                            !self.script_pending,
                        )
                        .clicked()
                        {
                            self.clear_workspace();
                        }
                    });
                    kit::studio_group(ui, p, "Script", |ui| {
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::RUN_FILLED),
                            p.success,
                            "Executar",
                            !self.script_pending,
                        )
                        .on_hover_text("Executar script (F5)")
                        .clicked()
                        {
                            self.run_current();
                        }
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::CLEAR),
                            accent,
                            "Limpar saída",
                            true,
                        )
                        .clicked()
                        {
                            self.console.clear();
                        }
                    });
                    kit::studio_group(ui, p, "Painéis", |ui| {
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::LAYOUT),
                            accent,
                            "Layout",
                            true,
                        )
                        .clicked()
                        {
                            self.hide_vars = false;
                            self.show_files = false;
                            self.show_vars = false;
                            self.focus_layout = !self.focus_layout;
                        }
                        if kit::studio_button(
                            ui,
                            p,
                            &icons::svg(icons::HELP),
                            accent,
                            "Ajuda",
                            true,
                        )
                        .clicked()
                        {
                            self.log(crate::HELP.trim_end().into(), "output");
                        }
                    });
                });
            });
        if self.layout.css_width <= 1000.0 {
            return;
        }
        let g = galley(ui, &self.status, sans(11.0, Weight::Regular), p.muted, 0.0);
        ui.painter().galley(
            pos2(
                inner.right() - g.size().x,
                rect.center().y - g.size().y / 2.0,
            ),
            g,
            p.muted,
        );
    }

    fn folder_bar(&mut self, ui: &mut Ui, p: &Palette) {
        let rect = ui.max_rect();
        hline(ui, rect.x_range(), rect.bottom() - 1.0, p.border);
        let inner = rect.shrink2(vec2(12.0, 4.0));
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let ui = &mut child;
        ui.spacing_mut().item_spacing.x = 2.0;
        let size = vec2(26.0, 24.0);
        if kit::icon_button(
            ui,
            p,
            &icons::folder_icon("back", p),
            size,
            18.0,
            self.history_cursor > 0,
        )
        .on_hover_text("Voltar (Alt+←)")
        .clicked()
        {
            self.navigate_history(-1);
        }
        if kit::icon_button(
            ui,
            p,
            &icons::folder_icon("forward", p),
            size,
            18.0,
            self.history_cursor + 1 < self.history.len(),
        )
        .on_hover_text("Avançar (Alt+→)")
        .clicked()
        {
            self.navigate_history(1);
        }
        if kit::icon_button(
            ui,
            p,
            &icons::folder_icon("up", p),
            size,
            18.0,
            self.parent.is_some(),
        )
        .on_hover_text("Subir um nível (Alt+↑)")
        .clicked()
            && let Some(parent) = self.parent.clone()
        {
            self.send(
                "folder",
                json!({"action":"script_request","operation":"list","path":parent}),
            );
        }
        if kit::icon_button(ui, p, &icons::folder_icon("select", p), size, 18.0, true)
            .on_hover_text("Selecionar pasta")
            .clicked()
        {
            self.pick(None, "folder");
        }
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.add_space(4.0);
        if kit::bar_button(ui, p, "Atualizar").clicked() {
            self.send(
                "folder",
                json!({"action":"script_request","operation":"list","path":self.prefs.folder}),
            );
        }
        let narrow = self.layout.css_width < 768.0;
        if !narrow {
            kit::text(ui, "Pasta atual:", sans(11.0, Weight::Regular), p.muted);
        }
        let go_width = 26.0;
        let width =
            (ui.available_width() - go_width - 6.0 - if narrow { 67.0 } else { 0.0 }).max(25.0);
        let (field, _) = ui.allocate_exact_size(vec2(width, 24.0), Sense::hover());
        let edit = egui::TextEdit::singleline(&mut self.folder_input)
            .font(sans(12.0, Weight::Regular))
            .text_color(p.text)
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(4, 2)))
            .vertical_align(egui::Align::Center);
        let response = ui.put(field, edit);
        if response.has_focus() {
            ui.painter().rect_stroke(
                field,
                0,
                Stroke::new(1.0, p.accent),
                egui::StrokeKind::Inside,
            );
        }
        let submit = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if kit::bar_button(ui, p, "Ir").clicked() || submit {
            let path = self.folder_input.clone();
            self.send(
                "folder",
                json!({"action":"script_request","operation":"list","path":path}),
            );
        }
        if narrow && kit::bar_button(ui, p, "Arquivos").clicked() {
            self.show_vars = false;
            self.show_files = !self.show_files;
        }
        if ui.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft)) {
            self.navigate_history(-1);
        }
        if ui.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight)) {
            self.navigate_history(1);
        }
    }

    fn navigate_history(&mut self, offset: isize) {
        let index = self.history_cursor as isize + offset;
        if index < 0 || index as usize >= self.history.len() {
            return;
        }
        self.history_cursor = index as usize;
        let path = self.history[self.history_cursor].clone();
        self.send(
            "history",
            json!({"action":"script_request","operation":"list","path":path}),
        );
    }

    fn files_panel(&mut self, ui: &mut Ui, p: &Palette) {
        ui.spacing_mut().item_spacing.y = 0.0;
        kit::panel_header(ui, p, "Arquivos", true, |_| {});
        let (row, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::click());
        hline(ui, row.x_range(), row.bottom() - 1.0, p.border);
        let path = if self.prefs.folder.is_empty() {
            "~/Documentos/ControLAB"
        } else {
            &self.prefs.folder
        };
        let g = galley(ui, path, mono(11.0, Weight::Regular), p.muted, 0.0);
        ui.painter()
            .with_clip_rect(row.shrink2(vec2(7.0, 0.0)))
            .galley(
                pos2(row.left() + 7.0, row.center().y - g.size().y / 2.0),
                g,
                p.muted,
            );
        if response
            .on_hover_text("Clique para selecionar outra pasta de trabalho")
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            self.pick(None, "folder");
        }
        ui.add_space(3.0);
        let (search, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
        let search = search.shrink2(vec2(5.0, 0.0));
        let edit = egui::TextEdit::singleline(&mut self.file_search)
            .font(sans(12.0, Weight::Regular))
            .text_color(p.text)
            .hint_text(
                egui::RichText::new("Buscar nesta pasta")
                    .color(p.faint)
                    .font(sans(12.0, Weight::Regular)),
            )
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(3, 6)));
        ui.put(search, edit);
        hline(ui, search.x_range(), search.bottom() - 1.0, p.border);
        ui.add_space(3.0);
        let query = self.file_search.to_lowercase();
        let files: Vec<Value> = list(&self.files)
            .iter()
            .filter(|f| {
                f["name"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&query)
            })
            .cloned()
            .collect();
        egui::ScrollArea::vertical()
            .id_salt("studio-files")
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                if files.is_empty() {
                    let hint = if query.is_empty() {
                        "Nenhum script .m nesta pasta."
                    } else {
                        "Nenhum arquivo encontrado."
                    };
                    empty_hint(ui, p, hint);
                }
                for file in files {
                    let directory = file["directory"].as_bool().unwrap_or(false);
                    let name = file["name"].as_str().unwrap_or("?");
                    let (row, response) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
                    let row = row.shrink2(vec2(4.0, 0.0));
                    if response.hovered() {
                        fill(ui, row, 0, p.header, Stroke::NONE);
                    }
                    let icon = Rect::from_min_size(
                        pos2(row.left() + 5.0, row.center().y - 7.0),
                        vec2(12.0, 14.0),
                    );
                    if directory {
                        let body = Rect::from_min_size(
                            pos2(icon.left(), icon.top() + 4.0),
                            vec2(12.0, 10.0),
                        );
                        let tab = Rect::from_min_size(
                            pos2(icon.left() - 1.0, icon.top()),
                            vec2(6.0, 3.0),
                        );
                        let (fill_c, stroke_c) = (
                            Color32::from_rgb(239, 209, 131),
                            Color32::from_rgb(180, 139, 49),
                        );
                        fill(ui, tab, 0, fill_c, Stroke::new(1.0, stroke_c));
                        fill(ui, body, 0, fill_c, Stroke::new(1.0, stroke_c));
                    } else {
                        ui.painter().rect_stroke(
                            icon,
                            0,
                            Stroke::new(1.0, p.accent),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let g = galley(ui, name, sans(12.0, Weight::Regular), p.text, 0.0);
                    ui.painter().with_clip_rect(row).galley(
                        pos2(icon.right() + 8.0, row.center().y - g.size().y / 2.0),
                        g,
                        p.text,
                    );
                    let path = file["path"].clone();
                    if response
                        .on_hover_text(path.as_str().unwrap_or(""))
                        .clicked()
                    {
                        let tag = if directory { "folder" } else { "read" };
                        let operation = if directory { "list" } else { "read" };
                        self.send(
                            tag,
                            json!({"action":"script_request","operation":operation,"path":path}),
                        );
                    }
                }
            });
    }

    fn center_panel(&mut self, ui: &mut Ui, p: &Palette) {
        ui.spacing_mut().item_spacing.y = 0.0;
        let welcome = self.documents.is_empty();
        let title = match self.active.and_then(|i| self.documents.get(i)) {
            Some(d) if !welcome => format!("Editor — {}", d.name),
            _ => "Início".into(),
        };
        let mut view = None;
        let count = self.script_plots.len();
        let showing = self.show_figures;
        kit::panel_header(ui, p, &title, true, |ui| {
            if welcome {
                return;
            }
            ui.spacing_mut().item_spacing.x = 3.0;
            if view_button(ui, p, &format!("Figuras {count}"), showing)
                .on_hover_text("Abrir figuras em novas janelas")
                .clicked()
            {
                view = Some(true);
            }
            if view_button(ui, p, "Editor", !showing).clicked() {
                view = Some(false);
            }
        });
        if let Some(v) = view {
            self.show_figures = v;
        }
        if welcome {
            egui::ScrollArea::vertical()
                .id_salt("studio-welcome")
                .show(ui, |ui| self.welcome(ui, p));
            return;
        }
        self.tabs(ui, p);
        if self.active.is_none() {
            return;
        }
        let status_height = 18.0;
        let body = ui.available_height() - status_height;
        let (area, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), body.max(40.0)), Sense::hover());
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area));
        let mut cursor = (1, 1);
        if self.show_figures {
            self.figures(&mut child, p);
        } else {
            cursor = self.editor(&mut child, p);
        }
        let (bar, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), status_height), Sense::hover());
        fill(ui, bar, 0, p.header, Stroke::NONE);
        hline(ui, bar.x_range(), bar.top(), p.border);
        if let Some(d) = self.active.and_then(|i| self.documents.get(i)) {
            let name = if d.path.is_empty() {
                d.name.clone()
            } else {
                d.path.clone()
            };
            let left = format!("{name}{}", if d.dirty() { " · Não salvo" } else { "" });
            let g = galley(ui, &left, sans(10.0, Weight::Regular), p.muted, 0.0);
            ui.painter().galley(
                pos2(bar.left() + 8.0, bar.center().y - g.size().y / 2.0),
                g,
                p.muted,
            );
            let g = galley(
                ui,
                &format!("Ln {}, Col {}", cursor.0, cursor.1),
                sans(10.0, Weight::Regular),
                p.muted,
                0.0,
            );
            ui.painter().galley(
                pos2(
                    bar.right() - 8.0 - g.size().x,
                    bar.center().y - g.size().y / 2.0,
                ),
                g,
                p.muted,
            );
        }
    }

    fn tabs(&mut self, ui: &mut Ui, p: &Palette) {
        let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 29.0), Sense::hover());
        fill(ui, bar, 0, p.header, Stroke::NONE);
        hline(ui, bar.x_range(), bar.bottom() - 1.0, p.border);
        let mut x = bar.left();
        let mut activate = None;
        let mut close = None;
        for (i, d) in self.documents.iter().enumerate() {
            let selected = self.active == Some(i);
            let label = format!("{}{}", d.name, if d.dirty() { " •" } else { "" });
            let g = galley(
                ui,
                &label,
                sans(12.0, Weight::Regular),
                if selected { p.text } else { p.muted },
                0.0,
            );
            let close_g = galley(ui, "×", sans(15.0, Weight::Regular), p.muted, 0.0);
            let width = g.size().x + 18.0 + close_g.size().x + 14.0;
            let tab = Rect::from_min_size(pos2(x, bar.top()), vec2(width, bar.height() - 1.0));
            if selected {
                fill(ui, tab, 0, p.surface, Stroke::NONE);
                fill(
                    ui,
                    Rect::from_min_size(tab.min, vec2(width, 2.0)),
                    0,
                    p.accent,
                    Stroke::NONE,
                );
            }
            vline(ui, tab.right() - 1.0, tab.y_range(), p.border);
            let label_rect = Rect::from_min_size(tab.min, vec2(g.size().x + 18.0, tab.height()));
            let close_rect = Rect::from_min_max(pos2(label_rect.right(), tab.top()), tab.max);
            let r = ui
                .interact(label_rect, ui.id().with(("tab", i)), Sense::click())
                .on_hover_text(if d.path.is_empty() { &d.name } else { &d.path });
            ui.painter().galley(
                pos2(label_rect.left() + 9.0, tab.center().y - g.size().y / 2.0),
                g,
                p.text,
            );
            let c = ui
                .interact(close_rect, ui.id().with(("close", i)), Sense::click())
                .on_hover_text(format!("Fechar {}", d.name));
            ui.painter().galley(
                pos2(
                    close_rect.left() + 7.0,
                    tab.center().y - close_g.size().y / 2.0,
                ),
                close_g,
                if c.hovered() { p.text } else { p.muted },
            );
            if r.clicked() {
                activate = Some(i);
            }
            if c.clicked() {
                close = Some(i);
            }
            x += width;
        }
        if let Some(i) = activate {
            self.active = Some(i);
            self.show_figures = false;
        }
        if let Some(i) = close {
            if self.documents[i].dirty() {
                self.confirm = Some((
                    "Fechar script".into(),
                    format!(
                        "Fechar {} e descartar as alterações não salvas?",
                        self.documents[i].name
                    ),
                ));
                self.pending_close = Some(i);
            } else {
                self.close_document(i);
            }
        }
    }

    pub(crate) fn close_document(&mut self, i: usize) {
        if i >= self.documents.len() {
            return;
        }
        self.documents.remove(i);
        self.active = match self.active {
            _ if self.documents.is_empty() => None,
            Some(a) if a == i => Some(self.documents.len() - 1),
            Some(a) if a > i => Some(a - 1),
            other => other,
        };
    }

    /// Editor with line numbers; returns the cursor line and column.
    fn editor(&mut self, ui: &mut Ui, p: &Palette) -> (usize, usize) {
        let Some(i) = self.active else { return (1, 1) };
        let light = self.light;
        let pending = self.script_pending;
        let area = ui.max_rect();
        let lines = self.documents[i].source.split('\n').count().max(1);
        let digits = lines.to_string().len().max(1) as f32;
        let gutter = (digits * 7.8 + 16.0).max(36.0);
        let mut cursor = (1, 1);
        egui::ScrollArea::both()
            .id_salt(("editor", i))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let height = (lines as f32 * 21.0 + 20.0).max(area.height());
                    let (g, _) = ui.allocate_exact_size(vec2(gutter, height), Sense::hover());
                    fill(ui, g, 0, p.header, Stroke::NONE);
                    vline(ui, g.right() - 1.0, g.y_range(), p.border);
                    for n in 0..lines {
                        let label = galley(
                            ui,
                            &(n + 1).to_string(),
                            mono(13.0, Weight::Regular),
                            p.muted,
                            0.0,
                        );
                        ui.painter().galley(
                            pos2(
                                g.right() - 8.0 - label.size().x,
                                g.top() + 10.0 + n as f32 * 21.0 + (21.0 - label.size().y) / 2.0,
                            ),
                            label,
                            p.muted,
                        );
                    }
                    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap: f32| {
                        let job = crate::theme::highlight(text.as_str(), light);
                        ui.fonts_mut(|f| f.layout_job(job))
                    };
                    let output = egui::TextEdit::multiline(&mut self.documents[i].source)
                        .code_editor()
                        .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(10)))
                        .desired_width(f32::INFINITY)
                        .min_size(vec2(area.width() - gutter, height))
                        .interactive(!pending)
                        .layouter(&mut layouter)
                        .show(ui);
                    // The soft keyboard covers the lower lines: bring the cursor back into view.
                    if output.response.has_focus()
                        && kit::content_resized(ui, output.response.id)
                        && let Some(range) = output.cursor_range
                    {
                        let rect = output.galley.pos_from_cursor(range.primary);
                        ui.scroll_to_rect(
                            rect.translate(output.galley_pos.to_vec2()),
                            Some(egui::Align::Center),
                        );
                    }
                    if let Some(range) = output.cursor_range {
                        let index = range.primary.index.0;
                        let before: String = self.documents[i].source.chars().take(index).collect();
                        let line = before.matches('\n').count() + 1;
                        let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
                        cursor = (line, col);
                    }
                });
            });
        cursor
    }

    /// `ControLABFigures.render`: axes, five ticks per axis and polylines per series.
    fn figures(&mut self, ui: &mut Ui, p: &Palette) {
        let plots = self.script_plots.clone();
        let mut export = None;
        egui::ScrollArea::vertical().id_salt("studio-figures").auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::new().inner_margin(10).show(ui, |ui| {
                if plots.is_empty() {
                    kit::paragraph(ui, "Execute um script com plot(x,y) ou step(G) para visualizar uma figura.", sans(13.6, Weight::Regular), p.text);
                }
                for (index, plot) in plots.iter().enumerate() {
                    ui.push_id(index, |ui| {
                        kit::frame(Color32::TRANSPARENT, p.border, 0, 8).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let title = plot["title"].as_str().unwrap_or("Gráfico");
                            kit::text(ui, &format!("Figura {} · {title}", index + 1), sans(12.0, Weight::Regular), p.text);
                            ui.add_space(6.0);
                            let width = ui.available_width();
                            let (rect, _) = ui.allocate_exact_size(vec2(width, (width * 340.0 / 680.0).max(180.0)), Sense::hover());
                            paint_figure(ui, p, rect, plot);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if kit::bar_button(ui, p, "Salvar SVG").clicked() {
                                    export = Some((index, figure_svg(plot)));
                                }
                            });
                        });
                    });
                    ui.add_space(10.0);
                }
            });
        });
        if let Some((index, svg)) = export {
            self.pick_export(
                Some(format!("figura-{}.svg", index + 1)),
                "svg",
                svg.into_bytes(),
            );
        }
    }

    fn welcome(&mut self, ui: &mut Ui, p: &Palette) {
        let outer = ui.available_width();
        let narrow = self.layout.css_width < 768.0;
        let pad = if narrow { 14 } else { 26 };
        let width = (outer.min(1040.0) - 2.0 * pad as f32).max(100.0);
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: pad,
                right: pad,
                top: if narrow { 16 } else { 22 },
                bottom: if narrow { 20 } else { 26 },
            })
            .show(ui, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 0.0;
                let description =
                    "Analise sistemas de controle nos módulos ou escreva scripts .m no editor.";
                if width >= 760.0 {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(width - 290.0);
                            kit::paragraph(
                                ui,
                                "Bem-vindo ao ControLAB",
                                sans(20.0, Weight::SemiBold),
                                p.text,
                            );
                            ui.add_space(4.0);
                            kit::paragraph(ui, description, sans(13.0, Weight::Regular), p.muted);
                        });
                        self.welcome_actions(ui, p, false);
                    });
                } else {
                    kit::paragraph(
                        ui,
                        "Bem-vindo ao ControLAB",
                        sans(if narrow { 18.0 } else { 20.0 }, Weight::SemiBold),
                        p.text,
                    );
                    ui.add_space(4.0);
                    kit::paragraph(ui, description, sans(13.0, Weight::Regular), p.muted);
                    ui.add_space(14.0);
                    ui.horizontal(|ui| self.welcome_actions(ui, p, narrow));
                }
                ui.add_space(18.0);
                let (line, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
                hline(ui, line.x_range(), line.top(), p.border);
                ui.add_space(20.0);
                welcome_heading(ui, p, "Módulos");
                let cols = if narrow {
                    1
                } else {
                    (((width + 8.0) / 238.0).floor() as usize).clamp(1, 6)
                };
                let card_w = (width - 8.0 * (cols - 1) as f32) / cols as f32;
                for chunk in MODULE_CARDS.chunks(cols) {
                    let desc_heights: Vec<f32> = chunk
                        .iter()
                        .map(|c| {
                            kit::wrapped(
                                ui,
                                c.3,
                                sans(12.0, Weight::Regular),
                                p.muted,
                                card_w - 62.0,
                            )
                            .size()
                            .y
                        })
                        .collect();
                    let height =
                        22.0 + 19.0 + 2.0 + desc_heights.iter().copied().fold(0.0, f32::max) + 6.0;
                    let (row, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
                    for (j, (id, icon, title, desc)) in chunk.iter().enumerate() {
                        let rect = Rect::from_min_size(
                            pos2(row.left() + j as f32 * (card_w + 8.0), row.top()),
                            vec2(card_w, height),
                        );
                        let response =
                            ui.interact(rect, ui.id().with(("module", id)), Sense::click());
                        let hovered = response.hovered();
                        fill(
                            ui,
                            rect,
                            4,
                            if hovered { p.header } else { p.inset },
                            Stroke::new(1.0, if hovered { p.accent } else { p.border }),
                        );
                        icons::paint(
                            ui,
                            Rect::from_min_size(rect.min + vec2(13.0, 12.0), vec2(24.0, 24.0)),
                            *icon,
                            p.accent,
                        );
                        let tx = rect.left() + 12.0 + 28.0 + 10.0;
                        let g = galley(ui, title, sans(13.0, Weight::SemiBold), p.text, 0.0);
                        ui.painter().galley(pos2(tx, rect.top() + 11.0), g, p.text);
                        let d = kit::wrapped(
                            ui,
                            desc,
                            sans(12.0, Weight::Regular),
                            p.muted,
                            rect.right() - 12.0 - tx,
                        );
                        ui.painter()
                            .galley(pos2(tx, rect.top() + 11.0 + 19.0), d, p.muted);
                        if response
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.page = (*id).into();
                        }
                    }
                    ui.add_space(8.0);
                }
                ui.add_space(12.0);
                let two = width >= 588.0;
                let col_w = if two { (width - 28.0) / 2.0 } else { width };
                let top = ui.cursor().top();
                let mut example = None;
                let mut recent = None;
                let left = Rect::from_min_size(pos2(ui.cursor().left(), top), vec2(col_w, 400.0));
                let mut lc = ui.new_child(egui::UiBuilder::new().max_rect(left));
                lc.spacing_mut().item_spacing.y = 0.0;
                welcome_heading(&mut lc, p, "Exemplos de script");
                list_border(&mut lc, p);
                for (i, (name, code, _, _)) in EXAMPLES.iter().enumerate() {
                    if welcome_item(&mut lc, p, name, code, true).clicked() {
                        example = Some(i);
                    }
                }
                let left_bottom = lc.min_rect().bottom();
                let right = if two {
                    Rect::from_min_size(pos2(left.right() + 28.0, top), vec2(col_w, 400.0))
                } else {
                    Rect::from_min_size(pos2(left.left(), left_bottom + 20.0), vec2(col_w, 400.0))
                };
                let mut rc = ui.new_child(egui::UiBuilder::new().max_rect(right));
                rc.spacing_mut().item_spacing.y = 0.0;
                welcome_heading(&mut rc, p, "Recentes");
                list_border(&mut rc, p);
                if self.prefs.recents.is_empty() {
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(4, 10))
                        .show(&mut rc, |ui| {
                            kit::italic(
                                ui,
                                "Scripts abertos e cálculos feitos nos módulos aparecem aqui.",
                                sans(12.0, Weight::Regular),
                                p.muted,
                            );
                        });
                }
                for (i, r) in self.prefs.recents.iter().take(6).enumerate() {
                    let meta = format!("{} · {}", origin(r), relative_time(r.timestamp));
                    if welcome_item(&mut rc, p, &r.name, &meta, false)
                        .on_hover_text(if r.file.is_empty() { &r.name } else { &r.file })
                        .clicked()
                    {
                        recent = Some(i);
                    }
                }
                let bottom = lc.min_rect().bottom().max(rc.min_rect().bottom());
                ui.allocate_space(vec2(width, (bottom - top).max(0.0)));
                if let Some(i) = example {
                    self.open_example(i);
                }
                if let Some(i) = recent {
                    self.open_recent(i);
                }
            });
    }

    fn welcome_actions(&mut self, ui: &mut Ui, p: &Palette, stretch: bool) {
        ui.spacing_mut().item_spacing.x = 8.0;
        if stretch {
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("welcome-button-width"),
                    (ui.available_width() - 8.0) / 2.0,
                )
            });
        }
        if kit::welcome_button(ui, p, icons::WELCOME_NEW, "Novo script", true).clicked() {
            self.new_document();
        }
        if kit::welcome_button(ui, p, icons::WELCOME_OPEN, "Abrir arquivo", false).clicked() {
            self.pick(None, "open");
        }
        ui.data_mut(|d| d.remove_temp::<f32>(egui::Id::new("welcome-button-width")));
    }

    fn studio_drawer(&mut self, ui: &mut Ui, p: &Palette) {
        let screen = ui.max_rect();
        let files = self.show_files;
        let width = 280.0_f32.min(screen.width() * 0.85);
        let left = if files {
            screen.left()
        } else {
            screen.right() - width
        };
        let rect = Rect::from_min_size(pos2(left, screen.top()), vec2(width, screen.height()));
        egui::Area::new(egui::Id::new("studio-drawer-backdrop"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ui.ctx(), |ui| {
                let (r, response) = ui.allocate_exact_size(screen.size(), Sense::click());
                fill(ui, r, 0, Color32::from_black_alpha(30), Stroke::NONE);
                if response.clicked() {
                    self.show_files = false;
                    self.show_vars = false;
                }
            });
        egui::Area::new(egui::Id::new("studio-drawer"))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.min)
            .show(ui.ctx(), |ui| {
                ui.set_width(width);
                ui.set_height(rect.height());
                fill(ui, rect, 0, p.surface, Stroke::new(1.0, p.border_strong));
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                let title = if files { "Arquivos" } else { "Variáveis" };
                kit::panel_header(&mut child, p, title, true, |ui| {
                    if kit::link_button(ui, p, "Fechar").clicked() {
                        self.show_files = false;
                        self.show_vars = false;
                    }
                });
                if files {
                    self.files_panel(&mut child, p);
                } else {
                    self.right_column(&mut child, p);
                }
            });
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_files = false;
            self.show_vars = false;
        }
    }

    pub(crate) fn open_example(&mut self, i: usize) {
        let (_, _, name, source) = EXAMPLES[i];
        if let Some(existing) = self
            .documents
            .iter()
            .position(|d| d.path.is_empty() && d.name == name)
        {
            self.active = Some(existing);
        } else {
            self.documents.push(crate::Document {
                name: name.into(),
                path: String::new(),
                source: source.into(),
                saved: source.into(),
            });
            self.active = Some(self.documents.len() - 1);
        }
        self.show_figures = false;
        self.status = format!("Exemplo: {name} · F5 executa");
    }

    /// `handleRecentItemClick`.
    pub(crate) fn open_recent(&mut self, i: usize) {
        let Some(r) = self.prefs.recents.get(i).cloned() else {
            return;
        };
        if r.kind == "script" && !r.file.is_empty() {
            self.send(
                "read",
                json!({"action":"script_request","operation":"read","path":r.file}),
            );
        } else if self.schema.contains_key(&r.nav) {
            let field = self.expression_field(&r.nav);
            let a = self.analyses.get_mut(&r.nav).unwrap();
            if !r.expr.is_empty() {
                a.values.insert(field.into(), r.expr.clone());
                a.preview.due = Some(self.now);
            }
            self.page = r.nav;
        } else if r.nav == "console" {
            self.command = r.expr;
        }
    }

    fn console_panel(&mut self, ui: &mut Ui, p: &Palette) {
        // Panels shrink to their content; the console keeps the height it was given.
        ui.set_min_height(ui.available_height());
        ui.spacing_mut().item_spacing.y = 0.0;
        kit::panel_header(ui, p, "Console", true, |ui| {
            kit::text(
                ui,
                if self.layout.css_width < 400.0 {
                    "↑ ↓ · Enter executa"
                } else {
                    "Histórico: ↑ ↓  ·  Enter executa"
                },
                sans(12.5, Weight::Regular),
                p.muted,
            );
        });
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                // Input row: 5 px margin, 1 px rule, 5 px padding and the 20 px field.
                let logs_height = (ui.available_height() - 31.0).max(20.0);
                egui::ScrollArea::vertical()
                    .id_salt("console-logs")
                    .max_height(logs_height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 3.1;
                        for (text, kind) in &self.console {
                            let color = match *kind {
                                "error" => p.error,
                                "cmd" => p.muted,
                                _ => p.text,
                            };
                            let mut job = kit::job(text, mono(12.0, Weight::Regular), color, 0.0);
                            job.wrap.max_width = ui.available_width();
                            for section in &mut job.sections {
                                section.format.line_height = Some(18.6);
                            }
                            let g = ui.fonts_mut(|f| f.layout_job(job));
                            let (rect, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), g.size().y),
                                Sense::hover(),
                            );
                            ui.painter().galley(rect.min, g, color);
                        }
                    });
                ui.add_space(5.0);
                let (line, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                hline(ui, line.x_range(), line.top(), p.border);
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.3;
                    kit::text(ui, ">>", mono(13.6, Weight::Bold), p.accent);
                    let edit = egui::TextEdit::singleline(&mut self.command)
                        .font(mono(12.0, Weight::Regular))
                        .text_color(p.text)
                        .hint_text(
                            egui::RichText::new("digite um comando")
                                .color(p.faint)
                                .font(mono(12.0, Weight::Regular)),
                        )
                        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(0, 2)))
                        .desired_width(f32::INFINITY);
                    let response = ui.add_enabled(!self.script_pending, edit);
                    if response.has_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && self.history_index > 0
                        {
                            self.history_index -= 1;
                            self.command = self.command_history[self.history_index].clone();
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            if self.history_index + 1 < self.command_history.len() {
                                self.history_index += 1;
                                self.command = self.command_history[self.history_index].clone();
                            } else {
                                self.history_index = self.command_history.len();
                                self.command.clear();
                            }
                        }
                    }
                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && !self.command.trim().is_empty()
                    {
                        let command = std::mem::take(&mut self.command);
                        self.command_history.push(command.clone());
                        self.history_index = self.command_history.len();
                        self.execute_command(&command);
                        response.request_focus();
                    }
                });
            });
    }

    fn right_column(&mut self, ui: &mut Ui, p: &Palette) {
        let total = ui.available_height();
        let recents_height =
            (total * 0.22).min(36.0 + 34.0 * self.prefs.recents.len().max(1) as f32);
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut clear = false;
        kit::panel_header(ui, p, "Variáveis", true, |ui| {
            if kit::link_button(ui, p, "Limpar")
                .on_hover_text("Limpar variáveis do workspace")
                .clicked()
            {
                clear = true;
            }
            kit::text(
                ui,
                "Nome · Valor",
                sans(
                    if self.layout.css_width <= 1000.0 {
                        10.5
                    } else {
                        12.5
                    },
                    Weight::Regular,
                ),
                p.muted,
            );
        });
        if clear {
            self.clear_workspace();
        }
        let vars: Vec<Value> = list(&self.workspace).to_vec();
        let mut inspect = None;
        egui::ScrollArea::vertical()
            .id_salt("studio-vars")
            .max_height(total - recents_height - 25.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                if vars.is_empty() {
                    empty_hint(ui, p, "Nenhuma variável no workspace");
                }
                for v in &vars {
                    let name = v["name"].as_str().unwrap_or("");
                    let value = v["value"].as_str().unwrap_or("");
                    let (row, response) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 29.0), Sense::click());
                    if response.hovered() {
                        fill(ui, row, 0, p.header, Stroke::NONE);
                    }
                    hline(ui, row.x_range(), row.bottom() - 1.0, p.border);
                    let name_w = ((row.width() - 21.0) * 0.375).max(50.0);
                    let g = galley(ui, name, mono(11.0, Weight::Regular), p.text, 0.0);
                    ui.painter()
                        .with_clip_rect(Rect::from_min_size(
                            row.min,
                            vec2(name_w + 7.0, row.height()),
                        ))
                        .galley(
                            pos2(row.left() + 7.0, row.center().y - g.size().y / 2.0),
                            g,
                            p.text,
                        );
                    let g = galley(ui, value, mono(11.0, Weight::Regular), p.muted, 0.0);
                    ui.painter().with_clip_rect(row.shrink(1.0)).galley(
                        pos2(
                            row.left() + 14.0 + name_w,
                            row.center().y - g.size().y / 2.0,
                        ),
                        g,
                        p.muted,
                    );
                    let tip = format!(
                        "{name} · {} · {}\n{value}",
                        v["type"].as_str().unwrap_or(""),
                        v["dimensions"].as_str().unwrap_or("")
                    );
                    if response.on_hover_text(tip).clicked() {
                        inspect = Some(name.to_owned());
                    }
                }
            });
        if let Some(name) = inspect {
            self.execute_command(&name);
        }
        let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        hline(ui, line.x_range(), line.top(), p.border);
        let mut clear_recent = false;
        kit::panel_header(ui, p, "Recentes", true, |ui| {
            if kit::link_button(ui, p, "Limpar")
                .on_hover_text("Limpar histórico recente")
                .clicked()
            {
                clear_recent = true;
            }
        });
        if clear_recent {
            self.prefs.recents.clear();
        }
        let mut open = None;
        egui::ScrollArea::vertical()
            .id_salt("studio-recents")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                if self.prefs.recents.is_empty() {
                    empty_hint(ui, p, "Nenhum item recente");
                }
                for (i, r) in self.prefs.recents.iter().take(8).enumerate() {
                    let time = relative_time(r.timestamp);
                    let tg = galley(ui, &time, sans(12.5, Weight::Regular), p.muted, 0.0);
                    let ng = kit::wrapped(
                        ui,
                        &r.name,
                        mono(13.0, Weight::Regular),
                        p.text,
                        ui.available_width() - 29.2 - tg.size().x - 12.0,
                    );
                    let (row, response) = ui.allocate_exact_size(
                        vec2(ui.available_width(), ng.size().y + 14.6),
                        Sense::click(),
                    );
                    if response.hovered() {
                        fill(ui, row, 0, p.header, Stroke::NONE);
                    }
                    ui.painter().galley(
                        pos2(row.left() + 14.6, row.center().y - ng.size().y / 2.0),
                        ng,
                        p.text,
                    );
                    ui.painter().galley(
                        pos2(
                            row.right() - 14.6 - tg.size().x,
                            row.center().y - tg.size().y / 2.0,
                        ),
                        tg,
                        p.muted,
                    );
                    if response
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        open = Some(i);
                    }
                }
            });
        if let Some(i) = open {
            self.open_recent(i);
        }
    }
}

/// `.home-empty-hint`: italic faint text.
fn empty_hint(ui: &mut Ui, p: &Palette, text: &str) {
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 15,
            right: 15,
            top: 12,
            bottom: 12,
        })
        .show(ui, |ui| {
            kit::italic(ui, text, sans(12.5, Weight::Regular), p.faint);
        });
}

fn view_button(ui: &mut Ui, p: &Palette, label: &str, pressed: bool) -> egui::Response {
    let g = galley(
        ui,
        label,
        sans(11.0, Weight::Regular),
        if pressed { p.accent } else { p.text },
        0.0,
    );
    let (rect, response) = ui.allocate_exact_size(vec2(g.size().x + 14.0, 19.0), Sense::click());
    fill(
        ui,
        rect,
        2,
        p.header,
        Stroke::new(1.0, if pressed { p.accent } else { p.border }),
    );
    ui.painter().galley(
        pos2(rect.left() + 7.0, rect.center().y - g.size().y / 2.0),
        g,
        if pressed { p.accent } else { p.text },
    );
    response
}

fn welcome_heading(ui: &mut Ui, p: &Palette, label: &str) {
    kit::spaced(
        ui,
        &label.to_uppercase(),
        sans(11.0, Weight::SemiBold),
        p.muted,
        0.66,
    );
    ui.add_space(10.0);
}

fn list_border(ui: &mut Ui, p: &Palette) {
    let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    hline(ui, line.x_range(), line.top(), p.border);
}

fn welcome_item(ui: &mut Ui, p: &Palette, name: &str, meta: &str, code: bool) -> egui::Response {
    let (row, response) = ui.allocate_exact_size(vec2(ui.available_width(), 43.0), Sense::click());
    let hovered = response.hovered();
    if hovered {
        fill(
            ui,
            Rect::from_min_max(row.min, pos2(row.right(), row.bottom() - 1.0)),
            0,
            p.header,
            Stroke::NONE,
        );
    }
    hline(ui, row.x_range(), row.bottom() - 1.0, p.border);
    let meta_font = if code {
        mono(11.0, Weight::Regular)
    } else {
        sans(11.0, Weight::Regular)
    };
    let mg = galley(ui, meta, meta_font, p.muted, 0.0);
    let mx = row.right() - 4.0 - mg.size().x.min(row.width() / 2.0);
    ui.painter()
        .with_clip_rect(Rect::from_min_max(pos2(mx, row.top()), row.max))
        .galley(pos2(mx, row.center().y - mg.size().y / 2.0), mg, p.muted);
    let ng = galley(
        ui,
        name,
        sans(13.0, Weight::Regular),
        if hovered { p.accent } else { p.text },
        0.0,
    );
    ui.painter()
        .with_clip_rect(Rect::from_min_max(row.min, pos2(mx - 12.0, row.bottom())))
        .galley(
            pos2(row.left() + 4.0, row.center().y - ng.size().y / 2.0),
            ng,
            if hovered { p.accent } else { p.text },
        );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn bounds(plot: &Value) -> Option<(f64, f64, f64, f64)> {
    let mut b = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for s in list(&plot["series"]) {
        for point in list(&s["points"]) {
            if let (Some(x), Some(y)) = (point[0].as_f64(), point[1].as_f64())
                && x.is_finite()
                && y.is_finite()
            {
                b = (b.0.min(x), b.1.max(x), b.2.min(y), b.3.max(y));
            }
        }
    }
    if !b.0.is_finite() {
        return None;
    }
    if b.0 == b.1 {
        b = (b.0 - 1.0, b.1 + 1.0, b.2, b.3);
    }
    if b.2 == b.3 {
        b = (b.0, b.1, b.2 - 1.0, b.3 + 1.0);
    }
    Some(b)
}

fn tick(v: f64) -> String {
    let s = format!("{:.4}", v);
    let n: f64 = format!("{:.4e}", v).parse().unwrap_or(v);
    let _ = s;
    let text = format!("{}", (n * 1e12).round() / 1e12);
    text.trim_end_matches(".0").to_owned()
}

/// Same geometry as `figures.js`, in a 680×340 view box scaled into `rect`.
fn paint_figure(ui: &Ui, p: &Palette, rect: Rect, plot: &Value) {
    let Some((xmin, xmax, ymin, ymax)) = bounds(plot) else {
        return;
    };
    let sx = rect.width() / 680.0;
    let sy = rect.height() / 340.0;
    let at = |x: f32, y: f32| pos2(rect.left() + x * sx, rect.top() + y * sy);
    let px = |v: f64| 62.0 + ((v - xmin) / (xmax - xmin) * 590.0) as f32;
    let py = |v: f64| 292.0 - ((v - ymin) / (ymax - ymin) * 265.0) as f32;
    let painter = ui.painter().with_clip_rect(rect);
    let axis = Stroke::new(1.0, p.border_strong);
    painter.line_segment([at(62.0, 20.0), at(62.0, 292.0)], axis);
    painter.line_segment([at(62.0, 292.0), at(652.0, 292.0)], axis);
    let font = sans(11.0 * sx.min(sy).clamp(0.6, 1.4), Weight::Regular);
    for i in 0..=4 {
        let xv = xmin + (xmax - xmin) * i as f64 / 4.0;
        let yv = ymin + (ymax - ymin) * i as f64 / 4.0;
        painter.text(
            at(px(xv), 306.0),
            egui::Align2::CENTER_CENTER,
            tick(xv),
            font.clone(),
            p.muted,
        );
        painter.text(
            at(55.0, py(yv)),
            egui::Align2::RIGHT_CENTER,
            tick(yv),
            font.clone(),
            p.muted,
        );
    }
    painter.text(
        at(357.0, 329.0),
        egui::Align2::CENTER_CENTER,
        plot["xLabel"].as_str().unwrap_or("x"),
        font.clone(),
        p.muted,
    );
    let ylabel = plot["yLabel"].as_str().unwrap_or("y");
    let g = ui.fonts_mut(|f| f.layout_no_wrap(ylabel.to_owned(), font.clone(), p.muted));
    let center = at(13.0, 158.0);
    painter.add(
        egui::epaint::TextShape::new(
            center + vec2(-g.size().y / 2.0, g.size().x / 2.0),
            g,
            p.muted,
        )
        .with_angle(-std::f32::consts::FRAC_PI_2),
    );
    for s in list(&plot["series"]) {
        let points: Vec<egui::Pos2> = list(&s["points"])
            .iter()
            .filter_map(|pt| Some((pt[0].as_f64()?, pt[1].as_f64()?)))
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .map(|(x, y)| at(px(x), py(y)))
            .collect();
        let c = list(&s["color"]);
        let color = Color32::from_rgb(
            c.first().and_then(Value::as_u64).unwrap_or(28) as u8,
            c.get(1).and_then(Value::as_u64).unwrap_or(125) as u8,
            c.get(2).and_then(Value::as_u64).unwrap_or(118) as u8,
        );
        painter.add(egui::Shape::line(points, Stroke::new(2.0, color)));
    }
}

/// Standalone SVG with explicit colors, as the previous "Salvar SVG".
fn figure_svg(plot: &Value) -> String {
    let Some((xmin, xmax, ymin, ymax)) = bounds(plot) else {
        return String::new();
    };
    let px = |v: f64| 62.0 + (v - xmin) / (xmax - xmin) * 590.0;
    let py = |v: f64| 292.0 - (v - ymin) / (ymax - ymin) * 265.0;
    let escape = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };
    let mut out = String::from(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 680 340"><path d="M62 20V292H652" fill="none" stroke="#a0acb8"/>"##,
    );
    for i in 0..=4 {
        let xv = xmin + (xmax - xmin) * i as f64 / 4.0;
        let yv = ymin + (ymax - ymin) * i as f64 / 4.0;
        out += &format!(
            r##"<text x="{}" y="310" text-anchor="middle" fill="#425466" font-size="11">{}</text>"##,
            px(xv),
            tick(xv)
        );
        out += &format!(
            r##"<text x="55" y="{}" text-anchor="end" fill="#425466" font-size="11">{}</text>"##,
            py(yv) + 4.0,
            tick(yv)
        );
    }
    out += &format!(
        r##"<text x="357" y="333" text-anchor="middle" fill="#425466" font-size="11">{}</text>"##,
        escape(plot["xLabel"].as_str().unwrap_or("x"))
    );
    out += &format!(
        r##"<text x="13" y="158" transform="rotate(-90 13 158)" text-anchor="middle" fill="#425466" font-size="11">{}</text>"##,
        escape(plot["yLabel"].as_str().unwrap_or("y"))
    );
    for s in list(&plot["series"]) {
        let points: Vec<String> = list(&s["points"])
            .iter()
            .filter_map(|pt| Some((pt[0].as_f64()?, pt[1].as_f64()?)))
            .map(|(x, y)| format!("{},{}", px(x), py(y)))
            .collect();
        let c = list(&s["color"])
            .iter()
            .filter_map(Value::as_u64)
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        out += &format!(
            r#"<polyline points="{}" fill="none" stroke="rgb({c})" stroke-width="2" stroke-linejoin="round"><title>{}</title></polyline>"#,
            points.join(" "),
            escape(s["name"].as_str().unwrap_or(""))
        );
    }
    out + "</svg>"
}
