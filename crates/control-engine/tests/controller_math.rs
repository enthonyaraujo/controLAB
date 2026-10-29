use controlab_engine::controller_math::*;
use serde_json::json;

#[test]
fn damping_and_settling_specifications_match_python() {
    assert!((mp_from_xi(0.5) - 16.303353482158048).abs() < 1e-12);
    assert!((xi_from_mp(10.).unwrap() - 0.5911550337988976).abs() < 1e-12);
    let s = resolve_transient(&json!({"ts":2.,"xi":0.5})).unwrap();
    assert_eq!(s["wn"], json!(4.));
    assert_eq!(s["re"], json!(2.));
    let s = resolve_transient(&json!({"desired_pole":"-1+sqrt(3)j","ts":999.,"xi":0.99})).unwrap();
    assert!((s["wn"].as_f64().unwrap() - 2.).abs() < 1e-12);
    assert_eq!(s["freq_source"], "pole");
}

#[test]
fn critical_manual_tuning_keeps_derivative_filter() {
    let p=pid_parameters(&json!({"method":"zn_critical_manual","controller_type":"PID","critical_gain":6.,"critical_period":2.})).unwrap();
    assert!((p["kp"].as_f64().unwrap() - 3.6).abs() < 1e-12);
    assert!((p["ti"].as_f64().unwrap() - 1.).abs() < 1e-12);
    let tf = pid_transfer(&p, "PID").unwrap();
    assert_eq!(tf.den, vec![0.025, 1., 0.]);
    assert!((tf.num[0] - 0.99).abs() < 1e-12);
}

#[test]
fn phase_geometry_and_static_error_match() {
    assert!((lgr_angle(0., 2., 2. * 3f64.sqrt()) - 120.).abs() < 1e-12);
    assert!((position_from_angle(43.89788624801398, 2., 2. * 3f64.sqrt()) - 5.6).abs() < 1e-7);
    assert!(required_single_angle(200., "z").is_err());
    assert_eq!(
        steady_state_error(&[5.], &[1., 2., 0.], "step")["json_value"],
        json!(0.)
    );
    assert_eq!(
        steady_state_error(&[5.], &[1., 2., 0.], "ramp")["json_value"],
        json!(0.4)
    );
}
