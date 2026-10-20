use crate::{error, Result, TransferFunction, MAX_SAMPLES};
use std::f64::consts::TAU;

pub(crate) fn automatic(model: &TransferFunction) -> Result<Vec<f64>> {
    if !model.is_proper() {
        return Err(error(
            "step: resposta requer função de transferência própria",
        ));
    }
    let d = model.denominator();
    let (duration, fast_rate) = match model.order() {
        0 => (1., 1.),
        1 => {
            let rate = d[1].abs();
            if rate == 0. {
                (10., 1.)
            } else {
                (8. / rate, rate)
            }
        }
        2 => {
            // Roots of s² + a*s + b. Scale coefficients before forming the
            // discriminant to avoid a² overflow; q avoids subtractive loss.
            let scale = d[1].abs().max(d[2].abs().sqrt());
            if scale == 0. {
                (10., 1.)
            } else {
                let a = d[1] / scale;
                let b = (d[2] / scale) / scale;
                let discriminant = a * a - 4. * b;
                if discriminant < 0. {
                    let decay = d[1].abs() / 2.;
                    let omega = (-discriminant).sqrt() * scale / 2.;
                    let duration = if decay > 0. {
                        8. / decay
                    } else {
                        10. * TAU / omega
                    };
                    (duration, scale.max(omega))
                } else {
                    let q = -0.5 * (a + discriminant.sqrt().copysign(a));
                    let r1 = q.abs() * scale;
                    let r2 = if q == 0. {
                        0.
                    } else {
                        ((d[2] / scale) / q).abs()
                    };
                    let slow = [r1, r2]
                        .into_iter()
                        .filter(|r| *r > 0.)
                        .fold(f64::INFINITY, f64::min);
                    (
                        if slow.is_finite() { 8. / slow } else { 10. },
                        r1.max(r2).max(1e-300),
                    )
                }
            }
        }
        _ => {
            let fast = d
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, a)| a.abs().powf(1. / i as f64))
                .fold(0., f64::max);
            let slow = d
                .windows(2)
                .skip(1)
                .filter_map(|pair| {
                    if pair[0] != 0. && pair[1] != 0. {
                        Some((pair[1] / pair[0]).abs())
                    } else {
                        None
                    }
                })
                .fold(fast, f64::min);
            if fast == 0. {
                (10., 1.)
            } else {
                (8. / slow.max(fast * 1e-6), fast)
            }
        }
    };
    if !duration.is_finite() || duration <= 0. || !fast_rate.is_finite() {
        return Err(error(
            "step: escala automática inválida; forneça um vetor de tempo explícito",
        ));
    }
    // At least 500 intervals and at least 40 intervals per inverse fast scale.
    let intervals = (duration * fast_rate * 40.).ceil().max(500.);
    if !intervals.is_finite() || intervals >= MAX_SAMPLES as f64 {
        return Err(error(
            "step: grade automática excede 100000 amostras; forneça tempo explícito",
        ));
    }
    let intervals = intervals as usize;
    let dt = duration / intervals as f64;
    if dt == 0. {
        return Err(error("step: passo automático abaixo da precisão f64"));
    }
    openmat_numeric::execution::payload(intervals+1,8)?;
    openmat_numeric::execution::checkpoint((intervals+1) as u64)?;
    Ok((0..=intervals)
        .map(|i| {
            if i == intervals {
                duration
            } else {
                i as f64 * dt
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_scale_tracks_fast_and_slow_models() {
        for rate in [0.001, 1., 1000.] {
            let g = TransferFunction::new(&[rate], &[1., rate]).unwrap();
            let t = g.step_times().unwrap();
            assert_eq!(t[0], 0.);
            assert!((t.last().unwrap() * rate - 8.).abs() < 1e-12);
            let y = g.step(&t).unwrap();
            assert!((y.last().unwrap() - (1. - (-8f64).exp())).abs() < 1e-11);
        }
    }
    #[test]
    fn repeated_and_oscillatory_poles_produce_valid_sampled_responses() {
        for d in [
            &[1., 2., 1.][..],
            &[1., 2., 6.],
            &[1., 3., 10.],
            &[1., 0., 1.],
            &[1., 0., 0.],
            &[1., 6., 11., 6.],
        ] {
            let g = TransferFunction::new(&[1.], d).unwrap();
            let t = g.step_times().unwrap();
            assert!(t.len() <= MAX_SAMPLES);
            assert!(g.step(&t).unwrap().iter().all(|v| v.is_finite()));
        }
    }
    #[test]
    fn excessive_resolution_and_improper_models_have_clear_errors() {
        assert!(TransferFunction::laplace().step_times().is_err());
        let g = TransferFunction::new(&[1.], &[1., 1e-9, 1.]).unwrap();
        assert!(g.step_times().unwrap_err().0.contains("tempo explícito"));
        assert!(g.step(&[0., 0.1, 0.2]).is_ok());
        // Do not silently lose a very slow pole to an intermediate underflow.
        assert!(TransferFunction::new(&[1.], &[1., 1e200, 1.])
            .unwrap()
            .step_times()
            .is_err());
    }
}
