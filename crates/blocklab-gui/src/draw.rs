//! Desenho do diagrama no canvas (só pinta; não trata entrada).

use std::collections::HashSet;

use blocklab_core::diagram::{Diagram, NodeId, NodeKind, PortDir, PortRef, Sign, WireId};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind,
    Vec2, pos2, vec2,
};

use crate::geometry::{self, GRID, Handle};
use crate::view::View;

/// Cores tiradas do tema atual (claro ou escuro).
pub struct Palette {
    pub fg: Color32,
    pub weak: Color32,
    pub fill: Color32,
    pub terminal_fill: Color32,
    pub selected: Color32,
    pub error: Color32,
    pub highlight: Color32,
    pub grid: Color32,
    pub grid_major: Color32,
}

impl Palette {
    pub fn from(v: &egui::Visuals) -> Self {
        let fg = v.text_color();
        Palette {
            fg,
            weak: v.weak_text_color(),
            fill: v.window_fill,
            terminal_fill: v.faint_bg_color,
            selected: v.selection.stroke.color,
            error: v.error_fg_color,
            highlight: v.warn_fg_color,
            grid: crate::theme::tokens(v).grid,
            grid_major: crate::theme::tokens(v).grid_major,
        }
    }
}

/// O que destacar no desenho.
#[derive(Default)]
pub struct Marks<'a> {
    pub selected_nodes: Option<&'a std::collections::BTreeSet<NodeId>>,
    pub selected_wires: Option<&'a std::collections::BTreeSet<WireId>>,
    pub errors: HashSet<NodeId>,
    pub highlight: HashSet<NodeId>,
    /// Nó sob o mouse (mostra as portas de ramificação).
    pub hovered: Option<NodeId>,
    /// Nós com as portas sempre à mostra (os selecionados, no toque).
    pub handles_of: Option<&'a std::collections::BTreeSet<NodeId>>,
    /// Interface de toque: alças maiores.
    pub touch: bool,
    /// Mostrar todas as alças de ligação (durante uma ligação).
    pub all_handles: bool,
    /// Alça compatível sob o mouse durante uma ligação.
    pub target: Option<Handle>,
    /// Nó em edição (o texto é desenhado pelo campo de edição).
    pub editing: Option<NodeId>,
}

pub fn grid(p: &Painter, view: &View, rect: Rect, pal: &Palette) {
    let step = GRID * view.zoom;
    if step < 6.0 {
        return;
    }
    let w0 = view.to_world(rect.min);
    let w1 = view.to_world(rect.max);
    let first = |v: f32| (v / GRID).floor() as i64;
    for i in first(w0.x)..=first(w1.x) + 1 {
        let x = view.to_screen(pos2(i as f32 * GRID, 0.0)).x;
        let c = if i % 5 == 0 { pal.grid_major } else { pal.grid };
        p.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.0, c),
        );
    }
    for j in first(w0.y)..=first(w1.y) + 1 {
        let y = view.to_screen(pos2(0.0, j as f32 * GRID)).y;
        let c = if j % 5 == 0 { pal.grid_major } else { pal.grid };
        p.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, c),
        );
    }
}

fn arrow_head(p: &Painter, tip: Pos2, dir: Vec2, size: f32, color: Color32) {
    let d = dir.normalized();
    let n = vec2(-d.y, d.x);
    let base = tip - d * size;
    p.add(Shape::convex_polygon(
        vec![tip, base + n * size * 0.45, base - n * size * 0.45],
        color,
        Stroke::NONE,
    ));
}

/// Fio já em coordenadas de tela, com seta na ponta (se `arrow`).
pub fn wire_path(p: &Painter, pts: &[Pos2], stroke: Stroke, arrow: bool, zoom: f32) {
    if pts.len() < 2 {
        return;
    }
    p.add(Shape::line(pts.to_vec(), stroke));
    if arrow {
        let (a, b) = (pts[pts.len() - 2], pts[pts.len() - 1]);
        if a.distance(b) > 0.5 {
            arrow_head(p, b, b - a, 9.0 * zoom.max(0.6), stroke.color);
        }
    }
}

