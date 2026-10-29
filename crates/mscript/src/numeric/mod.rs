//! Typed dense arrays in column-major order. Matrix preserves the original 2-D API.
mod array;
mod complex;
pub mod execution;
pub use array::DataType;
pub use complex::Complex;
pub type Array = Matrix;
use std::fmt;
use std::sync::Arc;

/// A bounded allocation policy for the initial interpreter, not a MATLAB limit.
pub const MAX_ELEMENTS: usize = 1_000_000;
const MAX_WORK: u128 = 50_000_000;

#[derive(Clone, Debug, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Arc<Vec<f64>>,
    shape: Vec<usize>,
    imaginary: Option<Arc<Vec<f64>>>,
    dtype: DataType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericError(pub String, pub Option<execution::ResourceFailure>);
impl NumericError {
    pub fn semantic(message: String) -> Self {
        Self(message, None)
    }
}
impl From<execution::ResourceFailure> for NumericError {
    fn from(e: execution::ResourceFailure) -> Self {
        Self(e.to_string(), Some(e))
    }
}

impl fmt::Display for NumericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for NumericError {}
type Result<T> = std::result::Result<T, NumericError>;

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<f64>) -> Result<Self> {
        let len = Self::checked_len(rows, cols)?;
        if len != data.len() {
            return Err(NumericError::semantic(
                "dimensões e dados incompatíveis".into(),
            ));
        }
        Self::new_nd(vec![rows, cols], data)
    }
    fn checked_len(rows: usize, cols: usize) -> Result<usize> {
        let len = rows
            .checked_mul(cols)
            .ok_or_else(|| NumericError::semantic("dimensões excedem o limite".into()))?;
        // Bound zero-width shapes as well: algorithms may iterate the dimensions.
        if rows > MAX_ELEMENTS || cols > MAX_ELEMENTS || len > MAX_ELEMENTS {
            return Err(NumericError::semantic(format!(
                "limite de {MAX_ELEMENTS} elementos excedido"
            )));
        }
        Ok(len)
    }
    pub fn scalar(value: f64) -> Self {
        Self {
            rows: 1,
            cols: 1,
            data: Arc::new(vec![value]),
            shape: vec![1, 1],
            imaginary: None,
            dtype: DataType::Double,
        }
    }
    pub fn filled(rows: usize, cols: usize, value: f64) -> Result<Self> {
        Self::filled_nd(vec![rows, cols], value)
    }
    pub fn identity(rows: usize, cols: usize) -> Result<Self> {
        let mut m = Self::filled(rows, cols, 0.0)?;
        for i in 0..rows.min(cols) {
            Arc::make_mut(&mut m.data)[i + i * rows] = 1.0;
        }
        Ok(m)
    }
    pub fn from_rows(rows: &[Vec<f64>]) -> Result<Self> {
        let nr = rows.len();
        let nc = rows.first().map_or(0, Vec::len);
        if rows.iter().any(|row| row.len() != nc) {
            return Err(NumericError::semantic(
                "linhas da matriz têm tamanhos diferentes".into(),
            ));
        }
        execution::payload(Self::checked_len(nr, nc)?, 8)?;
        let mut data = Vec::with_capacity(Self::checked_len(nr, nc)?);
        for c in 0..nc {
            execution::checkpoint(nr as u64)?;
            for row in rows {
                data.push(row[c]);
            }
        }
        Self::new_nd_inner(vec![nr, nc], data)
    }
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn data(&self) -> &[f64] {
        &self.data
    }
    pub fn is_scalar(&self) -> bool {
        self.data.len() == 1
    }
    pub fn scalar_value(&self) -> Result<f64> {
        if self.is_scalar() && !self.is_complex() {
            Ok(self.data[0])
        } else {
            Err(NumericError::semantic("esperado um escalar".into()))
        }
    }
    pub fn get(&self, row: usize, col: usize) -> Result<f64> {
        self.require_real()?;
        self.require_matrix()?;
        if row >= self.rows || col >= self.cols {
            return Err(NumericError::semantic("índice fora dos limites".into()));
        }
        Ok(self.data[row + col * self.rows])
    }
    /// Map real projections. Complex arithmetic uses map_complex explicitly.
    pub fn map(&self, f: impl Fn(f64) -> f64) -> Result<Self> {
        let mut out = self.real();
        out.data = Arc::new(execution::collect(self.data.len(), |i| f(self.data[i]))?);
        Ok(out)
    }
    pub fn zip(&self, rhs: &Self, f: impl Fn(f64, f64) -> f64) -> Result<Self> {
        self.require_real()?;
        rhs.require_real()?;
        self.zip_complex(rhs, |a, b| Complex::new(f(a.re, b.re), 0.))
    }
    pub fn multiply(&self, rhs: &Self) -> Result<Self> {
        if self.is_scalar() || rhs.is_scalar() {
            return self.zip_complex(rhs, |a, b| a * b);
        }
        self.require_matrix()?;
        rhs.require_matrix()?;
        if self.is_complex() || rhs.is_complex() {
            return self.multiply_complex(rhs);
        }
        if self.cols != rhs.rows {
            return Err(NumericError::semantic(
                "dimensões incompatíveis para multiplicação matricial".into(),
            ));
        }
        if self.rows as u128 * self.cols as u128 * rhs.cols as u128 > MAX_WORK {
            return Err(NumericError::semantic(
                "multiplicação excede o orçamento de operações deste marco".into(),
            ));
        }
        execution::checkpoint((self.rows * self.cols * rhs.cols) as u64)?;
        let mut out = Self::filled(self.rows, rhs.cols, 0.0)?;
        for c in 0..rhs.cols {
            for k in 0..self.cols {
                execution::checkpoint(0)?;
                let b = rhs.data[k + c * rhs.rows];
                for r in 0..self.rows {
                    Arc::make_mut(&mut out.data)[r + c * self.rows] +=
                        self.data[r + k * self.rows] * b;
                }
            }
        }
        Ok(out)
    }
    pub fn transpose(&self) -> Result<Self> {
        let mut shape = self.shape.clone();
        shape.swap(0, 1);
        let mut out = self.clone();
        out.shape = shape;
        out.rows = self.cols;
        out.cols = self.rows;
        execution::payload(self.data.len(), if self.is_complex() { 16 } else { 8 })?;
        let mut data = vec![0.; self.data.len()];
        let mut imaginary = self.imaginary.as_ref().map(|_| vec![0.; self.data.len()]);
        let page_len = self.rows * self.cols;
        for offset in (0..self.data.len()).step_by(page_len.max(1)) {
            for c in 0..self.cols {
                execution::checkpoint(self.rows as u64)?;
                for r in 0..self.rows {
                    let src = offset + r + c * self.rows;
                    let dst = offset + c + r * self.cols;
                    data[dst] = self.data[src];
                    if let Some(im) = &mut imaginary {
                        im[dst] = self.imaginary.as_ref().unwrap()[src];
                    }
                }
            }
        }
        out.data = Arc::new(data);
        out.imaginary = imaginary.map(Arc::new);
        Ok(out)
    }
    /// Determinant of a finite square matrix, using partial pivoting.
    pub fn determinant(&self) -> Result<f64> {
        self.require_real()?;
        self.require_matrix()?;
        if self.rows != self.cols {
            return Err(NumericError::semantic("det requer matriz quadrada".into()));
        }
        if self.data.iter().any(|v| !v.is_finite()) {
            return Err(NumericError::semantic(
                "det requer valores finitos neste marco".into(),
            ));
        }
        let n = self.rows;
        if (n as u128).pow(3) > MAX_WORK {
            return Err(NumericError::semantic(
                "det excede o orçamento de operações deste marco".into(),
            ));
        }
        execution::payload(self.data.len(), 8)?;
        let mut a = self.clone();
        let mut determinant = 1.0;
        for k in 0..n {
            execution::checkpoint(((n - k) * (n - k)) as u64)?;
            let pivot = (k..n)
                .max_by(|&i, &j| a.data[i + k * n].abs().total_cmp(&a.data[j + k * n].abs()))
                .unwrap_or(k);
            if a.data[pivot + k * n] == 0.0 {
                return Ok(0.0);
            }
            if pivot != k {
                for c in k..n {
                    Arc::make_mut(&mut a.data).swap(k + c * n, pivot + c * n);
                }
                determinant = -determinant;
            }
            let diagonal = a.data[k + k * n];
            determinant *= diagonal;
            for r in k + 1..n {
                let factor = a.data[r + k * n] / diagonal;
                for c in k + 1..n {
                    Arc::make_mut(&mut a.data)[r + c * n] -= factor * a.data[k + c * n];
                }
            }
        }
        Ok(determinant)
    }
    /// Solve square A*X=B by Gaussian elimination with partial pivoting.
    /// Rectangular least-squares systems are explicitly unsupported.
    pub fn solve(&self, rhs: &Self) -> Result<Self> {
        self.require_real()?;
        rhs.require_real()?;
        self.require_matrix()?;
        rhs.require_matrix()?;
        if self.is_scalar() {
            return rhs.map(|b| b / self.data[0]);
        }
        if self.rows != self.cols || self.rows != rhs.rows {
            return Err(NumericError::semantic(
                "A\\B requer A quadrada e B com o mesmo número de linhas neste marco".into(),
            ));
        }
        if self
            .data
            .iter()
            .chain(rhs.data.iter())
            .any(|v| !v.is_finite())
        {
            return Err(NumericError::semantic(
                "sistema linear requer valores finitos".into(),
            ));
        }
        let n = self.rows;
        if n as u128 * n as u128 * (n as u128 + rhs.cols as u128) > MAX_WORK {
            return Err(NumericError::semantic(
                "sistema excede o orçamento de operações deste marco".into(),
            ));
        }
        execution::payload(self.data.len() + rhs.data.len(), 8)?;
        let mut a = self.clone();
        let mut x = rhs.real();
        for k in 0..n {
            execution::checkpoint(((n - k) * (n - k)) as u64)?;
            let pivot = (k..n)
                .max_by(|&i, &j| a.data[i + k * n].abs().total_cmp(&a.data[j + k * n].abs()))
                .unwrap_or(k);
            if a.data[pivot + k * n] == 0.0 {
                return Err(NumericError::semantic("matriz singular".into()));
            }
            for c in 0..n {
                Arc::make_mut(&mut a.data).swap(k + c * n, pivot + c * n);
            }
            for c in 0..x.cols {
                Arc::make_mut(&mut x.data).swap(k + c * n, pivot + c * n);
            }
            for r in k + 1..n {
                let factor = a.data[r + k * n] / a.data[k + k * n];
                Arc::make_mut(&mut a.data)[r + k * n] = 0.0;
                for c in k + 1..n {
                    Arc::make_mut(&mut a.data)[r + c * n] -= factor * a.data[k + c * n];
                }
                for c in 0..x.cols {
                    Arc::make_mut(&mut x.data)[r + c * n] -= factor * x.data[k + c * n];
                }
            }
        }
        for r in (0..n).rev() {
            execution::checkpoint((n * x.cols) as u64)?;
            for c in 0..x.cols {
                for k in r + 1..n {
                    Arc::make_mut(&mut x.data)[r + c * n] -= a.data[r + k * n] * x.data[k + c * n];
                }
                Arc::make_mut(&mut x.data)[r + c * n] /= a.data[r + r * n];
            }
        }
        Ok(x)
    }
    pub fn power(&self, exponent: f64) -> Result<Self> {
        if self.is_scalar() {
            return self.element_power(&Self::scalar(exponent));
        }
        self.require_matrix()?;
        if self.rows != self.cols
            || !exponent.is_finite()
            || exponent.fract() != 0.0
            || exponent.abs() > 1024.0
        {
            return Err(NumericError::semantic(
                "potência matricial requer matriz quadrada e expoente inteiro entre -1024 e 1024"
                    .into(),
            ));
        }
        let mut base = if exponent < 0.0 {
            self.solve(&Self::identity(self.rows, self.cols)?)?
        } else {
            self.clone()
        };
        let mut out = Self::identity(self.rows, self.cols)?;
        let mut n = exponent.abs() as u32;
        while n > 0 {
            execution::checkpoint(0)?;
            if n & 1 == 1 {
                out = out.multiply(&base)?;
            }
            n >>= 1;
            if n > 0 {
                base = base.multiply(&base)?;
            }
        }
        Ok(out)
    }
    pub fn element_power(&self, rhs: &Self) -> Result<Self> {
        self.zip_complex(rhs, Complex::pow)
    }
    pub fn range(start: f64, step: f64, end: f64) -> Result<Self> {
        if !start.is_finite() || !step.is_finite() || !end.is_finite() || step == 0.0 {
            return Err(NumericError::semantic(
                "intervalo requer valores finitos e passo não nulo".into(),
            ));
        }
        let distance = (end - start) / step;
        if distance < 0.0 {
            return Self::new(1, 0, vec![]);
        }
        // Tolerance includes ordinary decimal endpoints, never an unbounded loop.
        let count = (distance + 4.0 * f64::EPSILON * distance.abs().max(1.0)).floor() + 1.0;
        if !count.is_finite() || count > MAX_ELEMENTS as f64 {
            return Err(NumericError::semantic(
                "intervalo excede o limite de elementos".into(),
            ));
        }
        let len = count as usize;
        Self::new_nd_inner(
            vec![1, len],
            execution::collect(len, |i| start + step * i as f64)?,
        )
    }
    pub fn sum(&self, dimension: Option<usize>) -> Result<Self> {
        self.reduce_sum(dimension)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.shape.len() > 2 {
            writeln!(
                f,
                "Array {:?} ({}) — primeira página:",
                self.shape,
                self.class()
            )?;
        }
        if self.data.is_empty() {
            return if self.shape.len() == 2 {
                write!(f, "[] ({}x{})", self.rows, self.cols)
            } else {
                write!(f, "[] {:?}", self.shape)
            };
        }
        for r in 0..self.rows {
            if r > 0 {
                writeln!(f)?;
            }
            for c in 0..self.cols {
                if c > 0 {
                    write!(f, "  ")?;
                }
                write!(f, "{}", self.element(r + c * self.rows))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn determinant_handles_pivot_sign_singularity_and_empty_matrix() {
        let a = Matrix::from_rows(&[vec![1., 2.], vec![3., 4.]]).unwrap();
        assert!((a.determinant().unwrap() + 2.).abs() < 1e-12);
        assert_eq!(
            Matrix::from_rows(&[vec![0., 2.], vec![1., 3.]])
                .unwrap()
                .determinant()
                .unwrap(),
            -2.
        );
        assert_eq!(Matrix::filled(2, 2, 1.).unwrap().determinant().unwrap(), 0.);
        assert_eq!(Matrix::filled(0, 0, 0.).unwrap().determinant().unwrap(), 1.);
        assert!(Matrix::filled(2, 3, 1.).unwrap().determinant().is_err());
        assert!(Matrix::scalar(f64::NAN).determinant().is_err());
    }
    #[test]
    fn storage_and_multiplication() {
        let a = Matrix::from_rows(&[vec![1., 2.], vec![3., 4.]]).unwrap();
        assert_eq!(a.data(), &[1., 3., 2., 4.]);
        assert_eq!(a.multiply(&a).unwrap().data(), &[7., 15., 10., 22.]);
        assert_eq!(a.transpose().unwrap().transpose().unwrap(), a);
    }
    #[test]
    fn solve_pivots_and_multiple_rhs() {
        let a = Matrix::from_rows(&[vec![0., 2.], vec![1., 3.]]).unwrap();
        let b = Matrix::from_rows(&[vec![4., 2.], vec![7., 4.]]).unwrap();
        let x = a.solve(&b).unwrap();
        assert_eq!(x.data(), &[1., 2., 1., 1.]);
        assert_eq!(a.multiply(&x).unwrap(), b);
        assert!(Matrix::filled(2, 2, 1.).unwrap().solve(&b).is_err());
    }
    #[test]
    fn scaled_system_and_negative_power() {
        let a = Matrix::from_rows(&[vec![1e-200, 0.], vec![0., 2e-200]]).unwrap();
        let b = Matrix::from_rows(&[vec![3e-200], vec![8e-200]]).unwrap();
        assert_eq!(a.solve(&b).unwrap().data(), &[3., 4.]);
        let d = Matrix::from_rows(&[vec![2., 0.], vec![0., 4.]]).unwrap();
        assert_eq!(d.power(-1.).unwrap().data(), &[0.5, 0., 0., 0.25]);
    }
    #[test]
    fn ranges_and_allocation_limits() {
        assert_eq!(Matrix::range(3., -1., 1.).unwrap().data(), &[3., 2., 1.]);
        assert_eq!(Matrix::range(1., 1., 0.).unwrap().cols(), 0);
        assert_eq!(Matrix::range(0., 0.1, 0.3).unwrap().cols(), 4);
        assert!(Matrix::range(1., 0., 2.).is_err());
        assert!(Matrix::range(0., 1., f64::MAX).is_err());
        assert!(Matrix::filled(usize::MAX, 0, 0.).is_err());
        assert!(Matrix::new(2, 2, vec![1.]).is_err());
    }
}
