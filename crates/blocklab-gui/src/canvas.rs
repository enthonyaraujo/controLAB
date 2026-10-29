//! Canvas: teste de clique, arrastar/ligar/selecionar, zoom/pan, edição
//! de texto no nó e menu de contexto.

use std::collections::HashSet;

use blocklab_core::diagram::{
    Diagram, NodeId, NodeKind, PortDir, PortRef, Pos, Side, Sign, SumInput, WireId,
};
use eframe::egui::{
    self, Align2, CursorIcon, FontId, Id, Key, Order, PointerButton, Pos2, Rect, Sense, Stroke,
    StrokeKind, Vec2, pos2, vec2,
};

use crate::app::{App, Interaction, Selection};
use crate::draw::{self, Marks, Palette};
use crate::geometry::{self, BRANCH_R, Handle, Port};
use crate::palette::PaletteItem;

/// O que está sob o cursor (em ordem de prioridade).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Handle(Handle),
    Sign(NodeId, u8),
    Node(NodeId),
    Wire(WireId),
    Empty,
}

/// Raio (em pixels de tela) para acertar portas, sinais e fios.
const HANDLE_RADIUS: f32 = 9.0;
const WIRE_RADIUS: f32 = 5.0;
/// Com o dedo, os alvos crescem. A porta só usa o raio maior fora dos
/// nós; dentro de um, vale o raio normal (senão um somador pequeno seria
/// todo "porta" e não daria para arrastá-lo).
const TOUCH_HANDLE_RADIUS: f32 = 18.0;
const TOUCH_SIGN_RADIUS: f32 = 13.0;
const TOUCH_WIRE_RADIUS: f32 = 12.0;

impl App {
    fn hit(&self, screen: Pos2) -> Hit {
        let d = &self.doc.diagram;
        let v = &self.view;
        let world = v.to_world(screen);
        let near = |p: Pos2, r: f32| v.to_screen(p).distance(screen) <= r;
        let branch_r = if self.touch { 14.0 } else { 7.0 };
        let node = d.nodes.iter().rev().find(|n| match n.kind {
            NodeKind::Sum { .. } => {
                geometry::center(n).distance(world) <= geometry::SUM_R + 2.0 / v.zoom
            }
            NodeKind::Branch { .. } => near(geometry::center(n), (BRANCH_R * v.zoom).max(branch_r)),
            _ => geometry::rect(n).expand(2.0 / v.zoom).contains(world),
        });
        let (handle_r, sign_r, wire_r) = if self.touch {
            let handle = if node.is_some() {
                HANDLE_RADIUS
            } else {
                TOUCH_HANDLE_RADIUS
            };
            (handle, TOUCH_SIGN_RADIUS, TOUCH_WIRE_RADIUS)
        } else {
            (HANDLE_RADIUS, HANDLE_RADIUS, WIRE_RADIUS)
        };

        // Alças: a mais próxima dentro do raio.
        let best = geometry::handles(d)
            .into_iter()
            .map(|h| (v.to_screen(h.pos).distance(screen), h))
            .filter(|(dist, _)| *dist <= handle_r)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        // Alças de ramificação só valem se o mouse estiver perto do ponto
        // (elas ficam invisíveis no resto do tempo).
        if let Some((_, h)) = best {
            return Hit::Handle(h);
        }
        for n in &d.nodes {
            if let NodeKind::Sum { inputs } = &n.kind {
                for i in 0..inputs.len() as u8 {
                    if near(geometry::sign_pos(n, i), sign_r) {
                        return Hit::Sign(n.id, i);
                    }
                }
            }
        }
        if let Some(n) = node {
            return Hit::Node(n.id);
        }
        for w in &d.wires {
            if let Some(pts) = geometry::wire_points(d, w.id) {
                let pts: Vec<Pos2> = pts.into_iter().map(|p| v.to_screen(p)).collect();
                if geometry::distance_to_polyline(screen, &pts) <= wire_r {
                    return Hit::Wire(w.id);
                }
            }
        }
        Hit::Empty
    }

    /// Abandona o arrasto em andamento (o segundo dedo virou pinça).
    fn cancel_interaction(&mut self) {
        match std::mem::take(&mut self.interaction) {
            Interaction::Moving { .. } | Interaction::Connecting { .. } => {
                self.doc.cancel_gesture()
            }
            Interaction::BoxSelect { .. } | Interaction::Panning | Interaction::Idle => {}
        }
    }

