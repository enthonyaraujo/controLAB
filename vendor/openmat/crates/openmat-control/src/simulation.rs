use openmat_numeric::execution;
use crate::{error, Result, TransferFunction, MAX_SAMPLES, MAX_WORK};

/// Dense, row-major local matrices, bounded by MAX_ORDER + 1.
/// Evaluate exp([[A,B],[0,0]]*dt) with scaling, Taylor series and squaring.
/// This provides a constant-input transition without solving a potentially
/// singular A (integrators are valid), or choosing an Euler integration step.
fn transition(mut matrix: Vec<f64>, n: usize) -> Result<Vec<f64>> {
    let norm = (0..n)
        .map(|c| (0..n).map(|r| matrix[r * n + c].abs()).sum::<f64>())
        .fold(0., f64::max);
    if !norm.is_finite() {
        return Err(error("step: matriz de transição excedeu f64"));
    }
    let scaling = if norm > 0.5 {
        (norm / 0.5).log2().ceil()
    } else {
        0.
    };
    if scaling > 32. {
        return Err(error(
            "step: escala excessiva; reduza o passo de tempo ou reescale o modelo",
        ));
    }
    let divisor = 2f64.powi(scaling as i32);
    for v in &mut matrix {
        *v /= divisor;
    }
    let mut result = identity(n)?;
    execution::payload(n*n,8)?;
    let mut term = result.clone();
    let mut converged = false;
    for k in 1..=64 {
        execution::checkpoint(0)?;
        term = multiply(&term, &matrix, n)?;
        for v in &mut term {
            *v /= k as f64;
        }
        for (value, addition) in result.iter_mut().zip(&term) {
            *value += addition;
        }
        let term_norm = term.iter().map(|v| v.abs()).fold(0., f64::max);
        let result_norm = result.iter().map(|v| v.abs()).fold(0., f64::max);
        if term_norm <= f64::EPSILON * result_norm {
            converged = true;
            break;
        }
    }
    if !converged {
        return Err(error("step: exponencial matricial não convergiu"));
    }
    for _ in 0..scaling as usize {
        result = multiply(&result, &result, n)?;
    }
    Ok(result)
}
fn identity(n: usize) -> Result<Vec<f64>> {
    execution::payload(n*n,8)?;
    let mut result = vec![0.; n * n];
    for i in 0..n {
        result[i * n + i] = 1.;
    }
    Ok(result)
}
fn multiply(a: &[f64], b: &[f64], n: usize) -> Result<Vec<f64>> {
    execution::payload(n*n,8)?;
    execution::checkpoint((n*n*n) as u64)?;
    let mut result = vec![0.; n * n];
    for r in 0..n {
        for k in 0..n {
            for c in 0..n {
                result[r * n + c] += a[r * n + k] * b[k * n + c];
            }
        }
    }
    if result.iter().any(|v| !v.is_finite()) {
        return Err(error(
            "step: transição não finita; reescale o modelo ou reduza o horizonte",
        ));
    }
    Ok(result)
}
pub(crate) fn step(model: &TransferFunction, times: &[f64]) -> Result<Vec<f64>> {
    if times.len() < 2
        || times.len() > MAX_SAMPLES
        || times[0] != 0.
        || times.iter().any(|v| !v.is_finite())
    {
        return Err(error(
            "step: tempo requer 2 a 100000 amostras finitas, começando em zero",
        ));
    }
    let dt = times[1];
    if dt <= 0.
        || times.windows(2).any(|w| {
            w[1] <= w[0] || ((w[1] - w[0]) - dt).abs() > 1e-9 * dt + 16. * f64::EPSILON * w[1].abs()
        })
    {
        return Err(error(
            "step: tempo deve ser crescente e uniformemente espaçado",
        ));
    }
    if !model.is_proper() {
        return Err(error(
            "step: resposta requer função de transferência própria",
        ));
    }
    let order = model.order();
    if order == 0 {
        execution::payload(times.len(),8)?;
        execution::checkpoint(times.len() as u64)?;
        return Ok(vec![model.dc_gain(); times.len()]);
    }
    let n = order + 1;
    // 96 cubic products conservatively cover Taylor + 32 squarings.
    if 96 * n * n * n + times.len() * n * n > MAX_WORK {
        return Err(error("step: simulação excede orçamento de operações"));
    }
    execution::payload(n+n*n+order+n+times.len(),8)?;
    let mut numerator = vec![0.; n];
    let offset = n - model.numerator().len();
    numerator[offset..].copy_from_slice(model.numerator());
    let direct = numerator[0];
    let mut matrix = vec![0.; n * n];
    let mut output = vec![0.; order];
    for i in 0..order {
        matrix[i] = -model.denominator()[i + 1] * dt;
        output[i] = numerator[i + 1] - direct * model.denominator()[i + 1];
        if i > 0 {
            matrix[i * n + i - 1] = dt;
        }
    }
    matrix[order] = dt; // B = e1, augmented constant input = 1.
    if output.iter().any(|v| !v.is_finite()) {
        return Err(error("step: realização excedeu f64"));
    }
    let transition = transition(matrix, n)?;
    let mut state = vec![0.; n];
    state[order] = 1.;
    let mut values = Vec::with_capacity(times.len());
    for sample in 0..times.len() {
        execution::checkpoint((n*n) as u64)?;
        let y = direct + output.iter().zip(&state).map(|(c, x)| c * x).sum::<f64>();
        if !y.is_finite() {
            return Err(error(
                "step: resposta não finita; reduza o horizonte ou reescale o modelo",
            ));
        }
        values.push(y);
        if sample + 1 == times.len() {
            break;
        }
        execution::payload(n,8)?;
        let mut next = vec![0.; n];
        for r in 0..n {
            next[r] = (0..n).map(|c| transition[r * n + c] * state[c]).sum();
        }
        if next.iter().any(|v| !v.is_finite()) {
            return Err(error(
                "step: estado não finito; reduza o horizonte ou reescale o modelo",
            ));
        }
        state = next;
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(n: &[f64], d: &[f64], dt: f64, expected: impl Fn(f64) -> f64) {
        let t: Vec<_> = (0..501).map(|i| i as f64 * dt).collect();
        let y = TransferFunction::new(n, d).unwrap().step(&t).unwrap();
        for (time, actual) in t.into_iter().zip(y) {
            let exact = expected(time);
            assert!(
                (actual - exact).abs() <= 2e-10 * exact.abs().max(1.),
                "t={time}: {actual} != {exact}"
            );
        }
    }
    #[test]
    fn first_order_stiff_and_direct_feedthrough() {
        check(&[2.], &[1., 2.], 0.01, |t| 1. - (-2. * t).exp());
        check(&[1000.], &[1., 1000.], 0.05, |t| 1. - (-1000. * t).exp());
        check(&[2., 3.], &[1., 1.], 0.01, |t| 3. - (-t).exp());
        check(&[3.], &[1.], 0.01, |_| 3.);
    }
    #[test]
    fn integrators_and_unstable_systems() {
        check(&[1.], &[1., 0.], 0.01, |t| t);
        check(&[1.], &[1., 0., 0.], 0.01, |t| t * t / 2.);
        check(&[1.], &[1., -1.], 0.01, |t| t.exp() - 1.);
    }
    #[test]
    fn second_order_oscillatory_and_repeated_poles() {
        let w = 7.75f64.sqrt();
        check(&[10.], &[1., 3., 10.], 0.01, |t| {
            1. - (-1.5 * t).exp() * ((w * t).cos() + 1.5 / w * (w * t).sin())
        });
        check(&[1.], &[1., 2., 1.], 0.01, |t| 1. - (-t).exp() * (1. + t));
    }
    #[test]
    fn invalid_times_improper_overflow_and_work_limits() {
        let g = TransferFunction::new(&[1.], &[1., 1.]).unwrap();
        for t in [
            &[][..],
            &[0.],
            &[1., 2.],
            &[0., 0.],
            &[0., -1.],
            &[0., 1., 3.],
            &[0., f64::NAN],
            &[0., f64::INFINITY],
        ] {
            assert!(g.step(t).is_err());
        }
        assert!(TransferFunction::laplace().step(&[0., 1.]).is_err());
        assert!(TransferFunction::new(&[1.], &[1., -1000.])
            .unwrap()
            .step(&[0., 1.])
            .is_err());
        assert!(g.step(&[0., 1e20]).is_err());
        assert!(g.step(&vec![0.; MAX_SAMPLES + 1]).is_err());
    }
}
