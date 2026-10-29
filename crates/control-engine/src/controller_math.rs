//! Shared analytic controller design equations, ported from controller_design.py.
use crate::{
    Result, input, polynomial,
    transfer::TransferFunction,
    values::{general, number},
};
use num_complex::Complex64;
use serde_json::{Value, json};
use std::f64::consts::PI;

pub fn optional(v: &Value) -> Result<Option<f64>> {
    match v {
        Value::Null => Ok(None),
        Value::String(s) if s.trim().is_empty() => Ok(None),
        Value::String(s) => s
            .trim()
            .replace(',', ".")
            .parse::<f64>()
            .map(|v| v.is_finite().then_some(v))
            .map_err(|_| "Valor numérico inválido.".into()),
        _ => Ok(v.as_f64().filter(|x| x.is_finite())),
    }
}
fn positive(v: f64, label: &str) -> Result<f64> {
    if v.is_finite() && v > 0. {
        Ok(v)
    } else {
        Err(format!("{label} deve ser maior que zero."))
    }
}
pub fn tex(v: f64, digits: usize) -> String {
    if !v.is_finite() {
        return "\\infty".into();
    }
    let v = if v.abs() < 1e-12 { 0. } else { v };
    if (v - v.round()).abs() < 1e-9 {
        format!("{:.0}", v)
    } else {
        general(v, digits).replace('.', "{,}")
    }
}
pub fn mp_from_xi(xi: f64) -> f64 {
    if xi <= 0. {
        100.
    } else if xi >= 1. {
        0.
    } else {
        (-xi * PI / (1. - xi * xi).sqrt()).exp() * 100.
    }
}
pub fn xi_from_mp(mp: f64) -> Result<f64> {
    if mp <= 0. {
        Ok(1.)
    } else if mp >= 100. {
        Err("O sobressinal máximo Mp(%) deve estar entre 0% e 100%.".into())
    } else {
        let l = (mp / 100.).ln();
        Ok(-l / (PI * PI + l * l).sqrt())
    }
}

pub fn desired_pole(v: &Value) -> Result<Option<Complex64>> {
    let Some(source) = v.as_str() else {
        return Ok(None);
    };
    if source.trim().is_empty() {
        return Ok(None);
    }
    let mut s = source
        .replace("s =", "")
        .replace("s=", "")
        .replace("+-", "+")
        .replace('±', "+")
        .replace('−', "-")
        .replace('√', "sqrt")
        .replace('i', "j")
        .replace(' ', "")
        .replace(',', ".");
    if s.contains('^') || s.contains("**") {
        return Err("Polo desejado inválido: use sqrt(n) em vez de potências.".into());
    }
    while let Some(start) = s.find("sqrt") {
        let mut end = start + 4;
        let parenthesis = s.as_bytes().get(end) == Some(&b'(');
        if parenthesis {
            end += 1;
        }
        let first = end;
        while end < s.len() && (s.as_bytes()[end].is_ascii_digit() || s.as_bytes()[end] == b'.') {
            end += 1;
        }
        let value = s[first..end]
            .parse::<f64>()
            .map_err(|_| "Polo desejado inválido.")?;
        if parenthesis {
            if s.as_bytes().get(end) != Some(&b')') {
                return Err("Polo desejado inválido.".into());
            }
            end += 1;
        }
        s.replace_range(start..end, &value.sqrt().to_string());
    }
    if !s.contains('j') {
        return Err(
            "Polo desejado inválido. Use o formato -Re + Im·j, ex.: -1+1j ou -1+sqrt(3)j.".into(),
        );
    }
    let pole = input::complex(&s)?;
    let pole = Complex64::new(pole.re, pole.im.abs());
    if pole.re >= 0. || pole.im <= 0. {
        return Err("O polo dominante desejado deve ser complexo e estar no semiplano esquerdo (Re < 0, Im ≠ 0).".into());
    }
    Ok(Some(pole))
}

