use controlab_engine::{
    simulation::{Input, simulate},
    transfer::TransferFunction,
};

#[test]
fn sampled_parabolic_input_preserves_python_controls_first_order_hold() {
    let tf = TransferFunction::parse("1/(s+1)").unwrap();
    let y = simulate(&tf, &[0., 0.5, 1., 1.5], Input::Parabolic).unwrap();
    // python-control 0.10.2: forced_response(tf([1], [1, 1]), T=t, U=t**2 / 2).
    let expected = [
        0.,
        0.026632664928158356,
        0.1452351901491773,
        0.41798759047914535,
    ];
    for (a, b) in y.iter().zip(expected) {
        assert!((a - b).abs() < 1e-12, "{a} vs {b}");
    }
}

#[test]
fn first_order_continuous_responses_match_closed_forms() {
    let tf = TransferFunction::parse("1/(s+1)").unwrap();
    let t: Vec<f64> = (0..=100).map(|i| i as f64 / 10.).collect();
    for (input, f) in [
        (Input::Step, (|t: f64| 1. - (-t).exp()) as fn(f64) -> f64),
        (Input::Impulse, (|t: f64| (-t).exp()) as fn(f64) -> f64),
        (
            Input::Ramp,
            (|t: f64| t - 1. + (-t).exp()) as fn(f64) -> f64,
        ),
    ] {
        let y = simulate(&tf, &t, input).unwrap();
        for (time, value) in t.iter().zip(y) {
            assert!((value - f(*time)).abs() < 1e-10, "t={time}, y={value}");
        }
    }
}

#[test]
fn singular_integrator_and_feedthrough_are_supported() {
    let t = vec![0., 0.1, 0.2, 0.3];
    let y = simulate(&TransferFunction::parse("1/s").unwrap(), &t, Input::Step).unwrap();
    for (a, b) in y.iter().zip(&t) {
        assert!((a - b).abs() < 1e-12);
    }
    assert_eq!(
        simulate(&TransferFunction::parse("2").unwrap(), &t, Input::Step).unwrap(),
        vec![2.; 4]
    );
    assert!(simulate(&TransferFunction::parse("s+1").unwrap(), &t, Input::Step).is_err());
}
