//! Scientific and numerical validation suite for Root Locus (Lugar Geometrico das Raizes - LGR)
//! based on UFERSA Control Systems II (PEX0257) academic list `Exercicios_-_LGR.pdf`.
//!
//! Covers all 18 numbered transfer functions (Equations 1 to 18) across Questions 1 to 4
//! and the parametric characteristic equation of Question 5 (Equation 19, with a2 = 0).
//! Preserves duplicate item labels (`2(i)-1` Eq. 12 and `2(i)-2` Eq. 13) and all
//! mathematically equivalent exercises (`1(a)=2(d)`, `1(b)=2(c)`, `1(c)=2(i)-1=3(a)`).
//!
//! Reference sources:
//! - Closed-form analytical derivation (Evans root-locus rules and Routh-Hurwitz criterion).
//! - External numerical reference: MATLAB R2022b (Control System Toolbox 9.13).

use controlab_engine::{lgr, polynomial as poly, routh, transfer::TransferFunction};
use num_complex::Complex64 as C;
use serde_json::{Value, json};

const TOL_COEFF: f64 = 1e-10;
const TOL_SIMPLE_ROOT: f64 = 1e-6;
const TOL_MULT_ROOT: f64 = 1e-4;
const TOL_CENTROID: f64 = 1e-6;
const TOL_ANGLE_DEG: f64 = 1e-3;
const TOL_BREAKAWAY_S: f64 = 1e-5;
const TOL_BREAKAWAY_K: f64 = 1e-4;
const TOL_JW_W: f64 = 1e-3;
const TOL_JW_K: f64 = 1e-2;

#[derive(Clone, Debug)]
struct LgrAcademicCase {
    id: &'static str,
    question_item: &'static str,
    equation_number: u32,
    g_expr: &'static str,
    h_expr: &'static str,
    ol_expr: &'static str,
    expected_num: &'static [f64],
    expected_den: &'static [f64],
    expected_poles: &'static [(f64, f64)],
    expected_zeros: &'static [(f64, f64)],
    expected_branches: usize,
    /// Midpoint test points (s_real, belongs_to_lgr) verifying real-axis LGR segments.
    real_axis_probes: &'static [(f64, bool)],
    expected_centroid: Option<f64>,
    expected_asymptote_angles_deg: &'static [f64],
    /// Smooth interior breakaway/break-in points (s_real, K_gain) where dK/ds = 0 and K > 0.
    expected_breakaway_points: &'static [(f64, f64)],
    /// Multiple open-loop poles on the real axis that act as K = 0 breakaway points.
    expected_k0_multiple_pole_breakaways: &'static [f64],
    /// Departure angles from upper-half-plane complex poles: (pole_re, pole_im, angle_deg).
    expected_departure_angles: &'static [(f64, f64, f64)],
    /// Arrival angles at upper-half-plane complex zeros: (zero_re, zero_im, angle_deg).
    expected_arrival_angles: &'static [(f64, f64, f64)],
    /// Upper-half-plane jw crossings: (omega_pos, k_crit).
    expected_jw_crossings: &'static [(f64, f64)],
    /// Expected positive-gain stability interval label in Routh-Hurwitz analysis.
    expected_positive_k_stability: &'static str,
    /// Sample gains (k_sample, expected_ strictly_stable) for closed-loop stability verification.
    stability_probes: &'static [(f64, bool)],
    root_tolerance: f64,
    justification: &'static str,
}

fn c(re: f64, im: f64) -> C {
    C::new(re, im)
}

/// Unordered Hausdorff / greedy optimal matching distance between two complex root sets.
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

/// Normalized polynomial residual for P(s) = D(s) + K * N(s) = 0:
/// |P(s)| / (||P||_1 * max(1, |s|)^deg)
fn normalized_char_poly_residual(den: &[f64], num: &[f64], k: f64, root: C) -> f64 {
    let char_poly = poly::add(den, num, k);
    let val = poly::evaluate(&char_poly, root).norm();
    let l1_norm: f64 = char_poly.iter().map(|c| c.abs()).sum();
    let deg = (char_poly.len().saturating_sub(1)) as i32;
    let scale = l1_norm * root.norm().max(1.0).powi(deg);
    val / scale.max(1e-15)
}

/// Verify complex conjugate symmetry of a root set.
fn assert_conjugate_symmetry(roots: &[C], tol: f64, context: &str) {
    let conjugates: Vec<C> = roots.iter().map(|r| r.conj()).collect();
    let dist = unordered_complex_distance(roots, &conjugates);
    assert!(
        dist <= tol,
        "{context}: conjugate symmetry violated, max distance = {dist:e} > {tol:e}"
    );
}

fn wrap_angle_180(deg: f64) -> f64 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