pub fn diagram(p: &Painter, view: &View, d: &Diagram, marks: &Marks, pal: &Palette) {
    let z = view.zoom;
    let line = |c: Color32, w: f32| Stroke::new((w * z).max(1.0), c);
    let is_branch = |id: NodeId| {
        d.node(id)
            .is_some_and(|n| matches!(n.kind, NodeKind::Branch { .. }))
    };

    // Fios primeiro, para os nós ficarem por cima.
    for w in &d.wires {
        let Some(pts) = geometry::wire_points(d, w.id) else {
            continue;
        };
        let pts: Vec<Pos2> = pts.into_iter().map(|q| view.to_screen(q)).collect();
        let selected = marks.selected_wires.is_some_and(|s| s.contains(&w.id));
        let lit = marks.highlight.contains(&w.from.node) && marks.highlight.contains(&w.to.node);
        let color = if selected {
            pal.selected
        } else if lit {
            pal.highlight
        } else {
            pal.fg
        };
        let width = if selected || lit { 2.5 } else { 1.4 };
        wire_path(p, &pts, line(color, width), !is_branch(w.to.node), z);
    }

    // Portas ligadas (para marcar as soltas).
    let connected: HashSet<(PortRef, PortDir)> = d
        .wires
        .iter()
        .flat_map(|w| [(w.from, PortDir::Out), (w.to, PortDir::In)])
        .collect();

    let font = |size: f32| FontId::proportional(size * z);
    for n in &d.nodes {
        let r = view.rect_to_screen(geometry::rect(n));
        let selected = marks.selected_nodes.is_some_and(|s| s.contains(&n.id));
        let border = if marks.errors.contains(&n.id) {
            pal.error
        } else if selected {
            pal.selected
        } else {
            pal.fg
        };
        let width = if selected || marks.errors.contains(&n.id) {
            2.5
        } else {
            1.5
        };
        let fill = if marks.highlight.contains(&n.id) {
            pal.highlight.gamma_multiply(0.25)
        } else {
            pal.fill
        };
        let editing = marks.editing == Some(n.id);
        match &n.kind {
            NodeKind::Block { expr } => {
                p.rect(
                    r,
                    CornerRadius::same(2),
                    fill,
                    line(border, width),
                    StrokeKind::Middle,
                );
                if !editing {
                    p.text(r.center(), Align2::CENTER_CENTER, expr, font(15.0), pal.fg);
                }
            }
            NodeKind::Input { name } | NodeKind::Output { name } => {
                let fill = if marks.highlight.contains(&n.id) {
                    fill
                } else {
                    pal.terminal_fill
                };
                p.rect(
                    r,
                    CornerRadius::same((6.0 * z).clamp(1.0, 255.0) as u8),
                    fill,
                    line(border, width),
                    StrokeKind::Middle,
                );
                if !editing {
                    p.text(r.center(), Align2::CENTER_CENTER, name, font(15.0), pal.fg);
                }
            }
            NodeKind::Sum { inputs } => {
                let c = r.center();
                let rad = r.width() / 2.0;
                p.circle(c, rad, fill, line(border, width));
                // O "X" clássico do somador.
                let k = rad * std::f32::consts::FRAC_1_SQRT_2;
                let s = line(pal.weak, 1.0);
                p.line_segment([c + vec2(-k, -k), c + vec2(k, k)], s);
                p.line_segment([c + vec2(-k, k), c + vec2(k, -k)], s);
                for (i, input) in inputs.iter().enumerate() {
                    let at = view.to_screen(geometry::sign_pos(n, i as u8));
                    let (glyph, color) = match input.sign {
                        Sign::Plus => ("+", pal.fg),
                        Sign::Minus => ("−", pal.fg),
                    };
                    p.text(
                        at,
                        Align2::CENTER_CENTER,
                        glyph,
                        FontId::monospace(15.0 * z),
                        color,
                    );
                }
            }
            NodeKind::Branch { .. } => {
                let color = if selected {
                    pal.selected
                } else if marks.errors.contains(&n.id) {
                    pal.error
                } else {
                    pal.fg
                };
                p.circle_filled(r.center(), (geometry::BRANCH_R * z).max(2.5), color);
            }
        }

        // Portas: entradas vazias e saídas sem fio ficam marcadas.
        if !matches!(n.kind, NodeKind::Branch { .. }) {
            for i in 0..n.n_inputs() as u8 {
                if !connected.contains(&(PortRef::new(n.id, i), PortDir::In)) {
                    let at = view.to_screen(geometry::in_port(n, i).pos);
                    p.circle(at, 3.5 * z.max(0.7), pal.fill, line(pal.error, 1.2));
                }
            }
            for i in 0..n.n_outputs() as u8 {
                if !connected.contains(&(PortRef::new(n.id, i), PortDir::Out)) {
                    let at = view.to_screen(geometry::out_port(n, i).pos);
                    p.circle_filled(at, 3.5 * z.max(0.7), pal.error);
                }
            }
        }
    }

    // Alças de ligação: sempre durante uma ligação; senão, só do nó sob o
    // mouse (as das ramificações só aparecem assim).
    for h in geometry::handles(d) {
        let show = marks.all_handles
            || marks.hovered == Some(h.port.node)
            || marks.handles_of.is_some_and(|s| s.contains(&h.port.node));
        if !show {
            continue;
        }
        let at = view.to_screen(h.pos);
        let is_target = marks.target == Some(h);
        let radius = match (is_target, marks.touch) {
            (true, false) => 6.5,
            (false, false) => 4.5,
            (true, true) => 8.5,
            (false, true) => 6.0,
        };
        let color = if is_target { pal.selected } else { pal.weak };
        match h.dir {
            PortDir::In => p.circle(at, radius, pal.fill, Stroke::new(1.5, color)),
            PortDir::Out => p.circle(at, radius, color, Stroke::new(1.5, color)),
        };
    }
}

