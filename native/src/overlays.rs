//! Floating layers: toast, confirmations, the LGR deductions drawer and the settings modal.
use crate::icons;
use crate::kit::{self, fill, galley, hline};
use crate::theme::{Palette, Weight, sans};
use crate::{Controlab, ToastKind};
use eframe::egui::{self, Color32, Rect, Sense, Stroke, pos2, vec2};

impl Controlab {
    pub(crate) fn overlays(&mut self, ctx: &egui::Context) {
        let p = self.palette();
        if self.page == "lgr" && self.analyses["lgr"].deductions {
            self.deductions_drawer(ctx, &p);
        }
        if self.settings {
            self.settings_modal(ctx, &p);
        }
        if let Some((title, message)) = self.confirm.clone() {
            match modal_confirm(ctx, &p, &title, &message, "Confirmar") {
                Some(true) => {
                    self.confirm = None;
                    if let Some(i) = self.pending_close.take() {
                        self.close_document(i);
                    } else if let Some(i) = self.pending_save.take() {
                        let path = self.documents[i].path.clone();
                        self.write_document(i, path, true);
                    }
                }
                Some(false) => {
                    self.confirm = None;
                    self.pending_close = None;
                    self.pending_save = None;
                }
                None => {}
            }
        }
        if self.quit_confirm {
            match modal_confirm(
                ctx,
                &p,
                "Alterações não salvas",
                "Há documentos com alterações não salvas. Deseja descartá-las e sair?",
                "Descartar e sair",
            ) {
                Some(true) => {
                    self.allow_close = true;
                    self.quit_confirm = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Some(false) => self.quit_confirm = false,
                None => {}
            }
        }
        self.toast_layer(ctx, &p);
    }

    /// `.toast`: bottom bar with an icon, 2.8 s by default.
    fn toast_layer(&mut self, ctx: &egui::Context, p: &Palette) {
        let Some((text, kind, until)) = self.toast.clone() else {
            return;
        };
        if self.now > until {
            self.toast = None;
            return;
        }
        let screen = ctx.content_rect();
        let phone = self.layout.mode == crate::Mode::Phone;
        let width = if phone {
            (screen.width() - 32.0).max(160.0)
        } else {
            (screen.width() - 108.0).max(200.0)
        };
        let text_width = (width - 65.0).max(80.0);
        let g = ctx.fonts_mut(|f| {
            let mut job = kit::job(&text, sans(14.7, Weight::SemiBold), p.text, 0.0);
            job.wrap.max_width = text_width;
            f.layout_job(job)
        });
        let height = (g.size().y + 22.0).max(46.0);
        let bottom = if phone && !self.layout.keyboard_open {
            (3.5 + if self.in_app() { 4.0 } else { 0.0 }) * crate::MEASURED_REM + 12.0
        } else if phone {
            12.0
        } else {
            43.0
        };
        egui::Area::new(egui::Id::new("toast"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos2(
                screen.center().x - width / 2.0,
                screen.bottom() - bottom - height,
            ))
            .interactable(false)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
                fill(ui, rect, 10, p.toast, Stroke::new(1.0, p.border));
                let (icon, color) = match kind {
                    ToastKind::Success => (icons::CHECK, Color32::from_rgb(16, 185, 129)),
                    ToastKind::Error => (icons::INFO, Color32::from_rgb(244, 63, 94)),
                    ToastKind::Info => (icons::INFO, Color32::from_rgb(56, 189, 248)),
                };
                icons::paint(
                    ui,
                    Rect::from_center_size(
                        pos2(rect.left() + 28.0, rect.center().y),
                        vec2(16.0, 16.0),
                    ),
                    icon,
                    color,
                );
                ui.painter().galley(
                    pos2(rect.left() + 46.0, rect.center().y - g.size().y / 2.0),
                    g,
                    p.text,
                );
            });
    }

