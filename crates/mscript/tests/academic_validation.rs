//! Cross-module validation of UFERSA Control Systems II academic lists
//! (`Exercicios_-_LGR.pdf` and `Projeto_de_controladores.pdf`) executed inside
//! ControLAB's `.m` script interpreter (`mscript::Session`), verifying consistency
//! with closed-form analytical solutions and MATLAB R2022b references.

use mscript::{Session, Value};

fn assert_vec_near(actual: &[f64], expected: &[f64], tol: f64, ctx: &str) {
    assert_eq!(actual.len(), expected.len(), "{ctx}: length mismatch");
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= tol * e.abs().max(1.0), "{ctx}: {a} != {e}");
    }
}

fn get_scalar(session: &Session, name: &str) -> f64 {
    let Some(Value::Numeric(m)) = session.get(name) else {
        panic!("expected scalar variable '{name}' in workspace");
    };
    assert_eq!((m.rows(), m.cols()), (1, 1), "{name} is not 1x1");
    m.data()[0]
}

fn get_tf(session: &Session, name: &str) -> (Vec<f64>, Vec<f64>) {
    let Some(Value::TransferFunction(tf)) = session.get(name) else {
        panic!("expected transfer function '{name}' in workspace");
    };
    (tf.numerator().to_vec(), tf.denominator().to_vec())
}

fn get_vector(session: &Session, name: &str) -> Vec<f64> {
    let Some(Value::Numeric(m)) = session.get(name) else {
        panic!("expected numeric vector '{name}' in workspace");
    };
    m.data().to_vec()
}