fn all_lgr_academic_cases() -> Vec<LgrAcademicCase> {
    let sqrt2 = 2.0_f64.sqrt();
    let sqrt3 = 3.0_f64.sqrt();
    let sqrt5 = 5.0_f64.sqrt();
    let sqrt7 = 7.0_f64.sqrt();
    let _ = (sqrt2, sqrt3, sqrt5, sqrt7);
    const SQRT2: f64 = std::f64::consts::SQRT_2;
    const SQRT3: f64 = 1.7320508075688772;
    const SQRT5: f64 = 2.23606797749979;
    const SQRT7: f64 = 2.6457513110645907;
    const SQRT2_6: f64 = 1.6124515496597098;
    const SQRT16_5: f64 = 4.06201920231798;
    const SQRT2665: f64 = 51.62363799656123;

    vec![
        // =========================================================================
        // Q1(a) - Eq. (1): G(s)H(s) = k / (s(s^2 + 6s + 25))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q1A_EQ01",
            question_item: "1(a)",
            equation_number: 1,
            g_expr: "1/(s*(s^2 + 6*s + 25))",
            h_expr: "1",
            ol_expr: "1/(s*(s^2 + 6*s + 25))",
            expected_num: &[1.0],
            expected_den: &[1.0, 6.0, 25.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-3.0, 4.0), (-3.0, -4.0)],
            expected_zeros: &[],
            expected_branches: 3,
            real_axis_probes: &[(1.0, false), (-1.0, true), (-5.0, true)],
            expected_centroid: Some(-2.0),
            expected_asymptote_angles_deg: &[60.0, 180.0, 300.0],
            expected_breakaway_points: &[],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-3.0, 4.0, -36.86989764584402)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(5.0, 150.0)],
            expected_positive_k_stability: "0 < K < 150",
            stability_probes: &[(1.0, true), (75.0, true), (149.0, true), (151.0, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "3rd-order all-pole system with origin pole and complex conjugate pair at -3 +/- 4j; dK/ds = -(3s^2+12s+25) has complex roots -2 +/- sqrt(13/3)j so no real breakaway; Routh gives K_crit = 6*25 = 150 at w = 5 rad/s.",
        },
        // =========================================================================
        // Q1(b) - Eq. (2): G(s)H(s) = k(s + 2) / (s^2 + 2s + 3)
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q1B_EQ02",
            question_item: "1(b)",
            equation_number: 2,
            g_expr: "(s + 2)/(s^2 + 2*s + 3)",
            h_expr: "1",
            ol_expr: "(s + 2)/(s^2 + 2*s + 3)",
            expected_num: &[1.0, 2.0],
            expected_den: &[1.0, 2.0, 3.0],
            expected_poles: &[(-1.0, SQRT2), (-1.0, -SQRT2)],
            expected_zeros: &[(-2.0, 0.0)],
            expected_branches: 2,
            real_axis_probes: &[(0.0, false), (-1.0, false), (-3.0, true)],
            expected_centroid: Some(0.0),
            expected_asymptote_angles_deg: &[180.0],
            expected_breakaway_points: &[(-2.0 - SQRT3, 2.0 + 2.0 * SQRT3)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-1.0, SQRT2, 144.73561031724535)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(0.5, true), (5.0, true), (100.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Circular root locus centered at s = -2 with radius sqrt(3); break-in at s = -2 - sqrt(3) with K = 2 + 2*sqrt(3); unconditionally stable for all K > 0.",
        },
        // =========================================================================
        // Q1(c) - Eq. (3): G(s)H(s) = k / (s(s^2 + 4s + 5))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q1C_EQ03",
            question_item: "1(c)",
            equation_number: 3,
            g_expr: "1/(s*(s^2 + 4*s + 5))",
            h_expr: "1",
            ol_expr: "1/(s*(s^2 + 4*s + 5))",
            expected_num: &[1.0],
            expected_den: &[1.0, 4.0, 5.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-2.0, 1.0), (-2.0, -1.0)],
            expected_zeros: &[],
            expected_branches: 3,
            real_axis_probes: &[(1.0, false), (-0.5, true), (-2.0, true)],
            expected_centroid: Some(-4.0 / 3.0),
            expected_asymptote_angles_deg: &[60.0, 180.0, 300.0],
            expected_breakaway_points: &[(-5.0 / 3.0, 50.0 / 27.0), (-1.0, 2.0)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-2.0, 1.0, -63.43494882292201)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(SQRT5, 20.0)],
            expected_positive_k_stability: "0 < K < 20",
            stability_probes: &[(1.0, true), (10.0, true), (19.5, true), (20.5, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "3rd-order system where dK/ds = -(3s^2 + 8s + 5) = -(3s+5)(s+1) = 0 gives both a break-in at s = -5/3 (K = 50/27) and a breakaway at s = -1 (K = 2); jw crossing at w = sqrt(5), K = 20.",
        },
        // =========================================================================
        // Q2(a) - Eq. (4): G(s) = k / (s(s + 4))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2A_EQ04",
            question_item: "2(a)",
            equation_number: 4,
            g_expr: "1/(s*(s + 4))",
            h_expr: "1",
            ol_expr: "1/(s*(s + 4))",
            expected_num: &[1.0],
            expected_den: &[1.0, 4.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-4.0, 0.0)],
            expected_zeros: &[],
            expected_branches: 2,
            real_axis_probes: &[(1.0, false), (-2.0, true), (-5.0, false)],
            expected_centroid: Some(-2.0),
            expected_asymptote_angles_deg: &[90.0, 270.0],
            expected_breakaway_points: &[(-2.0, 4.0)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(1.0, true), (4.0, true), (50.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "2nd-order Type 1 system; real segment [-4, 0], breakaway at midpoint s = -2 (K = 4), vertical asymptotes at Re(s) = -2; stable for all K > 0.",
        },
        // =========================================================================
        // Q2(b) - Eq. (5): G(s) = k / (s(s + 1)(s + 2))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2B_EQ05",
            question_item: "2(b)",
            equation_number: 5,
            g_expr: "1/(s*(s + 1)*(s + 2))",
            h_expr: "1",
            ol_expr: "1/(s*(s + 1)*(s + 2))",
            expected_num: &[1.0],
            expected_den: &[1.0, 3.0, 2.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-1.0, 0.0), (-2.0, 0.0)],
            expected_zeros: &[],
            expected_branches: 3,
            real_axis_probes: &[(0.5, false), (-0.5, true), (-1.5, false), (-3.0, true)],
            expected_centroid: Some(-1.0),
            expected_asymptote_angles_deg: &[60.0, 180.0, 300.0],
            expected_breakaway_points: &[(-1.0 + SQRT3 / 3.0, 2.0 * SQRT3 / 9.0)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(SQRT2, 6.0)],
            expected_positive_k_stability: "0 < K < 6",
            stability_probes: &[(0.2, true), (3.0, true), (5.9, true), (6.1, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "3 real poles at 0, -1, -2; dK/ds = -(3s^2+6s+2)=0 yields valid breakaway at s = -1 + sqrt(3)/3 in (-1, 0) with K = 2*sqrt(3)/9; Routh gives K_crit = 3*2 = 6 at w = sqrt(2).",
        },
        // =========================================================================
        // Q2(c) - Eq. (6): G(s) = k(s + 2) / (s^2 + 2s + 3) [Equivalent to 1(b)]
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2C_EQ06",
            question_item: "2(c)",
            equation_number: 6,
            g_expr: "(s + 2)/(s^2 + 2*s + 3)",
            h_expr: "1",
            ol_expr: "(s + 2)/(s^2 + 2*s + 3)",
            expected_num: &[1.0, 2.0],
            expected_den: &[1.0, 2.0, 3.0],
            expected_poles: &[(-1.0, SQRT2), (-1.0, -SQRT2)],
            expected_zeros: &[(-2.0, 0.0)],
            expected_branches: 2,
            real_axis_probes: &[(0.0, false), (-1.0, false), (-3.0, true)],
            expected_centroid: Some(0.0),
            expected_asymptote_angles_deg: &[180.0],
            expected_breakaway_points: &[(-2.0 - SQRT3, 2.0 + 2.0 * SQRT3)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-1.0, SQRT2, 144.73561031724535)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(0.5, true), (5.0, true), (100.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Mathematically equivalent to 1(b) Eq. (2); preserved in validation matrix.",
        },
        // =========================================================================
        // Q2(d) - Eq. (7): G(s) = k / (s(s^2 + 6s + 25)) [Equivalent to 1(a)]
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2D_EQ07",
            question_item: "2(d)",
            equation_number: 7,
            g_expr: "1/(s*(s^2 + 6*s + 25))",
            h_expr: "1",
            ol_expr: "1/(s*(s^2 + 6*s + 25))",
            expected_num: &[1.0],
            expected_den: &[1.0, 6.0, 25.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-3.0, 4.0), (-3.0, -4.0)],
            expected_zeros: &[],
            expected_branches: 3,
            real_axis_probes: &[(1.0, false), (-1.0, true), (-5.0, true)],
            expected_centroid: Some(-2.0),
            expected_asymptote_angles_deg: &[60.0, 180.0, 300.0],
            expected_breakaway_points: &[],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-3.0, 4.0, -36.86989764584402)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(5.0, 150.0)],
            expected_positive_k_stability: "0 < K < 150",
            stability_probes: &[(1.0, true), (75.0, true), (149.0, true), (151.0, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Mathematically equivalent to 1(a) Eq. (1); preserved in validation matrix.",
        },
        // =========================================================================
        // Q2(e) - Eq. (8): G(s) = k(s + 2)(s + 3) / (s(s + 1))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2E_EQ08",
            question_item: "2(e)",
            equation_number: 8,
            g_expr: "((s + 2)*(s + 3))/(s*(s + 1))",
            h_expr: "1",
            ol_expr: "((s + 2)*(s + 3))/(s*(s + 1))",
            expected_num: &[1.0, 5.0, 6.0],
            expected_den: &[1.0, 1.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-1.0, 0.0)],
            expected_zeros: &[(-2.0, 0.0), (-3.0, 0.0)],
            expected_branches: 2,
            real_axis_probes: &[
                (0.5, false),
                (-0.5, true),
                (-1.5, false),
                (-2.5, true),
                (-3.5, false),
            ],
            expected_centroid: None,
            expected_asymptote_angles_deg: &[],
            expected_breakaway_points: &[
                (-(3.0 + SQRT3) / 2.0, 7.0 + 4.0 * SQRT3),
                (-(3.0 - SQRT3) / 2.0, 7.0 - 4.0 * SQRT3),
            ],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(0.05, true), (1.0, true), (20.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Relative degree zero (P = Z = 2, no asymptotes); breakaway in (-1, 0) at s = -(3-sqrt(3))/2 (K = 7-4*sqrt(3)) and break-in in (-3, -2) at s = -(3+sqrt(3))/2 (K = 7+4*sqrt(3)); circular arc between them.",
        },
        // =========================================================================
        // Q2(f) - Eq. (9): G(s) = k(s^2 + 2s + 8) / (s(s + 10)(s + 5))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2F_EQ09",
            question_item: "2(f)",
            equation_number: 9,
            g_expr: "(s^2 + 2*s + 8)/(s*(s + 10)*(s + 5))",
            h_expr: "1",
            ol_expr: "(s^2 + 2*s + 8)/(s*(s + 10)*(s + 5))",
            expected_num: &[1.0, 2.0, 8.0],
            expected_den: &[1.0, 15.0, 50.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-5.0, 0.0), (-10.0, 0.0)],
            expected_zeros: &[(-1.0, SQRT7), (-1.0, -SQRT7)],
            expected_branches: 3,
            real_axis_probes: &[(1.0, false), (-2.0, true), (-7.5, false), (-12.0, true)],
            expected_centroid: Some(-13.0),
            expected_asymptote_angles_deg: &[180.0],
            expected_breakaway_points: &[(-1.667944789809077, 6.218909031131223)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[],
            expected_arrival_angles: &[(-1.0, SQRT7, -109.43118873073795)],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(1.0, true), (10.0, true), (100.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "P=3, Z=2 (complex conjugate zeros at -1 +/- sqrt(7)j); single asymptote at 180 deg with centroid -13; breakaway at s = -1.667945 (K = 6.218909) and arrival angle -109.4312 deg at -1 + sqrt(7)j.",
        },
        // =========================================================================
        // Q2(g) - Eq. (10): G(s) = k(s^2 + 4) / ((s + 2)^2 (s + 5)(s + 6))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2G_EQ10",
            question_item: "2(g)",
            equation_number: 10,
            g_expr: "(s^2 + 4)/(((s + 2)^2)*(s + 5)*(s + 6))",
            h_expr: "1",
            ol_expr: "(s^2 + 4)/(((s + 2)^2)*(s + 5)*(s + 6))",
            expected_num: &[1.0, 0.0, 4.0],
            expected_den: &[1.0, 15.0, 78.0, 164.0, 120.0],
            expected_poles: &[(-2.0, 0.0), (-2.0, 0.0), (-5.0, 0.0), (-6.0, 0.0)],
            expected_zeros: &[(0.0, 2.0), (0.0, -2.0)],
            expected_branches: 4,
            real_axis_probes: &[
                (0.0, false),
                (-1.0, false),
                (-3.5, false),
                (-5.5, true),
                (-7.0, false),
            ],
            expected_centroid: Some(-7.5),
            expected_asymptote_angles_deg: &[90.0, 270.0],
            expected_breakaway_points: &[(-5.530716069857954, 0.08976101605604443)],
            expected_k0_multiple_pole_breakaways: &[-2.0],
            expected_departure_angles: &[],
            expected_arrival_angles: &[(0.0, 2.0, -139.76363520094115)],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(0.05, true), (1.0, true), (100.0, true)],
            root_tolerance: TOL_MULT_ROOT,
            justification: "Repeated pole at s = -2 (multiplicity 2, departing immediately at +/-90 deg at K = 0) and simple poles at -5, -6 (breakaway at s = -5.530716, K = 0.089761); zeros on jw axis at +/-2j reached only as K -> +inf, so strictly stable for all finite K > 0.",
        },
        // =========================================================================
        // Q2(h) - Eq. (11): G(s) = k / ((s + 1)(s^2 + 4s + 5))
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2H_EQ11",
            question_item: "2(h)",
            equation_number: 11,
            g_expr: "1/((s + 1)*(s^2 + 4*s + 5))",
            h_expr: "1",
            ol_expr: "1/((s + 1)*(s^2 + 4*s + 5))",
            expected_num: &[1.0],
            expected_den: &[1.0, 5.0, 9.0, 5.0],
            expected_poles: &[(-1.0, 0.0), (-2.0, 1.0), (-2.0, -1.0)],
            expected_zeros: &[],
            expected_branches: 3,
            real_axis_probes: &[(0.0, false), (-0.5, false), (-2.0, true)],
            expected_centroid: Some(-5.0 / 3.0),
            expected_asymptote_angles_deg: &[60.0, 180.0, 300.0],
            expected_breakaway_points: &[],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-2.0, 1.0, -45.0)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(3.0, 40.0)],
            expected_positive_k_stability: "0 < K < 40",
            stability_probes: &[(1.0, true), (20.0, true), (39.5, true), (40.5, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "3rd-order shifted pole configuration; dK/ds = -(3s^2+10s+9) has negative discriminant (100-108=-8<0), so no real breakaway; Routh gives 5*9 - (5+K) = 0 => K_crit = 40 at w = 3 rad/s.",
        },
        // =========================================================================
        // Q2(i)-1 - Eq. (12): G(s) = k / (s(s^2 + 4s + 5)) [First 2(i) in PDF]
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2I1_EQ12",
            question_item: "2(i)-1",
            equation_number: 12,
            g_expr: "1/(s*(s^2 + 4*s + 5))",
            h_expr: "1",
            ol_expr: "1/(s*(s^2 + 4*s + 5))",
            expected_num: &[1.0],
            expected_den: &[1.0, 4.0, 5.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-2.0, 1.0), (-2.0, -1.0)],
            expected_zeros: &[],
            expected_branches: 3,
            real_axis_probes: &[(1.0, false), (-0.5, true), (-2.0, true)],
            expected_centroid: Some(-4.0 / 3.0),
            expected_asymptote_angles_deg: &[60.0, 180.0, 300.0],
            expected_breakaway_points: &[(-5.0 / 3.0, 50.0 / 27.0), (-1.0, 2.0)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-2.0, 1.0, -63.43494882292201)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(SQRT5, 20.0)],
            expected_positive_k_stability: "0 < K < 20",
            stability_probes: &[(1.0, true), (10.0, true), (19.5, true), (20.5, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "First item labeled 2(i) in PDF (Equation 12), mathematically equivalent to 1(c) Eq. (3) and 3(a) Eq. (14).",
        },
        // =========================================================================
        // Q2(i)-2 - Eq. (13): G(s) = k(s + 2) / ((s + 1)(s^2 + 6s + 10)) [Second 2(i) in PDF]
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q2I2_EQ13",
            question_item: "2(i)-2",
            equation_number: 13,
            g_expr: "(s + 2)/((s + 1)*(s^2 + 6*s + 10))",
            h_expr: "1",
            ol_expr: "(s + 2)/((s + 1)*(s^2 + 6*s + 10))",
            expected_num: &[1.0, 2.0],
            expected_den: &[1.0, 7.0, 16.0, 10.0],
            expected_poles: &[(-1.0, 0.0), (-3.0, 1.0), (-3.0, -1.0)],
            expected_zeros: &[(-2.0, 0.0)],
            expected_branches: 3,
            real_axis_probes: &[(0.0, false), (-1.5, true), (-3.0, false)],
            expected_centroid: Some(-2.5),
            expected_asymptote_angles_deg: &[90.0, 270.0],
            expected_breakaway_points: &[],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-3.0, 1.0, 71.56505117707799)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(1.0, true), (15.0, true), (200.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Second item labeled 2(i) in PDF (Equation 13); real pole at -1 moves directly to zero at -2; complex poles at -3 +/- 1j depart at +71.5651 deg and approach vertical asymptotes at Re(s) = -2.5; stable for all K > 0.",
        },
        // =========================================================================
        // Q3(a) - Eq. (14): G(s) = k / (s(s^2 + 4s + 5)), H(s) = 1 [Equivalent to 1(c)]
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q3A_EQ14",
            question_item: "3(a)",
            equation_number: 14,
            g_expr: "1/(s*(s^2 + 4*s + 5))",
            h_expr: "1",
            ol_expr: "1/(s*(s^2 + 4*s + 5))",
            expected_num: &[1.0],
            expected_den: &[1.0, 4.0, 5.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-2.0, 1.0), (-2.0, -1.0)],
            expected_zeros: &[],
            expected_branches: 3,
            real_axis_probes: &[(1.0, false), (-0.5, true), (-2.0, true)],
            expected_centroid: Some(-4.0 / 3.0),
            expected_asymptote_angles_deg: &[60.0, 180.0, 300.0],
            expected_breakaway_points: &[(-5.0 / 3.0, 50.0 / 27.0), (-1.0, 2.0)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-2.0, 1.0, -63.43494882292201)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(SQRT5, 20.0)],
            expected_positive_k_stability: "0 < K < 20",
            stability_probes: &[(1.0, true), (10.0, true), (19.5, true), (20.5, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Block diagram in Fig. 1 with unity feedback H(s) = 1; equivalent to 1(c) Eq. (3) and 2(i)-1 Eq. (12).",
        },
        // =========================================================================
        // Q3(b) - Eq. (15): G(s) = k / (s(s^2 + 4s + 13)), H(s) = 1 / (s + 1)
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q3B_EQ15",
            question_item: "3(b)",
            equation_number: 15,
            g_expr: "1/(s*(s^2 + 4*s + 13))",
            h_expr: "1/(s + 1)",
            ol_expr: "1/(s*(s + 1)*(s^2 + 4*s + 13))",
            expected_num: &[1.0],
            expected_den: &[1.0, 5.0, 17.0, 13.0, 0.0],
            expected_poles: &[(0.0, 0.0), (-1.0, 0.0), (-2.0, 3.0), (-2.0, -3.0)],
            expected_zeros: &[],
            expected_branches: 4,
            real_axis_probes: &[(0.5, false), (-0.5, true), (-2.0, false)],
            expected_centroid: Some(-1.25),
            expected_asymptote_angles_deg: &[45.0, 135.0, 225.0, 315.0],
            expected_breakaway_points: &[(-0.46637844124531597, 2.825166371537658)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-2.0, 3.0, -142.1250163489018)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(SQRT2_6, 37.44)],
            expected_positive_k_stability: "0 < K < 37.44",
            stability_probes: &[(1.0, true), (20.0, true), (37.0, true), (38.0, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Non-unity feedback H(s) = 1/(s+1) yields 4th-order open-loop G(s)H(s); breakaway in (-1, 0) at s = -0.466378 (K = 2.825166); Routh gives K_crit = 936/25 = 37.44 at w = sqrt(13/5) = sqrt(2.6) rad/s.",
        },
        // =========================================================================
        // Q3(c) - Eq. (16): G(s) = k / (s + 3.6), H(s) = (s + 1) / s^2
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q3C_EQ16",
            question_item: "3(c)",
            equation_number: 16,
            g_expr: "1/(s + 3.6)",
            h_expr: "(s + 1)/(s^2)",
            ol_expr: "(s + 1)/(s^2*(s + 3.6))",
            expected_num: &[1.0, 1.0],
            expected_den: &[1.0, 3.6, 0.0, 0.0],
            expected_poles: &[(0.0, 0.0), (0.0, 0.0), (-3.6, 0.0)],
            expected_zeros: &[(-1.0, 0.0)],
            expected_branches: 3,
            real_axis_probes: &[(0.5, false), (-0.5, false), (-2.0, true), (-5.0, false)],
            expected_centroid: Some(-1.3),
            expected_asymptote_angles_deg: &[90.0, 270.0],
            expected_breakaway_points: &[],
            expected_k0_multiple_pole_breakaways: &[0.0],
            expected_departure_angles: &[],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(0.5, true), (5.0, true), (100.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Non-unity feedback H(s)=(s+1)/s^2 with fractional pole at -3.6 and double pole at origin; double pole at s=0 breaks away immediately into LHP at +/-90 deg at K=0 because zero at -1 is closer than pole at -3.6; Routh first column (1, 3.6, 2.6K/3.6, K) > 0 for all K > 0.",
        },
        // =========================================================================
        // Q4(a) - Eq. (17): G(s) = k(s + 2) / (s^2 + 4s + 13)
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q4A_EQ17",
            question_item: "4(a)",
            equation_number: 17,
            g_expr: "(s + 2)/(s^2 + 4*s + 13)",
            h_expr: "1",
            ol_expr: "(s + 2)/(s^2 + 4*s + 13)",
            expected_num: &[1.0, 2.0],
            expected_den: &[1.0, 4.0, 13.0],
            expected_poles: &[(-2.0, 3.0), (-2.0, -3.0)],
            expected_zeros: &[(-2.0, 0.0)],
            expected_branches: 2,
            real_axis_probes: &[(0.0, false), (-1.0, false), (-3.5, true)],
            expected_centroid: Some(-2.0),
            expected_asymptote_angles_deg: &[180.0],
            expected_breakaway_points: &[(-5.0, 6.0)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-2.0, 3.0, -180.0)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "K > 0",
            stability_probes: &[(1.0, true), (6.0, true), (50.0, true)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Zero at s = -2 lies vertically below/above poles -2 +/- 3j; departure angle is 180 deg (-180 deg), forming a circle of radius 3 centered at -2 with break-in at s = -5 (K = 6); stable for all K > 0.",
        },
        // =========================================================================
        // Q4(b) - Eq. (18): G(s) = k(s - 2)(s - 4) / (s^2 + 6s + 25)
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q4B_EQ18",
            question_item: "4(b)",
            equation_number: 18,
            g_expr: "((s - 2)*(s - 4))/(s^2 + 6*s + 25)",
            h_expr: "1",
            ol_expr: "((s - 2)*(s - 4))/(s^2 + 6*s + 25)",
            expected_num: &[1.0, -6.0, 8.0],
            expected_den: &[1.0, 6.0, 25.0],
            expected_poles: &[(-3.0, 4.0), (-3.0, -4.0)],
            expected_zeros: &[(2.0, 0.0), (4.0, 0.0)],
            expected_branches: 2,
            real_axis_probes: &[(5.0, false), (3.0, true), (0.0, false), (-5.0, false)],
            expected_centroid: None,
            expected_asymptote_angles_deg: &[],
            expected_breakaway_points: &[((-17.0 + SQRT2665) / 12.0, 51.311818998280614)],
            expected_k0_multiple_pole_breakaways: &[],
            expected_departure_angles: &[(-3.0, 4.0, 21.59528822706794)],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[(SQRT16_5, 1.0)],
            expected_positive_k_stability: "0 < K < 1",
            stability_probes: &[(0.2, true), (0.8, true), (1.2, false), (10.0, false)],
            root_tolerance: TOL_SIMPLE_ROOT,
            justification: "Relative degree 0 non-minimum phase system with two RHP zeros at +2 and +4; characteristic polynomial (1+K)s^2 + 6(1-K)s + (25+8K) = 0 crosses jw axis at K = 1, w = sqrt(33/2) = sqrt(16.5) rad/s and breaks into real axis at s = (17+sqrt(2665))/24 in (2, 4) with K = 51.311819.",
        },
        // =========================================================================
        // Q5(a,b) - Eq. (19): s^3 + a2*s^2 + a1*s + a1 = 0 with a2 = 0 => G(s)H(s) = a1(s + 1)/s^3
        // =========================================================================
        LgrAcademicCase {
            id: "LGR_Q5_EQ19",
            question_item: "5(a,b)",
            equation_number: 19,
            g_expr: "(s + 1)/(s^3)",
            h_expr: "1",
            ol_expr: "(s + 1)/(s^3)",
            expected_num: &[1.0, 1.0],
            expected_den: &[1.0, 0.0, 0.0, 0.0],
            expected_poles: &[(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
            expected_zeros: &[(-1.0, 0.0)],
            expected_branches: 3,
            real_axis_probes: &[(0.5, false), (-0.5, true), (-2.0, false)],
            expected_centroid: Some(0.5),
            expected_asymptote_angles_deg: &[90.0, 270.0],
            expected_breakaway_points: &[],
            expected_k0_multiple_pole_breakaways: &[0.0],
            expected_departure_angles: &[],
            expected_arrival_angles: &[],
            expected_jw_crossings: &[],
            expected_positive_k_stability: "\\text{Sem faixa estável}",
            stability_probes: &[(0.1, false), (1.0, false), (10.0, false)],
            root_tolerance: TOL_MULT_ROOT,
            justification: "Parametric characteristic equation s^3 + a2*s^2 + a1*s + a1 = 0 with a2 = 0 and gain K = a1 > 0: equivalent open-loop transfer function G(s)H(s) = (s+1)/s^3; triple pole at s = 0 departs at 60 deg, 180 deg, 300 deg; centroid is sigma_a = +0.5 in RHP, so two branches remain in RHP for all K > 0 (system unstable for all K > 0).",
        },
    ]
}