    /// `.memorial-sidebar`: 580 px drawer over a dimmed page.
    fn deductions_drawer(&mut self, ctx: &egui::Context, p: &Palette) {
        let screen = ctx.content_rect();
        let backdrop = egui::Area::new(egui::Id::new("deductions-backdrop"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(screen.size(), Sense::click());
                ui.painter().rect_filled(
                    rect,
                    0,
                    if p.light {
                        Color32::from_rgba_unmultiplied(14, 20, 27, 115)
                    } else {
                        Color32::from_rgba_unmultiplied(3, 7, 18, 184)
                    },
                );
                response.clicked()
            });
        let width = 580.0f32.min(screen.width());
        let mut close = backdrop.inner;
        let mut filter = self.analyses["lgr"].deduction_filter;
        let result = self.analyses["lgr"].result.clone();
        egui::Area::new(egui::Id::new("deductions"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos2(screen.right() - width, screen.top()))
            .show(ctx, |ui| {
                let (rect, _) =
                    ui.allocate_exact_size(vec2(width, screen.height()), Sense::click());
                fill(ui, rect, 0, p.surface, Stroke::NONE);
                kit::vline(ui, rect.left(), rect.y_range(), p.border_strong);
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                let ui = &mut child;
                ui.spacing_mut().item_spacing.y = 0.0;
                let (head, _) = ui.allocate_exact_size(vec2(width, 86.0), Sense::hover());
                hline(ui, head.x_range(), head.bottom() - 1.0, p.border);
                let g = galley(
                    ui,
                    "Dedução Passo a Passo",
                    sans(18.0, Weight::SemiBold),
                    p.text,
                    0.0,
                );
                ui.painter()
                    .galley(pos2(head.left() + 21.0, head.top() + 17.0), g, p.text);
                let g = kit::ellipsized(
                    ui,
                    "Desenvolvimento algébrico formal até o resultado",
                    sans(13.0, Weight::Regular),
                    p.muted,
                    0.0,
                    width - 80.0,
                );
                ui.painter()
                    .galley(pos2(head.left() + 21.0, head.top() + 48.0), g, p.muted);
                let close_rect = Rect::from_min_size(
                    pos2(head.right() - 54.0, head.top() + 23.0),
                    vec2(34.0, 40.0),
                );
                let response = ui.interact(close_rect, ui.id().with("close"), Sense::click());
                fill(
                    ui,
                    close_rect,
                    6,
                    if response.hovered() {
                        p.header
                    } else {
                        p.surface
                    },
                    Stroke::new(1.0, p.border),
                );
                icons::paint(
                    ui,
                    close_rect.shrink2(vec2(8.0, 11.0)),
                    icons::X_THIN,
                    p.text,
                );
                if response.clicked() {
                    close = true;
                }
                let (nav, _) = ui.allocate_exact_size(vec2(width, 63.0), Sense::hover());
                hline(ui, nav.x_range(), nav.bottom() - 1.0, p.border);
                let mut x = nav.left() + 21.0;
                for (i, label) in [
                    "Todos", "1, 2, 3", "Passo 4", "Passo 5", "Passo 6", "Passo 7",
                ]
                .iter()
                .enumerate()
                {
                    let value = [0, 1, 4, 5, 6, 7][i];
                    let active = filter == value;
                    let g = galley(
                        ui,
                        label,
                        sans(12.5, Weight::SemiBold),
                        if active { p.accent } else { p.muted },
                        0.0,
                    );
                    let pill = Rect::from_min_size(
                        pos2(x, nav.center().y - 14.0),
                        vec2(g.size().x + 14.0, 28.0),
                    );
                    let r = ui.interact(pill, ui.id().with(("nav", i)), Sense::click());
                    if active || r.hovered() {
                        fill(
                            ui,
                            pill,
                            6,
                            if active {
                                p.accent.gamma_multiply(0.12)
                            } else {
                                p.header
                            },
                            Stroke::NONE,
                        );
                    }
                    ui.painter().galley(
                        pos2(pill.left() + 7.0, pill.center().y - g.size().y / 2.0),
                        g,
                        if active { p.accent } else { p.muted },
                    );
                    if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        filter = value;
                    }
                    x = pill.right() + 8.0;
                }
                egui::ScrollArea::vertical()
                    .id_salt("deductions-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(egui::Margin {
                                left: 22,
                                right: 22,
                                top: 20,
                                bottom: 20,
                            })
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                crate::results::lgr_deductions(
                                    ui,
                                    p,
                                    &mut self.formulas,
                                    &result,
                                    filter,
                                );
                            });
                    });
            });
        let a = self.analyses.get_mut("lgr").unwrap();
        a.deduction_filter = filter;
        if close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            a.deductions = false;
        }
    }

    /// `#modal-settings`: appearance, updates, repository and credits.
    fn settings_modal(&mut self, ctx: &egui::Context, p: &Palette) {
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("settings-backdrop"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(screen.size(), Sense::click());
                ui.painter().rect_filled(
                    rect,
                    0,
                    if p.light {
                        Color32::from_rgba_unmultiplied(14, 20, 27, 115)
                    } else {
                        Color32::from_rgba_unmultiplied(3, 7, 18, 184)
                    },
                );
                if response.clicked() {
                    self.settings = false;
                }
            });
        let width = 678.0f32.min(screen.width() - 32.0);
        let compact = screen.width() < 600.0;
        let body_pad = if compact { 16.0 } else { 24.0 };
        let mut theme_choice = None;
        let mut scale_choice = None;
        let mut check = false;
        let mut open_url = None;
        let top = (screen.height() * 0.07).clamp(16.0, 72.0);
        egui::Area::new(egui::Id::new("settings")).order(egui::Order::Foreground).fixed_pos(pos2(screen.center().x - width / 2.0, screen.top() + top)).show(ctx, |ui| {
            let shadow = egui::Shadow { offset: [0, 20], blur: 45, spread: 0, color: Color32::from_black_alpha(if p.light { 40 } else { 140 }) };
            egui::Frame::new().fill(p.surface).stroke(Stroke::new(1.0, p.border_strong)).corner_radius(6).shadow(shadow).show(ui, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 0.0;
                let (head, _) = ui.allocate_exact_size(vec2(width, 87.0), Sense::hover());
                fill(ui, head, egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, p.header, Stroke::NONE);
                hline(ui, head.x_range(), head.bottom() - 1.0, p.border);
                let icon = Rect::from_min_size(pos2(head.left() + 24.0, head.center().y - 19.0), vec2(38.0, 38.0));
                fill(ui, icon, 6, p.accent.gamma_multiply(0.12), Stroke::new(1.0, p.accent.gamma_multiply(0.3)));
                icons::paint(ui, icon.shrink(9.0), icons::GEAR, p.accent);
                let g = galley(ui, "Configurações & Sobre", sans(18.5, Weight::Bold), p.text, 0.0);
                ui.painter().galley(pos2(icon.right() + 14.0, head.center().y - 20.0), g, p.text);
                let g = kit::ellipsized(ui, concat!("ControLAB v", env!("CARGO_PKG_VERSION"), " • Engenharia de Controle"), sans(12.5, Weight::Regular), p.muted, 0.0, (width-140.0).max(80.0));
                ui.painter().galley(pos2(icon.right() + 14.0, head.center().y + 5.0), g, p.muted);
                let close = Rect::from_center_size(pos2(head.right() - 42.0, head.center().y), vec2(32.0, 32.0));
                let r = ui.interact(close, ui.id().with("close-settings"), Sense::click()).on_hover_text("Fechar");
                if r.hovered() {
                    fill(ui, close, 4, p.inset, Stroke::NONE);
                }
                icons::paint(ui, close.shrink(7.0), icons::CLEAR, p.muted);
                if r.clicked() {
                    self.settings = false;
                }
                let body_height = (screen.height() - top * 2.0 - 87.0).max(200.0);
                egui::ScrollArea::vertical().id_salt("settings-body").max_height(body_height).min_scrolled_height(body_height.min(760.0)).show(ui, |ui| {
                    egui::Frame::new().inner_margin(body_pad).show(ui, |ui| {
                        ui.set_width(width - 2.0*body_pad);
                        ui.spacing_mut().item_spacing.y = 0.0;
                        section(ui, p, None, "Aparência");
                        card(ui, p, |ui| {
                            ui.spacing_mut().item_spacing.y = 12.0;
                            row(ui, p, "Idioma do aplicativo", |ui| {
                                ui.set_max_width(170.0);
                                let mut language = "pt-BR".to_owned();
                                kit::select_with(ui, p, egui::Id::new("settings-language"), &[["pt-BR".into(), "Português (Brasil)".into()], ["en".into(), "English".into()]], &mut language, sans(15.0, Weight::Regular), 42.0);
                            });
                            kit::paragraph(ui, "A escolha de idioma é salva neste dispositivo.", sans(14.6, Weight::Regular), p.text);
                            let theme = self.prefs.theme.clone();
                            row(ui, p, "Tema", |ui| {
                                if let Some(v) = joined(ui, p, "theme", &[("light", "Claro"), ("dark", "Escuro"), ("system", "Sistema")], &theme) {
                                    theme_choice = Some(v);
                                }
                            });
                            let scale = self.prefs.scale.to_string();
                            row(ui, p, "Tamanho da interface", |ui| {
                                if let Some(v) = joined(ui, p, "scale", &[("90", "90%"), ("100", "100%"), ("110", "110%"), ("125", "125%"), ("150", "150%")], &scale) {
                                    scale_choice = v.parse::<u32>().ok();
                                }
                            });
                        });
                        ui.add_space(24.0);
                        section(ui, p, Some(icons::EXPORT), "Atualizações do Aplicativo");
                        card(ui, p, |ui| {
                            ui.allocate_ui_with_layout(vec2(ui.available_width(), 0.0), if compact{egui::Layout::top_down(egui::Align::Min)}else{egui::Layout::left_to_right(egui::Align::Center)},|ui| {
                                ui.vertical(|ui| {
                                    ui.set_max_width(if compact{ui.available_width()}else{width-48.0-30.0-190.0});
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;
                                        kit::text(ui, "Versão Instalada", sans(13.0, Weight::Regular), p.muted);
                                        badge(ui, concat!("v", env!("CARGO_PKG_VERSION")), crate::theme::mono(13.6, Weight::Bold), p.text, p.header, p.border);
                                    });
                                    ui.add_space(6.0);
                                    badge(ui, &environment(), sans(11.5, Weight::SemiBold), p.accent, p.accent.gamma_multiply(0.1), p.accent.gamma_multiply(0.35));
                                    ui.add_space(6.0);
                                    kit::paragraph(ui, "Consulte a versão estável mais recente publicada no GitHub Releases.", sans(13.0, Weight::Regular), p.muted);
                                });
                                ui.allocate_ui_with_layout(vec2(ui.available_width(), 40.0), if compact{egui::Layout::left_to_right(egui::Align::Center)}else{egui::Layout::right_to_left(egui::Align::Center)}, |ui| {
                                    let label = match &self.update {
                                        Some(UpdateState::Loading) => "Verificando...",
                                        Some(_) => "Verificar Novamente",
                                        None => "Verificar Atualizações",
                                    };
                                    if action_button(ui, p, icons::REFRESH, label, !matches!(self.update, Some(UpdateState::Loading))).clicked() {
                                        check = true;
                                    }
                                });
                            });
                            if let Some(state) = &self.update {
                                ui.add_space(12.0);
                                update_box(ui, p, state, &mut open_url);
                            }
                            ui.add_space(12.0);
                            let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                            hline(ui, line.x_range(), line.top(), p.border);
                            ui.add_space(10.0);
                            if link(ui, p, "Ver histórico completo de versões e notas de lançamento (Releases)").clicked() {
                                open_url = Some(format!("{REPOSITORY}/releases"));
                            }
                        });
                        ui.add_space(24.0);
                        section(ui, p, Some(icons::GITHUB), "Repositório no GitHub");
                        card(ui, p, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                kit::text(ui, "enthonyaraujo/controlab", crate::theme::mono(14.0, Weight::Bold), p.text);
                                ui.add_space(6.0);
                                badge(ui, "Open Source • MIT", sans(11.5, Weight::SemiBold), p.stable.fg, p.stable.bg, p.stable.border);
                            });
                            ui.add_space(8.0);
                            kit::paragraph(ui, "Código-fonte oficial, documentação, rastreamento de melhorias e instaladores para Linux, Windows e Android.", sans(13.0, Weight::Regular), p.muted);
                            ui.add_space(12.0);
                            ui.with_layout(if compact{egui::Layout::top_down(egui::Align::Min)}else{egui::Layout::left_to_right(egui::Align::Center)},|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                if secondary_button(ui, p, icons::GITHUB, "Acessar Repositório").clicked() {
                                    open_url = Some(REPOSITORY.to_owned());
                                }
                                if secondary_button(ui, p, icons::INFO, "Reportar Erro (Issues)").clicked() {
                                    open_url = Some(format!("{REPOSITORY}/issues"));
                                }
                            });
                        });
                        ui.add_space(24.0);
                        let half = (width - 2.0*body_pad - 10.0) / 2.0;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            for (label, value) in [("Desenvolvedor", "Enthony Araujo"), ("Licença", "MIT License")] {
                                kit::frame(p.surface, p.border, 4, egui::Margin::symmetric(12, 10)).show(ui, |ui| {
                                    ui.set_width(half - 24.0);
                                    ui.vertical(|ui| {
                                        kit::spaced(ui, &label.to_uppercase(), sans(11.5, Weight::Bold), p.muted, 0.5);
                                        ui.add_space(4.0);
                                        kit::text(ui, value, sans(13.6, Weight::SemiBold), p.text);
                                    });
                                });
                            }
                        });
                    });
                });
            });
        });
        if let Some(theme) = theme_choice {
            self.prefs.theme = theme;
            self.apply_theme(ctx);
            self.save_preferences();
        }
        if let Some(scale) = scale_choice {
            self.prefs.scale = scale;
            self.save_preferences();
        }
        if check {
            self.update = Some(UpdateState::Loading);
            self.send("updates", serde_json::json!({"action":"check_updates"}));
        }
        if let Some(url) = open_url {
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.settings = false;
        }
    }
}

