//! Controller synthesis: exact geometry/magnitude equations and classical tuning.
//! Deduction templates below are generated from the trusted migration source;
//! only formatting operations are interpreted, never user expressions or code.
use crate::{
    Result, controller_math as math, polynomial,
    simulation::{self, Input},
    transfer::{self, TransferFunction},
    values,
};
use num_complex::Complex64 as C;
use serde_json::{Map, Value, json};
use std::f64::consts::PI;
use std::sync::OnceLock;

type Context = Map<String, Value>;
fn n(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| match v.as_str() {
        Some("∞") => f64::INFINITY,
        Some("-∞") => f64::NEG_INFINITY,
        _ => 0.,
    })
}
fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".into(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        _ => v.to_string(),
    }
}
fn truth(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(m) => !m.is_empty(),
        _ => n(v) != 0.,
    }
}
fn set(c: &mut Context, k: &str, v: impl Into<Value>) {
    c.insert(k.into(), v.into());
}
fn put(c: &mut Context, k: &str, v: f64) {
    set(c, k, values::number(v));
}
fn get(c: &Context, k: &str) -> Value {
    c.get(k).cloned().unwrap_or(Value::Null)
}
fn num(c: &Context, k: &str) -> f64 {
    n(&get(c, k))
}
fn fmt(v: &Value, digits: usize, tex: bool) -> String {
    if v.is_null() {
        return if tex { "0" } else { "—" }.into();
    }
    if let Some(s) = v.as_str() {
        return s.into();
    }
    let mut x = n(v);
    if !x.is_finite() {
        return if tex {
            "\\infty"
        } else if x > 0. {
            "∞"
        } else {
            "-∞"
        }
        .into();
    }
    if x.abs() < 1e-12 {
        x = 0.;
    }
    if (x - x.round()).abs() < 1e-9 {
        return format!("{x:.0}");
    }
    values::general(x, digits).replace('.', if tex { "{,}" } else { "," })
}
fn ff(v: &Value, spec: &str) -> String {
    let v = n(v);
    let plus = spec.starts_with('+');
    let spec = spec.trim_start_matches('+').trim_start_matches('0');
    if let Some(d) = spec
        .strip_prefix('.')
        .and_then(|v| v.strip_suffix('g'))
        .and_then(|v| v.parse().ok())
    {
        let out = values::general(v, d);
        if plus && !out.starts_with('-') {
            format!("+{out}")
        } else {
            out
        }
    } else {
        text(&values::number(v))
    }
}
fn templates() -> &'static Value {
    static T: OnceLock<Value> = OnceLock::new();
    T.get_or_init(|| {
        serde_json::from_str(DEDUCTION_TEMPLATES).expect("validated static deduction templates")
    })
}
fn template(line: u32, c: &Context) -> Result<Value> {
    let key = line.to_string();
    let v = templates()
        .get(&key)
        .ok_or_else(|| format!("Template de dedução ausente: {line}."))?;
    eval(v, c)
}
fn eval(ast: &Value, c: &Context) -> Result<Value> {
    let tag = ast["t"].as_str().ok_or("Template inválido.")?;
    let e = |key: &str| eval(&ast[key], c);
    match tag {
        "Constant" => Ok(ast["value"].clone()),
        "Name" => c
            .get(ast["id"].as_str().unwrap())
            .cloned()
            .ok_or_else(|| format!("Variável de dedução ausente: {}", ast["id"])),
        "List" | "Tuple" | "Set" => Ok(Value::Array(
            ast["elts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| eval(a, c))
                .collect::<Result<_>>()?,
        )),
        "Dict" => {
            let mut out = Map::new();
            for (k, v) in ast["keys"]
                .as_array()
                .unwrap()
                .iter()
                .zip(ast["values"].as_array().unwrap())
            {
                out.insert(text(&eval(k, c)?), eval(v, c)?);
            }
            Ok(Value::Object(out))
        }
        "JoinedStr" => {
            let mut out = String::new();
            for a in ast["values"].as_array().unwrap() {
                out.push_str(&text(&eval(a, c)?));
            }
            Ok(json!(out))
        }
        "FormattedValue" => {
            let v = e("value")?;
            if ast["format_spec"].is_null() {
                Ok(json!(text(&v)))
            } else {
                Ok(json!(ff(&v, &text(&e("format_spec")?))))
            }
        }
        "Subscript" => {
            let a = e("value")?;
            let k = e("slice")?;
            Ok(if let Some(k) = k.as_str() {
                a[k].clone()
            } else {
                a[n(&k) as usize].clone()
            })
        }
        "Attribute" => {
            let a = e("value")?;
            let key = ast["attr"].as_str().unwrap();
            Ok(a[key].clone())
        }
        "IfExp" => {
            if truth(&e("test")?) {
                e("body")
            } else {
                e("orelse")
            }
        }
        "UnaryOp" => {
            let a = e("operand")?;
            match ast["op"]["t"].as_str().unwrap() {
                "USub" => Ok(values::number(-n(&a))),
                "Not" => Ok(json!(!truth(&a))),
                _ => Ok(a),
            }
        }
        "BinOp" => {
            let a = e("left")?;
            let b = e("right")?;
            match ast["op"]["t"].as_str().unwrap() {
                "Add" => {
                    if a.is_string() || b.is_string() {
                        Ok(json!(text(&a) + &text(&b)))
                    } else if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
                        Ok(json!(a.iter().chain(b).cloned().collect::<Vec<_>>()))
                    } else {
                        Ok(if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                            json!(a + b)
                        } else {
                            values::number(n(&a) + n(&b))
                        })
                    }
                }
                "Sub" => Ok(values::number(n(&a) - n(&b))),
                "Mult" => Ok(values::number(n(&a) * n(&b))),
                "Div" => Ok(values::number(n(&a) / n(&b))),
                "Pow" => Ok(values::number(n(&a).powf(n(&b)))),
                _ => Err("Operador de template inválido.".into()),
            }
        }
        "Compare" => {
            let mut a = e("left")?;
            let mut yes = true;
            for (op, b) in ast["ops"]
                .as_array()
                .unwrap()
                .iter()
                .zip(ast["comparators"].as_array().unwrap())
            {
                let b = eval(b, c)?;
                yes &= match op["t"].as_str().unwrap() {
                    "Eq" => {
                        if a.is_number() && b.is_number() {
                            n(&a) == n(&b)
                        } else {
                            a == b
                        }
                    }
                    "NotEq" => a != b,
                    "Gt" => n(&a) > n(&b),
                    "Lt" => n(&a) < n(&b),
                    "GtE" => n(&a) >= n(&b),
                    "LtE" => n(&a) <= n(&b),
                    "Is" => a == b,
                    "IsNot" => a != b,
                    "In" => b.as_array().is_some_and(|b| b.contains(&a)),
                    _ => false,
                };
                a = b;
            }
            Ok(json!(yes))
        }
        "BoolOp" => {
            let and = ast["op"]["t"] == "And";
            let mut v = Value::Null;
            for a in ast["values"].as_array().unwrap() {
                v = eval(a, c)?;
                if truth(&v) != and {
                    break;
                }
            }
            Ok(v)
        }
        "ListComp" => {
            let g = &ast["generators"][0];
            let values = eval(&g["iter"], c)?;
            let mut out = vec![];
            for v in values.as_array().ok_or("Compreensão inválida.")? {
                let mut scope = c.clone();
                set(&mut scope, g["target"]["id"].as_str().unwrap(), v.clone());
                let mut valid = true;
                for pred in g["ifs"].as_array().unwrap() {
                    valid &= truth(&eval(pred, &scope)?);
                }
                if valid {
                    out.push(eval(&ast["elt"], &scope)?);
                }
            }
            Ok(json!(out))
        }
        "Call" => {
            let args = ast["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| eval(a, c))
                .collect::<Result<Vec<_>>>()?;
            let f = &ast["func"];
            if f["t"] == "Name" {
                let name = f["id"].as_str().unwrap();
                match name {
                    "_fmt_num" | "_fmt_tex_num" => Ok(json!(fmt(
                        &args[0],
                        args.get(1).map(n).unwrap_or(4.) as usize,
                        name == "_fmt_tex_num"
                    ))),
                    "_fmt_tex_signed" => Ok(json!(format!(
                        "{} {}",
                        if n(&args[0]) < 0. { "-" } else { "+" },
                        fmt(
                            &values::number(n(&args[0]).abs()),
                            args.get(1).map(n).unwrap_or(4.) as usize,
                            true
                        )
                    ))),
                    "mp_from_xi" => Ok(values::number(math::mp_from_xi(n(&args[0])))),
                    "float" => Ok(values::number(n(&args[0]))),
                    "int" => Ok(json!(n(&args[0]) as i64)),
                    "str" => Ok(json!(text(&args[0]))),
                    "abs" => Ok(values::number(n(&args[0]).abs())),
                    "len" => Ok(json!(
                        args[0]
                            .as_array()
                            .map(Vec::len)
                            .or_else(|| args[0].as_object().map(Map::len))
                            .unwrap_or(0)
                    )),
                    "_build_controller_latex" => Ok(json!(controller_latex(
                        args[0].as_str().unwrap(),
                        &args[1]
                    )?)),
                    _ => Err(format!("Fonction de template indisponível: {name}")),
                }
            } else if f["t"] == "Attribute" {
                let object = eval(&f["value"], c)?;
                match f["attr"].as_str().unwrap() {
                    "get" => Ok(object
                        .get(text(&args[0]))
                        .cloned()
                        .unwrap_or_else(|| args.get(1).cloned().unwrap_or(Value::Null))),
                    "replace" => Ok(json!(
                        text(&object).replace(&text(&args[0]), &text(&args[1]))
                    )),
                    "strip" => Ok(json!(text(&object).trim())),
                    "lower" => Ok(json!(text(&object).to_lowercase())),
                    _ => Err(format!("Método de template indisponível: {}", f["attr"])),
                }
            } else {
                Err("Chamada de template inválida.".into())
            }
        }
        _ => Err(format!("Nó de template não suportado: {tag}")),
    }
}
fn assign(c: &mut Context, name: &str, line: u32) -> Result<()> {
    let v = template(line, c)?;
    set(c, name, v);
    Ok(())
}
fn append(c: &mut Context, name: &str, line: u32, extend: bool) -> Result<()> {
    let v = template(line, c)?;
    let a = c
        .entry(name)
        .or_insert(json!([]))
        .as_array_mut()
        .ok_or("Destino de dedução inválido.")?;
    if extend {
        a.extend(v.as_array().ok_or("Lista de dedução inválida.")?.clone());
    } else {
        a.push(v);
    }
    Ok(())
}
fn params(c: &mut Context, line: u32) -> Result<()> {
    let v = template(line, c)?;
    c.get_mut("controller_params")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .extend(v.as_object().unwrap().clone());
    Ok(())
}
fn optional(p: &Value, k: &str) -> Result<Option<f64>> {
    math::optional(&p[k])
}
fn positive(v: f64, label: &str) -> Result<f64> {
    if v.is_finite() && v > 0. {
        Ok(v)
    } else {
        Err(format!("{label} deve ser maior que zero."))
    }
}
fn root(v: C) -> Value {
    json!({"real":v.re,"imag":v.im})
}
fn error_raw(v: &Value) -> f64 {
    n(&v["value"])
}
fn tf_latex(tf: &TransferFunction, float: bool, factored: bool) -> String {
    let (a, b) = transfer::format_latex(&tf.num, &tf.den, float);
    if factored { b } else { a }
}