    /// Pinça (zoom) e dois dedos (mover a vista). Devolve se um gesto de
    /// vários dedos está em curso: até todos os dedos saírem, o canvas
    /// ignora toques, arrastos e cliques.
    fn multi_touch(&mut self, ui: &egui::Ui, rect: Rect) -> bool {
        let was_locked = self.touch_lock;
        let (multi, touching) = ui.input(|i| (i.multi_touch(), i.any_touches()));
        match multi {
            Some(mt) if self.touch_lock || rect.contains(mt.center_pos) => {
                if !self.touch_lock {
                    self.cancel_interaction();
                    self.touch_lock = true;
                }
                self.view.zoom_at(mt.center_pos, mt.zoom_delta);
                self.view.offset += mt.translation_delta;
            }
            _ if !touching => self.touch_lock = false,
            _ => {}
        }
        was_locked || self.touch_lock
    }

    fn snapped(&self, p: Pos2, alt: bool) -> Pos2 {
        if self.snap && !alt {
            geometry::snap_pos(p)
        } else {
            p
        }
    }

    /// Cria um nó da paleta na posição `world`.
    pub fn add_item(&mut self, item: PaletteItem, world: Pos2) {
        let at = self.snapped(world, false);
        let pos = Pos::new(at.x, at.y);
        let mut created = None;
        self.doc.edit(|d| {
            created = Some(match item {
                PaletteItem::Block => d.add_block(&next_block_name(d, 'G'), pos),
                PaletteItem::Feedback => {
                    let id = d.add_block(&next_block_name(d, 'H'), pos);
                    if let Some(n) = d.node_mut(id) {
                        n.flipped = true;
                    }
                    id
                }
                PaletteItem::Sum => d.add_sum(
                    &[(Sign::Plus, Side::Left), (Sign::Minus, Side::Bottom)],
                    pos,
                ),
                PaletteItem::Branch => d.add_branch(0, pos),
                PaletteItem::Input => d.add_input("X", pos),
                PaletteItem::Output => d.add_output("Y", pos),
            });
        });
        if let Some(id) = created {
            self.sel = Selection::only_node(id);
        }
    }

    /// Liga duas alças, na ordem certa (saída → entrada). Uma entrada já
    /// ocupada tem o fio antigo substituído.
    fn connect(&mut self, a: Handle, b: Handle) {
        let (out, inp) = match (a.dir, b.dir) {
            (PortDir::Out, PortDir::In) => (a.port, b.port),
            (PortDir::In, PortDir::Out) => (b.port, a.port),
            _ => return,
        };
        if out.node == inp.node && is_branch(&self.doc.diagram, out.node) {
            return;
        }
        let ok =
            self.doc.diagram.node(out.node).is_some() && self.doc.diagram.node(inp.node).is_some();
        if !ok {
            return;
        }
        self.doc.edit(|d| {
            d.wires.retain(|w| w.to != inp);
            // Nova saída de ramificação: aumenta o número de portas antes.
            if let Some(n) = d.node_mut(out.node)
                && let NodeKind::Branch { outputs } = &mut n.kind
            {
                *outputs = (*outputs).max(out.port + 1);
            }
            let _ = d.connect(out, inp);
        });
    }

    pub fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;
        self.canvas_rect = rect;
        self.view.origin = rect.min;
        if self.pending_fit && rect.is_positive() {
            self.fit_view();
            self.pending_fit = false;
        }
        let pal = Palette::from(ui.visuals());
        painter.rect_filled(rect, 0.0, crate::theme::tokens(ui.visuals()).canvas);
        if self.show_grid {
            draw::grid(&painter, &self.view, rect, &pal);
        }

        // Ao soltar o dedo não há mais "hover": vale o último ponto tocado.
        let (pointer, alt, shift, space, press_origin, finger) = ui.input(|i| {
            (
                i.pointer.hover_pos().or(i.pointer.interact_pos()),
                i.modifiers.alt,
                i.modifiers.shift,
                i.key_down(Key::Space),
                i.pointer.press_origin(),
                i.any_touches(),
            )
        });
        // No modo de seleção múltipla, tocar soma à seleção (como o Shift).
        let shift = shift || self.multi_select;
        let hover_hit = pointer
            .filter(|p| rect.contains(*p))
            .map_or(Hit::Empty, |p| self.hit(p));

