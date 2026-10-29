//! Ícones de contorno desenhados com o `Painter` (sem imagens), numa
//! caixa de cerca de 20 × 20 px centrada em `c`.

use std::f32::consts::PI;

use eframe::egui::{
    Color32, CornerRadius, CursorIcon, Id, Painter, Pos2, Rect, Response, Sense, Shape, Stroke,
    StrokeKind, Ui, WidgetInfo, WidgetType, pos2, vec2,
};

pub fn plus(p: &Painter, c: Pos2, k: f32, s: Stroke) {
    p.line_segment([c - vec2(k, 0.0), c + vec2(k, 0.0)], s);
    p.line_segment([c - vec2(0.0, k), c + vec2(0.0, k)], s);
}

pub fn minus(p: &Painter, c: Pos2, k: f32, s: Stroke) {
    p.line_segment([c - vec2(k, 0.0), c + vec2(k, 0.0)], s);
}

/// Pasta em contorno; `r` é o corpo, a aba fica por cima à esquerda.
pub fn folder(p: &Painter, r: Rect, s: Stroke) {
    let tab = r.width() * 0.38;
    let lift = (r.height() * 0.22).round();
    let pts = vec![
        pos2(r.left(), r.top() - lift),
        pos2(r.left() + tab, r.top() - lift),
        pos2(r.left() + tab + lift, r.top()),
        pos2(r.right(), r.top()),
        r.right_bottom(),
        r.left_bottom(),
    ];
    p.add(Shape::closed_line(pts, s));
}

pub fn search(p: &Painter, c: Pos2, s: Stroke) {
    let o = c + vec2(-1.5, -1.5);
    p.circle_stroke(o, 5.5, s);
    p.line_segment([o + vec2(4.0, 4.0), o + vec2(8.0, 8.0)], s);
}

pub fn home(p: &Painter, c: Pos2, s: Stroke) {
    let pts = vec![
        c + vec2(-7.0, -1.0),
        c + vec2(0.0, -7.5),
        c + vec2(7.0, -1.0),
        c + vec2(7.0, 7.5),
        c + vec2(2.5, 7.5),
        c + vec2(2.5, 2.5),
        c + vec2(-2.5, 2.5),
        c + vec2(-2.5, 7.5),
        c + vec2(-7.0, 7.5),
    ];
    p.add(Shape::closed_line(pts, s));
}

/// Quatro quadrados (galeria de exemplos).
pub fn grid(p: &Painter, c: Pos2, s: Stroke) {
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let r = Rect::from_center_size(c + vec2(4.5 * dx, 4.5 * dy), vec2(6.0, 6.0));
        p.rect_stroke(r, CornerRadius::same(1), s, StrokeKind::Middle);
    }
}

/// Livro aberto (tutoriais).
pub fn book(p: &Painter, c: Pos2, s: Stroke) {
    let top = c.y - 6.0;
    let bottom = c.y + 6.5;
    for side in [-1.0, 1.0] {
        let pts = vec![
            pos2(c.x, top + 1.5),
            pos2(c.x + side * 3.0, top),
            pos2(c.x + side * 8.0, top),
            pos2(c.x + side * 8.0, bottom - 1.5),
            pos2(c.x + side * 3.0, bottom - 1.5),
            pos2(c.x, bottom),
        ];
        p.add(Shape::line(pts, s));
    }
    p.line_segment([pos2(c.x, top + 1.5), pos2(c.x, bottom)], s);
}

/// Ponto de interrogação num círculo (ajuda).
pub fn help(p: &Painter, c: Pos2, s: Stroke) {
    p.circle_stroke(c, 8.5, s);
    let arc: Vec<Pos2> = (0..=8)
        .map(|k| {
            let a = PI * (1.0 + 1.25 * k as f32 / 8.0);
            c + vec2(0.0, -2.3) + 2.8 * vec2(a.cos(), a.sin())
        })
        .collect();
    let end = *arc.last().unwrap_or(&c);
    p.add(Shape::line(arc, s));
    p.line_segment([end, c + vec2(0.0, 2.0)], s);
    p.circle_filled(c + vec2(0.0, 4.6), s.width * 0.75, s.color);
}

/// Três pontos verticais ("mais opções").
pub fn kebab(p: &Painter, c: Pos2, color: Color32) {
    for dy in [-6.0, 0.0, 6.0] {
        p.circle_filled(c + vec2(0.0, dy), 1.9, color);
    }
}

/// Três pontos horizontais (opções do elemento).
pub fn dots(p: &Painter, c: Pos2, color: Color32) {
    for dx in [-6.0, 0.0, 6.0] {
        p.circle_filled(c + vec2(dx, 0.0), 1.9, color);
    }
}

pub fn back(p: &Painter, c: Pos2, s: Stroke) {
    p.line_segment([c + vec2(-7.0, 0.0), c + vec2(7.0, 0.0)], s);
    p.line_segment([c + vec2(-7.0, 0.0), c + vec2(-1.5, -5.5)], s);
    p.line_segment([c + vec2(-7.0, 0.0), c + vec2(-1.5, 5.5)], s);
}

