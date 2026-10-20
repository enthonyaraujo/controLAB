use super::execution;
use super::{Complex, Matrix, NumericError, Result, MAX_ELEMENTS, MAX_WORK};
use std::sync::Arc;

/// Initial dense language types. Logical values are canonical 0/1 internally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    Double,
    Logical,
}
pub const MAX_RANK: usize = 32;

impl Matrix {
    /// Concatenate along a zero-based axis. Empty operands contribute to type
    /// promotion, even when their dimensions are ignored beside nonempty arrays.
    pub fn concatenate(axis: usize, arrays: &[Self]) -> Result<Self> {
        if axis > 1 {
            return Err(NumericError::semantic(
                "concatenação literal requer eixo 1 ou 2".into(),
            ));
        }
        if arrays.is_empty() {
            return Self::new(0, 0, vec![]);
        }
        if arrays.len() == 1 {
            return Ok(arrays[0].clone());
        }
        let logical = arrays.iter().all(|a| a.dtype == DataType::Logical);
        let complex = arrays.iter().any(Self::is_complex);
        let nonempty = arrays.iter().any(|a| !a.data.is_empty());
        let active: Vec<_> = arrays
            .iter()
            .filter(|a| {
                if nonempty {
                    !a.data.is_empty()
                } else {
                    a.shape != [0, 0]
                }
            })
            .collect();
        if active.is_empty() {
            let out = Self::new(0, 0, vec![])?;
            return if logical { out.logical() } else { Ok(out) };
        }
        let rank = active.iter().map(|a| a.shape.len()).max().unwrap();
        let mut shape = (0..rank)
            .map(|d| active[0].dimension(d))
            .collect::<Vec<_>>();
        shape[axis] = 0;
        for a in &active {
            for (d, &size) in shape.iter().enumerate() {
                if d != axis && size != a.dimension(d) {
                    return Err(NumericError::semantic(
                        "dimensões incompatíveis na concatenação".into(),
                    ));
                }
            }
            shape[axis] = shape[axis]
                .checked_add(a.dimension(axis))
                .ok_or_else(|| NumericError::semantic("concatenação excede usize".into()))?;
        }
        let len = Self::shape_len(&shape)?;
        execution::payload(len,16)?;
        let mut data = Vec::with_capacity(len);
        if len != 0 {
            let prefix = shape[..axis].iter().product::<usize>();
            let outer = shape[axis + 1..].iter().product::<usize>();
            for page in 0..outer {
                for a in &active {
                    execution::checkpoint(0)?;
                    let block = prefix * a.dimension(axis);
                    let start = page * block;
                    for i in start..start+block{if (i-start)%256==0{execution::checkpoint((start+block-i).min(256) as u64)?;}data.push(a.element(i));}
                }
            }
        }
        let out = Self::from_complex(shape, data, complex)?;
        if logical {
            out.logical()
        } else {
            Ok(out)
        }
    }
    pub fn new_nd(shape:Vec<usize>,data:Vec<f64>)->Result<Self>{
        execution::payload(data.len(),8)?;
        Self::new_nd_inner(shape,data)
    }
    pub(crate) fn new_nd_inner(mut shape: Vec<usize>, data: Vec<f64>) -> Result<Self> {
        if shape.is_empty() || shape.len() > MAX_RANK {
            return Err(NumericError::semantic("rank fora da política 1..32".into()));
        }
        while shape.len() < 2 {
            shape.push(1);
        }
        while shape.len() > 2 && shape.last() == Some(&1) {
            shape.pop();
        }
        let len = Self::shape_len(&shape)?;
        if data.len() != len {
            return Err(NumericError::semantic("dimensões e dados incompatíveis".into()));
        }
        Ok(Self {
            rows: shape[0],
            cols: shape[1],
            shape,
            data: Arc::new(data),
            imaginary: None,
            dtype: DataType::Double,
        })
    }
    pub fn shape_len(shape: &[usize]) -> Result<usize> {
        if shape.len() > MAX_RANK || shape.iter().any(|&n| n > MAX_ELEMENTS) {
            return Err(NumericError::semantic(
                "dimensões excedem a política de alocação".into(),
            ));
        }
        let count = shape
            .iter()
            .try_fold(1usize, |a, &b| a.checked_mul(b))
            .ok_or_else(|| NumericError::semantic("dimensões excedem usize".into()))?;
        shape
            .iter()
            .rev()
            .try_fold(1usize, |a, &b| a.checked_mul(b))
            .ok_or_else(|| NumericError::semantic("dimensões colapsadas excedem usize".into()))?;
        if count > MAX_ELEMENTS {
            return Err(NumericError::semantic("limite de elementos excedido".into()));
        }
        Ok(count)
    }
    pub fn filled_nd(shape: Vec<usize>, value: f64) -> Result<Self> {
        let n = Self::shape_len(&shape)?;
        execution::payload(n,8)?;
        Self::new_nd_inner(shape, vec![value; n])
    }
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }
    pub fn dimension(&self, axis: usize) -> usize {
        self.shape.get(axis).copied().unwrap_or(1)
    }
    pub fn dtype(&self) -> DataType {
        self.dtype
    }
    pub fn class(&self) -> &'static str {
        match self.dtype {
            DataType::Double => "double",
            DataType::Logical => "logical",
        }
    }
    pub fn is_complex(&self) -> bool {
        self.imaginary.is_some()
    }
    pub fn imaginary(&self) -> Option<&[f64]> {
        self.imaginary.as_ref().map(|v| v.as_slice())
    }
    pub fn is_vector(&self) -> bool {
        self.shape.len() == 2 && (self.rows == 1 || self.cols == 1)
    }
    pub fn element(&self, index: usize) -> Complex {
        Complex::new(
            self.data[index],
            self.imaginary.as_ref().map_or(0., |v| v[index]),
        )
    }
    pub fn require_real(&self) -> Result<()> {
        if self.is_complex() {
            Err(NumericError::semantic(
                "operação requer dados reais neste incremento".into(),
            ))
        } else {
            Ok(())
        }
    }
    pub fn require_matrix(&self) -> Result<()> {
        if self.shape.len() > 2 {
            Err(NumericError::semantic("operação matricial requer array 2D".into()))
        } else {
            Ok(())
        }
    }
    pub fn reshape(&self, mut shape: Vec<usize>) -> Result<Self> {
        execution::checkpoint(0)?;
        if shape.is_empty() || shape.len() > MAX_RANK {
            return Err(NumericError::semantic("rank fora da política 1..32".into()));
        }
        while shape.len() < 2 {
            shape.push(1);
        }
        while shape.len() > 2 && shape.last() == Some(&1) {
            shape.pop();
        }
        if Self::shape_len(&shape)? != self.data.len() {
            return Err(NumericError::semantic("reshape deve preservar numel".into()));
        }
        let mut out = self.clone();
        out.rows = shape[0];
        out.cols = shape[1];
        out.shape = shape;
        Ok(out)
    }
    pub fn to_double(&self)->Self{let mut out=self.clone();out.dtype=DataType::Double;out}
    pub fn real(&self) -> Self {
        let mut out = self.clone();
        out.imaginary = None;
        out.dtype = DataType::Double;
        out
    }
    pub fn imag(&self) -> Result<Self> {
        let mut out = self.real();
        out.data = self
            .imaginary
            .clone()
            .map(Ok).unwrap_or_else(||execution::collect(self.data.len(),|_|0.).map(Arc::new))?;
        Ok(out)
    }
    pub fn logical(&self)->Result<Self>{
        self.require_real()?;let mut out=self.real();
        out.data=Arc::new(execution::collect(self.data.len(),|i|if self.data[i]!=0.{1.}else{0.})?);
        out.dtype=DataType::Logical;Ok(out)
    }
    pub fn logical_scalar(value: bool) -> Self {
        let mut out = Self::scalar(if value { 1. } else { 0. });
        out.dtype = DataType::Logical;
        out
    }
    pub fn from_complex(shape:Vec<usize>,values:Vec<Complex>,force:bool)->Result<Self>{
        let complex=force||values.iter().any(|v|v.im!=0.);
        let real=execution::collect(values.len(),|i|values[i].re)?;
        let mut out=Self::new_nd_inner(shape,real)?;
        if complex{out.imaginary=Some(Arc::new(execution::collect(values.len(),|i|values[i].im)?));}Ok(out)
    }
    pub fn map_complex(&self,f:impl Fn(Complex)->Complex)->Result<Self>{
        Self::from_complex(self.shape.clone(),execution::collect(self.data.len(),|i|f(self.element(i)))?,false)
    }
    pub fn project(&self,f:impl Fn(Complex)->f64)->Result<Self>{
        let mut out=self.real();out.data=Arc::new(execution::collect(self.data.len(),|i|f(self.element(i)))?);Ok(out)
    }
    pub fn zip_complex(&self, rhs: &Self, f: impl Fn(Complex, Complex) -> Complex) -> Result<Self> {
        let rank = self.shape.len().max(rhs.shape.len());
        let mut shape = Vec::with_capacity(rank);
        for axis in 0..rank {
            let (a, b) = (self.dimension(axis), rhs.dimension(axis));
            shape.push(if a == b {
                a
            } else if a == 1 {
                b
            } else if b == 1 {
                a
            } else {
                return Err(NumericError::semantic(
                    "dimensões incompatíveis para expansão implícita".into(),
                ));
            });
        }
        let n = Self::shape_len(&shape)?;
        execution::payload(n,16)?;
        let mut data = Vec::with_capacity(n);
        for index in 0..n {
            if index%256==0{execution::checkpoint(((n-index).min(256)*rank) as u64)?;}
            let (mut rest, mut a, mut b, mut sa, mut sb) = (index, 0, 0, 1, 1);
            for (axis, &size) in shape.iter().enumerate() {
                let coordinate = rest % size;
                rest /= size;
                if self.dimension(axis) != 1 {
                    a += coordinate * sa;
                }
                if rhs.dimension(axis) != 1 {
                    b += coordinate * sb;
                }
                sa *= self.dimension(axis);
                sb *= rhs.dimension(axis);
            }
            data.push(f(self.element(a), rhs.element(b)));
        }
        Self::from_complex(shape, data, false)
    }
    pub fn conjugate(&self)->Result<Self>{let mut out=self.clone();if let Some(im)=&self.imaginary{out.imaginary=Some(Arc::new(execution::collect(im.len(),|i|-im[i])?));}Ok(out)}
    pub fn gather(&self, indices: &[usize], shape: Vec<usize>) -> Result<Self> {
        if indices.iter().any(|&i| i >= self.data.len()) {
            return Err(NumericError::semantic("índice fora dos limites".into()));
        }
        let mut out = Self::from_complex(
            shape,
            execution::collect(indices.len(),|i|self.element(indices[i]))?,
            self.is_complex(),
        )?;
        out.dtype = self.dtype;
        Ok(out)
    }
    /// Validate the whole assignment before detaching or writing. Repeated indices
    /// apply in column-major order; later writes win, like ordinary assignments.
    pub fn scatter(&mut self,indices:&[usize],rhs:&Self)->Result<()>{
        let mut candidate=self.clone();candidate.scatter_inner(indices,rhs)?;*self=candidate;Ok(())
    }
    fn scatter_inner(&mut self, indices: &[usize], rhs: &Self) -> Result<()> {
        execution::checkpoint(0)?;
        if self.dtype == DataType::Logical {
            rhs.require_real()?;
        }
        if indices.iter().any(|&i| i >= self.data.len())
            || (!rhs.is_scalar() && indices.len() != rhs.data.len())
        {
            return Err(NumericError::semantic(
                "índices ou quantidade de valores incompatíveis".into(),
            ));
        }
        if indices.is_empty() {
            return Ok(());
        }
        if self.dtype == DataType::Double && rhs.is_complex() && self.imaginary.is_none() {
            execution::payload(self.data.len(),8)?;
            self.imaginary = Some(Arc::new(vec![0.; self.data.len()]));
        }
        execution::checkpoint(indices.len() as u64)?;
        if Arc::strong_count(&self.data)>1{execution::payload(self.data.len(),8)?;}
        if let Some(im)=&self.imaginary{if Arc::strong_count(im)>1{execution::payload(im.len(),8)?;}}
        let real = Arc::make_mut(&mut self.data);
        let mut imaginary = self.imaginary.as_mut().map(Arc::make_mut);
        for (position, &index) in indices.iter().enumerate() {
            if position%256==0{execution::checkpoint(0)?;}
            let value = rhs.element(if rhs.is_scalar() { 0 } else { position });
            real[index] = if self.dtype == DataType::Logical {
                if value.re != 0. || value.im != 0. {
                    1.
                } else {
                    0.
                }
            } else {
                value.re
            };
            if let Some(im) = &mut imaginary {
                im[index] = value.im;
            }
        }
        if self
            .imaginary
            .as_ref()
            .is_some_and(|im| im.iter().all(|&v| v == 0.))
        {
            self.imaginary = None;
        }
        Ok(())
    }
    pub fn multiply_complex(&self, rhs: &Self) -> Result<Self> {
        self.require_matrix()?;
        rhs.require_matrix()?;
        if self.cols != rhs.rows
            || self.rows as u128 * self.cols as u128 * rhs.cols as u128 > MAX_WORK
        {
            return Err(NumericError::semantic(
                "dimensões ou orçamento de multiplicação inválidos".into(),
            ));
        }
        let len = Self::shape_len(&[self.rows, rhs.cols])?;
        execution::payload(len,16)?;
        execution::checkpoint((self.rows*self.cols*rhs.cols) as u64)?;
        let mut data = vec![Complex::default(); len];
        for c in 0..rhs.cols {
            for k in 0..self.cols {
                execution::checkpoint(0)?;
                for r in 0..self.rows {
                    let index = r + c * self.rows;
                    data[index] = data[index]
                        + self.element(r + k * self.rows) * rhs.element(k + c * rhs.rows);
                }
            }
        }
        Self::from_complex(vec![self.rows, rhs.cols], data, false)
    }
    pub fn first_nonsingleton(&self) -> usize {
        self.shape.iter().position(|&n| n != 1).unwrap_or(0)
    }
    pub fn reduce_sum(&self, dimension: Option<usize>) -> Result<Self> {
        if dimension == Some(0) {
            return Err(NumericError::semantic("dimensão deve ser positiva".into()));
        }
        if dimension.is_none() && self.shape == [0, 0] {
            return Ok(Self::scalar(0.));
        }
        let axis = dimension.map_or_else(|| self.first_nonsingleton(), |d| d - 1);
        if axis >= self.shape.len() {
            return self.map_complex(|x| x);
        }
        let mut shape = self.shape.clone();
        shape[axis] = 1;
        let stride = self.shape[..axis].iter().product::<usize>();
        let n = self.shape[axis];
        execution::payload(Self::shape_len(&shape)?,16)?;
        let mut data = vec![Complex::default(); Self::shape_len(&shape)?];
        for (i, out) in data.iter_mut().enumerate() {
            execution::checkpoint(n as u64)?;
            let base = i % stride + (i / stride) * stride * n;
            for k in 0..n {
                *out = *out + self.element(base + k * stride);
            }
        }
        Self::from_complex(shape, data, false)
    }
}
