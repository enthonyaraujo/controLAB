//! Geometria do diagrama em coordenadas do "mundo" (pixels com zoom 1):
//! tamanho dos nós, posição das portas e o traçado ortogonal dos fios.
//!
//! Fica separada do desenho para ser usada igual na tela, no teste de
//! clique e (na fase 6) na exportação SVG/PNG.

use blocklab_core::diagram::{Diagram, Node, NodeKind, PortDir, PortRef, Side, WireId};
use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

/// Espaçamento da grade. Centros dos nós caem na grade, então portas na
/// mesma altura geram fios retos.
pub const GRID: f32 = 20.0;
pub const SUM_R: f32 = 18.0;
pub const BRANCH_R: f32 = 4.5;
/// Trecho reto que sai/chega perpendicular à porta antes da primeira curva.
pub const STUB: f32 = 14.0;
/// Distância das alças de ligação da ramificação até o ponto.
pub const BRANCH_HANDLE: f32 = 14.0;

pub fn snap(v: f32) -> f32 {
    (v / GRID).round() * GRID
}

pub fn snap_pos(p: Pos2) -> Pos2 {
    pos2(snap(p.x), snap(p.y))
}

/// Arredonda para cima até um múltiplo de 2·GRID (bordas na grade).
fn snap_width(w: f32) -> f32 {
    (w / (2.0 * GRID)).ceil() * 2.0 * GRID
}

pub fn center(n: &Node) -> Pos2 {
    pos2(n.pos.x, n.pos.y)
}

/// Largura de texto estimada (sem medir fonte), para a geometria não
/// depender da interface.
fn text_width(s: &str) -> f32 {
    s.chars().count() as f32 * 8.5
}

pub fn size(n: &Node) -> Vec2 {
    match &n.kind {
        // Entrada/saída: "pílula" mais larga que alta, para não parecer um
        // somador. Largura em múltiplos de GRID (o centro fica na grade).
        NodeKind::Input { name } | NodeKind::Output { name } => {
            let w = (text_width(name) + 28.0).max(56.0);
            vec2((w / GRID).ceil() * GRID, 1.5 * GRID)
        }
        NodeKind::Block { expr } => {
            vec2(snap_width((text_width(expr) + 28.0).max(80.0)), 2.0 * GRID)
        }
        NodeKind::Sum { .. } => vec2(2.0 * SUM_R, 2.0 * SUM_R),
        NodeKind::Branch { .. } => vec2(2.0 * BRANCH_R, 2.0 * BRANCH_R),
    }
}

pub fn rect(n: &Node) -> Rect {
    Rect::from_center_size(center(n), size(n))
}

/// Vetor unitário para fora do nó no lado dado; `flipped` espelha.
fn side_dir(side: Side, flipped: bool) -> Vec2 {
    let d = match side {
        Side::Left => vec2(-1.0, 0.0),
        Side::Right => vec2(1.0, 0.0),
        Side::Top => vec2(0.0, -1.0),
        Side::Bottom => vec2(0.0, 1.0),
    };
    if flipped { vec2(-d.x, d.y) } else { d }
}

/// Uma porta: posição e normal (direção "para fora" do nó). Ramificações
/// têm normal zero: a direção é escolhida pelo traçado (ver [`route`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Port {
    pub pos: Pos2,
    pub normal: Vec2,
}

pub fn out_port(n: &Node, _i: u8) -> Port {
    let c = center(n);
    match n.kind {
        NodeKind::Branch { .. } => Port {
            pos: c,
            normal: Vec2::ZERO,
        },
        _ => {
            let normal = side_dir(Side::Right, n.flipped);
            Port {
                pos: c + normal * size(n).x / 2.0,
                normal,
            }
        }
    }
}

