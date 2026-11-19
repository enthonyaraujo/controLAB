//! Etapa 10 — Scientific compliance audit, requirement verification, independent Q8 reference,
//! and generic constrained controller synthesis regression tests.
//!
//! Validates that:
//! 1. Numerically accurate LGR controllers are explicitly flagged as `REQUIREMENTS_FAIL` when
//!    real closed-loop metrics violate requested specifications (Q2 PD/PID, Q5 literal ts=1s,
//!    Q7 literal Mp<=16%, Q8 classic PI Mp=33.84%, Q9 classic PD Mp=29.84%).
//! 2. Unstable systems and internally unstable RHP pole-zero cancellations are never classified
//!    as successful designs.
//! 3. Non-dominant poles, approximate cancellations, extra zeros, non-unity feedback tracking
//!    error, and didactic specification conflicts (xi=0.5 vs Mp<=16%) are explicitly flagged.
//! 4. The independent Q8 reference solution (Kp = -1.0, Ki = -0.2 => T(s) = 1.44/(s+1.44))
//!    is verified nominally, for disturbance rejection, and under +/-10% plant perturbations.
//! 5. Generic constrained numerical refinement (`temporal_refined`) and automatic simplest
//!    controller selection (`auto_simplest`) find strictly compliant controllers for Q2, Q5,
//!    Q7, Q8, and Q9 without relaxing tolerances or hardcoding any exercise.

use controlab_engine::{
    controller_synthesis::{
        ControllerStructure, DampingSpec, DesignObjective, DesignSpecs, FeasibilityClass,
        RequirementStatus, SettlingSpec, default_settling_tolerance, synthesize_for_structure,
        synthesize_simplest, verify_controller,
    },
    controllers,
    simulation::{self, Input},
    time_response,
    transfer::TransferFunction,
};
use serde_json::json;

