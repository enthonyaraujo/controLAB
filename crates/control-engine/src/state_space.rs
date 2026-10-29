//! State-space conversion, structure, canonical forms and pole placement.
//!
//! The multivariable placement follows the real/complex pair updates of
//! Tits and Yang (1996), the YT method selected by scipy.signal.place_poles.
use crate::{
    Result, input, polynomial, presentation, simulation, transfer::TransferFunction, values,
};
use nalgebra::{DMatrix, DVector};
use num_complex::Complex64 as Complex;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use serde_json::{Value, json};

type Matrix = DMatrix<f64>;
type CMatrix = DMatrix<Complex>;

#[derive(Clone)]
struct System {
    a: Matrix,
    b: Matrix,
    c: Matrix,
    d: Matrix,
}

pub fn presets() -> Value {
    json!([
        {"id":"Segunda ordem controlável","title":"Segunda ordem controlável","A":"[[0, 1], [-2, -3]]","B":"[[0], [1]]","C":"[[1, 0]]","D":"[[0]]"},
        {"id":"Oscilador harmônico","title":"Oscilador harmônico","A":"[[0, 1], [-4, 0]]","B":"[[0], [1]]","C":"[[1, 0]]","D":"[[0]]"}
    ])
}

// Translate the non-executable literal subset accepted by ast.literal_eval:
// tuples, quoted numeric strings, booleans and trailing commas.
fn python_literal_json(text: &str) -> String {
    let chars: Vec<_> = text.chars().collect();
    let mut output = String::new();
    let mut quote = None;
    let mut escaped = false;
    for (i, &c) in chars.iter().enumerate() {
        if let Some(delimiter) = quote {
            if escaped {
                output.push(c);
                escaped = false;
                continue;
            }
            if c == '\\' {
                output.push(c);
                escaped = true;
                continue;
            }
            if c == delimiter {
                output.push('"');
                quote = None;
            } else if c == '"' {
                output.push_str("\\\"");
            } else {
                output.push(c);
            }
        } else {
            match c {
                '\'' | '"' => {
                    quote = Some(c);
                    output.push('"');
                }
                '(' => output.push('['),
                ')' => output.push(']'),
                ',' if chars[i + 1..]
                    .iter()
                    .find(|c| !c.is_whitespace())
                    .is_some_and(|c| *c == ']' || *c == ')') => {}
                '_' if i > 0
                    && i + 1 < chars.len()
                    && chars[i - 1].is_ascii_digit()
                    && chars[i + 1].is_ascii_digit() => {}
                _ => output.push(c),
            }
        }
    }
    output.replace("True", "true").replace("False", "false")
}

pub fn parse_matrix(value: &Value, name: &str) -> Result<Matrix> {
    fn scalar(v: &Value) -> Result<f64> {
        v.as_f64()
            .or_else(|| v.as_bool().map(|v| if v { 1. } else { 0. }))
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            .filter(|v| v.is_finite())
            .ok_or_else(|| "A matriz deve conter apenas números finitos.".into())
    }
    let parsed = if let Some(text) = value.as_str() {
        let text = text.trim();
        if text.is_empty() {
            return Err(format!("A matriz {name} não pode ficar vazia."));
        }
        if let Ok(v) = serde_json::from_str::<Value>(text) {
            v
        } else {
            // Python literal tuples are accepted as well as JSON lists.
            let replaced = python_literal_json(text);
            if let Ok(v) = serde_json::from_str::<Value>(&replaced) {
                v
            } else {
                let rows: Result<Vec<Vec<f64>>> = text
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|row| {
                        row.replace(',', " ")
                            .split_whitespace()
                            .map(|s| {
                                s.parse::<f64>().map_err(|_| {
                                    format!("could not convert string to float: '{s}'")
                                })
                            })
                            .collect()
                    })
                    .collect();
                json!(rows?)
            }
        }
    } else {
        value.clone()
    };
    let rows: Vec<Vec<f64>> = match &parsed {
        Value::Array(items) if items.first().is_some_and(Value::is_array) => items
            .iter()
            .map(|r| {
                r.as_array()
                    .ok_or_else(|| "Matriz irregular.".to_owned())
                    .and_then(|r| r.iter().map(scalar).collect())
            })
            .collect::<Result<_>>()?,
        Value::Array(items) => vec![items.iter().map(scalar).collect::<Result<_>>()?],
        v => vec![vec![scalar(v)?]],
    };
    let width = rows.first().map_or(0, Vec::len);
    if rows.is_empty()
        || width == 0
        || rows
            .iter()
            .any(|r| r.len() != width || r.iter().any(|v| !v.is_finite()))
    {
        return Err(format!(
            "A matriz {name} deve ser bidimensional e conter apenas números finitos."
        ));
    }
    Ok(Matrix::from_fn(rows.len(), width, |i, j| rows[i][j]))
}

fn validate(s: &System) -> Result<()> {
    let n = s.a.nrows();
    if s.a.ncols() != n {
        return Err("A matriz A deve ser quadrada.".into());
    }
    if s.b.nrows() != n {
        return Err("B deve possuir o mesmo número de linhas de A.".into());
    }
    if s.c.ncols() != n {
        return Err("C deve possuir o mesmo número de colunas de A.".into());
    }
    if s.d.shape() != (s.c.nrows(), s.b.ncols()) {
        return Err("D deve possuir dimensões saídas × entradas.".into());
    }
    Ok(())
}

