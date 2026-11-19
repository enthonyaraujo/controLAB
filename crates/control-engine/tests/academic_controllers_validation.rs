//! Scientific and numerical validation suite for Controller Design (Projeto de Controladores)
//! based on UFERSA Control Systems II (PEX0257) academic list `Projeto_de_controladores.pdf`.
//!
//! Covers all 9 Questions (Q1 to Q9), including:
//! - Question 1: Reconstructed 2nd-order plant from graphical LGR/step response + PI controller.
//! - Question 2: Reconstructed undamped plant from graphical LGR/step response + PD controller
//!   (plus PID zero-error alternative).
//! - Question 3: Multi-loop servo system reduction (inner velocity feedback 0.5 + outer unity loop) + PD controller.
//! - Question 4: Non-unity feedback system (H(s) = 0.05), desired poles -1 +/- sqrt(3)j, PI controller,
//!   and explicit distinction between summer actuating error e_a(inf) = 0 and output y(inf) = 1/H(0) = 20.
//! - Question 5: 3rd-order plant 1/(s^3 + 10s^2 + 17s + 8) + PID controller in all three synthesis modes
//!   (pole cancellation `cancel`, equal real zeros `equal`, and distinct zeros `distinct`), verifying all 4
//!   closed-loop poles and pole-zero cancellation.
//! - Question 6: 3rd-order Type 1 plant 1/(s(s+2)^2) + PD controller (kp + kd*s), verifying exact
//!   pole-zero cancellation of one (s+2) factor and reduced 2nd-order closed-loop transfer function.
//! - Question 7: Two-loop cascade system (inner feedback 2 around 1/s + outer 5/(s(s+2))) + PD controller.
//! - Question 8: Wind turbine speed control with negative plant gain k = -7.2 and time constant tau = 5
//!   (G(s) = -7.2/(5s+1)) + PI controller, verifying both reference step response and step load
//!   disturbance rejection T_p(s) = 1/s.
//! - Question 9: Second-order block diagram with wn = 1, xi = 0 reduced to double integrator 1/s^2 + PD controller.
//!
//! Reference sources:
//! - Closed-form analytical derivation (Root-Locus angle/magnitude conditions and Mason's gain formula).
//! - External numerical reference: MATLAB R2022b (Control System Toolbox 9.13 `feedback`, `minreal`, `stepinfo`).

use controlab_engine::{
    controllers, polynomial as poly,
    simulation::{self, Input},
    time_response,
    transfer::TransferFunction,
};
use num_complex::Complex64 as C;
use serde_json::{Value, json};

const TOL_COEFF: f64 = 1e-10;
const TOL_SIMPLE_ROOT: f64 = 1e-6;
const TOL_MULT_ROOT: f64 = 1e-4;
const TOL_GAIN: f64 = 1e-6;
const TOL_STEADY_STATE: f64 = 1e-5;
const TOL_WAVEFORM: f64 = 1e-4;
const TOL_OVERSHOOT_PP: f64 = 0.1;
const TOL_SETTLING_S: f64 = 0.02;

fn c(re: f64, im: f64) -> C {
    C::new(re, im)
}

fn unordered_complex_distance(actual: &[C], expected: &[C]) -> f64 {
    assert_eq!(
        actual.len(),
        expected.len(),
        "Root count mismatch: actual={} vs expected={}",
        actual.len(),
        expected.len()
    );
    let mut used = vec![false; expected.len()];
    let mut max_err = 0.0_f64;
    for a in actual {
        let mut best_idx = 0;
        let mut best_dist = f64::INFINITY;
        for (j, e) in expected.iter().enumerate() {
            if !used[j] {
                let d = (*a - *e).norm();
                if d < best_dist {
                    best_dist = d;
                    best_idx = j;
                }
            }
        }
        used[best_idx] = true;
        max_err = max_err.max(best_dist);
    }
    max_err
}

fn assert_tf_coeffs_eq(
    actual: &TransferFunction,
    expected_num: &[f64],
    expected_den: &[f64],
    ctx: &str,
) {
    let norm = actual.normalized();
    assert_eq!(norm.num.len(), expected_num.len(), "{ctx}: num len");
    assert_eq!(norm.den.len(), expected_den.len(), "{ctx}: den len");
    for (a, e) in norm.num.iter().zip(expected_num) {
        assert!(
            (a - e).abs() <= TOL_COEFF * e.abs().max(1.0),
            "{ctx}: num coeff {a} != {e}"
        );
    }
    for (a, e) in norm.den.iter().zip(expected_den) {
        assert!(
            (a - e).abs() <= TOL_COEFF * e.abs().max(1.0),
            "{ctx}: den coeff {a} != {e}"
        );
    }
}

#[derive(Clone, Debug)]
struct ControllerAcademicCase {
    id: &'static str,
    question: u32,
    payload: Value,
    expected_controller_type: &'static str,
    expected_kp: f64,
    expected_ki: Option<f64>,
    expected_kd: Option<f64>,
    expected_z: f64,
    expected_zi: Option<f64>,
    expected_td: Option<f64>,
    expected_cl_num: &'static [f64],
    expected_cl_den: &'static [f64],
    expected_cl_poles: &'static [(f64, f64)],
    expected_cl_zeros: &'static [(f64, f64)],
    /// Theoretical 2nd-order overshoot from dominant poles alone: exp(-xi*pi/sqrt(1-xi^2))*100.
    theoretical_2nd_order_mp: f64,
    /// Actual measured overshoot from step response (MATLAB R2022b `stepinfo` reference).
    matlab_measured_mp: f64,
    /// Actual measured 2% settling time (MATLAB R2022b `stepinfo` reference, definitive entry into +/-2% band).
    matlab_settling_time_2pct: f64,
    matlab_rise_time: f64,
    matlab_peak_time: f64,
    expected_steady_state_y: f64,
    expected_summer_error_ess: f64,
    pole_tolerance: f64,
    justification: &'static str,
}