const REPOSITORY: &str = "https://github.com/enthonyaraujo/controlab";

pub enum UpdateState {
    Loading,
    UpToDate,
    Available {
        version: String,
        date: String,
        url: String,
    },
    Failed(String),
}

impl UpdateState {
    /// Interprets the `check_updates` reply with the previous semantic version comparison.
    pub fn from_reply(reply: &serde_json::Value) -> Self {
        if reply["success"].as_bool() != Some(true) {
            return Self::Failed(
                reply["error"]
                    .as_str()
                    .unwrap_or("Verifique sua conexão à internet.")
                    .to_owned(),
            );
        }
        let tag = reply["tag_name"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches('v')
            .to_owned();
        let parse = |v: &str| {
            v.split('.')
                .map(|n| n.trim().parse::<u64>().unwrap_or(0))
                .collect::<Vec<_>>()
        };
        if !tag.is_empty() && parse(&tag) > parse(env!("CARGO_PKG_VERSION")) {
            let date = reply["published_at"].as_str().unwrap_or("");
            let date = if date.len() >= 10 {
                format!("{}/{}/{}", &date[8..10], &date[5..7], &date[0..4])
            } else {
                String::new()
            };
            Self::Available {
                version: tag,
                date,
                url: reply["html_url"]
                    .as_str()
                    .map_or(format!("{REPOSITORY}/releases"), str::to_owned),
            }
        } else {
            Self::UpToDate
        }
    }
}

fn environment() -> String {
    let os = if cfg!(target_os = "android") {
        "Android"
    } else if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Linux"
    };
    format!("{os} • Aplicativo nativo")
}

