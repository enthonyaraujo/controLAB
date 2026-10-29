//! Classical seven-step root locus, including the original fixed gain grid.
use crate::{
    Result, input, polynomial as poly, time_response::Chart, transfer::TransferFunction, values,
};
use num_complex::Complex64 as C;
use serde_json::{Value, json};

type Preset = (
    &'static str,
    &'static str,
    &'static [f64],
    &'static [f64],
    &'static str,
);
const PRESETS: [Preset; 6] = [
    (
        "Exemplo 1: 3ª Ordem com Breakaway e Cruzamento jω (Ogata)",
        "(s + 2) / (s * (s + 1) * (s + 4))",
        &[1., 2.],
        &[1., 5., 4., 0.],
        "Polos em 0, -1, -4 e Zero em -2. Apresenta ponto de partida (breakaway) e cruzamento com o eixo imaginário.",
    ),
    (
        "Exemplo 2: Polos Complexos Conjugados e Ângulo de Partida",
        "1 / (s * (s^2 + 2*s + 2))",
        &[1.],
        &[1., 2., 2., 0.],
        "Polo na origem e par complexo conjugado (-1 ± 1j). Demonstra o cálculo exato do ângulo de partida theta_d.",
    ),
    (
        "Exemplo 3: Zero de Fase Não-Mínima (Semiplano Direito)",
        "(s - 2) / (s * (s + 1) * (s + 3))",
        &[1., -2.],
        &[1., 4., 3., 0.],
        "Sistema com zero de fase não-mínima no semiplano direito (s = +2), atraindo um ramo para a região instável.",
    ),
    (
        "Exemplo 4: Sistema com Break-in e Breakaway",
        "(s + 2) / (s * (s + 1))",
        &[1., 2.],
        &[1., 1., 0.],
        "Dois polos (0, -1) e um zero (-2). Os ramos saem do eixo real e retornam a ele (break-in).",
    ),
    (
        "Exemplo 5: 4ª Ordem com 4 Assíntotas",
        "1 / (s * (s + 1) * (s + 2) * (s + 3))",
        &[1.],
        &[1., 6., 11., 6., 0.],
        "4 polos reais na origem e em -1, -2, -3. Quatro assíntotas simétricas a 45°, 135°, 225° e 315°.",
    ),
    (
        "Exemplo 6: Sistema de 2ª Ordem com Amortecimento",
        "1 / (s^2 + 2*s + 5)",
        &[1.],
        &[1., 2., 5.],
        "Par de polos conjugados em -1 ± 2j com ramos paralelos assintóticos verticais.",
    ),
];
pub fn presets() -> Value {
    json!(PRESETS.iter().map(|(id,expr,num,den,desc)|json!({"id":id,"title":id,"expr":expr,"num":num,"den":den,"desc":desc})).collect::<Vec<_>>())
}
fn format_complex(v: C) -> String {
    if v.im.abs() < 1e-5 {
        format!("{:0.2}", v.re)
    } else if v.re.abs() < 1e-5 {
        format!("{:+0.2}j", v.im)
    } else {
        format!("{:0.2} {:+0.2}j", v.re, v.im)
    }
}
fn real_num(v: f64) -> String {
    if (v - v.round_ties_even()).abs() < 1e-5 {
        format!("{:0.0}", v.round_ties_even())
    } else {
        fixed_trim(v, 2)
    }
}
fn fixed_trim(v: f64, digits: usize) -> String {
    format!("{v:.digits$}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .into()
}
fn float_latex(v: f64) -> String {
    let s = values::general(v, 15);
    if let Some((m, e)) = s.split_once('e') {
        let m = if m.contains('.') {
            m.to_owned()
        } else {
            format!("{m}.0")
        };
        format!("{m} \\cdot 10^{{{}}}", e.parse::<i32>().unwrap())
    } else if s.contains('.') {
        s
    } else {
        format!("{s}.0")
    }
}
fn join(terms: Vec<(String, bool)>) -> String {
    let mut out = String::new();
    for (term, negative) in terms {
        if out.is_empty() {
            if negative {
                out.push_str("- ")
            }
        } else {
            out.push_str(if negative { " - " } else { " + " })
        }
        out.push_str(&term)
    }
    if out.is_empty() { "0".into() } else { out }
}
fn polynomial_terms(
    p: &[f64],
    variable: &str,
    k: bool,
    imaginary: Option<bool>,
) -> Vec<(String, bool)> {
    let mut terms = vec![];
    for (i, &coefficient) in p.iter().enumerate() {
        let power = p.len() - 1 - i;
        if coefficient == 0. {
            continue;
        }
        if let Some(imaginary) = imaginary
            && (power % 2 == 1) != imaginary
        {
            continue;
        }
        let c = if imaginary.is_some() && power % 4 >= 2 {
            -coefficient
        } else {
            coefficient
        };
        let mut term = float_latex(c.abs());
        if k {
            term.push_str(" K")
        };
        if power > 0 {
            term.push(' ');
            term.push_str(variable);
            if power > 1 {
                term.push_str(&format!("^{{{power}}}"))
            }
        }
        terms.push((term, c < 0.));
    }
    terms
}
fn float_poly(p: &[f64]) -> String {
    join(polynomial_terms(p, "s", false, None))
}
fn characteristic(tf: &TransferFunction, imag: Option<bool>) -> String {
    let variable = if imag.is_some() { "w" } else { "s" };
    let mut terms = polynomial_terms(&tf.num, variable, true, imag);
    terms.extend(polynomial_terms(&tf.den, variable, false, imag));
    join(terms) + " = 0"
}
fn expanded(tf: &TransferFunction, force_float: bool) -> String {
    if force_float {
        format!(
            "G(s) = \\frac{{{}}}{{{}}}",
            float_poly(&tf.num),
            float_poly(&tf.den)
        )
    } else {
        tf.latex()
    }
}

/// `_RLFindRoots` and `_RLSortRoots`: each current root chooses its nearest
/// available predecessor, rather than each predecessor choosing a new root.
fn locus(tf: &TransferFunction) -> Result<(Vec<f64>, Vec<Vec<C>>)> {
    let mut gains = vec![0.];
    gains.extend((0..10000).map(|i| 10_f64.powf(-3. + 7. * i as f64 / 9999.)));
    let mut rows: Vec<Vec<C>> = Vec::with_capacity(gains.len());
    for &gain in &gains {
        let mut roots = poly::roots(&poly::add(&tf.den, &tf.num, gain))?;
        if roots.len() < tf.den.len() - 1 {
            roots.push(C::new(f64::INFINITY, 0.))
        }
        roots.sort_by(|a, b| a.re.total_cmp(&b.re).then(a.im.total_cmp(&b.im)));
        if let Some(prev) = rows.last() {
            if prev.len() != roots.len() {
                return Err("O número de raízes varia na faixa de ganhos do LGR.".into());
            }
            let mut available: Vec<usize> = (0..prev.len()).collect();
            let mut ordered = vec![C::new(0., 0.); roots.len()];
            for root in roots {
                let best = available
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        (root - prev[**a])
                            .norm()
                            .total_cmp(&(root - prev[**b]).norm())
                    })
                    .map(|(i, _)| i)
                    .unwrap();
                let index = available.remove(best);
                ordered[index] = root;
            }
            rows.push(ordered)
        } else {
            rows.push(roots)
        }
    }
    Ok((gains, rows))
}
fn angles(points: &[C], other: &[C], arrival: bool) -> (Vec<Value>, Vec<Value>) {
    let mut brief = vec![];
    let mut deductions = vec![];
    for &point in points.iter().filter(|v| v.im > 1e-4) {
        let mut other_terms = vec![];
        let mut same_terms = vec![];
        let mut other_sum = 0.;
        let mut same_sum = 0.;
        for &p in other {
            let diff = point - p;
            let angle = diff.arg().to_degrees();
            other_sum += angle;
            let key = if arrival { "polo_str" } else { "zero_str" };
            let mut term = json!({"vetor_str":format_complex(diff),"angulo":angle});
            term[key] = json!(format_complex(p));
            other_terms.push(term)
        }
        for &p in points {
            if (point - p).norm() > 1e-4 {
                let diff = point - p;
                let angle = diff.arg().to_degrees();
                same_sum += angle;
                let key = if arrival { "zero_str" } else { "polo_str" };
                let mut term = json!({"vetor_str":format_complex(diff),"angulo":angle});
                term[key] = json!(format_complex(p));
                same_terms.push(term)
            }
        }
        let angle = (180. + other_sum - same_sum + 180.).rem_euclid(360.) - 180.;
        let key = if arrival { "zero_str" } else { "polo_str" };
        let mut b = json!({"angulo":angle});
        b[key] = json!(format_complex(point));
        brief.push(b);
        let mut detail = json!({"angulo_final":angle});
        detail[key] = json!(format_complex(point));
        if arrival {
            detail["termos_polos"] = json!(other_terms);
            detail["termos_zeros"] = json!(same_terms);
            detail["soma_polos_graus"] = json!(other_sum);
            detail["soma_zeros_graus"] = json!(same_sum);
            detail["formula_aplicada"] =
                json!(r"\theta_a = 180^\circ + \sum \phi_p - \sum \theta_z");
            detail["calculo_substituicao"] = json!(format!(
                r"\theta_a = 180^\circ + ({other_sum:0.1}^\circ) - ({same_sum:0.1}^\circ) = {angle:0.1}^\circ"
            ))
        } else {
            detail["termos_zeros"] = json!(other_terms);
            detail["termos_polos"] = json!(same_terms);
            detail["soma_zeros_graus"] = json!(other_sum);
            detail["soma_polos_graus"] = json!(same_sum);
            detail["formula_aplicada"] =
                json!(r"\theta_d = 180^\circ + \sum \phi_z - \sum \theta_p");
            detail["calculo_substituicao"] = json!(format!(
                r"\theta_d = 180^\circ + ({other_sum:0.1}^\circ) - ({same_sum:0.1}^\circ) = {angle:0.1}^\circ"
            ))
        }
        deductions.push(detail);
    }
    (brief, deductions)
}
#[allow(clippy::too_many_arguments)]
fn seven_steps(
    tf: &TransferFunction,
    poles: &[C],
    zeros: &[C],
    centroid: Option<f64>,
    break_roots: &[C],
    crossings: &[Value],
    departure: &[Value],
    arrival: &[Value],
) -> Value {
    let p = poles.len();
    let z = zeros.len();
    let mut real: Vec<f64> = poles
        .iter()
        .chain(zeros)
        .filter(|v| v.im.abs() < 1e-4)
        .map(|v| v.re)
        .collect();
    real.sort_by(|a, b| b.total_cmp(a));
    let mut segments = vec![];
    if !real.is_empty() {
        segments.push(json!({"intervalo":format!("({}, +\\infty)",real_num(real[0])),"contagem_direita":0,"pertence":false,"motivo":"0 singularidades reais à direita (quantidade par)"}));
        for i in 0..real.len() {
            let count = i + 1;
            let belongs = count % 2 == 1;
            let interval = if i + 1 < real.len() {
                format!("({}, {})", real_num(real[i + 1]), real_num(real[i]))
            } else {
                format!("(-\\infty, {})", real_num(real[i]))
            };
            segments.push(json!({"intervalo":interval,"contagem_direita":count,"pertence":belongs,"motivo":format!("{count} singularidade(s) real(is) à direita ({})",if belongs{"ímpar: no LGR"}else{"par: fora do LGR"})}));
        }
    }
    let n = float_poly(&tf.num);
    let d = float_poly(&tf.den);
    let mut step4 = json!({"tem_assintotas":p>z,"n_assintotas":p.saturating_sub(z),"formula_centroide":r"\sigma_a = \frac{\sum_{i=1}^P p_i - \sum_{j=1}^Z z_j}{P - Z}","formula_angulos":r"\theta_k = \frac{(2k + 1) \cdot 180^\circ}{P - Z}, \quad k = 0, \dots, (P - Z - 1)","soma_polos_num":poles.iter().map(|v|v.re).sum::<f64>(),"soma_zeros_num":zeros.iter().map(|v|v.re).sum::<f64>(),"substituicao_centroide":"","angulos_deduzidos":[]});
    if let Some(c) = centroid {
        let sp = poles.iter().map(|v| v.re).fold(0.0, |a, b| a + b);
        let sz = zeros.iter().map(|v| v.re).fold(0.0, |a, b| a + b);
        let diff = p - z;
        step4["substituicao_centroide"] = json!(format!(
            r"\sigma_a = \frac{{({}) - ({})}}{{{diff}}} = \frac{{{:0.2}}}{{{diff}}} = {c:0.3}",
            fixed_trim(sp, 2),
            fixed_trim(sz, 2),
            sp - sz
        ));
        step4["angulos_deduzidos"]=json!((0..diff).map(|k|{let deg=(2*k+1)as f64*180./diff as f64;json!({"k":k,"formula":format!(r"\theta_{{{k}}} = \frac{{(2({k}) + 1) \cdot 180^\circ}}{{{diff}}} = \frac{{{}^\circ}}{{{diff}}} = {deg:0.1}^\circ",(2*k+1)*180),"graus":deg.rem_euclid(360.)})}).collect::<Vec<_>>());
    }
    let mut analyzed = vec![];
    for r in break_roots {
        if r.im.abs() > 1e-4 {
            analyzed.push(json!({"s_str":format!("{:0.2} ± j{:0.2}",r.re,r.im.abs()),"tipo":"complexa","valido":false,"motivo":"Raiz com parte imaginária (fora do eixo real)"}));
            continue;
        }
        let value = poly::evaluate(&tf.num, C::new(r.re, 0.)).re;
        let (k, valid, reason) = if value.abs() < 1e-8 {
            (
                None,
                false,
                "Raiz coincide com zero finito do numerador".to_owned(),
            )
        } else {
            let k = -poly::evaluate(&tf.den, C::new(r.re, 0.)).re / value;
            let on = real.iter().filter(|&&v| v > r.re).count() % 2 == 1;
            if k > 0. && on {
                (
                    Some(k),
                    true,
                    format!("Ponto de Quebra Válido ($K = {k:0.2} > 0$ e pertence ao LGR)"),
                )
            } else if k <= 0. {
                (
                    Some(k),
                    false,
                    format!(r"Descartado: $K = {k:0.2} \le 0$ (requer ganho positivo)"),
                )
            } else {
                (
                    Some(k),
                    false,
                    format!(
                        "Descartado: ponto $s = {:0.2}$ fora dos ramos reais do LGR",
                        r.re
                    ),
                )
            }
        };
        analyzed.push(json!({"s_str":format!("{:0.3}",r.re),"s_val":r.re,"tipo":"real","K":k,"valido":valid,"motivo":reason}));
    }
    let equation = poly::add(
        &poly::multiply(&poly::derivative(&tf.num), &tf.den),
        &poly::multiply(&tf.num, &poly::derivative(&tf.den)),
        -1.,
    );
    let mut seen = std::collections::HashSet::new();
    let mut unique = vec![];
    for v in crossings {
        let w = v["w"].as_f64().unwrap().abs();
        let k = v["K"].as_f64().unwrap();
        if seen.insert(format!("{w:0.2}_{k:0.2}")) {
            unique.push(json!({"w":w,"K":k,"s_str":format!(r"s = \pm j{w:0.2}"),"K_str":format!("K_{{crit}} = {k:0.2}")}));
        }
    }
    json!({"passo_1_2_3":{"eq_malha_aberta":expanded(tf,true),"eq_fatorada":expanded(tf,true),"eq_caracteristica":format!(r"1 + K \, G(s) = 0 \iff ({d}) + K ({n}) = 0"),"P":p,"Z":z,"ramos":p.max(z),"segmentos_eixo_real":segments,"simetria_justificativa":"Como os coeficientes de N(s) e D(s) são estritamente reais, as raízes complexas de D(s) + K N(s) = 0 ocorrem obrigatoriamente em pares conjugados, garantindo simetria reflexiva ao longo do eixo real."},
 "passo_4":step4,
 "passo_5":{"formula_derivada":r"K(s) = -\frac{D(s)}{N(s)} \implies \frac{dK}{ds} = -\frac{D'(s)N(s) - D(s)N'(s)}{N(s)^2} = 0","condicao":r"N'(s)D(s) - N(s)D'(s) = 0","derivada_N":float_poly(&poly::derivative(&tf.num)),"derivada_D":float_poly(&poly::derivative(&tf.den)),"polinomio_break":format!("{} = 0",float_poly(&equation)),"raizes_analisadas":analyzed},
 "passo_6":{"eq_caracteristica":characteristic(tf,None),"substituicao_jw":r"s = j\omega \implies D(j\omega) + K N(j\omega) = 0","parte_real_eq":characteristic(tf,Some(false)),"parte_imaginaria_eq":characteristic(tf,Some(true)),"tem_cruzamento":!unique.is_empty(),"cruzamentos":unique},
 "passo_7":{"tem_partida":!departure.is_empty(),"tem_chegada":!arrival.is_empty(),"deducoes_partida":departure,"deducoes_chegada":arrival}})
}
fn xy(chart: &Chart, x: f64, y: f64) -> (f64, f64) {
    let (a, b, c, d) = chart.bounds;
    let (l, t, w, h) = chart.rect;
    (l + (x - a) / (b - a) * w, t + h - (y - c) / (d - c) * h)
}
fn marker(chart: &mut Chart, x: f64, y: f64, kind: &str, color: &str, r: f64) {
    let (px, py) = xy(chart, x, y);
    chart.svg += &match kind {
        "x" => format!(
            r#"<path d="M {} {} l {} {} M {} {} l {} {}" stroke="{color}" stroke-width="2.2"/>"#,
            px - r,
            py - r,
            2. * r,
            2. * r,
            px - r,
            py + r,
            2. * r,
            -2. * r
        ),
        "+" => format!(
            r#"<path d="M {} {py} h {} M {px} {} v {}" stroke="{color}" stroke-width="1.8"/>"#,
            px - r,
            2. * r,
            py - r,
            2. * r
        ),
        "square" => format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{color}"/>"#,
            px - r,
            py - r,
            2. * r,
            2. * r
        ),
        _ => format!(
            r#"<circle cx="{px}" cy="{py}" r="{r}" fill="white" stroke="{color}" stroke-width="2"/>"#
        ),
    };
}
#[allow(clippy::too_many_arguments)]
fn badge(
    chart: &mut Chart,
    x: f64,
    y: f64,
    text: &str,
    dx: f64,
    dy: f64,
    color: &str,
    anchor: &str,
    arrow: bool,
) {
    let (px, py) = xy(chart, x, y);
    let bx = px + dx;
    let by = py + dy;
    let lines: Vec<&str> = text.lines().collect();
    let width = lines.iter().map(|s| s.chars().count()).max().unwrap_or(0) as f64 * 8. + 12.;
    let height = lines.len() as f64 * 18. + 7.;
    let left = match anchor {
        "middle" => bx - width / 2.,
        "end" => bx - width + 4.,
        _ => bx - 4.,
    };
    let top = by - 15.;
    if arrow {
        let xx = left + width / 2.;
        let yy = if dy < 0. { top + height } else { top };
        arrow_pixels(chart, (xx, yy), (px, py), color)
    }
    chart.svg += &format!(
        r#"<rect x="{left}" y="{top}" width="{width}" height="{height}" rx="4" fill="white" fill-opacity="0.96" stroke="{color}" stroke-width="0.7"/><text x="{bx}" y="{by}" text-anchor="{anchor}" font-size="14" fill="{color}">"#
    );
    for (i, line) in lines.iter().enumerate() {
        chart.svg += &format!(
            r#"<tspan x="{bx}" dy="{}">{}</tspan>"#,
            if i == 0 { 0 } else { 18 },
            crate::presentation::escape(line)
        )
    }
    chart.svg.push_str("</text>");
}
fn arrow_pixels(chart: &mut Chart, from: (f64, f64), to: (f64, f64), color: &str) {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let norm = dx.hypot(dy);
    if norm <= 0. {
        return;
    }
    let u = dx / norm;
    let v = dy / norm;
    chart.svg += &format!(
        r#"<path d="M {} {} L {} {} M {} {} L {} {} L {} {}" fill="none" stroke="{color}" stroke-width="1.5"/>"#,
        from.0,
        from.1,
        to.0,
        to.1,
        to.0 - u * 8. - v * 4.,
        to.1 - v * 8. + u * 4.,
        to.0,
        to.1,
        to.0 - u * 8. + v * 4.,
        to.1 - v * 8. - u * 4.
    );
}
fn angle_plot(chart: &mut Chart, point: C, angle: f64, radius: f64, arrival: bool) {
    let color = if arrival { "#15803d" } else { "#6d28d9" };
    let symbol = if arrival { "a" } else { "d" };
    for sign in [1., -1.] {
        let origin = C::new(point.re, sign * point.im);
        let degrees = sign * angle;
        let end = origin + C::from_polar(radius, degrees.to_radians());
        let from = xy(chart, origin.re, origin.im);
        let to = xy(chart, end.re, end.im);
        if arrival {
            arrow_pixels(chart, to, from, color)
        } else {
            arrow_pixels(chart, from, to, color)
        }
        chart.line(
            &[origin.re, origin.re + radius],
            &[origin.im, origin.im],
            "#64748b",
            0.8,
            "2 4",
        );
        let x: Vec<f64> = (0..=64)
            .map(|i| {
                origin.re
                    + radius / 2.
                        * (degrees.min(0.) + (degrees.abs() * i as f64 / 64.))
                            .to_radians()
                            .cos()
            })
            .collect();
        let y: Vec<f64> = (0..=64)
            .map(|i| {
                origin.im
                    + radius / 2.
                        * (degrees.min(0.) + (degrees.abs() * i as f64 / 64.))
                            .to_radians()
                            .sin()
            })
            .collect();
        chart.line(&x, &y, color, 1.3, "");
        badge(
            chart,
            end.re,
            end.im,
            &format!("θ_{symbol}={degrees:0.1}°"),
            14.,
            if sign > 0. { -30. } else { 34. },
            color,
            "start",
            false,
        );
    }
}
#[allow(clippy::too_many_arguments)]
fn graph(
    poles: &[C],
    zeros: &[C],
    rows: &[Vec<C>],
    centroid: Option<f64>,
    asymptotes: &[Value],
    breaks: &[Value],
    crossings: &[Value],
    departures: &[Value],
    arrivals: &[Value],
) -> Result<(String, String)> {
    let mut all_x: Vec<f64> = poles.iter().chain(zeros).map(|v| v.re).collect();
    all_x.push(0.);
    if let Some(v) = centroid {
        all_x.push(v)
    }
    all_x.extend(breaks.iter().map(|v| v["s"].as_f64().unwrap()));
    let x_min = all_x.iter().copied().fold(f64::INFINITY, f64::min);
    let x_max = all_x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let y_max = poles
        .iter()
        .chain(zeros)
        .map(|v| v.im.abs())
        .chain(crossings.iter().map(|v| v["w"].as_f64().unwrap().abs()))
        .fold(2., f64::max);
    let span_x = (x_max - x_min).max(4.);
    let span_y = (2. * y_max).max(4.);
    let limits = (
        (x_min - 0.18 * span_x).min(-1.),
        (x_max + 0.18 * span_x).max(1.5),
        -y_max - 0.15 * span_y,
        y_max + 0.15 * span_y,
    );
    let width = 1312.;
    let height = 1152.;
    let available = (110., 35., 1140., 900.);
    let scale = (available.2 / (limits.1 - limits.0)).min(available.3 / (limits.3 - limits.2));
    let pw = (limits.1 - limits.0) * scale;
    let ph = (limits.3 - limits.2) * scale;
    let rect = (
        available.0 + (available.2 - pw) / 2.,
        available.1 + (available.3 - ph) / 2.,
        pw,
        ph,
    );
    let mut chart = Chart::new(
        width,
        height,
        rect,
        "Eixo Real (σ)",
        "Eixo Imaginário (jω)",
        false,
    );
    chart.bounds = limits;
    chart.axes();
    chart.line(&[limits.0, limits.1], &[0., 0.], "black", 0.8, "");
    chart.line(&[0., 0.], &[limits.2, limits.3], "black", 0.8, "");
    if let Some(c) = centroid {
        for item in asymptotes {
            let rad = item["rad"].as_f64().unwrap();
            chart.line(
                &[c, c + 100. * rad.cos()],
                &[0., 100. * rad.sin()],
                "#64748B",
                1.2,
                "6 4",
            )
        }
    }
    if let Some(first) = rows.first() {
        for branch in 0..first.len() {
            let x: Vec<f64> = rows.iter().map(|r| r[branch].re).collect();
            let y: Vec<f64> = rows.iter().map(|r| r[branch].im).collect();
            chart.line(&x, &y, "black", 2., "");
        }
    }
    let real_poles: Vec<f64> = poles
        .iter()
        .filter(|v| v.im.abs() < 1e-5)
        .map(|v| v.re)
        .collect();
    let real_zeros: Vec<f64> = zeros
        .iter()
        .filter(|v| v.im.abs() < 1e-5)
        .map(|v| v.re)
        .collect();
    let mut sorted_poles = real_poles.clone();
    sorted_poles.sort_by(f64::total_cmp);
    let mut sorted_zeros = real_zeros.clone();
    sorted_zeros.sort_by(f64::total_cmp);
    let collision = span_x * 0.09;
    for p in poles {
        marker(&mut chart, p.re, p.im, "x", "black", 6.);
        let real = values::general(p.re, 2);
        if p.im.abs() < 1e-5 {
            let i = sorted_poles.iter().position(|&v| v == p.re).unwrap();
            let close_left = i > 0 && p.re - sorted_poles[i - 1] < collision;
            badge(
                &mut chart,
                p.re,
                0.,
                &format!("p={real}"),
                0.,
                if close_left && i % 2 == 1 { -45. } else { -25. },
                "#000000",
                "middle",
                false,
            )
        } else {
            badge(
                &mut chart,
                p.re,
                p.im,
                &format!(
                    "p={real}{}j{}",
                    if p.im >= 0. { "+" } else { "-" },
                    values::general(p.im.abs(), 2)
                ),
                -16.,
                if p.im > 0. { -25. } else { 35. },
                "#000000",
                if p.re < 0. { "end" } else { "start" },
                false,
            )
        }
    }
    for z in zeros {
        marker(&mut chart, z.re, z.im, "o", "black", 5.5);
        let real = values::general(z.re, 2);
        if z.im.abs() < 1e-5 {
            let i = sorted_zeros.iter().position(|&v| v == z.re).unwrap();
            let close = (i > 0 && z.re - sorted_zeros[i - 1] < collision)
                || (i + 1 < sorted_zeros.len() && sorted_zeros[i + 1] - z.re < collision);
            badge(
                &mut chart,
                z.re,
                0.,
                &format!("z={real}"),
                0.,
                if close && i % 2 == 1 { 55. } else { 36. },
                "#000000",
                "middle",
                false,
            )
        } else {
            badge(
                &mut chart,
                z.re,
                z.im,
                &format!(
                    "z={real}{}j{}",
                    if z.im >= 0. { "+" } else { "-" },
                    values::general(z.im.abs(), 2)
                ),
                16.,
                if z.im > 0. { -25. } else { 35. },
                "#000000",
                "start",
                false,
            )
        }
    }
    if let Some(c) = centroid {
        marker(&mut chart, c, 0., "+", "#64748B", 7.);
        let close = real_zeros.iter().any(|v| (v - c).abs() < collision * 1.3);
        badge(
            &mut chart,
            c,
            0.,
            &format!("σ_a={c:0.2}"),
            0.,
            if close { 81. } else { 50. },
            "#111827",
            "middle",
            close,
        );
        let radius = span_x.min(span_y) * 0.44;
        for item in asymptotes {
            let rad = item["rad"].as_f64().unwrap();
            badge(
                &mut chart,
                c + radius * rad.cos(),
                radius * rad.sin(),
                &format!("{:0.0}°", item["graus"].as_f64().unwrap()),
                0.,
                0.,
                "#111827",
                "middle",
                false,
            )
        }
    }
    for b in breaks {
        let s = b["s"].as_f64().unwrap();
        let k = b["K"].as_f64().unwrap();
        marker(&mut chart, s, 0., "square", "black", 5.);
        let near = real_poles.iter().any(|v| (v - s).abs() < collision * 1.4);
        badge(
            &mut chart,
            s,
            0.,
            &format!("Break: {s:0.2}\n(K={k:0.1})"),
            0.,
            if near { -105. } else { -80. },
            "#000000",
            "middle",
            true,
        )
    }
    for c in crossings {
        let w = c["w"].as_f64().unwrap();
        let k = c["K"].as_f64().unwrap();
        marker(&mut chart, 0., w, "o", "#B91C1C", 5.);
        badge(
            &mut chart,
            0.,
            w,
            &format!("jω={w:+0.2}\nK={k:0.1}"),
            35.,
            0.,
            "#B91C1C",
            "start",
            true,
        )
    }
    let arc_r = (span_y * 0.08).min(1.6);
    for (index, &p) in poles.iter().filter(|v| v.im > 1e-4).enumerate() {
        angle_plot(
            &mut chart,
            p,
            departures[index]["angulo"].as_f64().unwrap(),
            arc_r,
            false,
        );
    }
    for (index, &z) in zeros.iter().filter(|v| v.im > 1e-4).enumerate() {
        angle_plot(
            &mut chart,
            z,
            arrivals[index]["angulo"].as_f64().unwrap(),
            arc_r,
            true,
        );
    }
    let mut legends = vec![
        ("Ramos do LGR".to_string(), "black"),
        ("Polos (Início)".to_string(), "black"),
        (
            if zeros.is_empty() {
                "Zeros (Nenhum nesta FT)".to_string()
            } else {
                "Zeros (Término)".to_string()
            },
            "black",
        ),
    ];
    if let Some(c) = centroid {
        legends.push((format!("Centroide (σ_a = {c:0.2})"), "#64748B"));
        legends.push(("Assíntotas".into(), "#64748B"))
    }
    if let Some(b) = breaks.first() {
        legends.push((
            format!("Break-in/out (K={:0.1})", b["K"].as_f64().unwrap()),
            "black",
        ))
    }
    if let Some(c) = crossings.first() {
        legends.push((
            format!(
                "Cruz. jω (ω=±{:0.2}, K={:0.1})",
                c["w"].as_f64().unwrap().abs(),
                c["K"].as_f64().unwrap()
            ),
            "#B91C1C",
        ))
    }
    chart.svg += r#"<text x="656" y="1010" text-anchor="middle" font-weight="bold">Componentes do LGR</text>"#;
    let column_sizes = [
        legends.len().div_ceil(3),
        (legends.len() + 1) / 3,
        legends.len() / 3,
    ];
    for (i, (label, color)) in legends.iter().enumerate() {
        let (column, row) = if i < column_sizes[0] {
            (0, i)
        } else if i < column_sizes[0] + column_sizes[1] {
            (1, i - column_sizes[0])
        } else {
            (2, i - column_sizes[0] - column_sizes[1])
        };
        let x = 145. + column as f64 * 385.;
        let y = 1040. + row as f64 * 27.;
        let cx = x + 12.;
        let cy = y - 4.;
        if label.starts_with("Polos") {
            chart.svg += &format!(
                r#"<path d="M {} {} l 10 10 M {} {} l 10 -10" stroke="{color}" stroke-width="2.2"/>"#,
                cx - 5.,
                cy - 5.,
                cx - 5.,
                cy + 5.
            );
        } else if label.starts_with("Zeros") || label.starts_with("Cruz.") {
            chart.svg += &format!(
                r#"<circle cx="{cx}" cy="{cy}" r="5" fill="white" stroke="{color}" stroke-width="2"/>"#
            );
        } else if label.starts_with("Centroide") {
            chart.svg += &format!(
                r#"<path d="M {} {cy} h 12 M {cx} {} v 12" stroke="{color}" stroke-width="1.8"/>"#,
                cx - 6.,
                cy - 6.
            );
        } else if label.starts_with("Break") {
            chart.svg += &format!(
                r#"<rect x="{}" y="{}" width="9" height="9" fill="{color}"/>"#,
                cx - 4.5,
                cy - 4.5
            );
        } else {
            chart.svg += &format!(
                r#"<path d="M {x} {cy} h 25" stroke="{color}" stroke-width="2" stroke-dasharray="{}"/>"#,
                if label == "Assíntotas" { "6 4" } else { "" }
            );
        }
        chart.svg += &format!(
            r#"<text x="{}" y="{y}" font-size="13">{}</text>"#,
            x + 33.,
            crate::presentation::escape(label)
        );
    }
    chart.finish()
}
pub fn analyze(payload: &Value) -> Result<Value> {
    let tf = input::transfer(payload)?;
    let poles = tf.poles()?;
    let zeros = tf.zeros()?;
    let (gains, rows) = locus(&tf)?;
    let p = poles.len();
    let z = zeros.len();
    let centroid = if p > z {
        Some(
            (poles.iter().map(|v| v.re).fold(0.0, |a, b| a + b)
                - zeros.iter().map(|v| v.re).fold(0.0, |a, b| a + b))
                / (p - z) as f64,
        )
    } else {
        None
    };
    let asymptotes: Vec<Value> = (0..p.saturating_sub(z))
        .map(|k| {
            let rad = std::f64::consts::PI * (2 * k + 1) as f64 / (p - z) as f64;
            json!({"k":k,"graus":rad.to_degrees().rem_euclid(360.),"rad":rad})
        })
        .collect();
    let break_polynomial = poly::add(
        &poly::multiply(&poly::derivative(&tf.num), &tf.den),
        &poly::multiply(&tf.num, &poly::derivative(&tf.den)),
        -1.,
    );
    let break_roots = poly::roots(&break_polynomial)?;
    let breaks: Vec<Value> = break_roots
        .iter()
        .filter(|v| v.im.abs() < 1e-5)
        .filter_map(|r| {
            let numerator = poly::evaluate(&tf.num, C::new(r.re, 0.)).re;
            if numerator.abs() <= 1e-8 {
                return None;
            }
            let k = -poly::evaluate(&tf.den, C::new(r.re, 0.)).re / numerator;
            (k > 0.).then(|| json!({"s":r.re,"K":k}))
        })
        .collect();
    let mut crossings = vec![];
    for branch in 0..p.max(z) {
        for j in 0..rows.len() - 1 {
            let a = rows[j][branch];
            let b = rows[j + 1][branch];
            if a.re * b.re < 0. {
                let t = a.re.abs() / (a.re.abs() + b.re.abs());
                let w = a.im + t * (b.im - a.im);
                let k = gains[j] + t * (gains[j + 1] - gains[j]);
                if w.abs() > 1e-2 {
                    crossings.push(json!({"w":w,"K":k}))
                }
            }
        }
    }
    let (departures, departure_details) = angles(&poles, &zeros, false);
    let (arrivals, arrival_details) = angles(&zeros, &poles, true);
    let mut steps = seven_steps(
        &tf,
        &poles,
        &zeros,
        centroid,
        &break_roots,
        &crossings,
        &departure_details,
        &arrival_details,
    );
    let force_float = matches!(payload["mode"].as_str(), Some("coeffs" | "zpk" | "preset"));
    let (latex_exp, latex_fac) = tf.latex_formatted(force_float);
    let (_, factored) = tf.latex_formatted(true);
    steps["passo_1_2_3"]["eq_fatorada"] = json!(factored);
    let (image, svg) = graph(
        &poles,
        &zeros,
        &rows,
        centroid,
        &asymptotes,
        &breaks,
        &crossings,
        &departures,
        &arrivals,
    )?;
    let serialize = |values: &[C]| {
        values
            .iter()
            .map(|&v| json!({"re":v.re,"im":v.im,"str":format_complex(v)}))
            .collect::<Vec<_>>()
    };
    Ok(
        json!({"success":true,"image":image,"svg":svg,"latex_exp":latex_exp,"latex_fac":latex_fac,"detalhes":{"P":p,"Z":z,"ramos":p.max(z),"polos":serialize(&poles),"zeros":serialize(&zeros),"centroide":centroid,"angulos_assintotas":asymptotes,"break_points":breaks,"jw_cruzamentos":crossings,"angulos_partida":departures,"angulos_chegada":arrivals,"passo_a_passo":steps}}),
    )
}
