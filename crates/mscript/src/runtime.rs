use crate::ast::{Expr, Kind, Operator};
use crate::control::{ControlError, TransferFunction};
use crate::execution::Stream;
use crate::graphics::{Figure, MAX_EVENTS, MAX_EXECUTION_POINTS, PlotEvent, PlotSeries};
use crate::numeric::execution::{self, ExecutionScope};
use crate::numeric::{Complex, DataType, MAX_ELEMENTS, NumericError};
use crate::{Diagnostic, Matrix, Result, Span, parser};
use crate::{
    ExecutionEvent, ExecutionEventKind, ExecutionOptions, ExecutionOutcome, ExecutionStatus,
};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Numeric(Matrix),
    Text(String),
    TransferFunction(TransferFunction),
}
impl Value {
    pub fn payload_bytes(&self) -> u64 {
        match self {
            Self::Numeric(m) => {
                m.data().len() as u64 * if m.is_complex() { 16 } else { 8 }
                    + m.shape().len() as u64 * 8
            }
            Self::Text(s) => s.len() as u64,
            Self::TransferFunction(g) => (g.numerator().len() + g.denominator().len()) as u64 * 8,
        }
    }
    pub(crate) fn try_clone(&self, span: Span) -> Result<Self> {
        let bytes = match self {
            Self::Numeric(m) => m.shape().len() as u64 * 8,
            Self::Text(s) => s.len() as u64,
            Self::TransferFunction(g) => (g.numerator().len() + g.denominator().len()) as u64 * 8,
        };
        execution::allocation(bytes).map_err(|e| Diagnostic::resource(e, span))?;
        Ok(self.clone())
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numeric(m) => write!(f, "{m}"),
            Self::Text(s) => f.write_str(s),
            Self::TransferFunction(model) => write!(f, "{model}"),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub name: Option<String>,
    pub value: Value,
}
#[derive(Clone, Debug, Default)]
pub struct Execution {
    pub outputs: Vec<Output>,
    pub plots: Vec<PlotEvent>,
}

/// Output count belongs to the call, including discarded (~) output slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallContext {
    pub nargin: usize,
    pub nargout: usize,
}