        let locked = self.multi_touch(ui, rect);
        self.zoom_and_pan(ui, &response);
        self.palette_drop(ui, &response, &painter, pointer);

        // ── Início de arrasto ──────────────────────────────────────────
        if locked {
            // Pinça em curso: nada de arrastar nós ou ligar fios.
        } else if response.drag_started_by(PointerButton::Middle)
            || (response.drag_started_by(PointerButton::Primary) && space)
        {
            self.interaction = Interaction::Panning;
        } else if response.drag_started_by(PointerButton::Primary) {
            let origin = press_origin.unwrap_or(rect.center());
            match self.hit(origin) {
                Hit::Handle(h) => self.start_connection(h),
                Hit::Node(id) | Hit::Sign(id, _) => {
                    if !self.sel.nodes.contains(&id) {
                        if !shift {
                            self.sel.clear();
                        }
                        self.sel.nodes.insert(id);
                    }
                    let originals = self
                        .doc
                        .diagram
                        .nodes
                        .iter()
                        .filter(|n| self.sel.nodes.contains(&n.id))
                        .map(|n| (n.id, n.pos))
                        .collect();
                    self.doc.begin_gesture();
                    self.interaction = Interaction::Moving {
                        grab: self.view.to_world(origin),
                        originals,
                    };
                }
                // Com o dedo, arrastar no vazio move a vista (o retângulo
                // de seleção fica para o modo de seleção múltipla).
                Hit::Wire(_) | Hit::Empty
                    if (finger || self.layout.is_touch()) && !self.multi_select =>
                {
                    self.interaction = Interaction::Panning;
                }
                Hit::Wire(_) | Hit::Empty => {
                    self.interaction = Interaction::BoxSelect {
                        start: origin,
                        additive: shift,
                    };
                }
            }
        }