fn update_box(ui: &mut egui::Ui, p: &Palette, state: &UpdateState, open_url: &mut Option<String>) {
    let (tone, title, text) = match state {
        UpdateState::Loading => (
            None,
            String::new(),
            "Consultando os lançamentos mais recentes no GitHub...".to_owned(),
        ),
        UpdateState::UpToDate => (
            Some(p.stable),
            format!(
                "Você já está com a versão mais recente (v{})!",
                env!("CARGO_PKG_VERSION")
            ),
            format!(
                "Seu sistema ({}) está atualizado com as últimas melhorias de engenharia e controle.",
                environment()
            ),
        ),
        UpdateState::Available { version, date, .. } => (
            Some(p.dependent),
            format!(
                "Nova versão disponível: v{version}{}",
                if date.is_empty() {
                    String::new()
                } else {
                    format!(" ({date})")
                }
            ),
            "Uma versão mais recente do ControLAB está pronta para instalação.".to_owned(),
        ),
        UpdateState::Failed(message) => (
            Some(p.marginal),
            "Não foi possível consultar a API do GitHub".to_owned(),
            message.clone(),
        ),
    };
    let (bg, border) = tone.map_or((p.inset, p.border), |t| (t.bg, t.border));
    kit::frame(bg, border, 4, 12).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if !title.is_empty() {
            kit::paragraph(
                ui,
                &title,
                sans(13.6, Weight::Bold),
                tone.map_or(p.text, |t| t.fg),
            );
            ui.add_space(4.0);
        }
        kit::paragraph(ui, &text, sans(13.0, Weight::Regular), p.text);
        match state {
            UpdateState::Available { url, .. } => {
                ui.add_space(8.0);
                if link(ui, p, "Abrir Release no GitHub →").clicked() {
                    *open_url = Some(url.clone());
                }
            }
            UpdateState::Failed(_) => {
                ui.add_space(8.0);
                if link(ui, p, "Acessar Releases no GitHub").clicked() {
                    *open_url = Some(format!("{REPOSITORY}/releases"));
                }
            }
            _ => {}
        }
    });
}