fn complex_matrix(a: &Matrix) -> CMatrix {
    a.map(|x| Complex::new(x, 0.))
}
fn serialize(a: &Matrix) -> Value {
    json!(
        (0..a.nrows())
            .map(|i| (0..a.ncols()).map(|j| a[(i, j)]).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    )
}
fn serialize_complex(a: &CMatrix) -> Value {
    Value::Array(
        (0..a.nrows())
            .map(|i| {
                Value::Array(
                    (0..a.ncols())
                        .map(|j| {
                            let v = a[(i, j)];
                            if v.im.abs() < 1e-10 {
                                json!(v.re)
                            } else {
                                json!(format!(
                                    "{}{}{}j",
                                    values::general(v.re, 6),
                                    if v.im < 0. { "-" } else { "+" },
                                    values::general(v.im.abs(), 6)
                                ))
                            }
                        })
                        .collect(),
                )
            })
            .collect(),
    )
}
fn rank(a: &Matrix) -> usize {
    if a.is_empty() {
        return 0;
    }
    let sigma = a.clone().svd(false, false).singular_values;
    let tolerance = sigma.max() * a.nrows().max(a.ncols()) as f64 * f64::EPSILON;
    sigma.iter().filter(|&&v| v > tolerance).count()
}
fn controllability(a: &Matrix, b: &Matrix) -> Matrix {
    let n = a.nrows();
    let m = b.ncols();
    let mut out = Matrix::zeros(n, n * m);
    let mut power = b.clone();
    for i in 0..n {
        out.view_mut((0, i * m), (n, m)).copy_from(&power);
        power = a * power;
    }
    out
}
fn observability(a: &Matrix, c: &Matrix) -> Matrix {
    controllability(&a.transpose(), &c.transpose()).transpose()
}
fn eigenvalues(a: &Matrix) -> Result<Vec<Complex>> {
    if a.nrows() == 0 {
        return Ok(vec![]);
    }
    let schur = nalgebra::linalg::Schur::try_new(a.clone(), f64::EPSILON, 100_000)
        .ok_or("O cálculo de autovalores não convergiu.")?;
    Ok(schur.complex_eigenvalues().iter().copied().collect())
}
fn characteristic(a: &Matrix) -> Vec<f64> {
    let n = a.nrows();
    let mut coefficients = vec![1.];
    let mut work = Matrix::identity(n, n);
    for k in 1..=n {
        let product = a * &work;
        let coefficient = -product.trace() / k as f64;
        coefficients.push(coefficient);
        work = product + Matrix::identity(n, n) * coefficient;
    }
    coefficients
}
fn clean(v: f64) -> f64 {
    if v.abs() < 1e-9 { 0. } else { v }
}

fn poles(value: Option<&Value>, expected: usize, label: &str) -> Result<Vec<Complex>> {
    let Some(value) = value else {
        return Ok(vec![]);
    };
    if value.is_null() || value.as_str().is_some_and(|s| s.trim().is_empty()) {
        return Ok(vec![]);
    }
    let result = if let Some(items) = value.as_array() {
        items
            .iter()
            .map(|v| {
                if let Some(x) = v.as_f64() {
                    Ok(Complex::new(x, 0.))
                } else {
                    input::complex(v.as_str().ok_or("Polo inválido.")?)
                }
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        // Match the existing decimal-comma substitution, including its need
        // for whitespace between comma-separated integer poles.
        let text = value.as_str().ok_or("Polo inválido.")?;
        let chars: Vec<_> = text.chars().collect();
        let normalized: String = chars
            .iter()
            .enumerate()
            .map(|(i, &c)| {
                if c == ','
                    && i > 0
                    && i + 1 < chars.len()
                    && chars[i - 1].is_ascii_digit()
                    && chars[i + 1].is_ascii_digit()
                {
                    '.'
                } else {
                    c
                }
            })
            .collect();
        input::complex_list(&normalized)?
    };
    if result.len() != expected {
        return Err(format!("{label} exige exatamente {expected} polos."));
    }
    if result
        .iter()
        .any(|p| !p.re.is_finite() || !p.im.is_finite())
    {
        return Err("Os polos devem conter apenas números finitos.".into());
    }
    Ok(result)
}

/// RREF null space, with free variables ordered as in SymPy. Complex arithmetic
/// is needed for diagonal/Jordan transformations even though ct.ss casts A/B/C.
fn nullspace(a: &CMatrix, tolerance: f64) -> CMatrix {
    let mut r = a.clone();
    let mut pivots = vec![];
    let mut row = 0;
    for col in 0..r.ncols() {
        if row == r.nrows() {
            break;
        }
        let best = (row..r.nrows())
            .max_by(|&i, &j| r[(i, col)].norm().total_cmp(&r[(j, col)].norm()))
            .unwrap();
        if r[(best, col)].norm() <= tolerance {
            continue;
        }
        r.swap_rows(row, best);
        let p = r[(row, col)];
        for j in col..r.ncols() {
            r[(row, j)] /= p;
        }
        for i in 0..r.nrows() {
            if i == row {
                continue;
            }
            let p = r[(i, col)];
            for j in col..r.ncols() {
                let v = r[(row, j)];
                r[(i, j)] -= p * v;
            }
        }
        pivots.push(col);
        row += 1;
    }
    let free: Vec<_> = (0..r.ncols()).filter(|c| !pivots.contains(c)).collect();
    let mut out = CMatrix::zeros(r.ncols(), free.len());
    for (j, &f) in free.iter().enumerate() {
        out[(f, j)] = Complex::new(1., 0.);
        for (i, &p) in pivots.iter().enumerate() {
            out[(p, j)] = -r[(i, f)];
        }
    }
    out
}
fn column_rank(a: &CMatrix) -> usize {
    if a.is_empty() {
        0
    } else {
        let s = a.clone().svd(false, false).singular_values;
        let t = s.max() * a.nrows().max(a.ncols()) as f64 * 1e-10;
        s.iter().filter(|&&v| v > t).count()
    }
}
fn append_columns(a: &CMatrix, b: &CMatrix) -> CMatrix {
    CMatrix::from_fn(a.nrows(), a.ncols() + b.ncols(), |i, j| {
        if j < a.ncols() {
            a[(i, j)]
        } else {
            b[(i, j - a.ncols())]
        }
    })
}

fn eigenvectors(a: &Matrix, eigenvalues: &[Complex]) -> Result<CMatrix> {
    let n = a.nrows();
    let ac = complex_matrix(a);
    let mut vectors = CMatrix::zeros(n, n);
    let tol = 1e-9 * a.norm().max(1.);
    // Match DLANV2/DTR(E)VC's real 2x2 rotation and back substitution,
    // including its eigenvector signs (the diagonal fixtures expose them).
    if n == 2
        && eigenvalues.iter().all(|e| e.im == 0.)
        && (eigenvalues[0] - eigenvalues[1]).norm() > tol
    {
        let (cs, sn) = if a[(1, 0)] == 0. {
            (1., 0.)
        } else {
            let z = eigenvalues[0].re - a[(1, 1)];
            let norm = z.hypot(a[(1, 0)]);
            (z / norm, a[(1, 0)] / norm)
        };
        let q = Matrix::from_row_slice(2, 2, &[cs, -sn, sn, cs]);
        let t = q.transpose() * a * &q;
        let first = q.column(0).into_owned();
        let second = &q
            * DVector::from_vec(vec![
                -t[(0, 1)] / (eigenvalues[0].re - eigenvalues[1].re),
                1.,
            ]);
        vectors.set_column(0, &first.map(|v| Complex::new(v, 0.)));
        vectors.set_column(
            1,
            &(second.clone() / second.norm()).map(|v| Complex::new(v, 0.)),
        );
        return Ok(vectors);
    }
    for (i, &lambda) in eigenvalues.iter().enumerate() {
        let ns = nullspace(&(&ac - CMatrix::identity(n, n) * lambda), tol);
        if ns.ncols() == 0 {
            return Err("Não foi possível obter os autovetores numericamente.".into());
        }
        let mut v = ns.column(0).into_owned();
        v /= Complex::new(v.norm(), 0.);
        let pivot = (0..n)
            .max_by(|&i, &j| v[i].norm().total_cmp(&v[j].norm()))
            .unwrap();
        let phase = v[pivot] / v[pivot].norm();
        v /= phase;
        vectors.set_column(i, &v);
    }
    if column_rank(&vectors) < n {
        return Err("A matriz A não é diagonalizável; use a forma de Jordan.".into());
    }
    Ok(vectors)
}

// Jordan multiplicities cannot be inferred by clustering floating eigenvalues:
// a defective real triple pole is routinely returned as three complex poles.
// Compute the characteristic polynomial and its square-free factors exactly,
// then find only the simple roots numerically.
fn exact_characteristic(a: &Matrix) -> Vec<BigRational> {
    let n = a.nrows();
    let values: Vec<Vec<_>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| BigRational::from_float(a[(i, j)]).unwrap())
                .collect()
        })
        .collect();
    let mut work = vec![vec![BigRational::zero(); n]; n];
    for (i, row) in work.iter_mut().enumerate() {
        row[i] = BigRational::one();
    }
    let mut coefficients = vec![BigRational::one()];
    for k in 1..=n {
        let mut product = vec![vec![BigRational::zero(); n]; n];
        for i in 0..n {
            for j in 0..n {
                for r in 0..n {
                    product[i][j] += &values[i][r] * &work[r][j];
                }
            }
        }
        let trace = (0..n).map(|i| product[i][i].clone()).sum::<BigRational>();
        let coefficient = -trace / BigRational::from_integer(k.into());
        for (i, row) in product.iter_mut().enumerate() {
            row[i] += &coefficient;
        }
        coefficients.push(coefficient);
        work = product;
    }
    coefficients
}
fn rational_trim(p: &[BigRational]) -> Vec<BigRational> {
    let first = p.iter().position(|v| !v.is_zero()).unwrap_or(p.len() - 1);
    p[first..].to_vec()
}
fn rational_divide(a: &[BigRational], b: &[BigRational]) -> (Vec<BigRational>, Vec<BigRational>) {
    if a.len() < b.len() {
        return (vec![BigRational::zero()], a.to_vec());
    }
    let mut remainder = a.to_vec();
    let mut quotient = vec![BigRational::zero(); a.len() - b.len() + 1];
    while remainder.len() >= b.len() && !remainder[0].is_zero() {
        let factor = &remainder[0] / &b[0];
        let power = remainder.len() - b.len();
        let index = quotient.len() - 1 - power;
        quotient[index] = factor.clone();
        for i in 0..b.len() {
            remainder[i] -= &factor * &b[i];
        }
        remainder = rational_trim(&remainder);
    }
    (rational_trim(&quotient), remainder)
}
fn rational_gcd(a: &[BigRational], b: &[BigRational]) -> Vec<BigRational> {
    let mut a = rational_trim(a);
    let mut b = rational_trim(b);
    while !b.iter().all(Zero::is_zero) {
        let remainder = rational_divide(&a, &b).1;
        a = b;
        b = remainder;
    }
    let scale = a[0].clone();
    a.iter().map(|v| v / &scale).collect()
}
fn jordan_eigen_groups(a: &Matrix) -> Result<Vec<(Complex, usize)>> {
    if a.nrows() == 0 {
        return Ok(vec![]);
    }
    let f = exact_characteristic(a);
    let derivative: Vec<_> = f
        .iter()
        .take(f.len() - 1)
        .enumerate()
        .map(|(i, c)| c * BigRational::from_integer((f.len() - 1 - i).into()))
        .collect();
    let mut g = rational_gcd(&f, &derivative);
    let mut w = rational_divide(&f, &g).0;
    let mut multiplicity = 1;
    let mut groups = vec![];
    while w.len() > 1 {
        let y = rational_gcd(&w, &g);
        let z = rational_divide(&w, &y).0;
        if z.len() > 1 {
            let coefficients = z
                .iter()
                .map(|c| {
                    c.to_f64().ok_or_else(|| {
                        "Não foi possível obter uma forma de Jordan numericamente estável."
                            .to_owned()
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            for root in polynomial::roots(&coefficients)? {
                groups.push((root, multiplicity));
            }
        }
        w = y;
        g = rational_divide(&g, &w).0;
        multiplicity += 1;
        if multiplicity > f.len() {
            return Err("Não foi possível obter uma forma de Jordan numericamente estável.".into());
        }
    }
    groups.sort_by(|a, b| a.0.re.total_cmp(&b.0.re).then(a.0.im.total_cmp(&b.0.im)));
    Ok(groups)
}

fn jordan(a: &Matrix) -> Result<(CMatrix, CMatrix)> {
    let n = a.nrows();
    let ac = complex_matrix(a);
    let groups = jordan_eigen_groups(a)?;
    let mut basis = CMatrix::zeros(n, 0);
    let mut j = CMatrix::zeros(n, n);
    let mut column = 0;
    let tolerance = 1e-9 * a.norm().max(1.);
    for (lambda, multiplicity) in groups {
        let shifted = &ac - CMatrix::identity(n, n) * lambda;
        let mut powers = vec![CMatrix::identity(n, n)];
        for k in 1..=multiplicity {
            powers.push(&powers[k - 1] * &shifted);
        }
        let mut eig_basis = CMatrix::zeros(n, 0);
        for length in (1..=multiplicity).rev() {
            let ns = nullspace(&powers[length], tolerance);
            for k in 0..ns.ncols() {
                let top = ns.column(k).into_owned();
                let mut chain = vec![top];
                for _ in 1..length {
                    chain.push(&shifted * chain.last().unwrap());
                }
                chain.reverse();
                let candidate = CMatrix::from_columns(&chain);
                let expanded = append_columns(&eig_basis, &candidate);
                if column_rank(&expanded) == eig_basis.ncols() + length {
                    for offset in 0..length {
                        j[(column + offset, column + offset)] = lambda;
                        if offset + 1 < length {
                            j[(column + offset, column + offset + 1)] = Complex::new(1., 0.);
                        }
                    }
                    column += length;
                    eig_basis = expanded;
                }
                if eig_basis.ncols() == multiplicity {
                    break;
                }
            }
            if eig_basis.ncols() == multiplicity {
                break;
            }
        }
        if eig_basis.ncols() != multiplicity {
            return Err("Não foi possível obter uma forma de Jordan numericamente estável.".into());
        }
        basis = append_columns(&basis, &eig_basis);
    }
    if column != n
        || (&ac * &basis - &basis * &j).norm() > 1e-8 * a.norm().max(1.) * basis.norm().max(1.)
    {
        return Err("Não foi possível obter uma forma de Jordan numericamente estável.".into());
    }
    Ok((basis, j))
}

fn canonical(s: &System, form: &str) -> Result<(System, CMatrix, &'static str)> {
    let n = s.a.nrows();
    let form = form.to_lowercase();
    if matches!(form.as_str(), "none" | "original" | "") {
        return Ok((s.clone(), CMatrix::identity(n, n), "Original"));
    }
    if s.b.ncols() != 1 || s.c.nrows() != 1 {
        return Err("As formas canônicas selecionadas exigem um sistema SISO.".into());
    }
    let coefficients = characteristic(&s.a);
    match form.as_str() {
        "controllable" | "reachable" => {
            if n == 0 {
                return Err("index 0 is out of bounds for axis 0 with size 0".into());
            }
            let mut a = Matrix::zeros(n, n);
            for i in 0..n {
                a[(0, i)] = -coefficients[i + 1];
                if i > 0 {
                    a[(i, i - 1)] = 1.;
                }
            }
            let mut b = Matrix::zeros(n, 1);
            if n > 0 {
                b[(0, 0)] = 1.;
            }
            let wc = controllability(&s.a, &s.b);
            if rank(&wc) < n {
                return Err("System not controllable to working precision.".into());
            }
            let t = controllability(&a, &b)
                * wc.try_inverse()
                    .ok_or("Transformation matrix singular to working precision.")?;
            let inverse = t
                .clone()
                .try_inverse()
                .ok_or("Transformation matrix singular to working precision.")?;
            Ok((
                System {
                    a,
                    b,
                    c: &s.c * inverse,
                    d: s.d.clone(),
                },
                complex_matrix(&t),
                "Canônica controlável",
            ))
        }
        "observable" => {
            if n == 0 {
                return Err("index 0 is out of bounds for axis 1 with size 0".into());
            }
            let mut a = Matrix::zeros(n, n);
            for i in 0..n {
                a[(i, 0)] = -coefficients[i + 1];
                if i + 1 < n {
                    a[(i, i + 1)] = 1.;
                }
            }
            let mut c = Matrix::zeros(1, n);
            if n > 0 {
                c[(0, 0)] = 1.;
            }
            let wo = observability(&a, &c);
            let t = wo
                .try_inverse()
                .ok_or("Transformation matrix singular to working precision.")?
                * observability(&s.a, &s.c);
            if rank(&t) < n {
                return Err("Transformation matrix singular to working precision.".into());
            }
            Ok((
                System {
                    a,
                    b: &t * &s.b,
                    c,
                    d: s.d.clone(),
                },
                complex_matrix(&t),
                "Canônica observável",
            ))
        }
        "diagonal" | "jordan" => {
            let (p, a, label) = if form == "diagonal" {
                let eigen = eigenvalues(&s.a)?;
                let p = eigenvectors(&s.a, &eigen)?;
                (
                    p,
                    CMatrix::from_diagonal(&DVector::from_vec(eigen)),
                    "Canônica diagonal",
                )
            } else {
                let (p, j) = jordan(&s.a)?;
                (p, j, "Forma de Jordan")
            };
            let inverse = p
                .clone()
                .try_inverse()
                .ok_or("Não foi possível obter uma forma de Jordan numericamente estável.")?;
            // Deliberately preserve python-control StateSpace's real dtype cast:
            // the transformation remains complex, but canonical A/B/C lose Im.
            let b = (&inverse * complex_matrix(&s.b)).map(|x| x.re);
            let c = (complex_matrix(&s.c) * p).map(|x| x.re);
            Ok((
                System {
                    a: a.map(|x| x.re),
                    b,
                    c,
                    d: s.d.clone(),
                },
                inverse,
                label,
            ))
        }
        _ => Err("Forma canônica desconhecida.".into()),
    }
}

fn ackermann(a: &Matrix, b: &Matrix, poles: &[Complex]) -> Result<Matrix> {
    let n = a.nrows();
    let coefficients = polynomial::from_roots(poles, 1.);
    let mut evaluation = Matrix::identity(n, n) * coefficients[0].re;
    for coefficient in &coefficients[1..] {
        evaluation = a * evaluation + Matrix::identity(n, n) * coefficient.re;
    }
    let inverse = controllability(a, b)
        .try_inverse()
        .ok_or("System not reachable; pole placement invalid")?;
    Ok(inverse.row(n - 1) * evaluation).map(|row| Matrix::from_row_slice(1, n, row.as_slice()))
}

fn complete_q(a: &CMatrix) -> CMatrix {
    let n = a.nrows();
    let mut square = CMatrix::zeros(n, n);
    square.view_mut((0, 0), a.shape()).copy_from(a);
    square.qr().q()
}
fn orthogonal_q(x: &Matrix, removed: &[usize]) -> CMatrix {
    let cols: Vec<_> = (0..x.ncols()).filter(|i| !removed.contains(i)).collect();
    complete_q(&CMatrix::from_fn(x.nrows(), cols.len(), |i, j| {
        Complex::new(x[(i, cols[j])], 0.)
    }))
}

fn update_real(kernels: &[CMatrix], q: &CMatrix, x: &mut Matrix, i: usize, j: usize) {
    let n = x.nrows();
    let u = q.column(n - 2).map(|v| v.re);
    let v = q.column(n - 1).map(|v| v.re);
    let ki = kernels[i].map(|v| v.re);
    let kj = kernels[j].map(|v| v.re);
    let m = ki.transpose() * (&u * v.transpose() - &v * u.transpose()) * &kj;
    let svd = m.svd(true, true);
    let left = svd.u.unwrap();
    let right = svd.v_t.unwrap().transpose();
    let equal = (svd.singular_values[0] - svd.singular_values[1]).abs()
        <= 1e-8 + 1e-5 * svd.singular_values[1].abs();
    let columns = if equal { 2 } else { 1 };
    let combined = Matrix::from_fn(2 * n, columns, |r, c| {
        if r < n {
            (&ki * left.column(c))[r]
        } else {
            (&kj * right.column(c))[r - n]
        }
    });
    let original = DVector::from_fn(2 * n, |r, _| if r < n { x[(r, i)] } else { x[(r - n, j)] });
    let projected = &combined * combined.transpose() * original;
    let result = if projected.norm() > 1e-8 {
        projected.clone() * (2f64.sqrt() / projected.norm())
    } else {
        combined.column(0).into_owned()
    };
    x.set_column(i, &result.rows(0, n));
    x.set_column(j, &result.rows(n, n));
}

fn update_complex(kernels: &[CMatrix], q: &CMatrix, x: &mut Matrix, i: usize, j: usize) {
    let n = x.nrows();
    let u = DVector::from_fn(n, |r, _| {
        Complex::new(
            q[(r, n - 2)].re * 2f64.sqrt(),
            q[(r, n - 1)].re * 2f64.sqrt(),
        )
    });
    let conjugate = u.map(|z| z.conj());
    let kernel = &kernels[i];
    let m = kernel.adjoint() * (&u * u.adjoint() - &conjugate * conjugate.adjoint()) * kernel;
    let eigen = nalgebra::linalg::SymmetricEigen::new(m);
    let mut order: Vec<_> = (0..eigen.eigenvalues.len()).collect();
    order.sort_by(|&a, &b| {
        eigen.eigenvalues[a]
            .abs()
            .total_cmp(&eigen.eigenvalues[b].abs())
    });
    let first = order[order.len() - 1];
    let second = order[order.len() - 2];
    let equal = (eigen.eigenvalues[first].abs() - eigen.eigenvalues[second].abs()).abs()
        <= 1e-8 + 1e-5 * eigen.eigenvalues[second].abs();
    let eigenvectors = if equal {
        CMatrix::from_columns(&[
            eigen.eigenvectors.column(first).into_owned(),
            eigen.eigenvectors.column(second).into_owned(),
        ])
    } else {
        eigen.eigenvectors.columns(first, 1).into_owned()
    };
    let basis = kernel * eigenvectors;
    let original = DVector::from_fn(n, |r, _| Complex::new(x[(r, i)], x[(r, j)]));
    let mut result = &basis * basis.adjoint() * original;
    if result.norm() > 1e-8 {
        let norm = result.norm();
        result /= Complex::new(norm, 0.);
    } else {
        result = basis.column(0).into_owned();
    }
    x.set_column(i, &result.map(|z| z.re));
    x.set_column(j, &result.map(|z| z.im));
}

fn update_order(poles: &[Complex]) -> Vec<(usize, usize)> {
    let real = poles.iter().filter(|p| p.im == 0.).count();
    let half = real / 2;
    let mut pairs = vec![];
    if real > 0 {
        pairs.push((real, 1));
    }
    let complex_pairs: Vec<_> = ((real + 1)..=poles.len())
        .step_by(2)
        .map(|i| (i, i + 1))
        .collect();
    for i in 1..half + real % 2 {
        pairs.push((2 * i, 2 * i + 1));
    }
    pairs.extend(&complex_pairs);
    for i in 1..=half {
        pairs.push((2 * i - 1, 2 * i));
    }
    let add_single = |pairs: &mut Vec<(usize, usize)>| {
        if half == 0 && real > 0 {
            pairs.push((1, 1));
        }
    };
    add_single(&mut pairs);
    pairs.extend(&complex_pairs);
    for j in 2..half + real % 2 {
        for i in 1..=half {
            pairs.push((i, i + j));
        }
    }
    add_single(&mut pairs);
    pairs.extend(&complex_pairs);
    for j in 2..half + real % 2 {
        for i in half + 1..=real {
            let p = i + j;
            pairs.push((i, if p > real { p - real } else { p }));
        }
    }
    add_single(&mut pairs);
    pairs.extend(&complex_pairs);
    for i in 1..=half {
        pairs.push((i, i + half));
    }
    add_single(&mut pairs);
    pairs.extend(&complex_pairs);
    pairs.into_iter().map(|(i, j)| (i - 1, j - 1)).collect()
}

fn place(a: &Matrix, b: &Matrix, requested: &[Complex]) -> Result<Matrix> {
    let n = a.nrows();
    let rank_b = rank(b);
    let mut poles: Vec<_> = requested.iter().filter(|p| p.im == 0.).copied().collect();
    poles.sort_by(|a, b| a.re.total_cmp(&b.re));
    let mut negative: Vec<_> = requested.iter().filter(|p| p.im < 0.).copied().collect();
    negative.sort_by(|a, b| a.re.total_cmp(&b.re).then(a.im.total_cmp(&b.im)));
    for p in negative {
        if !requested.iter().any(|q| (*q - p.conj()).norm() < 1e-12) {
            return Err("Complex poles must come with their conjugates".into());
        }
        poles.push(p);
        poles.push(p.conj());
    }
    if poles.len() != n {
        return Err("Complex poles must come with their conjugates".into());
    }
    for &p in &poles {
        if poles.iter().filter(|q| (**q - p).norm() < 1e-12).count() > rank_b {
            return Err(
                "at least one of the requested pole is repeated more than rank(B) times".into(),
            );
        }
    }
    if rank_b == n {
        let mut target = Matrix::zeros(n, n);
        let mut i = 0;
        while i < n {
            target[(i, i)] = poles[i].re;
            if poles[i].im != 0. {
                target[(i, i + 1)] = -poles[i].im;
                target[(i + 1, i + 1)] = poles[i].re;
                target[(i + 1, i)] = poles[i].im;
                i += 1;
            }
            i += 1;
        }
        return b
            .clone()
            .svd(true, true)
            .solve(&(a - target), f64::EPSILON * b.norm().max(1.) * n as f64)
            .map_err(str::to_owned);
    }
    if rank_b == 0 || b.ncols() != rank_b {
        return Err("The poles you've chosen can't be placed. Check the controllability matrix and try another set of poles".into());
    }
    let q = complete_q(&complex_matrix(b));
    let u1 = q.columns(rank_b, n - rank_b);
    let ac = complex_matrix(a);
    let mut kernels = vec![];
    let mut x = Matrix::zeros(n, n);
    let mut i = 0;
    while i < n {
        let space = (u1.transpose() * (&ac - CMatrix::identity(n, n) * poles[i])).transpose();
        let q = complete_q(&space);
        let kernel = q.columns(n - rank_b, rank_b).into_owned();
        let mut vector = DVector::from_fn(n, |r, _| {
            (0..rank_b).map(|j| kernel[(r, j)]).sum::<Complex>()
        });
        let norm = vector.norm();
        vector /= Complex::new(norm, 0.);
        if poles[i].im == 0. {
            x.set_column(i, &vector.map(|v| v.re));
            kernels.push(kernel);
            i += 1;
        } else {
            x.set_column(i, &vector.map(|v| v.re));
            x.set_column(i + 1, &vector.map(|v| v.im));
            kernels.push(kernel.clone());
            kernels.push(kernel);
            i += 2;
        }
    }
    if rank_b > 1 {
        let order = update_order(&poles);
        for _ in 0..30 {
            let before = x.determinant().abs();
            for &(i, j) in &order {
                if i == j {
                    let q = orthogonal_q(&x, &[i]);
                    let kernel = kernels[i].map(|z| z.re);
                    let projected = &kernel * kernel.transpose() * q.column(n - 1).map(|z| z.re);
                    let norm = projected.norm();
                    if norm > 1e-8 {
                        x.set_column(i, &(projected / norm));
                    }
                } else {
                    let q = orthogonal_q(&x, &[i, j]);
                    if poles[i].im == 0. {
                        update_real(&kernels, &q, &mut x, i, j);
                    } else {
                        update_complex(&kernels, &q, &mut x, i, j);
                    }
                }
            }
            let after = x.determinant().abs().max(f64::EPSILON.sqrt());
            if ((after - before) / after).abs() < 1e-3 && after > f64::EPSILON.sqrt() {
                break;
            }
        }
    }
    let mut xc = complex_matrix(&x);
    let mut i = 0;
    while i < n {
        if poles[i].im != 0. {
            let real = x.column(i);
            let imag = x.column(i + 1);
            xc.set_column(
                i,
                &DVector::from_fn(n, |r, _| Complex::new(real[r], -imag[r])),
            );
            xc.set_column(
                i + 1,
                &DVector::from_fn(n, |r, _| Complex::new(real[r], imag[r])),
            );
            i += 2;
        } else {
            i += 1;
        }
    }
    let inverse=xc.clone().try_inverse().ok_or("The poles you've chosen can't be placed. Check the controllability matrix and try another set of poles")?;
    let target = xc * CMatrix::from_diagonal(&DVector::from_vec(poles)) * inverse;
    let gain = b
        .clone()
        .svd(true, true)
        .solve(
            &(a - target.map(|z| z.re)),
            f64::EPSILON * b.norm().max(1.) * n as f64,
        )
        .map_err(str::to_owned)?;
    let actual = eigenvalues(&(a - b * &gain))?;
    if requested.iter().any(|p| {
        !actual
            .iter()
            .any(|a| (*p - *a).norm() < 1e-5 * (1. + p.norm()))
    }) {
        return Err("The poles you've chosen can't be placed. Check the controllability matrix and try another set of poles".into());
    }
    Ok(gain)
}

fn representation(a: &Matrix) -> String {
    let fmt = |v: f64| {
        if (v - v.round()).abs() < 1e-9 {
            format!("{:.0}", v.round())
        } else {
            values::general(v, 4)
        }
    };
    if a.nrows() == 1 {
        return format!(
            "[{}]",
            (0..a.ncols())
                .map(|j| fmt(a[(0, j)]))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if a.ncols() == 1 {
        return format!(
            "[{}]",
            (0..a.nrows())
                .map(|i| fmt(a[(i, 0)]))
                .collect::<Vec<_>>()
                .join("; ")
        );
    }
    format!(
        "[{}]",
        (0..a.nrows())
            .map(|i| format!(
                "[{}]",
                (0..a.ncols())
                    .map(|j| fmt(a[(i, j)]))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn matrix_latex(name: &str, a: &Matrix) -> String {
    format!(
        "{name}=\\left[\\begin{{matrix}}{}\\end{{matrix}}\\right]",
        (0..a.nrows())
            .map(|i| (0..a.ncols())
                .map(|j| {
                    let v = a[(i, j)];
                    if v.fract() == 0. {
                        format!("{v:.1}")
                    } else {
                        values::general(v, 15)
                    }
                })
                .collect::<Vec<_>>()
                .join(" & "))
            .collect::<Vec<_>>()
            .join("\\\\")
    )
}

fn transfer_details(s: &System) -> Result<Value> {
    if s.b.ncols() != 1 || s.c.nrows() != 1 {
        return Ok(
            json!({"siso":false,"representation":"G(s) = C(sI-A)^{-1}B + D","inputs":s.b.ncols(),"outputs":s.c.nrows()}),
        );
    }
    let den = characteristic(&s.a);
    let mut num = vec![s.d[(0, 0)]];
    let mut work = s.b.clone();
    for &coefficient in den.iter().skip(1) {
        num.push((&s.c * &work)[(0, 0)] + s.d[(0, 0)] * coefficient);
        work = &s.a * work + &s.b * coefficient;
    }
    let num = polynomial::trim(&num.iter().map(|&x| clean(x)).collect::<Vec<_>>());
    let den = polynomial::trim(&den.iter().map(|&x| clean(x)).collect::<Vec<_>>());
    let tf = TransferFunction::new(num.clone(), den.clone())?;
    let (expanded, factored) = tf.latex_formatted(true);
    Ok(
        json!({"siso":true,"numerator":num,"denominator":den,"latex":factored.trim_start_matches("G(s) = "),"latex_expanded":expanded}),
    )
}

/// Pole maps use scatter symbols: connecting poles would falsely suggest a
/// trajectory. Both application themes intentionally use the academic white
/// plot palette, as the Python plot_theme does.
fn pole_plot(
    original: &[Complex],
    feedback: &[Complex],
    observer: &[Complex],
) -> Result<(String, String)> {
    use std::fmt::Write;
    let all: Vec<_> = original.iter().chain(feedback).chain(observer).collect();
    let mut xmin = all.iter().map(|p| p.re).fold(0., f64::min);
    let mut xmax = all.iter().map(|p| p.re).fold(0., f64::max);
    if xmin == xmax {
        xmin -= 1.;
        xmax += 1.;
    }
    let margin = (xmax - xmin) * 0.05;
    xmin -= margin;
    xmax += margin;
    let imaginary = all.iter().map(|p| p.im.abs()).fold(0., f64::max);
    let ymax = if imaginary < 1e-4 {
        1.
    } else {
        imaginary * 1.1
    };
    let x = |v: f64| 100. + (v - xmin) / (xmax - xmin) * 1110.;
    let y = |v: f64| 700. - (v + ymax) / (2. * ymax) * 660.;
    let mut svg = String::from(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1260" height="780" viewBox="0 0 1260 780"><rect width="1260" height="780" fill="white"/><g font-family="IBM Plex Sans" font-size="18" fill="black">"##,
    );
    for i in 0..=6 {
        let fraction = i as f64 / 6.;
        let xx = 100. + fraction * 1110.;
        let yy = 700. - fraction * 660.;
        let real = xmin + fraction * (xmax - xmin);
        let imag = -ymax + fraction * 2. * ymax;
        let _ = write!(
            svg,
            r##"<path d="M{xx:.4} 40V700 M100 {yy:.4}H1210" stroke="#CBD5E1" stroke-opacity=".75" stroke-dasharray="4 4" fill="none"/><text x="{xx:.4}" y="732" text-anchor="middle">{}</text><text x="85" y="{:.4}" text-anchor="end">{}</text>"##,
            presentation::escape(&values::general(real, 4)),
            yy + 6.,
            presentation::escape(&values::general(imag, 4))
        );
    }
    let _ = write!(
        svg,
        r##"<path data-axis="real-zero" d="M{:.4} 40V700" stroke="black" stroke-opacity=".7"/><path data-axis="imaginary-zero" d="M100 {:.4}H1210" stroke="black" stroke-opacity=".7"/><rect x="100" y="40" width="1110" height="660" stroke="black" fill="none"/>"##,
        x(0.),
        y(0.)
    );
    let groups = [
        ("original", "Sistema original", original, "#64748B"),
        ("feedback", "A - BK", feedback, "#000000"),
        ("observer", "A - LC", observer, "#334155"),
    ];
    let mut legend_row = 0;
    for (kind, label, points, color) in groups {
        if points.is_empty() {
            continue;
        }
        let mut marker = |px: f64, py: f64| match kind {
            "original" => {
                let _ = write!(
                    svg,
                    r##"<path data-pole-series="{kind}" d="M{:.4} {:.4}l12 12m-12 0l12 -12" stroke="{color}" stroke-width="2.2" fill="none"/>"##,
                    px - 6.,
                    py - 6.
                );
            }
            "feedback" => {
                let _ = write!(
                    svg,
                    r##"<circle data-pole-series="{kind}" cx="{px:.4}" cy="{py:.4}" r="7" fill="none" stroke="{color}" stroke-width="2"/>"##
                );
            }
            _ => {
                let _ = write!(
                    svg,
                    r##"<rect data-pole-series="{kind}" x="{:.4}" y="{:.4}" width="12" height="12" fill="none" stroke="{color}" stroke-width="2"/>"##,
                    px - 6.,
                    py - 6.
                );
            }
        };
        for point in points {
            marker(x(point.re), y(point.im));
        }
        let ly = 65. + legend_row as f64 * 28.;
        marker(1010., ly);
        let _ = write!(
            svg,
            r##"<text x="1030" y="{:.4}">{}</text>"##,
            ly + 6.,
            presentation::escape(label)
        );
        legend_row += 1;
    }
    svg.push_str(r##"<text x="655" y="770" text-anchor="middle">Parte real</text><text transform="translate(25 370) rotate(-90)" text-anchor="middle">Parte imaginária</text></g></svg>"##);
    Ok((presentation::rasterize(&svg)?, svg))
}

pub fn analyze(payload: &Value) -> Result<Value> {
    let mode = payload["mode"]
        .as_str()
        .unwrap_or("matrices")
        .to_lowercase();
    let mut source_latex = None;
    let system = match mode.as_str() {
        "matrices" => System {
            a: parse_matrix(
                payload.get("A").unwrap_or(&json!("[[0, 1], [-2, -3]]")),
                "A",
            )?,
            b: parse_matrix(payload.get("B").unwrap_or(&json!("[[0], [1]]")), "B")?,
            c: parse_matrix(payload.get("C").unwrap_or(&json!("[[1, 0]]")), "C")?,
            d: parse_matrix(payload.get("D").unwrap_or(&json!("[[0]]")), "D")?,
        },
        "transfer" => {
            let expression = payload["expr"]
                .as_str()
                .or_else(|| payload["expression"].as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("1 / (s^2 + 3*s + 2)");
            let tf = TransferFunction::parse(expression)?;
            source_latex = Some(tf.latex_formatted(false).0);
            let (a, b, c, d) = simulation::realization(&tf)?;
            let n = a.nrows();
            System {
                a,
                b: Matrix::from_column_slice(n, 1, b.as_slice()),
                c: Matrix::from_row_slice(1, n, c.as_slice()),
                d: Matrix::from_element(1, 1, d),
            }
        }
        _ => return Err("Escolha entrada por matrizes ou função de transferência.".into()),
    };
    validate(&system)?;
    let n = system.a.nrows();
    let wc = controllability(&system.a, &system.b);
    let wo = observability(&system.a, &system.c);
    let rank_c = rank(&wc);
    let rank_o = rank(&wo);
    let (canonical, transform, label) = canonical(
        &system,
        payload["canonical_form"].as_str().unwrap_or("controllable"),
    )?;
    let desired = poles(
        payload.get("desired_poles"),
        n,
        "A realimentação de estados",
    )?;
    let observer = poles(payload.get("observer_poles"), n, "O observador")?;
    let mut gain_k = None;
    let mut gain_l = None;
    let mut method_k = None;
    let mut method_l = None;
    let mut feedback_poles = vec![];
    let mut observer_poles = vec![];
    if !desired.is_empty() {
        if rank_c < n {
            return Err(
                "O sistema não é completamente controlável; não é possível alocar todos os polos."
                    .into(),
            );
        }
        let gain = if system.b.ncols() == 1 {
            method_k = Some("Fórmula de Ackermann");
            ackermann(&system.a, &system.b, &desired)?
        } else {
            method_k = Some("Alocação multivariável");
            place(&system.a, &system.b, &desired)?
        };
        feedback_poles = eigenvalues(&(&system.a - &system.b * &gain))?;
        gain_k = Some(gain);
    }
    if !observer.is_empty() {
        if rank_o < n {
            return Err("O sistema não é completamente observável; não é possível projetar o observador completo.".into());
        }
        let gain = if system.c.nrows() == 1 {
            method_l = Some("Ackermann dual — Luenberger");
            ackermann(&system.a.transpose(), &system.c.transpose(), &observer)?.transpose()
        } else {
            method_l = Some("Alocação dual multivariável — Luenberger");
            place(&system.a.transpose(), &system.c.transpose(), &observer)?.transpose()
        };
        observer_poles = eigenvalues(&(&system.a - &gain * &system.c))?;
        gain_l = Some(gain);
    }
    let original = eigenvalues(&system.a)?;
    let stable = original.iter().all(|p| p.re < -1e-9);
    let (image, svg) = pole_plot(&original, &feedback_poles, &observer_poles)?;
    let transfer = transfer_details(&system)?;
    // Preserve the conditional-expression precedence in the Python display:
    // integer real parts are displayed alone, even for a complex eigenvalue.
    let eigen_display = original
        .iter()
        .map(|p| {
            if (p.re - p.re.round()).abs() < 1e-9 {
                format!("{:.0}", p.re.round())
            } else if p.im.abs() < 1e-9 {
                values::general(p.re, 4)
            } else {
                format!(
                    "{}{}{}j",
                    values::general(p.re, 4),
                    if p.im < 0. { "-" } else { "+" },
                    values::general(p.im.abs(), 4)
                )
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    let mut metrics = vec![
        json!({"label":"Autovalores de A","value":eigen_display}),
        json!({"label":"Estabilidade","value":if stable{"Estável"}else{"Instável"}}),
        json!({"label":"Controlabilidade (posto)","value":format!("{rank_c} / {n}")}),
        json!({"label":"Observabilidade (posto)","value":format!("{rank_o} / {n}")}),
        json!({"label":"Forma canônica solicitada","value":label}),
    ];
    if label != "Original" {
        metrics.push(json!({"label":"Matrizes canônicas","value":format!("Ac={}, Bc={}, Cc={}, Dc={}",representation(&canonical.a),representation(&canonical.b),representation(&canonical.c),representation(&canonical.d))}));
    }
    metrics.push(json!({"label":"Ganho K","value":gain_k.as_ref().map(representation).unwrap_or_else(||"—".into())}));
    metrics.push(json!({"label":"Ganho L","value":gain_l.as_ref().map(representation).unwrap_or_else(||"—".into())}));
    if let Some(method) = method_k {
        metrics.push(json!({"label":"Realimentação","value":method}));
    }
    if let Some(method) = method_l {
        metrics.push(json!({"label":"Observador","value":method}));
    }
    let mut latex = vec![
        json!({"label":"Modelo","value":r"\dot{x}=Ax+Bu,\qquad y=Cx+Du"}),
        json!({"label":"Controlabilidade","value":r"\mathcal{C}=[B\;AB\;\cdots\;A^{n-1}B]"}),
        json!({"label":"Observabilidade","value":r"\mathcal{O}=[C^T\;A^TC^T\;\cdots\;(A^T)^{n-1}C^T]^T"}),
    ];
    if transfer["siso"] == true {
        latex.push(json!({"label":"Função de transferência","value":transfer["latex"]}));
    }
    if let Some(k) = &gain_k {
        latex.push(json!({"label":"Realimentação de estados","value":matrix_latex("K",k)}));
    }
    if let Some(l) = &gain_l {
        latex.push(json!({"label":"Ganho do observador","value":matrix_latex("L",l)}));
    }
    Ok(
        json!({"success":true,"module":"state_space","title":"Espaço de Estados","description":"Conversão de modelos, testes estruturais, realimentação de estados e observador de ordem plena.","image":image,"svg":svg,"latex":latex,"metrics":metrics,"details":{
            "source_mode":mode,"source_latex":source_latex,"stable":stable,"A":serialize(&system.a),"B":serialize(&system.b),"C":serialize(&system.c),"D":serialize(&system.d),"transfer_function":transfer,"controllability_matrix":serialize(&wc),"controllability_rank":rank_c,"observability_matrix":serialize(&wo),"observability_rank":rank_o,"canonical":{"label":label,"A":serialize(&canonical.a),"B":serialize(&canonical.b),"C":serialize(&canonical.c),"D":serialize(&canonical.d),"transformation":serialize_complex(&transform)},"state_feedback_gain":gain_k.as_ref().map(serialize),"state_feedback_method":method_k,"observer_gain":gain_l.as_ref().map(serialize),"observer_method":method_l,"original_poles":values::complex_list(&original),"feedback_poles":values::complex_list(&feedback_poles),"observer_error_poles":values::complex_list(&observer_poles)
        }}),
    )
}
