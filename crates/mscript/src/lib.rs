//! ControLAB's interpreter for `.m` scripts: a small, explicitly documented
//! subset of the MATLAB language with numeric, control and plotting built-ins.
mod ast;
pub mod control;
pub mod documents;
mod execution;
pub mod graphics;
mod lexer;
pub mod numeric;
mod parser;
mod runtime;

pub use crate::control::TransferFunction;
pub use crate::numeric::execution::{
    CancellationToken, ResourceFailure, ResourceLimits, ResourceUsage,
    checkpoint as resource_checkpoint,
};
pub use crate::numeric::{Array, Complex, DataType, Matrix};
pub use execution::{
    ExecutionEvent, ExecutionEventKind, ExecutionOptions, ExecutionOutcome, ExecutionStatus,
};
pub use runtime::{CallContext, Execution, Output, Session, Value};
use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub category: DiagnosticCategory,
    pub identifier: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticCategory {
    Syntax,
    Semantic,
    Limit,
    Cancelled,
    Transport,
    External,
}

impl Diagnostic {
    pub(crate) fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            category: DiagnosticCategory::Semantic,
            identifier: "mscript:Semantic",
        }
    }
    pub(crate) fn syntax(message: impl Into<String>, span: Span) -> Self {
        Self {
            category: DiagnosticCategory::Syntax,
            identifier: "mscript:Syntax",
            ..Self::new(message, span)
        }
    }
    pub(crate) fn limit(message: impl Into<String>, span: Span) -> Self {
        Self {
            category: DiagnosticCategory::Limit,
            identifier: "mscript:Limit",
            ..Self::new(message, span)
        }
    }
    pub fn resource(e: ResourceFailure, span: Span) -> Self {
        let (category, identifier) = match e {
            ResourceFailure::Cancelled => (DiagnosticCategory::Cancelled, "mscript:Cancelled"),
            ResourceFailure::Deadline => (DiagnosticCategory::Limit, "mscript:Deadline"),
            ResourceFailure::WorkLimit => (DiagnosticCategory::Limit, "mscript:WorkLimit"),
            ResourceFailure::AllocationLimit => {
                (DiagnosticCategory::Limit, "mscript:AllocationLimit")
            }
        };
        Self {
            message: e.to_string(),
            span,
            category,
            identifier,
        }
    }
    pub fn transport(message: impl Into<String>) -> Self {
        Self {
            category: DiagnosticCategory::Transport,
            identifier: "mscript:Transport",
            ..Self::new(message, Span { line: 1, column: 1 })
        }
    }
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}
impl std::error::Error for Diagnostic {}
pub(crate) type Result<T> = std::result::Result<T, Diagnostic>;