        // ── Durante o arrasto ──────────────────────────────────────────
        let mut target = None;
        match self.interaction.clone() {
            Interaction::Panning => {
                self.view.offset += response.drag_delta();
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
            Interaction::Moving { grab, originals } => {
                if let Some(p) = pointer {
                    let delta = self.view.to_world(p) - grab;
                    let snap = self.snap && !alt;
                    self.doc.gesture_edit(|d| {
                        for (id, orig) in &originals {
                            if let Some(n) = d.node_mut(*id) {
                                let mut q = pos2(orig.x, orig.y) + delta;
                                if snap {
                                    q = geometry::snap_pos(q);
                                }
                                n.pos = Pos::new(q.x, q.y);
                            }
                        }
                    });
                }
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
            Interaction::Connecting { from } => {
                if let Hit::Handle(h) = hover_hit
                    && compatible(&self.doc.diagram, from, h)
                {
                    target = Some(h);
                }
                if let Some(p) = pointer {
                    self.preview_wire(&painter, from, p, target, &pal);
                }
            }
            Interaction::BoxSelect { start, .. } => {
                if let Some(p) = pointer {
                    let r = Rect::from_two_pos(start, p);
                    painter.rect(
                        r,
                        0.0,
                        pal.selected.gamma_multiply(0.12),
                        Stroke::new(1.0, pal.selected),
                        StrokeKind::Inside,
                    );
                }
            }
            Interaction::Idle => {}
        }

        // ── Fim do arrasto ─────────────────────────────────────────────
        if response.drag_stopped() {
            match std::mem::take(&mut self.interaction) {
                Interaction::Moving { .. } => self.doc.end_gesture(),
                Interaction::Connecting { from } => {
                    if let Some(t) = target {
                        self.connect(from, t);
                    }
                    self.doc.end_gesture();
                }
                Interaction::BoxSelect { start, additive } => {
                    if let Some(p) = pointer {
                        self.box_select(Rect::from_two_pos(start, p), additive);
                    }
                }
                Interaction::Panning | Interaction::Idle => {}
            }
        }

        // ── Cliques ────────────────────────────────────────────────────
        if locked {
            return self.finish_canvas(ui, &painter, rect, hover_hit, target, &pal);
        }
        if response.clicked() {
            match hover_hit {
                Hit::Sign(id, i) => self.toggle_sign(id, i),
                Hit::Node(id) => {
                    if shift {
                        if !self.sel.nodes.remove(&id) {
                            self.sel.nodes.insert(id);
                        }
                    } else {
                        self.sel = Selection::only_node(id);
                    }
                }
                Hit::Wire(id) => {
                    if !shift {
                        self.sel.clear();
                    }
                    self.sel.wires.insert(id);
                }
                Hit::Handle(_) | Hit::Empty => {
                    if !shift {
                        self.sel.clear();
                    }
                }
            }
        }
        if response.double_clicked()
            && let Hit::Node(id) = hover_hit
        {
            self.start_editing(id);
        }
        if response.secondary_clicked() {
            self.context = Some(hover_hit);
            if let Some(p) = pointer {
                self.context_world = self.view.to_world(p);
            }
            if let Hit::Node(id) = hover_hit
                && !self.sel.nodes.contains(&id)
            {
                self.sel = Selection::only_node(id);
            }
        }
        let touch = self.layout.is_touch();
        response.context_menu(|ui| {
            if touch {
                crate::mobile::touch_spacing(ui);
            }
            self.context_menu(ui)
        });

        // Cursor sobre elementos clicáveis.
        if matches!(self.interaction, Interaction::Idle) {
            match hover_hit {
                Hit::Handle(_) => ui.ctx().set_cursor_icon(CursorIcon::Crosshair),
                Hit::Sign(..) | Hit::Wire(_) => ui.ctx().set_cursor_icon(CursorIcon::PointingHand),
                Hit::Node(_) => ui.ctx().set_cursor_icon(CursorIcon::Grab),
                Hit::Empty => {}
            }
        }

        self.finish_canvas(ui, &painter, rect, hover_hit, target, &pal);
    }

    /// Desenha o diagrama e o campo de edição.
    fn finish_canvas(
        &mut self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        rect: Rect,
        hover_hit: Hit,
        target: Option<Handle>,
        pal: &Palette,
    ) {
        let hovered = match hover_hit {
            Hit::Node(id) | Hit::Sign(id, _) => Some(id),
            Hit::Handle(h) => Some(h.port.node),
            _ => None,
        };
        let marks = Marks {
            selected_nodes: Some(&self.sel.nodes),
            selected_wires: Some(&self.sel.wires),
            errors: self.result.error_nodes(),
            highlight: self.highlight.iter().copied().collect::<HashSet<_>>(),
            hovered,
            // Sem mouse não há "passar por cima": as portas do que está
            // selecionado ficam à mostra.
            handles_of: self.touch.then_some(&self.sel.nodes),
            touch: self.touch,
            all_handles: matches!(self.interaction, Interaction::Connecting { .. }),
            target,
            editing: self.editing.as_ref().map(|e| e.node),
        };
        let clipped = painter.with_clip_rect(rect);
        draw::diagram(&clipped, &self.view, &self.doc.diagram, &marks, pal);
        if self.doc.diagram.nodes.is_empty() {
            let hint = if self.layout.is_touch() {
                "Toque num elemento da paleta (ou arraste-o)\npara começar, ou abra um exemplo no menu ⋮"
            } else {
                "Arraste elementos da paleta para cá\nou escolha um diagrama em Exemplos"
            };
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                hint,
                FontId::proportional(16.0),
                pal.weak,
            );
        }

        self.editor_overlay(ui);
    }

    fn zoom_and_pan(&mut self, ui: &egui::Ui, response: &egui::Response) {
        // A pinça já foi tratada em `multi_touch`.
        if !response.hovered() || ui.input(|i| i.multi_touch().is_some()) {
            return;
        }
        let (zoom, scroll, pointer) = ui.input(|i| {
            (
                i.zoom_delta(),
                i.smooth_scroll_delta(),
                i.pointer.hover_pos(),
            )
        });
        if zoom != 1.0 {
            if let Some(p) = pointer {
                self.view.zoom_at(p, zoom);
            }
        } else if scroll != Vec2::ZERO {
            self.view.offset += scroll;
        }
    }