fn all_controller_cases() -> Vec<ControllerAcademicCase> {
    const SQRT3: f64 = 1.7320508075688772;
    const SQRT84: f64 = 9.16515138991168;

    vec![
        // =========================================================================
        // Q1: Graphical reconstruction G(s) = 1/((s+0.2)(s+0.644)), xi=0.456, wn=0.706 -> PI
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q01_PI",
            question: 1,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "1/((s + 0.2)*(s + 0.644))",
                "controller_type": "PI",
                "xi": 0.456,
                "wn": 0.706,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PI",
            expected_kp: 0.49849281561599984,
            expected_ki: Some(0.09975099980799994),
            expected_kd: None,
            expected_z: 0.2001051904524144,
            expected_zi: None,
            expected_td: None,
            expected_cl_num: &[0.49849281561599984, 0.09975099980799994],
            expected_cl_den: &[1.0, 0.844, 0.6272928156159999, 0.09975099980799994],
            expected_cl_poles: &[
                (-0.321936, 0.6283257211860734),
                (-0.321936, -0.6283257211860734),
                (-0.200128, 0.0),
            ],
            expected_cl_zeros: &[(-0.2001051904524144, 0.0)],
            theoretical_2nd_order_mp: 19.995527167785674,
            matlab_measured_mp: 20.003167761706877,
            matlab_settling_time_2pct: 11.79426484164391,
            matlab_rise_time: 2.2070596916196417,
            matlab_peak_time: 5.006614870951746,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Reconstructed from graphical LGR poles at -0.2 and -0.644; desired pole sd = -0.321936 + 0.628326j lies to the right of uncompensated vertical LGR branch (-0.422), requiring negative phase from a PI controller whose zero z=0.200105 nearly cancels the slow pole at -0.2.",
        },
        // =========================================================================
        // Q2: Graphical reconstruction G(s) = 1/(s^2+1), xi=0.5, ts(2%)=4s -> PD
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q02_PD",
            question: 2,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "1/(s^2 + 1)",
                "controller_type": "PD",
                "xi": 0.5,
                "ts": 4.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PD",
            expected_kp: 3.0,
            expected_ki: None,
            expected_kd: Some(2.0),
            expected_z: 1.5,
            expected_zi: None,
            expected_td: Some(2.0 / 3.0),
            expected_cl_num: &[2.0, 3.0],
            expected_cl_den: &[1.0, 2.0, 4.0],
            expected_cl_poles: &[(-1.0, SQRT3), (-1.0, -SQRT3)],
            expected_cl_zeros: &[(-1.5, 0.0)],
            theoretical_2nd_order_mp: 16.303353482158046,
            matlab_measured_mp: 41.251081523683084,
            matlab_settling_time_2pct: 3.7232255355095183,
            matlab_rise_time: 0.3646132825300812,
            matlab_peak_time: 1.059189142777026,
            expected_steady_state_y: 0.75,
            expected_summer_error_ess: 0.25,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Reconstructed from undamped poles at +/-1j on jw axis; PD controller Gc(s)=3+2s places closed-loop poles at -1 +/- sqrt(3)j; closed-loop PD zero at s=-1.5 increases measured step overshoot to 41.25% compared to 16.30% pure 2nd-order estimate.",
        },
        // =========================================================================
        // Q3: Servo system reduced to G(s) = 3/(20*s*(s+1)*(s+5)), sd = -1 +/- 1j -> PD
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q03_PD",
            question: 3,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "3/(20*s*(s + 1)*(s + 5))",
                "controller_type": "PD",
                "desired_pole": "-1 + 1j",
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PD",
            expected_kp: 160.0 / 3.0,
            expected_ki: None,
            expected_kd: Some(100.0 / 3.0),
            expected_z: 1.6,
            expected_zi: None,
            expected_td: Some(0.625),
            expected_cl_num: &[5.0, 8.0],
            expected_cl_den: &[1.0, 6.0, 10.0, 8.0],
            expected_cl_poles: &[(-1.0, 1.0), (-1.0, -1.0), (-4.0, 0.0)],
            expected_cl_zeros: &[(-1.6, 0.0)],
            theoretical_2nd_order_mp: 4.321391826377226,
            matlab_measured_mp: 8.093793475727473,
            matlab_settling_time_2pct: 3.9134175958924926,
            matlab_rise_time: 1.1088150985328944,
            matlab_peak_time: 2.440740198573554,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Inner loop 18/((2s+11)(2s+1)) with H_in=0.5 reduces to 4.5/((s+1)(s+5)); cascaded with 1/(30s) gives Type-1 plant 0.15/(s(s+1)(s+5)) (e_ss=0 naturally); PD controller places dominant poles at -1 +/- 1j and 3rd pole at -4.",
        },
        // =========================================================================
        // Q4: G(s) = 5/(s+0.1), H(s) = 0.05, sd = -1 +/- sqrt(3)j, e_a(inf) = 0 -> PI
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q04_PI",
            question: 4,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "5/(s + 0.1)",
                "feedback_expr": "0.05",
                "controller_type": "PI",
                "desired_pole": "-1 + sqrt(3)j",
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PI",
            expected_kp: 7.6,
            expected_ki: Some(16.0),
            expected_kd: None,
            expected_z: 40.0 / 19.0,
            expected_zi: None,
            expected_td: None,
            expected_cl_num: &[38.0, 80.0],
            expected_cl_den: &[1.0, 2.0, 4.0],
            expected_cl_poles: &[(-1.0, SQRT3), (-1.0, -SQRT3)],
            expected_cl_zeros: &[(-40.0 / 19.0, 0.0)],
            theoretical_2nd_order_mp: 16.303353482158046,
            matlab_measured_mp: 28.38519483760318,
            matlab_settling_time_2pct: 3.758556766404977,
            matlab_rise_time: 0.48951297792656817,
            matlab_peak_time: 1.2433959502165088,
            expected_steady_state_y: 20.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Non-unity feedback H(s)=0.05 gives loop plant G(s)H(s)=0.25/(s+0.1); PI controller Kp=7.6, Ki=16 yields characteristic polynomial s^2+2s+4=0; actuating error at summer is e_a(inf)=0 while output steady state is y(inf)=1/H(0)=20.",
        },
        // =========================================================================
        // Q5 (Mode 1: Pole Cancellation): G(s) = 1/(s^3+10s^2+17s+8), ts=1, xi=0.4 -> PID
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q05_PID_CANCEL",
            question: 5,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "1/(s^3 + 10*s^2 + 17*s + 8)",
                "controller_type": "PID",
                "pid_zeros_mode": "cancel",
                "xi": 0.4,
                "ts": 1.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PID",
            expected_kp: 200.0,
            expected_ki: Some(100.0),
            expected_kd: Some(100.0),
            expected_z: 1.0,
            expected_zi: Some(1.0),
            expected_td: None,
            expected_cl_num: &[100.0, 200.0, 100.0],
            expected_cl_den: &[1.0, 10.0, 117.0, 208.0, 100.0],
            expected_cl_poles: &[(-4.0, SQRT84), (-4.0, -SQRT84), (-1.0, 0.0), (-1.0, 0.0)],
            expected_cl_zeros: &[(-1.0, 0.0), (-1.0, 0.0)],
            theoretical_2nd_order_mp: 25.382672198010745,
            matlab_measured_mp: 25.374074596318753,
            matlab_settling_time_2pct: 0.8409399301706838,
            matlab_rise_time: 0.14652080454767813,
            matlab_peak_time: 0.34538776394908755,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_MULT_ROOT,
            justification: "Plant factors as 1/((s+1)^2*(s+8)); canceling one pole at s=-1 requires the second PID zero via angle condition at sd=-4+sqrt(84)j, which also evaluates to z2=1, canceling the entire double pole (s+1)^2 and leaving exact 2nd-order dynamics 100/(s^2+8s+100).",
        },
        // =========================================================================
        // Q5 (Mode 2: Equal Zeros): G(s) = 1/(s^3+10s^2+17s+8), ts=1, xi=0.4 -> PID
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q05_PID_EQUAL",
            question: 5,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "1/(s^3 + 10*s^2 + 17*s + 8)",
                "controller_type": "PID",
                "pid_zeros_mode": "equal",
                "xi": 0.4,
                "ts": 1.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PID",
            expected_kp: 200.0,
            expected_ki: Some(100.0),
            expected_kd: Some(100.0),
            expected_z: 1.0,
            expected_zi: Some(1.0),
            expected_td: None,
            expected_cl_num: &[100.0, 200.0, 100.0],
            expected_cl_den: &[1.0, 10.0, 117.0, 208.0, 100.0],
            expected_cl_poles: &[(-4.0, SQRT84), (-4.0, -SQRT84), (-1.0, 0.0), (-1.0, 0.0)],
            expected_cl_zeros: &[(-1.0, 0.0), (-1.0, 0.0)],
            theoretical_2nd_order_mp: 25.382672198010745,
            matlab_measured_mp: 25.374074596318753,
            matlab_settling_time_2pct: 0.8409399301706838,
            matlab_rise_time: 0.14652080454767813,
            matlab_peak_time: 0.34538776394908755,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_MULT_ROOT,
            justification: "Because the pole at -8 and origin pole at 0 are symmetric relative to Re(sd)=-4, their angle sum at sd=-4+sqrt(84)j is 180 deg; hence the double zero angle equals the double pole angle at -1, yielding z0=1 and identical parameters Kp=200, Ki=100, Kd=100.",
        },
        // =========================================================================
        // Q5 (Mode 3: Distinct Zeros with zi = 0.4): G(s) = 1/(s^3+10s^2+17s+8) -> PID
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q05_PID_DISTINCT",
            question: 5,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "1/(s^3 + 10*s^2 + 17*s + 8)",
                "controller_type": "PID",
                "pid_zeros_mode": "distinct",
                "pi_zero": 0.4,
                "xi": 0.4,
                "ts": 1.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PID",
            expected_kp: 197.02970297029572,
            expected_ki: Some(62.87128712871236),
            expected_kd: Some(99.62871287128709),
            expected_z: 1.5776397515527827,
            expected_zi: Some(0.4),
            expected_td: None,
            expected_cl_num: &[99.62871287128709, 197.02970297029572, 62.87128712871236],
            expected_cl_den: &[
                1.0,
                10.0,
                116.62871287128709,
                205.02970297029572,
                62.87128712871236,
            ],
            expected_cl_poles: &[
                (-4.0, SQRT84),
                (-4.0, -SQRT84),
                (-1.60933334777678, 0.0),
                (-0.3906666522232091, 0.0),
            ],
            expected_cl_zeros: &[(-1.5776397515527827, 0.0), (-0.4, 0.0)],
            theoretical_2nd_order_mp: 25.382672198010745,
            matlab_measured_mp: 24.473866097034815,
            matlab_settling_time_2pct: 0.8737636210949157,
            matlab_rise_time: 0.14738119773290045,
            matlab_peak_time: 0.34538776394908755,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Cascade PI-PD synthesis fixing z_i = 0.1*Re(sd) = 0.4 places z_d = 254/161 = 1.577640; non-dominant closed-loop poles at -1.60933 and -0.39067 form near-dipoles with the zeros at -1.57764 and -0.4.",
        },
        // =========================================================================
        // Q6: G(s) = 1/(s*(s+2)^2), PD controller kp + kd*s, xi = 0.5, wn = 2 -> Kp=8, Kd=4
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q06_PD",
            question: 6,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "1/(s*(s + 2)^2)",
                "controller_type": "PD",
                "xi": 0.5,
                "wn": 2.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PD",
            expected_kp: 8.0,
            expected_ki: None,
            expected_kd: Some(4.0),
            expected_z: 2.0,
            expected_zi: None,
            expected_td: Some(0.5),
            expected_cl_num: &[4.0, 8.0],
            expected_cl_den: &[1.0, 4.0, 8.0, 8.0],
            expected_cl_poles: &[(-1.0, SQRT3), (-1.0, -SQRT3), (-2.0, 0.0)],
            expected_cl_zeros: &[(-2.0, 0.0)],
            theoretical_2nd_order_mp: 16.303353482158046,
            matlab_measured_mp: 16.292919869949184,
            matlab_settling_time_2pct: 4.037934116881343,
            matlab_rise_time: 0.8195145513072228,
            matlab_peak_time: 1.7960163725349574,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "At sd = -1 + sqrt(3)j, angle from s=0 is 120 deg and from (s+2)^2 is 2*60=120 deg, requiring 60 deg from PD zero => z = 2.0, which exactly cancels one (s+2) plant pole and reduces T(s) = (4s+8)/(s^3+4s^2+8s+8) to T_min(s) = 4/(s^2+2s+4).",
        },
        // =========================================================================
        // Q7: Inner loop 1/s with feedback 2 -> 1/(s+2), times 5/(s(s+2)) -> 5/(s(s+2)^2) -> PD
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q07_PD",
            question: 7,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "5/(s*(s + 2)^2)",
                "controller_type": "PD",
                "xi": 0.5,
                "ts": 4.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PD",
            expected_kp: 1.6,
            expected_ki: None,
            expected_kd: Some(0.8),
            expected_z: 2.0,
            expected_zi: None,
            expected_td: Some(0.5),
            expected_cl_num: &[4.0, 8.0],
            expected_cl_den: &[1.0, 4.0, 8.0, 8.0],
            expected_cl_poles: &[(-1.0, SQRT3), (-1.0, -SQRT3), (-2.0, 0.0)],
            expected_cl_zeros: &[(-2.0, 0.0)],
            theoretical_2nd_order_mp: 16.303353482158046,
            matlab_measured_mp: 16.292919869949184,
            matlab_settling_time_2pct: 4.037934116881343,
            matlab_rise_time: 0.8195145513072228,
            matlab_peak_time: 1.7960163725349574,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Inner loop (1/s)/(1+2/s) = 1/(s+2) in series with 5/(s(s+2)) gives 5/(s(s+2)^2); PD zero z=2 cancels one (s+2) pole with gains Kp=1.6, Kd=0.8 (1/5 of Q6 gains), producing the same closed-loop transfer function 4/(s^2+2s+4).",
        },
        // =========================================================================
        // Q8: Wind turbine G(s) = -7.2/(5s+1), ts=4s, Mp=25%, e(inf)=0 -> PI (negative gains)
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q08_PI",
            question: 8,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "-7.2/(5*s + 1)",
                "controller_type": "PI",
                "ts": 4.0,
                "mp": 25.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PI",
            expected_kp: -1.25,
            expected_ki: Some(-4.260813967920802),
            expected_kd: None,
            expected_z: 3.4086511743366414,
            expected_zi: None,
            expected_td: None,
            expected_cl_num: &[1.8, 6.135572113805955],
            expected_cl_den: &[1.0, 2.0, 6.135572113805955],
            expected_cl_poles: &[(-1.0, 2.266180070913597), (-1.0, -2.266180070913597)],
            expected_cl_zeros: &[(-3.4086511743366414, 0.0)],
            theoretical_2nd_order_mp: 25.0,
            matlab_measured_mp: 33.840604831375764,
            matlab_settling_time_2pct: 3.965373030165275,
            matlab_rise_time: 0.4229095765576573,
            matlab_peak_time: 1.059189142777026,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Negative plant gain k=-7.2 (tau=5 => G(s)=-1.44/(s+0.2)) requires negative PI gains Kp=-1.25, Ki=-4.260814; closed-loop poles are -1 +/- 2.26618j (2nd-order Mp=25%), while closed-loop PI zero at -3.40865 increases actual step overshoot to 33.84%.",
        },
        // =========================================================================
        // Q9: Second-order block diagram with wn=1, xi=0 -> G(s) = 1/s^2, ts=4s, xi=0.5 -> PD
        // =========================================================================
        ControllerAcademicCase {
            id: "CTRL_Q09_PD",
            question: 9,
            payload: json!({
                "action": "controller_design",
                "plant_expr": "1/(s^2)",
                "controller_type": "PD",
                "xi": 0.5,
                "ts": 4.0,
                "settling_criterion": "2",
                "points": 2000,
                "dpi": 72
            }),
            expected_controller_type: "PD",
            expected_kp: 4.0,
            expected_ki: None,
            expected_kd: Some(2.0),
            expected_z: 2.0,
            expected_zi: None,
            expected_td: Some(0.5),
            expected_cl_num: &[2.0, 4.0],
            expected_cl_den: &[1.0, 2.0, 4.0],
            expected_cl_poles: &[(-1.0, SQRT3), (-1.0, -SQRT3)],
            expected_cl_zeros: &[(-2.0, 0.0)],
            theoretical_2nd_order_mp: 16.303353482158046,
            matlab_measured_mp: 29.835150652637886,
            matlab_settling_time_2pct: 3.752715553619379,
            matlab_rise_time: 0.4704600578119872,
            matlab_peak_time: 1.197_344_248_356_638,
            expected_steady_state_y: 1.0,
            expected_summer_error_ess: 0.0,
            pole_tolerance: TOL_SIMPLE_ROOT,
            justification: "Inner loop (1/s)/(1+2*xi*wn/s) with xi=0, wn=1 reduces to 1/s, cascaded with wn^2/s gives double integrator G(s)=1/s^2; PD controller Gc(s)=4+2s places poles at -1 +/- sqrt(3)j; closed-loop zero at s=-2 raises step overshoot from 16.30% to 29.84%.",
        },
    ]
}