#[derive(Clone)]
struct Group {
    root: C,
    multiplicity: usize,
}
fn groups(mut roots: Vec<C>) -> Vec<Group> {
    roots.sort_by(|a, b| {
        let round = |v: f64| (v * 10000.).round() / 10000.;
        round(b.re)
            .total_cmp(&round(a.re))
            .then(round(a.im).total_cmp(&round(b.im)))
    });
    let mut out = Vec::<Group>::new();
    for mut r in roots {
        if r.im.abs() < 1e-4 {
            r.im = 0.;
        }
        if r.norm() < 1e-4 {
            r = C::new(0., 0.);
        }
        if let Some(g) = out.iter_mut().find(|g| (r - g.root).norm() <= 1e-4) {
            g.multiplicity += 1;
        } else {
            out.push(Group {
                root: r,
                multiplicity: 1,
            });
        }
    }
    out
}
fn angle(root_value: C, re: f64, im: f64) -> Result<Value> {
    let mut c = Context::new();
    put(&mut c, "re_dom", re);
    put(&mut c, "im_dom", im);
    set(&mut c, "root", root(root_value));
    let a;
    let line;
    if root_value.im.abs() < 1e-6 {
        let pos = -root_value.re;
        a = math::lgr_angle(pos, re, im);
        put(&mut c, "p_i", pos);
        put(&mut c, "re_v", re);
        put(&mut c, "im_v", im);
        put(&mut c, "atan_deg", 180. - a);
        line = if (pos - re).abs() <= 1e-7 {
            405
        } else if pos < re {
            413
        } else {
            422
        };
    } else {
        let raw = (im - root_value.im).atan2(-re - root_value.re).to_degrees();
        a = if raw < 0. { raw + 360. } else { raw };
        line = 446;
    }
    put(&mut c, "angle_deg", a);
    Ok(json!({"angle_deg":a,"formula_latex":template(line,&c)?}))
}
fn position(theta: f64, re: f64, im: f64, symbol: &str) -> Result<Value> {
    let mut c = Context::new();
    let pos = math::position_from_angle(theta, re, im);
    put(&mut c, "theta", theta);
    put(&mut c, "re_v", re);
    put(&mut c, "im_v", im);
    put(&mut c, "pos", pos);
    set(&mut c, "symbol", symbol);
    let line = if (theta - 90.).abs() <= 1e-6 {
        504
    } else if theta < 90. {
        509
    } else {
        519
    };
    Ok(json!({"pos":pos,"formula_latex":template(line,&c)?}))
}
fn magnitude(s: C, g: &Group, label: Option<String>) -> Result<Value> {
    let mut c = Context::new();
    let diff = s - g.root;
    let pos = -g.root.re;
    let m = g.multiplicity;
    let factor = if g.root.im.abs() < 1e-7 {
        if pos.abs() < 1e-9 {
            if m == 1 { "s".into() } else { format!("s^{m}") }
        } else {
            format!(
                "(s {} {}){}",
                if pos >= 0. { "+" } else { "-" },
                fmt(&json!(pos.abs()), 4, true),
                if m > 1 {
                    format!("^{m}")
                } else {
                    String::new()
                }
            )
        }
    } else {
        format!(
            "(s - ({} {} {}j))",
            fmt(&json!(g.root.re), 4, true),
            if g.root.im < 0. { "-" } else { "+" },
            fmt(&json!(g.root.im.abs()), 4, true)
        )
    };
    set(&mut c, "factor_tex", factor.clone());
    put(&mut c, "re_i", diff.re);
    put(&mut c, "im_i", diff.im);
    set(&mut c, "multiplicity", m);
    put(&mut c, "total_mag", diff.norm().powi(m as i32));
    let formula = template(676, &c)?;
    Ok(
        json!({"label":label.unwrap_or_else(||factor.clone()),"factor_tex":factor,"re_i":diff.re,"im_i":diff.im,"single_magnitude":diff.norm(),"multiplicity":m,"magnitude":diff.norm().powi(m as i32),"formula_latex":formula}),
    )
}
fn controller_latex(kind: &str, p: &Value) -> Result<String> {
    let mut c = Context::new();
    set(&mut c, "params", p.clone());
    for (key, param) in [
        ("k_s", "K"),
        ("kp_s", "kp"),
        ("td_s", "td"),
        ("z_s", "z"),
        ("p_s", "p"),
    ] {
        set(&mut c, key, fmt(&p[param], 4, true));
    }
    if kind == "LAG" {
        let k = n(&p["K"]);
        set(
            &mut c,
            "k_s",
            if (k - 1.).abs() < 1e-7 {
                "".into()
            } else if (k + 1.).abs() < 1e-7 {
                "-".into()
            } else {
                fmt(&p["K"], 4, true)
            },
        );
    }
    Ok(text(&template(
        match kind {
            "LEAD" => 699,
            "LAG" => 705,
            "P" => 708,
            "PD" if n(&p["td"]) == 0. => 712,
            "PD" => 715,
            "PI" => 719,
            "PID" => 722,
            _ => 726,
        },
        &c,
    )?))
}
fn uncompensated(tf: &TransferFunction, mode: &str) -> Result<Value> {
    let cl = tf.feedback(&TransferFunction::scalar(1.), -1.)?;
    let poles = cl.poles()?;
    let mut wn = None;
    let mut xi = None;
    let mut mp = None;
    let mut ts2 = None;
    let mut ts5 = None;
    if cl.den.len() == 3 && cl.den[0] != 0. {
        let a0 = cl.den[2] / cl.den[0];
        if a0 > 0. {
            let w = a0.sqrt();
            wn = Some(w);
            xi = Some(cl.den[1] / cl.den[0] / (2. * w));
        }
    } else if let Some(dom) = poles
        .iter()
        .filter(|r| r.im.abs() > 1e-6 && r.re < 0.)
        .max_by(|a, b| a.re.total_cmp(&b.re))
    {
        wn = Some(dom.norm());
        xi = Some(-dom.re / dom.norm());
    }
    if let (Some(w), Some(x)) = (wn, xi)
        && x > 0.
        && x < 1.
    {
        mp = Some(math::mp_from_xi(x));
        ts2 = Some(4. / (x * w));
        ts5 = Some(3. / (x * w));
    }
    let err = math::steady_state_error(&tf.num, &tf.den, mode);
    Ok(
        json!({"closed_loop_num":cl.num,"closed_loop_den":cl.den,"closed_loop_latex":tf_latex(&cl,true,false).replace("G(s) = ","T(s) = "),"closed_loop_poles":values::complex_list(&poles),"stable":!poles.is_empty()&&poles.iter().all(|r|r.re< -1e-9),"wn":wn,"xi":xi,"mp":mp,"ts_2pct":ts2,"ts_5pct":ts5,"steady_state_error":err["json_value"],"steady_state_error_raw":err["value"],"steady_state_error_latex":err["latex_value"]}),
    )
}

const DEDUCTION_TEMPLATES: &str = include_str!("controller_templates.json");