/// Persistent workspace, reusable by CLI and future native desktop frontends.
#[derive(Default)]
pub struct Session {
    variables: BTreeMap<String, Value>,
    workspace_bytes: u64,
}
impl Session {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn workspace(&self) -> &BTreeMap<String, Value> {
        &self.variables
    }
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
    pub fn clear(&mut self) {
        self.variables.clear();
        self.workspace_bytes = 0;
    }
    /// Compatibility adapter: successful runs return collected results; errors
    /// preserve prior workspace assignments. Streaming callers retain prefixes.
    pub fn execute(&mut self, source: &str) -> Result<Execution> {
        let outcome = self.execute_with_events(source, &ExecutionOptions::default(), |_| Ok(()));
        if let Some(error) = outcome.diagnostic {
            Err(error)
        } else {
            Ok(outcome.execution)
        }
    }
    pub fn execute_with_events(
        &mut self,
        source: &str,
        options: &ExecutionOptions,
        mut callback: impl FnMut(&ExecutionEvent) -> Result<()>,
    ) -> ExecutionOutcome {
        self.controlled(options, &mut callback, |session, stream| {
            session.run(source, options, stream)
        })
    }
    pub fn clear_with_events(
        &mut self,
        options: &ExecutionOptions,
        mut callback: impl FnMut(&ExecutionEvent) -> Result<()>,
    ) -> ExecutionOutcome {
        self.controlled(options, &mut callback, |session, stream| {
            execution::checkpoint(0)
                .map_err(|e| Diagnostic::resource(e, Span { line: 1, column: 1 }))?;
            session.clear();
            stream.emit(ExecutionEventKind::WorkspaceCleared, false)?;
            stream.output(Output {
                name: None,
                value: Value::Text("Workspace limpo.".into()),
            })
        })
    }
    fn controlled(
        &mut self,
        options: &ExecutionOptions,
        callback: &mut dyn FnMut(&ExecutionEvent) -> Result<()>,
        run: impl FnOnce(&mut Self, &mut Stream<'_>) -> Result<()>,
    ) -> ExecutionOutcome {
        let scope = ExecutionScope::enter(options.resources.clone(), options.cancellation.clone());
        let mut stream = Stream::new(options, callback);
        let result = stream
            .emit(ExecutionEventKind::Started, true)
            .and_then(|()| run(self, &mut stream));
        let mut diagnostic = result.err();
        let usage = scope.usage();
        if let Some(error) = &diagnostic {
            let _ = stream.emit(ExecutionEventKind::Error(error.clone()), true);
        }
        let status = match &diagnostic {
            None => ExecutionStatus::Succeeded,
            Some(e) if e.category == crate::DiagnosticCategory::Cancelled => {
                ExecutionStatus::Cancelled
            }
            _ => ExecutionStatus::Failed,
        };
        if let Err(error) = stream.emit(ExecutionEventKind::Finished { status, usage }, true)
            && diagnostic.is_none()
        {
            diagnostic = Some(error);
        }
        let status = match &diagnostic {
            None => ExecutionStatus::Succeeded,
            Some(e) if e.category == crate::DiagnosticCategory::Cancelled => {
                ExecutionStatus::Cancelled
            }
            _ => ExecutionStatus::Failed,
        };
        ExecutionOutcome {
            execution: stream.execution,
            diagnostic,
            usage,
            status,
        }
    }
    fn bind(
        &mut self,
        bindings: &[(String, Value)],
        options: &ExecutionOptions,
        span: Span,
    ) -> Result<()> {
        execution::checkpoint(bindings.len() as u64).map_err(|e| Diagnostic::resource(e, span))?;
        let proposed: BTreeMap<_, _> = bindings.iter().map(|(n, v)| (n.as_str(), v)).collect();
        let mut bytes = self.workspace_bytes;
        let mut variables = self.variables.len();
        for (name, value) in &proposed {
            if let Some(old) = self.variables.get(*name) {
                bytes -= old.payload_bytes() + name.len() as u64;
            } else {
                variables += 1;
            }
            bytes = bytes
                .checked_add(value.payload_bytes() + name.len() as u64)
                .ok_or_else(|| Diagnostic::limit("workspace excede orçamento", span))?;
        }
        if bytes > options.max_workspace_bytes || variables > options.max_variables {
            return Err(Diagnostic::limit(
                "workspace excede orçamento de dados ou variáveis",
                span,
            ));
        }
        let prepared = bindings
            .iter()
            .map(|(name, value)| Ok((name.clone(), value.try_clone(span)?)))
            .collect::<Result<Vec<_>>>()?;
        execution::checkpoint(0).map_err(|e| Diagnostic::resource(e, span))?;
        for (name, value) in prepared {
            self.variables.insert(name, value);
        }
        self.workspace_bytes = bytes;
        Ok(())
    }
    fn run(
        &mut self,
        source: &str,
        options: &ExecutionOptions,
        execution: &mut Stream<'_>,
    ) -> Result<()> {
        execution::checkpoint(0)
            .map_err(|e| Diagnostic::resource(e, Span { line: 1, column: 1 }))?;
        execution::checkpoint(source.len() as u64)
            .and_then(|()| execution::allocation(source.len() as u64))
            .map_err(|e| Diagnostic::resource(e, Span { line: 1, column: 1 }))?;
        let statements = parser::parse(source)?;
        for statement in statements {
            execution::checkpoint(1).map_err(|e| Diagnostic::resource(e, statement.expr.span))?;
            if let Some(args) = &statement.indices {
                let name = statement
                    .name
                    .as_ref()
                    .expect("indexed destination has name");
                let Some(Value::Numeric(original)) = self.variables.get(name) else {
                    return Err(Diagnostic::new(
                        "escrita requer array existente",
                        statement.expr.span,
                    ));
                };
                let mut array = original.clone();
                let (indices, shape) =
                    self.index_selection(&array, args, execution, statement.expr.span)?;
                let rhs = self.numeric(&statement.expr, execution)?;
                if args.len() > 1
                    && !rhs.is_scalar()
                    && rhs.shape() != shape
                    && !(rhs.is_vector() && shape.iter().filter(|&&n| n != 1).count() <= 1)
                {
                    return Err(Diagnostic::new(
                        "forma da atribuição incompatível",
                        statement.expr.span,
                    ));
                }
                num(array.scatter(&indices, &rhs), statement.expr.span)?;
                self.bind(
                    &[(name.clone(), Value::Numeric(array.clone()))],
                    options,
                    statement.expr.span,
                )?;
                execution.emit(
                    ExecutionEventKind::WorkspaceChanged {
                        name: name.clone(),
                        value: Value::Numeric(array.clone()),
                    },
                    false,
                )?;
                if !statement.silent {
                    execution.output(Output {
                        name: Some(name.clone()),
                        value: Value::Numeric(array),
                    })?;
                }
                continue;
            }
            if let Some(targets) = &statement.targets {
                let values = self.requested_outputs(&statement.expr, targets.len(), execution)?;
                let bindings = targets
                    .iter()
                    .zip(&values)
                    .filter_map(|(n, v)| {
                        n.as_ref()
                            .map(|name| Ok((name.clone(), v.try_clone(statement.expr.span)?)))
                    })
                    .collect::<Result<Vec<_>>>()?;
                self.bind(&bindings, options, statement.expr.span)?;
                for (name, value) in &bindings {
                    execution.emit(
                        ExecutionEventKind::WorkspaceChanged {
                            name: name.clone(),
                            value: value.try_clone(statement.expr.span)?,
                        },
                        false,
                    )?;
                }
                for (target, value) in targets.iter().zip(values) {
                    if let Some(name) = target
                        && !statement.silent
                    {
                        execution.output(Output {
                            name: Some(name.clone()),
                            value,
                        })?;
                    }
                }
                continue;
            }
            // Workspace commands require mutable Session access. An existing
            // variable still shadows this builtin, like other function names.
            let clear_workspace = statement.name.is_none()
                && !self.variables.contains_key("clear")
                && match &statement.expr.kind {
                    Kind::Variable(name) => name == "clear",
                    Kind::Apply(name, arguments) => name == "clear" && arguments.is_empty(),
                    _ => false,
                };
            if clear_workspace {
                self.clear();
                execution.emit(ExecutionEventKind::WorkspaceCleared, false)?;
                continue;
            }
            // Only a direct statement without a requested output draws. Nested
            // step calls and assignments use the numerical built-in as before.
            if statement.name.is_none() && self.plot_statement(&statement.expr, execution)? {
                continue;
            }
            let value = self.eval(&statement.expr, execution)?;
            if let Some(value) = value {
                let name = statement.name.unwrap_or_else(|| "ans".into());
                self.bind(
                    &[(name.clone(), value.try_clone(statement.expr.span)?)],
                    options,
                    statement.expr.span,
                )?;
                execution.emit(
                    ExecutionEventKind::WorkspaceChanged {
                        name: name.clone(),
                        value: value.try_clone(statement.expr.span)?,
                    },
                    false,
                )?;
                if !statement.silent {
                    execution.output(Output {
                        name: Some(name),
                        value,
                    })?;
                }
            } else if statement.name.is_some() {
                return Err(Diagnostic::new(
                    "função não retorna valor",
                    statement.expr.span,
                ));
            }
        }
        execution::checkpoint(0)
            .map_err(|e| Diagnostic::resource(e, Span { line: 1, column: 1 }))?;
        Ok(())
    }
    fn required(&self, expr: &Expr, outputs: &mut Stream<'_>) -> Result<Value> {
        self.eval(expr, outputs)?
            .ok_or_else(|| Diagnostic::new("esperado valor de expressão", expr.span))
    }
    fn requested_outputs(
        &self,
        expr: &Expr,
        nargout: usize,
        outputs: &mut Stream<'_>,
    ) -> Result<Vec<Value>> {
        if nargout == 1 {
            return Ok(vec![self.required(expr, outputs)?]);
        }
        let Kind::Apply(name, args) = &expr.kind else {
            return Err(Diagnostic::new(
                "múltiplas saídas requerem chamada de função",
                expr.span,
            ));
        };
        let context = CallContext {
            nargin: args.len(),
            nargout,
        };
        if self.variables.contains_key(name) {
            return Err(Diagnostic::new(
                "indexação numérica não fornece múltiplas saídas",
                expr.span,
            ));
        }
        let valid = match name.as_str() {
            "size" => context.nargin == 1,
            "step" => (1..=2).contains(&context.nargin) && context.nargout == 2,
            _ => false,
        };
        if !valid {
            return Err(Diagnostic::new(
                format!("{name}: combinação de entradas/saídas ainda não suportada"),
                expr.span,
            ));
        }
        let values: Vec<_> = args
            .iter()
            .map(|e| self.required(e, outputs))
            .collect::<Result<_>>()?;
        if name == "size" {
            let Value::Numeric(matrix) = &values[0] else {
                return Err(Diagnostic::new("size requer matriz numérica", expr.span));
            };
            Ok((0..context.nargout)
                .map(|i| {
                    Value::Numeric(Matrix::scalar(if i + 1 == context.nargout {
                        matrix.shape().iter().skip(i).product::<usize>() as f64
                    } else {
                        matrix.dimension(i) as f64
                    }))
                })
                .collect())
        } else {
            let (times, data) = control_step(&values, expr.span)?;
            Ok(vec![
                Value::Numeric(num(Matrix::new(data.len(), 1, data), expr.span)?),
                Value::Numeric(num(Matrix::new(times.len(), 1, times), expr.span)?),
            ])
        }
    }
    fn numeric(&self, expr: &Expr, outputs: &mut Stream<'_>) -> Result<Matrix> {
        match self.required(expr, outputs)? {
            Value::Numeric(m) => Ok(m),
            _ => Err(Diagnostic::new("esperada matriz numérica", expr.span)),
        }
    }
    fn eval(&self, expr: &Expr, outputs: &mut Stream<'_>) -> Result<Option<Value>> {
        let span = expr.span;
        execution::checkpoint(1).map_err(|e| Diagnostic::resource(e, span))?;
        execution::allocation(128).map_err(|e| Diagnostic::resource(e, span))?;
        let value = match &expr.kind {
            Kind::Number(n) => Value::Numeric(Matrix::scalar(*n)),
            Kind::Imaginary(n) => Value::Numeric(num(
                Matrix::from_complex(vec![1, 1], vec![Complex::new(0., *n)], true),
                span,
            )?),
            Kind::End => return Err(Diagnostic::new("end requer contexto de indexação", span)),
            Kind::Not(e) => {
                let m = self.numeric(e, outputs)?;
                m.require_real().map_err(|e| Diagnostic::new(e.0, span))?;
                Value::Numeric(num(
                    num(m.map(|v| if v == 0. { 1. } else { 0. }), span)?.logical(),
                    span,
                )?)
            }
            Kind::Text(s) => {
                execution::allocation(s.len() as u64).map_err(|e| Diagnostic::resource(e, span))?;
                Value::Text(s.clone())
            }
            Kind::Variable(name) => {
                if let Some(value) = self.variables.get(name) {
                    value.try_clone(span)?
                } else if matches!(name.as_str(), "i" | "j") {
                    Value::Numeric(num(
                        Matrix::from_complex(vec![1, 1], vec![Complex::new(0., 1.)], true),
                        span,
                    )?)
                } else if matches!(name.as_str(), "true" | "false") {
                    Value::Numeric(Matrix::logical_scalar(name == "true"))
                } else {
                    Value::Numeric(Matrix::scalar(match name.as_str() {
                        "pi" => std::f64::consts::PI,
                        "eps" => f64::EPSILON,
                        "Inf" | "inf" => f64::INFINITY,
                        "NaN" | "nan" => f64::NAN,
                        _ => {
                            return Err(Diagnostic::new(
                                format!("variável desconhecida: {name}"),
                                span,
                            ));
                        }
                    }))
                }
            }
            Kind::All => {
                return Err(Diagnostic::new(
                    "':' isolado é válido apenas em índices",
                    span,
                ));
            }
            Kind::Unary(negative, e) => match self.required(e, outputs)? {
                Value::Numeric(m) => Value::Numeric(if *negative {
                    num(m.map_complex(|v| -v), span)?
                } else {
                    num(m.map_complex(|v| v), span)?
                }),
                Value::TransferFunction(model) => Value::TransferFunction(if *negative {
                    control(model.scale(-1.), span)?
                } else {
                    model
                }),
                _ => return Err(Diagnostic::new("sinal requer matriz numérica ou tf", span)),
            },
            Kind::Transpose(conjugate, e) => {
                let m = self.numeric(e, outputs)?;
                num(m.require_matrix(), span)?;
                Value::Numeric(if *conjugate {
                    num(num(m.transpose(), span)?.conjugate(), span)?
                } else {
                    num(m.transpose(), span)?
                })
            }
            Kind::Binary(op, left, right) => {
                let left = self.required(left, outputs)?;
                let right = self.required(right, outputs)?;
                if matches!(left, Value::TransferFunction(_))
                    || matches!(right, Value::TransferFunction(_))
                {
                    return Ok(Some(Value::TransferFunction(control_binary(
                        *op, &left, &right, span,
                    )?)));
                }
                let (Value::Numeric(a), Value::Numeric(b)) = (left, right) else {
                    return Err(Diagnostic::new("esperada matriz numérica", span));
                };
                let result = match op {
                    Operator::Add => a.zip_complex(&b, |x, y| x + y),
                    Operator::Sub => a.zip_complex(&b, |x, y| x - y),
                    Operator::Mul => a.multiply(&b),
                    Operator::Div => {
                        if b.is_scalar() {
                            a.zip_complex(&b, |x, y| x / y)
                        } else {
                            num(b.transpose(), span)?
                                .solve(&num(a.transpose(), span)?)
                                .and_then(|m| m.transpose())
                        }
                    }
                    Operator::LeftDiv => a.solve(&b),
                    Operator::DotMul => a.zip_complex(&b, |x, y| x * y),
                    Operator::DotDiv => a.zip_complex(&b, |x, y| x / y),
                    Operator::DotLeftDiv => a.zip_complex(&b, |x, y| y / x),
                    Operator::Power => b.scalar_value().and_then(|p| a.power(p)),
                    Operator::DotPower => a.element_power(&b),
                    Operator::Eq
                    | Operator::Ne
                    | Operator::Lt
                    | Operator::Le
                    | Operator::Gt
                    | Operator::Ge
                    | Operator::And
                    | Operator::Or => {
                        if matches!(op, Operator::And | Operator::Or) {
                            num(a.require_real(), span)?;
                            num(b.require_real(), span)?;
                        }
                        let values = num(
                            a.zip_complex(&b, |x, y| {
                                Complex::new(
                                    if match op {
                                        Operator::Eq => x == y,
                                        Operator::Ne => x != y,
                                        Operator::Lt => x.re < y.re,
                                        Operator::Le => x.re <= y.re,
                                        Operator::Gt => x.re > y.re,
                                        Operator::Ge => x.re >= y.re,
                                        Operator::And => {
                                            (x.re != 0. || x.im != 0.) && (y.re != 0. || y.im != 0.)
                                        }
                                        Operator::Or => {
                                            (x.re != 0. || x.im != 0.) || (y.re != 0. || y.im != 0.)
                                        }
                                        _ => unreachable!(),
                                    } {
                                        1.
                                    } else {
                                        0.
                                    },
                                    0.,
                                )
                            }),
                            span,
                        )?;
                        values.logical()
                    }
                };
                Value::Numeric(num(result, span)?)
            }
            Kind::Range(start, step, end) => {
                let start = num(self.numeric(start, outputs)?.scalar_value(), span)?;
                let step = if let Some(step) = step {
                    num(self.numeric(step, outputs)?.scalar_value(), span)?
                } else {
                    1.0
                };
                let end = num(self.numeric(end, outputs)?.scalar_value(), span)?;
                Value::Numeric(num(Matrix::range(start, step, end), span)?)
            }
            Kind::Matrix(rows) => Value::Numeric(self.matrix(rows, outputs, span)?),
            Kind::Apply(name, args) => {
                if let Some(value) = self.variables.get(name) {
                    let Value::Numeric(m) = value else {
                        return Err(Diagnostic::new(
                            "indexação disponível somente para matrizes numéricas",
                            span,
                        ));
                    };
                    Value::Numeric(self.index(m, args, outputs, span)?)
                } else {
                    return self.builtin(name, args, outputs, span);
                }
            }
        };
        Ok(Some(value))
    }
    fn matrix(&self, rows: &[Vec<Expr>], outputs: &mut Stream<'_>, span: Span) -> Result<Matrix> {
        if rows.is_empty() {
            return num(Matrix::new(0, 0, vec![]), span);
        }
        if rows.len() == 1 && rows[0].len() == 1 {
            return self.numeric(&rows[0][0], outputs);
        }
        let mut lines = Vec::with_capacity(rows.len());
        for row in rows {
            let blocks = row
                .iter()
                .map(|e| self.numeric(e, outputs))
                .collect::<Result<Vec<_>>>()?;
            lines.push(num(Matrix::concatenate(1, &blocks), span)?);
        }
        num(Matrix::concatenate(0, &lines), span)
    }
    fn resolve_end(&self, expr: &Expr, limit: usize) -> Expr {
        let mut out = expr.clone();
        out.kind = match &expr.kind {
            Kind::End => Kind::Number(limit as f64),
            Kind::Unary(n, e) => Kind::Unary(*n, Box::new(self.resolve_end(e, limit))),
            Kind::Not(e) => Kind::Not(Box::new(self.resolve_end(e, limit))),
            Kind::Transpose(c, e) => Kind::Transpose(*c, Box::new(self.resolve_end(e, limit))),
            Kind::Binary(op, a, b) => Kind::Binary(
                *op,
                Box::new(self.resolve_end(a, limit)),
                Box::new(self.resolve_end(b, limit)),
            ),
            Kind::Range(a, b, c) => Kind::Range(
                Box::new(self.resolve_end(a, limit)),
                b.as_ref().map(|e| Box::new(self.resolve_end(e, limit))),
                Box::new(self.resolve_end(c, limit)),
            ),
            Kind::Matrix(rows) => Kind::Matrix(
                rows.iter()
                    .map(|r| r.iter().map(|e| self.resolve_end(e, limit)).collect())
                    .collect(),
            ),
            Kind::Apply(n, args) if !self.variables.contains_key(n) => Kind::Apply(
                n.clone(),
                args.iter().map(|e| self.resolve_end(e, limit)).collect(),
            ),
            kind => kind.clone(),
        };
        out
    }
    fn selector(
        &self,
        e: &Expr,
        limit: usize,
        outputs: &mut Stream<'_>,
    ) -> Result<(Vec<usize>, Vec<usize>, bool)> {
        if matches!(e.kind, Kind::All) {
            if limit > MAX_ELEMENTS {
                return Err(Diagnostic::new(
                    "seleção excede a política de alocação",
                    e.span,
                ));
            }
            num(
                execution::payload(limit, 8).map_err(NumericError::from),
                e.span,
            )?;
            return Ok(((0..limit).collect(), vec![limit, 1], false));
        }
        let m = self.numeric(&self.resolve_end(e, limit), outputs)?;
        num(m.require_real(), e.span)?;
        let logical = m.dtype() == DataType::Logical;
        num(
            execution::payload(m.data().len(), 8).map_err(NumericError::from),
            e.span,
        )?;
        let mut indices = Vec::new();
        for (i, &v) in m.data().iter().enumerate() {
            if i % 256 == 0 {
                num(
                    execution::checkpoint((m.data().len() - i).min(256) as u64)
                        .map_err(NumericError::from),
                    e.span,
                )?;
            }
            if logical {
                if v != 0. {
                    if i >= limit {
                        return Err(Diagnostic::new(
                            "máscara lógica seleciona fora dos limites",
                            e.span,
                        ));
                    }
                    indices.push(i);
                }
            } else {
                if !v.is_finite() || v < 1. || v.fract() != 0. || v > limit as f64 {
                    return Err(Diagnostic::new(
                        "índice deve ser inteiro positivo dentro dos limites",
                        e.span,
                    ));
                }
                indices.push(v as usize - 1);
            }
        }
        Ok((indices, m.shape().to_vec(), logical))
    }
    fn index_selection(
        &self,
        m: &Matrix,
        args: &[Expr],
        outputs: &mut Stream<'_>,
        span: Span,
    ) -> Result<(Vec<usize>, Vec<usize>)> {
        if args.is_empty() || args.len() > 32 {
            return Err(Diagnostic::new("indexação requer 1..32 índices", span));
        }
        if args.len() == 1 {
            let (indices, mut shape, logical) = self.selector(&args[0], m.data().len(), outputs)?;
            let index_vector = shape.len() == 2 && (shape[0] == 1 || shape[1] == 1);
            if logical {
                if indices.is_empty() && shape == [1, 1] {
                    shape = vec![0, 0];
                } else if m.is_vector() {
                    shape = if m.rows() == 1 {
                        vec![1, indices.len()]
                    } else {
                        vec![indices.len(), 1]
                    };
                } else if index_vector {
                    shape = if shape[0] == 1 {
                        vec![1, indices.len()]
                    } else {
                        vec![indices.len(), 1]
                    };
                } else {
                    shape = vec![indices.len(), 1];
                }
            } else if !matches!(args[0].kind, Kind::All) && m.is_vector() && index_vector {
                shape = if m.rows() == 1 {
                    vec![1, indices.len()]
                } else {
                    vec![indices.len(), 1]
                };
            }
            return Ok((indices, shape));
        }
        let mut selectors = Vec::new();
        let mut shape = Vec::new();
        let mut limits = Vec::new();
        for (axis, e) in args.iter().enumerate() {
            let limit = if axis + 1 == args.len() {
                m.shape().iter().skip(axis).product()
            } else {
                m.dimension(axis)
            };
            let (indices, _, _) = self.selector(e, limit, outputs)?;
            shape.push(indices.len());
            selectors.push(indices);
            limits.push(limit);
        }
        let count = num(Matrix::shape_len(&shape), span)?;
        num(
            execution::payload(count, 8).map_err(NumericError::from),
            span,
        )?;
        let mut indices = Vec::with_capacity(count);
        for i in 0..count {
            if i % 256 == 0 {
                num(
                    execution::checkpoint(((count - i).min(256) * args.len()) as u64)
                        .map_err(NumericError::from),
                    span,
                )?;
            }
            let (mut rest, mut offset, mut stride) = (i, 0, 1);
            for axis in 0..args.len() {
                let position = rest % shape[axis];
                rest /= shape[axis];
                offset += selectors[axis][position] * stride;
                stride *= limits[axis];
            }
            indices.push(offset);
        }
        while shape.len() > 2 && shape.last() == Some(&1) {
            shape.pop();
        }
        Ok((indices, shape))
    }
    fn index(
        &self,
        m: &Matrix,
        args: &[Expr],
        outputs: &mut Stream<'_>,
        span: Span,
    ) -> Result<Matrix> {
        let (indices, shape) = self.index_selection(m, args, outputs, span)?;
        num(m.gather(&indices, shape), span)
    }
    fn builtin(
        &self,
        name: &str,
        args: &[Expr],
        outputs: &mut Stream<'_>,
        span: Span,
    ) -> Result<Option<Value>> {
        // Validate function identity and arity before evaluating argument effects.
        let (min, max) = match name {
            "disp" | "sin" | "cos" | "tan" | "sqrt" | "abs" | "exp" | "log" | "numel"
            | "length" | "det" | "dcgain" => (1, 1),
            "sum" | "mean" | "size" | "eye" => (1, 2),
            "zeros" | "ones" => (1, 32),
            "true" | "false" => (0, 32),
            "complex" => (1, 2),
            "real" | "imag" | "conj" | "logical" | "double" | "ndims" | "isreal" | "islogical"
            | "class" => (1, 1),
            "linspace" => (2, 3),
            "reshape" => (2, 33),
            "tf" => (1, 2),
            "pid" => (1, 3),
            "feedback" => (2, 3),
            "step" => (1, 2),
            "plot" => (2, 2),
            _ => {
                return Err(Diagnostic::new(
                    format!("função desconhecida ou não implementada: {name}"),
                    span,
                ));
            }
        };
        if args.len() < min || args.len() > max {
            return Err(Diagnostic::new(
                format!("{name}: número de argumentos inválido"),
                span,
            ));
        }
        if name == "plot" {
            return Err(Diagnostic::new(
                "plot: chamadas com saída/handles ainda não suportadas; use plot(x,y) como comando",
                span,
            ));
        }
        let values: Vec<Value> = args
            .iter()
            .map(|e| self.required(e, outputs))
            .collect::<Result<_>>()?;
        if name == "disp" {
            outputs.output(Output {
                name: None,
                value: values[0].try_clone(span)?,
            })?;
            return Ok(None);
        }
        if matches!(name, "tf" | "pid" | "feedback" | "step" | "dcgain") {
            return control_builtin(name, &values, span).map(Some);
        }
        let matrices: Vec<&Matrix> = values
            .iter()
            .map(|v| match v {
                Value::Numeric(m) => Ok(m),
                _ => Err(Diagnostic::new("função requer argumentos numéricos", span)),
            })
            .collect::<Result<_>>()?;
        let default = Matrix::scalar(1.);
        let a = matrices.first().copied().unwrap_or(&default);
        let scalar = |i: usize| num(matrices[i].scalar_value(), span);
        let dim = |i: usize| dimension(scalar(i)?, span);
        let result = match name {
            "det" => Matrix::scalar(num(a.determinant(), span)?),
            "zeros" | "ones" | "true" | "false" | "eye" => {
                let shape = construction_shape(&matrices, span)?;
                if name == "eye" {
                    if shape.len() != 2 {
                        return Err(Diagnostic::new("eye requer forma 2D", span));
                    }
                    num(Matrix::identity(shape[0], shape[1]), span)?
                } else {
                    let array = num(
                        Matrix::filled_nd(
                            shape,
                            if matches!(name, "ones" | "true") {
                                1.
                            } else {
                                0.
                            },
                        ),
                        span,
                    )?;
                    if matches!(name, "true" | "false") {
                        num(array.logical(), span)?
                    } else {
                        array
                    }
                }
            }
            "class" => return Ok(Some(Value::Text(a.class().into()))),
            "logical" => num(a.logical(), span)?,
            "double" => a.to_double(),
            "real" => a.real(),
            "imag" => num(a.imag(), span)?,
            "conj" => {
                let out = num(a.conjugate(), span)?;
                if a.dtype() == DataType::Logical {
                    out.real()
                } else {
                    out
                }
            }
            "ndims" => Matrix::scalar(a.shape().len() as f64),
            "isreal" => Matrix::logical_scalar(!a.is_complex()),
            "islogical" => Matrix::logical_scalar(a.dtype() == DataType::Logical),
            "complex" => {
                num(a.require_real(), span)?;
                if matrices.len() == 1 {
                    num(
                        Matrix::from_complex(
                            a.shape().to_vec(),
                            num(
                                execution::collect(a.data().len(), |i| {
                                    Complex::new(a.data()[i], 0.)
                                })
                                .map_err(NumericError::from),
                                span,
                            )?,
                            true,
                        ),
                        span,
                    )?
                } else {
                    let b = matrices[1];
                    num(b.require_real(), span)?;
                    if a.shape() != b.shape() && !a.is_scalar() && !b.is_scalar() {
                        return Err(Diagnostic::new(
                            "complex requer formas iguais ou escalares",
                            span,
                        ));
                    }
                    let values = num(a.zip_complex(b, |r, i| Complex::new(r.re, i.re)), span)?;
                    num(
                        Matrix::from_complex(
                            values.shape().to_vec(),
                            num(
                                execution::collect(values.data().len(), |i| values.element(i))
                                    .map_err(NumericError::from),
                                span,
                            )?,
                            true,
                        ),
                        span,
                    )?
                }
            }
            "size" => {
                if matrices.len() == 2 {
                    let d = dim(1)?;
                    if d == 0 {
                        return Err(Diagnostic::new("dimensão deve ser positiva", span));
                    }
                    Matrix::scalar(a.dimension(d - 1) as f64)
                } else {
                    num(
                        Matrix::new(
                            1,
                            a.shape().len(),
                            a.shape().iter().map(|&n| n as f64).collect(),
                        ),
                        span,
                    )?
                }
            }
            "numel" => Matrix::scalar(a.data().len() as f64),
            "length" => Matrix::scalar(if a.data().is_empty() {
                0.
            } else {
                *a.shape().iter().max().unwrap() as f64
            }),
            "sum" | "mean" => {
                let dimension = if matrices.len() == 2 {
                    Some(dim(1)?)
                } else {
                    None
                };
                let sum = num(a.sum(dimension), span)?;
                if name == "sum" {
                    sum
                } else {
                    let axis = dimension.map_or_else(|| a.first_nonsingleton(), |d| d - 1);
                    let n = a.dimension(axis) as f64;
                    num(sum.map_complex(|x| x / Complex::new(n, 0.)), span)?
                }
            }
            "reshape" => {
                let dims = &matrices[1..];
                let mut shape = Vec::new();
                let mut infer = None;
                if dims.len() == 1 {
                    num(dims[0].require_real(), span)?;
                    if !dims[0].is_vector() || dims[0].data().len() < 2 {
                        return Err(Diagnostic::new(
                            "reshape requer vetor com pelo menos duas dimensões",
                            span,
                        ));
                    }
                    shape = dims[0]
                        .data()
                        .iter()
                        .map(|&v| dimension(v, span))
                        .collect::<Result<_>>()?;
                } else {
                    for (axis, m) in dims.iter().enumerate() {
                        if m.data().is_empty() {
                            if infer.is_some() {
                                return Err(Diagnostic::new(
                                    "reshape aceita apenas uma dimensão inferida",
                                    span,
                                ));
                            }
                            infer = Some(axis);
                            shape.push(1);
                        } else {
                            shape.push(dimension(num(m.scalar_value(), span)?, span)?);
                        }
                    }
                }
                if let Some(axis) = infer {
                    let known = num(Matrix::shape_len(&shape), span)?;
                    if known == 0 || a.data().len() % known != 0 {
                        return Err(Diagnostic::new("dimensão inferida incompatível", span));
                    }
                    shape[axis] = a.data().len() / known;
                }
                num(a.reshape(shape), span)?
            }
            "linspace" => {
                let start = scalar(0)?;
                let end = scalar(1)?;
                let n = if matrices.len() == 3 { dim(2)? } else { 100 };
                if !start.is_finite() || !end.is_finite() {
                    return Err(Diagnostic::new("linspace requer extremos finitos", span));
                }
                if n > MAX_ELEMENTS {
                    return Err(Diagnostic::new("linspace excede limite", span));
                }
                let data = num(
                    execution::collect(n, |i| {
                        if n == 1 || i == n - 1 {
                            end
                        } else {
                            let t = i as f64 / (n - 1) as f64;
                            (1. - t) * start + t * end
                        }
                    })
                    .map_err(NumericError::from),
                    span,
                )?;
                num(Matrix::new(1, n, data), span)?
            }
            "sqrt" => num(a.map_complex(Complex::sqrt), span)?,
            "log" => num(a.map_complex(Complex::log), span)?,
            "sin" => num(a.map_complex(Complex::sin), span)?,
            "cos" => num(a.map_complex(Complex::cos), span)?,
            "tan" => num(a.map_complex(|v| v.sin() / v.cos()), span)?,
            "abs" => num(a.project(Complex::abs), span)?,
            "exp" => num(a.map_complex(Complex::exp), span)?,
            _ => unreachable!("built-in already validated"),
        };
        Ok(Some(Value::Numeric(result)))
    }

    fn plot_statement(&self, expr: &Expr, execution: &mut Stream<'_>) -> Result<bool> {
        let Kind::Apply(name, args) = &expr.kind else {
            return Ok(false);
        };
        if self.variables.contains_key(name) || !matches!(name.as_str(), "plot" | "step") {
            return Ok(false);
        }
        let span = expr.span;
        let valid_arity = if name == "plot" {
            args.len() == 2
        } else {
            (1..=2).contains(&args.len())
        };
        if !valid_arity {
            return Err(Diagnostic::new(
                format!("{name}: número de argumentos inválido"),
                span,
            ));
        }
        if execution.execution.plots.len() >= MAX_EVENTS {
            return Err(Diagnostic::new(
                "limite de 16 eventos gráficos por execução",
                span,
            ));
        }
        let values: Vec<_> = args
            .iter()
            .map(|e| self.required(e, execution))
            .collect::<Result<_>>()?;
        let figure = if name == "step" {
            let (times, data) = control_step(&values, span)?;
            num(
                execution::payload(times.len(), 16).map_err(NumericError::from),
                span,
            )?;
            let series = PlotSeries::line(expression_label(&args[0], "Resposta"), &times, &data)
                .map_err(|e| Diagnostic::new(e.0, span))?;
            Figure::line("Resposta ao degrau", "Tempo (s)", "Amplitude", series)
        } else {
            let x = plot_vector(&values[0], span)?;
            let y = plot_vector(&values[1], span)?;
            num(
                execution::payload(x.len(), 16).map_err(NumericError::from),
                span,
            )?;
            let series = PlotSeries::line(expression_label(&args[1], "Curva 1"), x, y)
                .map_err(|e| Diagnostic::new(e.0, span))?;
            Figure::line(
                "Gráfico de linhas",
                expression_label(&args[0], "x"),
                expression_label(&args[1], "y"),
                series,
            )
        };
        let prior_points: usize = execution
            .execution
            .plots
            .iter()
            .map(|event| event.figure().point_count())
            .sum();
        if prior_points + figure.point_count() > MAX_EXECUTION_POINTS {
            return Err(Diagnostic::new(
                "limite de 200000 pontos gráficos por execução",
                span,
            ));
        }
        execution.plot(PlotEvent::Show(figure))?;
        Ok(true)
    }
}
fn expression_label<'a>(expr: &'a Expr, fallback: &'a str) -> &'a str {
    match &expr.kind {
        Kind::Variable(name) => name,
        _ => fallback,
    }
}
fn construction_shape(matrices: &[&Matrix], span: Span) -> Result<Vec<usize>> {
    if matrices.is_empty() {
        return Ok(vec![1, 1]);
    }
    if matrices.len() == 1 {
        let a = matrices[0];
        num(a.require_real(), span)?;
        if a.is_scalar() {
            let n = dimension(num(a.scalar_value(), span)?, span)?;
            return Ok(vec![n, n]);
        }
        if !a.is_vector() || a.data().len() < 2 {
            return Err(Diagnostic::new(
                "construtor requer dimensões escalares ou vetor de dimensões",
                span,
            ));
        }
        return a.data().iter().map(|&v| dimension(v, span)).collect();
    }
    matrices
        .iter()
        .map(|m| dimension(num(m.scalar_value(), span)?, span))
        .collect()
}
fn plot_vector(value: &Value, span: Span) -> Result<&[f64]> {
    let Value::Numeric(matrix) = value else {
        return Err(Diagnostic::new("plot: esperado vetor numérico real", span));
    };
    num(matrix.require_real(), span)?;
    if !matrix.data().is_empty() && !matrix.is_vector() {
        return Err(Diagnostic::new(
            "plot: esta etapa aceita somente vetores",
            span,
        ));
    }
    Ok(matrix.data())
}
fn control<T>(result: std::result::Result<T, ControlError>, span: Span) -> Result<T> {
    result.map_err(|e| match e.1 {
        Some(resource) => Diagnostic::resource(resource, span),
        None => Diagnostic::new(e.0, span),
    })
}
fn value_scalar(value: &Value, span: Span) -> Result<f64> {
    let Value::Numeric(matrix) = value else {
        return Err(Diagnostic::new("esperado escalar numérico", span));
    };
    num(matrix.scalar_value(), span)
}
fn value_vector(value: &Value, span: Span) -> Result<&[f64]> {
    let Value::Numeric(matrix) = value else {
        return Err(Diagnostic::new("esperado vetor numérico", span));
    };
    num(matrix.require_real(), span)?;
    if matrix.data().is_empty() || !matrix.is_vector() {
        return Err(Diagnostic::new("esperado vetor numérico não vazio", span));
    }
    Ok(matrix.data())
}
fn value_model(value: &Value, span: Span) -> Result<TransferFunction> {
    match value {
        Value::TransferFunction(model) => Ok(model.clone()),
        Value::Numeric(_) => control(TransferFunction::scalar(value_scalar(value, span)?), span),
        _ => Err(Diagnostic::new("esperado tf ou ganho escalar", span)),
    }
}
fn control_binary(op: Operator, a: &Value, b: &Value, span: Span) -> Result<TransferFunction> {
    let a = value_model(a, span)?;
    if matches!(op, Operator::Power) {
        return control(a.power(value_scalar(b, span)?), span);
    }
    let b = value_model(b, span)?;
    control(
        match op {
            Operator::Add => a.add(&b),
            Operator::Sub => a.subtract(&b),
            Operator::Mul => a.multiply(&b),
            Operator::Div => a.divide(&b),
            _ => return Err(Diagnostic::new("tf: operador ainda não suportado", span)),
        },
        span,
    )
}
fn control_builtin(name: &str, values: &[Value], span: Span) -> Result<Value> {
    let model = match name {
        "tf" => match values {
            [Value::Text(s)] if s == "s" => TransferFunction::laplace(),
            [Value::TransferFunction(model)] => model.clone(),
            [numerator, denominator] => control(
                TransferFunction::new(
                    value_vector(numerator, span)?,
                    value_vector(denominator, span)?,
                ),
                span,
            )?,
            _ => {
                return Err(Diagnostic::new(
                    "tf: use tf('s') ou tf(numerador, denominador)",
                    span,
                ));
            }
        },
        "pid" => {
            let kp = value_scalar(&values[0], span)?;
            let ki = if values.len() > 1 {
                value_scalar(&values[1], span)?
            } else {
                0.
            };
            let kd = if values.len() > 2 {
                value_scalar(&values[2], span)?
            } else {
                0.
            };
            control(TransferFunction::pid(kp, ki, kd), span)?
        }
        "feedback" => control(
            value_model(&values[0], span)?.feedback(
                &value_model(&values[1], span)?,
                if values.len() > 2 {
                    value_scalar(&values[2], span)?
                } else {
                    -1.
                },
            ),
            span,
        )?,
        "dcgain" => {
            let Value::TransferFunction(model) = &values[0] else {
                return Err(Diagnostic::new("função requer modelo tf", span));
            };
            return Ok(Value::Numeric(Matrix::scalar(model.dc_gain())));
        }
        "step" => {
            let (_, data) = control_step(values, span)?;
            return Ok(Value::Numeric(num(Matrix::new(data.len(), 1, data), span)?));
        }
        _ => unreachable!("control built-in already validated"),
    };
    Ok(Value::TransferFunction(model))
}
fn control_step(values: &[Value], span: Span) -> Result<(Vec<f64>, Vec<f64>)> {
    let Value::TransferFunction(model) = &values[0] else {
        return Err(Diagnostic::new("step: função requer modelo tf", span));
    };
    let times = if values.len() == 1 {
        control(model.step_times(), span)?
    } else {
        let data = value_vector(&values[1], span)?;
        num(
            execution::payload(data.len(), 8).map_err(NumericError::from),
            span,
        )?;
        data.to_vec()
    };
    let data = control(model.step(&times), span)?;
    Ok((times, data))
}
fn num<T>(result: std::result::Result<T, NumericError>, span: Span) -> Result<T> {
    result.map_err(|e| match e.1 {
        Some(resource) => Diagnostic::resource(resource, span),
        None => Diagnostic::new(e.0, span),
    })
}
fn dimension(value: f64, span: Span) -> Result<usize> {
    if !value.is_finite() || value < 0. || value.fract() != 0. || value > MAX_ELEMENTS as f64 {
        return Err(Diagnostic::new(
            "dimensão requer inteiro não negativo dentro do limite",
            span,
        ));
    }
    Ok(value as usize)
}
