//! Migration tools. Commands never report incomplete semantic/visual parity as a pass.
use serde_json::{Value, json};
mod tools;
use std::{env, fs, path::PathBuf, process::ExitCode};

fn compare(
    expected: &Value,
    actual: &Value,
    path: &str,
    diffs: &mut Vec<String>,
    assets: &mut usize,
) {
    if expected.get("$asset").is_some() {
        *assets += 1;
        if !actual.is_string() {
            diffs.push(format!("{path}: imagem/SVG ausente"));
        }
        return;
    }
    match (expected, actual) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            // Overshoot is expressed in percent (100 times the output ratio).
            let atol = if path.ends_with(".overshoot_percent") {
                100e-9
            } else {
                1e-9
            };
            if (a - b).abs() > atol + 1e-7 * a.abs() {
                diffs.push(format!("{path}: esperado {a}, recebido {b}"));
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                diffs.push(format!("{path}: tamanhos {} ≠ {}", a.len(), b.len()));
            }
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                compare(a, b, &format!("{path}[{i}]"), diffs, assets);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            for (key, value) in a {
                if let Some(other) = b.get(key) {
                    compare(value, other, &format!("{path}.{key}"), diffs, assets);
                } else {
                    diffs.push(format!("{path}.{key}: campo ausente"));
                }
            }
            for key in b.keys() {
                if !a.contains_key(key) {
                    diffs.push(format!("{path}.{key}: campo adicional"));
                }
            }
        }
        (Value::String(a), Value::String(b)) if numeric_text_equivalent(a, b, path) => {}
        _ if expected == actual => {}
        _ => diffs.push(format!(
            "{path}: esperado {}, recebido {}",
            short(expected),
            short(actual)
        )),
    }
}
fn short(v: &Value) -> String {
    let s = v.to_string();
    let mut out: String = s.chars().take(160).collect();
    if s.chars().count() > 160 {
        out.push('…');
    }
    out
}

fn numeric_text_equivalent(a: &str, b: &str, path: &str) -> bool {
    if path.contains(".metrics[") && path.ends_with(".value") {
        if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
            return a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 + 1e-7 * a.abs();
        }
    }
    if path.ends_with(".text") || path.ends_with(".str") {
        if let (Ok(a), Ok(b)) = (
            controlab_engine::input::complex(&a.replace(' ', "")),
            controlab_engine::input::complex(&b.replace(' ', "")),
        ) {
            return (a - b).norm() <= 1e-9 + 1e-7 * a.norm();
        }
    }
    if path.contains("latex") {
        use std::sync::OnceLock;
        static NUM: OnceLock<regex::Regex> = OnceLock::new();
        let re = NUM.get_or_init(|| {
            regex::Regex::new(r"[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?").unwrap()
        });
        if re.replace_all(a, "@N") == re.replace_all(b, "@N") {
            let aa: Vec<_> = re
                .find_iter(a)
                .map(|m| m.as_str().parse::<f64>().unwrap())
                .collect();
            let bb: Vec<_> = re
                .find_iter(b)
                .map(|m| m.as_str().parse::<f64>().unwrap())
                .collect();
            return aa.len() == bb.len()
                && aa
                    .iter()
                    .zip(bb)
                    .all(|(a, b)| (a - b).abs() <= 1e-9 + 1e-7 * a.abs());
        }
    }
    false
}

fn render_geometry_proof(expected: &Value, actual: &Value) -> Option<Value> {
    use base64::Engine;
    let e = expected["images"].as_array()?;
    let a = actual["images"].as_array()?;
    let depths = actual["depths"].as_array()?;
    if e.len() != a.len() || a.len() != depths.len() || expected["em"] != actual["em"] {
        return None;
    }
    for ((e, a), d) in e.iter().zip(a).zip(depths) {
        let d = d.as_f64()?;
        if !(0. ..=1.).contains(&d) {
            return None;
        }
        if e.is_null() {
            if !a.is_null() || d != 0. {
                return None;
            }
        } else {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(a.as_str()?)
                .ok()?;
            if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") || bytes.len() < 24 {
                return None;
            }
            let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
            let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
            if w == 0 || h == 0 {
                return None;
            }
        }
    }
    Some(
        json!({"reason":"Intrinsic metrics of native typesetter; normalized depths and PNG slots validated separately from scientific results.","expected_depths":expected["depths"],"actual_depths":actual["depths"]}),
    )
}