#[test]
fn validate_all_9_controller_questions_in_mscript_studio_interpreter() {
    let mut session = Session::new();
    let script = r#"
        s = tf('s');

        % Q1: Reconstructed plant G1 = 1/((s+0.2)*(s+0.644)), PI controller
        G1 = 1 / ((s + 0.2) * (s + 0.644));
        Gc1 = pid(0.49849281561599984, 0.09975099980799994, 0);
        T1 = feedback(Gc1 * G1, 1);
        dc1 = dcgain(T1);

        % Q2: Reconstructed plant G2 = 1/(s^2+1), PD controller Kp=3, Kd=2
        G2 = 1 / (s^2 + 1);
        Gc2 = pid(3, 0, 2);
        T2 = feedback(Gc2 * G2, 1);
        dc2 = dcgain(T2);

        % Q3: Servo system with inner velocity feedback 0.5 and outer unity feedback
        G3_in = feedback(18 / ((2*s + 11) * (2*s + 1)), 0.5);
        G3 = G3_in * (1 / (30 * s));
        Gc3 = pid(160/3, 0, 100/3);
        T3 = feedback(Gc3 * G3, 1);
        dc3 = dcgain(T3);

        % Q4: Non-unity feedback H4 = 0.05, G4 = 5/(s+0.1), PI controller Kp=7.6, Ki=16
        G4 = 5 / (s + 0.1);
        H4 = tf(0.05, 1);
        Gc4 = pid(7.6, 16.0, 0);
        T4_yr = feedback(Gc4 * G4, H4);
        T4_ear = feedback(tf(1, 1), Gc4 * G4 * H4);
        dc4_y = dcgain(T4_yr);
        dc4_ea = dcgain(T4_ear);

        % Q5: 3rd-order plant 1/(s^3+10s^2+17s+8), PID controller (Kp=200, Ki=100, Kd=100)
        G5 = 1 / (s^3 + 10*s^2 + 17*s + 8);
        Gc5 = pid(200, 100, 100);
        T5 = feedback(Gc5 * G5, 1);
        dc5 = dcgain(T5);

        % Q6: 3rd-order plant 1/(s*(s+2)^2), PD controller Kp=8, Kd=4
        G6 = 1 / (s * (s + 2)^2);
        Gc6 = pid(8, 0, 4);
        T6 = feedback(Gc6 * G6, 1);
        dc6 = dcgain(T6);
        t6 = linspace(0, 10, 1001);
        y6 = step(T6, t6);

        % Q7: Two-loop cascade system: inner 1/s with feedback 2, outer 5/(s*(s+2)), PD Kp=1.6, Kd=0.8
        G7_in = feedback(1 / s, 2);
        G7 = (5 / (s * (s + 2))) * G7_in;
        Gc7 = pid(1.6, 0, 0.8);
        T7 = feedback(Gc7 * G7, 1);
        dc7 = dcgain(T7);
        y7 = step(T7, t6);

        % Q8: Wind turbine G8 = -7.2/(5s+1), PI controller Kp=-1.25, Ki=-4.260813967920802
        G8 = -7.2 / (5*s + 1);
        Gc8 = pid(-1.25, -4.260813967920802, 0);
        T8_ref = feedback(Gc8 * G8, 1);
        T8_dist = feedback(G8, Gc8);
        dc8_ref = dcgain(T8_ref);
        dc8_dist = dcgain(T8_dist);
        y8_ref = step(T8_ref, t6);
        y8_dist = step(T8_dist, t6);

        % Q9: Second-order diagram with wn=1, xi=0 -> G9 = 1/s^2, PD controller Kp=4, Kd=2
        G9 = 1 / (s^2);
        Gc9 = pid(4, 0, 2);
        T9 = feedback(Gc9 * G9, 1);
        dc9 = dcgain(T9);
        y9 = step(T9, t6);
    "#;

    let outcome = session.execute(script).unwrap();
    assert!(outcome.plots.is_empty());

    // Verify transfer function coefficients and DC gains across Q1..Q9:
    assert!((get_scalar(&session, "dc1") - 1.0).abs() <= 1e-12);
    assert!((get_scalar(&session, "dc2") - 0.75).abs() <= 1e-12);
    assert!((get_scalar(&session, "dc3") - 1.0).abs() <= 1e-12);
    assert!((get_scalar(&session, "dc4_y") - 20.0).abs() <= 1e-12);
    assert!(get_scalar(&session, "dc4_ea").abs() <= 1e-12);
    assert!((get_scalar(&session, "dc5") - 1.0).abs() <= 1e-12);
    assert!((get_scalar(&session, "dc6") - 1.0).abs() <= 1e-12);
    assert!((get_scalar(&session, "dc7") - 1.0).abs() <= 1e-12);
    assert!((get_scalar(&session, "dc8_ref") - 1.0).abs() <= 1e-12);
    assert!(get_scalar(&session, "dc8_dist").abs() <= 1e-12);
    assert!((get_scalar(&session, "dc9") - 1.0).abs() <= 1e-12);

    let (t2_num, t2_den) = get_tf(&session, "T2");
    assert_vec_near(&t2_num, &[2.0, 3.0], 1e-12, "Q2 T2 num");
    assert_vec_near(&t2_den, &[1.0, 2.0, 4.0], 1e-12, "Q2 T2 den");

    let (t3_num, t3_den) = get_tf(&session, "T3");
    assert_vec_near(&t3_num, &[5.0, 8.0], 1e-12, "Q3 T3 num");
    assert_vec_near(&t3_den, &[1.0, 6.0, 10.0, 8.0], 1e-12, "Q3 T3 den");

    let (t4_num, t4_den) = get_tf(&session, "T4_yr");
    assert_vec_near(&t4_num, &[38.0, 80.0], 1e-12, "Q4 T4_yr num");
    assert_vec_near(&t4_den, &[1.0, 2.0, 4.0], 1e-12, "Q4 T4_yr den");

    let (t5_num, t5_den) = get_tf(&session, "T5");
    assert_vec_near(&t5_num, &[100.0, 200.0, 100.0], 1e-12, "Q5 T5 num");
    assert_vec_near(
        &t5_den,
        &[1.0, 10.0, 117.0, 208.0, 100.0],
        1e-12,
        "Q5 T5 den",
    );

    let (t6_num, t6_den) = get_tf(&session, "T6");
    let (t7_num, t7_den) = get_tf(&session, "T7");
    assert_vec_near(&t6_num, &[4.0, 8.0], 1e-12, "Q6 T6 num");
    assert_vec_near(&t6_den, &[1.0, 4.0, 8.0, 8.0], 1e-12, "Q6 T6 den");
    assert_vec_near(&t7_num, &t6_num, 1e-12, "Q7 == Q6 num");
    assert_vec_near(&t7_den, &t6_den, 1e-12, "Q7 == Q6 den");

    // Verify step response waveforms for Q6, Q7, Q8 (ref & dist), and Q9:
    let y6 = get_vector(&session, "y6");
    let y7 = get_vector(&session, "y7");
    assert_vec_near(
        &y6,
        &y7,
        1e-13,
        "Q6 and Q7 step waveforms must be identical",
    );
    let sqrt3 = 3.0_f64.sqrt();
    for (i, &actual) in y6.iter().enumerate() {
        let t = (i as f64) * 0.01;
        let exact = 1.0 - (-t).exp() * ((sqrt3 * t).cos() + (sqrt3 * t).sin() / sqrt3);
        assert!(
            (actual - exact).abs() <= 1e-10,
            "mscript Q6 step response at t={t}: {actual} != {exact}"
        );
    }

    let y8_dist = get_vector(&session, "y8_dist");
    let wd8 = 2.266180070913597_f64;
    for (i, &actual) in y8_dist.iter().enumerate() {
        let t = (i as f64) * 0.01;
        let exact = (-1.44 / wd8) * (-t).exp() * (wd8 * t).sin();
        assert!(
            (actual - exact).abs() <= 1e-10,
            "mscript Q8 disturbance response at t={t}: {actual} != {exact}"
        );
    }
}

