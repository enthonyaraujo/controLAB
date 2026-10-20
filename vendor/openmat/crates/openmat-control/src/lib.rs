//! Continuous, real SISO models. Independent of the language and desktop.
mod simulation;
mod time_grid;

use std::fmt;

pub const MAX_ORDER: usize = 16;
pub const MAX_SAMPLES: usize = 100_000;
pub(crate) const MAX_WORK: usize = 50_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlError(pub String,pub Option<openmat_numeric::execution::ResourceFailure>);
impl From<openmat_numeric::execution::ResourceFailure> for ControlError{fn from(e:openmat_numeric::execution::ResourceFailure)->Self{Self(e.to_string(),Some(e))}}
impl fmt::Display for ControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ControlError {}
pub type Result<T> = std::result::Result<T, ControlError>;
pub(crate) fn error(message: &str) -> ControlError {
    ControlError(message.into(),None)
}

/// Coefficients in descending powers of s; denominator leading coefficient is 1.
/// No approximate pole-zero cancellation is performed.
#[derive(Clone, Debug, PartialEq)]
pub struct TransferFunction {
    numerator: Vec<f64>,
    denominator: Vec<f64>,
}
impl TransferFunction {
    pub fn new(numerator: &[f64], denominator: &[f64]) -> Result<Self> {
        openmat_numeric::execution::checkpoint(1)?;
        openmat_numeric::execution::payload(numerator.len()+denominator.len(),8)?;
        let mut numerator = polynomial(numerator)?;
        let mut denominator = polynomial(denominator)?;
        if denominator[0] == 0. {
            return Err(error("tf: denominador não pode ser zero"));
        }
        let leading = denominator[0];
        for value in numerator.iter_mut().chain(denominator.iter_mut()) {
            *value /= leading;
            if !value.is_finite() {
                return Err(error("tf: normalização excedeu a representação f64"));
            }
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
    pub fn scalar(gain: f64) -> Result<Self> {
        Self::new(&[gain], &[1.])
    }
    pub fn laplace() -> Self {
        Self {
            numerator: vec![1., 0.],
            denominator: vec![1.],
        }
    }
    pub fn numerator(&self) -> &[f64] {
        &self.numerator
    }
    pub fn denominator(&self) -> &[f64] {
        &self.denominator
    }
    pub fn order(&self) -> usize {
        self.denominator.len() - 1
    }
    pub fn is_proper(&self) -> bool {
        self.numerator.len() <= self.denominator.len()
    }
    pub fn scale(&self, gain: f64) -> Result<Self> {
        let numerator: Vec<_> = self.numerator.iter().map(|x| x * gain).collect();
        Self::new(&numerator, &self.denominator)
    }
    pub fn add(&self, other: &Self) -> Result<Self> {
        Self::new(
            &add_polynomials(
                &multiply(&self.numerator, &other.denominator)?,
                &multiply(&other.numerator, &self.denominator)?,
                1.,
            ),
            &multiply(&self.denominator, &other.denominator)?,
        )
    }
    pub fn subtract(&self, other: &Self) -> Result<Self> {
        self.add(&other.scale(-1.)?)
    }
    pub fn multiply(&self, other: &Self) -> Result<Self> {
        Self::new(
            &multiply(&self.numerator, &other.numerator)?,
            &multiply(&self.denominator, &other.denominator)?,
        )
    }
    pub fn divide(&self, other: &Self) -> Result<Self> {
        Self::new(
            &multiply(&self.numerator, &other.denominator)?,
            &multiply(&self.denominator, &other.numerator)?,
        )
    }
    pub fn power(&self, exponent: f64) -> Result<Self> {
        if !exponent.is_finite() || exponent.fract() != 0. || exponent.abs() > MAX_ORDER as f64 {
            return Err(error("tf: expoente requer inteiro entre -16 e 16"));
        }
        let base = if exponent < 0. {
            Self::scalar(1.)?.divide(self)?
        } else {
            self.clone()
        };
        let mut value = Self::scalar(1.)?;
        for _ in 0..exponent.abs() as usize {
            openmat_numeric::execution::checkpoint(1)?;
            value = value.multiply(&base)?;
        }
        Ok(value)
    }
    /// sign = -1 for negative feedback, +1 for positive feedback.
    pub fn feedback(&self, other: &Self, sign: f64) -> Result<Self> {
        if sign != -1. && sign != 1. {
            return Err(error("feedback: sinal deve ser -1 ou +1"));
        }
        Self::new(
            &multiply(&self.numerator, &other.denominator)?,
            &add_polynomials(
                &multiply(&self.denominator, &other.denominator)?,
                &multiply(&self.numerator, &other.numerator)?,
                -sign,
            ),
        )
    }
    /// Ideal parallel PID: Kp + Ki/s + Kd*s, represented as a transfer function.
    pub fn pid(kp: f64, ki: f64, kd: f64) -> Result<Self> {
        if ![kp, ki, kd].iter().all(|x| x.is_finite()) {
            return Err(error("pid: ganhos devem ser reais e finitos"));
        }
        if ki == 0. {
            Self::new(&[kd, kp], &[1.])
        } else {
            Self::new(&[kd, kp, ki], &[1., 0.])
        }
    }
    /// Limit as s approaches zero. Common exact powers of s are accounted for
    /// without changing the stored polynomials or cancelling approximate roots.
    pub fn dc_gain(&self) -> f64 {
        let Some(num_zeros) = self.numerator.iter().rev().position(|&x| x != 0.) else {
            return 0.;
        };
        let den_zeros = self
            .denominator
            .iter()
            .rev()
            .position(|&x| x != 0.)
            .unwrap();
        let gain = self.numerator[self.numerator.len() - num_zeros - 1]
            / self.denominator[self.denominator.len() - den_zeros - 1];
        match num_zeros.cmp(&den_zeros) {
            std::cmp::Ordering::Greater => 0.,
            std::cmp::Ordering::Equal => gain,
            std::cmp::Ordering::Less => f64::INFINITY.copysign(gain),
        }
    }
    /// Zero initial state, unit step. Returns one value for each supplied time.
    pub fn step(&self, times: &[f64]) -> Result<Vec<f64>> {
        simulation::step(self, times)
    }
    /// Bounded automatic grid, selected independently of any frontend.
    /// First/second order pole time scales are analytic; higher orders use a
    /// coefficient heuristic, not an eigenvalue or stability calculation.
    pub fn step_times(&self) -> Result<Vec<f64>> {
        time_grid::automatic(self)
    }
}
impl fmt::Display for TransferFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tf({:?}, {:?}) [SISO contínuo]",
            self.numerator, self.denominator
        )
    }
}
fn polynomial(values: &[f64]) -> Result<Vec<f64>> {
    if values.is_empty() || values.len() > MAX_ORDER + 1 || values.iter().any(|x| !x.is_finite()) {
        return Err(error(
            "tf: vetor não vazio de coeficientes finitos, até grau 16",
        ));
    }
    let first = values
        .iter()
        .position(|&x| x != 0.)
        .unwrap_or(values.len() - 1);
    Ok(values[first..].to_vec())
}
fn multiply(a: &[f64], b: &[f64]) -> Result<Vec<f64>> {
    let length = a.len() + b.len() - 1;
    if length > MAX_ORDER + 1 {
        return Err(error("tf: operação excede o grau máximo 16"));
    }
    let mut result = vec![0.; length];
    for (i, &x) in a.iter().enumerate() {
        for (j, &y) in b.iter().enumerate() {
            result[i + j] += x * y;
        }
    }
    Ok(result)
}
fn add_polynomials(a: &[f64], b: &[f64], sign: f64) -> Vec<f64> {
    let length = a.len().max(b.len());
    let mut result = vec![0.; length];
    for (i, &x) in a.iter().enumerate() {
        result[length - a.len() + i] += x;
    }
    for (i, &x) in b.iter().enumerate() {
        result[length - b.len() + i] += sign * x;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn polynomial_arithmetic_models_the_motor() {
        let s = TransferFunction::laplace();
        let denominator = s
            .power(2.)
            .unwrap()
            .add(&s.scale(3.).unwrap())
            .unwrap()
            .add(&TransferFunction::scalar(10.).unwrap())
            .unwrap();
        let motor = TransferFunction::scalar(10.)
            .unwrap()
            .divide(&denominator)
            .unwrap();
        assert_eq!(motor.numerator(), &[10.]);
        assert_eq!(motor.denominator(), &[1., 3., 10.]);
        let closed = motor
            .feedback(&TransferFunction::scalar(1.).unwrap(), -1.)
            .unwrap();
        assert_eq!(closed.denominator(), &[1., 3., 20.]);
        assert_eq!(closed.dc_gain(), 0.5);
        assert_eq!(s.power(-1.).unwrap().denominator(), &[1., 0.]);
    }
    #[test]
    fn feedback_sign_and_nonunity_sensor() {
        let g = TransferFunction::new(&[2.], &[1., 1.]).unwrap();
        let h = TransferFunction::new(&[3.], &[1., 4.]).unwrap();
        let closed = g.feedback(&h, -1.).unwrap();
        assert_eq!(closed.numerator(), &[2., 8.]);
        assert_eq!(closed.denominator(), &[1., 5., 10.]);
        assert_eq!(g.feedback(&h, 1.).unwrap().denominator(), &[1., 5., -2.]);
        assert!(TransferFunction::scalar(1.)
            .unwrap()
            .feedback(&TransferFunction::scalar(1.).unwrap(), 1.)
            .is_err());
    }
    #[test]
    fn ideal_pid_preserves_gain_and_integrator() {
        assert_eq!(
            TransferFunction::pid(2., 3., 4.).unwrap(),
            TransferFunction::new(&[4., 2., 3.], &[1., 0.]).unwrap()
        );
        assert_eq!(
            TransferFunction::pid(2., 0., 0.).unwrap(),
            TransferFunction::scalar(2.).unwrap()
        );
        assert_eq!(
            TransferFunction::pid(0., 3., 0.).unwrap().dc_gain(),
            f64::INFINITY
        );
    }
    #[test]
    fn dc_gain_handles_exact_cancellation_at_the_origin() {
        assert_eq!(
            TransferFunction::new(&[2., 0.], &[1., 0.])
                .unwrap()
                .dc_gain(),
            2.
        );
        assert_eq!(
            TransferFunction::new(&[0.], &[1., 0.]).unwrap().dc_gain(),
            0.
        );
        assert_eq!(
            TransferFunction::new(&[1., 0., 0.], &[1., 0.])
                .unwrap()
                .dc_gain(),
            0.
        );
        assert_eq!(
            TransferFunction::new(&[-1., 0.], &[1., 0., 0.])
                .unwrap()
                .dc_gain(),
            f64::NEG_INFINITY
        );
    }
    #[test]
    fn invalid_models_and_unbounded_operations_are_rejected() {
        for (n, d) in [
            (vec![], vec![1.]),
            (vec![1.], vec![0.]),
            (vec![f64::NAN], vec![1.]),
            (vec![1.], vec![f64::INFINITY]),
            (vec![f64::MAX], vec![f64::MIN_POSITIVE]),
        ] {
            assert!(TransferFunction::new(&n, &d).is_err());
        }
        let s = TransferFunction::laplace();
        assert!(s.power(1.5).is_err());
        assert!(s.power(17.).is_err());
        assert!(s.power(16.).unwrap().multiply(&s).is_err());
        assert!(s.divide(&TransferFunction::scalar(0.).unwrap()).is_err());
        assert!(s.feedback(&s, 0.).is_err());
        assert!(TransferFunction::pid(1., f64::NAN, 0.).is_err());
    }
}