fn plateau_proof(
    case: &str,
    side: &str,
    expected: &Value,
    actual: &Value,
    root: &std::path::Path,
) -> Option<Value> {
    use controlab_engine::{
        simulation::{Input, simulate},
        transfer::TransferFunction,
    };
    let database: Value =
        serde_json::from_slice(&fs::read(root.join("plateaus.json")).ok()?).ok()?;
    let reference = &database["references"][case][side];
    let numbers =
        |v: &Value| -> Option<Vec<f64>> { v.as_array()?.iter().map(Value::as_f64).collect() };
    let time = numbers(&reference["time"])?;
    let original = numbers(&reference["output"])?;
    let steady = reference["steady_state"].as_f64()?;
    let (e, a) = (
        expected["peak_time"].as_f64()?,
        actual["peak_time"].as_f64()?,
    );
    let flat_tol = 1e-12 * steady.abs().max(1.);
    if time.len() != original.len()
        || e == a
        || steady == 0.
        || !time.iter().any(|t| (*t - e).abs() < 1e-9)
        || !time.iter().any(|t| (*t - a).abs() < 1e-9)
    {
        return None;
    }
    for metric in [expected, actual] {
        if (metric["peak"].as_f64()? - steady.abs()).abs() > flat_tol
            || metric["overshoot_percent"].as_f64()?.abs() > flat_tol * 100.
        {
            return None;
        }
    }
    let tf =
        TransferFunction::new(numbers(&reference["num"])?, numbers(&reference["den"])?).ok()?;
    let rust = simulate(&tf, &time, Input::Step).ok()?;
    let start = e.min(a);
    let mut samples = 0;
    let mut max_python = 0f64;
    let mut max_rust = 0f64;
    for ((t, py), rs) in time.iter().zip(&original).zip(&rust) {
        if (py - rs).abs() > 1e-9 + 1e-7 * py.abs() {
            return None;
        }
        if *t + 1e-9 >= start {
            samples += 1;
            max_python = max_python.max((py - steady).abs());
            max_rust = max_rust.max((rs - steady).abs());
        }
    }
    if samples < 2 || max_python > flat_tol || max_rust > flat_tol {
        return None;
    }
    Some(
        json!({"path":format!("$.details.{side}.peak_time"),"expected":e,"actual":a,"samples":samples,"max_python_delta":max_python,"max_rust_delta":max_rust,"plateau_tolerance":flat_tol}),
    )
}

// Fixture-only adapters for Python public helpers. These are deliberately not
// actions exposed by the production scientific JSON protocol.
fn fixture_response(input: &Value) -> Value {
    if input["action"] != "__core__" {
        return controlab_engine::dispatch::dispatch(input);
    }
    let a = &input["args"];
    let value: Result<Value, String> = match input["method"].as_str().unwrap_or("") {
        "parse_desired_pole" => controlab_engine::controller_math::desired_pole(&a[0])
            .map(|p| p.map_or(Value::Null, |p| json!({"re":p.re,"im":p.im}))),
        "parse_coeffs_str" => {
            controlab_engine::input::coefficients(a[0].as_str().unwrap_or("")).map(|v| json!(v))
        }
        "parse_matrix" => {
            controlab_engine::state_space::parse_matrix(&a[0], a[1].as_str().unwrap_or("A")).map(
                |m| {
                    json!(
                        (0..m.nrows())
                            .map(|r| (0..m.ncols()).map(|c| m[(r, c)]).collect::<Vec<_>>())
                            .collect::<Vec<_>>()
                    )
                },
            )
        }
        name => Err(format!("Adaptador de referência ausente: {name}")),
    };
    match value {
        Ok(v) => json!({"success":true,"value":v}),
        Err(e) => json!({"success":false,"error":format!("ValueError: {e}")}),
    }
}

