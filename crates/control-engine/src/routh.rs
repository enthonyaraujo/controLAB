//! Routh-Hurwitz table and gain sign chart, using exact symbolic arithmetic.
use crate::{
    Result, polynomial,
    symbolic::{
        Expr, Polynomial, Rational, RealRoot, SPolynomial, parse_characteristic, rational_latex,
        real_roots,
    },
};
use num_complex::Complex64;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde_json::{Value, json};

fn power(n: usize) -> String {
    if n > 1 {
        format!("s^{{{n}}}")
    } else if n == 1 {
        "s".into()
    } else {
        "s^{0}".into()
    }
}
fn latex_values(row: &[Expr]) -> Vec<String> {
    row.iter().map(Expr::latex).collect()
}
fn input(data: &Value) -> &str {
    ["expr", "input", "den"]
        .into_iter()
        .filter_map(|k| data[k].as_str())
        .find(|s| !s.is_empty())
        .unwrap_or("s^3 + 2s^2 + s + 1")
}

struct Table {
    rows: Vec<Vec<Expr>>,
    special: Vec<Value>,
    steps: Vec<Value>,
    cells: Vec<Value>,
}
fn table(coeffs: &[Expr]) -> Result<Table> {
    let n = coeffs.len() - 1;
    let m = (n + 2) / 2;
    let mut rows = vec![vec![Expr::integer(0); m]; 2];
    for (i, c) in coeffs.iter().enumerate() {
        rows[i % 2][i / 2] = c.clone();
    }
    let mut steps = vec![
        json!({"power":format!("s^{{{n}}}"),"desc":format!("Primeira linha formada pelos coeficientes de potências de mesma paridade de $s^{{{n}}}$."),"values":latex_values(&rows[0])}),
        json!({"power":format!("s^{{{}}}",n-1),"desc":format!("Segunda linha formada pelos coeficientes de potências alternadas a partir de $s^{{{}}}$.",n-1),"values":latex_values(&rows[1])}),
    ];
    let mut special = vec![];
    let mut cells = vec![];
    for i in 2..=n {
        let p = n - i;
        if rows[i - 1].iter().all(|c| c.is_zero() && !c.approximate) {
            let aux_power = n - i + 2;
            let mut a = vec![Expr::integer(0); aux_power + 1];
            for (j, c) in rows[i - 2].iter().enumerate() {
                if 2 * j <= aux_power {
                    a[aux_power - 2 * j] = c.clone();
                }
            }
            let aux = SPolynomial::new(a);
            let deriv = SPolynomial::new(
                aux.0
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(p, c)| c.mul(&Expr::integer(p as i64)))
                    .collect(),
            );
            let mut replacement = deriv.0.iter().rev().step_by(2).cloned().collect::<Vec<_>>();
            replacement.resize(m, Expr::integer(0));
            rows[i - 1] = replacement;
            let note = format!(
                "Linha de zeros detectada em $s^{{{}}}$. Polinômio auxiliar: $A(s) = {}$. Derivada substituída: $\\frac{{dA}}{{ds}} = {}$.",
                p + 1,
                aux.latex(),
                deriv.latex()
            );
            special.push(json!({"type":"row_of_zeros","power":format!("s^{{{}}}",p+1),"aux_poly_latex":aux.latex(),"aux_deriv_latex":deriv.latex(),"desc":note}));
            steps.push(json!({"power":format!("s^{{{}}} (Ajustada)",p+1),"desc":note,"values":latex_values(&rows[i-1])}));
        } else if rows[i - 1][0].is_zero() && !rows[i - 1][0].approximate {
            rows[i - 1][0] = Expr::epsilon();
            let note = format!(
                "Primeiro elemento nulo em $s^{{{}}}$. Substituído por termo infinitesimal positivo $\\varepsilon > 0$.",
                p + 1
            );
            special.push(
                json!({"type":"zero_first_element","power":format!("s^{{{}}}",p+1),"desc":note}),
            );
        }
        let mut row = vec![Expr::integer(0); m];
        for j in 0..m - 1 {
            let a1 = &rows[i - 2][0];
            let a2 = &rows[i - 2][j + 1];
            let b1 = &rows[i - 1][0];
            let b2 = &rows[i - 1][j + 1];
            row[j] = b1.mul(a2).sub(&a1.mul(b2)).div(b1)?;
            cells.push(json!({"row":i,"col":j,"power":power(p),"formula":format!("\\frac{{({})({}) - ({})({})}}{{{}}}",b1.latex(),a2.latex(),a1.latex(),b2.latex(),b1.latex()),"value_latex":row[j].latex()}));
        }
        steps.push(json!({"power":power(p),"desc":"Linha calculada a partir dos determinantes das duas linhas anteriores.","values":latex_values(&row)}));
        rows.push(row);
    }
    Ok(Table {
        rows,
        special,
        steps,
        cells,
    })
}