fn card(ui: &mut egui::Ui, p: &Palette, add: impl FnOnce(&mut egui::Ui)) {
    kit::frame(p.surface, p.border, 4, 14).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui);
    });
}

fn row(ui: &mut egui::Ui, p: &Palette, label: &str, control: impl FnOnce(&mut egui::Ui)) {
    if ui.available_width() < 380.0 {
        kit::paragraph(ui, label, sans(14.0, Weight::Regular), p.text);
        ui.add_space(6.0);
        ui.horizontal(|ui| control(ui));
        return;
    }
    ui.horizontal(|ui| {
        ui.set_min_height(34.0);
        kit::text(ui, label, sans(14.0, Weight::Regular), p.text);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), control);
    });
}

/// `.appearance-seg`: joined buttons, the pressed one tinted with the accent.
fn joined(
    ui: &mut egui::Ui,
    p: &Palette,
    id: &str,
    options: &[(&str, &str)],
    value: &str,
) -> Option<String> {
    let font = sans(13.0, Weight::Regular);
    let mut widths: Vec<f32> = options
        .iter()
        .map(|(_, l)| galley(ui, l, font.clone(), p.text, 0.0).size().x + 26.0)
        .collect();
    let factor = (ui.available_width() / widths.iter().sum::<f32>()).min(1.0);
    for width in &mut widths {
        *width *= factor;
    }
    let (rect, _) = ui.allocate_exact_size(vec2(widths.iter().sum(), 34.0), Sense::hover());
    fill(ui, rect, 4, p.surface, Stroke::new(1.0, p.border));
    let mut x = rect.left();
    let mut chosen = None;
    for (i, (key, label)) in options.iter().enumerate() {
        let seg = Rect::from_min_size(pos2(x, rect.top()), vec2(widths[i], rect.height()));
        let active = *key == value;
        let response = ui.interact(seg, ui.id().with((id, i)), Sense::click());
        if active {
            fill(
                ui,
                seg.shrink(1.0),
                3,
                p.accent.gamma_multiply(0.12),
                Stroke::NONE,
            );
        } else if response.hovered() {
            fill(ui, seg.shrink(1.0), 3, p.inset, Stroke::NONE);
        }
        if i > 0 {
            kit::vline(ui, seg.left(), seg.y_range(), p.border);
        }
        let g = galley(
            ui,
            label,
            sans(
                13.0,
                if active {
                    Weight::SemiBold
                } else {
                    Weight::Regular
                },
            ),
            if active { p.accent } else { p.muted },
            0.0,
        );
        ui.painter().galley(
            seg.center() - g.size() / 2.0,
            g,
            if active { p.accent } else { p.muted },
        );
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
            && !active
        {
            chosen = Some((*key).to_owned());
        }
        x += widths[i];
    }
    chosen
}

