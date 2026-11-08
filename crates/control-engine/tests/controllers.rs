use controlab_engine::controllers;
use serde_json::{Value, json};
#[test]
fn universal_controller_parameters_follow_python_references() {
    let base =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/python/cases");
    for index in 0..8 {
        let name = format!("controller_preset_{index:02}.json");
        let reference: Value =
            serde_json::from_str(&std::fs::read_to_string(base.join(&name)).unwrap()).unwrap();
        let actual = controllers::analyze(&reference["input"]).unwrap();
        for (key, value) in reference["output"]["details"]["controller_parameters"]
            .as_object()
            .unwrap()
        {
            if let Some(expected) = value.as_f64() {
                let observed = actual["details"]["controller_parameters"][key]
                    .as_f64()
                    .unwrap();
                assert!(
                    (observed - expected).abs() < 1e-7 * expected.abs().max(1.),
                    "{name}: {key}: {observed} vs {expected}"
                );
            }
        }
        assert_eq!(
            actual["details"]["steps"], reference["output"]["details"]["steps"],
            "{name}: deductions"
        );
    }
}
#[test]
fn classical_pid_manual_parameters_and_endpoint() {
    let r=controllers::analyze(&json!({"plant_expr":"1/(s*(s+1)*(s+5))","controller_type":"PID","method":"zn_critical_manual","critical_gain":6,"critical_period":2,"points":200})).unwrap();
    assert!(
        (r["details"]["controller_parameters"]["kp"]
            .as_f64()
            .unwrap()
            - 3.6)
            .abs()
            < 1e-12
    );
    assert_eq!(r["module"], "controller_design");
}

#[test]
fn pid_design_question_5_cancellation_and_equal_zeros() {
    // G(s) = 1 / (s^3 + 10s^2 + 17s + 8), ts(2%) = 1, xi = 0.4
    // Modo 1: Cancelamento de polo
    let r_cancel = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "1/(s^3 + 10s^2 + 17s + 8)",
        "controller_type": "PID",
        "pid_zeros_mode": "cancel",
        "ts": 1.0,
        "xi": 0.4,
        "settling_criterion": "2"
    })).unwrap();
    let p_cancel = &r_cancel["details"]["controller_parameters"];
    assert!((p_cancel["kp"].as_f64().unwrap() - 200.0).abs() < 1e-3);
    assert!((p_cancel["ki"].as_f64().unwrap() - 100.0).abs() < 1e-3);
    assert!((p_cancel["kd"].as_f64().unwrap() - 100.0).abs() < 1e-3);
    assert!((p_cancel["zi"].as_f64().unwrap() - 1.0).abs() < 1e-3);
    assert!((p_cancel["z"].as_f64().unwrap() - 1.0).abs() < 1e-3);

    // Modo 2: Zeros reais e iguais (duplo)
    let r_equal = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "1/(s^3 + 10s^2 + 17s + 8)",
        "controller_type": "PID",
        "pid_zeros_mode": "equal",
        "ts": 1.0,
        "xi": 0.4,
        "settling_criterion": "2"
    })).unwrap();
    let p_equal = &r_equal["details"]["controller_parameters"];
    assert!((p_equal["kp"].as_f64().unwrap() - 200.0).abs() < 1e-3);
    assert!((p_equal["ki"].as_f64().unwrap() - 100.0).abs() < 1e-3);
    assert!((p_equal["kd"].as_f64().unwrap() - 100.0).abs() < 1e-3);
    assert!((p_equal["zi"].as_f64().unwrap() - 1.0).abs() < 1e-3);
    assert!((p_equal["z"].as_f64().unwrap() - 1.0).abs() < 1e-3);
}

#[test]
fn pid_design_question_2_cancellation_and_equal_zeros() {
    // Inverted pendulum: G(s) = -20 / (s^2 - 20), xi = 0.7, wn = 6.57
    // Modo 1: Cancelamento de polo estável em s = -sqrt(20) ~ -4.4721
    let r_cancel = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "-20 / (s^2 - 20)",
        "controller_type": "PID",
        "pid_zeros_mode": "cancel",
        "wn": 6.57,
        "xi": 0.7,
        "settling_criterion": "2"
    })).unwrap();
    let p_cancel = &r_cancel["details"]["controller_parameters"];
    assert!((p_cancel["kp"].as_f64().unwrap() - (-5.215)).abs() < 1e-2);
    assert!((p_cancel["ki"].as_f64().unwrap() - (-9.652)).abs() < 1e-2);
    assert!((p_cancel["kd"].as_f64().unwrap() - (-0.6835)).abs() < 1e-2);
    assert!((p_cancel["zi"].as_f64().unwrap() - 20.0f64.sqrt()).abs() < 1e-3);
    assert!((p_cancel["z"].as_f64().unwrap() - 3.158).abs() < 1e-2);

    // Modo 2: Zeros reais e iguais (duplo)
    let r_equal = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "-20 / (s^2 - 20)",
        "controller_type": "PID",
        "pid_zeros_mode": "equal",
        "wn": 6.57,
        "xi": 0.7,
        "settling_criterion": "2"
    })).unwrap();
    let p_equal = &r_equal["details"]["controller_parameters"];
    assert!((p_equal["kp"].as_f64().unwrap() - (-5.335)).abs() < 1e-2);
    assert!((p_equal["ki"].as_f64().unwrap() - (-10.22)).abs() < 1e-2);
    assert!((p_equal["kd"].as_f64().unwrap() - (-0.6966)).abs() < 1e-2);
    assert!((p_equal["zi"].as_f64().unwrap() - 3.83).abs() < 1e-2);
    assert!((p_equal["z"].as_f64().unwrap() - 3.83).abs() < 1e-2);
}