#[test]
fn validate_all_19_lgr_cases_analytical_and_matlab_reference() {
    let cases = all_lgr_academic_cases();
    assert_eq!(cases.len(), 19, "Must validate all 19 LGR cases");

    for (idx, case) in cases.iter().enumerate() {
        assert!(
            !case.question_item.is_empty(),
            "{}: empty question_item",
            case.id
        );
        assert_eq!(
            case.equation_number as usize,
            idx + 1,
            "{}: equation_number mismatch",
            case.id
        );
        // 1. Verify open-loop transfer function G(s)*H(s) and coefficient normalization.
        let g_tf = TransferFunction::parse(case.g_expr).unwrap();
        let h_tf = TransferFunction::parse(case.h_expr).unwrap();
        let gh_tf = g_tf.multiply(&h_tf).unwrap().normalized();
        let ol_tf = TransferFunction::parse(case.ol_expr).unwrap().normalized();

        assert_eq!(
            gh_tf.num.len(),
            case.expected_num.len(),
            "{} ({}): numerator length mismatch",
            case.id,
            case.justification
        );
        assert_eq!(
            gh_tf.den.len(),
            case.expected_den.len(),
            "{} ({}): denominator length mismatch",
            case.id,
            case.justification
        );
        for (a, e) in gh_tf.num.iter().zip(case.expected_num) {
            assert!(
                (a - e).abs() <= TOL_COEFF,
                "{}: numerator coeff {a} != {e}",
                case.id
            );
        }
        for (a, e) in gh_tf.den.iter().zip(case.expected_den) {
            assert!(
                (a - e).abs() <= TOL_COEFF,
                "{}: denominator coeff {a} != {e}",
                case.id
            );
        }
        for (a, e) in ol_tf.num.iter().zip(case.expected_num) {
            assert!((a - e).abs() <= TOL_COEFF, "{}: ol num {a} != {e}", case.id);
        }
        for (a, e) in ol_tf.den.iter().zip(case.expected_den) {
            assert!((a - e).abs() <= TOL_COEFF, "{}: ol den {a} != {e}", case.id);
        }

        // 2. Run ControLAB LGR engine analysis.
        let dpi = if idx == 0 { 140 } else { 72 };
        let res = lgr::analyze(&json!({"expr": case.ol_expr, "dpi": dpi})).unwrap();
        assert_eq!(res["success"], true, "{}: LGR analysis failed", case.id);
        assert!(
            res["svg"].as_str().unwrap().contains("<svg"),
            "{}: missing SVG output",
            case.id
        );
        assert!(
            res["image"]
                .as_str()
                .unwrap()
                .starts_with("data:image/png;base64,iVBOR"),
            "{}: missing PNG output",
            case.id
        );
        let det = &res["detalhes"];

        // 3. Verify branch count, P, Z.
        assert_eq!(
            det["P"].as_u64().unwrap() as usize,
            case.expected_poles.len(),
            "{}: pole count P",
            case.id
        );
        assert_eq!(
            det["Z"].as_u64().unwrap() as usize,
            case.expected_zeros.len(),
            "{}: zero count Z",
            case.id
        );
        assert_eq!(
            det["ramos"].as_u64().unwrap() as usize,
            case.expected_branches,
            "{}: branch count",
            case.id
        );

        // 4. Verify open-loop poles and zeros (unordered set comparison).
        let actual_poles: Vec<C> = det["polos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| c(p["re"].as_f64().unwrap(), p["im"].as_f64().unwrap()))
            .collect();
        let expected_poles: Vec<C> = case
            .expected_poles
            .iter()
            .map(|&(re, im)| c(re, im))
            .collect();
        let pole_err = unordered_complex_distance(&actual_poles, &expected_poles);
        assert!(
            pole_err <= case.root_tolerance,
            "{}: pole set error {pole_err:e} > {:e}",
            case.id,
            case.root_tolerance
        );
        assert_conjugate_symmetry(&actual_poles, case.root_tolerance, case.id);

        let actual_zeros: Vec<C> = det["zeros"]
            .as_array()
            .unwrap()
            .iter()
            .map(|z| c(z["re"].as_f64().unwrap(), z["im"].as_f64().unwrap()))
            .collect();
        let expected_zeros: Vec<C> = case
            .expected_zeros
            .iter()
            .map(|&(re, im)| c(re, im))
            .collect();
        let zero_err = unordered_complex_distance(&actual_zeros, &expected_zeros);
        assert!(
            zero_err <= case.root_tolerance,
            "{}: zero set error {zero_err:e} > {:e}",
            case.id,
            case.root_tolerance
        );
        assert_conjugate_symmetry(&actual_zeros, case.root_tolerance, case.id);

        // 5. Verify real-axis segments both via the odd-count rule and phase condition G(sigma)H(sigma) < 0.
        let segs = det["passo_a_passo"]["passo_1_2_3"]["segmentos_eixo_real"]
            .as_array()
            .unwrap();
        assert!(
            !segs.is_empty(),
            "{}: real-axis segments must be populated",
            case.id
        );
        for &(sigma, expected_on_lgr) in case.real_axis_probes {
            let val = ol_tf.evaluate(c(sigma, 0.0));
            assert!(
                val.im.abs() < 1e-12,
                "{}: real-axis evaluation at s={sigma} had non-zero imaginary part {}",
                case.id,
                val.im
            );
            let on_lgr_by_phase = val.re < 0.0;
            let count_right = expected_poles
                .iter()
                .chain(expected_zeros.iter())
                .filter(|r| r.im.abs() < 1e-6 && r.re > sigma)
                .count();
            let on_lgr_by_count = count_right % 2 == 1;
            assert_eq!(
                on_lgr_by_phase, expected_on_lgr,
                "{}: phase condition at s={sigma} disagrees with expected={expected_on_lgr}",
                case.id
            );
            assert_eq!(
                on_lgr_by_count, expected_on_lgr,
                "{}: odd-count rule at s={sigma} disagrees with expected={expected_on_lgr}",
                case.id
            );
        }

        // 6. Verify asymptotes (centroid and angles).
        match case.expected_centroid {
            Some(exp_c) => {
                let act_c = det["centroide"]
                    .as_f64()
                    .unwrap_or_else(|| panic!("{}: expected centroid Some({exp_c})", case.id));
                assert!(
                    (act_c - exp_c).abs() <= TOL_CENTROID,
                    "{}: centroid {act_c} != {exp_c}",
                    case.id
                );
            }
            None => {
                assert!(
                    det["centroide"].is_null(),
                    "{}: expected null centroid for relative-degree-0 system",
                    case.id
                );
            }
        }
        let mut act_asym_angles: Vec<f64> = det["angulos_assintotas"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["graus"].as_f64().unwrap())
            .collect();
        act_asym_angles.sort_by(f64::total_cmp);
        assert_eq!(
            act_asym_angles.len(),
            case.expected_asymptote_angles_deg.len(),
            "{}: asymptote count mismatch",
            case.id
        );
        for (a, e) in act_asym_angles
            .iter()
            .zip(case.expected_asymptote_angles_deg)
        {
            assert!(
                (a - e).abs() <= TOL_ANGLE_DEG,
                "{}: asymptote angle {a} != {e}",
                case.id
            );
        }

        // 7. Verify breakaway and break-in points on the real axis.
        let mut act_bp: Vec<(f64, f64)> = det["break_points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| (b["s"].as_f64().unwrap(), b["K"].as_f64().unwrap()))
            .collect();
        act_bp.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert_eq!(
            act_bp.len(),
            case.expected_breakaway_points.len(),
            "{}: breakaway point count mismatch: actual={act_bp:?} vs expected={:?}",
            case.id,
            case.expected_breakaway_points
        );
        let d_num = poly::derivative(&ol_tf.num);
        let d_den = poly::derivative(&ol_tf.den);
        let dk_ds_num = poly::add(
            &poly::multiply(&d_num, &ol_tf.den),
            &poly::multiply(&ol_tf.num, &d_den),
            -1.0,
        );
        for ((act_s, act_k), (exp_s, exp_k)) in act_bp.iter().zip(case.expected_breakaway_points) {
            assert!(
                (act_s - exp_s).abs() <= TOL_BREAKAWAY_S,
                "{}: breakaway s {act_s} != {exp_s}",
                case.id
            );
            assert!(
                (act_k - exp_k).abs() <= TOL_BREAKAWAY_K,
                "{}: breakaway K {act_k} != {exp_k}",
                case.id
            );
            // Analytical stationarity check: N'(s)D(s) - N(s)D'(s) = 0 and D(s) + K*N(s) = 0
            let stationarity = poly::evaluate(&dk_ds_num, c(*exp_s, 0.0)).norm();
            assert!(
                stationarity <= 1e-7,
                "{}: stationarity residual at s={exp_s} is {stationarity:e}",
                case.id
            );
            let char_res =
                normalized_char_poly_residual(&ol_tf.den, &ol_tf.num, *exp_k, c(*exp_s, 0.0));
            assert!(
                char_res <= 1e-10,
                "{}: char equation residual at breakaway s={exp_s}, K={exp_k} is {char_res:e}",
                case.id
            );
        }
        // Also verify K=0 multiple-pole breakaway roots of dK/ds = 0 when present (Eq. 10, Eq. 16, Eq. 19).
        for &mult_s in case.expected_k0_multiple_pole_breakaways {
            let stationarity = poly::evaluate(&dk_ds_num, c(mult_s, 0.0)).norm();
            assert!(
                stationarity <= 1e-9,
                "{}: multiple pole at s={mult_s} must be a root of dK/ds=0, got {stationarity:e}",
                case.id
            );
            let char_at_k0 = poly::evaluate(&ol_tf.den, c(mult_s, 0.0)).norm();
            assert!(
                char_at_k0 <= 1e-9,
                "{}: multiple pole at s={mult_s} must satisfy D(s)=0 at K=0",
                case.id
            );
        }

        // 8. Verify angles of departure and arrival.
        let act_dep = det["angulos_partida"].as_array().unwrap();
        assert_eq!(
            act_dep.len(),
            case.expected_departure_angles.len(),
            "{}: departure angle count mismatch",
            case.id
        );
        for (act, &(re, im, exp_deg)) in act_dep.iter().zip(case.expected_departure_angles) {
            let act_deg = act["angulo"].as_f64().unwrap();
            let diff = wrap_angle_180(act_deg - exp_deg).abs();
            assert!(
                diff <= TOL_ANGLE_DEG,
                "{}: departure angle at ({re}, {im}) was {act_deg} != {exp_deg}",
                case.id
            );
        }

        let act_arr = det["angulos_chegada"].as_array().unwrap();
        assert_eq!(
            act_arr.len(),
            case.expected_arrival_angles.len(),
            "{}: arrival angle count mismatch",
            case.id
        );
        for (act, &(re, im, exp_deg)) in act_arr.iter().zip(case.expected_arrival_angles) {
            let act_deg = act["angulo"].as_f64().unwrap();
            let diff = wrap_angle_180(act_deg - exp_deg).abs();
            assert!(
                diff <= TOL_ANGLE_DEG,
                "{}: arrival angle at ({re}, {im}) was {act_deg} != {exp_deg}",
                case.id
            );
        }

        // 9. Verify jw-axis crossings and Routh-Hurwitz critical gains / stability intervals.
        let act_jw: Vec<(f64, f64)> = det["jw_cruzamentos"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| {
                let w = v["w"].as_f64().unwrap();
                let k = v["K"].as_f64().unwrap();
                (w > 0.0).then_some((w, k))
            })
            .collect();
        assert_eq!(
            act_jw.len(),
            case.expected_jw_crossings.len(),
            "{}: jw crossing count mismatch: actual={act_jw:?} vs expected={:?}",
            case.id,
            case.expected_jw_crossings
        );
        for ((act_w, act_k), (exp_w, exp_k)) in act_jw.iter().zip(case.expected_jw_crossings) {
            assert!(
                (act_w - exp_w).abs() <= TOL_JW_W,
                "{}: jw crossing omega {act_w} != {exp_w}",
                case.id
            );
            assert!(
                (act_k - exp_k).abs() <= TOL_JW_K,
                "{}: jw crossing K {act_k} != {exp_k}",
                case.id
            );
            // Exact characteristic equation check at s = j * exp_w, K = exp_k:
            let jw_res =
                normalized_char_poly_residual(&ol_tf.den, &ol_tf.num, *exp_k, c(0.0, *exp_w));
            assert!(
                jw_res <= 1e-12,
                "{}: exact jw crossing residual at w={exp_w}, K={exp_k} is {jw_res:e}",
                case.id
            );
        }

        // Cross-check with Routh-Hurwitz module:
        let routh_res = routh::analyze(&json!({"expr": case.ol_expr, "dpi": dpi})).unwrap();
        assert_eq!(
            routh_res["success"], true,
            "{}: Routh analysis failed",
            case.id
        );
        let k_range = &routh_res["k_range"];
        let stable_latex = k_range["stable_latex"].as_str().unwrap();
        assert!(
            stable_latex.contains(case.expected_positive_k_stability),
            "{}: Routh stable_latex '{stable_latex}' does not contain '{}'",
            case.id,
            case.expected_positive_k_stability
        );
        // Verify Routh critical_k matches expected jw crossing K_crit:
        let routh_crit: Vec<f64> = k_range["critical_k"]
            .as_array()
            .unwrap()
            .iter()
            .map(|ck| ck["k_val"].as_f64().unwrap())
            .collect();
        for &(_exp_w, exp_k) in case.expected_jw_crossings {
            assert!(
                routh_crit.iter().any(|&rk| (rk - exp_k).abs() <= 1e-6),
                "{}: Routh critical_k {routh_crit:?} missing expected K_crit={exp_k}",
                case.id
            );
        }

        // 10. Validate characteristic equation D(s) + K*N(s) = 0, conjugate symmetry,
        //     stability transitions, and branch continuity across gains.
        for &(k_probe, expected_stable) in case.stability_probes {
            let roots = poly::roots(&poly::add(&ol_tf.den, &ol_tf.num, k_probe)).unwrap();
            assert_eq!(roots.len(), case.expected_branches);
            assert_conjugate_symmetry(&roots, case.root_tolerance, case.id);
            for r in &roots {
                let res_norm = normalized_char_poly_residual(&ol_tf.den, &ol_tf.num, k_probe, *r);
                assert!(
                    res_norm <= 1e-9,
                    "{}: char poly residual at K={k_probe} for root {r} is {res_norm:e}",
                    case.id
                );
            }
            let is_stable = roots.iter().all(|r| r.re < -1e-7);
            assert_eq!(
                is_stable, expected_stable,
                "{}: stability mismatch at K={k_probe}: roots={roots:?}",
                case.id
            );
        }

        // 11. Verify behavior for K -> 0+ and K -> +infty, plus trajectory branch continuity.
        let k_small = 1e-8;
        let roots_small = poly::roots(&poly::add(&ol_tf.den, &ol_tf.num, k_small)).unwrap();
        let small_dist = unordered_complex_distance(&roots_small, &expected_poles);
        // For triple pole (m=3), perturbation scales as (k_small)^(1/3) = 1e-8/3 ~ 2.15e-3.
        let small_tol = if case.id == "LGR_Q5_EQ19" {
            5e-3
        } else if !case.expected_k0_multiple_pole_breakaways.is_empty() {
            5e-4
        } else {
            1e-5
        };
        assert!(
            small_dist <= small_tol,
            "{}: roots as K->0+ did not converge to open-loop poles: dist={small_dist:e} > {small_tol:e}",
            case.id
        );

        let k_large = 1e6;
        let roots_large = poly::roots(&poly::add(&ol_tf.den, &ol_tf.num, k_large)).unwrap();
        assert_conjugate_symmetry(&roots_large, 1e-4, case.id);
        for r in &roots_large {
            let res_norm = normalized_char_poly_residual(&ol_tf.den, &ol_tf.num, k_large, *r);
            assert!(
                res_norm <= 1e-8,
                "{}: high-gain residual at K={k_large} is {res_norm:e}",
                case.id
            );
        }
        // Each finite open-loop zero must attract a closed-loop root as K -> +infty:
        for z in &expected_zeros {
            let min_dist = roots_large
                .iter()
                .map(|r| (*r - *z).norm())
                .fold(f64::INFINITY, f64::min);
            assert!(
                min_dist <= 1e-2,
                "{}: open-loop zero {z} not approached at K={k_large}, min_dist={min_dist:e}",
                case.id
            );
        }
        // If P > Z, the remaining (P - Z) roots must diverge to infinity along asymptotes:
        if let Some(centroid) = case.expected_centroid {
            let mut diverging: Vec<C> = roots_large
                .iter()
                .copied()
                .filter(|r| expected_zeros.iter().all(|z| (*r - *z).norm() > 1.0))
                .collect();
            assert_eq!(
                diverging.len(),
                case.expected_poles.len() - case.expected_zeros.len(),
                "{}: diverging root count mismatch at K={k_large}",
                case.id
            );
            let mut div_angles: Vec<f64> = diverging
                .drain(..)
                .map(|r| {
                    let deg = (r - c(centroid, 0.0)).arg().to_degrees();
                    deg.rem_euclid(360.0)
                })
                .collect();
            div_angles.sort_by(f64::total_cmp);
            for (act_a, exp_a) in div_angles.iter().zip(case.expected_asymptote_angles_deg) {
                let diff = wrap_angle_180(act_a - exp_a).abs();
                assert!(
                    diff <= 2.5,
                    "{}: high-gain diverging root angle {act_a} deg differs from asymptote {exp_a} deg by {diff} deg",
                    case.id
                );
            }
        }
    }
}

#[test]
fn validate_lgr_branch_continuity_and_equivalences_and_q5_parametric_departure() {
    let cases = all_lgr_academic_cases();

    // 1. Verify branch continuity on dense logarithmic grid for every case.
    let gains: Vec<f64> = (0..=400)
        .map(|i| 10.0_f64.powf(-3.0 + 6.0 * (i as f64) / 400.0))
        .collect();
    for case in &cases {
        let ol_tf = TransferFunction::parse(case.ol_expr).unwrap().normalized();
        let mut prev_roots = poly::roots(&poly::add(&ol_tf.den, &ol_tf.num, gains[0])).unwrap();
        for &k in &gains[1..] {
            let roots = poly::roots(&poly::add(&ol_tf.den, &ol_tf.num, k)).unwrap();
            assert!(
                roots.iter().all(|r| r.re.is_finite() && r.im.is_finite()),
                "{}: non-finite root at K={k}",
                case.id
            );
            let step_dist = unordered_complex_distance(&roots, &prev_roots);
            let max_norm = roots.iter().map(|r| r.norm()).fold(1.0, f64::max);
            assert!(
                step_dist <= 0.25 * max_norm,
                "{}: branch discontinuity at K={k}: step_dist={step_dist} (max_norm={max_norm})",
                case.id
            );
            prev_roots = roots;
        }
    }

    // 2. Verify mathematical equivalences required by the problem statement:
    //    - 1(a) Eq.(1) == 2(d) Eq.(7)
    //    - 1(b) Eq.(2) == 2(c) Eq.(6)
    //    - 1(c) Eq.(3) == 2(i)-1 Eq.(12) == 3(a) Eq.(14)
    let find_case = |id: &str| -> Value {
        let c = cases.iter().find(|c| c.id == id).unwrap();
        lgr::analyze(&json!({"expr": c.ol_expr, "dpi": 72})).unwrap()["detalhes"].clone()
    };
    assert_eq!(find_case("LGR_Q1A_EQ01"), find_case("LGR_Q2D_EQ07"));
    assert_eq!(find_case("LGR_Q1B_EQ02"), find_case("LGR_Q2C_EQ06"));
    let q1c = find_case("LGR_Q1C_EQ03");
    assert_eq!(q1c, find_case("LGR_Q2I1_EQ12"));
    assert_eq!(q1c, find_case("LGR_Q3A_EQ14"));

    // 3. Verify Question 5 Eq. (19) triple-pole departure angles (60 deg, 180 deg, 300 deg)
    //    from s = 0 for s^3 + a2*s^2 + a1*s + a1 = 0 with a2 = 0 and small a1 > 0.
    let q5_tf = TransferFunction::parse("(s + 1)/(s^3)").unwrap();
    let k_eps = 1e-6;
    let q5_roots = poly::roots(&poly::add(&q5_tf.den, &q5_tf.num, k_eps)).unwrap();
    let mut dep_angles: Vec<f64> = q5_roots
        .iter()
        .map(|r| r.arg().to_degrees().rem_euclid(360.0))
        .collect();
    dep_angles.sort_by(f64::total_cmp);
    for (act, exp) in dep_angles.iter().zip(&[60.0, 180.0, 300.0]) {
        assert!(
            (act - exp).abs() <= 0.5,
            "Q5 Eq.(19): triple pole departure angle {act} deg != {exp} deg"
        );
    }
}