#[test]
fn numerically_correct_classic_lgr_controllers_fail_requirements_when_specs_violated() {
    // 1. Q2 classic PD: G(s) = 1/(s^2+1), Kp=3, Kd=2 -> Mp = 41.25% > 16.3%, ts = 3.72s != 4s
    let q2_pd = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "1/(s^2 + 1)",
        "controller_type": "PD",
        "mp": 16.3,
        "xi": 0.5,
        "ts": 4.0,
        "ts_constraint": "equal",
        "ts_tolerance": 0.1,
        "settling_criterion": "2",
        "synthesis_mode": "lgr_classic",
        "verify_requirements": true,
        "dpi": 72
    }))
    .unwrap();
    let v = &q2_pd["details"]["verification"];
    assert_eq!(v["numerically_valid"], true);
    assert_eq!(v["closed_loop_stable"], true);
    assert_eq!(v["requirements_passed"], false);
    assert_eq!(v["overall_status"], "FAIL");
    assert!(v["measured"]["overshoot_percent"].as_f64().unwrap() > 40.0);
    assert!(
        v["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("zero(s) de malha fechada"))
    );

    // 2. Q2 classic PID (equal zeros): Mp = 13.99% (MATLAB: 13.97%), but ts = 7.64s != 4s
    //    (MATLAB: 7.639s) due to slow real pole at s = -0.3685 destroying dominance.
    let q2_pid = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "1/(s^2 + 1)",
        "controller_type": "PID",
        "pid_zeros_mode": "equal",
        "mp": 16.3,
        "xi": 0.5,
        "ts": 4.0,
        "ts_constraint": "equal",
        "ts_tolerance": 0.1,
        "require_zero_error": true,
        "settling_criterion": "2",
        "synthesis_mode": "lgr_classic",
        "verify_requirements": true,
        "dpi": 72
    }))
    .unwrap();
    let v_pid = &q2_pid["details"]["verification"];
    assert_eq!(v_pid["requirements_passed"], false);
    assert_eq!(v_pid["overall_status"], "FAIL");
    let pid_mp = v_pid["measured"]["overshoot_percent"].as_f64().unwrap();
    let pid_ts = v_pid["measured"]["settling_time"].as_f64().unwrap();
    assert!(
        (pid_mp - 13.985).abs() < 0.05,
        "unexpected Q2 PID Mp={pid_mp}"
    );
    assert!(
        (pid_ts - 7.639).abs() < 0.05,
        "unexpected Q2 PID ts={pid_ts}"
    );
    assert!(
        v_pid["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("Dominância comprometida"))
    );

    // 3. Q7 classic PD: G(s) = 5/(s*(s+2)^2), xi=0.5 gives Mp = 16.303% > literal 16.0% and ts = 4.038s
    let q7_pd = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "5/(s*(s + 2)^2)",
        "controller_type": "PD",
        "mp": 16.0,
        "xi": 0.5,
        "ts": 4.0,
        "ts_constraint": "equal",
        "ts_tolerance": 0.05,
        "require_zero_error": true,
        "settling_criterion": "2",
        "synthesis_mode": "lgr_classic",
        "verify_requirements": true,
        "dpi": 72
    }))
    .unwrap();
    let v7 = &q7_pd["details"]["verification"];
    assert_eq!(v7["requirements_passed"], false);
    assert_eq!(v7["overall_status"], "FAIL");
    assert!(
        v7["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("Incompatibilidade didática"))
    );

    // 4. Q8 classic PI: G(s) = -7.2/(5s+1), Kp=-1.25, Ki=-4.2608 -> Mp = 33.84% > 25%
    let q8_pi = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "-7.2/(5*s + 1)",
        "controller_type": "PI",
        "mp": 25.0,
        "ts": 4.0,
        "ts_constraint": "at_most",
        "require_zero_error": true,
        "settling_criterion": "2",
        "synthesis_mode": "lgr_classic",
        "verify_requirements": true,
        "dpi": 72
    }))
    .unwrap();
    let v8 = &q8_pi["details"]["verification"];
    assert_eq!(v8["requirements_passed"], false);
    assert_eq!(v8["overall_status"], "FAIL");
    assert!(v8["measured"]["overshoot_percent"].as_f64().unwrap() > 33.0);

    // 5. Q9 classic PD: G(s) = 1/s^2, Kp=4, Kd=2 -> Mp = 29.84% > 16%
    let q9_pd = controllers::analyze(&json!({
        "action": "controller_design",
        "plant_expr": "1/(s^2)",
        "controller_type": "PD",
        "mp": 16.0,
        "xi": 0.5,
        "ts": 4.0,
        "ts_constraint": "at_most",
        "require_zero_error": true,
        "settling_criterion": "2",
        "synthesis_mode": "lgr_classic",
        "verify_requirements": true,
        "dpi": 72
    }))
    .unwrap();
    let v9 = &q9_pd["details"]["verification"];
    assert_eq!(v9["requirements_passed"], false);
    assert_eq!(v9["overall_status"], "FAIL");
    assert!(v9["measured"]["overshoot_percent"].as_f64().unwrap() > 29.0);
}

#[test]
fn unstable_systems_and_rhp_cancellations_are_never_classified_as_compliant() {
    let specs = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(25.0),
        settling_time: Some(SettlingSpec::AtMost(4.0)),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };

    // 1. Positive gain on negative-gain plant G(s) = -7.2/(5s+1) -> unstable closed loop!
    let plant_neg = TransferFunction::parse("-7.2/(5*s + 1)").unwrap();
    let bad_gc = TransferFunction::new(vec![1.0, 0.2], vec![1.0, 0.0]).unwrap();
    let rep_unstable = verify_controller(&plant_neg, None, &bad_gc, &specs).unwrap();
    assert!(!rep_unstable.closed_loop_stable);
    assert!(!rep_unstable.requirements_passed);
    assert_eq!(rep_unstable.overall_status, RequirementStatus::Fail);

    // 2. Unstable RHP pole-zero cancellation: G(s) = 1/(s - 1), Gc(s) = 5*(s - 1)/(s + 2)
    //    Unreduced characteristic polynomial has hidden unstable root at s = +1!
    let plant_rhp = TransferFunction::parse("1/(s - 1)").unwrap();
    let cancel_rhp_gc = TransferFunction::parse("5*(s - 1)/(s + 2)").unwrap();
    let rep_cancel = verify_controller(&plant_rhp, None, &cancel_rhp_gc, &specs).unwrap();
    assert!(!rep_cancel.internally_stable);
    assert!(!rep_cancel.requirements_passed);
    assert_eq!(rep_cancel.overall_status, RequirementStatus::Fail);
}

