use controlab_engine::lgr;
use serde_json::{Value, json};
fn near(v: &Value, e: f64, t: f64) {
    assert!((v.as_f64().unwrap() - e).abs() < t, "{v} != {e}")
}
#[test]
fn geometry_and_all_seven_steps_match_reference() {
    let r = lgr::analyze(&json!({"expr":"1/(s*(s+1)*(s+2))"})).unwrap();
    let d = &r["detalhes"];
    assert_eq!(d["P"], 3);
    assert_eq!(d["Z"], 0);
    assert_eq!(d["ramos"], 3);
    near(&d["centroide"], -1., 1e-10);
    assert_eq!(d["jw_cruzamentos"].as_array().unwrap().len(), 2);
    for c in d["jw_cruzamentos"].as_array().unwrap() {
        near(&c["K"], 6., 1e-4);
        near(&json!(c["w"].as_f64().unwrap().abs()), 2_f64.sqrt(), 1e-4)
    }
    let steps = &d["passo_a_passo"];
    for key in ["passo_1_2_3", "passo_4", "passo_5", "passo_6", "passo_7"] {
        assert!(steps[key].is_object())
    }
    assert_eq!(steps["passo_4"]["n_assintotas"], 3);
    assert_eq!(steps["passo_6"]["cruzamentos"].as_array().unwrap().len(), 1);
    assert!(r["svg"].as_str().unwrap().contains("Componentes do LGR"));
    assert!(
        r["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,iVBOR")
    );
}
#[test]
fn conjugate_departure_arrival_and_deductions_are_retained() {
    let r = lgr::analyze(&json!({"expr":"1/(s*(s^2+2*s+2))"})).unwrap();
    let d = &r["detalhes"];
    near(&d["angulos_partida"][0]["angulo"], -45., 1e-8);
    assert_eq!(
        d["passo_a_passo"]["passo_7"]["deducoes_partida"][0]["termos_polos"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let r = lgr::analyze(&json!({"mode":"coeffs","num":"1 6 12","den":"1 4 4 0"})).unwrap();
    let a = &r["detalhes"]["angulos_chegada"][0];
    assert!(a["angulo"].as_f64().unwrap().is_finite());
    assert!(
        r["svg"]
            .as_str()
            .unwrap()
            .contains(r#"θ<tspan baseline-shift="sub" font-size="75%">a</tspan>"#)
    );
}
#[test]
fn all_presets_and_input_modes_work() {
    let p = lgr::presets();
    assert_eq!(p.as_array().unwrap().len(), 6);
    for v in p.as_array().unwrap() {
        let r = lgr::analyze(&json!({"mode":"preset","preset_key":v["id"]})).unwrap();
        assert_eq!(r["success"], true);
    }
    assert!(lgr::analyze(&json!({"expr":"__import__('os')"})).is_err());
}
fn compare_reference(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            let a = a.as_f64().unwrap();
            let b = b.as_f64().unwrap();
            assert!(
                (a - b).abs() < 1e-6_f64.max(b.abs() * 1e-7),
                "{path}: {a} != {b}"
            )
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}");
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                compare_reference(a, b, &format!("{path}/{i}"))
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len(), "{path}");
            for (k, b) in b {
                compare_reference(&a[k], b, &format!("{path}/{k}"))
            }
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}
#[test]
fn frozen_python_deduction_payloads_match() {
    for i in 0..6 {
        let path = format!(
            "{}/../../tests/reference/python/cases/lgr_preset_{i:02}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let fixture: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let actual = lgr::analyze(&fixture["input"]).unwrap();
        compare_reference(
            &actual["detalhes"],
            &fixture["output"]["detalhes"],
            &format!("preset {i}"),
        );
        assert_eq!(actual["latex_exp"], fixture["output"]["latex_exp"]);
        assert_eq!(actual["latex_fac"], fixture["output"]["latex_fac"]);
    }
}
#[test]
fn frozen_python_complex_zero_deductions_match() {
    let path = format!(
        "{}/../../tests/reference/python/cases/lgr_complex_zero.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let fixture: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let actual = lgr::analyze(&fixture["input"]).unwrap();
    compare_reference(
        &actual["detalhes"],
        &fixture["output"]["detalhes"],
        "complex zero",
    );
    assert_eq!(actual["latex_exp"], fixture["output"]["latex_exp"]);
    assert_eq!(actual["latex_fac"], fixture["output"]["latex_fac"]);
}