/// Seta curva para a esquerda (desfazer); `mirror` vira para a direita
/// (refazer).
pub fn undo(p: &Painter, c: Pos2, s: Stroke, mirror: bool) {
    let m = if mirror { -1.0 } else { 1.0 };
    let o = c + vec2(m * 1.0, 1.5);
    let arc: Vec<Pos2> = (0..=10)
        .map(|k| {
            let a = -PI / 2.0 + PI * k as f32 / 10.0;
            o + vec2(m * 5.5 * a.cos(), 5.0 * a.sin())
        })
        .collect();
    p.add(Shape::line(arc, s));
    let head = o + vec2(0.0, -5.0);
    p.line_segment([head, head + vec2(-m * 6.5, 0.0)], s);
    let tip = head + vec2(-m * 6.5, 0.0);
    p.line_segment([tip, tip + vec2(m * 3.5, -3.5)], s);
    p.line_segment([tip, tip + vec2(m * 3.5, 3.5)], s);
    // Cauda até a base.
    p.line_segment([o + vec2(0.0, 5.0), o + vec2(-m * 5.0, 5.0)], s);
}

pub fn trash(p: &Painter, c: Pos2, s: Stroke) {
    p.line_segment([c + vec2(-7.0, -4.5), c + vec2(7.0, -4.5)], s);
    p.line_segment([c + vec2(-2.5, -7.0), c + vec2(2.5, -7.0)], s);
    let pts = vec![
        c + vec2(-5.5, -4.5),
        c + vec2(-4.5, 7.5),
        c + vec2(4.5, 7.5),
        c + vec2(5.5, -4.5),
    ];
    p.add(Shape::line(pts, s));
    p.line_segment([c + vec2(-1.5, -1.5), c + vec2(-1.5, 4.5)], s);
    p.line_segment([c + vec2(1.5, -1.5), c + vec2(1.5, 4.5)], s);
}

/// Dois triângulos espelhados em torno de um eixo (espelhar).
pub fn flip(p: &Painter, c: Pos2, s: Stroke) {
    p.add(Shape::dashed_line(
        &[c + vec2(0.0, -8.0), c + vec2(0.0, 8.0)],
        s,
        2.0,
        2.0,
    ));
    for side in [-1.0, 1.0] {
        let pts = vec![
            c + vec2(side * 2.5, -5.5),
            c + vec2(side * 8.0, 5.5),
            c + vec2(side * 2.5, 5.5),
        ];
        p.add(Shape::closed_line(pts, s));
    }
}

pub fn pencil(p: &Painter, c: Pos2, s: Stroke) {
    let pts = vec![
        c + vec2(-7.0, 7.0),
        c + vec2(-6.0, 3.0),
        c + vec2(3.5, -6.5),
        c + vec2(6.5, -3.5),
        c + vec2(-3.0, 6.0),
    ];
    p.add(Shape::closed_line(pts, s));
    p.line_segment([c + vec2(1.0, -4.0), c + vec2(4.0, -1.0)], s);
}

/// Quatro cantos (ajustar à tela).
pub fn fit(p: &Painter, c: Pos2, s: Stroke) {
    let (k, l) = (7.0, 4.0);
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let corner = c + vec2(dx * k, dy * k);
        p.line_segment([corner, corner - vec2(dx * l, 0.0)], s);
        p.line_segment([corner, corner - vec2(0.0, dy * l)], s);
    }
}

/// Retângulo tracejado com seta (seleção por área).
pub fn select_area(p: &Painter, c: Pos2, s: Stroke) {
    let r = Rect::from_center_size(c + vec2(-1.5, -1.5), vec2(13.0, 11.0));
    let pts = vec![
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
        r.left_top(),
    ];
    p.add(Shape::dashed_line(&pts, s, 2.5, 2.0));
    let tip = r.center() + vec2(1.0, 1.0);
    let arrow = vec![
        tip,
        tip + vec2(0.0, 8.0),
        tip + vec2(2.2, 5.8),
        tip + vec2(5.8, 5.8),
    ];
    p.add(Shape::convex_polygon(arrow, s.color, Stroke::NONE));
}

pub fn chevron(p: &Painter, c: Pos2, s: Stroke, up: bool) {
    let d = if up { -1.0 } else { 1.0 };
    p.line_segment([c + vec2(-5.0, -d * 2.5), c + vec2(0.0, d * 2.5)], s);
    p.line_segment([c + vec2(0.0, d * 2.5), c + vec2(5.0, -d * 2.5)], s);
}

pub fn close(p: &Painter, c: Pos2, s: Stroke) {
    p.line_segment([c + vec2(-5.5, -5.5), c + vec2(5.5, 5.5)], s);
    p.line_segment([c + vec2(-5.5, 5.5), c + vec2(5.5, -5.5)], s);
}

/// Área clicável com nome acessível (leitores de tela e testes).
pub fn clickable(ui: &Ui, rect: Rect, id: Id, label: &str) -> Response {
    let resp = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    resp
}

/// Botão redondo só com ícone (barras superiores, controles do canvas).
/// `draw` recebe o centro e o traço já na cor certa (apagada se
/// desabilitado).
pub fn icon_button(
    ui: &Ui,
    rect: Rect,
    label: &str,
    enabled: bool,
    draw: impl FnOnce(&Painter, Pos2, Stroke),
) -> Response {
    let t = crate::theme::tokens(ui.visuals());
    let id = ui.id().with(("icone", label));
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let resp = ui.interact(rect, id, sense);
    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
    let p = ui.painter();
    if enabled && (resp.hovered() || resp.is_pointer_button_down_on()) {
        let fill = if resp.is_pointer_button_down_on() {
            t.pressed
        } else {
            t.hover
        };
        p.rect_filled(rect.shrink(2.0), CornerRadius::same(10), fill);
    }
    let color = if enabled { t.text } else { t.dashed };
    draw(p, rect.center(), Stroke::new(1.6, color));
    if enabled {
        resp.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        resp
    }
}