#[test]
fn validate_independent_q8_reference_solution_disturbance_and_robustness() {
    // Section 4 Independent Reference Case:
    // G(s) = -7.2 / (5s + 1) = -1.44 / (s + 0.2)
    // Gc(s) = -1 - 0.2/s = -(s + 0.2) / s  (Kp = -1.0, Ki = -0.2)
    // Reduced closed-loop reference transfer function: T(s) = 1.44 / (s + 1.44)
    let plant = TransferFunction::parse("-7.2/(5*s + 1)").unwrap();
    let gc_ref = TransferFunction::new(vec![-1.0, -0.2], vec![1.0, 0.0]).unwrap();
    let specs = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(25.0),
        settling_time: Some(SettlingSpec::AtMost(4.0)),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };

    let rep = verify_controller(&plant, None, &gc_ref, &specs).unwrap();
    assert!(
        rep.closed_loop_stable,
        "Nominal Q8 reference must be stable"
    );
    assert!(
        rep.internally_stable,
        "Nominal Q8 reference cancels stable LHP pole -0.2, so internally stable"
    );
    assert!(
        rep.requirements_passed,
        "Nominal Q8 reference must satisfy all Q8 specs: failed={:?}",
        rep.failed_requirements
    );
    assert!(
        rep.measured_overshoot_pct.unwrap() <= 1e-6,
        "Expected 0% overshoot, got {:?}",
        rep.measured_overshoot_pct
    );
    let exact_ts = 50.0_f64.ln() / 1.44; // 2.7166826529... s
    assert!(
        (rep.measured_settling_time_s.unwrap() - exact_ts).abs() <= 0.01,
        "Expected ts(2%) ~ {exact_ts:.5}s, got {:?}",
        rep.measured_settling_time_s
    );
    assert!((rep.measured_steady_state_y.unwrap() - 1.0).abs() <= 1e-6);
    assert!(rep.summer_steady_state_error.unwrap().abs() <= 1e-6);
    // Must emit nominal pole-zero cancellation warning so sensitivity is transparent!
    assert!(
        rep.cancellations
            .iter()
            .any(|c| (c.pole.re + 0.2).abs() < 1e-5 && c.exact && !c.unstable)
    );

    // Cross-check in time_response module for reduced T(s) = 1.44 / (s + 1.44):
    let tr = time_response::analyze(&json!({
        "expr": "1.44 / (s + 1.44)",
        "settling_threshold": 0.02,
        "points": 4000,
        "dpi": 72
    }))
    .unwrap();
    assert!(tr["details"]["step"]["overshoot_percent"].as_f64().unwrap() <= 1e-6);
    assert!((tr["details"]["step"]["settling_time"].as_f64().unwrap() - exact_ts).abs() <= 0.02);

    // Disturbance rejection verification:
    // T_d(s) = G(s) / (1 + Gc(s)G(s)) = (-1.44*s) / ((s + 0.2)*(s + 1.44))
    let td = plant.feedback(&gc_ref, -1.0).unwrap().normalized();
    assert!(
        td.dc_gain().abs() <= 1e-12,
        "Zero steady-state disturbance error"
    );
    let t_dist: Vec<f64> = (0..=4000).map(|i| i as f64 * 0.01).collect();
    let y_dist = simulation::simulate(&td, &t_dist, Input::Step).unwrap();
    // Exact closed-form disturbance step response:
    // L^-1{ -1.44 / ((s + 0.2)(s + 1.44)) } = (-1.44 / 1.24) * (e^(-0.2t) - e^(-1.44t))
    for (i, &t) in t_dist.iter().enumerate() {
        let exact_wd = (-1.44 / 1.24) * ((-0.2 * t).exp() - (-1.44 * t).exp());
        assert!(
            (y_dist[i] - exact_wd).abs() <= 1e-6,
            "Q8 reference disturbance waveform mismatch at t={t}: {} != {exact_wd}",
            y_dist[i]
        );
    }
    assert!(y_dist.last().unwrap().abs() <= 5e-4);

    // Robustness under +/-10% plant parameter perturbations (tau in {4.5, 5.5}, k in {-6.48, -7.92}):
    // Verifies internal stability and non-fragility of the stable pole cancellation at s = -0.2.
    for &(tau_p, k_p) in &[(4.5, -7.2), (5.5, -7.2), (5.0, -6.48), (5.0, -7.92)] {
        let pert_plant = TransferFunction::new(vec![k_p], vec![tau_p, 1.0]).unwrap();
        let pert_rep = verify_controller(&pert_plant, None, &gc_ref, &specs).unwrap();
        assert!(
            pert_rep.closed_loop_stable && pert_rep.internally_stable,
            "Perturbed Q8 plant (tau={tau_p}, k={k_p}) must remain internally stable"
        );
        assert!(
            pert_rep.measured_overshoot_pct.unwrap() <= 1.0,
            "Perturbed Q8 overshoot should stay near 0%, got {:?}",
            pert_rep.measured_overshoot_pct
        );
        assert!(pert_rep.summer_steady_state_error.unwrap().abs() <= 1e-6);
    }
}

