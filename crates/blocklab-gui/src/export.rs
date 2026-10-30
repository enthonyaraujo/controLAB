//! Exportação do diagrama como imagem.
//!
//! * **SVG** — escrito à mão a partir de `geometry.rs`, então o traçado é
//!   exatamente o da tela. Fundo branco e traço preto, pronto para
//!   relatório (independe do tema claro/escuro da interface).
//! * **PNG** — o próprio SVG rasterizado com `resvg`, com a fonte DejaVu
//!   embutida (o texto sai igual em qualquer máquina).

use std::fmt::Write as _;

use blocklab_core::diagram::{Diagram, NodeKind, Sign};
use eframe::egui::{Pos2, Rect, Vec2, vec2};

use crate::fonts::{DEJAVU_FAMILY, DEJAVU_SANS};
use crate::geometry::{self, BRANCH_R};

const MARGIN: f32 = 24.0;
const INK: &str = "#1a1a1a";
const WEAK: &str = "#8a8a8a";
const FILL: &str = "#ffffff";
const BACKGROUND: &str = "#ffffff";
const TERMINAL_FILL: &str = "#f0f0f0";
const FONT_SIZE: f32 = 15.0;

/// Escapa texto para dentro de XML (`<`, `&`... em expressões como a<b).
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Texto centrado em `c`. Sem `dominant-baseline` (suporte irregular
/// entre leitores de SVG): a linha de base desce ~0,35 em.
fn text(out: &mut String, c: Pos2, s: &str, size: f32, color: &str) {
    let _ = writeln!(
        out,
        r#"  <text x="{:.1}" y="{:.1}" font-family="{DEJAVU_FAMILY}, sans-serif" font-size="{size}" text-anchor="middle" fill="{color}">{}</text>"#,
        c.x,
        c.y + size * 0.35,
        xml(s)
    );
}

fn arrow_head(out: &mut String, tip: Pos2, dir: Vec2) {
    let d = dir.normalized();
    let n = vec2(-d.y, d.x);
    let size = 9.0;
    let base = tip - d * size;
    let (a, b) = (base + n * size * 0.45, base - n * size * 0.45);
    let _ = writeln!(
        out,
        r#"  <polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="{INK}"/>"#,
        tip.x, tip.y, a.x, a.y, b.x, b.y
    );
}

/// Retângulo (mundo) que contém todos os nós e fios.
fn bounds(d: &Diagram) -> Rect {
    let mut r = Rect::NOTHING;
    for n in &d.nodes {
        r = r.union(geometry::rect(n));
    }
    for w in &d.wires {
        for p in geometry::wire_points(d, w.id).unwrap_or_default() {
            r.extend_with(p);
        }
    }
    if r.is_positive() {
        r
    } else {
        Rect::from_min_size(Pos2::ZERO, vec2(200.0, 100.0))
    }
}

pub fn to_svg(d: &Diagram) -> String {
    let b = bounds(d).expand(MARGIN);
    let mut out = String::new();
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="{x:.1} {y:.1} {w:.1} {h:.1}">"#,
        x = b.min.x,
        y = b.min.y,
        w = b.width(),
        h = b.height()
    );
    let _ = writeln!(out, "  <!-- Gerado pelo ControLAB -->");
    let _ = writeln!(
        out,
        r#"  <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{BACKGROUND}"/>"#,
        b.min.x,
        b.min.y,
        b.width(),
        b.height()
    );

    // Fios (por baixo dos nós).
    for w in &d.wires {
        let Some(pts) = geometry::wire_points(d, w.id) else {
            continue;
        };
        let list: Vec<String> = pts
            .iter()
            .map(|p| format!("{:.1},{:.1}", p.x, p.y))
            .collect();
        let _ = writeln!(
            out,
            r#"  <polyline points="{}" fill="none" stroke="{INK}" stroke-width="1.4" stroke-linejoin="round"/>"#,
            list.join(" ")
        );
        let into_branch = d
            .node(w.to.node)
            .is_some_and(|n| matches!(n.kind, NodeKind::Branch { .. }));
        if !into_branch && pts.len() >= 2 {
            let (a, z) = (pts[pts.len() - 2], pts[pts.len() - 1]);
            arrow_head(&mut out, z, z - a);
        }
    }

    for n in &d.nodes {
        let r = geometry::rect(n);
        let c = r.center();
        match &n.kind {
            NodeKind::Block { expr } => {
                let _ = writeln!(
                    out,
                    r#"  <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="2" fill="{FILL}" stroke="{INK}" stroke-width="1.5"/>"#,
                    r.min.x,
                    r.min.y,
                    r.width(),
                    r.height()
                );
                text(&mut out, c, expr, FONT_SIZE, INK);
            }
            NodeKind::Input { name } | NodeKind::Output { name } => {
                let _ = writeln!(
                    out,
                    r#"  <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="6" fill="{TERMINAL_FILL}" stroke="{INK}" stroke-width="1.5"/>"#,
                    r.min.x,
                    r.min.y,
                    r.width(),
                    r.height()
                );
                text(&mut out, c, name, FONT_SIZE, INK);
            }
            NodeKind::Sum { inputs } => {
                let rad = r.width() / 2.0;
                let k = rad * std::f32::consts::FRAC_1_SQRT_2;
                let _ = writeln!(
                    out,
                    r#"  <circle cx="{:.1}" cy="{:.1}" r="{rad:.1}" fill="{FILL}" stroke="{INK}" stroke-width="1.5"/>"#,
                    c.x, c.y
                );
                for (dx, dy) in [(k, k), (k, -k)] {
                    let _ = writeln!(
                        out,
                        r#"  <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{WEAK}" stroke-width="1"/>"#,
                        c.x - dx,
                        c.y - dy,
                        c.x + dx,
                        c.y + dy
                    );
                }
                for (i, input) in inputs.iter().enumerate() {
                    let glyph = match input.sign {
                        Sign::Plus => "+",
                        Sign::Minus => "−",
                    };
                    text(
                        &mut out,
                        geometry::sign_pos(n, i as u8),
                        glyph,
                        FONT_SIZE,
                        INK,
                    );
                }
            }
            NodeKind::Branch { .. } => {
                let _ = writeln!(
                    out,
                    r#"  <circle cx="{:.1}" cy="{:.1}" r="{BRANCH_R:.1}" fill="{INK}"/>"#,
                    c.x, c.y
                );
            }
        }
    }
    out.push_str("</svg>\n");
    out
}

