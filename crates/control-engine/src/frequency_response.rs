//! Continuous Bode/Nyquist analysis and python-control polynomial margins.
use crate::{
    Result, polynomial as poly,
    time_response::{Chart, display, number, points, safe, svg_document},
    transfer::TransferFunction,
};
use num_complex::Complex64 as C;
use serde_json::{Value, json};

pub fn presets() -> Value {
    json!([
 {"id":"Primeira ordem","title":"Primeira ordem","expr":"1 / (s + 1)"},
 {"id":"Segunda ordem","title":"Segunda ordem","expr":"25 / (s^2 + 4*s + 25)"},
 {"id":"Tipo 1 com dois polos","title":"Tipo 1 com dois polos","expr":"10 / (s * (s + 2) * (s + 5))"}])
}
fn iw(p: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut re = vec![0.; p.len()];
    let mut im = re.clone();
    for (i, &v) in p.iter().enumerate() {
        match (p.len() - 1 - i) % 4 {
            0 => re[i] = v,
            1 => im[i] = v,
            2 => re[i] = -v,
            _ => im[i] = -v,
        }
    }
    (poly::trim(&re), poly::trim(&im))
}
fn sqr(p: &(Vec<f64>, Vec<f64>)) -> Vec<f64> {
    poly::add(&poly::multiply(&p.0, &p.0), &poly::multiply(&p.1, &p.1), 1.)
}
fn real_roots(p: &[f64], inclusive: bool) -> Result<Vec<f64>> {
    let mut roots: Vec<f64> = poly::roots(p)?
        .iter()
        .filter(|v| v.im == 0. && (if inclusive { v.re >= 0. } else { v.re > 0. }))
        .map(|v| v.re)
        .collect();
    roots.sort_by(f64::total_cmp);
    Ok(roots)
}
#[derive(Debug)]
pub(crate) struct Margins {
    pub(crate) gain: f64,
    pub(crate) phase: f64,
    pub(crate) stability: f64,
    pub(crate) phase_cross: f64,
    pub(crate) gain_cross: f64,
    pub(crate) stability_cross: f64,
}
/// Algebraic crossing equations from control.margins, independent of plot range.
pub(crate) fn margins(tf: &TransferFunction) -> Result<Margins> {
    let n = iw(&tf.num);
    let d = iw(&tf.den);
    let phase_equation = poly::add(
        &poly::multiply(&n.1, &d.0),
        &poly::multiply(&n.0, &d.1),
        -1.,
    );
    let phase_roots = real_roots(&phase_equation, true)?;
    let gain_equation = poly::add(&sqr(&n), &sqr(&d), -1.);
    let gain_roots = real_roots(&gain_equation, false)?;
    let sum = (poly::add(&n.0, &d.0, 1.), poly::add(&n.1, &d.1, 1.));
    let sn = sqr(&sum);
    let sd = sqr(&d);
    let stab_equation = poly::add(
        &poly::multiply(&poly::derivative(&sn), &sd),
        &poly::multiply(&poly::derivative(&sd), &sn),
        -1.,
    );
    let stab_deriv = poly::derivative(&stab_equation);
    let stab_roots = real_roots(&stab_equation, false)?;
    let mut result = Margins {
        gain: f64::INFINITY,
        phase: f64::INFINITY,
        stability: f64::INFINITY,
        phase_cross: f64::NAN,
        gain_cross: f64::NAN,
        stability_cross: f64::NAN,
    };
    for w in phase_roots {
        let response = tf.evaluate(C::new(0., w));
        if response.re <= 0. {
            let gain = 1. / response.norm();
            if gain.ln().abs() < result.gain.ln().abs() {
                result.gain = gain;
                result.phase_cross = w;
            }
        }
    }
    for w in gain_roots {
        let phase = tf
            .evaluate(C::new(0., w))
            .arg()
            .to_degrees()
            .rem_euclid(360.)
            - 180.;
        if phase.abs() < result.phase.abs() {
            result.phase = phase;
            result.gain_cross = w;
        }
    }
    for w in stab_roots {
        if poly::evaluate(&stab_deriv, C::new(w, 0.)).re > 0. {
            let stability = (tf.evaluate(C::new(0., w)) + C::new(1., 0.)).norm();
            if stability < result.stability {
                result.stability = stability;
                result.stability_cross = w;
            }
        }
    }
    Ok(result)
}
fn logspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| 10_f64.powf(a + (b - a) * i as f64 / (n - 1) as f64))
        .collect()
}
/// Match LTI.bandwidth: default feature grid first, then bisection at -3 dB.
fn bandwidth(tf: &TransferFunction) -> Result<Option<f64>> {
    let dc = tf.dc_gain();
    if dc.is_infinite() {
        return Ok(Some(f64::INFINITY));
    }
    let mut features = tf.poles()?;
    features.extend(tf.zeros()?);
    let mut f: Vec<f64> = features
        .iter()
        .map(|v| v.norm())
        .filter(|v| *v > 1e-8)
        .collect();
    if f.is_empty() {
        f.push(1.)
    }
    let lo = (f.iter().copied().fold(f64::INFINITY, f64::min).log10() - 1.).round_ties_even();
    let hi = (f.iter().copied().fold(f64::NEG_INFINITY, f64::max).log10() + 1.).round_ties_even();
    let omega = logspace(lo, hi, 1000);
    let factor = 10_f64.powf(-3. / 20.);
    let Some(idx) = omega
        .iter()
        .position(|&w| tf.evaluate(C::new(0., w)).norm() - dc * factor < 0.)
    else {
        return Ok(Some(f64::INFINITY));
    };
    let mut a = omega[if idx == 0 { omega.len() - 1 } else { idx - 1 }];
    let mut b = omega[idx];
    let target = dc.abs() * factor;
    let eval = |w: f64| tf.evaluate(C::new(0., w)).norm() - target;
    let mut fa = eval(a);
    let fb = eval(b);
    if fa == 0. {
        return Ok(Some(a.abs()));
    }
    if fb == 0. {
        return Ok(Some(b.abs()));
    }
    if !fa.is_finite() || !fb.is_finite() || fa.signum() == fb.signum() {
        return Ok(None);
    }
    for _ in 0..100 {
        let c = (a + b) / 2.;
        let fc = eval(c);
        if fc == 0. || (b - a).abs() / 2. <= 2e-12 + 4. * f64::EPSILON * c.abs() {
            return Ok(Some(c.abs()));
        }
        if fa.signum() == fc.signum() {
            a = c;
            fa = fc
        } else {
            b = c
        }
    }
    Ok(None)
}
fn asymptote(tf: &TransferFunction, w: &[f64], db: &[f64]) -> Result<Vec<f64>> {
    let zeros = tf.zeros()?;
    let poles = tf.poles()?;
    let mut slope = 20.
        * (zeros.iter().filter(|v| v.norm() <= 1e-10).count() as f64
            - poles.iter().filter(|v| v.norm() <= 1e-10).count() as f64);
    let mut events: Vec<(f64, f64)> = zeros
        .iter()
        .filter(|v| v.norm() > 1e-10)
        .map(|v| (v.norm(), 20.))
        .chain(
            poles
                .iter()
                .filter(|v| v.norm() > 1e-10)
                .map(|v| (v.norm(), -20.)),
        )
        .collect();
    events.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut j = 0;
    let mut y = vec![db[0]];
    for i in 1..w.len() {
        let midpoint = (w[i - 1] * w[i]).sqrt();
        while j < events.len() && events[j].0 <= midpoint {
            slope += events[j].1;
            j += 1
        }
        y.push(y[i - 1] + slope * (w[i] / w[i - 1]).log10())
    }
    Ok(y)
}
fn metric(label: &str, value: f64, unit: Option<&str>) -> Value {
    let mut v = json!({"label":label,"value":display(Some(value),"Não definido")});
    if let Some(unit) = unit {
        v["unit"] = json!(unit)
    }
    v
}

