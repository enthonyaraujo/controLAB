use controlab_engine::controllers;
use serde_json::{json, Value};
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
    assert_eq!(r["details"]["controller_parameters"]["kp"], 3.6);
    assert_eq!(r["module"], "controller_design");
}