fn signs(first: &[Expr], special: &[Value]) -> (Vec<Value>, usize, &'static str, String) {
    let signs = first
        .iter()
        .map(|e| {
            let (order, c) = e.epsilon_leading();
            let symbolic = order <= 0 && c.has_k();
            let sign = if symbolic {
                "expr"
            } else {
                match e.limit_sign(0.) {
                    -1 => "-",
                    1 => "+",
                    _ => "0",
                }
            };
            json!({"term_latex":e.latex(),"sign":sign,"is_symbolic":symbolic})
        })
        .collect::<Vec<_>>();
    if signs.iter().any(|s| s["is_symbolic"] == true) {
        return(signs,0,"Dependente de K","A estabilidade depende do ganho escalar K. Consulte a seção de Faixa de K para as desigualdades analíticas.".into());
    }
    let filtered: Vec<_> = signs
        .iter()
        .filter_map(|s| match s["sign"].as_str() {
            Some("+") => Some(1),
            Some("-") => Some(-1),
            _ => None,
        })
        .collect();
    let changes = filtered.windows(2).filter(|w| w[0] != w[1]).count();
    let zero_rows = special.iter().any(|s| s["type"] == "row_of_zeros");
    let zero_col = signs.iter().any(|s| s["sign"] == "0");
    let (verdict, summary) = if changes == 0 && !zero_rows && !zero_col {
        ("Estável","Todos os elementos da primeira coluna possuem o mesmo sinal (sem trocas de sinal). O sistema é BIBO estável e todas as raízes situam-se estritamente no semiplano esquerdo (LHP).".into())
    } else if changes == 0 && zero_rows {
        ("Marginalmente Estável","Não ocorrem trocas de sinal fora da linha de zeros, indicando a existência de pares de polos puramente imaginários sobre o eixo jω (sem amortecimento). O sistema oscila com amplitude sustentada.".into())
    } else {
        (
            "Instável",
            format!(
                "Ocorrem {changes} troca(s) de sinal na primeira coluna da tabela de Routh, o que indica exatamente {changes} polo(s) em malha fechada no semiplano direito (RHP)."
            ),
        )
    };
    (signs, changes, verdict, summary)
}

