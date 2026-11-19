//! Generic controller compliance verification and constrained numerical synthesis.
//!
//! Separates two fundamentally different engineering workflows:
//! 1. Classic Root-Locus (LGR) pole placement (`lgr_classic`) followed by full closed-loop
//!    time-domain compliance verification (detecting overshoot inflation from extra zeros,
//!    settling-time mismatch, non-dominant poles, approximate cancellations, and instability).
//! 2. Constrained time-domain synthesis (`temporal_refined` / `auto_simplest`) using generic
//!    candidate generation (including stable plant-pole cancellation and multi-scale grids)
//!    coupled with bounded Nelder-Mead / coordinate refinement over exact matrix-exponential
//!    step simulations.

use crate::{
    Result, controller_math as math, polynomial as poly,
    simulation::{self, Input},
    transfer::TransferFunction,
};
use num_complex::Complex64 as C;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesignObjective {
    /// Specifications target dominant closed-loop pole locations (e.g. sd, or xi/wn of poles).
    PolePlacement,
    /// Specifications impose strict time-domain response limits (e.g. Mp <= 25%, ts <= 4s).
    TimeDomainConstraints,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettlingSpec {
    /// Inequality constraint: measured settling time <= max_s.
    AtMost(f64),
    /// Equality constraint: |measured settling time - target| <= tolerance.
    Equal { target: f64, tolerance: f64 },
}

/// Uniform documented default tolerance for settling-time equality constraints:
/// +/- 2% of target, with a 0.02 s floor (e.g., +/-0.02 s at ts=1 s, +/-0.08 s at ts=4 s).
pub fn default_settling_tolerance(target: f64) -> f64 {
    (0.02 * target.abs()).max(0.02)
}

/// Uniform documented default tolerance for damping-ratio equality constraints.
pub fn default_damping_tolerance() -> f64 {
    0.01
}

/// Uniform documented default tolerance for natural-frequency equality constraints (+/- 2%, floor 0.01 rad/s).
pub fn default_wn_tolerance(target: f64) -> f64 {
    (0.02 * target.abs()).max(0.01)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DampingSpec {
    /// Mandatory exact equality on the complex conjugate pole pair: |xi - target| <= tolerance.
    Equal { target: f64, tolerance: f64 },
    /// Mandatory minimum damping ratio: xi >= min.
    AtLeast(f64),
    /// Optional didactic estimate (e.g., derived from Mp); reported transparently, not enforced as equality.
    DidacticEstimate(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NaturalFreqSpec {
    /// Mandatory exact equality on the reference pole/pair: |wn - target| <= tolerance.
    Equal { target: f64, tolerance: f64 },
    /// Mandatory minimum natural frequency: wn >= min.
    AtLeast(f64),
    /// Optional informational estimate.
    OptionalEstimate(f64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeasibilityClass {
    /// Feasible controller found satisfying all mandatory constraints.
    Feasible,
    /// Mathematically proven infeasible for the requested structure and constraints.
    ProvenInfeasible,
    /// No feasible solution found within the numerical search bounds (not a mathematical impossibility proof).
    NotFoundWithinSearchBounds,
}

impl FeasibilityClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feasible => "feasible",
            Self::ProvenInfeasible => "proven_infeasible",
            Self::NotFoundWithinSearchBounds => "not_found_within_search_bounds",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerStructure {
    P,
    PI,
    PD,
    Lead,
    PID,
}

impl ControllerStructure {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::P => "P",
            Self::PI => "PI",
            Self::PD => "PD",
            Self::Lead => "LEAD",
            Self::PID => "PID",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "P" => Some(Self::P),
            "PI" => Some(Self::PI),
            "PD" => Some(Self::PD),
            "LEAD" | "AVANCO" | "AVANÇO" => Some(Self::Lead),
            "PID" => Some(Self::PID),
            _ => None,
        }
    }

    pub fn has_integrator(self) -> bool {
        matches!(self, Self::PI | Self::PID)
    }

    pub fn complexity_order() -> [Self; 5] {
        [Self::P, Self::PI, Self::PD, Self::Lead, Self::PID]
    }
}

#[derive(Clone, Debug)]
pub struct DesignSpecs {
    pub objective: DesignObjective,
    pub max_overshoot_pct: Option<f64>,
    pub settling_time: Option<SettlingSpec>,
    pub settling_criterion_pct: u8,
    pub damping: Option<DampingSpec>,
    pub natural_freq: Option<NaturalFreqSpec>,
    pub require_complex_dominance: bool,
    pub require_zero_step_error: bool,
    pub desired_dominant_pole: Option<C>,
    pub theoretical_xi: Option<f64>,
    pub theoretical_wn: Option<f64>,
    pub theoretical_mp_pct: Option<f64>,
    pub theoretical_ts_s: Option<f64>,
}

impl Default for DesignSpecs {
    fn default() -> Self {
        Self {
            objective: DesignObjective::TimeDomainConstraints,
            max_overshoot_pct: None,
            settling_time: None,
            settling_criterion_pct: 2,
            damping: None,
            natural_freq: None,
            require_complex_dominance: false,
            require_zero_step_error: false,
            desired_dominant_pole: None,
            theoretical_xi: None,
            theoretical_wn: None,
            theoretical_mp_pct: None,
            theoretical_ts_s: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequirementStatus {
    Pass,
    Warning,
    Fail,
}

impl RequirementStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warning => "WARNING",
            Self::Fail => "FAIL",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RequirementCheck {
    pub id: String,
    pub label: String,
    pub requested: String,
    pub theoretical: String,
    pub measured: String,
    pub difference: String,
    pub status: RequirementStatus,
    pub mandatory: bool,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct CancellationInfo {
    pub pole: C,
    pub zero: C,
    pub distance: f64,
    pub exact: bool,
    pub unstable: bool,
}

#[derive(Clone, Debug)]
pub struct VerificationReport {
    pub overall_status: RequirementStatus,
    pub requirements_passed: bool,
    pub numerically_valid: bool,
    pub closed_loop_stable: bool,
    pub internally_stable: bool,
    pub closed_loop_poles: Vec<C>,
    pub closed_loop_zeros: Vec<C>,
    pub dominant_pole: Option<C>,
    pub dominant_pole_ratio: Option<f64>,
    pub non_dominant_poles: Vec<C>,
    pub extra_closed_loop_zeros: Vec<C>,
    pub cancellations: Vec<CancellationInfo>,
    pub measured_xi: Option<f64>,
    pub measured_wn: Option<f64>,
    pub damping_pole_label: Option<String>,
    pub has_complex_pair: bool,
    pub complex_pair_is_dominant: bool,
    pub non_dominant_residue_ratio: Option<f64>,
    pub measured_overshoot_pct: Option<f64>,
    pub measured_settling_time_s: Option<f64>,
    pub measured_rise_time_s: Option<f64>,
    pub measured_peak_time_s: Option<f64>,
    pub measured_steady_state_y: Option<f64>,
    pub summer_steady_state_error: Option<f64>,
    pub output_tracking_error: Option<f64>,
    pub checks: Vec<RequirementCheck>,
    pub warnings: Vec<String>,
    pub failed_requirements: Vec<String>,
}

impl VerificationReport {
    pub fn to_json(&self) -> Value {
        let fmt_c = |v: C| json!({"re": v.re, "im": v.im});
        json!({
            "overall_status": self.overall_status.as_str(),
            "requirements_passed": self.requirements_passed,
            "numerically_valid": self.numerically_valid,
            "closed_loop_stable": self.closed_loop_stable,
            "internally_stable": self.internally_stable,
            "closed_loop_poles": self.closed_loop_poles.iter().copied().map(fmt_c).collect::<Vec<_>>(),
            "closed_loop_zeros": self.closed_loop_zeros.iter().copied().map(fmt_c).collect::<Vec<_>>(),
            "dominant_pole": self.dominant_pole.map(fmt_c),
            "dominant_pole_ratio": self.dominant_pole_ratio,
            "non_dominant_poles": self.non_dominant_poles.iter().copied().map(fmt_c).collect::<Vec<_>>(),
            "extra_closed_loop_zeros": self.extra_closed_loop_zeros.iter().copied().map(fmt_c).collect::<Vec<_>>(),
            "cancellations": self.cancellations.iter().map(|c| json!({
                "pole": fmt_c(c.pole),
                "zero": fmt_c(c.zero),
                "distance": c.distance,
                "exact": c.exact,
                "unstable": c.unstable,
            })).collect::<Vec<_>>(),
            "modal_analysis": {
                "measured_xi": self.measured_xi,
                "measured_wn": self.measured_wn,
                "damping_pole_label": self.damping_pole_label,
                "has_complex_pair": self.has_complex_pair,
                "complex_pair_is_dominant": self.complex_pair_is_dominant,
                "non_dominant_residue_ratio": self.non_dominant_residue_ratio,
            },
            "measured": {
                "overshoot_percent": self.measured_overshoot_pct,
                "settling_time": self.measured_settling_time_s,
                "rise_time": self.measured_rise_time_s,
                "peak_time": self.measured_peak_time_s,
                "steady_state_y": self.measured_steady_state_y,
                "summer_steady_state_error": self.summer_steady_state_error,
                "output_tracking_error": self.output_tracking_error,
                "xi": self.measured_xi,
                "wn": self.measured_wn,
            },
            "checks": self.checks.iter().map(|ck| json!({
                "id": ck.id,
                "label": ck.label,
                "requested": ck.requested,
                "theoretical": ck.theoretical,
                "measured": ck.measured,
                "difference": ck.difference,
                "status": ck.status.as_str(),
                "mandatory": ck.mandatory,
                "message": ck.message,
            })).collect::<Vec<_>>(),
            "warnings": self.warnings,
            "failed_requirements": self.failed_requirements,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SynthesizedController {
    pub structure: ControllerStructure,
    pub transfer: TransferFunction,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub z: Option<f64>,
    pub zi: Option<f64>,
    pub p: Option<f64>,
    pub k: Option<f64>,
    pub method_used: String,
    pub verification: VerificationReport,
}

#[derive(Clone, Debug)]
pub struct SynthesisOutcome {
    pub feasible: bool,
    pub feasibility_class: FeasibilityClass,
    pub selected: Option<SynthesizedController>,
    pub infeasibility_reasons: Vec<String>,
    pub alternatives: Vec<SynthesizedController>,
    pub candidates_evaluated: usize,
}

fn fmt_val(v: f64, decimals: usize) -> String {
    format!("{v:.decimals$}").replace('.', ",")
}

fn fmt_complex(z: C) -> String {
    if z.im.abs() < 1e-5 {
        fmt_val(z.re, 4)
    } else {
        format!(
            "{} {} {}j",
            fmt_val(z.re, 4),
            if z.im >= 0.0 { "+" } else { "-" },
            fmt_val(z.im.abs(), 4)
        )
    }
}

/// Exact limit lim_{s -> 0} N(s)/D(s), accounting for exact common powers of s.
fn exact_dc_gain(tf: &TransferFunction) -> f64 {
    let nz = tf
        .num
        .iter()
        .rev()
        .take_while(|&&v| v.abs() < 1e-14)
        .count();
    let dz = tf
        .den
        .iter()
        .rev()
        .take_while(|&&v| v.abs() < 1e-14)
        .count();
    if nz == tf.num.len() {
        return 0.0;
    }
    let num_c = tf.num[tf.num.len() - nz - 1];
    let den_c = tf.den[tf.den.len() - dz - 1];
    match nz.cmp(&dz) {
        std::cmp::Ordering::Greater => 0.0,
        std::cmp::Ordering::Equal => num_c / den_c,
        std::cmp::Ordering::Less => f64::INFINITY.copysign(num_c / den_c),
    }
}

/// System type (number of pure integrators at s = 0) of an open-loop transfer function.
pub fn system_type(ol: &TransferFunction) -> usize {
    let nz = ol
        .num
        .iter()
        .rev()
        .take_while(|&&v| v.abs() < 1e-12)
        .count();
    let dz = ol
        .den
        .iter()
        .rev()
        .take_while(|&&v| v.abs() < 1e-12)
        .count();
    dz.saturating_sub(nz)
}

/// Dominant high-frequency / steady-state sign of plant G(s)H(s) for negative feedback stability.
fn plant_sign(ol: &TransferFunction) -> f64 {
    let lead = ol.num[0] / ol.den[0];
    if lead < 0.0 { -1.0 } else { 1.0 }
}

/// Exact step response metrics with sub-sample linear interpolation for definitive settling time.
struct SampledMetrics {
    overshoot_pct: f64,
    settling_time: Option<f64>,
    rise_time: Option<f64>,
    peak_time: f64,
}

fn compute_step_metrics(
    cl_tf: &TransferFunction,
    horizon_s: f64,
    points: usize,
    threshold: f64,
) -> Result<SampledMetrics> {
    let steady = exact_dc_gain(cl_tf);
    if !steady.is_finite() || steady.abs() < 1e-12 {
        return Err("Ganho estático em malha fechada nulo ou infinito.".into());
    }
    let dt = horizon_s / (points.saturating_sub(1).max(1)) as f64;
    let t: Vec<f64> = (0..points).map(|i| i as f64 * dt).collect();
    let y = simulation::simulate(cl_tf, &t, Input::Step)?;
    if y.iter().any(|v| !v.is_finite()) {
        return Err("Resposta temporal apresentou valores não finitos.".into());
    }
    let sign = steady.signum();
    let mut peak_idx = 0usize;
    let mut max_signed = sign * y[0];
    for (i, &val) in y.iter().enumerate().skip(1) {
        let sv = sign * val;
        if sv > max_signed {
            max_signed = sv;
            peak_idx = i;
        }
    }
    let overshoot_pct = ((max_signed - steady.abs()) / steady.abs() * 100.0).max(0.0);

    // 10% to 90% rise time with linear interpolation
    let interp_crossing = |target_frac: f64| -> Option<f64> {
        let target = target_frac * steady.abs();
        let idx = y.iter().position(|&v| sign * v >= target)?;
        if idx == 0 {
            return Some(t[0]);
        }
        let y0 = sign * y[idx - 1];
        let y1 = sign * y[idx];
        let frac = if (y1 - y0).abs() > 1e-15 {
            ((target - y0) / (y1 - y0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Some(t[idx - 1] + frac * (t[idx] - t[idx - 1]))
    };
    let rise_time = interp_crossing(0.1)
        .zip(interp_crossing(0.9))
        .map(|(t10, t90)| (t90 - t10).max(0.0));

    // Definitive entry into +/- threshold band (last sample outside band)
    let outside = y
        .iter()
        .rposition(|&v| (v / steady - 1.0).abs() > threshold);
    let settling_time = match outside {
        None => Some(0.0),
        Some(i) if i + 1 < t.len() => {
            let e0 = (y[i] / steady - 1.0).abs();
            let e1 = (y[i + 1] / steady - 1.0).abs();
            let frac = if (e0 - e1).abs() > 1e-15 {
                ((e0 - threshold) / (e0 - e1)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            Some(t[i] + frac * (t[i + 1] - t[i]))
        }
        _ => None,
    };

    Ok(SampledMetrics {
        overshoot_pct,
        settling_time,
        rise_time,
        peak_time: t[peak_idx],
    })
}

struct ModalDampingInfo {
    measured_xi: Option<f64>,
    measured_wn: Option<f64>,
    pole_label: Option<String>,
    has_complex_pair: bool,
    complex_pair_is_dominant: bool,
    non_dominant_residue_ratio: Option<f64>,
}

/// Evaluate polynomial P(s) at complex point s using Horner's method.
fn eval_poly_c(coeffs: &[f64], s: C) -> C {
    coeffs
        .iter()
        .fold(C::new(0.0, 0.0), |acc, &c| acc * s + C::new(c, 0.0))
}

/// Identify the closed-loop pole or complex conjugate pair to which damping ratio (xi)
/// and natural frequency (wn) apply, and estimate the modal step-response residue ratio
/// of non-dominant modes versus the reference complex pair.
fn identify_damping_mode(
    cl_poles: &[C],
    active_poles: &[C],
    cl_num: &[f64],
    char_poly: &[f64],
) -> ModalDampingInfo {
    if cl_poles.is_empty() {
        return ModalDampingInfo {
            measured_xi: None,
            measured_wn: None,
            pole_label: None,
            has_complex_pair: false,
            complex_pair_is_dominant: false,
            non_dominant_residue_ratio: None,
        };
    }

    // Prefer active (non-cancelled) complex conjugate pairs, falling back to all complex pairs.
    let complex_candidates: Vec<C> = active_poles
        .iter()
        .copied()
        .filter(|p| p.im > 1e-6)
        .collect();
    let complex_candidates = if complex_candidates.is_empty() {
        cl_poles.iter().copied().filter(|p| p.im > 1e-6).collect()
    } else {
        complex_candidates
    };

    if let Some(pair) = complex_candidates
        .iter()
        .copied()
        .max_by(|a, b| a.re.total_cmp(&b.re))
    {
        let wn = pair.norm();
        let xi = if wn > 1e-12 {
            (-pair.re / wn).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        // Check if any active (uncancelled) real pole is slower than this complex pair
        let slower_active_real: Vec<C> = active_poles
            .iter()
            .copied()
            .filter(|p| p.im.abs() <= 1e-6 && p.re > pair.re + 1e-4)
            .collect();
        let complex_pair_is_dominant = slower_active_real.is_empty();

        // Compute exact Heaviside modal residue ratio for T(s)/s when poles are well separated
        let mut dom_res = 0.0;
        let mut other_res = 0.0;
        let mut residues_valid = !char_poly.is_empty() && char_poly[0].abs() > 1e-14;
        if residues_valid {
            let lead = char_poly[0];
            for (i, &p) in cl_poles.iter().enumerate() {
                if p.norm() < 1e-9 {
                    residues_valid = false;
                    break;
                }
                let mut denom = C::new(lead, 0.0) * p;
                for (j, &q) in cl_poles.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let diff = p - q;
                    if diff.norm() < 1e-5 {
                        residues_valid = false;
                        break;
                    }
                    denom *= diff;
                }
                if !residues_valid {
                    break;
                }
                let r_mag = (eval_poly_c(cl_num, p) / denom).norm();
                if (p.re - pair.re).abs() < 1e-4 && (p.im.abs() - pair.im.abs()).abs() < 1e-4 {
                    dom_res += r_mag;
                } else {
                    other_res += r_mag;
                }
            }
        }
        let non_dominant_residue_ratio = if residues_valid && dom_res > 1e-12 {
            Some(other_res / dom_res)
        } else {
            None
        };

        let label = if cl_poles.len() <= 2 {
            format!(
                "par complexo ({} ± {}j)",
                fmt_val(pair.re, 4),
                fmt_val(pair.im.abs(), 4)
            )
        } else if complex_pair_is_dominant {
            format!(
                "par complexo dominante ({} ± {}j)",
                fmt_val(pair.re, 4),
                fmt_val(pair.im.abs(), 4)
            )
        } else {
            let slow_str: Vec<String> = slower_active_real
                .iter()
                .map(|p| fmt_val(p.re, 4))
                .collect();
            format!(
                "par complexo ({} ± {}j; polo real mais lento em {})",
                fmt_val(pair.re, 4),
                fmt_val(pair.im.abs(), 4),
                slow_str.join(", ")
            )
        };

        ModalDampingInfo {
            measured_xi: Some(xi),
            measured_wn: Some(wn),
            pole_label: Some(label),
            has_complex_pair: true,
            complex_pair_is_dominant,
            non_dominant_residue_ratio,
        }
    } else {
        // All closed-loop poles are real (overdamped / critically damped => xi = 1.0)
        let slowest = active_poles
            .iter()
            .chain(cl_poles.iter())
            .copied()
            .max_by(|a, b| a.re.total_cmp(&b.re));
        let wn = slowest.map(|p| p.re.abs());
        let poles_str: Vec<String> = cl_poles.iter().map(|p| fmt_val(p.re, 4)).collect();
        ModalDampingInfo {
            measured_xi: Some(1.0),
            measured_wn: wn,
            pole_label: Some(format!(
                "polos reais sem par complexo ({})",
                poles_str.join(", ")
            )),
            has_complex_pair: false,
            complex_pair_is_dominant: false,
            non_dominant_residue_ratio: None,
        }
    }
}

/// Comprehensive compliance verification of a candidate controller against plant, feedback, and specs.
pub fn verify_controller(
    plant: &TransferFunction,
    feedback: Option<&TransferFunction>,
    controller: &TransferFunction,
    specs: &DesignSpecs,
) -> Result<VerificationReport> {
    let unity = TransferFunction::scalar(1.0);
    let h = feedback.unwrap_or(&unity);

    // Unreduced characteristic polynomial: D_c*D_g*D_h + N_c*N_g*N_h = 0.
    // Checking its roots detects BOTH external instability AND internal instability
    // from unstable (RHP or jw) pole-zero cancellations.
    let fwd_num = poly::multiply(&controller.num, &plant.num);
    let fwd_den = poly::multiply(&controller.den, &plant.den);
    let loop_num = poly::multiply(&fwd_num, &h.num);
    let loop_den = poly::multiply(&fwd_den, &h.den);
    let char_poly = poly::add(&loop_den, &loop_num, 1.0);

    let cl_poles = poly::roots(&char_poly)?;
    let closed_loop_stable = !cl_poles.is_empty() && cl_poles.iter().all(|p| p.re < -1e-7);

    // Closed-loop T_YR(s) = (N_c*N_g*D_h) / (D_c*D_g*D_h + N_c*N_g*N_h)
    let cl_num = poly::multiply(&fwd_num, &h.den);
    let cl_zeros = poly::roots(&cl_num)?;

    // Detect pole-zero cancellations (both in open-loop Gc*G*H and closed-loop T_YR)
    let mut cancellations = Vec::new();
    let mut cancelled_pole_idx = vec![false; cl_poles.len()];
    let mut cancelled_zero_idx = vec![false; cl_zeros.len()];
    for (zi, &z) in cl_zeros.iter().enumerate() {
        let mut best: Option<(usize, f64)> = None;
        for (pi, &p) in cl_poles.iter().enumerate() {
            if cancelled_pole_idx[pi] {
                continue;
            }
            let d = (p - z).norm();
            let tol = 0.05 * p.norm().max(1.0);
            if d <= tol && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((pi, d));
            }
        }
        if let Some((pi, d)) = best {
            cancelled_pole_idx[pi] = true;
            cancelled_zero_idx[zi] = true;
            cancellations.push(CancellationInfo {
                pole: cl_poles[pi],
                zero: z,
                distance: d,
                exact: d < 1e-4,
                unstable: cl_poles[pi].re >= -1e-7,
            });
        }
    }

    // Also check open-loop unstable cancellations between Gc and G*H
    let mut ol_unstable_cancel = false;
    let gc_zeros = controller.zeros()?;
    let gh_poles = poly::roots(&poly::multiply(&plant.den, &h.den))?;
    for &z in &gc_zeros {
        for &p in &gh_poles {
            if p.re >= -1e-7 && (p - z).norm() < 1e-2 * p.norm().max(1.0) {
                ol_unstable_cancel = true;
            }
        }
    }
    let internally_stable =
        closed_loop_stable && !ol_unstable_cancel && !cancellations.iter().any(|c| c.unstable);

    // Remaining uncancelled poles & zeros
    let active_poles: Vec<C> = cl_poles
        .iter()
        .enumerate()
        .filter(|(i, _)| !cancelled_pole_idx[*i])
        .map(|(_, &p)| p)
        .collect();
    let extra_zeros: Vec<C> = cl_zeros
        .iter()
        .enumerate()
        .filter(|(i, _)| !cancelled_zero_idx[*i])
        .map(|(_, &z)| z)
        .collect();

    // Identify dominant pole(s) and non-dominant poles among active (uncancelled) poles
    let dominant_pole = active_poles.iter().copied().max_by(|a, b| {
        a.re.total_cmp(&b.re)
            .then(b.im.abs().total_cmp(&a.im.abs()))
    });
    let mut non_dominant_poles = Vec::new();
    let mut dominant_pole_ratio: Option<f64> = None;
    if let Some(dom) = dominant_pole {
        let dom_abs_re = dom.re.abs().max(1e-9);
        for &p in &active_poles {
            let is_dom_or_conj = (p - dom).norm() < 1e-4 || (p - dom.conj()).norm() < 1e-4;
            if !is_dom_or_conj {
                non_dominant_poles.push(p);
                let ratio = p.re.abs() / dom_abs_re;
                dominant_pole_ratio = Some(match dominant_pole_ratio {
                    Some(cur) => cur.min(ratio),
                    None => ratio,
                });
            }
        }
    }

    // Identify reference pole/pair for damping ratio (xi) and natural frequency (wn)
    let modal = identify_damping_mode(&cl_poles, &active_poles, &cl_num, &char_poly);

    // Build closed-loop TF for simulation (applying fast derivative filter only if improper)
    let sim_cl_tf = {
        let raw_cl = TransferFunction::new(cl_num.clone(), char_poly.clone())?;
        if raw_cl.is_proper() {
            raw_cl
        } else {
            let mut f = controller.multiply(plant)?;
            while !f.is_proper() {
                f = f.multiply(&TransferFunction::new(vec![1.0], vec![0.001, 1.0])?)?;
            }
            f.feedback(h, -1.0)?
        }
    };

    // Steady-state error at summer and output tracking error
    let loop_tf = TransferFunction::new(loop_num, loop_den)?;
    let h_dc = exact_dc_gain(h);
    let ol_type = system_type(&loop_tf);
    let summer_ess = if !closed_loop_stable {
        None
    } else if ol_type >= 1 {
        Some(0.0)
    } else {
        let kp_dc = exact_dc_gain(&loop_tf);
        if kp_dc.is_infinite() {
            Some(0.0)
        } else {
            Some(1.0 / (1.0 + kp_dc))
        }
    };
    let steady_y = if closed_loop_stable {
        Some(exact_dc_gain(&sim_cl_tf))
    } else {
        None
    };
    let output_tracking_error = steady_y.map(|y| 1.0 - y);

    // High-resolution time-domain simulation when closed-loop is stable
    let threshold = if specs.settling_criterion_pct == 5 {
        0.05
    } else {
        0.02
    };
    let mut measured_mp = None;
    let mut measured_ts = None;
    let mut measured_tr = None;
    let mut measured_tp = None;
    if closed_loop_stable {
        let slowest_decay = active_poles
            .iter()
            .map(|p| p.re.abs())
            .filter(|&r| r > 1e-6)
            .fold(f64::INFINITY, f64::min);
        let spec_ts = match specs.settling_time {
            Some(SettlingSpec::AtMost(v)) => v,
            Some(SettlingSpec::Equal { target, .. }) => target,
            None => specs.theoretical_ts_s.unwrap_or(4.0),
        };
        let mut horizon = if slowest_decay.is_finite() {
            (14.0 / slowest_decay).max(spec_ts * 3.0).max(4.0)
        } else {
            (spec_ts * 3.0).max(10.0)
        };
        horizon = horizon.clamp(1.0, 500.0);
        if let Ok(mut m) = compute_step_metrics(&sim_cl_tf, horizon, 2500, threshold) {
            // If settling time not reached yet, extend horizon once
            if m.settling_time.is_none() && horizon < 500.0 {
                let ext = (horizon * 3.0).min(500.0);
                if let Ok(m2) = compute_step_metrics(&sim_cl_tf, ext, 3000, threshold) {
                    m = m2;
                }
            }
            measured_mp = Some(m.overshoot_pct);
            measured_ts = m.settling_time;
            measured_tr = m.rise_time;
            measured_tp = Some(m.peak_time);
        }
    }

    // Assemble diagnostic warnings
    let mut warnings = Vec::new();
    for c in &cancellations {
        if c.unstable {
            warnings.push(format!(
                "Cancelamento polo-zero instável em s = {} (distância {:0.2e}): malha internamente instável.",
                fmt_complex(c.pole),
                c.distance
            ));
        } else if c.exact {
            warnings.push(format!(
                "Cancelamento polo-zero nominal em s = {} (distância {:0.2e}): verificar robustez sob variações paramétricas da planta.",
                fmt_complex(c.pole),
                c.distance
            ));
        } else {
            warnings.push(format!(
                "Cancelamento polo-zero aproximado entre polo s = {} e zero s = {} (distância {:0.4}): dipolo residual sensível a perturbações.",
                fmt_complex(c.pole),
                fmt_complex(c.zero),
                c.distance
            ));
        }
    }
    if let (Some(dom), Some(ratio)) = (dominant_pole, dominant_pole_ratio)
        && ratio < 5.0
    {
        let extra_list: Vec<String> = non_dominant_poles
            .iter()
            .copied()
            .map(fmt_complex)
            .collect();
        warnings.push(format!(
            "Dominância comprometida: polo(s) adicional(is) em {} possuem razão |Re(p)|/|Re(p_dom)| = {} < 5 em relação ao polo dominante {}.",
            extra_list.join(", "),
            fmt_val(ratio, 2),
            fmt_complex(dom)
        ));
    }
    if let Some(dom) = dominant_pole {
        let dom_abs = dom.re.abs().max(1e-6);
        let close_zeros: Vec<String> = extra_zeros
            .iter()
            .filter(|z| z.norm() < 6.0 * dom_abs)
            .copied()
            .map(fmt_complex)
            .collect();
        if !close_zeros.is_empty() {
            warnings.push(format!(
                "Limitação da aproximação de 2ª ordem: zero(s) de malha fechada não cancelado(s) em {} alteram o sobressinal real e o tempo de acomodação em relação às fórmulas padrão sem zeros.",
                close_zeros.join(", ")
            ));
        }
    }
    if let (Some(xi), Some(mp_limit)) = (specs.theoretical_xi, specs.max_overshoot_pct) {
        let mp_from_xi_val = math::mp_from_xi(xi);
        if mp_from_xi_val > mp_limit + 0.05 {
            warnings.push(format!(
                "Incompatibilidade didática nas especificações: ξ = {} implica sobressinal teórico de 2ª ordem de {}%, superior ao limite literal Mp ≤ {}%.",
                fmt_val(xi, 3),
                fmt_val(mp_from_xi_val, 2),
                fmt_val(mp_limit, 2)
            ));
        }
    }
    if (h_dc - 1.0).abs() > 1e-6
        && let (Some(y_ss), Some(ea)) = (steady_y, summer_ess)
    {
        warnings.push(format!(
            "Realimentação não unitária H(0) = {}: o erro atuante no somador é e_a(∞) = {}, enquanto a saída converge para y(∞) = 1/H(0) = {} (erro de rastreamento r(∞) - y(∞) = {}).",
            fmt_val(h_dc, 4),
            fmt_val(ea, 4),
            fmt_val(y_ss, 4),
            fmt_val(1.0 - y_ss, 4)
        ));
    }

    // Build individual requirement checks
    let mut checks = Vec::new();
    let mut failed_requirements = Vec::new();

    // 1. Stability
    checks.push(RequirementCheck {
        id: "stability".into(),
        label: "Estabilidade da malha fechada".into(),
        requested: "Re(p) < 0 para todos os polos".into(),
        theoretical: "Estável".into(),
        measured: if closed_loop_stable {
            "Estável".into()
        } else {
            "Instável".into()
        },
        difference: "—".into(),
        status: if closed_loop_stable {
            RequirementStatus::Pass
        } else {
            RequirementStatus::Fail
        },
        mandatory: true,
        message: if closed_loop_stable {
            "Todos os polos da equação característica estão no semiplano esquerdo.".into()
        } else {
            "Há polo(s) no semiplano direito ou sobre o eixo imaginário.".into()
        },
    });
    if !closed_loop_stable {
        failed_requirements.push("Estabilidade da malha fechada".into());
    }

    // 2. Internal stability
    checks.push(RequirementCheck {
        id: "internal_stability".into(),
        label: "Estabilidade interna".into(),
        requested: "Sem cancelamento polo-zero no SPD/jω".into(),
        theoretical: "Internamente estável".into(),
        measured: if internally_stable {
            "Internamente estável".into()
        } else {
            "Instável internamente".into()
        },
        difference: "—".into(),
        status: if internally_stable {
            RequirementStatus::Pass
        } else {
            RequirementStatus::Fail
        },
        mandatory: true,
        message: if internally_stable {
            "Nenhum modo instável oculto por cancelamento polo-zero.".into()
        } else {
            "Detectada instabilidade interna ou cancelamento instável.".into()
        },
    });
    if !internally_stable {
        failed_requirements.push("Estabilidade interna".into());
    }

    // 3. Dominant pole placement (when desired_dominant_pole is provided)
    if let Some(sd) = specs.desired_dominant_pole {
        let min_dist = cl_poles
            .iter()
            .map(|p| (*p - sd).norm())
            .fold(f64::INFINITY, f64::min);
        let closest = cl_poles
            .iter()
            .copied()
            .min_by(|a, b| (*a - sd).norm().total_cmp(&(*b - sd).norm()))
            .unwrap_or(C::new(f64::NAN, f64::NAN));
        let pole_ok = min_dist <= 1e-3;
        let mandatory = specs.objective == DesignObjective::PolePlacement;
        let status = if pole_ok {
            RequirementStatus::Pass
        } else if mandatory {
            RequirementStatus::Fail
        } else {
            RequirementStatus::Warning
        };
        checks.push(RequirementCheck {
            id: "dominant_poles".into(),
            label: "Polos dominantes".into(),
            requested: format!("s = {}", fmt_complex(sd)),
            theoretical: format!("s = {}", fmt_complex(sd)),
            measured: format!("s = {}", fmt_complex(closest)),
            difference: format!("{:0.2e}", min_dist),
            status,
            mandatory,
            message: if pole_ok {
                "Polo dominante alocado exatamente na posição desejada.".into()
            } else {
                "Polo fechado mais próximo difere do polo nominal de 2ª ordem.".into()
            },
        });
        if status == RequirementStatus::Fail {
            failed_requirements.push("Polos dominantes".into());
        }
    }

    // 4. Overshoot Mp
    if let Some(mp_limit) = specs.max_overshoot_pct {
        let theo = specs
            .theoretical_mp_pct
            .map(|v| format!("{} %", fmt_val(v, 2)))
            .unwrap_or_else(|| "—".into());
        let mandatory = specs.objective == DesignObjective::TimeDomainConstraints;
        let (meas_str, diff_str, status, msg) = match measured_mp {
            Some(mp) => {
                let diff = mp - mp_limit;
                let ok = mp <= mp_limit + 1e-6;
                let st = if ok {
                    RequirementStatus::Pass
                } else if mandatory {
                    RequirementStatus::Fail
                } else {
                    RequirementStatus::Warning
                };
                let m = if ok {
                    format!(
                        "Sobressinal real ({}%) atende ao limite Mp ≤ {}%.",
                        fmt_val(mp, 2),
                        fmt_val(mp_limit, 2)
                    )
                } else {
                    format!(
                        "Sobressinal real ({}%) excede o limite solicitado Mp ≤ {}% em {} pp.",
                        fmt_val(mp, 2),
                        fmt_val(mp_limit, 2),
                        fmt_val(diff, 2)
                    )
                };
                (
                    format!("{} %", fmt_val(mp, 2)),
                    format!(
                        "{}{} pp",
                        if diff >= 0.0 { "+" } else { "" },
                        fmt_val(diff, 2)
                    ),
                    st,
                    m,
                )
            }
            None => (
                "Indefinido".into(),
                "—".into(),
                if mandatory {
                    RequirementStatus::Fail
                } else {
                    RequirementStatus::Warning
                },
                "Não foi possível medir o sobressinal (sistema instável ou ganho DC nulo).".into(),
            ),
        };
        checks.push(RequirementCheck {
            id: "overshoot".into(),
            label: "Sobressinal real (Mp)".into(),
            requested: format!("≤ {} %", fmt_val(mp_limit, 2)),
            theoretical: theo,
            measured: meas_str,
            difference: diff_str,
            status,
            mandatory,
            message: msg,
        });
        if status == RequirementStatus::Fail {
            failed_requirements.push(format!("Sobressinal real (Mp ≤ {}%)", fmt_val(mp_limit, 2)));
        }
    }

    // 5. Settling time ts
    if let Some(ts_spec) = specs.settling_time {
        let theo = specs
            .theoretical_ts_s
            .map(|v| format!("{} s", fmt_val(v, 3)))
            .unwrap_or_else(|| "—".into());
        let mandatory = specs.objective == DesignObjective::TimeDomainConstraints;
        let req_str = match ts_spec {
            SettlingSpec::AtMost(max_s) => format!(
                "≤ {} s ({}%)",
                fmt_val(max_s, 3),
                specs.settling_criterion_pct
            ),
            SettlingSpec::Equal { target, tolerance } => format!(
                "= {} ± {} s ({}%)",
                fmt_val(target, 3),
                fmt_val(tolerance, 2),
                specs.settling_criterion_pct
            ),
        };
        let (meas_str, diff_str, status, msg) = match measured_ts {
            Some(ts) => {
                let (ok, diff) = match ts_spec {
                    SettlingSpec::AtMost(max_s) => (ts <= max_s + 1e-6, ts - max_s),
                    SettlingSpec::Equal { target, tolerance } => {
                        ((ts - target).abs() <= tolerance + 1e-6, ts - target)
                    }
                };
                let st = if ok {
                    RequirementStatus::Pass
                } else if mandatory {
                    RequirementStatus::Fail
                } else {
                    RequirementStatus::Warning
                };
                let m = if ok {
                    format!(
                        "Tempo de acomodação definitivo ({} s) atende à especificação ({req_str}).",
                        fmt_val(ts, 3)
                    )
                } else {
                    format!(
                        "Tempo de acomodação definitivo ({} s) viola a especificação ({req_str}).",
                        fmt_val(ts, 3)
                    )
                };
                (
                    format!("{} s", fmt_val(ts, 3)),
                    format!("{}{} s", if diff >= 0.0 { "+" } else { "" }, fmt_val(diff, 3)),
                    st,
                    m,
                )
            }
            None => (
                "Não acomodou".into(),
                "—".into(),
                if mandatory {
                    RequirementStatus::Fail
                } else {
                    RequirementStatus::Warning
                },
                "A resposta não entrou definitivamente na faixa de tolerância no horizonte simulado.".into(),
            ),
        };
        checks.push(RequirementCheck {
            id: "settling_time".into(),
            label: format!(
                "Tempo de acomodação real (ts {}%)",
                specs.settling_criterion_pct
            ),
            requested: req_str.clone(),
            theoretical: theo,
            measured: meas_str,
            difference: diff_str,
            status,
            mandatory,
            message: msg,
        });
        if status == RequirementStatus::Fail {
            failed_requirements.push(format!("Tempo de acomodação ({req_str})"));
        }
    }

    // 6. Steady-state error (actuating error at summer)
    if specs.require_zero_step_error {
        let (meas_str, diff_str, status, msg) = match summer_ess {
            Some(ea) => {
                let ok = ea.abs() <= 1e-5;
                (
                    fmt_val(ea, 5),
                    fmt_val(ea, 5),
                    if ok {
                        RequirementStatus::Pass
                    } else {
                        RequirementStatus::Fail
                    },
                    if ok {
                        "Erro estacionário nulo no somador para entrada degrau unitário.".into()
                    } else {
                        format!(
                            "Erro estacionário não nulo no somador: e_a(∞) = {} ≠ 0.",
                            fmt_val(ea, 5)
                        )
                    },
                )
            }
            None => (
                "Indefinido".into(),
                "—".into(),
                RequirementStatus::Fail,
                "Erro estacionário indefinido (sistema instável).".into(),
            ),
        };
        checks.push(RequirementCheck {
            id: "steady_state_error".into(),
            label: "Erro estacionário (degrau)".into(),
            requested: "= 0".into(),
            theoretical: "0".into(),
            measured: meas_str,
            difference: diff_str,
            status,
            mandatory: true,
            message: msg,
        });
        if status == RequirementStatus::Fail {
            failed_requirements.push("Erro estacionário nulo (e(∞) = 0)".into());
        }
    }

    // 7. Damping ratio (xi): explicitly distinguishes mandatory equality, mandatory minimum,
    //    and optional didactic estimate, identifying which pole/pair it refers to.
    let eff_damping = specs
        .damping
        .or_else(|| specs.theoretical_xi.map(DampingSpec::DidacticEstimate));
    if let Some(d_spec) = eff_damping {
        let pole_ctx = modal
            .pole_label
            .as_deref()
            .unwrap_or("polos de malha fechada");
        let (req_str, theo_str, mandatory) = match d_spec {
            DampingSpec::Equal { target, tolerance } => (
                format!(
                    "= {} ± {} (obrigatória)",
                    fmt_val(target, 4),
                    fmt_val(tolerance, 3)
                ),
                fmt_val(target, 4),
                true,
            ),
            DampingSpec::AtLeast(min_xi) => (
                format!("≥ {} (obrigatória)", fmt_val(min_xi, 4)),
                fmt_val(min_xi, 4),
                true,
            ),
            DampingSpec::DidacticEstimate(est) => (
                format!("≈ {} (estimativa didática opcional)", fmt_val(est, 4)),
                fmt_val(est, 4),
                false,
            ),
        };
        let (meas_str, diff_str, status, msg) = match modal.measured_xi {
            Some(xi_m) => {
                let meas_with_pole = format!("{} [{}]", fmt_val(xi_m, 4), pole_ctx);
                match d_spec {
                    DampingSpec::Equal { target, tolerance } => {
                        let diff = xi_m - target;
                        let needs_complex = target < 1.0 - 1e-6;
                        let ok = (!needs_complex || modal.has_complex_pair)
                            && diff.abs() <= tolerance + 1e-6;
                        let st = if ok {
                            RequirementStatus::Pass
                        } else {
                            RequirementStatus::Fail
                        };
                        let m = if ok {
                            format!(
                                "Amortecimento calculado ({}) em {} atende à igualdade obrigatória ξ = {} ± {}.",
                                fmt_val(xi_m, 4),
                                pole_ctx,
                                fmt_val(target, 4),
                                fmt_val(tolerance, 3)
                            )
                        } else if needs_complex && !modal.has_complex_pair {
                            format!(
                                "Solicitada igualdade ξ = {} (subamortecido), mas todos os polos de malha fechada são reais ({}).",
                                fmt_val(target, 4),
                                pole_ctx
                            )
                        } else {
                            format!(
                                "Amortecimento calculado ({}) em {} viola a igualdade obrigatória ξ = {} ± {} (dif.: {}{}).",
                                fmt_val(xi_m, 4),
                                pole_ctx,
                                fmt_val(target, 4),
                                fmt_val(tolerance, 3),
                                if diff >= 0.0 { "+" } else { "" },
                                fmt_val(diff, 4)
                            )
                        };
                        (
                            meas_with_pole,
                            format!("{}{}", if diff >= 0.0 { "+" } else { "" }, fmt_val(diff, 4)),
                            st,
                            m,
                        )
                    }
                    DampingSpec::AtLeast(min_xi) => {
                        let diff = xi_m - min_xi;
                        let ok = xi_m >= min_xi - 1e-6;
                        let st = if ok {
                            RequirementStatus::Pass
                        } else {
                            RequirementStatus::Fail
                        };
                        let m = if ok {
                            format!(
                                "Amortecimento calculado ({}) em {} atende ao limite mínimo ξ ≥ {}.",
                                fmt_val(xi_m, 4),
                                pole_ctx,
                                fmt_val(min_xi, 4)
                            )
                        } else {
                            format!(
                                "Amortecimento calculado ({}) em {} fica abaixo do limite mínimo ξ ≥ {}.",
                                fmt_val(xi_m, 4),
                                pole_ctx,
                                fmt_val(min_xi, 4)
                            )
                        };
                        (
                            meas_with_pole,
                            format!("{}{}", if diff >= 0.0 { "+" } else { "" }, fmt_val(diff, 4)),
                            st,
                            m,
                        )
                    }
                    DampingSpec::DidacticEstimate(est) => {
                        let diff = xi_m - est;
                        let close = diff.abs() <= 0.02;
                        let st = if close {
                            RequirementStatus::Pass
                        } else {
                            RequirementStatus::Warning
                        };
                        let m = format!(
                            "Estimativa didática opcional ξ ≈ {}; valor calculado em {} é {} (não é igualdade obrigatória).",
                            fmt_val(est, 4),
                            pole_ctx,
                            fmt_val(xi_m, 4)
                        );
                        (
                            meas_with_pole,
                            format!("{}{}", if diff >= 0.0 { "+" } else { "" }, fmt_val(diff, 4)),
                            st,
                            m,
                        )
                    }
                }
            }
            None => (
                "Indefinido".into(),
                "—".into(),
                if mandatory {
                    RequirementStatus::Fail
                } else {
                    RequirementStatus::Warning
                },
                "Não foi possível calcular o coeficiente de amortecimento.".into(),
            ),
        };
        checks.push(RequirementCheck {
            id: "damping_ratio".into(),
            label: "Coeficiente de amortecimento (ξ)".into(),
            requested: req_str.clone(),
            theoretical: theo_str,
            measured: meas_str,
            difference: diff_str,
            status,
            mandatory,
            message: msg,
        });
        if mandatory && status == RequirementStatus::Fail {
            failed_requirements.push(format!("Coeficiente de amortecimento ({req_str})"));
        }
    }

    // 8. Natural frequency (wn): mandatory equality / minimum vs optional estimate
    let eff_wn = specs
        .natural_freq
        .or_else(|| specs.theoretical_wn.map(NaturalFreqSpec::OptionalEstimate));
    if let Some(wn_spec) = eff_wn {
        let pole_ctx = modal
            .pole_label
            .as_deref()
            .unwrap_or("polos de malha fechada");
        let (req_str, theo_str, mandatory) = match wn_spec {
            NaturalFreqSpec::Equal { target, tolerance } => (
                format!(
                    "= {} ± {} rad/s (obrigatória)",
                    fmt_val(target, 4),
                    fmt_val(tolerance, 3)
                ),
                format!("{} rad/s", fmt_val(target, 4)),
                true,
            ),
            NaturalFreqSpec::AtLeast(min_wn) => (
                format!("≥ {} rad/s (obrigatória)", fmt_val(min_wn, 4)),
                format!("{} rad/s", fmt_val(min_wn, 4)),
                true,
            ),
            NaturalFreqSpec::OptionalEstimate(est) => (
                format!("≈ {} rad/s (estimativa opcional)", fmt_val(est, 4)),
                format!("{} rad/s", fmt_val(est, 4)),
                false,
            ),
        };
        let (meas_str, diff_str, status, msg) = match modal.measured_wn {
            Some(wn_m) => {
                let meas_with_pole = format!("{} rad/s [{}]", fmt_val(wn_m, 4), pole_ctx);
                match wn_spec {
                    NaturalFreqSpec::Equal { target, tolerance } => {
                        let diff = wn_m - target;
                        let ok = diff.abs() <= tolerance + 1e-6;
                        (
                            meas_with_pole,
                            format!(
                                "{}{} rad/s",
                                if diff >= 0.0 { "+" } else { "" },
                                fmt_val(diff, 4)
                            ),
                            if ok {
                                RequirementStatus::Pass
                            } else {
                                RequirementStatus::Fail
                            },
                            if ok {
                                format!(
                                    "Frequência natural calculada ({} rad/s) atende à restrição obrigatória.",
                                    fmt_val(wn_m, 4)
                                )
                            } else {
                                format!(
                                    "Frequência natural calculada ({} rad/s) viola ωn = {} ± {} rad/s.",
                                    fmt_val(wn_m, 4),
                                    fmt_val(target, 4),
                                    fmt_val(tolerance, 3)
                                )
                            },
                        )
                    }
                    NaturalFreqSpec::AtLeast(min_wn) => {
                        let diff = wn_m - min_wn;
                        let ok = wn_m >= min_wn - 1e-6;
                        (
                            meas_with_pole,
                            format!(
                                "{}{} rad/s",
                                if diff >= 0.0 { "+" } else { "" },
                                fmt_val(diff, 4)
                            ),
                            if ok {
                                RequirementStatus::Pass
                            } else {
                                RequirementStatus::Fail
                            },
                            if ok {
                                format!(
                                    "Frequência natural ({} rad/s) atende a ωn ≥ {} rad/s.",
                                    fmt_val(wn_m, 4),
                                    fmt_val(min_wn, 4)
                                )
                            } else {
                                format!(
                                    "Frequência natural ({} rad/s) abaixo de ωn ≥ {} rad/s.",
                                    fmt_val(wn_m, 4),
                                    fmt_val(min_wn, 4)
                                )
                            },
                        )
                    }
                    NaturalFreqSpec::OptionalEstimate(est) => {
                        let diff = wn_m - est;
                        (
                            meas_with_pole,
                            format!(
                                "{}{} rad/s",
                                if diff >= 0.0 { "+" } else { "" },
                                fmt_val(diff, 4)
                            ),
                            if diff.abs() <= default_wn_tolerance(est) {
                                RequirementStatus::Pass
                            } else {
                                RequirementStatus::Warning
                            },
                            format!(
                                "Estimativa opcional ωn ≈ {} rad/s; obtido {} rad/s em {}.",
                                fmt_val(est, 4),
                                fmt_val(wn_m, 4),
                                pole_ctx
                            ),
                        )
                    }
                }
            }
            None => (
                "Indefinido".into(),
                "—".into(),
                if mandatory {
                    RequirementStatus::Fail
                } else {
                    RequirementStatus::Warning
                },
                "Frequência natural indefinida.".into(),
            ),
        };
        checks.push(RequirementCheck {
            id: "natural_frequency".into(),
            label: "Frequência natural (ωn)".into(),
            requested: req_str.clone(),
            theoretical: theo_str,
            measured: meas_str,
            difference: diff_str,
            status,
            mandatory,
            message: msg,
        });
        if mandatory && status == RequirementStatus::Fail {
            failed_requirements.push(format!("Frequência natural ({req_str})"));
        }
    }

    // 9. Optional explicit dominance check on the complex pair when required
    if specs.require_complex_dominance {
        let ok = modal.has_complex_pair && modal.complex_pair_is_dominant;
        checks.push(RequirementCheck {
            id: "complex_dominance".into(),
            label: "Dominância do par complexo".into(),
            requested: "Par complexo como modo dominante ativo".into(),
            theoretical: "Dominante".into(),
            measured: modal
                .pole_label
                .clone()
                .unwrap_or_else(|| "Indefinido".into()),
            difference: modal
                .non_dominant_residue_ratio
                .map(|r| format!("razão res. {}", fmt_val(r, 3)))
                .unwrap_or_else(|| "—".into()),
            status: if ok {
                RequirementStatus::Pass
            } else {
                RequirementStatus::Fail
            },
            mandatory: true,
            message: if ok {
                "O par complexo conjugado é o modo dominante ativo da malha fechada.".into()
            } else {
                "Há polo real ativo mais lento que o par complexo conjugado (ou ausência de par complexo).".into()
            },
        });
        if !ok {
            failed_requirements.push("Dominância do par complexo".into());
        }
    }

    let requirements_passed = closed_loop_stable
        && internally_stable
        && checks
            .iter()
            .all(|ck| !ck.mandatory || ck.status == RequirementStatus::Pass);
    let has_warnings = !warnings.is_empty()
        || checks
            .iter()
            .any(|ck| ck.status == RequirementStatus::Warning);
    let overall_status = if !requirements_passed {
        RequirementStatus::Fail
    } else if has_warnings {
        RequirementStatus::Warning
    } else {
        RequirementStatus::Pass
    };

    Ok(VerificationReport {
        overall_status,
        requirements_passed,
        numerically_valid: true,
        closed_loop_stable,
        internally_stable,
        closed_loop_poles: cl_poles,
        closed_loop_zeros: cl_zeros,
        dominant_pole,
        dominant_pole_ratio,
        non_dominant_poles,
        extra_closed_loop_zeros: extra_zeros,
        cancellations,
        measured_xi: modal.measured_xi,
        measured_wn: modal.measured_wn,
        damping_pole_label: modal.pole_label,
        has_complex_pair: modal.has_complex_pair,
        complex_pair_is_dominant: modal.complex_pair_is_dominant,
        non_dominant_residue_ratio: modal.non_dominant_residue_ratio,
        measured_overshoot_pct: measured_mp,
        measured_settling_time_s: measured_ts,
        measured_rise_time_s: measured_tr,
        measured_peak_time_s: measured_tp,
        measured_steady_state_y: steady_y,
        summer_steady_state_error: summer_ess,
        output_tracking_error,
        checks,
        warnings,
        failed_requirements,
    })
}

struct BuiltController {
    tf: TransferFunction,
    kp: f64,
    ki: f64,
    kd: f64,
    z: Option<f64>,
    zi: Option<f64>,
    p: Option<f64>,
    k: Option<f64>,
}

/// Build a TransferFunction from a parameter vector for a given ControllerStructure.
/// Parameters are stored in strictly positive canonical magnitudes (sign applied externally):
/// - P:    [kp_mag]
/// - PI:   [kp_mag, z]          => sgn * kp_mag * (s + z) / s
/// - PD:   [kd_mag, z]          => sgn * kd_mag * (s + z)
/// - Lead: [k_mag, z, ratio]    => sgn * k_mag * (s + z) / (s + z * (1 + ratio))
/// - PID:  [kd_mag, z1, z2]     => sgn * kd_mag * (s + z1) * (s + z2) / s
fn build_candidate_tf(
    structure: ControllerStructure,
    params: &[f64],
    sign: f64,
) -> Option<BuiltController> {
    if params
        .iter()
        .any(|v| !v.is_finite() || *v <= 1e-6 || *v > 1e5)
    {
        return None;
    }
    match structure {
        ControllerStructure::P => {
            let kp = sign * params[0];
            let tf = TransferFunction::new(vec![kp], vec![1.0]).ok()?;
            Some(BuiltController {
                tf,
                kp,
                ki: 0.0,
                kd: 0.0,
                z: None,
                zi: None,
                p: None,
                k: Some(kp),
            })
        }
        ControllerStructure::PI => {
            let kp = sign * params[0];
            let z = params[1];
            let ki = kp * z;
            let tf = TransferFunction::new(vec![kp, ki], vec![1.0, 0.0]).ok()?;
            Some(BuiltController {
                tf,
                kp,
                ki,
                kd: 0.0,
                z: Some(z),
                zi: None,
                p: Some(0.0),
                k: Some(kp),
            })
        }
        ControllerStructure::PD => {
            let kd = sign * params[0];
            let z = params[1];
            let kp = kd * z;
            let tf = TransferFunction::new(vec![kd, kp], vec![1.0]).ok()?;
            Some(BuiltController {
                tf,
                kp,
                ki: 0.0,
                kd,
                z: Some(z),
                zi: None,
                p: None,
                k: Some(kd),
            })
        }
        ControllerStructure::Lead => {
            let k = sign * params[0];
            let z = params[1];
            let p = z * (1.0 + params[2].max(0.25));
            let tf = TransferFunction::new(vec![k, k * z], vec![1.0, p]).ok()?;
            Some(BuiltController {
                tf,
                kp: k,
                ki: 0.0,
                kd: 0.0,
                z: Some(z),
                zi: None,
                p: Some(p),
                k: Some(k),
            })
        }
        ControllerStructure::PID => {
            let kp = sign * params[0];
            let ki = sign * params[1];
            let kd = sign * params[2];
            let tf = TransferFunction::new(vec![kd, kp, ki], vec![1.0, 0.0]).ok()?;
            // Real zeros if discriminant >= 0; None if complex conjugate pair (still LHP minimum-phase)
            let disc = kp * kp - 4.0 * kd * ki;
            let (zi, z) = if disc >= 0.0 {
                let s = disc.sqrt();
                let r1 = (kp.abs() - s) / (2.0 * kd.abs());
                let r2 = (kp.abs() + s) / (2.0 * kd.abs());
                (Some(r1), Some(r2))
            } else {
                (None, None)
            };
            Some(BuiltController {
                tf,
                kp,
                ki,
                kd,
                z,
                zi,
                p: Some(0.0),
                k: Some(kd),
            })
        }
    }
}

/// Fast constraint-violation + regularization cost for numerical refinement.
fn evaluate_candidate_cost(
    plant: &TransferFunction,
    h: &TransferFunction,
    structure: ControllerStructure,
    params: &[f64],
    sign: f64,
    specs: &DesignSpecs,
) -> (f64, bool) {
    let Some(built) = build_candidate_tf(structure, params, sign) else {
        return (1e12, false);
    };
    let (gc, kp, ki, kd) = (built.tf, built.kp, built.ki, built.kd);
    let fwd_num = poly::multiply(&gc.num, &plant.num);
    let fwd_den = poly::multiply(&gc.den, &plant.den);
    let loop_num = poly::multiply(&fwd_num, &h.num);
    let loop_den = poly::multiply(&fwd_den, &h.den);
    let char_poly = poly::add(&loop_den, &loop_num, 1.0);
    let Ok(poles) = poly::roots(&char_poly) else {
        return (1e12, false);
    };
    if poles.is_empty() {
        return (1e12, false);
    }
    let max_re = poles.iter().map(|p| p.re).fold(f64::NEG_INFINITY, f64::max);
    if max_re >= -1e-4 {
        return (1e9 + 1e6 * (max_re + 0.01).max(0.0), false);
    }

    let cl_num = poly::multiply(&fwd_num, &h.den);
    let Ok(raw_cl) = TransferFunction::new(cl_num.clone(), char_poly.clone()) else {
        return (1e12, false);
    };
    let sim_tf = if raw_cl.is_proper() {
        raw_cl
    } else {
        let Ok(mut f) = gc.multiply(plant) else {
            return (1e12, false);
        };
        while !f.is_proper() {
            let Ok(next) = f.multiply(&TransferFunction::new(vec![1.0], vec![0.001, 1.0]).unwrap())
            else {
                return (1e12, false);
            };
            f = next;
        }
        let Ok(cl) = f.feedback(h, -1.0) else {
            return (1e12, false);
        };
        cl
    };

    let modal = identify_damping_mode(&poles, &poles, &cl_num, &char_poly);

    let threshold = if specs.settling_criterion_pct == 5 {
        0.05
    } else {
        0.02
    };
    let spec_ts = match specs.settling_time {
        Some(SettlingSpec::AtMost(v)) => v,
        Some(SettlingSpec::Equal { target, .. }) => target,
        None => specs.theoretical_ts_s.unwrap_or(4.0),
    };
    let slowest = poles
        .iter()
        .map(|p| p.re.abs())
        .fold(f64::INFINITY, f64::min)
        .max(1e-3);
    let horizon = (10.0 / slowest).max(spec_ts * 2.5).clamp(1.0, 200.0);
    let Ok(m) = compute_step_metrics(&sim_tf, horizon, 500, threshold) else {
        return (1e11, false);
    };

    let mut penalty = 0.0;
    let mut feasible = true;

    // Mandatory damping ratio constraint
    if let Some(d_spec) = specs.damping {
        match d_spec {
            DampingSpec::Equal { target, tolerance } => match modal.measured_xi {
                Some(xi_m) => {
                    let needs_complex = target < 1.0 - 1e-6;
                    if needs_complex && !modal.has_complex_pair {
                        penalty += 5e5 + 1e4 * (1.0 - target);
                        feasible = false;
                    } else {
                        let err = (xi_m - target).abs();
                        let safe_tol = (tolerance * 0.4).max(1e-3);
                        if err > safe_tol {
                            penalty += 5e4 * ((err - safe_tol) + (err - safe_tol).powi(2));
                        }
                        penalty += 50.0 * err;
                        if err > tolerance {
                            feasible = false;
                        }
                    }
                }
                None => {
                    penalty += 1e6;
                    feasible = false;
                }
            },
            DampingSpec::AtLeast(min_xi) => match modal.measured_xi {
                Some(xi_m) => {
                    let deficit = min_xi - xi_m;
                    if deficit > 0.0 {
                        penalty += 5e4 * (deficit + deficit * deficit);
                        feasible = false;
                    }
                }
                None => {
                    penalty += 1e6;
                    feasible = false;
                }
            },
            DampingSpec::DidacticEstimate(_) => {}
        }
    }

    // Mandatory natural frequency constraint
    if let Some(wn_spec) = specs.natural_freq {
        match wn_spec {
            NaturalFreqSpec::Equal { target, tolerance } => match modal.measured_wn {
                Some(wn_m) => {
                    let err = (wn_m - target).abs();
                    let safe_tol = (tolerance * 0.4).max(1e-3);
                    if err > safe_tol {
                        penalty += 5e4 * ((err - safe_tol) + (err - safe_tol).powi(2));
                    }
                    penalty += 25.0 * err;
                    if err > tolerance {
                        feasible = false;
                    }
                }
                None => {
                    penalty += 1e6;
                    feasible = false;
                }
            },
            NaturalFreqSpec::AtLeast(min_wn) => match modal.measured_wn {
                Some(wn_m) => {
                    let deficit = min_wn - wn_m;
                    if deficit > 0.0 {
                        penalty += 5e4 * (deficit + deficit * deficit);
                        feasible = false;
                    }
                }
                None => {
                    penalty += 1e6;
                    feasible = false;
                }
            },
            NaturalFreqSpec::OptionalEstimate(_) => {}
        }
    }

    if specs.require_complex_dominance
        && !(modal.has_complex_pair && modal.complex_pair_is_dominant)
    {
        penalty += 2e5;
        feasible = false;
    }

    if let Some(mp_max) = specs.max_overshoot_pct {
        // Keep a tiny 0.05 pp safety margin during fast 500-point search so 2500-point verification passes cleanly.
        let excess = m.overshoot_pct - (mp_max - 0.05);
        if excess > 0.0 {
            penalty += 1e4 * (excess + excess * excess);
        }
        if m.overshoot_pct > mp_max {
            feasible = false;
        }
    }

    if let Some(ts_spec) = specs.settling_time {
        match m.settling_time {
            Some(ts) => match ts_spec {
                SettlingSpec::AtMost(max_s) => {
                    let excess = ts - (max_s - 0.01);
                    if excess > 0.0 {
                        penalty += 1e4 * (excess + excess * excess);
                    }
                    if ts > max_s {
                        feasible = false;
                    }
                }
                SettlingSpec::Equal { target, tolerance } => {
                    let err = (ts - target).abs();
                    let safe_tol = (tolerance * 0.4).max(0.005);
                    if err > safe_tol {
                        penalty += 1e4 * ((err - safe_tol) + (err - safe_tol).powi(2));
                    }
                    penalty += 25.0 * err;
                    if err > tolerance {
                        feasible = false;
                    }
                }
            },
            None => {
                penalty += 1e6;
                feasible = false;
            }
        }
    }

    if specs.require_zero_step_error {
        let Ok(ol) = TransferFunction::new(loop_num, loop_den) else {
            return (1e12, false);
        };
        if system_type(&ol) == 0 {
            let kp_dc = exact_dc_gain(&ol);
            let ess = 1.0 / (1.0 + kp_dc);
            penalty += 1e6 * ess * ess;
            if ess.abs() > 1e-5 {
                feasible = false;
            }
        }
    }

    // Mild regularization favoring modest controller gains, avoiding ultra-slow near-origin
    // pole-zero dipoles, and preferring cleaner modal dominance.
    let sigma_ref = (4.0 / spec_ts.max(0.1)).clamp(0.1, 20.0);
    let slow_pole_pen = if slowest < 0.05 * sigma_ref {
        50.0 * (0.05 * sigma_ref - slowest) / (0.05 * sigma_ref)
    } else {
        0.0
    };
    let gain_reg = 1e-3 * (kp.abs() + ki.abs() + kd.abs()).ln_1p();
    let res_reg = modal
        .non_dominant_residue_ratio
        .map(|r| 0.05 * r.min(20.0))
        .unwrap_or(0.0);
    (penalty + slow_pole_pen + gain_reg + res_reg, feasible)
}

/// Adaptive coordinate + coupled-diagonal pattern search (derivative-free, deterministic, bounded).
fn pattern_search_refine(
    plant: &TransferFunction,
    h: &TransferFunction,
    structure: ControllerStructure,
    mut best_params: Vec<f64>,
    sign: f64,
    specs: &DesignSpecs,
    eval_counter: &mut usize,
) -> Vec<f64> {
    let dim = best_params.len();
    let (mut best_cost, _) =
        evaluate_candidate_cost(plant, h, structure, &best_params, sign, specs);
    *eval_counter += 1;
    let mut step: Vec<f64> = best_params.iter().map(|&v| (v * 0.35).max(0.05)).collect();

    for _iter in 0..110 {
        let mut improved = false;
        for d in 0..dim {
            for dir in [1.0, -1.0] {
                let mut cand = best_params.clone();
                cand[d] += dir * step[d];
                if cand[d] <= 1e-4 {
                    continue;
                }
                let (cost, _) = evaluate_candidate_cost(plant, h, structure, &cand, sign, specs);
                *eval_counter += 1;
                if cost < best_cost - 1e-12 {
                    best_cost = cost;
                    best_params = cand;
                    step[d] *= 1.15;
                    improved = true;
                    break;
                }
            }
        }
        // Coupled pairwise moves help traverse narrow constraint valleys (e.g., exact xi + exact ts)
        if !improved && dim >= 2 {
            'diag: for i in 0..dim {
                for j in (i + 1)..dim {
                    for &(si, sj) in &[(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                        let mut cand = best_params.clone();
                        cand[i] += si * step[i];
                        cand[j] += sj * step[j];
                        if cand[i] <= 1e-4 || cand[j] <= 1e-4 {
                            continue;
                        }
                        let (cost, _) =
                            evaluate_candidate_cost(plant, h, structure, &cand, sign, specs);
                        *eval_counter += 1;
                        if cost < best_cost - 1e-12 {
                            best_cost = cost;
                            best_params = cand;
                            improved = true;
                            break 'diag;
                        }
                    }
                }
            }
        }
        if !improved {
            for s in &mut step {
                *s *= 0.55;
            }
            if step.iter().all(|&s| s < 1e-4) {
                break;
            }
        }
    }
    best_params
}

/// Check algebraic infeasibility of a controller structure before numerical search.
fn algebraic_infeasibility_reason(
    plant: &TransferFunction,
    h: &TransferFunction,
    structure: ControllerStructure,
    specs: &DesignSpecs,
) -> Option<String> {
    let ol = plant.multiply(h).ok()?;
    let otype = system_type(&ol);
    if specs.require_zero_step_error && otype == 0 && !structure.has_integrator() {
        return Some(format!(
            "Inviabilidade comprovada — estrutura {} para e(∞) = 0 ao degrau: a planta em malha aberta G(s)H(s) é Tipo 0 (sem polo na origem), resultando em erro estacionário finito e(∞) = 1/(1 + Kp) ≠ 0 sem ação integral.",
            structure.as_str()
        ));
    }
    // Pure double integrator (s^2 in denominator, no finite zeros) cannot be stabilized by P or PI alone
    let ol_zeros = ol.zeros().unwrap_or_default();
    if otype >= 2
        && ol_zeros.is_empty()
        && matches!(structure, ControllerStructure::P | ControllerStructure::PI)
    {
        return Some(format!(
            "Inviabilidade comprovada — estrutura {} para planta Tipo {} sem zeros finitos: o lugar das raízes permanece sobre o eixo imaginário ou no semiplano direito (impossível estabilizar sem avanço de fase/ação derivativa).",
            structure.as_str(),
            otype
        ));
    }
    // Exact proof for 2nd-order undamped plants G(s) = k/(s^2 + c) with c >= 0 under unity feedback and PD:
    // With k*Kd = 2*xi*wn and c + k*Kp = wn^2:
    //   T(s) = (2*xi*wn*s + wn^2 - c) / (s^2 + 2*xi*wn*s + wn^2).
    // Steady-state gain is y(inf) = T(0) = alpha = 1 - c/wn^2.
    // - If c = 0 (double integrator): alpha = 1, T(wn*u) = (2*xi*u + 1)/(u^2 + 2*xi*u + 1), strictly time-scale
    //   invariant for all wn > 0 => constant overshoot Mp(xi).
    // - If c > 0 (undamped oscillator, e.g. 1/(s^2 + 1)): positive steady-state tracking requires k*Kp > 0
    //   (wn^2 > c), so alpha = 1 - c/wn^2 in (0, 1). Normalizing by y(inf) = alpha yields
    //   T_norm(wn*u) = T(wn*u)/alpha = ((2*xi/alpha)*u + 1)/(u^2 + 2*xi*u + 1) = H0(u) + (2*xi/alpha)*u*H0(u).
    //   In dimensionless time tau = wn*t, h0(tau) = L^{-1}{u*H0(u)} > 0 on (0, pi/sqrt(1 - xi^2)), and since
    //   2*xi/alpha > 2*xi strictly, max_tau y_norm(tau) >= y_norm(tau*) > max_tau y_{alpha=1}(tau) for every wn > sqrt(c).
    let h_is_unity =
        h.num.len() == 1 && h.den.len() == 1 && (h.num[0] / h.den[0] - 1.0).abs() < 1e-12;
    if structure == ControllerStructure::PD
        && h_is_unity
        && ol.num.len() == 1
        && ol.den.len() == 3
        && ol.den[1].abs() < 1e-12
        && ol.den[2] / ol.den[0] >= -1e-12
        && let (
            Some(DampingSpec::Equal {
                target: xi0,
                tolerance: xi_tol,
            }),
            Some(mp_max),
        ) = (specs.damping, specs.max_overshoot_pct)
        && xi0 > 0.0
        && xi0 < 1.0
    {
        let c_val = ol.den[2] / ol.den[0];
        let xi_upper = (xi0 + xi_tol).min(0.999);
        // Scale-invariant lower bound for alpha = 1: T_ref(u) = (2*xi_upper*u + 1)/(u^2 + 2*xi_upper*u + 1)
        if let Ok(t_ref) =
            TransferFunction::new(vec![2.0 * xi_upper, 1.0], vec![1.0, 2.0 * xi_upper, 1.0])
            && let Ok(m_ref) = compute_step_metrics(&t_ref, 25.0, 2000, 0.02)
            && m_ref.overshoot_pct > mp_max + 0.05
        {
            if c_val <= 1e-12 {
                return Some(format!(
                    "Inviabilidade matemática comprovada — PD para duplo integrador G(s)=k/s² com igualdade exata ξ = {} ± {} e Mp ≤ {}%: com y(∞)=1, T(ωn·u) = (2ξu + 1)/(u² + 2ξu + 1) é invariante em escala e impõe sobressinal constante ≥ {}% para todo ωn > 0.",
                    fmt_val(xi0, 4),
                    fmt_val(xi_tol, 3),
                    fmt_val(mp_max, 2),
                    fmt_val(m_ref.overshoot_pct, 2)
                ));
            } else {
                return Some(format!(
                    "Inviabilidade matemática comprovada — PD para oscilador não amortecido G(s)=k/(s² + {}) com igualdade exata ξ = {} ± {} e Mp ≤ {}%: qualquer PD estabilizante com rastreamento positivo exige ωn² > {}, resultando em ganho estacionário y(∞) = α = 1 - {}/ωn² ∈ (0, 1) e função normalizada T(ωn·u)/y(∞) = ((2ξ/α)u + 1)/(u² + 2ξu + 1); como 2ξ/α > 2ξ e h₀(τ) > 0 no primeiro semiciclo, o sobressinal percentual é estritamente superior à cota de {}% para todo ωn > √{}.",
                    fmt_val(c_val, 4),
                    fmt_val(xi0, 4),
                    fmt_val(xi_tol, 3),
                    fmt_val(mp_max, 2),
                    fmt_val(c_val, 4),
                    fmt_val(c_val, 4),
                    fmt_val(m_ref.overshoot_pct, 2),
                    fmt_val(c_val, 4)
                ));
            }
        }
    }
    None
}

/// Generic constrained synthesis for a specific controller structure.
pub fn synthesize_for_structure(
    plant: &TransferFunction,
    feedback: Option<&TransferFunction>,
    structure: ControllerStructure,
    specs: &DesignSpecs,
) -> Result<SynthesisOutcome> {
    let unity = TransferFunction::scalar(1.0);
    let h = feedback.unwrap_or(&unity);
    let ol = plant.multiply(h)?;

    if let Some(reason) = algebraic_infeasibility_reason(plant, h, structure, specs) {
        return Ok(SynthesisOutcome {
            feasible: false,
            feasibility_class: FeasibilityClass::ProvenInfeasible,
            selected: None,
            infeasibility_reasons: vec![reason],
            alternatives: vec![],
            candidates_evaluated: 0,
        });
    }

    let sign = plant_sign(&ol);
    let poles = ol.poles()?;
    let stable_real_poles: Vec<f64> = poles
        .iter()
        .filter(|p| p.im.abs() < 1e-4 && p.re < -1e-4)
        .map(|p| -p.re)
        .collect();

    let ts_ref = match specs.settling_time {
        Some(SettlingSpec::AtMost(v)) => v,
        Some(SettlingSpec::Equal { target, .. }) => target,
        None => specs.theoretical_ts_s.unwrap_or(4.0),
    };
    let sigma_scale = (4.0 / ts_ref.max(0.1)).clamp(0.1, 20.0);
    let gain_scale = (ol.den[0] / ol.num[0]).abs().clamp(1e-3, 1e3);

    // Generate generic structured parameter seeds
    let mut seeds: Vec<(String, Vec<f64>)> = Vec::new();
    match structure {
        ControllerStructure::P => {
            for exp in -20..=20 {
                let k = gain_scale * 10.0_f64.powf(exp as f64 * 0.15);
                seeds.push(("grid_p".into(), vec![k]));
            }
        }
        ControllerStructure::PI => {
            // 1. Generic stable real plant pole cancellation family (z = -p_i)
            for &zp in &stable_real_poles {
                for &mult in &[0.2, 0.5, 0.8, 1.0, 1.4, 2.0, 3.5, 6.0, 12.0] {
                    seeds.push((
                        "pole_cancellation_pi".into(),
                        vec![gain_scale * sigma_scale * mult, zp],
                    ));
                }
            }
            // 2. General 2D (Kp, z) grid
            for &z_mult in &[0.1, 0.3, 0.6, 1.0, 1.8, 3.5, 7.0] {
                for &k_mult in &[0.2, 0.6, 1.5, 4.0, 10.0, 25.0] {
                    seeds.push((
                        "constrained_pi".into(),
                        vec![gain_scale * sigma_scale * k_mult, sigma_scale * z_mult],
                    ));
                }
            }
        }
        ControllerStructure::PD => {
            // 1. Generic stable real plant pole cancellation family (z = -p_i)
            for &zp in &stable_real_poles {
                for &mult in &[0.2, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0] {
                    seeds.push((
                        "pole_cancellation_pd".into(),
                        vec![gain_scale * sigma_scale * mult, zp],
                    ));
                }
            }
            // 2. General 2D (Kd, z) grid spanning underdamped, critically damped, and overdamped
            for &z_mult in &[0.2, 0.5, 0.8, 1.0, 1.5, 2.0, 3.0, 5.0] {
                for &k_mult in &[0.25, 0.6, 1.2, 2.0, 3.5, 6.0, 12.0, 25.0, 50.0] {
                    seeds.push((
                        "constrained_pd".into(),
                        vec![gain_scale * sigma_scale * k_mult, sigma_scale * z_mult],
                    ));
                }
            }
        }
        ControllerStructure::Lead => {
            for &z_mult in &[0.3, 0.7, 1.2, 2.0, 3.5] {
                for &ratio in &[1.5, 4.0, 10.0] {
                    for &k_mult in &[0.5, 2.0, 8.0, 30.0] {
                        seeds.push((
                            "constrained_lead".into(),
                            vec![
                                gain_scale * sigma_scale * k_mult,
                                sigma_scale * z_mult,
                                ratio,
                            ],
                        ));
                    }
                }
            }
        }
        ControllerStructure::PID => {
            // Parameterized directly in canonical PID gains [kp_mag, ki_mag, kd_mag] so both
            // real-zero and complex-conjugate-zero minimum-phase PIDs are represented uniformly.
            // 1. Double-pole or single-pole cancellation seeds
            for &zp in &stable_real_poles {
                for &k_mult in &[0.5, 2.0, 8.0, 30.0, 100.0] {
                    let kd = gain_scale * k_mult;
                    seeds.push((
                        "pole_cancellation_pid".into(),
                        vec![kd * (2.0 * zp), kd * zp * zp, kd],
                    ));
                    let z2 = sigma_scale * 1.5;
                    seeds.push((
                        "pole_cancellation_pid".into(),
                        vec![kd * (zp + z2), kd * zp * z2, kd],
                    ));
                }
            }
            // 2. General real-zero (well-separated & equal) and complex-zero PID seeds
            for &z1_m in &[0.06, 0.25, 0.7, 1.5] {
                for &z2_m in &[0.4, 0.9, 2.0] {
                    for &k_mult in &[0.5, 2.0, 8.0, 20.0, 50.0] {
                        let kd = gain_scale * sigma_scale * k_mult;
                        let z1 = sigma_scale * z1_m;
                        let z2 = sigma_scale * z2_m;
                        seeds.push((
                            "constrained_pid".into(),
                            vec![kd * (z1 + z2), kd * z1 * z2, kd],
                        ));
                    }
                }
            }
            // 3. Complex-conjugate-zero minimum-phase PID seeds (kp^2 < 4*kd*ki)
            for &wn_z_m in &[0.5, 1.0, 1.8] {
                for &xi_z in &[0.35, 0.65] {
                    for &k_mult in &[0.5, 2.0, 5.0, 15.0] {
                        let kd = gain_scale * sigma_scale * k_mult;
                        let wz = sigma_scale * wn_z_m;
                        seeds.push((
                            "constrained_pid_complex_zeros".into(),
                            vec![kd * (2.0 * xi_z * wz), kd * wz * wz, kd],
                        ));
                    }
                }
            }
        }
    }

    let mut eval_counter = 0usize;
    let mut ranked: Vec<(f64, String, Vec<f64>)> = Vec::with_capacity(seeds.len());
    for (tag, params) in seeds {
        let (cost, _) = evaluate_candidate_cost(plant, h, structure, &params, sign, specs);
        eval_counter += 1;
        ranked.push((cost, tag, params));
    }
    ranked.sort_by(|a, b| a.0.total_cmp(&b.0));

    // Refine top 6 seeds and perform full 2500-point verification on each
    let mut best_feasible: Option<(f64, SynthesizedController)> = None;
    let mut best_infeasible: Option<(f64, SynthesizedController)> = None;

    for (_, tag, seed) in ranked.into_iter().take(6) {
        let refined =
            pattern_search_refine(plant, h, structure, seed, sign, specs, &mut eval_counter);
        let Some(built) = build_candidate_tf(structure, &refined, sign) else {
            continue;
        };
        let Ok(verification) = verify_controller(plant, feedback, &built.tf, specs) else {
            continue;
        };
        let (score, _) = evaluate_candidate_cost(plant, h, structure, &refined, sign, specs);
        let synth = SynthesizedController {
            structure,
            transfer: built.tf,
            kp: built.kp,
            ki: built.ki,
            kd: built.kd,
            z: built.z,
            zi: built.zi,
            p: built.p,
            k: built.k,
            method_used: format!("{tag}_refined"),
            verification,
        };
        if synth.verification.requirements_passed {
            if best_feasible.as_ref().is_none_or(|(s, _)| score < *s) {
                best_feasible = Some((score, synth));
            }
        } else if best_infeasible.as_ref().is_none_or(|(s, _)| score < *s) {
            best_infeasible = Some((score, synth));
        }
    }

    if let Some((_, winner)) = best_feasible {
        Ok(SynthesisOutcome {
            feasible: true,
            feasibility_class: FeasibilityClass::Feasible,
            selected: Some(winner),
            infeasibility_reasons: vec![],
            alternatives: vec![],
            candidates_evaluated: eval_counter,
        })
    } else {
        let mut reasons = Vec::new();
        let selected = best_infeasible.map(|(_, c)| {
            if !c.verification.failed_requirements.is_empty() {
                reasons.push(format!(
                    "Solução não encontrada nos limites de busca numérica para {} (não implica prova de inviabilidade matemática); requisitos violados no melhor candidato: {}.",
                    structure.as_str(),
                    c.verification.failed_requirements.join(", ")
                ));
            }
            c
        });
        if reasons.is_empty() {
            reasons.push(format!(
                "Solução não encontrada nos limites de busca numérica para {} (nenhum candidato estabilizante viável localizado).",
                structure.as_str()
            ));
        }
        Ok(SynthesisOutcome {
            feasible: false,
            feasibility_class: FeasibilityClass::NotFoundWithinSearchBounds,
            selected,
            infeasibility_reasons: reasons,
            alternatives: vec![],
            candidates_evaluated: eval_counter,
        })
    }
}

/// Automatically select the simplest controller structure (P -> PI -> PD -> Lead -> PID)
/// that satisfies all specifications, returning explicit infeasibility diagnostics for
/// rejected simpler structures.
pub fn synthesize_simplest(
    plant: &TransferFunction,
    feedback: Option<&TransferFunction>,
    specs: &DesignSpecs,
) -> Result<SynthesisOutcome> {
    let mut total_evals = 0usize;
    let mut infeasibility_reasons = Vec::new();
    let mut best_fallback: Option<SynthesizedController> = None;
    let mut all_proven_infeasible = true;

    for structure in ControllerStructure::complexity_order() {
        let outcome = synthesize_for_structure(plant, feedback, structure, specs)?;
        total_evals += outcome.candidates_evaluated;
        if outcome.feasible
            && let Some(winner) = outcome.selected
        {
            return Ok(SynthesisOutcome {
                feasible: true,
                feasibility_class: FeasibilityClass::Feasible,
                selected: Some(winner),
                infeasibility_reasons,
                alternatives: vec![],
                candidates_evaluated: total_evals,
            });
        }
        if outcome.feasibility_class != FeasibilityClass::ProvenInfeasible {
            all_proven_infeasible = false;
        }
        infeasibility_reasons.extend(outcome.infeasibility_reasons);
        if best_fallback.is_none() {
            best_fallback = outcome.selected;
        }
    }

    Ok(SynthesisOutcome {
        feasible: false,
        feasibility_class: if all_proven_infeasible {
            FeasibilityClass::ProvenInfeasible
        } else {
            FeasibilityClass::NotFoundWithinSearchBounds
        },
        selected: best_fallback,
        infeasibility_reasons,
        alternatives: vec![],
        candidates_evaluated: total_evals,
    })
}