/// PNG do diagrama. `scale = 2` dá uma imagem nítida em telas de alta
/// densidade e em impressão.
pub fn to_png(d: &Diagram, scale: f32) -> Result<Vec<u8>, String> {
    use resvg::{tiny_skia, usvg};
    let svg = to_svg(d);
    let mut opt = usvg::Options {
        font_family: DEJAVU_FAMILY.to_string(),
        ..usvg::Options::default()
    };
    opt.fontdb_mut().load_font_data(DEJAVU_SANS.to_vec());
    let tree = usvg::Tree::from_str(&svg, &opt).map_err(|e| format!("SVG inválido: {e}"))?;
    let size = tree.size();
    let w = (size.width() * scale).ceil() as u32;
    let h = (size.height() * scale).ceil() as u32;
    let mut pixmap =
        tiny_skia::Pixmap::new(w, h).ok_or_else(|| format!("imagem grande demais: {w}×{h}"))?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocklab_core::diagram::{Pos, examples};

    fn png_size(png: &[u8]) -> (u32, u32) {
        // Cabeçalho IHDR: largura e altura em big-endian nos bytes 16..24.
        let n = |i: usize| u32::from_be_bytes(png[i..i + 4].try_into().unwrap());
        (n(16), n(20))
    }

    #[test]
    fn svg_do_exemplo_de_referencia() {
        let d = examples::reference(Sign::Plus);
        let svg = to_svg(&d);
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert_eq!(svg.matches("<polyline").count(), d.wires.len());
        // 6 blocos + 2 terminais + 1 retângulo de fundo.
        assert_eq!(svg.matches("<rect").count(), 6 + 2 + 1);
        assert!(svg.contains(">G1</text>"));
        assert!(svg.contains(">−</text>"));
    }

    #[test]
    fn todos_os_exemplos_viram_png_valido() {
        for (name, d) in examples::all() {
            let png = to_png(&d, 2.0).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "{name}");
            let svg_w: f32 = bounds(&d).expand(MARGIN).width();
            let (w, _) = png_size(&png);
            assert_eq!(w, (svg_w * 2.0).ceil() as u32, "{name}");
        }
    }

    #[test]
    fn texto_com_caracteres_especiais_e_escapado() {
        let mut d = Diagram::new();
        d.add_block("a<b & c", Pos::new(0.0, 0.0));
        let svg = to_svg(&d);
        assert!(svg.contains(">a&lt;b &amp; c</text>"));
        to_png(&d, 1.0).unwrap();
    }

    #[test]
    fn diagrama_vazio_exporta() {
        assert!(to_png(&Diagram::new(), 1.0).is_ok());
    }
}