pub fn in_port(n: &Node, i: u8) -> Port {
    let c = center(n);
    match &n.kind {
        NodeKind::Branch { .. } => Port {
            pos: c,
            normal: Vec2::ZERO,
        },
        NodeKind::Sum { inputs } => {
            let side = inputs.get(i as usize).map_or(Side::Left, |s| s.side);
            let normal = side_dir(side, n.flipped);
            // Várias entradas no mesmo lado: espalha ao longo da borda.
            let same: Vec<usize> = (0..inputs.len())
                .filter(|&k| inputs[k].side == side)
                .collect();
            let rank = same.iter().position(|&k| k == i as usize).unwrap_or(0) as f32;
            let offset = (rank - (same.len() as f32 - 1.0) / 2.0) * 10.0;
            let tangent = vec2(-normal.y, normal.x);
            Port {
                pos: c + normal * SUM_R + tangent * offset,
                normal,
            }
        }
        _ => {
            let normal = side_dir(Side::Left, n.flipped);
            Port {
                pos: c + normal * size(n).x / 2.0,
                normal,
            }
        }
    }
}

/// Onde desenhar (e clicar) o sinal + / − da entrada `i` de um somador:
/// ao lado da seta, do lado de fora do círculo.
pub fn sign_pos(n: &Node, i: u8) -> Pos2 {
    let p = in_port(n, i);
    let tangent = if p.normal.x != 0.0 {
        vec2(0.0, -1.0)
    } else {
        vec2(1.0, 0.0)
    };
    p.pos + p.normal * 10.0 + tangent * 11.0
}

/// Ponto onde o usuário agarra/solta para ligar fios.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Handle {
    pub port: PortRef,
    pub dir: PortDir,
    pub pos: Pos2,
}

/// Todas as alças de ligação. A ramificação tem uma alça de entrada à
/// esquerda e uma de saída à direita, que sempre cria uma saída nova (as
/// saídas da ramificação são geridas automaticamente).
pub fn handles(d: &Diagram) -> Vec<Handle> {
    let mut out = Vec::new();
    for n in &d.nodes {
        if let NodeKind::Branch { outputs } = n.kind {
            let c = center(n);
            out.push(Handle {
                port: PortRef::new(n.id, 0),
                dir: PortDir::In,
                pos: c - vec2(BRANCH_HANDLE, 0.0),
            });
            out.push(Handle {
                port: PortRef::new(n.id, outputs),
                dir: PortDir::Out,
                pos: c + vec2(BRANCH_HANDLE, 0.0),
            });
            continue;
        }
        for i in 0..n.n_inputs() as u8 {
            out.push(Handle {
                port: PortRef::new(n.id, i),
                dir: PortDir::In,
                pos: in_port(n, i).pos,
            });
        }
        for i in 0..n.n_outputs() as u8 {
            out.push(Handle {
                port: PortRef::new(n.id, i),
                dir: PortDir::Out,
                pos: out_port(n, i).pos,
            });
        }
    }
    out
}

fn axis_toward(from: Pos2, to: Pos2, prefer_vertical: bool) -> Vec2 {
    let d = to - from;
    let sign = |v: f32| if v < 0.0 { -1.0 } else { 1.0 };
    // Prefere o eixo pedido se houver distância nele; senão, o eixo de
    // maior distância.
    let vertical = if prefer_vertical {
        d.y.abs() > 1.0 || d.y.abs() > d.x.abs()
    } else {
        d.x.abs() <= 1.0 && d.y.abs() > d.x.abs()
    };
    if vertical {
        vec2(0.0, sign(d.y))
    } else {
        vec2(sign(d.x), 0.0)
    }
}

/// Normal efetiva de uma porta de ramificação (normal zero): perpendicular
/// à porta do outro lado, como num desenho de livro — o fio sai do ponto
/// na vertical e depois vira para o bloco.
fn resolve(p: Port, other: Port) -> Port {
    if p.normal != Vec2::ZERO {
        return p;
    }
    let other_horizontal = other.normal.x != 0.0 || other.normal == Vec2::ZERO;
    Port {
        pos: p.pos,
        normal: axis_toward(p.pos, other.pos, other_horizontal),
    }
}

