//! A small, explicitly documented MATLAB-language interpreter.
mod ast;
mod lexer;
mod parser;
mod runtime;
mod execution;

pub use openmat_control::TransferFunction;
pub use openmat_numeric::{Array, Complex, DataType, Matrix};
pub use runtime::{CallContext, Execution, Output, Session, Value};
pub use execution::{ExecutionOptions,ExecutionEvent,ExecutionEventKind,ExecutionOutcome,ExecutionStatus};
pub use openmat_numeric::execution::{CancellationToken,ResourceLimits,ResourceUsage,ResourceFailure,checkpoint as resource_checkpoint};
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

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DiagnosticCategory{Syntax,Semantic,Limit,Cancelled,Transport,External}

impl Diagnostic {
    pub(crate) fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            category:DiagnosticCategory::Semantic,
            identifier:"openmat:Semantic",
        }
    }
    pub(crate) fn syntax(message:impl Into<String>,span:Span)->Self{Self{category:DiagnosticCategory::Syntax,identifier:"openmat:Syntax",..Self::new(message,span)}}
    pub(crate) fn limit(message:impl Into<String>,span:Span)->Self{Self{category:DiagnosticCategory::Limit,identifier:"openmat:Limit",..Self::new(message,span)}}
    pub fn resource(e:ResourceFailure,span:Span)->Self{
        let (category,identifier)=match e{ResourceFailure::Cancelled=>(DiagnosticCategory::Cancelled,"openmat:Cancelled"),ResourceFailure::Deadline=>(DiagnosticCategory::Limit,"openmat:Deadline"),ResourceFailure::WorkLimit=>(DiagnosticCategory::Limit,"openmat:WorkLimit"),ResourceFailure::AllocationLimit=>(DiagnosticCategory::Limit,"openmat:AllocationLimit")};
        Self{message:e.to_string(),span,category,identifier}
    }
    pub fn transport(message:impl Into<String>)->Self{Self{category:DiagnosticCategory::Transport,identifier:"openmat:Transport",..Self::new(message,Span{line:1,column:1})}}
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