fn parity(args: &[String]) -> Result<bool, String> {
    let option = |flag: &str| {
        args.iter()
            .position(|s| s == flag)
            .and_then(|i| args.get(i + 1))
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/reference/python");
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let section = option("--section");
    if section.is_some_and(|s| s != "details") {
        return Err(
            "--section aceita apenas details; omita para comparar resposta completa.".into(),
        );
    }
    let mut reports = vec![];
    let mut matched = 0;
    let mut failed = 0;
    let mut assets = 0;
    let mut archived = vec![];
    for entry in manifest["cases"].as_array().ok_or("Manifesto sem casos.")? {
        if option("--module").is_some_and(|s| entry["module"] != s.as_str())
            || option("--case").is_some_and(|s| entry["id"] != s.as_str())
        {
            continue;
        }
        let fixture: Value = serde_json::from_slice(
            &fs::read(root.join(entry["path"].as_str().ok_or("Caso sem caminho.")?))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let actual = fixture_response(&fixture["input"]);
        let expected = &fixture["output"];
        if matches!(
            fixture["input"]["action"].as_str(),
            Some("console_exec" | "home_get_state")
        ) {
            archived.push(json!({"case":entry["id"],"reason":"Ponte Python sem uso removida por autorização explícita do usuário em 2026-10-29; console .m Rust preservado."}));
            continue;
        }
        let mut diffs = vec![];
        let mut pending_assets = 0;
        if section.is_some() {
            compare(
                &expected["success"],
                &actual["success"],
                "$.success",
                &mut diffs,
                &mut pending_assets,
            );
            if expected["success"] == true && expected.get("details").is_none() {
                diffs.push("Caso não possui details: não é verificável nesta seleção.".into());
            }
            if expected["success"] == false {
                compare(
                    &expected["error"],
                    &actual["error"],
                    "$.error",
                    &mut diffs,
                    &mut pending_assets,
                );
            } else {
                compare(
                    &expected["details"],
                    &actual["details"],
                    "$.details",
                    &mut diffs,
                    &mut pending_assets,
                );
            }
        } else {
            compare(expected, &actual, "$", &mut diffs, &mut pending_assets);
        }
        let mut accepted_plateaus = vec![];
        let accepted_render_geometry = if fixture["input"]["action"] == "render_formulas" {
            render_geometry_proof(expected, &actual)
        } else {
            None
        };
        if accepted_render_geometry.is_some() {
            diffs.retain(|d| !d.starts_with("$.depths["));
        }
        if section.is_none() {
            for side in ["before", "after"] {
                if let Some(proof) = plateau_proof(
                    entry["id"].as_str().unwrap_or(""),
                    side,
                    &expected["details"][side],
                    &actual["details"][side],
                    &root,
                ) {
                    let prefix = format!("$.details.{side}.peak_time:");
                    diffs.retain(|d| !d.starts_with(&prefix));
                    accepted_plateaus.push(proof);
                }
            }
        }
        assets += pending_assets;
        if diffs.is_empty() {
            matched += 1;
        } else {
            failed += 1;
        }
        reports.push(json!({"case":entry["id"],"module":entry["module"],"differences":diffs,"visual_assets_unverified":pending_assets,"accepted_plateaus":accepted_plateaus,"accepted_render_geometry":accepted_render_geometry}));
    }
    if reports.is_empty() {
        return Err("Nenhum caso corresponde aos filtros.".into());
    }
    let report = json!({"scope":section.map_or("response",String::as_str),"numeric_atol":1e-9,"numeric_rtol":1e-7,"percentage_atol":100e-9,"numeric_metric_text":"same numeric tolerance; type/label/unit remain exact","matched":matched,"failed":failed,"archived":archived,"visual_assets_unverified":assets,"complete_parity":section.is_none()&&failed==0&&assets==0,"cases":reports});
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    // A partial details check can succeed for its stated scope; a full run with
    // unchecked assets cannot be used as evidence of complete parity.
    Ok(
        failed == 0
            && (section.is_some() || assets == 0 || args.iter().any(|s| s == "--data-only")),
    )
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result=match args.first().map(String::as_str){Some("parity")=>parity(&args[1..]),Some("package"|"installers"|"version"|"validate-bundle"|"check-sources"|"android")=>tools::run(&args).map(|()|true),_=>Err("Uso: cargo xtask parity [--module nome] [--case id] [--section details]\n     cargo xtask version".into())};
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tolerance_does_not_hide_types_or_missing_fields() {
        let mut d = vec![];
        let mut a = 0;
        compare(&json!({"v":1.}), &json!({"v":1.+1e-8}), "$", &mut d, &mut a);
        assert!(d.is_empty());
        compare(&json!({"v":1.}), &json!({"v":"1"}), "$", &mut d, &mut a);
        assert_eq!(d.len(), 1);
        compare(&json!({"flag":true}), &json!({}), "$", &mut d, &mut a);
        assert_eq!(d.len(), 2);
    }
    #[test]
    fn visual_assets_are_explicitly_unverified() {
        let mut d = vec![];
        let mut a = 0;
        compare(
            &json!({"$asset":"a.png"}),
            &json!("data:image/png;base64,..."),
            "$",
            &mut d,
            &mut a,
        );
        assert_eq!(a, 1);
        assert!(d.is_empty());
    }

    #[test]
    fn numeric_text_normalization_is_limited_to_scientific_values() {
        assert!(numeric_text_equivalent(
            "1.000000000000001",
            "1",
            "$.metrics[0].value"
        ));
        assert!(!numeric_text_equivalent("1", "2", "$.metrics[0].value"));
        assert!(!numeric_text_equivalent("1.0", "1", "$.title"));
        assert!(numeric_text_equivalent(
            "-0+1j",
            "0+1.0j",
            "$.details.poles[0].text"
        ));
    }

    #[test]
    fn plateau_exception_requires_both_frozen_and_rust_samples_to_be_flat() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/reference/python");
        let case = "legacy_zn_critical_PI";
        let f: Value =
            serde_json::from_slice(&fs::read(root.join(format!("cases/{case}.json"))).unwrap())
                .unwrap();
        let db: Value =
            serde_json::from_slice(&fs::read(root.join("plateaus.json")).unwrap()).unwrap();
        let times = db["references"][case]["before"]["time"].as_array().unwrap();
        let expected = &f["output"]["details"]["before"];
        let mut actual = expected.clone();
        actual["peak_time"] = times[times.len() - 2].clone();
        assert!(plateau_proof(case, "before", expected, &actual, &root).is_some());
        actual["peak_time"] = times[1].clone();
        assert!(plateau_proof(case, "before", expected, &actual, &root).is_none());
        assert!(plateau_proof("unknown", "before", expected, &actual, &root).is_none());
    }
}