#[test]
fn validate_all_9_controller_design_questions_against_analytical_and_matlab() {
    let cases = all_controller_cases();
    assert_eq!(cases.len(), 11, "11 configurations covering Q1 through Q9");

    for case in &cases {
        assert!((1..=9).contains(&case.question));
        let res = controllers::analyze(&case.payload).unwrap();
        assert_eq!(
            res["success"], true,
            "{} ({}): controller design failed",
            case.id, case.justification
        );
        assert!(res["svg"].as_str().unwrap().contains("<svg"));
        assert!(
            res["image"]
                .as_str()
                .unwrap()
                .starts_with("data:image/png;base64,iVBOR")
        );

        let det = &res["details"];
        assert_eq!(det["stable"], true, "{}: must be stable", case.id);
        assert_eq!(
            det["controller_type"].as_str().unwrap(),
            case.expected_controller_type,
            "{}: controller_type mismatch",
            case.id
        );

        // 1. Validate synthesized controller parameters (Kp, Ki, Kd, z, zi, td).
        let cp = &det["controller_parameters"];
        let act_kp = cp["kp"].as_f64().unwrap();
        assert!(
            (act_kp - case.expected_kp).abs() <= TOL_GAIN * case.expected_kp.abs().max(1.0),
            "{}: Kp {act_kp} != {}",
            case.id,
            case.expected_kp
        );
        if let Some(exp_ki) = case.expected_ki {
            let act_ki = cp["ki"].as_f64().unwrap();
            assert!(
                (act_ki - exp_ki).abs() <= TOL_GAIN * exp_ki.abs().max(1.0),
                "{}: Ki {act_ki} != {exp_ki}",
                case.id
            );
        }
        if let Some(exp_kd) = case.expected_kd {
            let act_kd = cp["kd"].as_f64().unwrap();
            assert!(
                (act_kd - exp_kd).abs() <= TOL_GAIN * exp_kd.abs().max(1.0),
                "{}: Kd {act_kd} != {exp_kd}",
                case.id
            );
        }
        let act_z = cp["z"].as_f64().unwrap();
        assert!(
            (act_z - case.expected_z).abs() <= TOL_GAIN * case.expected_z.abs().max(1.0),
            "{}: zero z {act_z} != {}",
            case.id,
            case.expected_z
        );
        if let Some(exp_zi) = case.expected_zi {
            let act_zi = cp["zi"].as_f64().unwrap();
            assert!(
                (act_zi - exp_zi).abs() <= TOL_GAIN * exp_zi.abs().max(1.0),
                "{}: zero zi {act_zi} != {exp_zi}",
                case.id
            );
        }
        if let Some(exp_td) = case.expected_td {
            let act_td = cp["td"].as_f64().unwrap();
            assert!(
                (act_td - exp_td).abs() <= TOL_GAIN * exp_td.abs().max(1.0),
                "{}: td {act_td} != {exp_td}",
                case.id
            );
        }

        // 2. Validate closed-loop transfer function, all closed-loop poles, and all closed-loop zeros.
        let plant = TransferFunction::parse(case.payload["plant_expr"].as_str().unwrap()).unwrap();
        let feedback = case
            .payload
            .get("feedback_expr")
            .and_then(|v| v.as_str())
            .map(|s| TransferFunction::parse(s).unwrap())
            .unwrap_or_else(|| TransferFunction::scalar(1.0));
        let gc = match case.expected_controller_type {
            "PI" => TransferFunction::new(vec![act_kp, cp["ki"].as_f64().unwrap()], vec![1.0, 0.0])
                .unwrap(),
            "PD" => {
                TransferFunction::new(vec![cp["kd"].as_f64().unwrap(), act_kp], vec![1.0]).unwrap()
            }
            "PID" => TransferFunction::new(
                vec![
                    cp["kd"].as_f64().unwrap(),
                    act_kp,
                    cp["ki"].as_f64().unwrap(),
                ],
                vec![1.0, 0.0],
            )
            .unwrap(),
            other => panic!("Unsupported controller type {other}"),
        };
        let forward = gc.multiply(&plant).unwrap();
        let cl_tf = forward.feedback(&feedback, -1.0).unwrap().normalized();
        assert_tf_coeffs_eq(&cl_tf, case.expected_cl_num, case.expected_cl_den, case.id);

        let actual_poles: Vec<C> = det["closed_loop_poles_after"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| c(p["re"].as_f64().unwrap(), p["im"].as_f64().unwrap()))
            .collect();
        let expected_poles: Vec<C> = case
            .expected_cl_poles
            .iter()
            .map(|&(re, im)| c(re, im))
            .collect();
        let pole_err = unordered_complex_distance(&actual_poles, &expected_poles);
        assert!(
            pole_err <= case.pole_tolerance,
            "{}: closed-loop pole error {pole_err:e} > {:e}",
            case.id,
            case.pole_tolerance
        );

        let cl_zeros = cl_tf.zeros().unwrap();
        let expected_zeros: Vec<C> = case
            .expected_cl_zeros
            .iter()
            .map(|&(re, im)| c(re, im))
            .collect();
        let zero_err = unordered_complex_distance(&cl_zeros, &expected_zeros);
        assert!(
            zero_err <= case.pole_tolerance,
            "{}: closed-loop zero error {zero_err:e} > {:e}",
            case.id,
            case.pole_tolerance
        );

        // 3. Validate theoretical 2nd-order estimate vs actual measured step response metrics.
        let theoretical_mp = cp["mp"].as_f64().unwrap();
        assert!(
            (theoretical_mp - case.theoretical_2nd_order_mp).abs() <= 1e-3,
            "{}: theoretical 2nd-order Mp {theoretical_mp} != {}",
            case.id,
            case.theoretical_2nd_order_mp
        );

        let after_metrics = &det["after"];
        let act_mp = after_metrics["overshoot_percent"].as_f64().unwrap();
        assert!(
            (act_mp - case.matlab_measured_mp).abs() <= TOL_OVERSHOOT_PP,
            "{}: measured overshoot {act_mp}% vs MATLAB {}% exceeds {TOL_OVERSHOOT_PP} pp",
            case.id,
            case.matlab_measured_mp
        );

        let act_ts = after_metrics["settling_time"].as_f64().unwrap();
        assert!(
            (act_ts - case.matlab_settling_time_2pct).abs() <= TOL_SETTLING_S,
            "{}: settling time (2%) {act_ts}s vs MATLAB {}s exceeds {TOL_SETTLING_S}s",
            case.id,
            case.matlab_settling_time_2pct
        );

        let act_tr = after_metrics["rise_time"].as_f64().unwrap();
        assert!(
            (act_tr - case.matlab_rise_time).abs() <= 0.025,
            "{}: rise time {act_tr}s vs MATLAB {}s",
            case.id,
            case.matlab_rise_time
        );

        let act_tp = after_metrics["peak_time"].as_f64().unwrap();
        assert!(
            (act_tp - case.matlab_peak_time).abs() <= 0.035,
            "{}: peak time {act_tp}s vs MATLAB {}s",
            case.id,
            case.matlab_peak_time
        );

        // 4. Validate steady-state output value y(inf) and summer actuating error e_a(inf).
        let act_final_y = after_metrics["steady_state_value"].as_f64().unwrap();
        let dc_gain = cl_tf.dc_gain();
        assert!(
            (dc_gain - case.expected_steady_state_y).abs() <= TOL_STEADY_STATE,
            "{}: exact dc_gain {dc_gain} != {}",
            case.id,
            case.expected_steady_state_y
        );
        assert!(
            (act_final_y - case.expected_steady_state_y).abs() <= TOL_STEADY_STATE,
            "{}: steady_state_value {act_final_y} != {}",
            case.id,
            case.expected_steady_state_y
        );
        let act_ess = cp["e_novo"].as_f64().unwrap();
        assert!(
            (act_ess - case.expected_summer_error_ess).abs() <= TOL_STEADY_STATE,
            "{}: steady-state error {act_ess} != {}",
            case.id,
            case.expected_summer_error_ess
        );
    }
}