pub fn resolve_transient(p: &Value) -> Result<Value> {
    let crit = p["settling_criterion"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| p["settling_criterion"].to_string())
        .replace('%', "");
    let factor = if crit.trim() == "5" { 3. } else { 4. };
    let pct = if factor == 3. { 5 } else { 2 };
    let ts = optional(&p["ts"])?;
    let mp = optional(&p["mp"])?;
    let xi = optional(&p["xi"])?;
    let wn = optional(&p["wn"])?;
    if let Some(pole) = desired_pole(&p["desired_pole"])? {
        let wn = pole.norm();
        let xi = -pole.re / wn;
        let mp = mp_from_xi(xi);
        return Ok(
            json!({"criterion_pct":pct,"settling_factor":factor,"ts":factor/-pole.re,"mp":mp,"mp_calculated":mp,"xi":xi,"wn":wn,"re":-pole.re,"im":pole.im,"dominant_pole":{"re":pole.re,"im":pole.im},"damping_source":"pole","freq_source":"pole"}),
        );
    }
    let (xi, mp, damping_source) =
        if p["primary_damping_spec"] == "mp" && mp.is_some_and(|v| v > 0. && v < 100.) {
            let mp = mp.unwrap();
            (xi_from_mp(mp)?, mp, "mp")
        } else if xi.is_some_and(|v| v > 0.) {
            let xi = xi.unwrap();
            (
                xi,
                mp.filter(|v| *v >= 0.).unwrap_or_else(|| mp_from_xi(xi)),
                "xi",
            )
        } else if mp.is_some_and(|v| v > 0. && v < 100.) {
            let mp = mp.unwrap();
            (xi_from_mp(mp)?, mp, "mp")
        } else if ts.is_some_and(|v| v > 0.) && wn.is_some_and(|v| v > 0.) {
            let xi = factor / ts.unwrap() / wn.unwrap();
            if xi <= 0. || xi >= 1. {
                return Err(
                    "A combinação de ts e ωn resulta em ξ fora da faixa subamortecida (0 < ξ < 1)."
                        .into(),
                );
            }
            (xi, mp_from_xi(xi), "ts_wn")
        } else {
            (0.5, mp_from_xi(0.5), "default")
        };
    if xi <= 0. || xi >= 1. {
        return Err("O coeficiente de amortecimento ξ deve estar entre 0 e 1 para polos dominantes complexos.".into());
    }
    let (ts, wn, re, freq_source) = if p["primary_freq_spec"] == "wn" && wn.is_some_and(|v| v > 0.)
    {
        let w = wn.unwrap();
        (factor / (xi * w), w, xi * w, "wn")
    } else if ts.is_some_and(|v| v > 0.) && (wn.is_none() || p["primary_freq_spec"] != "wn") {
        let t = ts.unwrap();
        (t, factor / t / xi, factor / t, "ts")
    } else if wn.is_some_and(|v| v > 0.) {
        let w = wn.unwrap();
        (factor / (xi * w), w, xi * w, "wn")
    } else {
        (2., factor / 2. / xi, factor / 2., "default")
    };
    let im = wn * (1. - xi * xi).sqrt();
    Ok(
        json!({"criterion_pct":pct,"settling_factor":factor,"ts":ts,"mp":mp,"mp_calculated":mp_from_xi(xi),"xi":xi,"wn":wn,"re":re,"im":im,"dominant_pole":{"re":-re,"im":im},"damping_source":damping_source,"freq_source":freq_source}),
    )
}

pub fn lgr_angle(pos: f64, re: f64, im: f64) -> f64 {
    if (pos - re).abs() <= 1e-7 {
        90.
    } else if pos < re {
        180. - (im / (re - pos)).atan().to_degrees()
    } else {
        (im / (pos - re)).atan().to_degrees()
    }
}
pub fn position_from_angle(theta: f64, re: f64, im: f64) -> f64 {
    if (theta - 90.).abs() <= 1e-6 {
        re
    } else {
        re + im / theta.to_radians().tan()
    }
}
pub fn required_single_angle(raw: f64, symbol: &str) -> Result<f64> {
    let angle = raw.rem_euclid(360.);
    if angle > 180. + 1e-6 {
        let element = if symbol == "z" { "zero" } else { "polo" };
        return Err(format!(
            "A condição de fase exige que o {element} do controlador contribua com {}°, o que um único elemento real no semiplano esquerdo não consegue. {}",
            general(-(360. - angle), 4),
            if symbol == "z" {
                "O ponto desejado fica à direita do LGR: use PI ou Atraso de Fase."
            } else {
                "Escolha outra posição para o zero z ou outro tipo de controlador."
            }
        ));
    }
    Ok(angle.clamp(1e-4, 180.))
}

pub fn steady_state_error(num: &[f64], den: &[f64], mode: &str) -> Value {
    let (mode, order, r) = match mode.trim().to_lowercase().as_str() {
        "parabola" | "parábola" | "parabolic" | "parabolica" | "parabólica" | "1/s^3" | "1/s3" => {
            ("parabola", 2, "\\frac{1}{s^3}")
        }
        "ramp" | "rampa" | "1/s^2" | "1/s2" => ("ramp", 1, "\\frac{1}{s^2}"),
        _ => ("step", 0, "\\frac{1}{s}"),
    };
    let sum = polynomial::add(den, num, 1.);
    let nz = den.iter().rev().take_while(|&&v| v == 0.).count();
    let dz = sum.iter().rev().take_while(|&&v| v == 0.).count() + order;
    let value = if den.iter().all(|&v| v == 0.) {
        0.
    } else if sum.iter().all(|&v| v == 0.) || nz < dz {
        f64::INFINITY
    } else if nz > dz {
        0.
    } else {
        den[den.len() - 1 - nz] / sum[sum.len() - 1 - (dz - order)]
    };
    let value = if value.abs() < 1e-12 { 0. } else { value };
    let latex = tex(value, 4);
    json!({"value":number(value),"json_value":number(value),"latex_value":latex,"formula_latex":format!("e(\\infty) = \\lim_{{s \\to 0}} s \\left(\\frac{{1}}{{1 + G_{{ma}}(s)}}\\right) {r} = {latex}"),"input_type":mode})
}