/// Traçado ortogonal de `a` (porta de saída) até `b` (porta de entrada).
///
/// Sai de `a` um trecho reto na direção da normal, chega em `b` também
/// perpendicular, e liga os dois trechos com no máximo três segmentos:
/// * normais opostas e `b` à frente → "Z" (vira no meio);
/// * normais opostas e `b` atrás → contorna por uma linha horizontal no
///   meio (ou abaixo, se estiverem na mesma altura);
/// * mesma normal → "U" pelo lado mais externo (realimentação);
/// * normais perpendiculares → "L".
pub fn route(a: Port, b: Port) -> Vec<Pos2> {
    // Porta de ramificação (normal zero): o fio sai direto do ponto, sem
    // trecho reto.
    let stub = |p: &Port| if p.normal == Vec2::ZERO { 0.0 } else { STUB };
    let (sa, sb) = (stub(&a), stub(&b));
    let a = resolve(a, b);
    let b = resolve(b, a);
    let p1 = a.pos + a.normal * sa;
    let p2 = b.pos + b.normal * sb;
    let (na, nb) = (a.normal, b.normal);
    let ha = na.x != 0.0;
    let hb = nb.x != 0.0;

    let mut pts = vec![a.pos, p1];
    if ha == hb {
        // Trabalha num eixo "principal" (x se horizontais, y se verticais)
        // e troca de volta no fim.
        let main = |p: Pos2| if ha { p.x } else { p.y };
        let cross = |p: Pos2| if ha { p.y } else { p.x };
        let mk = |m: f32, c: f32| if ha { pos2(m, c) } else { pos2(c, m) };
        let dir_a = if ha { na.x } else { na.y };
        let dir_b = if hb { nb.x } else { nb.y };
        if dir_a == -dir_b {
            if (main(p2) - main(p1)) * dir_a >= 0.0 {
                let m = (main(p1) + main(p2)) / 2.0;
                pts.push(mk(m, cross(p1)));
                pts.push(mk(m, cross(p2)));
            } else {
                let c = if (cross(p1) - cross(p2)).abs() < 2.0 * GRID {
                    cross(p1).max(cross(p2)) + 2.0 * GRID
                } else {
                    (cross(p1) + cross(p2)) / 2.0
                };
                pts.push(mk(main(p1), c));
                pts.push(mk(main(p2), c));
            }
        } else {
            let m = if dir_a > 0.0 {
                main(p1).max(main(p2))
            } else {
                main(p1).min(main(p2))
            };
            pts.push(mk(m, cross(p1)));
            pts.push(mk(m, cross(p2)));
        }
    } else {
        // Perpendiculares: um canto só, se ele ficar "à frente" das duas
        // portas; senão, o outro canto.
        let corner = if ha {
            pos2(p2.x, p1.y)
        } else {
            pos2(p1.x, p2.y)
        };
        let ahead = |p: Pos2, n: Vec2| (corner - p).dot(n) >= 0.0;
        if ahead(p1, na) && ahead(p2, nb) {
            pts.push(corner);
        } else if ha {
            pts.push(pos2(p1.x, p2.y));
        } else {
            pts.push(pos2(p2.x, p1.y));
        }
    }
    pts.push(p2);
    pts.push(b.pos);
    simplify(pts)
}

/// Remove pontos repetidos e pontos no meio de segmentos retos.
fn simplify(pts: Vec<Pos2>) -> Vec<Pos2> {
    let mut out: Vec<Pos2> = Vec::with_capacity(pts.len());
    for p in pts {
        if out.last().is_some_and(|q| q.distance(p) < 0.01) {
            continue;
        }
        if out.len() >= 2 {
            let a = out[out.len() - 2];
            let b = out[out.len() - 1];
            let collinear = (a.x - b.x).abs() < 0.01 && (b.x - p.x).abs() < 0.01
                || (a.y - b.y).abs() < 0.01 && (b.y - p.y).abs() < 0.01;
            if collinear {
                out.pop();
            }
        }
        out.push(p);
    }
    out
}