    /// Item arrastado da paleta e solto no canvas.
    fn palette_drop(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        painter: &egui::Painter,
        pointer: Option<Pos2>,
    ) {
        let Some(item) = egui::DragAndDrop::payload::<PaletteItem>(ui.ctx()) else {
            return;
        };
        let Some(p) = pointer else {
            return;
        };
        if response.rect.contains(p) {
            // Prévia: um contorno onde o elemento vai cair.
            let at = self
                .view
                .to_screen(self.snapped(self.view.to_world(p), false));
            painter.circle_stroke(
                at,
                10.0,
                Stroke::new(1.5, ui.visuals().selection.stroke.color),
            );
            painter.text(
                at + egui::vec2(14.0, -14.0),
                Align2::LEFT_BOTTOM,
                item.label(),
                FontId::proportional(13.0),
                ui.visuals().text_color(),
            );
        }
        if ui.input(|i| i.pointer.any_released()) {
            egui::DragAndDrop::clear_payload(ui.ctx());
            if response.rect.contains(p) {
                let world = self.view.to_world(p);
                self.add_item(*item, world);
            }
        }
    }

    fn start_connection(&mut self, h: Handle) {
        self.doc.begin_gesture();
        // Puxar uma entrada já ligada "solta" o fio e reconecta a partir da
        // saída de origem.
        if h.dir == PortDir::In {
            let existing = self.doc.diagram.wire_into(h.port).copied();
            if let Some(w) = existing {
                self.doc.gesture_edit(|d| d.remove_wire(w.id));
                let pos = self
                    .doc
                    .diagram
                    .node(w.from.node)
                    .map(|n| geometry::out_port(n, w.from.port).pos)
                    .unwrap_or(h.pos);
                self.interaction = Interaction::Connecting {
                    from: Handle {
                        port: w.from,
                        dir: PortDir::Out,
                        pos,
                    },
                };
                return;
            }
        }
        self.interaction = Interaction::Connecting { from: h };
    }

    fn preview_wire(
        &self,
        painter: &egui::Painter,
        from: Handle,
        pointer: Pos2,
        target: Option<Handle>,
        pal: &Palette,
    ) {
        let d = &self.doc.diagram;
        let port_of = |h: Handle| -> Port {
            match d.node(h.port.node) {
                Some(n) if !is_branch(d, n.id) => match h.dir {
                    PortDir::Out => geometry::out_port(n, h.port.port),
                    PortDir::In => geometry::in_port(n, h.port.port),
                },
                Some(n) => Port {
                    pos: geometry::center(n),
                    normal: Vec2::ZERO,
                },
                None => Port {
                    pos: h.pos,
                    normal: Vec2::ZERO,
                },
            }
        };
        let a = port_of(from);
        let b = match target {
            Some(t) => port_of(t),
            None => Port {
                pos: self.view.to_world(pointer),
                normal: Vec2::ZERO,
            },
        };
        let (a, b, arrow) = match from.dir {
            PortDir::Out => (a, b, target.is_some()),
            PortDir::In => (b, a, true),
        };
        let pts: Vec<Pos2> = geometry::route(a, b)
            .into_iter()
            .map(|p| self.view.to_screen(p))
            .collect();
        draw::wire_path(
            painter,
            &pts,
            Stroke::new(2.0, pal.selected),
            arrow,
            self.view.zoom,
        );
    }

    fn box_select(&mut self, r: Rect, additive: bool) {
        if !additive {
            self.sel.clear();
        }
        let world = Rect::from_two_pos(self.view.to_world(r.min), self.view.to_world(r.max));
        let d = &self.doc.diagram;
        for n in &d.nodes {
            if world.intersects(geometry::rect(n)) {
                self.sel.nodes.insert(n.id);
            }
        }
        for w in &d.wires {
            let inside = geometry::wire_points(d, w.id)
                .is_some_and(|pts| pts.iter().all(|p| world.contains(*p)));
            if inside {
                self.sel.wires.insert(w.id);
            }
        }
    }

    fn toggle_sign(&mut self, id: NodeId, i: u8) {
        self.doc.edit(|d| {
            if let Some(NodeKind::Sum { inputs }) = d.node_mut(id).map(|n| &mut n.kind)
                && let Some(input) = inputs.get_mut(i as usize)
            {
                input.sign = input.sign.toggled();
            }
        });
    }