fn badge(
    ui: &mut egui::Ui,
    text: &str,
    font: egui::FontId,
    fg: Color32,
    bg: Color32,
    border: Color32,
) {
    let g = galley(ui, text, font, fg, 0.0);
    let (rect, _) = ui.allocate_exact_size(g.size() + vec2(12.0, 6.0), Sense::hover());
    fill(ui, rect, 3, bg, Stroke::new(1.0, border));
    ui.painter().galley(rect.min + vec2(6.0, 3.0), g, fg);
}

fn action_button(
    ui: &mut egui::Ui,
    p: &Palette,
    icon: icons::Icon,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let g = galley(ui, label, sans(13.6, Weight::SemiBold), Color32::WHITE, 0.0);
    let (rect, response) = ui.allocate_exact_size(
        vec2(g.size().x + 53.0, 40.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
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
    icons::paint(
        ui,
        Rect::from_min_size(
            pos2(rect.left() + 16.0, rect.center().y - 7.5),
            vec2(15.0, 15.0),
        ),
        icon,
        Color32::WHITE,
    );
    ui.painter().galley(
        pos2(rect.left() + 38.0, rect.center().y - g.size().y / 2.0),
        g,
        Color32::WHITE,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn secondary_button(
    ui: &mut egui::Ui,
    p: &Palette,
    icon: icons::Icon,
    label: &str,
) -> egui::Response {
    let g = galley(ui, label, sans(13.0, Weight::SemiBold), p.text, 0.0);
    let (rect, response) = ui.allocate_exact_size(vec2(g.size().x + 46.0, 36.0), Sense::click());
    fill(
        ui,
        rect,
        4,
        p.header,
        Stroke::new(
            1.0,
            if response.hovered() {
                p.accent
            } else {
                p.border
            },
        ),
    );
    icons::paint(
        ui,
        Rect::from_min_size(
            pos2(rect.left() + 14.0, rect.center().y - 7.5),
            vec2(15.0, 15.0),
        ),
        icon,
        p.text,
    );
    ui.painter().galley(
        pos2(rect.left() + 35.0, rect.center().y - g.size().y / 2.0),
        g,
        p.text,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn link(ui: &mut egui::Ui, p: &Palette, label: &str) -> egui::Response {
    let g = kit::wrapped(
        ui,
        label,
        sans(13.0, Weight::SemiBold),
        p.accent,
        (ui.available_width() - 20.0).max(80.0),
    );
    let (rect, response) = ui.allocate_exact_size(g.size() + vec2(20.0, 0.0), Sense::click());
    ui.painter().galley(rect.min, g.clone(), p.accent);
    icons::paint(
        ui,
        Rect::from_min_size(
            pos2(rect.left() + g.size().x + 6.0, rect.center().y - 7.0),
            vec2(14.0, 14.0),
        ),
        icons::EXTERNAL,
        p.accent,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// `.section-label`: accent uppercase caption with an optional icon.
fn section(ui: &mut egui::Ui, p: &Palette, icon: Option<icons::Icon>, title: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if let Some(icon) = icon {
            let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            icons::paint(ui, r, icon, p.accent);
        }
        kit::spaced(
            ui,
            &title.to_uppercase(),
            sans(12.5, Weight::Bold),
            p.accent,
            0.6,
        );
    });
    ui.add_space(10.0);
}

/// `.studio-dialog`-style confirmation; returns the choice once made.
fn modal_confirm(
    ctx: &egui::Context,
    p: &Palette,
    title: &str,
    message: &str,
    confirm: &str,
) -> Option<bool> {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("confirm-backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                Rect::from_min_size(screen.min, screen.size()),
                0,
                Color32::from_black_alpha(115),
            );
            ui.allocate_exact_size(screen.size(), Sense::click());
        });
    let mut choice = None;
    egui::Area::new(egui::Id::new("confirm"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(p.surface)
                .stroke(Stroke::new(1.0, p.border_strong))
                .corner_radius(4)
                .inner_margin(22)
                .show(ui, |ui| {
                    ui.set_width(376.0f32.min((screen.width() - 64.0).max(140.0)));
                    kit::paragraph(ui, title, sans(18.0, Weight::SemiBold), p.text);
                    ui.add_space(16.0);
                    kit::paragraph(ui, message, sans(13.0, Weight::Regular), p.text);
                    ui.add_space(16.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if kit::bar_button(ui, p, confirm).clicked() {
                            choice = Some(true);
                        }
                        if kit::bar_button(ui, p, "Cancelar").clicked() {
                            choice = Some(false);
                        }
                    });
                });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        choice = Some(false);
    }
    choice
}