#[test]
fn generic_constrained_synthesis_and_auto_simplest_resolve_q2_q5_q7_q8_q9() {
    // 1. Q2: G(s) = 1/(s^2 + 1), Mp <= 16.3%, ts(2%) = 4s (+/- 0.1s)
    let g2 = TransferFunction::parse("1/(s^2 + 1)").unwrap();
    let specs_q2 = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(16.3),
        settling_time: Some(SettlingSpec::Equal {
            target: 4.0,
            tolerance: 0.1,
        }),
        settling_criterion_pct: 2,
        require_zero_step_error: false,
        ..Default::default()
    };
    let out_q2_pd =
        synthesize_for_structure(&g2, None, ControllerStructure::PD, &specs_q2).unwrap();
    assert!(
        out_q2_pd.feasible,
        "Q2 refined PD must satisfy Mp<=16.3% and ts=4s"
    );
    let c2 = out_q2_pd.selected.unwrap();
    assert!(c2.verification.requirements_passed);
    assert!(c2.verification.measured_overshoot_pct.unwrap() <= 16.3);
    assert!((c2.verification.measured_settling_time_s.unwrap() - 4.0).abs() <= 0.1);

    // Q2 with e(inf) = 0 -> PID refined
    let mut specs_q2_zero = specs_q2.clone();
    specs_q2_zero.require_zero_step_error = true;
    let out_q2_pid =
        synthesize_for_structure(&g2, None, ControllerStructure::PID, &specs_q2_zero).unwrap();
    assert!(
        out_q2_pid.feasible,
        "Q2 refined PID must satisfy Mp<=16.3%, ts=4s, ess=0"
    );
    let c2_pid = out_q2_pid.selected.unwrap();
    assert!(c2_pid.verification.requirements_passed);
    assert!(c2_pid.verification.summer_steady_state_error.unwrap().abs() <= 1e-5);

    // 2. Q5: G(s) = 1/(s^3 + 10s^2 + 17s + 8), literal ts(2%) = 1s (+/- 0.05s), Mp <= 25.38%, ess = 0
    let g5 = TransferFunction::parse("1/(s^3 + 10*s^2 + 17*s + 8)").unwrap();
    let specs_q5 = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(25.38),
        settling_time: Some(SettlingSpec::Equal {
            target: 1.0,
            tolerance: 0.05,
        }),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let out_q5 = synthesize_for_structure(&g5, None, ControllerStructure::PID, &specs_q5).unwrap();
    assert!(
        out_q5.feasible,
        "Q5 refined PID must satisfy literal ts=1s, Mp<=25.38%, ess=0"
    );
    let c5 = out_q5.selected.unwrap();
    assert!(c5.verification.requirements_passed);
    assert!((c5.verification.measured_settling_time_s.unwrap() - 1.0).abs() <= 0.05);

    // 3. Q7: G(s) = 5/(s*(s+2)^2), literal Mp <= 16.0%, ts(2%) = 4s (+/- 0.05s), ess = 0
    let g7 = TransferFunction::parse("5/(s*(s + 2)^2)").unwrap();
    let specs_q7 = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(16.0),
        settling_time: Some(SettlingSpec::Equal {
            target: 4.0,
            tolerance: 0.05,
        }),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let out_q7 = synthesize_for_structure(&g7, None, ControllerStructure::PD, &specs_q7).unwrap();
    assert!(
        out_q7.feasible,
        "Q7 refined PD must satisfy literal Mp<=16% and ts=4s"
    );
    let c7 = out_q7.selected.unwrap();
    assert!(c7.verification.requirements_passed);
    assert!(c7.verification.measured_overshoot_pct.unwrap() <= 16.0);
    assert!((c7.verification.measured_settling_time_s.unwrap() - 4.0).abs() <= 0.05);

    // 4. Q8: G(s) = -7.2/(5s+1), Mp <= 25%, ts(2%) <= 4s, ess = 0 -> auto_simplest selects PI!
    let g8 = TransferFunction::parse("-7.2/(5*s + 1)").unwrap();
    let specs_q8 = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(25.0),
        settling_time: Some(SettlingSpec::AtMost(4.0)),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let out_q8 = synthesize_simplest(&g8, None, &specs_q8).unwrap();
    assert!(out_q8.feasible);
    assert!(
        out_q8
            .infeasibility_reasons
            .iter()
            .any(|r| r.contains("Tipo 0")),
        "P must be rejected algebraically on Type 0 plant"
    );
    let c8 = out_q8.selected.unwrap();
    assert_eq!(c8.structure, ControllerStructure::PI);
    assert!(
        c8.kp < 0.0 && c8.ki < 0.0,
        "Negative plant gain requires negative PI gains"
    );
    assert!(c8.verification.requirements_passed);
    assert!(c8.verification.measured_overshoot_pct.unwrap() <= 25.0);
    assert!(c8.verification.measured_settling_time_s.unwrap() <= 4.0);
    assert!(c8.verification.summer_steady_state_error.unwrap().abs() <= 1e-5);

    // 5. Q9: G(s) = 1/s^2, Mp <= 16%, ts(2%) <= 4s, ess = 0 -> auto_simplest selects PD!
    let g9 = TransferFunction::parse("1/(s^2)").unwrap();
    let specs_q9 = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(16.0),
        settling_time: Some(SettlingSpec::AtMost(4.0)),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let out_q9 = synthesize_simplest(&g9, None, &specs_q9).unwrap();
    assert!(out_q9.feasible);
    assert!(
        out_q9
            .infeasibility_reasons
            .iter()
            .any(|r| r.contains("Tipo 2")),
        "P and PI must be rejected algebraically on double integrator"
    );
    let c9 = out_q9.selected.unwrap();
    assert_eq!(c9.structure, ControllerStructure::PD);
    assert!(c9.verification.requirements_passed);
    assert!(c9.verification.measured_overshoot_pct.unwrap() <= 16.0);
    assert!(c9.verification.measured_settling_time_s.unwrap() <= 4.0);
    assert!(c9.verification.summer_steady_state_error.unwrap().abs() <= 1e-5);
}