#[test]
fn validate_block_diagram_reductions_pole_zero_cancellations_and_q8_disturbance() {
    // 1. Q3 Block Diagram Reduction:
    //    Inner loop: 18/((2s+11)(2s+1)) with feedback 0.5 -> 4.5/((s+1)(s+5))
    //    Cascaded with 1/(30s) -> 3/(20s(s+1)(s+5)) = 0.15/(s^3 + 6s^2 + 5s)
    let q3_fwd_inner = TransferFunction::parse("18/((2*s + 11)*(2*s + 1))").unwrap();
    let q3_h_inner = TransferFunction::scalar(0.5);
    let q3_inner_cl = q3_fwd_inner
        .feedback(&q3_h_inner, -1.0)
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q3_inner_cl, &[4.5], &[1.0, 6.0, 5.0], "Q3 inner loop");
    let q3_outer_plant = q3_inner_cl
        .multiply(&TransferFunction::parse("1/(30*s)").unwrap())
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(
        &q3_outer_plant,
        &[0.15],
        &[1.0, 6.0, 5.0, 0.0],
        "Q3 reduced plant",
    );

    // 2. Q4 Non-Unity Feedback Distinction (Summer Error vs Output Tracking):
    //    G(s) = 5/(s+0.1), H(s) = 0.05, Gc(s) = (7.6s + 16)/s
    //    T_YR(s) = Gc*G / (1 + Gc*G*H) = (38s + 80)/(s^2 + 2s + 4) -> y(inf) = 20
    //    T_EaR(s) = 1 / (1 + Gc*G*H) = s(s + 0.1)/(s^2 + 2s + 4) -> e_a(inf) = 0
    //    T_EY_R(s) = 1 - T_YR(s) -> output tracking difference r(inf) - y(inf) = 1 - 20 = -19
    let q4_g = TransferFunction::parse("5/(s + 0.1)").unwrap();
    let q4_h = TransferFunction::scalar(0.05);
    let q4_gc = TransferFunction::new(vec![7.6, 16.0], vec![1.0, 0.0]).unwrap();
    let q4_tyr = q4_gc
        .multiply(&q4_g)
        .unwrap()
        .feedback(&q4_h, -1.0)
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q4_tyr, &[38.0, 80.0], &[1.0, 2.0, 4.0], "Q4 T_YR(s)");
    assert!((q4_tyr.dc_gain() - 20.0).abs() <= TOL_STEADY_STATE);
    let q4_tear = TransferFunction::scalar(1.0)
        .feedback(
            &q4_gc.multiply(&q4_g).unwrap().multiply(&q4_h).unwrap(),
            -1.0,
        )
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q4_tear, &[1.0, 0.1, 0.0], &[1.0, 2.0, 4.0], "Q4 T_EaR(s)");
    assert!(q4_tear.dc_gain().abs() <= TOL_STEADY_STATE);

    // 3. Q6 and Q7 Exact Pole-Zero Cancellation and Analytical Step Waveform Verification:
    //    Unreduced T(s) = (4s + 8)/(s^3 + 4s^2 + 8s + 8) = 4(s+2)/((s+2)(s^2 + 2s + 4))
    //    Reduced T_min(s) = 4/(s^2 + 2s + 4)
    //    Exact unit step response: y(t) = 1 - e^(-t)*(cos(sqrt(3)*t) + (1/sqrt(3))*sin(sqrt(3)*t))
    let q6_unreduced = TransferFunction::new(vec![4.0, 8.0], vec![1.0, 4.0, 8.0, 8.0]).unwrap();
    let q6_reduced = TransferFunction::new(vec![4.0], vec![1.0, 2.0, 4.0]).unwrap();
    let t_grid: Vec<f64> = (0..=1000).map(|i| (i as f64) * 0.01).collect();
    let y_unreduced = simulation::simulate(&q6_unreduced, &t_grid, Input::Step).unwrap();
    let y_reduced = simulation::simulate(&q6_reduced, &t_grid, Input::Step).unwrap();
    let sqrt3 = 3.0_f64.sqrt();
    for (idx, &t) in t_grid.iter().enumerate() {
        let y_exact = 1.0 - (-t).exp() * ((sqrt3 * t).cos() + (sqrt3 * t).sin() / sqrt3);
        assert!(
            (y_unreduced[idx] - y_reduced[idx]).abs() <= 1e-12,
            "Q6/Q7 pole-zero cancellation mismatch at t={t}: {} vs {}",
            y_unreduced[idx],
            y_reduced[idx]
        );
        assert!(
            (y_reduced[idx] - y_exact).abs() <= TOL_WAVEFORM,
            "Q6/Q7 step waveform mismatch at t={t}: {} vs {y_exact}",
            y_reduced[idx]
        );
    }

    // 4. Q7 Block Diagram Reduction:
    //    Inner loop: (1/s) with feedback 2 -> 1/(s+2)
    //    Series with 5/(s(s+2)) -> 5/(s(s+2)^2) = 5/(s^3 + 4s^2 + 4s)
    let q7_inner = TransferFunction::parse("1/s")
        .unwrap()
        .feedback(&TransferFunction::scalar(2.0), -1.0)
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q7_inner, &[1.0], &[1.0, 2.0], "Q7 inner loop");
    let q7_plant = q7_inner
        .multiply(&TransferFunction::parse("5/(s*(s + 2))").unwrap())
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q7_plant, &[5.0], &[1.0, 4.0, 4.0, 0.0], "Q7 reduced plant");

    // 5. Q8 Step Disturbance Response Validation:
    //    G_p(s) = -7.2/(5s + 1) = -1.44/(s + 0.2)
    //    G_c(s) = (-1.25s - 4.260813967920802)/s
    //    Disturbance transfer function from T_p(s) to w(s):
    //    T_d(s) = G_p(s) / (1 + G_c(s)G_p(s)) = -1.44*s / (s^2 + 2s + 6.135572113805955)
    let q8_gp = TransferFunction::parse("-7.2/(5*s + 1)")
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q8_gp, &[-1.44], &[1.0, 0.2], "Q8 plant");
    let q8_gc = TransferFunction::new(vec![-1.25, -4.260813967920802], vec![1.0, 0.0]).unwrap();
    let q8_td = q8_gp.feedback(&q8_gc, -1.0).unwrap().normalized();
    assert_tf_coeffs_eq(
        &q8_td,
        &[-1.44, 0.0],
        &[1.0, 2.0, 6.135572113805955],
        "Q8 disturbance transfer function T_d(s)",
    );
    // Steady-state disturbance response to unit step T_p(s) = 1/s is T_d(0) = 0:
    assert!(
        q8_td.dc_gain().abs() <= TOL_STEADY_STATE,
        "Q8 must completely reject constant step load disturbance in steady state"
    );
    // Verify analytical disturbance step response waveform:
    // w_d(t) = L^-1{ -1.44 / ((s+1)^2 + wd^2) } = (-1.44 / wd) * e^(-t) * sin(wd * t)
    let wd = 2.266180070913597_f64;
    let t_dist: Vec<f64> = (0..=1500).map(|i| (i as f64) * 0.01).collect();
    let y_dist = simulation::simulate(&q8_td, &t_dist, Input::Step).unwrap();
    for (idx, &t) in t_dist.iter().enumerate() {
        let exact_wd = (-1.44 / wd) * (-t).exp() * (wd * t).sin();
        assert!(
            (y_dist[idx] - exact_wd).abs() <= TOL_WAVEFORM,
            "Q8 disturbance step waveform error at t={t}: {} != {exact_wd}",
            y_dist[idx]
        );
    }
    assert!(
        y_dist.last().unwrap().abs() <= TOL_STEADY_STATE,
        "Q8 disturbance response at t=15s must settle to 0 within TOL_STEADY_STATE"
    );

    // 6. Q9 Block Diagram Reduction (wn = 1, xi = 0):
    //    Inner loop: (1/s) with feedback 2*xi*wn = 0 -> 1/s
    //    Outer plant: wn^2 * (1/s) * (1/s) = 1/s^2
    let q9_inner = TransferFunction::parse("1/s")
        .unwrap()
        .feedback(&TransferFunction::scalar(0.0), -1.0)
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q9_inner, &[1.0], &[1.0, 0.0], "Q9 inner loop");
    let q9_plant = q9_inner
        .multiply(&TransferFunction::parse("1/s").unwrap())
        .unwrap()
        .normalized();
    assert_tf_coeffs_eq(&q9_plant, &[1.0], &[1.0, 0.0, 0.0], "Q9 double integrator");

    // 7. Q2 Alternative PID Controller (if zero steady-state error e(inf) = 0 is also imposed):
    let q2_pid = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "1/(s^2 + 1)",
        "controller_type": "PID",
        "pid_zeros_mode": "equal",
        "xi": 0.5,
        "ts": 4.0,
        "settling_criterion": "2",
        "dpi": 72
    }))
    .unwrap();
    assert_eq!(q2_pid["success"], true);
    let q2_cp = &q2_pid["details"]["controller_parameters"];
    assert!((q2_cp["kp"].as_f64().unwrap() - 3.737033899038282).abs() <= TOL_GAIN);
    assert!((q2_cp["ki"].as_f64().unwrap() - 1.474067798076564).abs() <= TOL_GAIN);
    assert!((q2_cp["kd"].as_f64().unwrap() - 2.368516949519141).abs() <= TOL_GAIN);
    assert!((q2_cp["z"].as_f64().unwrap() - 0.7888974490720842).abs() <= TOL_GAIN);
    assert!(q2_cp["e_novo"].as_f64().unwrap().abs() <= TOL_STEADY_STATE);
}

