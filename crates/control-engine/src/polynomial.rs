//! Descending-order polynomial arithmetic, matching NumPy coefficient conventions.
use crate::Result;
use nalgebra::DMatrix;
use num_complex::Complex64;

pub fn trim(p: &[f64]) -> Vec<f64> {
    let start = p
        .iter()
        .position(|&x| x != 0.)
        .unwrap_or(p.len().saturating_sub(1));
    if p.is_empty() {
        vec![0.]
    } else {
        p[start..].to_vec()
    }
}

pub fn add(a: &[f64], b: &[f64], scale: f64) -> Vec<f64> {
    let n = a.len().max(b.len());
    let mut out = vec![0.; n];
    for (i, v) in a.iter().enumerate() {
        out[n - a.len() + i] += v;
    }
    for (i, v) in b.iter().enumerate() {
        out[n - b.len() + i] += scale * v;
    }
    trim(&out)
}

pub fn multiply(a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut out = vec![0.; a.len() + b.len() - 1];
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            out[i + j] += x * y;
        }
    }
    trim(&out)
}

pub fn derivative(p: &[f64]) -> Vec<f64> {
    if p.len() < 2 {
        return vec![0.];
    }
    p[..p.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, v)| v * (p.len() - 1 - i) as f64)
        .collect()
}

pub fn evaluate(p: &[f64], x: Complex64) -> Complex64 {
    p.iter().fold(Complex64::new(0., 0.), |acc, &v| acc * x + v)
}

pub fn roots(p: &[f64]) -> Result<Vec<Complex64>> {
    crate::execution::checkpoint()?;
    let p = trim(p);
    if p.iter().any(|v| !v.is_finite()) {
        return Err("Coeficientes devem ser finitos.".into());
    }
    let zero_count = p.iter().rev().take_while(|&&v| v == 0.).count();
    if zero_count == p.len() {
        return Ok(vec![]);
    }
    let p = &p[..p.len() - zero_count];
    let n = p.len() - 1;
    if n == 0 {
        return Ok(vec![Complex64::new(0., 0.); zero_count]);
    }
    // Even polynomials occur in analytic frequency-margin equations. Reducing
    // p(s)=q(s²) avoids a symmetric companion matrix that can stall Francis QR.
    // This is an algebraic reduction, not a frequency-grid approximation.
    if n > 2 && n.is_multiple_of(2) && p.iter().enumerate().all(|(i, &v)| i % 2 == 0 || v == 0.) {
        let q: Vec<_> = p.iter().step_by(2).copied().collect();
        let mut out = Vec::with_capacity(n + zero_count);
        for r in roots(&q)? {
            let s = r.sqrt();
            out.push(s);
            out.push(-s);
        }
        out.extend(vec![Complex64::new(0., 0.); zero_count]);
        return Ok(out);
    }
    let mut a = DMatrix::zeros(n, n);
    for j in 0..n {
        a[(0, j)] = -p[j + 1] / p[0];
    }
    for i in 1..n {
        a[(i, i - 1)] = 1.;
    }
    // Companion matrices need the same power-of-two balancing used before
    // LAPACK eigvals; without it QR can stall on control margin polynomials.
    for _ in 0..64 {
        let mut changed = false;
        for i in 0..n {
            let mut col = (0..n)
                .filter(|&j| j != i)
                .map(|j| a[(j, i)].abs())
                .sum::<f64>();
            let row = (0..n)
                .filter(|&j| j != i)
                .map(|j| a[(i, j)].abs())
                .sum::<f64>();
            if col == 0. || row == 0. || !col.is_finite() || !row.is_finite() {
                continue;
            }
            let original = col + row;
            let mut factor = 1.;
            for _ in 0..512 {
                if col >= row / 2. {
                    break;
                }
                col *= 4.;
                factor *= 2.;
            }
            for _ in 0..512 {
                if col < row * 2. {
                    break;
                }
                col /= 4.;
                factor /= 2.;
            }
            if (col + row) / factor < 0.95 * original && factor.is_finite() && factor > 0. {
                changed = true;
                for j in 0..n {
                    a[(i, j)] /= factor;
                    a[(j, i)] *= factor;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let schur = nalgebra::linalg::Schur::try_new(a, f64::EPSILON, 100_000)
        .ok_or("O cálculo de raízes não convergiu.")?;
    let mut result = schur
        .complex_eigenvalues()
        .iter()
        .copied()
        .collect::<Vec<_>>();
    // NumPy strips zero roots before eigvals and appends them afterwards.
    result.extend(vec![Complex64::new(0., 0.); zero_count]);
    Ok(result)
}

pub fn from_roots(roots: &[Complex64], gain: f64) -> Vec<Complex64> {
    let mut p = vec![Complex64::new(gain, 0.)];
    for r in roots {
        let mut q = vec![Complex64::new(0., 0.); p.len() + 1];
        for (i, c) in p.iter().enumerate() {
            q[i] += c;
            q[i + 1] -= c * r;
        }
        p = q;
    }
    p
}