/// Python's .4g representation (used only for presentation, never algebra).
pub fn general(v: f64) -> String {
    if v == 0. {
        return if v.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    if !v.is_finite() {
        return format!("{v}");
    }
    let exponent = v.abs().log10().floor() as i32;
    let decimals = (3 - exponent).max(0) as usize;
    if !(-4..4).contains(&exponent) {
        let s = format!("{v:.3e}");
        let (m, e) = s.split_once('e').unwrap();
        let m = m.trim_end_matches('0').trim_end_matches('.');
        let e = e.parse::<i32>().unwrap();
        format!("{m}e{}{e:02}", if e >= 0 { "+" } else { "" })
    } else {
        let s = format!("{v:.decimals$}");
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').into()
        } else {
            s
        }
    }
}
fn round4(v: f64) -> f64 {
    (v * 10000.).round() / 10000.
}
fn root_string(z: Complex64) -> String {
    let real = general(z.re);
    if z.im.abs() > 1e-6 {
        format!(
            "{real} {} {}j",
            if z.im >= 0. { "+" } else { "-" },
            general(z.im.abs())
        )
    } else {
        real
    }
}
fn root_json(z: Complex64, zone: bool) -> Value {
    let r = round4(z.re);
    let im = round4(z.im);
    let mut value =
        json!({"real":r,"imag":im,"str":root_string(if zone{Complex64::new(r,im)}else{z})});
    if zone {
        value["zone"] = json!(if r > 1e-4 {
            "SPD"
        } else if r < -1e-4 {
            "SPE"
        } else {
            "jw"
        });
    }
    value
}
fn boundary_latex(p: &Polynomial, r: &RealRoot, approximate: bool) -> String {
    if approximate {
        return crate::symbolic::float_latex(
            &r.exact.clone().unwrap_or_else(|| {
                crate::symbolic::parse_rational(&r.value().to_string()).unwrap()
            }),
        );
    }
    if let Some(v) = &r.exact {
        return rational_latex(v);
    }
    let c = p.k_coefficients();
    if c.len() == 3 {
        let two = Rational::from_integer(2.into());
        let center = -&c[1] / (&two * &c[0]);
        let delta = &center * &center - &c[2] / &c[0];
        let mut coefficient = Rational::one();
        let mut radicand = delta.clone();
        if let (Some(n), Some(d)) = (delta.numer().to_u64(), delta.denom().to_u64())
            && let Some(mut value) = n.checked_mul(d)
        {
            let mut square = 1u64;
            let mut rem = 1u64;
            let mut prime = 2u64;
            while prime * prime <= value && prime <= 1_000_000 {
                let mut power = 0;
                while value % prime == 0 {
                    value /= prime;
                    power += 1;
                }
                for _ in 0..power / 2 {
                    square *= prime;
                }
                if power % 2 == 1 {
                    rem *= prime;
                }
                prime += 1;
            }
            rem *= value;
            coefficient = Rational::new(square.into(), d.into());
            radicand = Rational::from_integer(rem.into());
        }
        let radical = if radicand.is_one() {
            rational_latex(&coefficient)
        } else {
            let radical = format!("\\sqrt{{{}}}", rational_latex(&radicand));
            if coefficient.is_one() {
                radical
            } else if coefficient.is_integer() {
                format!("{} {radical}", coefficient.to_integer())
            } else {
                let top = if coefficient.numer().is_one() {
                    radical
                } else {
                    format!("{} {radical}", coefficient.numer())
                };
                format!("\\frac{{{top}}}{{{}}}", coefficient.denom())
            }
        };
        let center_value = center.to_f64().unwrap();
        if r.value() > center_value {
            if center.is_zero() {
                radical
            } else if center.is_positive() {
                format!("{radical} + {}", rational_latex(&center))
            } else {
                format!("{radical} - {}", rational_latex(&center.abs()))
            }
        } else if center.is_zero() {
            format!("- {radical}")
        } else {
            format!("{} - {radical}", rational_latex(&center))
        }
    } else {
        let index = real_roots(p)
            .ok()
            .and_then(|roots| roots.iter().position(|root| same_boundary(root, r)))
            .unwrap_or(0);
        format!(
            "\\operatorname{{CRootOf}}\\left({}, {index}\\right)",
            crate::symbolic::polynomial_latex(p)
        )
    }
}
fn same_boundary(a: &RealRoot, b: &RealRoot) -> bool {
    a.low <= b.high && b.low <= a.high
}
fn numerical_roots(poly: &SPolynomial, k: f64) -> Result<Vec<Complex64>> {
    polynomial::roots(&poly.numeric(k))
}
fn require_affine_gain(poly: &SPolynomial) -> Result<()> {
    if poly
        .0
        .iter()
        .any(|c| c.den.has_k() || c.num.0.keys().any(|(k, _)| *k > 1))
    {
        Err("Cannot convert expression to float".into())
    } else {
        Ok(())
    }
}
fn exact_limit_sign(e: &Expr, k: &Rational) -> i8 {
    let Ok(e) = e.substitute_k(k) else { return 0 };
    let (order, c) = e.epsilon_leading();
    if order > 0 || c.is_zero() {
        return 0;
    }
    if order == 0 && c.evaluate(0., 0.).abs() <= 1e-12 {
        return 0;
    }
    let n = c.num.0.get(&(0, 0)).cloned().unwrap_or_else(Rational::zero);
    let d = c.den.0.get(&(0, 0)).cloned().unwrap_or_else(Rational::one);
    if n.is_zero() {
        0
    } else if n.is_positive() == d.is_positive() {
        1
    } else {
        -1
    }
}
fn k_range(first: &[Expr], poly: &SPolynomial) -> Result<Value> {
    if !first.iter().any(Expr::has_k) {
        return Ok(
            json!({"has_k":false,"range_latex":"Não aplicável (sem ganho K variável)","conditions":[],"critical_k":[]}),
        );
    }
    let lead_positive = first[0].has_k() || first[0].evaluate(0., 0.) > 0.;
    let mut conditions = vec![];
    let mut critical = vec![];
    let mut boundaries = vec![RealRoot {
        low: Rational::zero(),
        high: Rational::zero(),
        exact: Some(Rational::zero()),
    }];
    for term in first {
        if term.has_k() {
            conditions.push(json!({"term_latex":term.latex(),"inequality_latex":format!("{} {} 0",term.latex(),if lead_positive{">"}else{"<"})}));
        }
        let (_, limit) = term.epsilon_leading();
        for numerator in [true, false] {
            let p = if numerator { &limit.num } else { &limit.den };
            if !p.has_k() {
                continue;
            }
            for r in real_roots(p)? {
                if numerator && term.has_k() && r.value() > 0. {
                    let mut ck =
                        json!({"k_val":r.value(),"k_latex":boundary_latex(p,&r,term.approximate)});
                    let roots = numerical_roots(poly, r.value())?;
                    let omega = roots
                        .iter()
                        .filter(|z| z.re.abs() < 1e-5 && z.im.abs() > 1e-5)
                        .map(|z| z.im.abs())
                        .min_by(f64::total_cmp);
                    if let Some(w) = omega {
                        ck["omega_osc"] = json!(round4(w));
                        ck["omega_latex"] =
                            json!(format!("\\omega_{{osc}} = {} \\text{{ rad/s}}", general(w)));
                    }
                    critical.push(ck);
                }
                if numerator && r.value() > 0. && !boundaries.iter().any(|b| same_boundary(b, &r)) {
                    boundaries.push(r);
                }
            }
        }
    }
    boundaries.sort_by(|a, b| a.low.cmp(&b.low));
    let mut stable = vec![];
    let mut unstable = vec![];
    for i in 0..=boundaries.len() {
        let low = i.checked_sub(1).map(|j| &boundaries[j]);
        let high = boundaries.get(i);
        let sample = match (low, high) {
            (None, Some(h)) => &h.low - Rational::from_integer(5.into()),
            (Some(l), None) => &l.high + Rational::from_integer(5.into()),
            (Some(l), Some(h)) => (&l.high + &h.low) / Rational::from_integer(2.into()),
            _ => Rational::one(),
        };
        let lead = if lead_positive { 1 } else { -1 };
        let ok = first
            .iter()
            .all(|term| exact_limit_sign(term, &sample) == lead);
        let text = match (low, high) {
            (None, Some(h)) => format!("K < {}", general(h.value())),
            (Some(l), None) => format!("K > {}", general(l.value())),
            (Some(l), Some(h)) => format!("{} < K < {}", general(l.value()), general(h.value())),
            _ => "K \\in \\mathbb{R}".into(),
        };
        if ok {
            stable.push(text);
        } else {
            unstable.push(text);
        }
    }
    let join = " \\quad \\text{ou} \\quad ";
    let stable_latex = if stable.is_empty() {
        "\\text{Sem faixa estável}".into()
    } else {
        stable.join(join)
    };
    let unstable_latex = if unstable.is_empty() {
        "\\text{Sem faixa instável}".into()
    } else {
        unstable.join(join)
    };
    let marginal=critical.iter().filter_map(|ck|ck.get("omega_osc").map(|w|json!({"k_val":ck["k_val"],"k_latex":format!("K = {}",general(ck["k_val"].as_f64().unwrap())),"omega_osc":w,"omega_latex":ck["omega_latex"],"poles_latex":format!("s = \\pm {} j",general(w.as_f64().unwrap()))}))).collect::<Vec<_>>();
    let marginal_latex = if marginal.is_empty() {
        "\\text{Nenhum ponto marginal em } K > 0".into()
    } else {
        marginal
            .iter()
            .map(|m| {
                format!(
                    "{} \\; ({})",
                    m["k_latex"].as_str().unwrap(),
                    m["omega_latex"].as_str().unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(join)
    };
    Ok(
        json!({"has_k":true,"range_latex":stable_latex,"stable_latex":stable_latex,"unstable_latex":unstable_latex,"marginal_latex":marginal_latex,"stable_intervals":stable,"unstable_intervals":unstable,"marginal_cases":marginal,"conditions":conditions,"critical_k":critical}),
    )
}

pub fn analyze(data: &Value) -> Result<Value> {
    let source = input(data);
    let (poly, is_fraction) =
        parse_characteristic(source, data["has_k"].as_bool().unwrap_or(true))?;
    require_affine_gain(&poly)?;
    let coeffs = poly.0.iter().rev().cloned().collect::<Vec<_>>();
    let t = table(&coeffs)?;
    let first = t.rows.iter().map(|r| r[0].clone()).collect::<Vec<_>>();
    let (evaluated, changes, verdict, summary) = signs(&first, &t.special);
    let k = k_range(&first, &poly)?;
    let formatted=t.rows.iter().enumerate().map(|(i,row)|json!({"power":power(poly.degree()-i),"row_index":i,"cells":row.iter().enumerate().map(|(j,c)|json!({"col":j,"latex":c.latex(),"is_first_col":j==0,"sign":if j==0{evaluated[i]["sign"].as_str().unwrap()}else{""}})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let roots = if poly.has_k() {
        vec![]
    } else {
        numerical_roots(&poly, 0.)?
    };
    let (theme, title) = (data["theme"].as_str().unwrap_or("dark"), "Plano complexo s");
    let mut curves: Vec<(String, Vec<f64>, Vec<f64>)> = vec![];
    if poly.has_k() {
        let mut gains = vec![0.];
        let max = k["critical_k"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| c["k_val"].as_f64())
            .fold(0., f64::max);
        let upper = if max > 0. {
            (max * 5.).max(100.)
        } else {
            1000.
        };
        for i in 0..2500 {
            gains.push(10f64.powf(-3. + (upper.log10() + 3.) * i as f64 / 2499.));
        }
        for c in k["critical_k"].as_array().into_iter().flatten() {
            if let Some(v) = c["k_val"].as_f64() {
                gains.push(v);
            }
        }
        gains.sort_by(f64::total_cmp);
        let mut prior = Vec::<Complex64>::new();
        let mut branches = vec![(vec![], vec![]); poly.degree()];
        for gain in gains {
            let mut current = numerical_roots(&poly, gain)?;
            if current.len() != branches.len() {
                continue;
            }
            if !prior.is_empty() {
                let mut ordered = vec![];
                for z in &prior {
                    let closest = current
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            (**a - *z).norm_sqr().total_cmp(&(**b - *z).norm_sqr())
                        })
                        .unwrap()
                        .0;
                    ordered.push(current.remove(closest));
                }
                current = ordered;
            }
            for (i, z) in current.iter().enumerate() {
                branches[i].0.push(z.re);
                branches[i].1.push(z.im);
            }
            prior = current;
        }
        for (i, (x, y)) in branches.into_iter().enumerate() {
            curves.push((format!("Ramo {}", i + 1), x, y));
        }
    } else {
        for (i, z) in roots.iter().enumerate() {
            curves.push((
                format!("p{}={}", i + 1, root_string(*z)),
                vec![z.re],
                vec![z.im],
            ));
        }
    }
    let _ = (theme, title); // Both canonical scientific themes use the same palette.
    let (image, svg) = stability_plot(&poly, &k, &roots, &curves)?;
    Ok(
        json!({"success":true,"input_str":source,"is_fraction":is_fraction,"char_poly_latex":format!("{} = 0",poly.latex()),"degree":poly.degree(),"coeffs_latex":latex_values(&coeffs),"routh_table":formatted,"special_cases":t.special,"steps":t.steps,"cell_details":t.cells,"first_col_signs":evaluated,"sign_changes":changes,"rhp_poles":changes,"verdict":verdict,"summary":summary,"k_range":k,"plot_image":image,"plot_svg":svg,"exact_roots":roots.into_iter().map(|z|root_json(z,false)).collect::<Vec<_>>()}),
    )
}

fn stability_plot(
    poly: &SPolynomial,
    k: &Value,
    roots: &[Complex64],
    curves: &[(String, Vec<f64>, Vec<f64>)],
) -> Result<(String, String)> {
    use crate::time_response::Chart;
    let mut chart = Chart::new(
        1148.,
        1008.,
        (110., 45., 990., 865.),
        "Eixo Real (σ)",
        "Eixo Imaginário (jω)",
        false,
    );
    let mut xs = vec![0.];
    let mut ys = vec![0.];
    for (_, x, y) in curves {
        xs.extend(x);
        ys.extend(y);
    }
    let min = xs.iter().copied().fold(0., f64::min);
    let max = xs.iter().copied().fold(0., f64::max);
    let im = ys.iter().map(|v| v.abs()).fold(2., f64::max);
    let pad = (max - min).max(4.) * 0.18;
    let (mut left, mut right, mut top) = ((min - pad).min(-1.), (max + pad).max(1.5), im * 1.3);
    let aspect = 990. / 865.;
    if (right - left) / (2. * top) < aspect {
        let c = (left + right) / 2.;
        let span = 2. * top * aspect;
        left = c - span / 2.;
        right = c + span / 2.;
    } else {
        top = (right - left) / aspect / 2.;
    }
    chart.bounds = (left, right, -top, top);
    chart.axes();
    chart.line(&[left, right], &[0., 0.], "black", 0.8, "");
    chart.line(&[0., 0.], &[-top, top], "black", 0.8, "");
    let poles = if poly.has_k() {
        numerical_roots(poly, 0.)?
    } else {
        roots.to_vec()
    };
    if poly.has_k() {
        for (_, x, y) in curves {
            chart.line(x, y, "black", 2., "");
        }
    }
    for pole in &poles {
        let (x, y) = chart.xy(pole.re, pole.im);
        chart.svg += &format!(
            r#"<path data-pole="original" d="M{} {}l12 12m-12 0l12 -12" stroke="black" stroke-width="2.2" fill="none"/>"#,
            x - 6.,
            y - 6.
        );
        let tag = if pole.re < -1e-4 {
            "SPE"
        } else if pole.re > 1e-4 {
            "SPD"
        } else {
            "jω"
        };
        let label = if poly.has_k() {
            format!("p={}", root_string(*pole))
        } else {
            format!("p={} [{tag}]", root_string(*pole))
        };
        chart.label(
            pole.re,
            pole.im,
            &label,
            0.,
            if pole.im >= 0. { -18. } else { 24. },
            true,
        );
    }
    if poly.has_k() {
        let n = polynomial::add(&poly.numeric(1.), &poly.numeric(0.), -1.);
        for zero in polynomial::roots(&n)? {
            let (x, y) = chart.xy(zero.re, zero.im);
            chart.svg += &format!(
                r#"<circle data-zero="finite" cx="{x}" cy="{y}" r="6" stroke="black" stroke-width="2" fill="white"/>"#
            );
            chart.label(
                zero.re,
                zero.im,
                &format!("z={}", root_string(zero)),
                0.,
                24.,
                true,
            );
        }
        for critical in k["critical_k"].as_array().into_iter().flatten() {
            if let Some(gain) = critical["k_val"].as_f64() {
                for p in numerical_roots(poly, gain)? {
                    if p.re.abs() < 1e-2 && p.im.abs() > 1e-4 {
                        let (x, y) = chart.xy(0., p.im);
                        chart.svg += &format!(
                            r##"<circle data-critical-k="{gain}" cx="{x}" cy="{y}" r="6" stroke="#B91C1C" stroke-width="2" fill="white"/>"##
                        );
                        chart.label(
                            0.,
                            p.im,
                            &format!("jω={:+.2}; K={gain:.1}", p.im),
                            14.,
                            0.,
                            true,
                        );
                    }
                }
            }
        }
        chart.legend(
            &[
                ("Ramos do LGR", "black", false),
                ("Polos (Início)", "black", false),
                ("Zeros (Término)", "black", false),
            ],
            true,
        );
    } else {
        chart.legend(&[("Polos no plano s", "black", false)], true);
    }
    chart.finish()
}

pub fn evaluate_k(data: &Value) -> Result<Value> {
    let source = data["expr"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("s^3 + 3s^2 + 2s + K");
    let (poly, _) = parse_characteristic(source, true)?;
    require_affine_gain(&poly)?;
    let k = data["k_val"].as_f64().unwrap_or(1.);
    if !k.is_finite() {
        return Err("O ganho K deve ser finito.".into());
    }
    // The Python endpoint uses feedback(K*N,D), with N=1 when K is absent.
    let roots = if k == 0. {
        vec![]
    } else if !poly.has_k() {
        let mut c = poly.numeric(0.);
        *c.last_mut().unwrap() += k;
        polynomial::roots(&c)?
    } else {
        numerical_roots(&poly, k)?
    };
    let spd = roots.iter().filter(|r| r.re > 1e-4).count();
    let jw = roots.iter().filter(|r| r.re.abs() <= 1e-4).count();
    let verdict = if spd > 0 {
        "Instável"
    } else if jw > 0 {
        "Marginalmente Estável"
    } else {
        "Estável"
    };
    Ok(
        json!({"success":true,"k_val":k,"verdict":verdict,"spd_count":spd,"jw_count":jw,"roots":roots.into_iter().map(|r|root_json(r,true)).collect::<Vec<_>>()}),
    )
}

pub fn presets() -> Value {
    let catalog = json!({
"Exemplo 1: 3ª Ordem com Faixa de Ganho K (Ogata)":{"expr":"1 / (s * (s + 1) * (s + 2))","desc":"Sistema de 3ª ordem em malha fechada. Determina a faixa analítica 0 < K < 6 e ganho crítico K_crit = 6."},
"Exemplo 2: 3ª Ordem Estável (Numérico)":{"expr":"s^3 + 6*s^2 + 11*s + 6","desc":"Polinômio cúbico sem trocas de sinal na 1ª coluna. Todos os polos no semiplano esquerdo (LHP)."},
"Exemplo 3: 3ª Ordem Instável (2 Raízes no RHP)":{"expr":"s^3 + s^2 + 2*s + 24","desc":"Duas trocas de sinal na primeira coluna (+, +, -, +), indicando 2 polos no semiplano direito."},
"Exemplo 4: 4ª Ordem com Primeiro Elemento Nulo (Caso ε)":{"expr":"s^4 + s^3 + 2*s^2 + 2*s + 3","desc":"Elemento zero na linha s², contornado via limite infinitesimal com ε > 0."},
"Exemplo 5: 4ª Ordem com Linha de Zeros (Marginalmente Estável)":{"expr":"s^4 + 2*s^3 + 6*s^2 + 8*s + 8","desc":"Linha nula em s¹, gerando polinômio auxiliar A(s) = 2s² + 8 e polos puramente imaginários em ±2j."},
"Exemplo 6: 5ª Ordem de Alta Complexidade":{"expr":"s^5 + 2*s^4 + 2*s^3 + 4*s^2 + 11*s + 10","desc":"Sistema de 5ª ordem com caso especial de elemento nulo e 2 trocas de sinal."}});
    json!({"success":true,"presets":catalog.as_object().unwrap().iter().map(|(id,preset)|json!({"id":id,"title":id,"expr":preset["expr"],"desc":preset["desc"]})).collect::<Vec<_>>()})
}
