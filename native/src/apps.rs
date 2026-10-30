//! Apps: gallery of the analysis apps, reached from the APPS tab.
use crate::Controlab;
use crate::icons::{self, Icon};
use crate::kit::{self, fill, galley};
use crate::theme::{Palette, Weight, sans};
use eframe::egui::{self, Rect, Sense, Stroke, Ui, pos2, vec2};

pub const APPS: [(&str, Icon, &str, &str); 7] = [
    (
        "block-diagram",
        icons::S_STATE,
        "Diagrama de Blocos",
        "Editor de diagramas e redução simbólica e numérica.",
    ),
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

/// Name of an app as the gallery shows it.
pub fn title(id: &str) -> Option<&'static str> {
    APPS.iter().find(|app| app.0 == id).map(|app| app.2)
}

impl Controlab {
    pub(crate) fn apps_page(&mut self, ui: &mut Ui) {
        let p = self.palette();
        if self.layout.mode == crate::Mode::Tablet {
            egui::Panel::left("apps-rail")
                .exact_size(3.625 * crate::MEASURED_REM * self.layout.rem)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    if let Some(id) = kit::rail(ui, &p, "apps", self.layout.rem) {
                        self.page = id.into();
                    }
                });
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(p.bg))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("apps-gallery")
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.gallery(ui, &p));
            });
    }

    fn gallery(&mut self, ui: &mut Ui, p: &Palette) {
        let narrow = self.layout.css_width < 768.0;
        let pad = if narrow { 14 } else { 26 };
        let width = (ui.available_width().min(1040.0) - 2.0 * pad as f32).max(100.0);
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
                kit::paragraph(
                    ui,
                    "Apps",
                    sans(if narrow { 18.0 } else { 20.0 }, Weight::SemiBold),
                    p.text,
                );
                ui.add_space(4.0);
                kit::paragraph(
                    ui,
                    "Ferramentas de análise e projeto de sistemas de controle.",
                    sans(13.0, Weight::Regular),
                    p.muted,
                );
                ui.add_space(20.0);
                let cols = if narrow {
                    1
                } else {
                    (((width + 8.0) / 238.0).floor() as usize).clamp(1, 6)
                };
                let card_w = (width - 8.0 * (cols - 1) as f32) / cols as f32;
                for chunk in APPS.chunks(cols) {
                    let description = chunk
                        .iter()
                        .map(|app| {
                            kit::wrapped(
                                ui,
                                app.3,
                                sans(12.0, Weight::Regular),
                                p.muted,
                                card_w - 62.0,
                            )
                            .size()
                            .y
                        })
                        .fold(0.0, f32::max);
                    let height = 22.0 + 19.0 + 2.0 + description + 6.0;
                    let (row, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
                    for (j, (id, icon, title, desc)) in chunk.iter().enumerate() {
                        let rect = Rect::from_min_size(
                            pos2(row.left() + j as f32 * (card_w + 8.0), row.top()),
                            vec2(card_w, height),
                        );
                        if app_card(ui, p, rect, id, *icon, title, desc).clicked() {
                            if *id == "block-diagram" {
                                self.block_open = true;
                                #[cfg(not(target_os = "android"))]
                                ui.ctx().send_viewport_cmd_to(
                                    egui::ViewportId::from_hash_of("block-diagram"),
                                    egui::ViewportCommand::Focus,
                                );
                            } else {
                                self.page = (*id).into();
                            }
                        }
                    }
                    ui.add_space(8.0);
                }
            });
    }
}

fn app_card(
    ui: &mut Ui,
    p: &Palette,
    rect: Rect,
    id: &str,
    icon: Icon,
    title: &str,
    description: &str,
) -> egui::Response {
    let response = ui.interact(rect, ui.id().with(("app", id)), Sense::click());
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
        icon,
        p.accent,
    );
    let x = rect.left() + 12.0 + 28.0 + 10.0;
    let g = galley(ui, title, sans(13.0, Weight::SemiBold), p.text, 0.0);
    ui.painter().galley(pos2(x, rect.top() + 11.0), g, p.text);
    let d = kit::wrapped(
        ui,
        description,
        sans(12.0, Weight::Regular),
        p.muted,
        rect.right() - 12.0 - x,
    );
    ui.painter()
        .galley(pos2(x, rect.top() + 11.0 + 19.0), d, p.muted);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