    fn context_menu(&mut self, ui: &mut egui::Ui) {
        let target = self.context.unwrap_or(Hit::Empty);
        match target {
            Hit::Node(id)
            | Hit::Sign(id, _)
            | Hit::Handle(Handle {
                port: PortRef { node: id, .. },
                ..
            }) => self.node_menu(ui, id),
            Hit::Wire(id) => {
                if ui.button("Apagar fio").clicked() {
                    self.doc.edit(|d| d.remove_wire(id));
                    self.sel.wires.remove(&id);
                    ui.close();
                }
            }
            Hit::Empty => {
                ui.label(egui::RichText::new("Adicionar aqui").weak());
                for item in PaletteItem::ALL {
                    if ui.button(item.label()).clicked() {
                        let at = self.context_world;
                        self.add_item(item, at);
                        ui.close();
                    }
                }
            }
        }
    }

    pub fn node_menu(&mut self, ui: &mut egui::Ui, id: NodeId) {
        let Some(node) = self.doc.diagram.node(id).cloned() else {
            ui.close();
            return;
        };
        ui.label(egui::RichText::new(node.describe()).weak());
        match &node.kind {
            NodeKind::Block { .. } => {
                if ui.button("Editar expressão (F2)").clicked() {
                    self.start_editing(id);
                    ui.close();
                }
            }
            NodeKind::Input { .. } | NodeKind::Output { .. } => {
                if ui.button("Renomear (F2)").clicked() {
                    self.start_editing(id);
                    ui.close();
                }
            }
            NodeKind::Sum { inputs } => {
                if ui.button("Adicionar entrada").clicked() {
                    let used: Vec<Side> = inputs.iter().map(|i| i.side).collect();
                    let side = [Side::Left, Side::Bottom, Side::Top, Side::Right]
                        .into_iter()
                        .find(|s| !used.contains(s))
                        .unwrap_or(Side::Left);
                    self.edit_sum(id, |ins| {
                        ins.push(SumInput {
                            sign: Sign::Plus,
                            side,
                        })
                    });
                    ui.close();
                }
                if inputs.len() > 1 && ui.button("Remover última entrada").clicked() {
                    let last = inputs.len() as u8 - 1;
                    self.doc.edit(|d| {
                        d.wires.retain(|w| w.to != PortRef::new(id, last));
                        if let Some(NodeKind::Sum { inputs }) = d.node_mut(id).map(|n| &mut n.kind)
                        {
                            inputs.pop();
                        }
                    });
                    ui.close();
                }
                ui.separator();
                for (k, input) in inputs.iter().enumerate() {
                    let sign = match input.sign {
                        Sign::Plus => "+",
                        Sign::Minus => "−",
                    };
                    ui.menu_button(
                        format!("Entrada {} ({sign}, {})", k + 1, side_name(input.side)),
                        |ui| {
                            if ui.button("Trocar sinal").clicked() {
                                self.toggle_sign(id, k as u8);
                                ui.close();
                            }
                            ui.separator();
                            for side in [Side::Left, Side::Top, Side::Bottom, Side::Right] {
                                if ui
                                    .selectable_label(input.side == side, side_name(side))
                                    .clicked()
                                {
                                    self.edit_sum(id, |ins| ins[k].side = side);
                                    ui.close();
                                }
                            }
                        },
                    );
                }
            }
            NodeKind::Branch { .. } => {}
        }
        if !matches!(node.kind, NodeKind::Branch { .. }) && ui.button("Espelhar (F)").clicked() {
            self.doc.edit(|d| {
                if let Some(n) = d.node_mut(id) {
                    n.flipped = !n.flipped;
                }
            });
            ui.close();
        }
        ui.separator();
        if ui.button("Apagar (Del)").clicked() {
            self.doc.edit(|d| d.remove_node(id));
            self.sel.nodes.remove(&id);
            ui.close();
        }
    }

    fn edit_sum(&mut self, id: NodeId, f: impl FnOnce(&mut Vec<SumInput>)) {
        self.doc.edit(|d| {
            if let Some(NodeKind::Sum { inputs }) = d.node_mut(id).map(|n| &mut n.kind) {
                f(inputs);
            }
        });
    }