#[test]
fn validate_lgr_closed_loop_models_at_critical_and_breakaway_gains_in_mscript() {
    let mut session = Session::new();
    let script = r#"
        s = tf('s');

        % Eq.(1) & Eq.(7): G(s) = 1/(s*(s^2+6s+25)), K_crit = 150 -> sustained oscillation at w = 5 rad/s
        G_eq1 = 1 / (s * (s^2 + 6*s + 25));
        T_eq1_crit = feedback(150 * G_eq1, 1);

        % Eq.(3), Eq.(12), Eq.(14): G(s) = 1/(s*(s^2+4s+5)), breakaway K = 2 at s = -1
        G_eq3 = 1 / (s * (s^2 + 4*s + 5));
        T_eq3_break = feedback(2 * G_eq3, 1);

        % Eq.(15): G(s) = 1/(s*(s^2+4s+13)), H(s) = 1/(s+1), K_crit = 37.44
        G_eq15 = 1 / (s * (s^2 + 4*s + 13));
        H_eq15 = 1 / (s + 1);
        T_eq15_crit = feedback(37.44 * G_eq15, H_eq15);

        % Eq.(16): G(s) = 1/(s+3.6), H(s) = (s+1)/s^2
        G_eq16 = 1 / (s + 3.6);
        H_eq16 = (s + 1) / (s^2);
        T_eq16 = feedback(10 * G_eq16, H_eq16);
        dc_eq16 = dcgain(T_eq16);
    "#;
    session.execute(script).unwrap();

    // Eq.(1) at K = 150: denominator s^3 + 6s^2 + 25s + 150 = (s + 6)(s^2 + 25)
    let (_, den1) = get_tf(&session, "T_eq1_crit");
    assert_vec_near(&den1, &[1.0, 6.0, 25.0, 150.0], 1e-12, "Eq.1 K_crit den");

    // Eq.(3) at K = 2: denominator s^3 + 4s^2 + 5s + 2 = (s + 1)^2 * (s + 2)
    let (_, den3) = get_tf(&session, "T_eq3_break");
    assert_vec_near(&den3, &[1.0, 4.0, 5.0, 2.0], 1e-12, "Eq.3 breakaway den");

    // Eq.(15) at K = 37.44: denominator s^4 + 5s^3 + 17s^2 + 13s + 37.44
    let (num15, den15) = get_tf(&session, "T_eq15_crit");
    assert_vec_near(&num15, &[37.44, 37.44], 1e-12, "Eq.15 num");
    assert_vec_near(
        &den15,
        &[1.0, 5.0, 17.0, 13.0, 37.44],
        1e-12,
        "Eq.15 K_crit den",
    );

    // Eq.(16) at K = 10: T(s) = 10*s^2 / (s^3 + 3.6*s^2 + 10*s + 10) -> dcgain = 0
    let (num16, den16) = get_tf(&session, "T_eq16");
    assert_vec_near(&num16, &[10.0, 0.0, 0.0], 1e-12, "Eq.16 num");
    assert_vec_near(&den16, &[1.0, 3.6, 10.0, 10.0], 1e-12, "Eq.16 den");
    assert!(get_scalar(&session, "dc_eq16").abs() <= 1e-12);
}

