//! Deterministic SVG/PNG output with canonical bundled fonts, no external renderer.
use crate::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{fmt::Write, sync::OnceLock};

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn rasterize(svg: &str) -> Result<String> {
    static OPTIONS: OnceLock<resvg::usvg::Options<'static>> = OnceLock::new();
    let options = OPTIONS.get_or_init(|| {
        let mut opts = resvg::usvg::Options::default();
        opts.fontdb_mut().load_font_data(
            include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf").to_vec(),
        );
        opts.fontdb_mut().load_font_data(
            include_bytes!("../../../assets/fonts/jetbrains-mono/JetBrainsMono-Regular.ttf")
                .to_vec(),
        );
        opts.font_family = "IBM Plex Sans".into();
        opts
    });
    let tree =
        resvg::usvg::Tree::from_str(svg, options).map_err(|e| format!("SVG inválido: {e}"))?;
    let size = tree.size().to_int_size();
    if u64::from(size.width()) * u64::from(size.height()) > 24_000_000 {
        return Err("Imagem excede o limite de recursos.".into());
    }
    let mut pixels = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or("Dimensões de imagem inválidas.")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixels.as_mut(),
    );
    let png = pixels.encode_png().map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(png)))
}

pub fn plot(
    series: &[(&str, &[f64], &[f64])],
    title: &str,
    xlabel: &str,
    ylabel: &str,
    _theme: &str,
    log_x: bool,
) -> Result<(String, String)> {
    let mut xmin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymin = f64::INFINITY;
    let mut ymax = f64::NEG_INFINITY;
    for (_, xs, ys) in series {
        if xs.len() != ys.len() {
            return Err("Séries do gráfico possuem dimensões incompatíveis.".into());
        }
        for (&x, &y) in xs.iter().zip(*ys) {
            if x.is_finite() && y.is_finite() && (!log_x || x > 0.) {
                let x = if log_x { x.log10() } else { x };
                xmin = xmin.min(x);
                xmax = xmax.max(x);
                ymin = ymin.min(y);
                ymax = ymax.max(y);
            }
        }
    }
    if !xmin.is_finite() {
        return Err("O gráfico não contém pontos finitos.".into());
    }
    if xmin == xmax {
        xmin -= 1.;
        xmax += 1.;
    }
    if ymin == ymax {
        ymin -= 1.;
        ymax += 1.;
    }
    let dy = (ymax - ymin) * 0.05;
    ymin -= dy;
    ymax += dy;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1288" height="728" viewBox="0 0 1288 728"><rect width="1288" height="728" fill="white"/><defs><clipPath id="plot-area"><rect x="95" y="45" width="1155" height="600"/></clipPath></defs><g font-family="IBM Plex Sans" font-size="17" fill="black"><text x="644" y="25" text-anchor="middle">{}</text>"#,
        escape(title)
    );
    for i in 0..=5 {
        let f = i as f64 / 5.;
        let x = 95. + f * 1155.;
        let y = 645. - f * 600.;
        let xv = xmin + f * (xmax - xmin);
        let yv = ymin + f * (ymax - ymin);
        let _ = write!(
            svg,
            r##"<path d="M{x} 45V645 M95 {y}H1250" fill="none" stroke="#CBD5E1" stroke-opacity=".75" stroke-dasharray="4 4"/><text x="{x}" y="674" text-anchor="middle">{}</text><text x="82" y="{}" text-anchor="end">{}</text>"##,
            if log_x {
                format!("{:.4}", 10f64.powf(xv))
            } else {
                format!("{xv:.4}")
            },
            y + 5.,
            format_args!("{yv:.4}")
        );
    }
    svg.push_str(r#"<path d="M95 45V645H1250" stroke="black" fill="none"/>"#);
    let colors = ["#000000", "#64748B", "#334155", "#B91C1C"];
    for (index, (label, xs, ys)) in series.iter().enumerate() {
        let color = colors[index % colors.len()];
        let mut path = String::new();
        let mut connected = false;
        for (&x, &y) in xs.iter().zip(*ys) {
            if !x.is_finite() || !y.is_finite() || (log_x && x <= 0.) {
                connected = false;
                continue;
            }
            let x = if log_x { x.log10() } else { x };
            let px = 95. + (x - xmin) / (xmax - xmin) * 1155.;
            let py = 645. - (y - ymin) / (ymax - ymin) * 600.;
            let _ = write!(
                path,
                "{} {:.4} {:.4}",
                if connected { "L" } else { "M" },
                px,
                py
            );
            connected = true;
        }
        let _ = write!(
            svg,
            r#"<path d="{path}" fill="none" stroke="{color}" stroke-width="2" clip-path="url(#plot-area)"/><text x="{}" y="62" fill="{color}">{}</text>"#,
            115. + index as f64 * 260.,
            escape(label)
        );
    }
    let _ = write!(
        svg,
        r#"<text x="672" y="715" text-anchor="middle">{}</text><text transform="translate(25 345) rotate(-90)" text-anchor="middle">{}</text></g></svg>"#,
        escape(xlabel),
        escape(ylabel)
    );
    Ok((rasterize(&svg)?, svg))
}