    /// Campo de texto por cima do nó em edição.
    fn editor_overlay(&mut self, ui: &egui::Ui) {
        let Some(mut ed) = self.editing.clone() else {
            return;
        };
        let Some(node) = self.doc.diagram.node(ed.node).cloned() else {
            self.editing = None;
            return;
        };
        let mut r = self.view.rect_to_screen(geometry::rect(&node));
        // O teclado virtual encolhe o canvas (área segura): traz o nó em
        // edição para a parte visível, com folga para o campo e o aviso de
        // erro embaixo dele.
        let visible = self.canvas_rect.shrink2(vec2(16.0, 12.0));
        if visible.width() > r.width() && visible.height() > r.height() + 48.0 {
            let dx = (visible.left() - r.left()).max(0.0) + (visible.right() - r.right()).min(0.0);
            let dy = (visible.top() - r.top()).max(0.0)
                + (visible.bottom() - 48.0 - r.bottom()).min(0.0);
            if dx != 0.0 || dy != 0.0 {
                self.view.offset += vec2(dx, dy);
                r = r.translate(vec2(dx, dy));
            }
        }
        let width = r.width().max(140.0);
        let mut commit = false;
        let mut cancel = false;
        egui::Area::new(Id::new("editor-no"))
            .order(Order::Foreground)
            .fixed_pos(pos2(r.center().x - width / 2.0, r.center().y - 12.0))
            .show(ui.ctx(), |ui| {
                let edit = egui::TextEdit::singleline(&mut ed.text)
                    .desired_width(width)
                    .font(FontId::proportional(15.0))
                    .horizontal_align(egui::Align::Center);
                let what = match node.kind {
                    NodeKind::Block { .. } => "Expressão do bloco",
                    _ => "Nome do sinal",
                };
                let resp = ui.add(edit).on_hover_text(what);
                if !ed.focus_requested {
                    resp.request_focus();
                    ed.focus_requested = true;
                }
                let (enter, esc) =
                    ui.input(|i| (i.key_pressed(Key::Enter), i.key_pressed(Key::Escape)));
                if esc {
                    cancel = true;
                } else if resp.lost_focus() || enter {
                    commit = true;
                }
                // Erro de sintaxe ao vivo, para blocos.
                if matches!(node.kind, NodeKind::Block { .. })
                    && let Err(e) = blocklab_core::parser::parse_rational(&ed.text)
                {
                    ui.label(
                        egui::RichText::new(e.to_string())
                            .small()
                            .color(ui.visuals().error_fg_color),
                    );
                }
            });
        if cancel {
            self.editing = None;
        } else if commit {
            self.editing = None;
            let text = ed.text.trim().to_string();
            self.doc.edit(|d| {
                if let Some(n) = d.node_mut(ed.node) {
                    match &mut n.kind {
                        NodeKind::Block { expr } => *expr = text,
                        NodeKind::Input { name } | NodeKind::Output { name }
                            if !text.is_empty() =>
                        {
                            *name = text
                        }
                        _ => {}
                    }
                }
            });
        } else {
            self.editing = Some(ed);
        }
    }
}

fn is_branch(d: &Diagram, id: NodeId) -> bool {
    d.node(id)
        .is_some_and(|n| matches!(n.kind, NodeKind::Branch { .. }))
}

/// Pode ligar `from` em `to`? Precisa ser saída↔entrada, em nós
/// diferentes ou num laço próprio que não seja de ramificação.
fn compatible(d: &Diagram, from: Handle, to: Handle) -> bool {
    if from.dir == to.dir {
        return false;
    }
    !(from.port.node == to.port.node && is_branch(d, from.port.node))
}

fn side_name(s: Side) -> &'static str {
    match s {
        Side::Left => "Esquerda",
        Side::Top => "Cima",
        Side::Bottom => "Baixo",
        Side::Right => "Direita",
    }
}

/// Próximo nome livre com o prefixo dado: "G1", "G2", ... para um bloco
/// novo, "H1", "H2", ... para um de realimentação.
fn next_block_name(d: &Diagram, prefix: char) -> String {
    let max = d
        .nodes
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Block { expr } => expr.strip_prefix(prefix)?.parse::<u32>().ok(),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    format!("{prefix}{}", max + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::examples;

    #[test]
    fn nome_do_proximo_bloco() {
        assert_eq!(next_block_name(&Diagram::new(), 'G'), "G1");
        assert_eq!(next_block_name(&examples::reference(Sign::Plus), 'G'), "G5");
        assert_eq!(next_block_name(&examples::reference(Sign::Plus), 'H'), "H3");
    }
}
