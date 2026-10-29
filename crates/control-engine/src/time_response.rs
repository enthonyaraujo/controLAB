//! Continuous temporal analysis, preserving the Python module's sampled metrics.
use crate::{
    Result,
    simulation::{self, Input},
    transfer::TransferFunction,
};
use num_complex::Complex64;
use serde_json::{Value, json};

pub fn presets() -> Value {
    json!([
 {"id":"Segunda ordem subamortecida","title":"Segunda ordem subamortecida","expr":"4 / (s^2 + 2*s + 4)"},
 {"id":"Primeira ordem","title":"Primeira ordem","expr":"1 / (2*s + 1)"},
 {"id":"Sistema tipo 1","title":"Sistema tipo 1","expr":"5 / (s * (s + 2))"}])
}
pub(crate) fn number(p: &Value, key: &str, default: f64) -> Result<f64> {
    match p.get(key) {
        None => Ok(default),
        Some(v) => v
            .as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            .ok_or_else(|| format!("Valor inválido para {key}.")),
    }
}
pub(crate) fn points(p: &Value) -> Result<usize> {
    let v = match p.get("points") {
        None | Some(Value::Null) => 900.,
        _ => number(p, "points", 900.)?,
    };
    if !v.is_finite() {
        return Err("Número de pontos inválido.".into());
    }
    Ok((if v == 0. { 900. } else { v.trunc() }).clamp(200., 4000.) as usize)
}
pub(crate) fn safe(v: f64) -> Value {
    if v.is_nan() {
        Value::Null
    } else if v == f64::INFINITY {
        json!("∞")
    } else if v == f64::NEG_INFINITY {
        json!("-∞")
    } else {
        json!(v)
    }
}
/// Python's 0.5g formatting, including significant digits and exponent padding.
pub(crate) fn display(v: Option<f64>, missing: &str) -> String {
    let Some(v) = v else { return missing.into() };
    if v.is_nan() {
        return missing.into();
    }
    if v.is_infinite() {
        return "∞".into();
    }
    if v == 0. {
        return if v.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    let sci = format!("{v:0.4e}");
    let (m, e) = sci.split_once('e').unwrap();
    let exponent = e.parse::<i32>().unwrap();
    if !(-4..5).contains(&exponent) {
        let m = m.trim_end_matches('0').trim_end_matches('.');
        return format!(
            "{m}e{}{:02}",
            if exponent < 0 { '-' } else { '+' },
            exponent.abs()
        );
    }
    let decimals = (4 - exponent).max(0) as usize;
    let s = format!("{v:.decimals$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').into()
    } else {
        s
    }
}
fn constants(tf: &TransferFunction) -> (usize, [f64; 3], [f64; 3]) {
    let nz = tf.num.iter().rev().take_while(|&&v| v == 0.).count();
    let dz = tf.den.iter().rev().take_while(|&&v| v == 0.).count();
    let kind = dz.saturating_sub(nz);
    let mut k = [0.; 3];
    for (order, v) in k.iter_mut().enumerate() {
        if nz == tf.num.len() {
            *v = 0.;
            continue;
        }
        let exponent = nz as isize + order as isize - dz as isize;
        *v = if exponent < 0 {
            f64::INFINITY
        } else if exponent > 0 {
            0.
        } else {
            tf.num[tf.num.len() - nz - 1] / tf.den[tf.den.len() - dz - 1]
        };
    }
    let mut ess = [0.; 3];
    for i in 0..3 {
        let d = k[i] + if i == 0 { 1. } else { 0. };
        ess[i] = if k[i].is_infinite() {
            0.
        } else if d.abs() < 1e-14 {
            f64::INFINITY
        } else {
            1. / d
        };
    }
    (kind, k, ess)
}
fn step_metrics(t: &[f64], y: &[f64], tf: &TransferFunction, threshold: f64) -> Value {
    let final_value = y[y.len() - 1];
    let mut peak = 0;
    for i in 1..y.len() {
        if y[i] > y[peak] {
            peak = i
        }
    }
    let overshoot = if final_value.abs() > 1e-12 {
        Some(((y[peak] - final_value) / final_value.abs() * 100.).max(0.))
    } else {
        None
    };
    let (low, high) = if final_value >= 0. {
        (0.1 * final_value, 0.9 * final_value)
    } else {
        (0.9 * final_value, 0.1 * final_value)
    };
    let a = y.iter().position(|&v| v >= low);
    let b = y.iter().position(|&v| v >= high);
    let rise = a.zip(b).map(|(a, b)| (t[b] - t[a]).max(0.));
    let band = threshold * final_value.abs().max(1e-9);
    let outside = y.iter().rposition(|&v| (v - final_value).abs() > band);
    let settling = match outside {
        None => Some(0.),
        Some(i) if i + 1 < t.len() => Some(t[i + 1]),
        _ => None,
    };
    let wn = if tf.den.len() == 3 && tf.den[2] / tf.den[0] > 0. {
        Some((tf.den[2] / tf.den[0]).sqrt())
    } else {
        None
    };
    let zeta = wn.map(|w| tf.den[1] / tf.den[0] / (2. * w));
    json!({"final_value":final_value,"peak_value":y[peak],"overshoot_percent":overshoot,"rise_time":rise,"peak_time":t[peak],"settling_time":settling,"damping_ratio":zeta,"natural_frequency":wn})
}
pub fn analyze(p: &Value) -> Result<Value> {
    let threshold = number(p, "settling_threshold", 0.02)?;
    if threshold != 0.02 && threshold != 0.05 {
        return Err("O critério de acomodação deve ser 0,02 ou 0,05.".into());
    }
    let expr = p["expr"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("4 / (s^2 + 2*s + 4)");
    let tf = TransferFunction::parse(expr)?;
    if !tf.is_proper() {
        return Err("A função de transferência deve ser própria (grau do numerador ≤ grau do denominador) para análise temporal.".into());
    }
    let end = if p.get("final_time").is_some_and(|v| !v.is_null()) {
        Some(number(p, "final_time", 10.)?)
    } else {
        None
    };
    let t = simulation::time_vector(&tf, end, points(p)?)?;
    let y = simulation::simulate(&tf, &t, Input::Step)?;
    let ramp = simulation::simulate(&tf, &t, Input::Ramp)?;
    let impulse = simulation::simulate(&tf, &t, Input::Impulse)?;
    let poles = tf.poles()?;
    let stable = poles.iter().all(|v| v.re < -1e-9);
    let metrics = step_metrics(&t, &y, &tf, threshold);
    let (kind, k, ess) = constants(&tf);
    let mut width = 1288.;
    let dpi = p["dpi"]
        .as_u64()
        .or_else(|| {
            p["dpi"]
                .as_str()
                .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
                .and_then(|s| s.parse().ok())
        })
        .filter(|&v| v > 0)
        .unwrap_or(140) as f64;
    let mut height = 728.;
    if let Ok(w) = number(p, "width", 0.)
        && (200.0..=2400.0).contains(&w)
    {
        let inches = (w / dpi).clamp(4., 12.);
        width = inches * dpi;
        height = (inches * 0.56).max(2.6) * dpi;
    }
    if p.get("width").is_none() {
        width = 9.2 * dpi;
        height = 5.2 * dpi;
    }
    let mut step = Chart::new(
        width,
        height,
        (100., 25., width - 135., height - 95.),
        "Tempo (s)",
        "y(t)",
        false,
    );
    let min = y.iter().copied().fold(f64::INFINITY, f64::min);
    let max = y.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1e-3);
    step.bounds = (
        0.,
        t[t.len() - 1] * 1.09,
        (min - 0.04 * range).min(0.),
        max + 0.20 * range,
    );
    step.axes();
    step.line(&t, &y, "#000000", 2., "");
    let final_value = metrics["final_value"].as_f64().unwrap();
    let peak = metrics["peak_value"].as_f64().unwrap();
    let peak_time = metrics["peak_time"].as_f64().unwrap();
    step.line(
        &[0., t[t.len() - 1] * 1.09],
        &[final_value, final_value],
        "#64748B",
        1.2,
        "6 4",
    );
    step.label(
        t[t.len() - 1],
        final_value,
        &format!("{final_value:0.2}").replace('.', ","),
        6.,
        0.,
        false,
    );
    step.line(&[peak_time, peak_time], &[0., peak], "#334155", 1.1, "2 4");
    step.dot(peak_time, peak);
    let os = metrics["overshoot_percent"].as_f64();
    let label = if os.is_some_and(|v| v > 0.) {
        format!("Mp {:0.1}%", os.unwrap())
    } else {
        format!("{peak:0.2}")
    }
    .replace('.', ",");
    step.label(peak_time, peak, &label, 10., -8., true);
    step.legend(&[("Resposta ao degrau y(t)", "#000000", false)], false);
    let (mut image, mut svg) = step.finish()?;
    let step_plot = json!({"image":image,"svg":svg});
    let mut r = Chart::new(
        width,
        height,
        (100., 25., width - 135., height - 95.),
        "Tempo (s)",
        "y(t)",
        false,
    );
    r.autobounds(&t, &[&ramp, &t]);
    r.axes();
    r.line(&t, &t, "#64748B", 1.5, "6 4");
    r.line(&t, &ramp, "#000000", 2., "");
    r.legend(
        &[
            ("Entrada r(t) = t", "#64748B", true),
            ("Resposta à rampa y(t)", "#000000", false),
        ],
        true,
    );
    let (ri, rs) = r.finish()?;
    let mut im = Chart::new(
        width,
        height,
        (100., 25., width - 135., height - 95.),
        "Tempo (s)",
        "y_δ(t)",
        false,
    );
    im.autobounds(&t, &[&impulse]);
    im.axes();
    im.line(&t, &impulse, "#000000", 2., "");
    im.line(&[0., t[t.len() - 1]], &[0., 0.], "#000000", 0.8, "");
    let (ii, is) = im.finish()?;
    let plots =
        json!({"step":step_plot,"ramp":{"image":ri,"svg":rs},"impulse":{"image":ii,"svg":is}});
    let selected = p["response_type"].as_str().unwrap_or("step");
    if let Some(v) = plots.get(selected) {
        image = v["image"].as_str().unwrap().into();
        svg = v["svg"].as_str().unwrap().into();
    }
    let mut rows =
        vec![json!({"label":"Estabilidade","value":if stable{"Estável"}else{"Instável"}})];
    for (label, key, unit) in [
        ("Sobressinal (%OS)".into(), "overshoot_percent", Some("%")),
        (
            format!("Acomodação ({}%)", (threshold * 100.) as i32),
            "settling_time",
            Some("s"),
        ),
        ("Tempo de subida (tr)".into(), "rise_time", Some("s")),
        ("Tempo de pico (tp)".into(), "peak_time", Some("s")),
        ("Amortecimento (ζ)".into(), "damping_ratio", None),
        (
            "Frequência natural (ωn)".into(),
            "natural_frequency",
            Some(if metrics["natural_frequency"].is_null() {
                ""
            } else {
                "rad/s"
            }),
        ),
    ] {
        let mut row = json!({"label":label,"value":display(metrics[key].as_f64(),"—")});
        if let Some(u) = unit {
            row["unit"] = json!(u)
        }
        rows.push(row);
    }
    rows.push(json!({"label":"Tipo do sistema","value":format!("Tipo {kind}")}));
    for (label, v) in ["Kp", "Kv", "Ka"].iter().zip(k).chain(
        ["ess degrau", "ess rampa", "ess parabólica"]
            .iter()
            .zip(ess),
    ) {
        rows.push(json!({"label":label,"value":display(Some(v),"—")}));
    }
    let (latex, factored) = crate::transfer::expression_latex(expr)?;
    let expression = latex.trim_start_matches("G(s) = ");
    let factored_expression = factored.trim_start_matches("G(s) = ");
    let poles: Vec<Value> = poles.iter().map(|v| complex(*v)).collect();
    Ok(
        json!({"success":true,"module":"time_response","title":"Resposta no Domínio do Tempo","description":"Resposta temporal contínua com métricas de desempenho transitório e permanente.","image":image,"svg":svg,"plots":plots,
 "latex":[{"label":"Função de transferência","value":factored_expression},{"label":"Forma polinomial","value":expression},{"label":"Constantes de erro","value":r"K_p=\lim_{s\to0}G(s),\quad K_v=\lim_{s\to0}sG(s),\quad K_a=\lim_{s\to0}s^2G(s)"}],"metrics":rows,
 "details":{"stable":stable,"poles":poles,"step":metrics,"steady_state":{"type":kind,"kp":safe(k[0]),"kv":safe(k[1]),"ka":safe(k[2]),"ess_step":safe(ess[0]),"ess_ramp":safe(ess[1]),"ess_parabolic":safe(ess[2])},"latex_expanded":latex}}),
    )
}
pub(crate) fn complex(v: Complex64) -> Value {
    json!({"re":v.re,"im":v.im,"text":format!("{}{}{}j",display(Some(v.re),"—"),if v.im.is_sign_negative(){"-"}else{"+"},display(Some(v.im.abs()),"—"))})
}

/// Scientific SVG axes shared with the frequency compositor. All coordinates are
/// transformed from scientific values; curves are clipped to their own panel.
pub(crate) struct Chart {
    pub svg: String,
    pub bounds: (f64, f64, f64, f64),
    pub rect: (f64, f64, f64, f64),
    width: f64,
    height: f64,
    log: bool,
    xlabel: String,
    ylabel: String,
}
impl Chart {
    pub fn new(
        width: f64,
        height: f64,
        rect: (f64, f64, f64, f64),
        xlabel: &str,
        ylabel: &str,
        log: bool,
    ) -> Self {
        Self {
            svg: String::new(),
            bounds: (0., 1., 0., 1.),
            rect,
            width,
            height,
            log,
            xlabel: xlabel.into(),
            ylabel: ylabel.into(),
        }
    }
    pub fn autobounds(&mut self, x: &[f64], ys: &[&[f64]]) {
        let min = ys
            .iter()
            .flat_map(|v| v.iter())
            .copied()
            .filter(|v| v.is_finite())
            .fold(f64::INFINITY, f64::min);
        let max = ys
            .iter()
            .flat_map(|v| v.iter())
            .copied()
            .filter(|v| v.is_finite())
            .fold(f64::NEG_INFINITY, f64::max);
        let pad = if max == min {
            max.abs().max(1.) * 0.05
        } else {
            (max - min) * 0.05
        };
        let xp = (x[x.len() - 1] - x[0]) * 0.05;
        self.bounds = (
            if self.log { x[0] } else { x[0] - xp },
            if self.log {
                x[x.len() - 1]
            } else {
                x[x.len() - 1] + xp
            },
            min - pad,
            max + pad,
        );
    }
    pub(crate) fn xy(&self, x: f64, y: f64) -> (f64, f64) {
        let (a, b, c, d) = self.bounds;
        let (l, t, w, h) = self.rect;
        let xx = if self.log {
            (x.log10() - a.log10()) / (b.log10() - a.log10())
        } else {
            (x - a) / (b - a)
        };
        (l + w * xx, t + h - h * (y - c) / (d - c))
    }
    pub fn axes(&mut self) {
        let (l, t, w, h) = self.rect;
        let (a, b, c, d) = self.bounds;
        let mut x_ticks = vec![];
        if self.log {
            for e in a.log10().ceil() as i32..=b.log10().floor() as i32 {
                x_ticks.push((10_f64.powi(e), format!("10^{}", e)))
            }
        } else {
            for i in 0..=5 {
                let v = a + (b - a) * i as f64 / 5.;
                x_ticks.push((v, display(Some(v), "")))
            }
        }
        for (x, label) in x_ticks {
            let (px, _) = self.xy(x, c);
            self.svg += &format!(
                r##"<path d="M {px} {t} v {h}" stroke="#CBD5E1" stroke-opacity="0.75" stroke-dasharray="3 3"/><text x="{px}" y="{}" text-anchor="middle">{}</text>"##,
                t + h + 23.,
                crate::presentation::escape(&label)
            );
        }
        for i in 0..=5 {
            let y = c + (d - c) * i as f64 / 5.;
            let (_, py) = self.xy(a, y);
            self.svg += &format!(
                r##"<path d="M {l} {py} h {w}" stroke="#CBD5E1" stroke-opacity="0.75" stroke-dasharray="3 3"/><text x="{}" y="{}" text-anchor="end">{}</text>"##,
                l - 9.,
                py + 4.,
                display(Some(y), "")
            );
        }
        self.svg += &format!(
            r##"<rect x="{l}" y="{t}" width="{w}" height="{h}" fill="none" stroke="#000000"/><text x="{}" y="{}" text-anchor="middle">{}</text><text transform="translate({} {}) rotate(-90)" text-anchor="middle">{}</text>"##,
            l + w / 2.,
            t + h + 52.,
            crate::presentation::escape(&self.xlabel),
            l - 80.,
            t + h / 2.,
            crate::presentation::escape(&self.ylabel)
        );
    }
    pub fn line(&mut self, x: &[f64], y: &[f64], color: &str, width: f64, dash: &str) {
        let (l, t, w, h) = self.rect;
        let mut path = String::new();
        let mut started = false;
        for (&x, &y) in x.iter().zip(y) {
            let (px, py) = self.xy(x, y);
            if !px.is_finite() || !py.is_finite() {
                started = false;
                continue;
            }
            path += &format!(
                "{} {:0.5} {:0.5} ",
                if started { "L" } else { "M" },
                px.clamp(-1e8, 1e8),
                py.clamp(-1e8, 1e8)
            );
            started = true;
        }
        self.svg += &format!(
            r#"<svg x="{l}" y="{t}" width="{w}" height="{h}" viewBox="{l} {t} {w} {h}" overflow="hidden"><path d="{path}" stroke="{color}" stroke-width="{width}" stroke-dasharray="{dash}" fill="none"/></svg>"#
        );
    }
    pub fn label(&mut self, x: f64, y: f64, text: &str, dx: f64, dy: f64, bold: bool) {
        let (px, py) = self.xy(x, y);
        self.svg += &format!(
            r#"<text x="{}" y="{}" font-family="JetBrains Mono" font-weight="{}">{}</text>"#,
            px + dx,
            py + dy,
            if bold { "bold" } else { "normal" },
            crate::presentation::escape(text)
        );
    }
    pub fn dot(&mut self, x: f64, y: f64) {
        let (px, py) = self.xy(x, y);
        self.svg += &format!(r#"<circle cx="{px}" cy="{py}" r="4" fill="black" stroke="black"/>"#);
    }
    pub fn legend(&mut self, items: &[(&str, &str, bool)], upper: bool) {
        self.legend_at(items, upper, !upper);
    }
    pub fn legend_at(&mut self, items: &[(&str, &str, bool)], upper: bool, right: bool) {
        let (l, t, w, h) = self.rect;
        let width = items.iter().map(|v| v.0.chars().count()).max().unwrap_or(0) as f64 * 7.7 + 45.;
        let height = items.len() as f64 * 22. + 10.;
        let x = if right { l + w - width - 8. } else { l + 8. };
        let y = if upper { t + 8. } else { t + h - height - 8. };
        self.svg += &format!(
            r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="white" fill-opacity="0.85"/>"#
        );
        for (i, (label, color, dash)) in items.iter().enumerate() {
            let yy = y + 20. + 22. * i as f64;
            self.svg += &format!(
                r#"<path d="M {} {} h 22" stroke="{color}" stroke-width="2" stroke-dasharray="{}"/><text x="{}" y="{yy}">{}</text>"#,
                x + 7.,
                yy - 4.,
                if *dash { "6 4" } else { "" },
                x + 36.,
                crate::presentation::escape(label)
            );
        }
    }
    pub fn finish(self) -> Result<(String, String)> {
        let svg = svg_document(self.width, self.height, &self.svg);
        let image = crate::presentation::rasterize(&svg)?;
        Ok((image, svg))
    }
}
pub(crate) fn svg_document(width: f64, height: f64, body: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><rect width="100%" height="100%" fill="white"/><g font-family="IBM Plex Sans" font-size="14" fill="black">{body}</g></svg>"#
    )
}