#[test]
fn validate_numerical_robustness_repeatability_and_definitive_settling_time() {
    // 1. Repeatability across multiple executions (Etapa 04):
    let payload = json!({
        "action": "controller_design",
        "plant_expr": "-7.2/(5*s + 1)",
        "controller_type": "PI",
        "ts": 4.0,
        "mp": 25.0,
        "settling_criterion": "2",
        "dpi": 72
    });
    let first = controllers::analyze(&payload).unwrap();
    for _ in 0..5 {
        let next = controllers::analyze(&payload).unwrap();
        assert_eq!(
            first["details"]["controller_parameters"],
            next["details"]["controller_parameters"]
        );
        assert_eq!(first["details"]["after"], next["details"]["after"]);
        assert_eq!(
            first["details"]["closed_loop_poles_after"],
            next["details"]["closed_loop_poles_after"]
        );
    }

    // 2. Definitive 2% settling-time entry criterion (Etapa 04):
    //    For underdamped 2nd-order system T(s) = 4/(s^2 + 2s + 4), y(t) oscillates through
    //    the [0.98, 1.02] band multiple times before entering it definitively at t ~ 4.038s.
    //    First crossing of 0.98 occurs during the initial rise at t ~ 1.09s; settling time
    //    must NEVER report the first crossing.
    let tr_res = time_response::analyze(&json!({
        "expr": "4/(s^2 + 2*s + 4)",
        "settling_threshold": 0.02,
        "points": 4000,
        "dpi": 72
    }))
    .unwrap();
    let ts_2pct = tr_res["details"]["step"]["settling_time"].as_f64().unwrap();
    assert!(
        (ts_2pct - 4.037934).abs() <= TOL_SETTLING_S,
        "Settling time {ts_2pct}s must be definitive entry (~4.038s), not first crossing (~1.09s)"
    );

    // 3. Sensitivity to small perturbations on well-conditioned polynomial roots:
    let base_roots = poly::roots(&[1.0, 6.0, 10.0, 8.0]).unwrap();
    let pert_roots = poly::roots(&[1.0, 6.0, 10.0, 8.0 + 1e-9]).unwrap();
    let sens = unordered_complex_distance(&base_roots, &pert_roots);
    assert!(
        sens <= 1e-8,
        "Excessive root sensitivity to 1e-9 perturbation: {sens:e}"
    );
}