/// Aparência de uma miniatura (cartões e pré-visualização da tela inicial).
pub struct MiniStyle {
    pub line: Color32,
    pub text: Color32,
    pub fill: Color32,
    /// Margem dentro do retângulo.
    pub margin: f32,
    /// Escala máxima (diagramas pequenos não ficam enormes).
    pub max_zoom: f32,
    /// Quanto a escala vertical pode passar da horizontal (diagramas largos
    /// ocupam melhor uma área mais alta).
    pub stretch: f32,
    /// Altura dos blocos em relação à escala.
    pub node_scale: f32,
    /// Limites do texto em pixels de tela.
    pub font_min: f32,
    pub font_max: f32,
    pub sum_max: f32,
    pub dot_r: f32,
}

/// Desenha o diagrama inteiro dentro de `rect`: sem setas nem sinais dos
/// somadores, com nós um pouco maiores que a escala para o texto caber
/// (os fios continuam nos mesmos lugares, cobertos pelo nó).
pub fn miniature(p: &Painter, d: &Diagram, rect: Rect, st: &MiniStyle) {
    let wires: Vec<Vec<Pos2>> = d
        .wires
        .iter()
        .filter_map(|w| geometry::wire_points(d, w.id))
        .collect();
    let Some(bounds) = d
        .nodes
        .iter()
        .map(geometry::rect)
        .chain(wires.iter().map(|pts| Rect::from_points(pts)))
        .reduce(|a, b| a.union(b))
    else {
        return;
    };
    let area = rect.shrink(st.margin);
    let zoom = (area.width() / bounds.width().max(1.0))
        .min(area.height() / bounds.height().max(1.0))
        .min(st.max_zoom);
    let zy = (area.height() / bounds.height().max(1.0)).min(zoom * st.stretch);
    let to = |q: Pos2| {
        let v = q - bounds.center();
        area.center() + vec2(v.x * zoom, v.y * zy)
    };
    let stroke = Stroke::new(1.2, st.line);

    for pts in &wires {
        p.add(Shape::line(pts.iter().map(|&q| to(q)).collect(), stroke));
    }
    let block_h = 2.0 * GRID * zy * st.node_scale;
    let font = FontId::proportional((block_h * 0.5).clamp(st.font_min, st.font_max));
    for n in &d.nodes {
        let r = geometry::rect(n);
        let c = to(r.center());
        match &n.kind {
            NodeKind::Block { expr: text }
            | NodeKind::Input { name: text }
            | NodeKind::Output { name: text } => {
                let r = Rect::from_center_size(
                    c,
                    vec2(
                        r.width() * zoom * 1.15,
                        block_h.min(r.height() * zy * st.node_scale),
                    ),
                );
                p.rect(
                    r,
                    CornerRadius::same(3),
                    st.fill,
                    stroke,
                    StrokeKind::Middle,
                );
                p.text(c, Align2::CENTER_CENTER, text, font.clone(), st.text);
            }
            NodeKind::Sum { .. } => {
                let rad = (geometry::SUM_R * zoom * 1.3).min(st.sum_max);
                p.circle(c, rad, st.fill, stroke);
                let k = rad * std::f32::consts::FRAC_1_SQRT_2;
                p.line_segment([c + vec2(-k, -k), c + vec2(k, k)], stroke);
                p.line_segment([c + vec2(-k, k), c + vec2(k, -k)], stroke);
            }
            NodeKind::Branch { .. } => {
                p.circle_filled(c, st.dot_r, st.text);
            }
        }
    }
}