#[test]
fn etapa11_damping_ratio_semantics_q5_exact_xi_and_proven_infeasibility() {
    // 0. Uniform settling-time equality tolerance consistency (+/- 2%, floor 0.02 s)
    assert!((default_settling_tolerance(1.0) - 0.02).abs() < 1e-12);
    assert!((default_settling_tolerance(4.0) - 0.08).abs() < 1e-12);

    // 1. Verify that Etapa 10's Q5 PID (xi = 0.6669) is REJECTED when mandatory xi = 0.4 +/- 0.01 is enforced!
    let g5 = TransferFunction::parse("1/(s^3 + 10*s^2 + 17*s + 8)").unwrap();
    let gc5_e10 = TransferFunction::new(
        vec![35.985636144021534, 71.88638085895549, 35.900742386408645],
        vec![1.0, 0.0],
    )
    .unwrap();
    let q5_strict_specs = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        damping: Some(DampingSpec::Equal {
            target: 0.4,
            tolerance: 0.01,
        }),
        settling_time: Some(SettlingSpec::Equal {
            target: 1.0,
            tolerance: default_settling_tolerance(1.0),
        }),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let rep_e10 = verify_controller(&g5, None, &gc5_e10, &q5_strict_specs).unwrap();
    assert!(!rep_e10.requirements_passed);
    assert_eq!(rep_e10.overall_status, RequirementStatus::Fail);
    assert!(
        (rep_e10.measured_xi.unwrap() - 0.666854).abs() < 1e-3,
        "Etapa 10 Q5 PID had xi = 0.666854, got {:?}",
        rep_e10.measured_xi
    );

    // 2. Q5 PID synthesized with mandatory exact xi = 0.4 +/- 0.01 AND ts(2%) = 1.0 +/- 0.02 s AND ess = 0!
    let out_q5 =
        synthesize_for_structure(&g5, None, ControllerStructure::PID, &q5_strict_specs).unwrap();
    assert!(
        out_q5.feasible,
        "Generic PID synthesis must find Q5 solution with exact xi=0.4 and ts=1.0s"
    );
    assert_eq!(out_q5.feasibility_class, FeasibilityClass::Feasible);
    let c5 = out_q5.selected.unwrap();
    assert!(c5.verification.requirements_passed);
    assert!(c5.verification.internally_stable);
    assert!(c5.verification.has_complex_pair);
    assert!(
        (c5.verification.measured_xi.unwrap() - 0.4).abs() <= 0.01,
        "Q5 exact xi violated: {:?}",
        c5.verification.measured_xi
    );
    assert!(
        (c5.verification.measured_settling_time_s.unwrap() - 1.0).abs() <= 0.02,
        "Q5 exact ts violated: {:?}",
        c5.verification.measured_settling_time_s
    );
    assert!(c5.verification.summer_steady_state_error.unwrap().abs() <= 1e-5);
    assert!(
        c5.verification.non_dominant_residue_ratio.unwrap() < 0.25,
        "Q5 dominant complex pair must carry >80% of transient modal residue"
    );

    // 3. Q2 PID with mandatory exact xi = 0.5 +/- 0.01, Mp <= 16.3%, ts = 4.0 +/- 0.08 s, ess = 0!
    let g2 = TransferFunction::parse("1/(s^2 + 1)").unwrap();
    let q2_pid_exact = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(16.3),
        damping: Some(DampingSpec::Equal {
            target: 0.5,
            tolerance: 0.01,
        }),
        settling_time: Some(SettlingSpec::Equal {
            target: 4.0,
            tolerance: default_settling_tolerance(4.0),
        }),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let out_q2_pid =
        synthesize_for_structure(&g2, None, ControllerStructure::PID, &q2_pid_exact).unwrap();
    assert!(
        out_q2_pid.feasible,
        "Generic PID synthesis must find Q2 PID solution with exact xi=0.5, Mp<=16.3%, ts=4s, ess=0"
    );
    let c2_pid = out_q2_pid.selected.unwrap();
    assert!(c2_pid.verification.requirements_passed);
    assert!(c2_pid.verification.complex_pair_is_dominant);
    assert!((c2_pid.verification.measured_xi.unwrap() - 0.5).abs() <= 0.01);
    assert!(c2_pid.verification.measured_overshoot_pct.unwrap() <= 16.3);
    assert!((c2_pid.verification.measured_settling_time_s.unwrap() - 4.0).abs() <= 0.08);

    // 4. Q2 PD and Q9 PD with mandatory exact xi = 0.5 +/- 0.01 + Mp <= 16.3% / 16.0%:
    //    MUST be classified as mathematically ProvenInfeasible (time-scale invariance proof)!
    let q2_pd_exact = DesignSpecs {
        require_zero_step_error: false,
        ..q2_pid_exact.clone()
    };
    let out_q2_pd =
        synthesize_for_structure(&g2, None, ControllerStructure::PD, &q2_pd_exact).unwrap();
    assert!(!out_q2_pd.feasible);
    assert_eq!(
        out_q2_pd.feasibility_class,
        FeasibilityClass::ProvenInfeasible
    );
    assert!(
        out_q2_pd
            .infeasibility_reasons
            .iter()
            .any(|r| r.contains("y(∞) = α = 1 -")),
        "Q2 PD proof must explicitly account for steady-state gain y(inf) = 1 - c/wn^2"
    );

    let g9 = TransferFunction::parse("1/(s^2)").unwrap();
    let q9_pd_exact = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(16.0),
        damping: Some(DampingSpec::Equal {
            target: 0.5,
            tolerance: 0.01,
        }),
        settling_time: Some(SettlingSpec::AtMost(4.0)),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let out_q9_pd =
        synthesize_for_structure(&g9, None, ControllerStructure::PD, &q9_pd_exact).unwrap();
    assert!(!out_q9_pd.feasible);
    assert_eq!(
        out_q9_pd.feasibility_class,
        FeasibilityClass::ProvenInfeasible
    );
    assert!(
        out_q9_pd
            .infeasibility_reasons
            .iter()
            .any(|r| r.contains("duplo integrador")),
        "Q9 PD proof must explicitly apply to the double integrator 1/s^2"
    );

    // 5. Q7 PD with mandatory xi = 0.50 +/- 0.01, Mp <= 16.0%, ts = 4.0 +/- 0.08 s, dominant complex pair
    let g7 = TransferFunction::parse("5/(s*(s + 2)^2)").unwrap();
    let q7_pd_exact = DesignSpecs {
        objective: DesignObjective::TimeDomainConstraints,
        max_overshoot_pct: Some(16.0),
        damping: Some(DampingSpec::Equal {
            target: 0.5,
            tolerance: 0.01,
        }),
        require_complex_dominance: true,
        settling_time: Some(SettlingSpec::Equal {
            target: 4.0,
            tolerance: default_settling_tolerance(4.0),
        }),
        settling_criterion_pct: 2,
        require_zero_step_error: true,
        ..Default::default()
    };
    let out_q7_pd =
        synthesize_for_structure(&g7, None, ControllerStructure::PD, &q7_pd_exact).unwrap();
    assert!(out_q7_pd.feasible);
    let c7 = out_q7_pd.selected.unwrap();
    assert!(c7.verification.requirements_passed);
    assert!(c7.verification.complex_pair_is_dominant);
    assert!((c7.verification.measured_xi.unwrap() - 0.5).abs() <= 0.01);
    assert!(c7.verification.measured_overshoot_pct.unwrap() <= 16.0);
    assert!((c7.verification.measured_settling_time_s.unwrap() - 4.0).abs() <= 0.08);

    // 6. Tightening Q7 PD tolerance to exact xi = 0.500 +/- 0.001 with dominance must NOT falsely pass
    //    and must classify as NotFoundWithinSearchBounds (never claiming proof without algebraic theorem).
    let q7_pd_ultra_tight = DesignSpecs {
        damping: Some(DampingSpec::Equal {
            target: 0.5,
            tolerance: 0.001,
        }),
        ..q7_pd_exact
    };
    let out_q7_tight =
        synthesize_for_structure(&g7, None, ControllerStructure::PD, &q7_pd_ultra_tight).unwrap();
    assert!(!out_q7_tight.feasible);
    assert_eq!(
        out_q7_tight.feasibility_class,
        FeasibilityClass::NotFoundWithinSearchBounds
    );
}