pub fn pid_parameters(p: &Value) -> Result<Value> {
    let method = p["method"].as_str().unwrap_or("zn_critical").to_lowercase();
    let kind = p["controller_type"]
        .as_str()
        .unwrap_or("PID")
        .to_uppercase();
    if !["P", "PI", "PD", "PID"].contains(&kind.as_str()) {
        return Err("O controlador deve ser P, PI, PD ou PID.".into());
    }
    let value = |key: &str, default: f64, label: &str| {
        positive(optional(&p[key])?.unwrap_or(default), label)
    };
    let mut ti = None;
    let mut td = 0.;
    let mut ku = None;
    let mut pu = None;
    let mut wcr = None;
    let kp;
    let label;
    match method.as_str(){
        "zn_critical_manual"=>{
            let k=value("critical_gain",6.,"O ganho crítico Kcr")?;let t=value("critical_period",2.,"O período crítico Pcr")?;ku=Some(k);pu=Some(t);wcr=Some(2.*PI/t);
            match kind.as_str(){"P"=>kp=0.5*k,"PI"=>{kp=0.45*k;ti=Some(t/1.2);},"PD"=>{kp=0.8*k;td=t/8.;},_=>{kp=0.6*k;ti=Some(t/2.);td=t/8.;}}
            label="Ziegler-Nichols — oscilação crítica (manual)".to_owned();
        }
        "zn_reaction"|"cohen_coon"|"chr"=>{
            let gain=value("process_gain",1.,"O ganho do processo")?;let lag=value("delay",1.,"O atraso L")?;let tau=value("time_constant",4.,"A constante de tempo T")?;let base=tau/(gain*lag);let ratio=lag/tau;
            if method=="zn_reaction"{match kind.as_str(){"P"=>kp=base,"PI"=>{kp=0.9*base;ti=Some(3.33*lag);},"PD"=>{kp=1.2*base;td=0.5*lag;},_=>{kp=1.2*base;ti=Some(2.*lag);td=0.5*lag;}}label="Ziegler-Nichols — curva de reação".into();}
            else if method=="cohen_coon"{match kind.as_str(){"P"=>kp=base*(1.+ratio/3.),"PI"=>{kp=base*(0.9+ratio/12.);ti=Some(lag*(30.+3.*ratio)/(9.+20.*ratio));},"PD"=>{kp=base*(1.25+ratio/6.);td=lag*(6.-2.*ratio)/(22.+3.*ratio);},_=>{kp=base*(4./3.+ratio/4.);ti=Some(lag*(32.+6.*ratio)/(13.+8.*ratio));td=lag*4./(11.+2.*ratio);}}label="Cohen-Coon".into();}
            else{let response=p["chr_response"].as_str().unwrap_or("0");let(f,t,d)=match(kind.as_str(),response=="20"){("P",true)=>(0.7,None,0.),("PI",true)=>(0.6,Some(tau),0.),("PD",true)=>(0.7,None,0.45*lag),("PID",true)=>(0.95,Some(1.4*tau),0.47*lag),("P",false)=>(0.3,None,0.),("PI",false)=>(0.35,Some(1.2*tau),0.),("PD",false)=>(0.3,None,0.5*lag),_=>(0.6,Some(tau),0.5*lag)};kp=f*base;ti=t;td=d;label=format!("CHR — {response}% de sobressinal");}
        }
        "zn_critical"|"zn_critical_auto"=>return Err("A sintonia crítica automática requer as margens da planta; utilize critical_parameters.".into()),
        _=>return Err("Método de sintonia desconhecido.".into()),
    }
    Ok(
        json!({"kp":kp,"ki":ti.map_or(0.,|t|kp/t),"kd":kp*td,"ti":ti,"td":td,"kcr":ku,"pcr":pu,"wcr":wcr,"method_label":label}),
    )
}

pub fn pid_transfer(p: &Value, kind: &str) -> Result<TransferFunction> {
    let kp = p["kp"].as_f64().ok_or("Kp ausente.")?;
    let ki = p["ki"].as_f64().unwrap_or(0.);
    let td = p["td"].as_f64().unwrap_or(0.);
    match kind {
        "P" => TransferFunction::new(vec![kp], vec![1.]),
        "PI" => TransferFunction::new(vec![kp, ki], vec![1., 0.]),
        "PD" if td > 0. => {
            TransferFunction::new(vec![kp * (td / 10. + td), kp], vec![td / 10., 1.])
        }
        "PD" => TransferFunction::new(vec![kp], vec![1.]),
        _ if td > 0. => TransferFunction::new(
            vec![kp * (td / 10. + td), kp + ki * td / 10., ki],
            vec![td / 10., 1., 0.],
        ),
        _ => TransferFunction::new(vec![kp, ki], vec![1., 0.]),
    }
}