/// Pontos do fio `w` (em coordenadas do mundo), ou `None` se ele aponta
/// para nós/portas que não existem.
pub fn wire_points(d: &Diagram, w: WireId) -> Option<Vec<Pos2>> {
    let w = d.wires.iter().find(|x| x.id == w)?;
    let a = out_port(d.node(w.from.node)?, w.from.port);
    let b = in_port(d.node(w.to.node)?, w.to.port);
    Some(route(a, b))
}

/// Distância de um ponto a uma poligonal.
pub fn distance_to_polyline(p: Pos2, pts: &[Pos2]) -> f32 {
    pts.windows(2)
        .map(|s| {
            let (a, b) = (s[0], s[1]);
            let ab = b - a;
            let t = if ab.length_sq() == 0.0 {
                0.0
            } else {
                ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
            };
            p.distance(a + ab * t)
        })
        .fold(f32::INFINITY, f32::min)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::{Sign, examples};

    fn is_orthogonal(pts: &[Pos2]) -> bool {
        pts.windows(2)
            .all(|s| (s[0].x - s[1].x).abs() < 0.01 || (s[0].y - s[1].y).abs() < 0.01)
    }

    #[test]
    fn todos_os_fios_dos_exemplos_sao_ortogonais() {
        for (name, d) in examples::all() {
            for w in &d.wires {
                let pts = wire_points(&d, w.id).unwrap();
                assert!(is_orthogonal(&pts), "{name}: {pts:?}");
                assert!(pts.len() >= 2);
            }
        }
    }

    #[test]
    fn fio_reto_entre_portas_alinhadas() {
        let a = Port {
            pos: pos2(0.0, 100.0),
            normal: vec2(1.0, 0.0),
        };
        let b = Port {
            pos: pos2(200.0, 100.0),
            normal: vec2(-1.0, 0.0),
        };
        assert_eq!(route(a, b), vec![pos2(0.0, 100.0), pos2(200.0, 100.0)]);
    }

    #[test]
    fn realimentacao_contorna_por_baixo_em_l() {
        // Saída de H (virado, porta à esquerda) para a entrada de baixo do
        // somador: um "L" — vai para a esquerda e sobe.
        let d = examples::feedback(Sign::Minus);
        let h = d.nodes.iter().find(|n| n.flipped).unwrap();
        let w = d.wires.iter().find(|w| w.from.node == h.id).unwrap();
        let pts = wire_points(&d, w.id).unwrap();
        assert!(is_orthogonal(&pts));
        assert_eq!(pts.len(), 3, "{pts:?}"); // porta, canto, chegada
    }

    #[test]
    fn ramificacao_sai_na_vertical_para_bloco_acima() {
        let d = examples::parallel();
        let br = d
            .nodes
            .iter()
            .find(|n| matches!(n.kind, NodeKind::Branch { .. }))
            .unwrap();
        let w = d.wires.iter().find(|w| w.from.node == br.id).unwrap();
        let pts = wire_points(&d, w.id).unwrap();
        // Primeiro segmento vertical, partindo do próprio ponto.
        assert_eq!(pts[0], center(br));
        assert!((pts[0].x - pts[1].x).abs() < 0.01, "{pts:?}");
    }

    #[test]
    fn distancia_a_poligonal() {
        let pts = [pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 10.0)];
        assert!((distance_to_polyline(pos2(5.0, 3.0), &pts) - 3.0).abs() < 1e-4);
        assert!((distance_to_polyline(pos2(12.0, 5.0), &pts) - 2.0).abs() < 1e-4);
    }
}
