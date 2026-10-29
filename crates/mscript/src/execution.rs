use crate::graphics::PlotEvent;
use crate::numeric::execution::{self, CancellationToken, ResourceLimits, ResourceUsage};
use crate::{Diagnostic, Execution, Output, Span, Value};

#[derive(Clone, Debug)]
pub struct ExecutionOptions {
    pub id: u64,
    pub resources: ResourceLimits,
    pub cancellation: CancellationToken,
    pub max_events: u64,
    pub max_workspace_bytes: u64,
    pub max_variables: usize,
}
impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            id: 0,
            resources: ResourceLimits::default(),
            cancellation: CancellationToken::default(),
            max_events: 10_000,
            max_workspace_bytes: 128 * 1024 * 1024,
            max_variables: 4096,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    Succeeded,
    Failed,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionEvent {
    pub id: u64,
    pub sequence: u64,
    pub kind: ExecutionEventKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionEventKind {
    Started,
    Output(Output),
    WorkspaceChanged {
        name: String,
        value: Value,
    },
    WorkspaceCleared,
    Plot(PlotEvent),
    Warning {
        identifier: String,
        message: String,
    },
    Error(Diagnostic),
    Finished {
        status: ExecutionStatus,
        usage: ResourceUsage,
    },
}
#[derive(Clone, Debug)]
pub struct ExecutionOutcome {
    pub execution: Execution,
    pub diagnostic: Option<Diagnostic>,
    pub usage: ResourceUsage,
    pub status: ExecutionStatus,
}
pub(crate) struct Stream<'a> {
    pub execution: Execution,
    options: &'a ExecutionOptions,
    sequence: u64,
    count: u64,
    callback: &'a mut dyn FnMut(&ExecutionEvent) -> Result<(), Diagnostic>,
}
impl<'a> Stream<'a> {
    pub fn new(
        options: &'a ExecutionOptions,
        callback: &'a mut dyn FnMut(&ExecutionEvent) -> Result<(), Diagnostic>,
    ) -> Self {
        Self {
            execution: Execution::default(),
            options,
            sequence: 0,
            count: 0,
            callback,
        }
    }
    pub fn emit(&mut self, kind: ExecutionEventKind, lifecycle: bool) -> Result<(), Diagnostic> {
        if !lifecycle {
            execution::checkpoint(0)
                .map_err(|e| Diagnostic::resource(e, Span { line: 1, column: 1 }))?;
            if self.count >= self.options.max_events {
                return Err(Diagnostic::limit(
                    "orçamento de eventos da execução excedido",
                    Span { line: 1, column: 1 },
                ));
            }
            self.count += 1;
        }
        let event = ExecutionEvent {
            id: self.options.id,
            sequence: self.sequence,
            kind,
        };
        self.sequence = self.sequence.checked_add(1).ok_or_else(|| {
            Diagnostic::limit("sequência de eventos excedida", Span { line: 1, column: 1 })
        })?;
        (self.callback)(&event)
    }
    pub fn output(&mut self, output: Output) -> Result<(), Diagnostic> {
        let event = Output {
            name: output.name.clone(),
            value: output.value.try_clone(Span { line: 1, column: 1 })?,
        };
        self.emit(ExecutionEventKind::Output(event), false)?;
        self.execution.outputs.push(output);
        Ok(())
    }
    pub fn plot(&mut self, event: PlotEvent) -> Result<(), Diagnostic> {
        self.emit(ExecutionEventKind::Plot(event.clone()), false)?;
        self.execution.plots.push(event);
        Ok(())
    }
}
