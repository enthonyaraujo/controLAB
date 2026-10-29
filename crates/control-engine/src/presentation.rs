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

/// Escapes text and renders `x_a`, `x_{ab}`, `x^2` and `x^{-1}` as SVG
/// sub/superscripts, like the Matplotlib mathtext labels of the Python plots.
pub fn rich(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let shift = match c {
            '_' => "sub",
            '^' => "super",
            _ => {
                out.push_str(&escape(c.encode_utf8(&mut [0; 4])));
                continue;
            }
        };
        let mut script = String::new();
        if chars.peek() == Some(&'{') {
            chars.next();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                script.push(c);
            }
        } else if let Some(c) = chars.next() {
            script.push(c);
        }
        let _ = write!(
            out,
            r#"<tspan baseline-shift="{shift}" font-size="75%">{}</tspan>"#,
            escape(&script)
        );
    }
    out
}

/// Tick positions inside `[lo, hi]` with steps 1, 2, 2.5 or 5 × 10ⁿ, as
/// Matplotlib's MaxNLocator chooses for at most `bins` intervals.
pub fn nice_ticks(lo: f64, hi: f64, bins: usize) -> (Vec<f64>, f64) {
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
    let raw = (hi - lo) / bins.max(1) as f64;
    if !raw.is_finite() || raw <= 0. {
        return (vec![lo], 1.);
    }
    let scale = 10f64.powf(raw.log10().floor());
    let mut step = 10. * scale;
    for factor in [1., 2., 2.5, 5., 10.] {
        let candidate = factor * scale;
        let count = (hi / candidate + 1e-9).floor() - (lo / candidate - 1e-9).ceil();
        if candidate >= raw * (1. - 1e-9) && count <= bins as f64 {
            step = candidate;
            break;
        }
    }
    let first = (lo / step - 1e-9).ceil() as i64;
    let last = (hi / step + 1e-9).floor() as i64;
    let ticks = (first..=last)
        .map(|i| {
            let v = i as f64 * step;
            if v.abs() < step * 1e-9 { 0. } else { v }
        })
        .collect();
    (ticks, step)
}

/// Tick text with the decimals implied by the step and a typographic minus.
pub fn tick_label(v: f64, step: f64) -> String {
    let decimals = (0..=8)
        .find(|&d| {
            let scaled = step * 10f64.powi(d);
            (scaled - scaled.round()).abs() < 1e-6 * scaled.max(1.)
        })
        .unwrap_or(8) as usize;
    let text = format!("{v:.decimals$}");
    match text.strip_prefix('-') {
        Some(rest) if rest.chars().any(|c| c != '0' && c != '.') => format!("−{rest}"),
        Some(rest) => rest.into(),
        None => text,
    }
}

/// Decade tick text, e.g. 10⁻² as `10<tspan …>−2</tspan>`.
pub fn decade_label(exponent: i32) -> String {
    format!(
        r#"10<tspan baseline-shift="super" font-size="75%">{}</tspan>"#,
        exponent.to_string().replace('-', "−")
    )
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