fn root_description(g: &[Group]) -> String {
    if g.is_empty() {
        return "nenhum".into();
    }
    g.iter()
        .map(|g| {
            let r = g.root;
            if r.im == 0. {
                format!(
                    "s = {}{}",
                    fmt(&json!(r.re), 4, false),
                    if g.multiplicity > 1 {
                        format!(" (×{})", g.multiplicity)
                    } else {
                        String::new()
                    }
                )
            } else {
                format!(
                    "s = {}{}j",
                    fmt(&json!(r.re), 4, false),
                    ff(&json!(r.im), "+0.3g")
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
fn add_angles(
    c: &mut Context,
    g: &[Group],
    dominant: (f64, f64),
    zero: bool,
    total_poles: bool,
    kind: &str,
    line: u32,
) -> Result<f64> {
    let (re, im) = dominant;
    let mut sum = 0.;
    for (idx, g) in g.iter().enumerate() {
        let a = angle(g.root, re, im)?;
        let theta = n(&a["angle_deg"]);
        let total = theta * g.multiplicity as f64;
        sum += total;
        set(
            c,
            if zero { "gz" } else { "gp" },
            json!({"root":root(g.root),"multiplicity":g.multiplicity}),
        );
        set(c, "idx", idx + 1);
        set(c, "ang_info", a);
        put(c, "tot_ang", total);
        set(
            c,
            "mult_str",
            if g.multiplicity > 1 {
                format!("{} \\times ", g.multiplicity)
            } else {
                String::new()
            },
        );
        let origin = g.root.norm() < 1e-7;
        set(
            c,
            "origin_note",
            if origin {
                format!(" (inclui polo do {kind} em s = 0)")
            } else {
                String::new()
            },
        );
        if line > 0 {
            append(c, "angle_eqs", line, false)?;
        }
        let label = if total_poles {
            format!(
                "Polo s = {}{}",
                fmt(&json!(g.root.re), 4, false),
                if g.multiplicity > 1 {
                    format!(" (×{})", g.multiplicity)
                } else {
                    String::new()
                }
            )
        } else {
            format!(
                "{} G (s = {})",
                if zero { "Zero" } else { "Polo" },
                fmt(&json!(g.root.re), 4, false)
            )
        };
        let category = if zero {
            "plant_zero"
        } else if total_poles && origin && g.multiplicity == 1 {
            "controller_pole"
        } else {
            "plant_pole"
        };
        add_contribution(c, category, label, g.root, g.multiplicity, theta);
    }
    Ok(sum)
}
fn add_contribution(c: &mut Context, category: &str, label: String, r: C, m: usize, theta: f64) {
    c.get_mut("angle_contributions").unwrap().as_array_mut().unwrap().push(json!({"type":category,"label":label,"root":{"re":r.re,"im":r.im},"multiplicity":m,"angle_deg":theta,"total_angle_deg":theta*m as f64}));
}
fn group_magnitudes(s: C, g: &[Group]) -> Result<Vec<Value>> {
    g.iter().map(|g| magnitude(s, g, None)).collect()
}
fn extra_magnitude(s: C, pos: f64) -> Result<Value> {
    magnitude(
        s,
        &Group {
            root: C::new(-pos, 0.),
            multiplicity: 1,
        },
        Some(format!("(s + {})", fmt(&json!(pos), 4, false))),
    )
}
fn set_magnitudes(c: &mut Context, mut zeros: Vec<Value>, poles: Vec<Value>, gain: f64) -> f64 {
    let a = gain.abs() * zeros.iter().map(|v| n(&v["magnitude"])).product::<f64>();
    let b = poles.iter().map(|v| n(&v["magnitude"])).product::<f64>();
    put(c, "num_mag_prod", a);
    put(c, "den_mag_prod", b);
    set(c, "mag_num_factors", std::mem::take(&mut zeros));
    set(c, "mag_den_factors", poles);
    b / a
}
fn rounded_tf(tf: &TransferFunction, digits: usize, zero: f64) -> TransferFunction {
    let round = |c: &f64| {
        let v = if c.abs() < zero { 0. } else { *c };
        values::general(v, digits).parse::<f64>().unwrap_or(v)
    };
    TransferFunction::new(
        tf.num.iter().map(round).collect(),
        tf.den.iter().map(round).collect(),
    )
    .unwrap()
}
fn steady_mode(p: &Value) -> String {
    let raw = p["input_type"].as_str().unwrap_or("step").to_lowercase();
    match raw.trim() {
        "parabola" | "parábola" | "parabolic" | "parabolica" | "parabólica" | "1/s^3" | "1/s3" => {
            "parabola"
        }
        "ramp" | "rampa" | "1/s^2" | "1/s2" => "ramp",
        _ => "step",
    }
    .into()
}
fn minreal(tf: &TransferFunction) -> Result<TransferFunction> {
    let mut poles = tf.poles()?;
    let mut zeros = tf.zeros()?;
    let tolerance = f64::EPSILON.sqrt();
    zeros.retain(|z| {
        if let Some(index) = poles
            .iter()
            .position(|p| (*p - *z).norm() < 1000. * z.norm().max(tolerance) * tolerance)
        {
            poles.remove(index);
            false
        } else {
            true
        }
    });
    let gain = tf.num[0] / tf.den[0];
    TransferFunction::new(
        polynomial::from_roots(&zeros, gain)
            .iter()
            .map(|z| z.re)
            .collect(),
        polynomial::from_roots(&poles, 1.)
            .iter()
            .map(|z| z.re)
            .collect(),
    )
}

pub fn analyze(p: &Value) -> Result<Value> {
    let expr = p["plant_expr"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| p["expr"].as_str().filter(|s| !s.is_empty()))
        .unwrap_or("5 / (s * (s + 1))");
    let plant = TransferFunction::parse(expr)?;
    let method = p["method"].as_str().unwrap_or("").to_lowercase();
    if [
        "zn_critical",
        "zn_critical_auto",
        "zn_critical_manual",
        "zn_reaction",
        "cohen_coon",
        "chr",
    ]
    .contains(&method.as_str())
        || p["design_type"]
            .as_str()
            .is_some_and(|s| s.eq_ignore_ascii_case("lead_lag"))
    {
        return legacy(p, &plant);
    }
    universal(p, &plant)
}
fn universal(p: &Value, forward: &TransferFunction) -> Result<Value> {
    let mut c = Context::new();
    let default_kind = if p.get("controller_type").is_none() {
        "lead"
    } else {
        p["design_type"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("lead")
    };
    let mut kind = p["controller_type"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or(default_kind)
        .trim()
        .to_uppercase();
    if ["AVANCO", "AVANÇO"].contains(&kind.as_str()) {
        kind = "LEAD".into();
    } else if kind == "ATRASO" {
        kind = "LAG".into();
    }
    let feedback = p["feedback_expr"]
        .as_str()
        .filter(|s| !s.trim().is_empty() && !matches!(s.trim(), "1" | "1.0"))
        .map(TransferFunction::parse)
        .transpose()?;
    let plant = if let Some(h) = &feedback {
        minreal(&forward.multiply(h)?)?
    } else {
        forward.clone()
    };
    let _float_input = feedback.is_some()
        || p["plant_expr"]
            .as_str()
            .or_else(|| p["expr"].as_str())
            .unwrap_or("")
            .contains('.');
    let source = p["plant_expr"]
        .as_str()
        .or_else(|| p["expr"].as_str())
        .unwrap_or("5 / (s * (s + 1))");
    let (source_exp, source_fact) = transfer::expression_latex(source)?;
    let forward_latex = source_fact.clone();
    let mut plant_exp = if feedback.is_some() {
        tf_latex(&plant, true, false)
    } else {
        source_exp
    };
    let mut plant_fact = if feedback.is_some() {
        tf_latex(&plant, true, true)
    } else {
        source_fact
    };
    let feedback_latex = feedback.as_ref().map(|h| {
        transfer::expression_latex(p["feedback_expr"].as_str().unwrap_or("1"))
            .map(|(_, f)| f)
            .unwrap_or_else(|_| tf_latex(h, true, true))
            .replace("G(s) = ", "H(s) = ")
    });
    if feedback.is_some() {
        plant_exp = plant_exp.replace("G(s) = ", "G(s)H(s) = ");
        plant_fact = plant_fact.replace("G(s) = ", "G(s)H(s) = ");
    }
    let mut mode = steady_mode(p);
    let input_label = match mode.as_str() {
        "ramp" => "Rampa unitária R(s) = 1/s²",
        "parabola" => "Parábola unitária R(s) = 1/s³",
        _ => "Degrau unitário R(s) = 1/s",
    };
    let gain = plant.num[0] / plant.den[0];
    let gain_sign = if gain < 0. { -1. } else { 1. };
    let poles_raw = plant.poles()?;
    let zeros_raw = plant.zeros()?;
    let poles = groups(poles_raw.clone());
    let zeros = groups(zeros_raw.clone());
    let uncomp = uncompensated(&plant, &mode)?;
    set(&mut c, "kind", kind.clone());
    set(
        &mut c,
        "feedback_tf",
        feedback
            .as_ref()
            .map(|_| json!(true))
            .unwrap_or(Value::Null),
    );
    put(
        &mut c,
        "feedback_dc",
        feedback.as_ref().map_or(1., TransferFunction::dc_gain),
    );
    put(&mut c, "lead_gain", gain);
    put(&mut c, "gain_sign", gain_sign);
    set(&mut c, "plant_latex_factored", plant_fact.clone());
    set(&mut c, "plant_latex_expanded", plant_exp.clone());
    set(&mut c, "forward_latex_factored", forward_latex.clone());
    set(&mut c, "feedback_latex", feedback_latex.clone());
    set(&mut c, "uncomp", uncomp.clone());
    set(&mut c, "input_label", input_label);
    set(&mut c, "poles_desc", root_description(&poles));
    set(&mut c, "zeros_desc", root_description(&zeros));
    set(&mut c, "steps", json!([]));
    set(&mut c, "angle_contributions", json!([]));
    set(&mut c, "design_warnings", json!([]));
    set(&mut c, "controller_params", json!({"controller_type":kind}));
    assign(&mut c, "step1_eqs", 808)?;
    if !uncomp["wn"].is_null() && !uncomp["xi"].is_null() {
        let eq = format!(
            "s^2 + 2\\xi\\omega_n s + \\omega_n^2 \\implies \\omega_{{n,\\text{{atual}}}} = {}\\text{{ rad/s}}, \\quad \\xi_{{\\text{{atual}}}} = {}",
            fmt(&uncomp["wn"], 4, true),
            fmt(&uncomp["xi"], 4, true)
        );
        c.get_mut("step1_eqs")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(json!(eq));
    }
    c.get_mut("step1_eqs").unwrap().as_array_mut().unwrap().push(json!(format!("e_{{\\text{{atual}}}}(\\infty) = \\lim_{{s \\to 0}} s\\left(\\frac{{1}}{{1 + G(s)}}\\right)R(s) = {}",uncomp["steady_state_error_latex"].as_str().unwrap())));
    assign(&mut c, "step1_notes", 832)?;
    if let Some(h) = &feedback {
        let a = c.get_mut("step1_eqs").unwrap().as_array_mut().unwrap();
        a.splice(
            0..1,
            [
                json!(forward_latex),
                json!(feedback_latex),
                json!(plant_fact),
            ],
        );
        c.get_mut("step1_notes").unwrap().as_array_mut().unwrap().push(json!("Realimentação não unitária: o LGR, os ângulos e o ganho são calculados sobre a malha aberta G(s)H(s)."));
        let _ = h;
    }
    if gain_sign < 0. {
        c.get_mut("step1_notes").unwrap().as_array_mut().unwrap().push(json!("A planta tem ganho negativo: o projeto é feito para -G(s) e o ganho do controlador resulta negativo."));
    }
    append(&mut c, "steps", 841, false)?;
    let mut controller;
    if kind == "LAG" {
        assign(&mut c, "design_label", 859)?;
        let k = positive(optional(p, "lag_gain")?.unwrap_or(1.), "O ganho K")?;
        let pole = positive(
            optional(p, "lag_pole")?.unwrap_or(0.01),
            "O polo p do controlador",
        )?;
        let design = plant.scale(gain_sign)?;
        let mut old = if gain_sign > 0. {
            n(&uncomp["steady_state_error_raw"])
        } else {
            error_raw(&math::steady_state_error(&design.num, &design.den, &mode))
        };
        if !old.is_finite() || old <= 0. {
            let mut found = false;
            for candidate in ["ramp", "parabola"] {
                if candidate == mode {
                    continue;
                }
                let value = error_raw(&math::steady_state_error(
                    &design.num,
                    &design.den,
                    candidate,
                ));
                if value.is_finite() && value > 0. {
                    old = value;
                    mode = candidate.into();
                    found = true;
                    break;
                }
            }
            if !found {
                return Err("O erro em regime permanente atual é zero ou infinito para a entrada selecionada; verifique a planta ou o tipo de entrada R(s).".into());
            }
        }
        let factor = optional(p, "error_reduction_factor")?;
        let target = optional(p, "target_steady_state_error")?;
        let (new, reduction) = if target.is_some_and(|v| v > 0.) && !factor.is_some_and(|v| v > 0.)
        {
            let new = target.unwrap();
            (new, old / new)
        } else {
            let reduction = factor.filter(|v| *v > 0.).unwrap_or(10.);
            (old / reduction, reduction)
        };
        if new <= 0. || new >= old {
            return Err("O novo erro em regime permanente e_novo(∞) deve ser positivo e menor que o erro atual.".into());
        }
        let ratio = if mode == "step" {
            (1. / new - 1.) / (k * design.dc_gain())
        } else {
            old / (k * new)
        };
        if !ratio.is_finite() || ratio <= 0. {
            return Err(
                "Não foi possível encontrar uma razão z/p positiva para a meta de erro informada."
                    .into(),
            );
        }
        let z = ratio * pole;
        if pole >= z {
            return Err(
                "No compensador por Atraso de Fase, o polo p deve ser menor que o zero z (p < z)."
                    .into(),
            );
        }
        controller = TransferFunction::new(vec![k, k * z], vec![1., pole])?;
        let cl = controller
            .multiply(&plant)?
            .scale(gain_sign)?
            .feedback(&TransferFunction::scalar(1.), -1.)?;
        let poles = cl.poles()?;
        let dom = poles
            .iter()
            .filter(|r| r.im.abs() > 1e-5 && r.re < 0.)
            .max_by(|a, b| a.re.total_cmp(&b.re))
            .copied()
            .map(|r| C::new(r.re, r.im.abs()))
            .unwrap_or(C::new(-1., 1.));
        let re = -dom.re;
        let im = dom.im;
        let wn = dom.norm();
        let xi = re / wn;
        for (key, v) in [
            ("k_val", k),
            ("p_val", pole),
            ("z_val", z),
            ("z_over_p", ratio),
            ("e_atual", old),
            ("e_novo", new),
            ("reduction_factor", reduction),
            ("re_dom", re),
            ("im_dom", im),
            ("wn_dom", wn),
            ("xi_dom", xi),
        ] {
            put(&mut c, key, v);
        }
        set(
            &mut c,
            "settling_criterion",
            p.get("settling_criterion").cloned().unwrap_or(json!("2")),
        );
        set(&mut c, "input_mode", mode.clone());
        params(&mut c, 929)?;
        append(&mut c, "steps", 947, false)?;
        append(&mut c, "steps", 962, false)?;
    } else {
        let specs = math::resolve_transient(p)?;
        for (key, spec) in [
            ("re_dom", "re"),
            ("im_dom", "im"),
            ("xi_val", "xi"),
            ("wn_val", "wn"),
            ("ts_val", "ts"),
            ("mp_val", "mp"),
            ("crit_pct", "criterion_pct"),
            ("settling_factor", "settling_factor"),
        ] {
            set(&mut c, key, specs[spec].clone());
        }
        set(&mut c, "specs", specs.clone());
        assign(
            &mut c,
            "step2_eqs",
            if specs["damping_source"] == "pole" {
                1006
            } else {
                1000
            },
        )?;
        append(&mut c, "steps", 1011, false)?;
        let mut re = num(&c, "re_dom");
        let mut im = num(&c, "im_dom");
        let mut dom = C::new(-re, im);
        set(&mut c, "angle_eqs", json!([]));
        match kind.as_str() {
            "LEAD" => {
                assign(&mut c, "design_label", 1026)?;
                let z = optional(p, "lead_zero")?.filter(|v| *v > 0.).unwrap_or(re);
                let info = angle(C::new(-z, 0.), re, im)?;
                let theta = n(&info["angle_deg"]);
                put(&mut c, "z_val", z);
                put(&mut c, "theta_z", theta);
                assign(&mut c, "angle_eqs", 1032)?;
                add_contribution(
                    &mut c,
                    "controller_zero",
                    format!("Zero Gc (s = -{})", fmt(&json!(z), 4, false)),
                    C::new(-z, 0.),
                    1,
                    theta,
                );
                let sumz = add_angles(&mut c, &zeros, (re, im), true, false, &kind, 1050)?;
                let sump = add_angles(&mut c, &poles, (re, im), false, false, &kind, 1066)?;
                let tp = math::required_single_angle(theta + sumz - sump + 180., "p")?;
                let pos = position(tp, re, im, "p")?;
                let pole = n(&pos["pos"]);
                if pole <= 0. {
                    return Err(format!(
                        "A condição de fase resultou em um polo p = {} fora do semiplano esquerdo. Ajuste as especificações ou a posição do zero z.",
                        values::general(pole, 4)
                    ));
                }
                if pole < z {
                    let warning = format!(
                        "O compensador resultante tem p = {} < z = {}, ou seja, é de atraso e não de avanço: o ponto desejado fica à direita do LGR. Considere um PI ou Atraso de Fase.",
                        fmt(&json!(pole), 4, false),
                        fmt(&json!(z), 4, false)
                    );
                    c.get_mut("design_warnings")
                        .unwrap()
                        .as_array_mut()
                        .unwrap()
                        .push(json!(warning));
                }
                for (key, v) in [
                    ("sum_zeros_g", sumz),
                    ("sum_poles_g", sump),
                    ("theta_p", tp),
                    ("p_val", pole),
                ] {
                    put(&mut c, key, v);
                }
                set(&mut c, "pos_p_info", pos);
                add_contribution(
                    &mut c,
                    "controller_pole",
                    format!("Polo Gc (s = -{})", fmt(&json!(pole), 4, false)),
                    C::new(-pole, 0.),
                    1,
                    tp,
                );
                append(&mut c, "angle_eqs", 1098, true)?;
                append(&mut c, "steps", 1104, false)?;
                let mut numerator = vec![extra_magnitude(dom, z)?];
                numerator.extend(group_magnitudes(dom, &zeros)?);
                let mut denominator = vec![extra_magnitude(dom, pole)?];
                denominator.extend(group_magnitudes(dom, &poles)?);
                let k = set_magnitudes(&mut c, numerator, denominator, gain);
                put(&mut c, "k_val", k);
                assign(&mut c, "mag_eqs", 1127)?;
                append(&mut c, "mag_eqs", 1128, false)?;
                append(&mut c, "steps", 1131, false)?;
                params(&mut c, 1142)?;
                controller = TransferFunction::new(vec![k, k * z], vec![1., pole])?;
            }
            "P" => {
                assign(&mut c, "design_label", 1162)?;
                put(&mut c, "re_theoretical", re);
                put(&mut c, "im_theoretical", im);
                let branch = optional(p, "lgr_branch_re")?
                    .filter(|v| *v > 0.)
                    .or_else(|| {
                        (poles_raw.len() == 2
                            && zeros_raw.is_empty()
                            && poles_raw.iter().all(|r| r.im.abs() < 1e-6))
                        .then(|| -poles_raw.iter().map(|r| r.re).sum::<f64>() / 2.)
                    });
                set(
                    &mut c,
                    "detected_branch_re",
                    branch.map(values::number).unwrap_or(Value::Null),
                );
                set(&mut c, "step3_eqs", json!([]));
                set(&mut c, "step3_notes", json!([]));
                if branch.is_some_and(|b| (b - re).abs() > 1e-5 && b > 0.) {
                    let tan = im / re;
                    let angle = tan.atan().to_degrees();
                    re = branch.unwrap();
                    im = re * tan;
                    dom = C::new(-re, im);
                    put(&mut c, "re_dom", re);
                    put(&mut c, "im_dom", im);
                    put(&mut c, "tan_theta", tan);
                    put(&mut c, "theta_deg", angle);
                    append(&mut c, "step3_eqs", 1183, true)?;
                    append(&mut c, "step3_notes", 1189, false)?;
                } else {
                    append(&mut c, "step3_eqs", 1191, false)?;
                    append(&mut c, "step3_notes", 1194, false)?;
                }
                add_angles(&mut c, &poles, (re, im), false, false, &kind, 0)?;
                append(&mut c, "steps", 1208, false)?;
                let a = group_magnitudes(dom, &zeros)?;
                let b = group_magnitudes(dom, &poles)?;
                let kp = set_magnitudes(&mut c, a, b, gain);
                put(&mut c, "kp_val", kp);
                assign(&mut c, "mag_eqs", 1222)?;
                append(&mut c, "mag_eqs", 1223, false)?;
                append(&mut c, "steps", 1226, false)?;
                params(&mut c, 1237)?;
                controller = TransferFunction::scalar(kp);
            }
            "PD" | "PI" | "PID" => {
                let is_pd = kind == "PD";
                let is_pid = kind == "PID";
                assign(
                    &mut c,
                    "design_label",
                    if is_pd {
                        1256
                    } else if is_pid {
                        1514
                    } else {
                        1390
                    },
                )?;
                let mut total = poles.clone();
                if !is_pd {
                    let mut raw = poles_raw.clone();
                    raw.push(C::new(0., 0.));
                    total = groups(raw);
                }
                let zi = if is_pid {
                    optional(p, "pid_pi_zero")?
                        .filter(|v| *v > 0.)
                        .unwrap_or(re / 10.)
                } else {
                    0.
                };
                let zi_info = angle(C::new(-zi, 0.), re, im)?;
                if is_pid {
                    put(&mut c, "zi_val", zi);
                    set(&mut c, "zi_angle", zi_info.clone());
                    assign(&mut c, "angle_eqs", 1523)?;
                    add_contribution(
                        &mut c,
                        "controller_zero",
                        format!("Zero PI (s = -{})", fmt(&json!(zi), 4, false)),
                        C::new(-zi, 0.),
                        1,
                        n(&zi_info["angle_deg"]),
                    );
                }
                let sump = add_angles(
                    &mut c,
                    &total,
                    (re, im),
                    false,
                    !is_pd,
                    &kind,
                    if is_pd {
                        1264
                    } else if is_pid {
                        1543
                    } else {
                        1404
                    },
                )?;
                let sumz = add_angles(
                    &mut c,
                    &zeros,
                    (re, im),
                    true,
                    false,
                    &kind,
                    if is_pd {
                        1280
                    } else if is_pid {
                        1561
                    } else {
                        1423
                    },
                )?;
                put(&mut c, "sum_poles_g", sump);
                put(&mut c, "sum_poles_total", sump);
                put(&mut c, "sum_zeros_g", sumz);
                let raw = sump - sumz + 180. - if is_pid { n(&zi_info["angle_deg"]) } else { 0. };
                let on_locus = is_pd && ((raw + 180.).rem_euclid(360.) - 180.).abs() < 1e-3;
                let (theta, z, td) = if on_locus {
                    (0., None, 0.)
                } else {
                    let theta = math::required_single_angle(raw, "z")?;
                    let info = position(theta, re, im, if is_pid { "z_d" } else { "z" })?;
                    let z = n(&info["pos"]);
                    if z <= 0. {
                        let error = if is_pid {
                            format!(
                                "A condição de fase resultou em z_d = {} <= 0. Ajuste o zero z_i da parte PI ou as especificações.",
                                values::general(z, 4)
                            )
                        } else if is_pd {
                            format!(
                                "A condição de fase resultou em z = {} <= 0. Verifique as especificações transitórias.",
                                values::general(z, 4)
                            )
                        } else {
                            format!(
                                "A condição de fase resultou em z = {} <= 0. Ajuste as especificações transitórias.",
                                values::general(z, 4)
                            )
                        };
                        return Err(error);
                    }
                    set(&mut c, "pos_z_info", info);
                    (theta, Some(z), 1. / z)
                };
                put(&mut c, "theta_z", theta);
                set(
                    &mut c,
                    "z_val",
                    z.map(values::number).unwrap_or(Value::Null),
                );
                put(&mut c, "td_val", td);
                put(&mut c, "ti_val", td);
                if let Some(z) = z {
                    add_contribution(
                        &mut c,
                        "controller_zero",
                        format!(
                            "Zero {} (s = -{})",
                            if is_pid || is_pd { "PD" } else { "PI" },
                            fmt(&json!(z), 4, false)
                        ),
                        C::new(-z, 0.),
                        1,
                        theta,
                    );
                }
                if is_pid {
                    put(&mut c, "zd_val", z.unwrap());
                    append(&mut c, "angle_eqs", 1586, true)?;
                    append(&mut c, "steps", 1590, false)?;
                } else if is_pd {
                    append(
                        &mut c,
                        "angle_eqs",
                        if on_locus { 1297 } else { 1321 },
                        true,
                    )?;
                    assign(&mut c, "step3_notes", if on_locus { 1301 } else { 1326 })?;
                    append(&mut c, "steps", 1331, false)?;
                } else {
                    append(&mut c, "angle_eqs", 1450, true)?;
                    append(&mut c, "steps", 1456, false)?;
                }
                let mut a = vec![];
                if is_pid {
                    a.push(extra_magnitude(dom, zi)?);
                }
                if !is_pd {
                    a.push(extra_magnitude(dom, z.unwrap())?);
                }
                a.extend(group_magnitudes(dom, &zeros)?);
                let b = group_magnitudes(dom, &total)?;
                let pd = C::new(1., 0.) + td * dom;
                let multiplier = if is_pd { pd.norm() } else { 1. };
                let k = set_magnitudes(&mut c, a, b, gain * multiplier);
                if is_pd {
                    let kp = k;
                    put(&mut c, "kp_val", kp);
                    put(&mut c, "kd_val", kp * td);
                    set(&mut c, "pd_factor", root(pd));
                    put(&mut c, "pd_mag", pd.norm());
                    let eq = format!(
                        "\\left|1 + \\tau_d s\\right| = \\left|1 + {}(-{} + {}j)\\right| = \\sqrt{{({})^2 + ({})^2}} = {}",
                        fmt(&json!(td), 4, true),
                        fmt(&json!(re), 4, true),
                        fmt(&json!(im), 4, true),
                        fmt(&json!(pd.re), 4, true),
                        fmt(&json!(pd.im), 4, true),
                        fmt(&json!(pd.norm()), 4, true)
                    );
                    set(&mut c, "pd_factor_eq", eq);
                    assign(&mut c, "mag_eqs", 1354)?;
                    append(&mut c, "mag_eqs", 1355, false)?;
                    append(&mut c, "steps", 1358, false)?;
                    params(&mut c, 1370)?;
                    controller = TransferFunction::new(vec![kp * td, kp], vec![1.])?;
                } else if is_pid {
                    let zd = z.unwrap();
                    put(&mut c, "k_val", k);
                    put(&mut c, "kp_val", k * (zi + zd));
                    put(&mut c, "ki_val", k * zi * zd);
                    put(&mut c, "kd_val", k);
                    assign(&mut c, "mag_eqs", 1616)?;
                    append(&mut c, "mag_eqs", 1617, true)?;
                    append(&mut c, "steps", 1621, false)?;
                    params(&mut c, 1632)?;
                    controller =
                        TransferFunction::new(vec![k, k * (zi + zd), k * zi * zd], vec![1., 0.])?;
                } else {
                    put(&mut c, "kp_val", k);
                    put(&mut c, "ki_val", k * z.unwrap());
                    assign(&mut c, "mag_eqs", 1478)?;
                    append(&mut c, "mag_eqs", 1479, false)?;
                    append(&mut c, "steps", 1482, false)?;
                    params(&mut c, 1494)?;
                    controller = TransferFunction::new(vec![k, k * z.unwrap()], vec![1., 0.])?;
                }
            }
            _ => {
                return Err(format!(
                    "Tipo de controlador não suportado: {}",
                    p["controller_type"].as_str().unwrap_or("lead")
                ));
            }
        }
    }
    if gain_sign < 0. {
        controller = controller.scale(-1.)?;
        let params = c
            .get_mut("controller_params")
            .unwrap()
            .as_object_mut()
            .unwrap();
        for key in ["K", "kp", "ki", "kd"] {
            if let Some(v) = params.get_mut(key) {
                *v = values::number(-n(v));
            }
        }
        append(&mut c, "steps", 1662, false)?;
    }
    finish_universal(p, forward, &plant, feedback.as_ref(), controller, mode, c)
}

fn metrics(tf: &TransferFunction, t: &[f64], threshold: f64) -> Result<Value> {
    let y = simulation::simulate(tf, t, Input::Step)?;
    let steady = tf.dc_gain();
    let sign = steady.signum();
    if !steady.is_finite() || sign == 0. {
        return Ok(
            json!({"overshoot_percent":null,"settling_time":null,"rise_time":null,"peak":"∞","peak_time":"∞","steady_state_value":null}),
        );
    }
    let low = y.iter().position(|v| sign * (*v - 0.1 * steady) >= 0.);
    let high = y.iter().position(|v| sign * (*v - 0.9 * steady) >= 0.);
    let (Some(low), Some(high)) = (low, high) else {
        return Ok(json!({}));
    };
    let outside = y
        .iter()
        .rposition(|v| (*v / steady - 1.).abs() >= threshold);
    let settling = match outside {
        None => Some(t[0]),
        Some(i) if i + 1 < t.len() => Some(t[i + 1]),
        _ => None,
    };
    let maximum = y
        .iter()
        .copied()
        .map(|v| sign * v)
        .fold(f64::NEG_INFINITY, f64::max);
    let overshoot = ((maximum - steady.abs()) / steady.abs() * 100.).max(0.);
    let peak = y
        .iter()
        .enumerate()
        .max_by(|(ia, a), (ib, b)| a.abs().total_cmp(&b.abs()).then(ib.cmp(ia)))
        .unwrap()
        .0;
    Ok(
        json!({"rise_time":t[high]-t[low],"settling_time":settling,"overshoot_percent":overshoot,"peak":y[peak].abs(),"peak_time":t[peak],"steady_state_value":steady}),
    )
}
fn locus(tf: &TransferFunction, gains: &[f64]) -> Result<Vec<(Vec<f64>, Vec<f64>)>> {
    let mut out = vec![(vec![], vec![]); tf.den.len().max(tf.num.len()) - 1];
    for k in gains {
        let mut roots = polynomial::roots(&polynomial::add(&tf.den, &tf.num, *k))?;
        roots.sort_by(|a, b| a.re.total_cmp(&b.re).then(a.im.total_cmp(&b.im)));
        for (i, r) in roots.iter().enumerate() {
            out[i].0.push(r.re);
            out[i].1.push(r.im);
        }
    }
    Ok(out)
}
fn logspace(low: f64, high: f64, count: usize) -> Vec<f64> {
    (0..count)
        .map(|i| 10f64.powf(low + (high - low) * i as f64 / (count - 1) as f64))
        .collect()
}
fn plot_universal(
    plant: &TransferFunction,
    loop_tf: &TransferFunction,
    t: &[f64],
    before: &[f64],
    after: &[f64],
    mode: &str,
    c: &Context,
) -> Result<(String, String)> {
    use crate::time_response::{Chart, Key, Marker, svg_document};
    let width = 1652.;
    let height = 756.;
    let mut left = Chart::new(
        width,
        height,
        (110., 45., 620., 600.),
        "Eixo Real (σ)",
        "Eixo Imaginário (jω)",
        false,
    );
    let p = get(c, "controller_params");
    let re = n(&p["re"]);
    let im = n(&p["im"]);
    let poles = plant.poles()?;
    let zeros = plant.zeros()?;
    let mut interest = vec![-re, 0.];
    interest.extend(poles.iter().chain(&zeros).map(|z| z.re));
    for key in ["z", "p"] {
        if !p[key].is_null() {
            interest.push(-n(&p[key]));
        }
    }
    let min = interest.iter().copied().fold(f64::INFINITY, f64::min);
    let max = interest.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let sx = (max - min).abs().max(re * 1.5).max(2.);
    let sy = (im * 1.6).max(sx * 0.65).max(2.);
    left.bounds = (min - 0.3 * sx, (max + 0.25 * sx).max(0.8), -sy, sy);
    left.axes();
    let mut gains = vec![0.];
    gains.extend(logspace(-3., 2.5, 280));
    const REFERENCE: &str = "#64748B";
    const DANGER: &str = "#B91C1C";
    const DOMINANT: &str = "#F59E0B";
    let mut legend = vec![];
    for (x, y) in locus(plant, &gains)? {
        left.line(&x, &y, "#64748B", 1.1, "2 3");
    }
    legend.push((
        "LGR sem controlador".to_string(),
        REFERENCE,
        Key::Line("2 3"),
    ));
    let mut gains = vec![0.];
    gains.extend((0..80).map(|i| 0.01 + 0.99 * i as f64 / 79.));
    gains.extend(logspace(0.01, 2.2, 180));
    for (x, y) in locus(loop_tf, &gains)? {
        left.line(&x, &y, "black", 2., "");
    }
    legend.push(("LGR compensado Gma(s)".to_string(), "black", Key::Line("")));
    left.line(
        &[left.bounds.0, left.bounds.1],
        &[0., 0.],
        "#64748B",
        0.8,
        "",
    );
    left.line(&[0., 0.], &[-sy, sy], "#64748B", 0.8, "");
    if im > 0. {
        for a in get(c, "angle_contributions").as_array().unwrap() {
            let root = &a["root"];
            let rr = n(&root["re"]);
            let ri = n(&root["im"]);
            left.line(&[rr, -re], &[ri, im], DOMINANT, 1., "5 4");
            left.label(
                0.55 * rr - 0.45 * re,
                0.55 * ri + 0.45 * im,
                &format!("{}°", fmt(&a["angle_deg"], 3, false)),
                0.,
                -3.,
                false,
            );
        }
    }
    for r in &poles {
        left.marker(r.re, r.im, Marker::Cross, REFERENCE, 6.);
    }
    if !poles.is_empty() {
        legend.push((
            "Polos de G(s)".to_string(),
            REFERENCE,
            Key::Marker(Marker::Cross),
        ));
    }
    for r in &zeros {
        left.marker(r.re, r.im, Marker::Ring, REFERENCE, 5.5);
    }
    if !zeros.is_empty() {
        legend.push((
            "Zeros de G(s)".to_string(),
            REFERENCE,
            Key::Marker(Marker::Ring),
        ));
    }
    if !p["z"].is_null() {
        let z = -n(&p["z"]);
        left.marker(z, 0., Marker::Disc, "black", 6.5);
        legend.push((
            format!("Zero Gc (-z = {})", fmt(&json!(z), 4, false)),
            "black",
            Key::Marker(Marker::Disc),
        ));
    }
    if !p["p"].is_null() {
        let pole = -n(&p["p"]);
        left.marker(pole, 0., Marker::FilledPlus, DANGER, 8.);
        legend.push((
            format!("Polo Gc (-p = {})", fmt(&json!(pole), 4, false)),
            DANGER,
            Key::Marker(Marker::FilledPlus),
        ));
    }
    if im > 0. {
        left.marker(-re, im, Marker::Star, DOMINANT, 11.);
        left.marker(-re, -im, Marker::Star, DOMINANT, 11.);
        legend.push((
            format!(
                "Polo dom. (-{} ± {}j)",
                fmt(&p["re"], 3, false),
                fmt(&p["im"], 3, false)
            ),
            DOMINANT,
            Key::Marker(Marker::Star),
        ));
    }
    let legend: Vec<_> = legend
        .iter()
        .map(|(l, c, k)| (l.as_str(), *c, *k))
        .collect();
    left.legend_keys(&legend, true, false);
    let mut right = Chart::new(
        width,
        height,
        (900., 45., 650., 600.),
        "Tempo (s)",
        "Saída y(t)",
        false,
    );
    let reference = t
        .iter()
        .map(|v| match mode {
            "ramp" => *v,
            "parabola" => 0.5 * v * v,
            _ => 1.,
        })
        .collect::<Vec<_>>();
    right.autobounds(t, &[before, after, &reference]);
    right.axes();
    right.line(t, &reference, "black", 1.2, "2 3");
    right.line(t, before, "#64748B", 1.6, "6 4");
    right.line(t, after, "black", 2.2, "");
    right.legend_keys(
        &[
            ("Referência r(t)", "black", Key::Line("2 3")),
            ("Sem controlador", "#64748B", Key::Line("6 4")),
            ("Com controlador Gc(s)", "black", Key::Line("")),
        ],
        true,
        true,
    );
    let title = match mode {
        "ramp" => "Resposta à Rampa Unitária",
        "parabola" => "Resposta à Parábola Unitária",
        _ => "Resposta ao Degrau Unitário",
    };
    let body = format!(
        "{}{}<text x=\"420\" y=\"24\" text-anchor=\"middle\">Lugar Geométrico das Raízes e Ângulos θ</text><text x=\"1225\" y=\"24\" text-anchor=\"middle\">{title}</text>",
        left.svg, right.svg
    );
    let svg = svg_document(width, height, &body);
    Ok((crate::presentation::rasterize(&svg)?, svg))
}
fn finish_universal(
    p: &Value,
    forward: &TransferFunction,
    plant: &TransferFunction,
    feedback: Option<&TransferFunction>,
    controller: TransferFunction,
    mode: String,
    mut c: Context,
) -> Result<Value> {
    let mut loop_tf = controller.multiply(plant)?;
    for v in loop_tf.num.iter_mut().chain(&mut loop_tf.den) {
        if v.abs() < 1e-10 {
            *v = 0.;
        }
    }
    let simulation_loop = if !loop_tf.is_proper() {
        loop_tf.multiply(&TransferFunction::new(vec![1.], vec![0.001, 1.])?)?
    } else {
        loop_tf.clone()
    };
    let (uncomp_cl, comp_cl) = if let Some(h) = feedback {
        let mut f = controller.multiply(forward)?;
        if !f.is_proper() {
            f = f.multiply(&TransferFunction::new(vec![1.], vec![0.001, 1.])?)?;
        }
        (forward.feedback(h, -1.)?, f.feedback(h, -1.)?)
    } else {
        (
            plant.feedback(&TransferFunction::scalar(1.), -1.)?,
            simulation_loop.feedback(&TransferFunction::scalar(1.), -1.)?,
        )
    };
    let comp_poles = comp_cl.poles()?;
    let stable = comp_poles.iter().all(|r| r.re < -1e-7);
    if !stable {
        c.get_mut("design_warnings")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .insert(0, json!("Malha fechada instável com estes parâmetros."));
    }
    let warnings = get(&c, "design_warnings");
    let warning = if warnings.as_array().unwrap().is_empty() {
        Value::Null
    } else {
        json!(
            warnings
                .as_array()
                .unwrap()
                .iter()
                .map(text)
                .collect::<Vec<_>>()
                .join(" ")
        )
    };
    let err = math::steady_state_error(&loop_tf.num, &loop_tf.den, &mode);
    let old = get(&c, "uncomp")["steady_state_error_raw"].clone();
    {
        let cp = c.get_mut("controller_params").unwrap();
        cp["e_atual"] = old;
        cp["e_novo"] = err["value"].clone();
    }
    let cp = get(&c, "controller_params");
    let kind = text(&get(&c, "kind"));
    let gc = controller_latex(&kind, &cp)?;
    let plant_clean = text(&get(&c, "plant_latex_factored"))
        .replace("G(s)H(s) = ", "")
        .replace("G(s) = ", "");
    let loop_label = if feedback.is_some() {
        "G_c(s)G(s)H(s)"
    } else {
        "G_c(s)G(s)"
    };
    let factored = format!(
        "G_{{ma}}(s) = {loop_label} = {} \\cdot \\left({plant_clean}\\right)",
        gc.replace("G_c(s) = ", "")
    );
    let expanded =
        tf_latex(&rounded_tf(&loop_tf, 5, 0.), true, false).replace("G(s) = ", "G_{ma}(s) = ");
    let tc = tf_latex(&rounded_tf(&comp_cl, 5, 0.), true, false).replace("G(s) = ", "T(s) = ");
    for (key, val) in [
        ("gc_latex", gc.clone()),
        ("gma_factored_latex", factored.clone()),
        ("gma_expanded_latex", expanded.clone()),
    ] {
        set(&mut c, key, val);
    }
    set(&mut c, "err_after", err.clone());
    set(&mut c, "compensated_stable", stable);
    append(&mut c, "steps", 1722, false)?;
    let end = optional(p, "final_time")?;
    let t = simulation::time_vector(&comp_cl, end, crate::time_response::points(p)?)?;
    let input = match mode.as_str() {
        "ramp" => Input::Ramp,
        "parabola" => Input::Parabolic,
        _ => Input::Step,
    };
    let before = simulation::simulate(&uncomp_cl, &t, input)?;
    let after = simulation::simulate(&comp_cl, &t, input)?;
    let threshold = if cp["criterion_pct"] == 5 { 0.05 } else { 0.02 };
    let mb = metrics(&uncomp_cl, &t, threshold)?;
    let ma = metrics(&comp_cl, &t, threshold)?;
    let (image, svg) = plot_universal(plant, &loop_tf, &t, &before, &after, &mode, &c)?;
    put(&mut c, "dom_re", n(&cp["re"]));
    put(&mut c, "dom_im", n(&cp["im"]));
    set(
        &mut c,
        "dom_pole_str",
        format!(
            "-{} ± {}j",
            fmt(&cp["re"], 4, false),
            fmt(&cp["im"], 4, false)
        ),
    );
    assign(&mut c, "metric_rows", 1913)?;
    match kind.as_str() {
        "LEAD" | "LAG" => {
            append(&mut c, "metric_rows", 1920, true)?;
            if !cp["z_over_p"].is_null() {
                append(&mut c, "metric_rows", 1926, false)?;
            }
            if !cp["theta_p"].is_null() {
                append(&mut c, "metric_rows", 1928, false)?;
            }
        }
        "P" => append(&mut c, "metric_rows", 1930, false)?,
        "PD" => append(&mut c, "metric_rows", 1932, true)?,
        "PID" => append(&mut c, "metric_rows", 1940, true)?,
        "PI" => append(&mut c, "metric_rows", 1949, true)?,
        _ => {}
    }
    append(&mut c, "metric_rows", 1956, true)?;
    set(&mut c, "warning", warning.clone());
    if !warning.is_null() {
        append(&mut c, "metric_rows", 1967, false)?;
    }
    let details = json!({"design_type":kind.to_lowercase(),"controller_type":kind,"design_label":get(&c,"design_label"),"input_type":mode,"settling_criterion":cp["criterion_pct"],"stable":stable,"warning":warning,"controller_parameters":cp,"uncompensated":get(&c,"uncomp"),"angle_contributions":get(&c,"angle_contributions"),"steps":get(&c,"steps"),"gc_latex":gc,"gma_factored_latex":factored,"gma_expanded_latex":expanded,"tc_latex":tc,"plant_latex_factored":get(&c,"plant_latex_factored"),"plant_latex_expanded":get(&c,"plant_latex_expanded"),"forward_latex_factored":get(&c,"forward_latex_factored"),"feedback_latex":get(&c,"feedback_latex"),"warnings":warnings,"before":mb,"after":ma,"closed_loop_poles_before":values::complex_list(&uncomp_cl.poles()?),"closed_loop_poles_after":values::complex_list(&comp_poles)});
    let mut latex = vec![
        json!({"label":"Controlador Gc(s)","value":gc}),
        json!({"label":format!("Malha Aberta Compensada Gma(s) = {}",if feedback.is_some(){"Gc(s)G(s)H(s)"}else{"Gc(s)G(s)"}),"value":factored}),
        json!({"label":"Malha Aberta Expandida","value":expanded}),
        json!({"label":"Malha Fechada Compensada T(s)","value":tc}),
        json!({"label":"Planta Original G(s)","value":get(&c,"forward_latex_factored")}),
    ];
    if feedback.is_some() {
        latex.push(json!({"label":"Realimentação H(s)","value":get(&c,"feedback_latex")}));
        latex.push(json!({"label":"Malha Aberta sem Controlador G(s)H(s)","value":get(&c,"plant_latex_factored")}));
    }
    Ok(
        json!({"success":true,"module":"controller_design","title":"Projeto de Controladores","description":get(&c,"design_label"),"image":image,"svg":svg,"latex":latex,"metrics":get(&c,"metric_rows"),"details":details}),
    )
}

fn margin_details(tf: &TransferFunction) -> Result<Value> {
    let m = crate::frequency_response::margins(tf)?;
    Ok(
        json!({"gain_margin_db":values::number(if m.gain.is_infinite(){f64::INFINITY}else{20.*m.gain.max(f64::MIN_POSITIVE).log10()}),"phase_margin_deg":values::number(m.phase),"phase_crossover_frequency":values::number(m.phase_cross),"gain_crossover_frequency":values::number(m.gain_cross)}),
    )
}
fn legacy(p: &Value, plant: &TransferFunction) -> Result<Value> {
    let design = p["design_type"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("pid")
        .to_lowercase();
    let mut kind = p["controller_type"]
        .as_str()
        .unwrap_or("lead")
        .to_uppercase();
    if !["P", "PI", "PD", "PID"].contains(&kind.as_str()) {
        kind = "PID".into();
    }
    let method = p["method"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("zn_critical")
        .to_lowercase();
    let (controller, parameters, label) = if design == "pid" {
        let mut payload = p.clone();
        payload["controller_type"] = json!(kind);
        let mut auto = None;
        if ["zn_critical", "zn_critical_auto"].contains(&method.as_str()) {
            let m = crate::frequency_response::margins(plant)?;
            if !(m.gain.is_finite()
                && m.gain > 0.
                && m.phase_cross.is_finite()
                && m.phase_cross > 0.)
            {
                return Err("A planta não possui frequência de oscilação crítica (-180°). Utilize a opção Manual ou sintonia por curva de reação.".into());
            }
            payload["method"] = json!("zn_critical_manual");
            payload["critical_gain"] = json!(m.gain);
            payload["critical_period"] = json!(2. * PI / m.phase_cross);
            auto = Some(m.phase_cross);
        } else {
            payload["method"] = json!(method);
        }
        let mut params = math::pid_parameters(&payload)?;
        if let Some(w) = auto {
            params["method_label"] = json!("Ziegler-Nichols — oscilação crítica (automático)");
            params["wcr"] = json!(w);
        }
        let controller = math::pid_transfer(&params, &kind)?;
        let label = format!("{kind} por {}", params["method_label"].as_str().unwrap());
        (controller, params, label)
    } else if design == "lead_lag" {
        let zero = positive(
            optional(p, "compensator_zero")?.unwrap_or(1.),
            "A frequência do zero",
        )?;
        let pole = positive(
            optional(p, "compensator_pole")?.unwrap_or(5.),
            "A frequência do polo",
        )?;
        let gain = positive(
            optional(p, "compensator_gain")?.unwrap_or(1.),
            "O ganho do compensador",
        )?;
        let compensator = p["compensator_type"]
            .as_str()
            .unwrap_or("lead")
            .to_lowercase();
        if compensator == "lead" && pole <= zero {
            return Err("Para avanço de fase, informe polo maior que o zero.".into());
        }
        if compensator == "lag" && pole >= zero {
            return Err("Para atraso de fase, informe polo menor que o zero.".into());
        }
        let label = format!(
            "Compensador por {} de fase — referência {}",
            if compensator == "lead" {
                "avanço"
            } else {
                "atraso"
            },
            if p["design_domain"]
                .as_str()
                .unwrap_or("frequency")
                .eq_ignore_ascii_case("frequency")
            {
                "Bode"
            } else {
                "LGR"
            }
        );
        (
            TransferFunction::new(vec![gain, gain * zero], vec![1., pole])?,
            Value::Null,
            label,
        )
    } else {
        return Err("Escolha sintonia PID ou compensador avanço/atraso.".into());
    };
    let before = plant.feedback(&TransferFunction::scalar(1.), -1.)?;
    let loop_tf = controller.multiply(plant)?;
    let after = loop_tf.feedback(&TransferFunction::scalar(1.), -1.)?;
    let poles = after.poles()?;
    let stable = poles.iter().all(|r| r.re < -1e-9);
    let warning = if stable {
        Value::Null
    } else {
        json!("Malha fechada instável com estes parâmetros")
    };
    let t = simulation::time_vector(
        &after,
        optional(p, "final_time")?,
        crate::time_response::points(p)?,
    )?;
    let yb = simulation::simulate(&before, &t, Input::Step)?;
    let ya = simulation::simulate(&after, &t, Input::Step)?;
    let mb = metrics(&before, &t, 0.02)?;
    let ma = metrics(&after, &t, 0.02)?;
    let marg_before = margin_details(plant)?;
    let marg_after = margin_details(&loop_tf)?;
    let (image, svg) = plot_legacy(plant, &loop_tf, &t, &yb, &ya)?;
    let mut c = Context::new();
    set(&mut c, "compensated_stable", stable);
    set(&mut c, "warning", warning.clone());
    set(&mut c, "parameters", parameters.clone());
    set(&mut c, "design_label", label.clone());
    assign(&mut c, "metric_rows", 2353)?;
    if !warning.is_null() {
        append(&mut c, "metric_rows", 2357, false)?;
    }
    if !parameters.is_null() {
        append(&mut c, "metric_rows", 2360, true)?;
    }
    let mut rows = get(&c, "metric_rows").as_array().unwrap().clone();
    for (key, caption, unit) in [
        ("overshoot_percent", "Sobressinal (%OS)", "%"),
        ("settling_time", "Tempo de acomodação (ts)", "s"),
    ] {
        rows.push(json!({"label":caption,"value":if ma.get(key).is_some_and(|v|!v.is_null()){text(&ma[key])}else{"Não definido".into()},"unit":unit}));
    }
    for (key, caption, unit) in [
        ("gain_margin_db", "Margem de ganho (MG)", "dB"),
        ("phase_margin_deg", "Margem de fase (MF)", "°"),
    ] {
        rows.push(json!({"label":caption,"value":text(&marg_after[key]),"unit":unit}));
    }
    rows.push(json!({"label":"Projeto","value":label}));
    let float_input = p["plant_expr"]
        .as_str()
        .or_else(|| p["expr"].as_str())
        .unwrap_or("")
        .contains('.');
    let plant_exp = tf_latex(plant, float_input, false);
    let plant_fact = tf_latex(plant, float_input, true);
    let controller_latex =
        tf_latex(&rounded_tf(&controller, 4, 1e-9), true, false).replace("G(s) = ", "");
    Ok(
        json!({"success":true,"module":"controller_design","title":"Projeto de Controladores","description":"Comparação da resposta ao degrau e do LGR antes e depois da compensação.","image":image,"svg":svg,"latex":[{"label":"Planta","value":plant_fact.replace("G(s) = ","")},{"label":"Controlador","value":format!("C(s)={controller_latex}")},{"label":"Malha fechada compensada","value":"T_c(s)=\\frac{C(s)G(s)}{1+C(s)G(s)}"}],"metrics":rows,"details":{"design_type":design,"design_label":label,"stable":stable,"warning":warning,"controller_parameters":parameters,"kcr":parameters["kcr"],"pcr":parameters["pcr"],"wcr":parameters["wcr"],"before":mb,"after":ma,"frequency_margins_before":marg_before,"frequency_margins_after":marg_after,"closed_loop_poles_before":values::complex_list(&before.poles()?),"closed_loop_poles_after":values::complex_list(&poles),"plant_latex_expanded":plant_exp}}),
    )
}
fn plot_legacy(
    plant: &TransferFunction,
    loop_tf: &TransferFunction,
    t: &[f64],
    before: &[f64],
    after: &[f64],
) -> Result<(String, String)> {
    use crate::time_response::{Chart, svg_document};
    let (w, h) = (1568., 1092.);
    let mut step = Chart::new(
        w,
        h,
        (90., 40., 640., 420.),
        "Tempo (s)",
        "Saída y(t)",
        false,
    );
    step.autobounds(t, &[before, after]);
    step.axes();
    step.line(t, before, "#64748B", 1.8, "6 4");
    step.line(t, after, "black", 2.2, "");
    step.legend(
        &[("Antes", "#64748B", true), ("Depois", "black", false)],
        true,
    );
    let mut gains = vec![0.];
    gains.extend(logspace(-3., 3., 260));
    let lb = locus(plant, &gains)?;
    let la = locus(loop_tf, &gains)?;
    let xs = lb
        .iter()
        .chain(&la)
        .flat_map(|v| v.0.iter().copied())
        .collect::<Vec<_>>();
    let ys = lb
        .iter()
        .chain(&la)
        .flat_map(|v| v.1.iter().copied())
        .collect::<Vec<_>>();
    let mut lgr = Chart::new(
        w,
        h,
        (870., 40., 640., 420.),
        "Eixo real",
        "Eixo imaginário",
        false,
    );
    let min = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let max = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    lgr.autobounds(&[min, max], &[&ys]);
    lgr.axes();
    for (x, y) in lb {
        lgr.line(&x, &y, "#64748B", 1.2, "6 4");
    }
    for (x, y) in la {
        lgr.line(&x, &y, "black", 2., "");
    }
    lgr.legend(
        &[("Antes", "#64748B", true), ("Depois", "black", false)],
        true,
    );
    let omega = logspace(-3., 3., 700);
    let response = |tf: &TransferFunction| {
        let vals = omega
            .iter()
            .map(|w| tf.evaluate(C::new(0., *w)))
            .collect::<Vec<_>>();
        let magnitude = vals
            .iter()
            .map(|v| 20. * v.norm().max(f64::MIN_POSITIVE).log10())
            .collect::<Vec<_>>();
        let mut phase = vec![];
        let mut last = 0.;
        for (i, v) in vals.iter().enumerate() {
            let mut a = v.arg();
            if i > 0 {
                while a - last > PI {
                    a -= 2. * PI;
                }
                while a - last < -PI {
                    a += 2. * PI;
                }
            }
            last = a;
            phase.push(a * 180. / PI);
        }
        (magnitude, phase)
    };
    let (mb, pb) = response(plant);
    let (ma, pa) = response(loop_tf);
    let mut mag = Chart::new(
        w,
        h,
        (90., 600., 640., 420.),
        "Frequência (rad/s)",
        "Magnitude (dB)",
        true,
    );
    mag.autobounds(&omega, &[&mb, &ma]);
    mag.axes();
    mag.line(&omega, &mb, "#64748B", 1.6, "6 4");
    mag.line(&omega, &ma, "black", 2., "");
    mag.legend(
        &[("Antes", "#64748B", true), ("Depois", "black", false)],
        true,
    );
    let mut phase = Chart::new(
        w,
        h,
        (870., 600., 640., 420.),
        "Frequência (rad/s)",
        "Fase (graus)",
        true,
    );
    phase.autobounds(&omega, &[&pb, &pa]);
    phase.axes();
    phase.line(&omega, &pb, "#64748B", 1.6, "6 4");
    phase.line(&omega, &pa, "black", 2., "");
    phase.legend(
        &[("Antes", "#64748B", true), ("Depois", "black", false)],
        true,
    );
    let svg = svg_document(
        w,
        h,
        &format!("{}{}{}{}", step.svg, lgr.svg, mag.svg, phase.svg),
    );
    Ok((crate::presentation::rasterize(&svg)?, svg))
}

pub fn presets() -> Value {
    json!({"success":true,"presets":serde_json::from_str::<Value>(PRESETS).expect("validated controller presets")})
}

const PRESETS: &str = r####"[{"id":"Avanço de Fase — G(s) = 5 / (s(s+1))","title":"Avanço de Fase — G(s) = 5 / (s(s+1))","plant_expr":"5 / (s * (s + 1))","controller_type":"lead","input_type":"step","settling_criterion":"2","ts":2.0,"mp":16.0,"xi":0.5},{"id":"Atraso de Fase — G(s) = 1 / ((s+0.5)(s+2))","title":"Atraso de Fase — G(s) = 1 / ((s+0.5)(s+2))","plant_expr":"1 / ((s + 0.5) * (s + 2))","controller_type":"lag","input_type":"step","settling_criterion":"2","error_reduction_factor":10.0,"lag_pole":0.01,"lag_gain":1.0},{"id":"Proporcional (P) Ex. 1 — G(s) = 1 / (s(s+1)(s+2))","title":"Proporcional (P) Ex. 1 — G(s) = 1 / (s(s+1)(s+2))","plant_expr":"1 / (s * (s + 1) * (s + 2))","controller_type":"P","input_type":"step","settling_criterion":"2","ts":10.0,"mp":10.0,"xi":0.591},{"id":"Proporcional (P) Ex. 2 — G(s) = 1 / ((s+1)(s+5))","title":"Proporcional (P) Ex. 2 — G(s) = 1 / ((s+1)(s+5))","plant_expr":"1 / ((s + 1) * (s + 5))","controller_type":"P","input_type":"step","settling_criterion":"2","ts":2.0,"mp":10.0,"xi":0.591},{"id":"Proporcional-Derivativo (PD) Ex. 1 — G(s) = 40 / (s(s+2))","title":"Proporcional-Derivativo (PD) Ex. 1 — G(s) = 40 / (s(s+2))","plant_expr":"40 / (s * (s + 2))","controller_type":"PD","input_type":"step","settling_criterion":"2","ts":1.0,"mp":25.0,"xi":0.4},{"id":"Proporcional-Derivativo (PD) Ex. 2 — G(s) = 1 / (s(s+4)(s+6))","title":"Proporcional-Derivativo (PD) Ex. 2 — G(s) = 1 / (s(s+4)(s+6))","plant_expr":"1 / (s * (s + 4) * (s + 6))","controller_type":"PD","input_type":"step","settling_criterion":"2","xi":0.456,"wn":5.0},{"id":"Proporcional-Integral (PI) Ex. 1 — G(s) = 10 / ((s+2)(s+5))","title":"Proporcional-Integral (PI) Ex. 1 — G(s) = 10 / ((s+2)(s+5))","plant_expr":"10 / ((s + 2) * (s + 5))","controller_type":"PI","input_type":"step","settling_criterion":"5","ts":1.5,"mp":0.63,"xi":0.857},{"id":"Proporcional-Integral (PI) Ex. 2 — G(s) = 1 / (s(s+4))","title":"Proporcional-Integral (PI) Ex. 2 — G(s) = 1 / (s(s+4))","plant_expr":"1 / (s * (s + 4))","controller_type":"PI","input_type":"ramp","settling_criterion":"5","ts":2.0,"mp":10.0,"xi":0.6}]"####;
