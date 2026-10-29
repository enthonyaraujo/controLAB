//! Paleta lateral: arraste um item para o canvas (ou clique para criá-lo
//! no centro da vista).

use eframe::egui::{
    self, Align2, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, WidgetInfo, WidgetType,
    pos2, vec2,
};

use crate::app::App;
use crate::geometry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteItem {
    Block,
    /// Bloco que já nasce espelhado (entrada à direita, saída à esquerda),
    /// pronto para o ramo de retorno de uma malha.
    Feedback,
    Sum,
    Branch,
    Input,
    Output,
}

impl PaletteItem {
    pub const ALL: [PaletteItem; 6] = [
        PaletteItem::Block,
        PaletteItem::Feedback,
        PaletteItem::Sum,
        PaletteItem::Branch,
        PaletteItem::Input,
        PaletteItem::Output,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PaletteItem::Block => "Bloco",
            PaletteItem::Feedback => "Realimentação",
            PaletteItem::Sum => "Soma",
            PaletteItem::Branch => "Ramificação",
            PaletteItem::Input => "Entrada",
            PaletteItem::Output => "Saída",
        }
    }
}

impl App {
    pub fn palette_ui(&mut self, ui: &mut egui::Ui) {
        let touch = self.layout.is_touch();
        let n = PaletteItem::ALL.len() as f32;
        let gap = ui.spacing().item_spacing.y;
        // Tablet deitado e baixo: sem o título e com itens mais baixos,
        // para os cinco caberem sem rolar.
        let header = !touch || ui.available_height() >= n * (56.0 + gap) + 64.0;
        if header {
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Elementos").strong());
            let hint = if touch {
                "toque ou arraste"
            } else {
                "arraste para o canvas"
            };
            ui.label(egui::RichText::new(hint).weak().small());
            ui.add_space(6.0);
        }
        let h = if touch {
            ((ui.available_height() - gap * (n - 1.0)) / n).clamp(44.0, 66.0)
        } else {
            58.0
        };
        for item in PaletteItem::ALL {
            let (rect, resp) =
                ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click_and_drag());
            self.palette_cell(ui, rect, &resp, item);
        }
    }

    /// Paleta do celular: os cinco elementos lado a lado, embaixo.
    pub fn palette_strip(&mut self, ui: &mut egui::Ui) {
        let r = ui.max_rect();
        let w = r.width() / PaletteItem::ALL.len() as f32;
        for (k, item) in PaletteItem::ALL.into_iter().enumerate() {
            let cell =
                Rect::from_min_size(pos2(r.left() + w * k as f32, r.top()), vec2(w, r.height()))
                    .shrink2(vec2(2.0, 5.0));
            let resp = ui.interact(cell, ui.id().with(("paleta", k)), Sense::click_and_drag());
            self.palette_cell(ui, cell, &resp, item);
        }
    }

    /// Desenha um item da paleta e trata o arrastar (solta no canvas) e o
    /// clique/toque (cria no centro da vista).
    fn palette_cell(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        resp: &egui::Response,
        item: PaletteItem,
    ) {
        resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, item.label()));
        let v = ui.visuals();
        let fg = v.text_color();
        let bg = if resp.hovered() || resp.is_pointer_button_down_on() {
            v.widgets.hovered.weak_bg_fill
        } else {
            v.widgets.inactive.weak_bg_fill
        };
        let p = ui.painter();
        p.rect(
            rect.shrink(2.0),
            CornerRadius::same(6),
            bg,
            Stroke::NONE,
            StrokeKind::Inside,
        );
        let icon = rect.center() - vec2(0.0, 9.0);
        let s = Stroke::new(1.5, fg);
        match item {
            PaletteItem::Block => {
                let r = egui::Rect::from_center_size(icon, vec2(40.0, 20.0));
                p.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
                p.text(
                    icon,
                    Align2::CENTER_CENTER,
                    "G",
                    FontId::proportional(12.0),
                    fg,
                );
            }
            PaletteItem::Feedback => {
                // Bloco "H" com o sinal entrando pela direita e saindo pela
                // esquerda, como fica no ramo de retorno.
                let r = egui::Rect::from_center_size(icon, vec2(30.0, 20.0));
                p.rect_stroke(r, CornerRadius::same(2), s, StrokeKind::Middle);
                p.text(
                    icon,
                    Align2::CENTER_CENTER,
                    "H",
                    FontId::proportional(12.0),
                    fg,
                );
                p.line_segment([r.right_center(), r.right_center() + vec2(7.0, 0.0)], s);
                let tip = r.left_center() - vec2(8.0, 0.0);
                p.line_segment([r.left_center(), tip], s);
                p.line_segment([tip, tip + vec2(3.5, -3.0)], s);
                p.line_segment([tip, tip + vec2(3.5, 3.0)], s);
            }
            PaletteItem::Sum => {
                p.circle_stroke(icon, 10.0, s);
                let k = 7.0;
                p.line_segment(
                    [icon + vec2(-k, -k), icon + vec2(k, k)],
                    Stroke::new(1.0, fg),
                );
                p.line_segment(
                    [icon + vec2(-k, k), icon + vec2(k, -k)],
                    Stroke::new(1.0, fg),
                );
            }
            PaletteItem::Branch => {
                p.line_segment([icon - vec2(18.0, 0.0), icon + vec2(18.0, 0.0)], s);
                p.line_segment([icon, icon + vec2(0.0, 10.0)], s);
                p.circle_filled(icon, 4.0, fg);
            }
            PaletteItem::Input | PaletteItem::Output => {
                let r = egui::Rect::from_center_size(icon, vec2(34.0, 20.0));
                p.rect_stroke(r, CornerRadius::same(10), s, StrokeKind::Middle);
                let t = if item == PaletteItem::Input { "X" } else { "Y" };
                p.text(
                    icon,
                    Align2::CENTER_CENTER,
                    t,
                    FontId::proportional(12.0),
                    fg,
                );
            }
        }
        // "Ramificação" e "Realimentação" não cabem na faixa estreita do
        // celular.
        let label = match item {
            PaletteItem::Branch if rect.width() < 84.0 => "Ramif.",
            PaletteItem::Feedback if rect.width() < 92.0 => "Realim.",
            _ => item.label(),
        };
        p.text(
            rect.center() + vec2(0.0, 17.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(13.0),
            fg,
        );
        if resp.drag_started() {
            egui::DragAndDrop::set_payload(ui.ctx(), item);
        }
        if resp.clicked() {
            self.add_item_at_center(item);
        }
        if !self.layout.is_touch() {
            resp.clone().on_hover_text(format!(
                "Arraste {} para o canvas (ou clique)",
                item.label().to_lowercase()
            ));
        }
    }

    /// Cria no centro da vista; se já houver um nó ali, desce na diagonal
    /// até achar lugar (tocar várias vezes não empilha os elementos).
    pub fn add_item_at_center(&mut self, item: PaletteItem) {
        let step = vec2(2.0 * geometry::GRID, 2.0 * geometry::GRID);
        let mut at = geometry::snap_pos(self.view.to_world(self.canvas_rect.center()));
        for _ in 0..20 {
            let taken = self
                .doc
                .diagram
                .nodes
                .iter()
                .any(|n| geometry::rect(n).expand(geometry::GRID).contains(at));
            if !taken {
                break;
            }
            at += step;
        }
        self.add_item(item, at);
    }
}