pub fn analyze(p: &Value) -> Result<Value> {
    let omega_min = number(p, "omega_min", 0.01)?;
    let omega_max = number(p, "omega_max", 100.)?;
    if omega_min <= 0. || omega_max <= omega_min || !omega_min.is_finite() || !omega_max.is_finite()
    {
        return Err("Use frequências positivas com ω máximo maior que ω mínimo.".into());
    }
    if omega_max / omega_min > 1e12 {
        return Err("A faixa de frequências não pode exceder 12 décadas.".into());
    }
    let tf = TransferFunction::parse(
        p["expr"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("10 / (s * (s + 2) * (s + 5))"),
    )?;
    let omega = logspace(omega_min.log10(), omega_max.log10(), points(p)?);
    let response: Vec<C> = omega.iter().map(|&w| tf.evaluate(C::new(0., w))).collect();
    let magnitude: Vec<f64> = response.iter().map(|v| v.norm()).collect();
    let db: Vec<f64> = magnitude
        .iter()
        .map(|v| 20. * v.max(f64::MIN_POSITIVE).log10())
        .collect();
    let mut phase: Vec<f64> = vec![];
    let mut offset = 0.;
    let mut last = 0.;
    for (i, r) in response.iter().enumerate() {
        let v = r.arg();
        if i > 0 {
            let delta = v - last;
            if delta > std::f64::consts::PI {
                offset -= 2. * std::f64::consts::PI
            } else if delta < -std::f64::consts::PI {
                offset += 2. * std::f64::consts::PI
            }
        }
        phase.push((v + offset).to_degrees());
        last = v;
    }
    let asym = asymptote(&tf, &omega, &db)?;
    let m = margins(&tf)?;
    let gain_db = if m.gain.is_infinite() {
        f64::INFINITY
    } else {
        20. * m.gain.max(f64::MIN_POSITIVE).log10()
    };
    let closed = tf.feedback(&TransferFunction::scalar(1.), -1.)?;
    let closed_mag: Vec<f64> = omega
        .iter()
        .map(|&w| closed.evaluate(C::new(0., w)).norm())
        .collect();
    let mut resonance = 0;
    for i in 1..closed_mag.len() {
        if closed_mag[i] > closed_mag[resonance] {
            resonance = i;
        }
    }
    let bw = bandwidth(&closed)?;
    let open_poles = tf.poles()?;
    let closed_poles = closed.poles()?;
    let pc = open_poles.iter().filter(|v| v.re > 1e-8).count() as i32;
    let zc = closed_poles.iter().filter(|v| v.re > 1e-8).count() as i32;
    let nc = zc - pc;
    let stable = closed_poles.iter().all(|v| v.re < -1e-9);
    let origin = open_poles.iter().any(|v| v.norm() < 1e-8);
    let asym_re = if origin {
        Some(tf.evaluate(C::new(0., 1e-6)).re)
    } else {
        None
    };
    // Bode magnitude/phase left, Nyquist spanning the complete right column.
    let width = 1568.;
    let height = 1008.;
    let mut mag = Chart::new(
        width,
        height,
        (115., 35., 660., 385.),
        "",
        "Magnitude (dB)",
        true,
    );
    mag.autobounds(&omega, &[&db, &asym]);
    mag.axes();
    mag.line(&omega, &db, "#000000", 2., "");
    mag.line(&omega, &asym, "#64748B", 1.5, "6 4");
    mag.legend_at(
        &[
            ("Curva real", "#000000", false),
            ("Assíntotas", "#64748B", true),
        ],
        true,
        true,
    );
    let mut ph = Chart::new(
        width,
        height,
        (115., 530., 660., 385.),
        "Frequência (rad/s)",
        "Fase (graus)",
        true,
    );
    ph.autobounds(&omega, &[&phase]);
    ph.axes();
    ph.line(&omega, &phase, "#000000", 2., "");
    let re: Vec<f64> = response.iter().map(|v| v.re).collect();
    let im: Vec<f64> = response.iter().map(|v| v.im).collect();
    let mirrored: Vec<f64> = im.iter().map(|v| -v).collect();
    let mut refs = vec![-1., 0.];
    if m.gain.is_finite() && m.gain > 0. {
        refs.push(-1. / m.gain)
    }
    if let Some(v) = asym_re.filter(|v| v.is_finite()) {
        refs.push(v)
    }
    let relevant: Vec<usize> = response
        .iter()
        .enumerate()
        .filter(|(_, v)| v.norm() <= 8. && v.re.is_finite() && v.im.is_finite())
        .map(|(i, _)| i)
        .collect();
    let (mut x_min, mut x_max, mut y_max) = if relevant.is_empty() {
        (-2., 1., 1.5)
    } else {
        (
            relevant
                .iter()
                .map(|&i| re[i])
                .chain(refs.iter().copied())
                .fold(f64::INFINITY, f64::min)
                - 0.4,
            relevant
                .iter()
                .map(|&i| re[i])
                .chain(refs.iter().copied())
                .fold(f64::NEG_INFINITY, f64::max)
                + 0.4,
            relevant.iter().map(|&i| im[i].abs()).fold(1.2, f64::max) + 0.3,
        )
    };
    x_min = x_min.clamp(-50., -1.2);
    x_max = x_max.clamp(0.5, 50.);
    y_max = y_max.clamp(1.2, 50.);
    let span_x = x_max - x_min;
    let span_y = 2. * y_max;
    if span_x / span_y > 3. {
        y_max = span_x / 3.
    } else if span_y / span_x > 3. {
        let span = span_y / 3.;
        let mid = (x_max + x_min) / 2.;
        x_min = mid - span / 2.;
        x_max = mid + span / 2.;
    }
    let mut ny = Chart::new(
        width,
        height,
        (900., 35., 600., 880.),
        "Parte real",
        "Parte imaginária",
        false,
    );
    ny.bounds = (x_min, x_max, -y_max, y_max);
    ny.axes();
    ny.line(&re, &im, "#000000", 2., "");
    ny.line(&re, &mirrored, "#000000", 1.4, "6 4");
    ny.line(&[x_min, x_max], &[0., 0.], "#000000", 0.8, "");
    ny.line(&[0., 0.], &[-y_max, y_max], "#000000", 0.8, "");
    let dx = (x_max - x_min) * 0.009;
    let dy = 2. * y_max * 0.009;
    ny.line(&[-1. - dx, -1. + dx], &[-dy, dy], "#B91C1C", 2.2, "");
    ny.line(&[-1. - dx, -1. + dx], &[dy, -dy], "#B91C1C", 2.2, "");
    let asym_label = asym_re
        .filter(|v| v.is_finite())
        .map(|v| format!("Assíntota Re ≈ {v:0.2}"));
    if let Some(v) = asym_re.filter(|v| v.is_finite()) {
        ny.line(&[v, v], &[-y_max, y_max], "#64748B", 1.2, "2 4")
    }
    let mut legend = vec![
        ("ω ≥ 0", "#000000", false),
        ("ω < 0", "#000000", true),
        ("Ponto crítico (-1, 0)", "#B91C1C", false),
    ];
    if let Some(label) = asym_label.as_deref() {
        legend.push((label, "#64748B", true))
    }
    ny.legend_at(&legend, true, true);
    let svg = svg_document(width, height, &format!("{}{}{}", mag.svg, ph.svg, ny.svg));
    let image = crate::presentation::rasterize(&svg)?;
    let mut rows = vec![
        json!({"header":"Margens"}),
        json!({"label":"Estabilidade (malha fechada)","value":if stable{"Estável"}else{"Instável"}}),
        metric("Margem de ganho (MG)", gain_db, Some("dB")),
        metric("Cruzamento de fase (ωcf)", m.phase_cross, Some("rad/s")),
        metric("Margem de fase (MF)", m.phase, Some("°")),
        metric("Cruzamento de ganho (ωcg)", m.gain_cross, Some("rad/s")),
        json!({"label":"Critério de Nyquist (Z = N + P)","value":format!("{zc} = {nc} + {pc}")}),
    ];
    if origin {
        rows.push(json!({"label":"Observação","value":"Malha aberta com polo na origem"}))
    }
    if stable && m.phase > 0. && m.phase < 90. {
        let zeta = m.phase / 100.;
        let os = (-std::f64::consts::PI * zeta / (1. - zeta * zeta).sqrt()).exp() * 100.;
        rows.push(json!({"header":"Estimativas no tempo"}));
        rows.push(metric("Amortecimento estimado (ζ)", zeta, None));
        rows.push(metric("Sobressinal estimado (%OS)", os, Some("%")));
        if let Some(b) = bw.filter(|v| v.is_finite() && *v > 0.) {
            rows.push(metric("Largura de banda (ωBW)", b, Some("rad/s")));
            rows.push(metric("Tempo de subida estimado (tr)", 1.8 / b, Some("s")));
            rows.push(metric(
                "Tempo de acomodação (ts, 2%)",
                4. / (zeta * b),
                Some("s"),
            ))
        }
        if closed_mag[resonance].is_finite() {
            rows.push(metric(
                "Pico de ressonância (Mr)",
                closed_mag[resonance],
                None,
            ));
            rows.push(metric(
                "Frequência de ressonância",
                omega[resonance],
                Some("rad/s"),
            ))
        }
    }
    let (latex, factored) = crate::transfer::expression_latex(
        p["expr"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("10 / (s * (s + 2) * (s + 5))"),
    )?;
    Ok(
        json!({"success":true,"module":"frequency_response","title":"Resposta em Frequência","description":"Bode real e assintótico, Nyquist e métricas de estabilidade relativa.","image":image,"svg":svg,
 "latex":[{"label":"Planta","value":factored.trim_start_matches("G(s) = ")},{"label":"Resposta senoidal","value":r"G(j\omega)=G(s)\vert_{s=j\omega}"},{"label":"Critério de Nyquist","value":format!(r"Z=N+P\;\Rightarrow\;{zc}={nc}+{pc}")}],"metrics":rows,
 "details":{"stable":stable,"has_origin_pole":origin,"asymptote_re":asym_re.map(safe),"gain_margin":safe(m.gain),"gain_margin_db":safe(gain_db),"phase_margin_deg":safe(m.phase),"phase_crossover_frequency":safe(m.phase_cross),"gain_crossover_frequency":safe(m.gain_cross),"stability_margin":safe(m.stability),"stability_margin_frequency":safe(m.stability_cross),"bandwidth":bw.map(safe),"resonance_peak":closed_mag[resonance],"resonance_peak_db":20.*closed_mag[resonance].max(f64::MIN_POSITIVE).log10(),"resonance_frequency":omega[resonance],"nyquist":{"Z":zc,"N":nc,"P":pc},"latex_expanded":latex}}),
    )
}
