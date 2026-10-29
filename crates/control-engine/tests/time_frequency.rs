use controlab_engine::{frequency_response, time_response};
use serde_json::json;
fn near(value: &serde_json::Value, expected: f64, tolerance: f64) {
    assert!(
        (value.as_f64().unwrap() - expected).abs() < tolerance,
        "{value} != {expected}"
    );
}
#[test]
fn temporal_metrics_match_python_samples() {
    let r = time_response::analyze(&json!({"expr":"4/(s^2+2*s+4)","final_time":10,"points":1000}))
        .unwrap();
    let s = &r["details"]["step"];
    near(&s["final_value"], 1.000024293994804, 1e-10);
    near(&s["overshoot_percent"], 16.30039910180924, 1e-7);
    near(&s["rise_time"], 0.8208208208208209, 1e-10);
    near(&s["settling_time"], 4.044044044044044, 1e-10);
    assert_eq!(r["details"]["stable"], true);
    assert!(
        r["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,iVBOR")
    );
    assert!(
        r["plots"]["ramp"]["svg"]
            .as_str()
            .unwrap()
            .contains("Entrada r(t) = t")
    );
}
#[test]
fn temporal_negative_gain_and_type_one_preserve_metrics() {
    let r =
        time_response::analyze(&json!({"expr":"-2/(s+1)","final_time":10,"points":1000})).unwrap();
    near(&r["details"]["step"]["overshoot_percent"], 100., 1e-10);
    near(&r["details"]["step"]["rise_time"], 0., 1e-10);
    let r = time_response::analyze(
        &json!({"expr":"5/(s*(s+2))","final_time":8,"points":300,"response_type":"impulse"}),
    )
    .unwrap();
    assert_eq!(r["details"]["steady_state"]["kp"], "∞");
    near(&r["details"]["steady_state"]["kv"], 2.5, 1e-12);
    near(&r["details"]["steady_state"]["ess_ramp"], 0.4, 1e-12);
    assert_eq!(r["image"], r["plots"]["impulse"]["image"]);
    assert!(time_response::analyze(&json!({"expr":"2s"})).is_err());
    assert!(time_response::analyze(&json!({"settling_threshold":0.03})).is_err());
}
#[test]
fn exact_polynomial_margins_and_bandwidth_match_python() {
    let r = frequency_response::analyze(
        &json!({"expr":"10/(s*(s+2)*(s+5))","omega_max":200,"points":350}),
    )
    .unwrap();
    let d = &r["details"];
    near(&d["gain_margin"], 7., 1e-9);
    near(&d["phase_crossover_frequency"], 3.1622776601683795, 1e-9);
    near(&d["phase_margin_deg"], 55.64117265273913, 1e-8);
    near(&d["gain_crossover_frequency"], 0.8979134005379903, 1e-9);
    near(&d["stability_margin"], 0.6624153550455198, 1e-9);
    near(&d["stability_margin_frequency"], 1.5174600722875031, 1e-8);
    near(&d["bandwidth"], 1.5878571475014416, 1e-9);
    assert_eq!(d["nyquist"], json!({"Z":0,"N":0,"P":0}));
    assert!(r["svg"].as_str().unwrap().contains("Ponto crítico (-1, 0)"));
}
#[test]
fn frequency_infinite_and_negative_margins_are_not_sampled() {
    let r = frequency_response::analyze(&json!({"expr":"1/(s+1)","omega_min":10,"omega_max":100}))
        .unwrap();
    assert_eq!(r["details"]["gain_margin"], "∞");
    assert!(r["details"]["gain_crossover_frequency"].is_null());
    near(&r["details"]["bandwidth"], 1.9952566902229083, 1e-9);
    let r = frequency_response::analyze(&json!({"expr":"-2/(s+1)"})).unwrap();
    near(&r["details"]["gain_margin"], 0.5, 1e-10);
    near(&r["details"]["phase_margin_deg"], -60., 1e-8);
    assert_eq!(r["details"]["stable"], false);
    assert!(frequency_response::analyze(&json!({"omega_min":0})).is_err());
    assert!(frequency_response::analyze(&json!({"omega_min":1e-8,"omega_max":1e8})).is_err());
}