#[test]
fn validate_q8_independent_reference_in_mscript() {
    // Etapa 10 Section 4 — Independent Q8 reference in Studio .m:
    // G(s) = -7.2/(5s+1), Gc(s) = -1 - 0.2/s => T(s) = 1.44/(s+1.44)
    // Analytic step response: y(t) = 1 - exp(-1.44*t), Mp = 0%, ts(2%) = ln(50)/1.44 = 2.7166826... s
    let mut session = Session::new();
    let script = r#"
        s = tf('s');
        G8 = -7.2 / (5*s + 1);
        Gc8_ref = pid(-1, -0.2, 0);
        T8_ref = feedback(Gc8_ref * G8, 1);
        T8_dist = feedback(G8, Gc8_ref);
        dc_ref = dcgain(T8_ref);
        dc_dist = dcgain(T8_dist);
        t = 0:0.01:8;
        y_ref = step(T8_ref, t);
        y_dist = step(T8_dist, t);
    "#;
    session.execute(script).unwrap();

    assert!((get_scalar(&session, "dc_ref") - 1.0).abs() <= 1e-12);
    assert!(get_scalar(&session, "dc_dist").abs() <= 1e-12);

    let y_ref = get_vector(&session, "y_ref");
    let mut peak = 0.0_f64;
    for (i, &actual) in y_ref.iter().enumerate() {
        let t = (i as f64) * 0.01;
        let exact = 1.0 - (-1.44 * t).exp();
        assert!(
            (actual - exact).abs() <= 1e-10,
            "mscript Q8 reference step at t={t}: {actual} != {exact}"
        );
        peak = peak.max(actual);
    }
    assert!(
        peak <= 1.0 + 1e-9,
        "Q8 reference must have 0% overshoot (peak={peak})"
    );

    // Verify 2% settling time directly from simulated samples:
    let mut last_out = 0usize;
    for (i, &y) in y_ref.iter().enumerate() {
        if (y - 1.0).abs() > 0.02 {
            last_out = i;
        }
    }
    let ts_approx = (last_out as f64) * 0.01;
    let exact_ts = 50.0_f64.ln() / 1.44;
    assert!(
        (ts_approx - exact_ts).abs() <= 0.02,
        "Q8 reference ts(2%) {ts_approx}s != {exact_ts}s"
    );

    // Verify constant load disturbance rejection in time domain:
    // T_d(s) = -1.44*s / ((s+0.2)(s+1.44)), step response = (-1.44/1.24)*(e^(-0.2t) - e^(-1.44t))
    let y_dist = get_vector(&session, "y_dist");
    for (i, &actual) in y_dist.iter().enumerate() {
        let t = (i as f64) * 0.01;
        let exact_d = (-1.44 / 1.24) * ((-0.2 * t).exp() - (-1.44 * t).exp());
        assert!(
            (actual - exact_d).abs() <= 1e-10,
            "mscript Q8 reference disturbance at t={t}: {actual} != {exact_d}"
        );
    }
}
